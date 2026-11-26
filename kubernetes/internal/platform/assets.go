package platform

import (
	"crypto/sha256"
	"encoding/hex"
	"errors"
	"io"
	"net/http"
	"os"
	"path/filepath"
	"regexp"
	"strings"

	wf "github.com/lesomnus/wafl/kubernetes/gen/wafl"
	"google.golang.org/protobuf/encoding/protojson"
)

var bundleHash = regexp.MustCompile(`^[0-9a-f]{64}$`)

// AssetHandler is the optional shared development artifact store. Completed
// content is immutable and named by its compressed SHA-256. Partial uploads are
// private temporary files, and publication is an atomic rename on one volume.
func AssetHandler(directory string) (http.Handler, error) {
	if e := os.MkdirAll(directory, 0755); e != nil {
		return nil, e
	}
	mux := http.NewServeMux()
	mux.HandleFunc("GET /{name}", func(w http.ResponseWriter, r *http.Request) {
		name := r.PathValue("name")
		digest := strings.TrimSuffix(name, ".tar.gz")
		if !bundleHash.MatchString(digest) || name != digest+".tar.gz" {
			http.NotFound(w, r)
			return
		}
		w.Header().Set("Cache-Control", "public, max-age=31536000, immutable")
		http.ServeFile(w, r, filepath.Join(directory, name))
	})
	mux.HandleFunc("PUT /{name}", func(w http.ResponseWriter, r *http.Request) {
		name := r.PathValue("name")
		digest := strings.TrimSuffix(name, ".tar.gz")
		if !bundleHash.MatchString(digest) || name != digest+".tar.gz" {
			http.Error(w, "invalid bundle digest", 400)
			return
		}
		file, e := os.CreateTemp(directory, ".upload-")
		if e != nil {
			http.Error(w, "artifact storage unavailable", 503)
			return
		}
		defer os.Remove(file.Name())
		defer file.Close()
		h := sha256.New()
		reader := http.MaxBytesReader(w, r.Body, 1<<30)
		_, e = io.Copy(io.MultiWriter(file, h), reader)
		if e != nil {
			var tooLarge *http.MaxBytesError
			if errors.As(e, &tooLarge) {
				http.Error(w, "bundle exceeds 1 GiB", 413)
			} else {
				http.Error(w, "incomplete upload", 400)
			}
			return
		}
		if hex.EncodeToString(h.Sum(nil)) != digest {
			http.Error(w, "bundle digest mismatch", 400)
			return
		}
		if e = file.Chmod(0644); e == nil {
			e = file.Sync()
		}
		if e == nil {
			e = file.Close()
		}
		if e == nil {
			e = os.Rename(file.Name(), filepath.Join(directory, name))
		}
		if e != nil {
			http.Error(w, "artifact storage unavailable", 503)
			return
		}
		w.WriteHeader(http.StatusCreated)
	})
	return mux, nil
}

func (c *Controller) upload(w http.ResponseWriter, r *http.Request) {
	digest := r.PathValue("digest")
	if !bundleHash.MatchString(digest) {
		http.Error(w, "invalid bundle digest", 400)
		return
	}
	if c.Assets == "" {
		http.Error(w, "bundle uploads are not configured; publish an HTTP(S) URL", http.StatusNotImplemented)
		return
	}
	target := strings.TrimRight(c.Assets, "/") + "/" + digest + ".tar.gz"
	// The destination is configured by the operator, never supplied by a client.
	request, e := http.NewRequestWithContext(r.Context(), http.MethodPut, target, http.MaxBytesReader(w, r.Body, 1<<30))
	if e != nil {
		http.Error(w, "artifact storage unavailable", 503)
		return
	}
	response, e := http.DefaultClient.Do(request)
	if e != nil {
		http.Error(w, "bundle upload failed", 502)
		return
	}
	defer response.Body.Close()
	if response.StatusCode != http.StatusCreated {
		code := http.StatusBadGateway
		if response.StatusCode == 400 || response.StatusCode == 413 {
			code = response.StatusCode
		}
		http.Error(w, "bundle upload rejected", code)
		return
	}
	data, _ := protojson.Marshal(&wf.Asset{Kind: &wf.Asset_Url{Url: target}, Digest: "sha256:" + digest})
	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(http.StatusCreated)
	_, _ = w.Write(data)
}
