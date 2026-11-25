# YAML로 앱 등록하고 자동 증감 사용하기

`app.yaml`에 도메인, 번들 URL, 실행 설정과 증감 정책을 적으면 controller가 PostgreSQL에 저장하고 runner를 선택합니다. 앱은 표준 소켓으로 HTTP/gRPC 서버를 여는 WASIp2 component이며 전용 SDK가 필요하지 않습니다.

일반 요청은 **Traefik → WASM 소켓**으로 갑니다. Traefik의 WAFL 미들웨어가 진행 중인 요청을 세고 주기적으로 controller에 보고합니다. 실행이 0개이면 **Traefik → activator → 새 WASM 소켓**으로 첫 요청을 전달합니다. 첫 요청은 응답 본문·gRPC trailers가 끝날 때까지 activator를 통과하고, 이후 요청은 새 직결 경로를 이용합니다.

## Docker Compose로 시작

저장소 루트에서 실행합니다. Docker Compose, Rust 1.100 이상과 `wasm32-wasip2` target이 필요합니다. 이미지는 Linux amd64용입니다. controller와 runner는 같은 `ghcr.io/lesomnus/wafl` 이미지를 사용하고 실행 명령으로 역할을 정합니다. `ghcr.io/lesomnus/wafl-traefik`에는 Traefik 3.8.0과 WAFL 미들웨어가 들어 있습니다.

```sh
rustup target add wasm32-wasip2
bash scripts/bundle.sh "$PWD/examples/compose/bundles/hello.tar.gz"
docker compose -f examples/compose/compose.yaml up -d
docker compose -f examples/compose/compose.yaml logs -f controller gateway
```

PostgreSQL, runner 2개, controller, Traefik, 번들 HTTP 서버가 실행됩니다. 예제 DB 비밀번호와 고정 Docker subnet은 운영 환경에 맞게 바꾸세요. 관리 API는 localhost:8082, 앱 진입점은 localhost:8080입니다. gateway는 Docker 내부 네트워크로 WASM 포트에 접근합니다. endpoint 예약은 runner별 volume, 캐시는 공유 volume에 보관됩니다.

CLI를 설치했다면 다음과 같이 등록합니다.

```sh
export WAFL_ENDPOINT=http://127.0.0.1:8082
wafl app add examples/compose/app.yaml
wafl --json app get @hello
wafl placement list --app @hello
curl --max-time 125 -H 'Host: hello.localhost' http://127.0.0.1:8080/
curl -H 'Host: hello.localhost' http://127.0.0.1:8080/index.html
```

CLI 설치 없이도 이미지 안의 CLI로 등록할 수 있습니다. 두 방법 중 하나를 사용합니다. `Add`는 새 리소스를 만들며 같은 alias/domain으로 반복하면 충돌합니다.

```sh
docker compose -f examples/compose/compose.yaml cp examples/compose/app.yaml controller:/tmp/app.yaml
docker compose -f examples/compose/compose.yaml exec controller \
  wafl --endpoint http://127.0.0.1:8082 app add /tmp/app.yaml
```

0개 상태에서 첫 요청을 보내면 다운로드·검증·컴파일 후 응답합니다. 요청이 끊기고 유휴 시간과 축소 대기 시간이 지나면 Instance를 종료·삭제합니다. 번들은 캐시에 남아 재기동에 사용됩니다. 캐시 공간이 필요하면 [LRU GC 정책](store.md)이 적용됩니다.

배포 버전을 고정하려면 두 이미지를 같은 `sha-<main 커밋 전체 SHA>` 태그로 선택합니다.

```sh
export WAFL_IMAGE=ghcr.io/lesomnus/wafl:sha-<commit>
export WAFL_GATEWAY_IMAGE=ghcr.io/lesomnus/wafl-traefik:sha-<commit>
docker compose -f examples/compose/compose.yaml up -d
```

## app.yaml 계약

```yaml
alias: hello
name: Hello WASM
domain: hello.localhost
manifest:
  runtime: wasmtime
  abi: wasip2
  bundle:
    url: https://assets.example.com/hello.tar.gz
    # digest: sha256:<압축 파일의 SHA-256>
  port: 8080
  protocol: http
  memory_bytes: 67108864
  env:
    GREETING: Hello
scaling:
  min: 0
  max: 4
  target: 8
  up_ms: 1000
  down_ms: 10000
  idle_ms: 30000
  waiting: 64
  wait_ms: 120000
```

YAML과 JSON을 받습니다. 최상위 필드는 `alias`, `name`, `domain`, `tenant`, `manifest`, `scaling`, `replicas`입니다. 파일은 1 MiB 이하이며 알 수 없는 필드는 거절합니다. `tenant` 생략 시 로컬 tenant를 사용하고, 지정하면 domain-1 UUID 문자열이어야 합니다. `manifest.port` 생략 시 CLI가 8080을 사용합니다. 환경변수 값은 문자열이므로 숫자·불리언처럼 보이는 값은 따옴표로 감쌉니다. 변수 치환이나 외부 파일 include는 하지 않습니다. `snake_case`와 protobuf JSON의 `camelCase`를 모두 받습니다.

