package platform

import (
	"encoding/json"
	"errors"
	"regexp"

	wf "github.com/lesomnus/wafl/kubernetes/gen/wafl"
	core "k8s.io/api/core/v1"
)

const Endpoints = Group + "/endpoints"
const TCPFirst = 10000
const TCPLast = 10015

var socketName = regexp.MustCompile(`^[a-z]([a-z0-9-]{0,13}[a-z0-9])?$`)

// Normalize both wire representations to the same first-socket compatibility
// fields. Empty application protocol denotes an opaque TCP stream.
func sockets(m *wf.Manifest) ([]*wf.Socket, error) {
	if len(m.Sockets) == 0 {
		if m.Port == 0 {
			m.Port = 8080
		}
		if m.Protocol == "" {
			m.Protocol = "http"
		}
		if m.Port > 65535 || (m.Protocol != "http" && m.Protocol != "h2c" && m.Protocol != "tcp") {
			return nil, errors.New("invalid legacy port/protocol")
		}
		protocol := m.Protocol
		if protocol == "tcp" {
			protocol = ""
		}
		return []*wf.Socket{{Name: "app", Port: m.Port, Transport: "tcp", Protocol: protocol}}, nil
	}
	if len(m.Sockets) > 16 {
		return nil, errors.New("at most 16 sockets are supported")
	}
	names, ports := map[string]bool{}, map[uint32]bool{}
	for _, s := range m.Sockets {
		if s == nil {
			return nil, errors.New("socket is required")
		}
		if s.Transport == "" {
			s.Transport = "tcp"
		}
		if !socketName.MatchString(s.Name) || names[s.Name] || ports[s.Port] || s.Port == 0 || s.Port > 65535 || s.Transport != "tcp" || (s.Protocol != "" && s.Protocol != "http" && s.Protocol != "h2c") {
			return nil, errors.New("sockets require unique names/ports, TCP transport and empty/http/h2c protocol")
		}
		names[s.Name], ports[s.Port] = true, true
	}
	first := m.Sockets[0]
	protocol := first.Protocol
	if protocol == "" {
		protocol = "tcp"
	}
	if m.Port != 0 && m.Port != first.Port || m.Protocol != "" && m.Protocol != protocol {
		return nil, errors.New("legacy port/protocol must match the first socket")
	}
	m.Port, m.Protocol = first.Port, protocol
	return m.Sockets, nil
}

func validateListeners(a *wf.App) error {
	ss, err := sockets(a.Manifest)
	if err != nil {
		return err
	}
	if a.Manifest.Protocol == "tcp" {
		if a.Domain != "" || len(a.Listeners) == 0 {
			return errors.New("TCP Apps require listeners and omit domain")
		}
		if a.Readiness != nil {
			return errors.New("TCP Apps use socket readiness; HTTP/gRPC probes require an HTTP App")
		}
	} else if a.Domain == "" || len(a.Listeners) != 0 {
		return errors.New("HTTP Apps require domain and omit TCP listeners")
	}
	names, ports := map[string]bool{}, map[uint32]bool{}
	for _, listener := range a.Listeners {
		found := false
		for _, s := range ss {
			if s.Name == listener.GetSocket() && s.Protocol == "" {
				found = true
			}
		}
		if !found || names[listener.Socket] || ports[listener.Port] || listener.Port < TCPFirst || listener.Port > TCPLast {
			return errors.New("listeners require unique raw socket names and public ports 10000..10015")
		}
		names[listener.Socket], ports[listener.Port] = true, true
	}
	if a.Manifest.Protocol == "tcp" && len(names) != len(ss) {
		return errors.New("each raw TCP socket requires a listener")
	}
	if a.Manifest.Protocol != "tcp" && len(ss) != 1 {
		return errors.New("HTTP Apps currently publish one socket; multiple sockets are supported by TCP Apps")
	}
	return nil
}

func endpointMap(i *wf.Instance) string {
	addresses := map[string]string{}
	for _, e := range i.Endpoints {
		addresses[e.Name] = e.Address
	}
	if len(addresses) == 0 && i.GetEndpoint().GetAddress() != "" {
		addresses["app"] = i.GetEndpoint().GetAddress()
	}
	b, _ := json.Marshal(addresses)
	return string(b)
}
func socketAddress(p *core.Pod, name string) string {
	addresses := map[string]string{}
	_ = json.Unmarshal([]byte(p.Annotations[Endpoints]), &addresses)
	if len(addresses) == 0 && name == "app" {
		return p.Annotations[Endpoint]
	}
	return addresses[name]
}

// Same stable winner rule for reconciliation, HTTP config and TCP admission.
// Invalid declarations cannot reserve a listener or a domain.
func claimRoutes(a *wf.App, domains map[string]bool, ports map[uint32]bool) bool {
	if a.Domain != "" && domains[a.Domain] {
		return false
	}
	for _, l := range a.Listeners {
		if ports[l.Port] {
			return false
		}
	}
	if a.Domain != "" {
		domains[a.Domain] = true
	}
	for _, l := range a.Listeners {
		ports[l.Port] = true
	}
	return true
}
