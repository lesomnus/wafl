# Pingora 프록시

`wafl serve proxy`는 Pingora 0.9.1을 Rust 라이브러리로 사용하는 WAFL 데이터 평면입니다. controller는 App 배치와 기동을 담당하고, 프록시는 HTTP/gRPC 요청과 일반 TCP 연결을 받아 runner의 WASM 소켓에 직접 연결합니다. 첫 요청도 이 경로를 사용합니다.

```text
클라이언트 → wafl proxy → runner Pod IP:배정 포트 → WASM
                 ↕
             controller API
```

## 실행

Kubernetes의 `wafl-kube bootstrap install` 또는 `bootstrap dev`는 프록시까지 설치합니다. 모든 역할은 같은 `ghcr.io/lesomnus/wafl` 이미지를 사용합니다. 별도 Traefik 이미지나 플러그인은 필요하지 않습니다.

```sh
wafl serve proxy --controller http://127.0.0.1:8082 \
  --listen 0.0.0.0:8080 --health 127.0.0.1:8084
```

controller API와 runner의 배정 소켓에 접근할 수 있어야 합니다. 프록시는 Kubernetes API나 kubeconfig를 사용하지 않습니다. 기본 TCP 공개 포트는 10000–10015이며 `--tcp-ip`로 바인딩 주소를 고릅니다. HTTP/gRPC만 필요하면 `--http-only`를 지정합니다.

- `--threads`: HTTP/TCP 서비스 각각의 worker 수, 기본 2.
- `--capacity`: 대기 중이거나 처리 중인 HTTP 요청·TCP 연결 합계 상한, 기본 1024.
- `--drain-seconds`: 정상 종료 때 기존 스트림을 기다리는 시간, 기본 30초.
- `--tls-listen`, `--tls-cert`, `--tls-key`: 선택적인 HTTPS/gRPC TLS 리스너. HTTP/2 ALPN을 지원합니다. 인증서 발급·갱신은 cert-manager 등 외부 도구로 관리하고 프록시를 rolling update합니다.

TLS를 종료하는 기존 ingress 뒤에 배치할 수도 있습니다. 일반 TCP의 라우팅 키는 수신 포트이며 payload를 HTTP나 TLS로 해석하지 않습니다. 앱의 서버 선행 응답과 TLS passthrough가 가능합니다. UDP는 아직 지원하지 않습니다.

## 기동과 전달

1. 프록시는 controller의 `GET /proxy`에서 `Routing` protobuf JSON 전체 snapshot을 250ms 간격으로 받습니다. App ID, tenant, hostname/port, 후보 placement와 endpoint만 포함합니다. 환경 변수·Secret·번들은 포함하지 않습니다.
2. HTTP는 Host/`:authority`, 일반 TCP는 수신 포트로 App을 고릅니다. 준비된 실행이 있으면 placement의 `TrafficService.Add` 등록을 확인하고 직접 전달합니다. 정상 경로의 각 요청마다 controller RPC를 호출하지 않습니다.
3. 실행이 없으면 App을 지정한 `TrafficService.Add`로 대기 요청을 기록하고 `AppService.Wake`를 호출합니다. Wake는 0→1 기동 요청을 중복 방지하며 고정 0개 App은 기동하지 않습니다. 요청 본문과 TCP 데이터는 프록시에 남습니다.
4. 준비된 endpoint가 snapshot에 나타나면 placement 관측을 등록한 뒤 대기 관측을 종료하고 전달합니다. 기다리는 동안 App ID가 바뀌면 새 App에 요청을 넘기지 않습니다.

`scaling.waiting`은 앱 전체 대기 상한, `scaling.waitMs`는 대기 시간입니다. 프록시 capacity도 별도로 적용됩니다. gRPC deadline과 HTTP/2 stream 취소는 준비 대기에 적용합니다. 일반 TCP의 FIN은 송신 반쪽 종료일 수 있으므로 요청 취소로 간주하지 않습니다. 대기 중 데이터를 모두 메모리에 읽어 두지 않으며, TCP 양방향 전달은 반닫힘을 보존합니다.

HTTP/1 keep-alive와 HTTP/2 연결 풀을 사용합니다. 풀이 다른 실행 세대로 넘어가지 않도록 관측 gate별로 분리합니다. HTTP/gRPC와 TCP 모두 전달을 시작한 데이터를 다른 실행에 재전송하지 않습니다.

## 종료와 장애

프록시는 실행별 활성 요청·연결 수, 시작 횟수, 누적 처리 시간을 약 500ms마다 보고합니다. 경로에서 빠진 실행은 신규 수신을 차단하고 기존 스트림이 끝난 후 `closed=true, active=0`을 보고합니다. controller/provider는 이 확인을 받은 뒤 예약을 회수합니다.

snapshot을 5초 이상 갱신하지 못하면 신규 요청·연결을 거절합니다. 이미 전달 중인 스트림은 계속 처리합니다. readiness 주소도 이때 503을 반환합니다. 정상 종료는 신규 수신을 차단하고 기존 작업을 기다립니다. `--drain-seconds`가 지나면 남은 HTTP/gRPC 스트림과 TCP 연결을 종료하고 관측 종료를 보고할 시간을 둡니다. Pod의 `terminationGracePeriodSeconds`는 이 시간보다 5초 이상 길게 설정합니다. 강제 종료 또는 API 장애로 보고가 남으면 기존의 `wafl traffic close ID --withdrawn` 복구 계약을 사용합니다. heartbeat 만료를 포트 재사용 증거로 사용하지 않습니다.

프록시를 여러 replica로 배포할 수 있습니다. 각 replica는 자기 연결과 관측 gate를 소유하며 controller는 관측을 합산합니다. 열린 TCP 연결을 다른 프록시로 이동하지는 않습니다.

## Traefik 설치에서 전환

새 설치는 `wafl-proxy` Deployment/Service를 만듭니다. 기존 bootstrap 설치를 갱신할 때는 공개 주소와 immutable selector를 유지하기 위해 `wafl-traefik` 이름의 Deployment/Service를 유지하고 실행 이미지를 `wafl serve proxy`로 교체합니다. 이름이 남아도 실행되는 프록시는 Pingora입니다. controller와 proxy를 같은 버전으로 갱신하고 proxy readiness 및 앱 요청을 확인합니다. 서로 다른 버전이 공존하는 전환 구간의 cold-start 무중단은 보장하지 않습니다.

수동 설치는 새 [platform.yaml](../kubernetes/install/platform.yaml)을 기준으로 프록시 workload와 Service selector를 옮깁니다. 새 Service와 기존 Service가 같은 NodePort를 동시에 예약할 수 없으므로 기존 공개 Service를 유지해 새 프록시로 전환하는 방식을 권장합니다.

`--gateway-image`, controller의 `--control`, `--activate`, `--activation-listen`, `--entrypoint`와 `/traefik` 설정 endpoint는 제거했습니다. controller는 앱 데이터용 8083/TCP 포트를 열지 않습니다. 이전 벤치마크는 당시 Traefik 구성을 측정한 기록이며 Pingora 성능 수치가 아닙니다.