명시적인 CLI 플래그는 파일 값을 덮어쓰며 `--manifest`는 Manifest 전체를 교체합니다. 기존 URL 입력도 지원합니다.

```sh
wafl app add app.yaml --alias staging --domain staging.example.com --env MODE=staging
wafl app add https://assets.example.com/hello.tar.gz --domain hello.example.com --replicas 2
```

번들은 tar.gz이며 루트의 `main.wasm`과 선택적인 `data/`가 들어갑니다. `data/`는 앱에서 읽기 전용 `/data`로 보입니다. HTTP·HTTPS·file URL을 지원하며 **URL을 읽는 주체는 선택된 runner**입니다. YAML 옆의 파일을 자동 업로드하지 않습니다. `file:`은 runner 파일시스템 경로입니다. 여러 호스트에서는 모든 runner가 접근할 HTTP/HTTPS URL을 사용하세요. 첫 성공 실행의 압축 파일 digest를 App에 고정하므로 복제본도 같은 번들을 사용합니다. ORAS와 registry credential은 아직 지원하지 않습니다.

## 증감 정책

| 필드 | 의미 | 생략/0 기본값 |
|---|---|---|
| `min` | 최소 실행 수. 0이면 첫 요청으로 기동 | 0 |
| `max` | 앱별 최대 목표 실행 수 | controller 한도 내 8 |
| `target` | 인스턴스당 목표 동시 요청 수 | 16 |
| `up_ms` | 높은 부하가 유지되어야 하는 시간 | 1000 |
| `down_ms` | 축소 조건이 유지되어야 하는 시간 | 30000 |
| `idle_ms` | 0개로 내리기 전 유휴 시간 | 60000 |
| `waiting` | 앱별 activator 요청 수 한도 | 64 |
| `wait_ms` | 기동 대기 시간 한도 | 120000 |

현재 진행 중인 요청 수와 보고 구간 평균 동시 요청 수 중 큰 값을 사용합니다. `ceil(부하 / target)`을 min/max로 제한하고 대기 시간이 충족되면 한 번에 하나씩 늘리거나 줄입니다. 0개에서 대기 요청이 생기면 목표를 즉시 1로 올립니다. 앱마다 동시에 하나의 신규 Placement만 준비하고 전체 준비 작업은 `max_pending`으로 제한합니다. 이미 앱을 실행 중인 runner를 우선 사용하며 메모리·실행 개수 한도가 부족하면 다른 runner를 선택합니다.

`scaling`이 있으면 `replicas`는 0 또는 생략해야 합니다. `scaling`을 생략하면 고정 replica 방식이며 기본값은 1입니다. 고정 방식의 `replicas: 0`은 자동 기동을 하지 않습니다. 정책 변경에는 Scaling 메시지만 담은 파일을 사용합니다.

```sh
wafl app patch @hello --scaling examples/compose/scaling-load.yaml
wafl --json app get @hello
wafl traffic list --app @hello
wafl placement list --app @hello
```

`App.state`의 `desired`, `active`, `waiting`, `load`, `fresh`와 관측 시각으로 상태를 확인합니다. `Placement.error`는 다운로드·기동·회수 실패를 보여줍니다. 증감 기준은 요청/스트림 동시성입니다. runner의 OTLP CPU·메모리 지표는 별도로 사용할 수 있습니다.

## 시나리오

**간헐적인 도구:** `min: 0`을 사용합니다. `idle_ms`를 길게 잡으면 재기동 횟수가 줄어듭니다. `wait_ms`와 proxy·클라이언트 timeout은 다운로드/컴파일 시간을 포함하도록 맞춥니다. activator 수용 한도를 넘으면 429, 기동 대기 시간이 지나면 HTTP 504를 반환합니다. gRPC deadline은 기동 대기를 포함하며 만료 시 status 4를 반환합니다. 요청 본문을 통째로 저장하거나 upstream 오류 후 재전송하지 않습니다.

**지속적인 API:** `min: 1`과 적절한 `target`, `max`를 사용합니다. 다음 예제는 요청마다 100 ms를 기다리는 HTTP 앱으로 확장을 관찰합니다. 지연은 예제 앱의 `RESPONSE_DELAY_MS` 환경변수이며 WAFL 예약 변수가 아닙니다.

```sh
wafl app add examples/compose/app-load.yaml
seq 200 | xargs -P 32 -I '{}' curl --silent --show-error --max-time 60 \
  -H 'Host: load.localhost' http://127.0.0.1:8080/
wafl --json app get @load
wafl placement list --app @load
```

**gRPC 스트리밍:** `manifest.protocol: h2c`를 지정합니다. TLS는 Traefik에서 종료하고 WASM에는 HTTP/2 prior knowledge로 연결합니다. 한 스트림은 종료/취소까지 활성 요청 하나입니다. 열린 스트림이 있으면 0개로 축소하지 않습니다. 배포 변경으로 Placement가 퇴역해도 기존 스트림이 끝날 때까지 실행을 유지합니다. 기존 gateway의 요청 본문 읽기·응답 쓰기 timeout도 스트림 길이에 맞게 설정합니다. Compose 예제는 `entryPoints.web.transport.respondingTimeouts`의 `readTimeout`과 `writeTimeout`을 `0s`로 두고 클라이언트 gRPC deadline을 사용합니다.

**외부 DB:** 앱이 사용하는 이름으로 connection string을 `manifest.env`에 넣습니다. 일반 DB 클라이언트가 외부 TCP/DNS로 연결합니다. DB 생성·credential 공급은 운영 환경에서 담당하며 WAFL은 특정 환경변수 이름을 강제하지 않습니다. [DB 예제](network.md)를 참고하세요.

## 기존 Traefik과 Kubernetes

기존 Traefik 3.8.0에도 `plugins/traefik`을 작업 디렉터리의 `plugins-local/src/github.com/lesomnus/wafl/traefik`으로 마운트할 수 있습니다. 다음은 재시작이 필요한 정적 설정입니다. 앱별 동적 설정은 controller가 생성합니다.

```yaml
experimental:
  localPlugins:
    wafl:
      moduleName: github.com/lesomnus/wafl/traefik
providers:
  http:
    endpoint: http://controller:8082/traefik
    pollInterval: 500ms
```

controller의 `traffic.control`은 관측 RPC 주소, `traffic.activate`는 첫 요청 전달 주소, `traffic.listen`은 별도 activator listener입니다. [설정 예제](../examples/compose/controller.json)를 참고하세요. 관리 listener와 분리되어 앱 경로로 관리 RPC가 노출되지 않습니다.

Kubernetes에서는 gateway, controller, runner를 각각 Pod로 실행합니다. `direct.advertise`는 **gateway에서 접근 가능한 해당 runner Pod의 IPv4**이고 `runners`에는 각 runner의 관리 주소를 넣습니다. 여러 runner를 하나의 load-balancing Service로 숨기지 않습니다. Pod IP/runner 자동 발견은 아직 없으므로 배포 시 설정을 제공해야 합니다. 예약 디렉터리는 재시작에도 유지하고 동일 IP·포트 풀을 쓰는 프로세스끼리 공유합니다. [직결 운영 계약](direct.md)을 참고하세요.

controller replicas는 같은 PostgreSQL과 공유 설정을 사용합니다. `traffic.control`과 `traffic.activate`에는 replicas를 가리키는 내부 Service 주소를 사용합니다. 각 Pod의 listen은 다르게 둘 수 있습니다. 관측 등록·배치·회수 판단은 DB 트랜잭션으로 직렬화합니다. activator 장애로 중계 요청이 실패하면 자동 재전송하지 않습니다.

## 안전한 축소와 복구

Placement를 Retiring으로 바꾸고 새 경로에서 제거하며 runner에 `Seal → Drain`을 보냅니다. 관측자는 수신 gate를 영구적으로 닫고 기존 요청이 0개가 되었음을 보고합니다. 오래된 경로에 들어온 새 요청은 WASM에 전달하기 전에 activator로 보냅니다. 모든 관측자가 확인하면 `Release → Erase`로 실행과 포트를 회수합니다. `Seal`은 같은 session에서 지연 도착한 Add가 회수된 ID를 재생성하는 것을 막습니다.

보고가 끊겼다는 이유만으로 포트를 회수하지 않습니다. 오래된 관측이 있으면 자동 축소를 멈추고 Retiring 실행을 유지합니다. gateway/activator 장애 후에는 **해당 프로세스와 그 연결이 모두 종료되었음을 확인한 뒤** 관측자를 닫습니다.

```sh
wafl --json traffic list --app @hello
wafl traffic close <Traffic-UUID> --withdrawn
```

실행 중인 관측자를 강제로 닫으면 안전한 회수를 보장할 수 없습니다. 단순한 heartbeat timeout은 확인 근거가 아닙니다. runner 장애 후 남은 포트는 [예약 복구 절차](reservations.md)를 따릅니다. 고정 replica 앱은 기존 `placement release --withdrawn` 절차를 사용합니다.

runner는 session당 `capacity.tombstones`(기본 65536)개의 Seal ID를 기억합니다. 한도에 가까워지면 신규 자동 배치를 멈춥니다. 실행을 정상 회수한 뒤 runner를 재시작하여 session을 교체할 수 있습니다. Placement/Traffic 이력은 App 삭제 시 함께 지우며 자동 보관 기간 정리는 아직 없습니다.

```sh
wafl app patch @hello --fixed --replicas 0
wafl placement list --app @hello
# 모든 placement가 Released이고 관측자의 active가 0인지 확인 후
wafl app erase @hello
```

고정 방식으로 전환해도 기존 자동 관리 Placement는 gate 확인 후 회수합니다. Manifest 변경과 자동/고정 방식 전환은 App revision을 증가시킵니다.

공유 설정 변경/이전 버전 업그레이드 시 기존 controller replicas를 모두 종료합니다. 한 개를 `wafl serve controller --config controller.json --configure`로 시작해 DB의 설정 fingerprint를 교체하고, 동일 설정으로 나머지를 시작합니다. 관리 API와 관측 RPC에는 인증이 없으므로 내부 관리 네트워크에서 접근시킵니다.
