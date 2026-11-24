# Runner 사용하기

## 로컬 앱을 그대로 빌드하기

`examples/hello/src/main.rs`는 `std::net::TcpListener`를 사용하는 작은 HTTP 서버입니다. 특정 웹 프레임워크의 WASI 호환성을 가정하지 않고, 먼저 표준 소켓부터 검증하는 예제입니다.

```sh
PORT=9000 GREETING=Local cargo run --locked -p wafl-example-hello
curl http://127.0.0.1:9000/
```

로컬 서버를 종료한 뒤 같은 소스를 WASM으로 빌드합니다.

```sh
cargo build --locked -p wafl-example-hello --target wasm32-wasip2
WAFL_TOKENS='{"local-dev":"00000000-0000-8000-8001-000000000001"}' \
  cargo run --locked -- examples/runner.json
```

Rust 1.100.0과 Wasmtime 51.0.0을 고정했습니다. `wasm32-wasip2` 빌드는 실행 가능한 component를 생성합니다. 일반 core WASM 파일, WASI Preview 1, WASIX, `wasi:http/incoming-handler` 전용 component는 이 runner의 앱 계약이 아닙니다. 다른 언어나 서버 라이브러리를 사용하려면 WASIp2 CLI와 sockets 지원을 확인해야 합니다. 모든 기존 HTTP/gRPC 프레임워크가 수정 없이 빌드된다는 의미는 아닙니다.

`CARGO_TARGET_DIR`을 변경한 경우 manifest의 `artifact` 경로도 해당 빌드 경로로 변경합니다. 상대 경로는 manifest 파일의 디렉터리를 기준으로 해석합니다.

## Manifest

```json
{
  "listen": "127.0.0.1:8080",
  "control": "127.0.0.1:8081",
  "apps": [{
    "id": "00000000-0000-8000-8007-000000000001",
    "tenant": "00000000-0000-8000-8001-000000000001",
    "name": "jobs",
    "artifact": "jobs.wasm",
    "runtime": "wasmtime",
    "abi": "wasip2",
    "domain": "jobs.apps.internal",
    "port": 8080,
    "protocol": "http",
    "idle_ms": 60000,
    "startup_ms": 10000,
    "memory_bytes": 67108864,
    "max_active": 128,
    "env": {"APP_MODE": "production"},
    "bindings": [{"port": 5432, "target": "10.0.0.20:5432"}]
  }]
}
```

| 필드 | 의미 |
| --- | --- |
| `listen` | gateway 또는 클라이언트가 접속하는 HTTP/1.1·HTTP/2 cleartext 주소 |
| `control` | paydirt 관리 API 주소. 앱 트래픽과 별도 listener |
| `id`, `tenant` | 각각 domain 7 Instance ID와 domain 1 Tenant ID. Payday UUIDv8 문자열이며 manifest에 명시 |
| `name`, `domain` | runner 안에서 각각 유일한 앱 이름과 소문자 hostname |
| `artifact` | WASIp2 CLI component 파일. 부팅 시 읽고 컴파일 |
| `port` | 앱의 IPv4 TCP listen 포트. runner가 `PORT` 환경 변수로도 전달 |
| `protocol` | upstream이 HTTP/1.1이면 `http`(기본), HTTP/2 prior knowledge면 `h2c` |
| `idle_ms` | 활성 요청이 0이 된 뒤 회수까지의 시간. 기본 60초, 검사 간격 100ms |
| `startup_ms` | WASM 인스턴스 생성부터 선언된 소켓의 listen 완료까지 제한. 기본 10초 |
| `memory_bytes` | 인스턴스의 단일 linear memory 상한. 기본 64 MiB |
| `max_active` | 기동 대기 중인 요청까지 포함한 앱별 동시 요청 상한. 기본 128 |
| `env` | 명시적으로 전달할 환경 변수. 호스트 환경은 상속하지 않음 |
| `bindings` | 게스트의 IPv4 loopback TCP 포트를 운영자가 지정한 실제 IPv4 주소에 매핑 |

앱은 `0.0.0.0:port` 또는 IPv4 loopback에 bind할 수 있습니다. 실제 소켓은 인스턴스마다 별도 host loopback 포트를 사용합니다. 소켓 가상화는 WASI 호출 수준에서 이루어지며 OS 네트워크 namespace를 만들지는 않습니다. 같은 runner Pod의 네이티브 프로세스는 신뢰하는 운영 구성요소여야 합니다.

파일시스템 preopen, 호스트 환경 상속, UDP, DNS, 선언되지 않은 outbound 연결은 허용하지 않습니다. WASI resource table은 인스턴스당 1,024개로 제한합니다. 하나의 epoch clock이 CPU를 계속 사용하는 게스트도 주기적으로 yield하도록 하여 시작 제한과 취소가 동작합니다. 이는 CPU 사용량 quota는 아닙니다. stdout은 버리고, stderr는 최대 16 KiB를 저장하여 게스트 실패 진단에 사용합니다. 별도 로그 조회·스트리밍 API는 아직 없습니다.

## 첫 요청부터 회수까지

1. runner는 부팅 시 manifest를 검증하고 component 코드를 컴파일합니다. 아직 앱의 `Store`나 서버는 실행하지 않습니다. 동일 경로의 아티팩트는 컴파일 결과를 공유합니다.
2. HTTP `Host` 또는 HTTP/2 `:authority`로 앱을 찾습니다. 알 수 없는 도메인은 404입니다.
3. 인스턴스가 없으면 한 번만 생성합니다. 동시 첫 요청은 같은 시작 결과를 기다립니다. 첫 호출자가 취소해도 다른 호출자의 시작은 계속됩니다.
4. 게스트가 선언된 포트의 WASI listen을 완료하면 내부 주소를 얻어 요청을 전달합니다. probe 요청을 보내서 앱 카운터를 증가시키지 않습니다.
5. 응답 body와 trailers가 끝나거나 요청이 취소될 때까지 인스턴스를 사용 중으로 표시합니다. body 전체를 메모리에 버퍼링하지 않습니다.
6. 활성 요청이 없고 유휴 시간이 지나면 WASM 실행을 취소하고 `Store`와 소켓을 해제합니다. 코드 캐시는 runner에 남습니다. 다음 요청은 새 메모리로 시작합니다.

시작 실패는 503, upstream 연결 실패는 502로 응답합니다. 전송을 시작한 요청은 자동 재시도하지 않습니다. 실패 후 약 1초의 재시도 간격을 둡니다. 상세 원인은 관리 API의 `error`에서 확인합니다. `startup_ms`는 runner 부팅 시 아티팩트를 컴파일하는 시간을 포함하지 않습니다.

선언된 listener를 닫으면 그 인스턴스도 종료합니다. 현재 요청별 upstream TCP 연결을 사용하며 연결 풀은 없습니다. WebSocket/HTTP Upgrade와 CONNECT는 501로 거절합니다.

## 관리 API

[`Instance`](../proto/wafl/instance.proto)는 `(orm.message)`와 `(payday.entity)`를 가진 일반 entity입니다. `id = 1`, `tenant = 2`, `name = 5`를 사용하며 실행 상태는 8–12, 16번에 있습니다. 도메인 7의 ID와 tenant 소속은 manifest에서 주어지고, `InstanceService.Get/List`, 참조와 선택·페이지 계약은 paydirt가 생성합니다. 앱 고유 동작인 `Start/Stop`은 [확장 proto](../proto/ext/wafl/instance_svc.ext.proto)에 추가합니다.

서버 구성은 `Unimplemented → Instances → Authentication`입니다. 요청은 마지막에 붙인 Authentication부터 들어오고, 검증한 tenant scope를 Instances에 전달합니다. Instances는 생성된 service layer를 구현해 메모리의 실행 상태를 조회·제어합니다. DB Sink를 조합하지 않으므로 DB 연결이나 마이그레이션을 실행하지 않습니다. entity 선언과 서버의 저장 구현은 독립적입니다.

`paydirt-server`는 Connect/gRPC transport를, `paydirt::spin::Workers`는 유휴 회수와 종료 수명을 관리합니다.

### 토큰과 tenant

`WAFL_TOKENS`는 관리용 bearer token을 Tenant ID에 연결하는 JSON 객체이며 loopback에서도 필수입니다. 예제의 `local-dev`는 로컬 실행을 위한 예시 값입니다. 운영에서는 비밀 저장소 등에서 토큰을 주입합니다.

```sh
export WAFL_TOKENS='{
  "tenant-a-token":"00000000-0000-8000-8001-000000000001",
  "tenant-b-token":"00000000-0000-8000-8001-000000000002"
}'
```

관리 요청은 인증된 tenant의 Instance만 조회·시작·종료할 수 있습니다. 다른 tenant의 ID를 지정하면 없는 ID와 동일한 `NOT_FOUND`입니다. 클라이언트가 보낸 tenant 헤더는 scope를 바꾸지 않습니다. `List`는 tenant로 먼저 좁힌 뒤 필터와 페이지를 적용하므로 다른 tenant의 Instance가 페이지 크기나 cursor에 영향을 주지 않습니다.

이 토큰은 runner 관리용이며 앱 사용자의 인증은 해당 앱이 담당합니다. 외부 노출 시 gateway/서비스 메시에서 TLS를 종료하고 control 포트 접근을 제한합니다.

### 호출과 페이지 조회

RPC는 `List`, `Get`, `Start`, `Stop`입니다. `id`는 앱 이름이 아닌 Payday UUIDv8의 16바이트 값이며, protobuf JSON에서는 base64로 표현합니다. 예제 hello의 `00000000-0000-8000-8007-000000000001`은 `AAAAAAAAgACABwAAAAAAAQ==`입니다. `List`에서 받은 ID를 `ref.id`에 넣으면 됩니다.

```sh
curl -H 'Authorization: Bearer local-dev' -H 'Content-Type: application/json' \
  -H 'Connect-Protocol-Version: 1' -d '{}' \
  http://127.0.0.1:8081/wafl.InstanceService/List

curl -H 'Authorization: Bearer local-dev' -H 'Content-Type: application/json' \
  -H 'Connect-Protocol-Version: 1' -d '{"ref":{"id":"AAAAAAAAgACABwAAAAAAAQ=="}}' \
  http://127.0.0.1:8081/wafl.InstanceService/Start

curl -H 'Authorization: Bearer local-dev' -H 'Content-Type: application/json' \
  -H 'Connect-Protocol-Version: 1' \
  -d '{"ref":{"id":"AAAAAAAAgACABwAAAAAAAQ=="},"select":{"name":true,"phase":true}}' \
  http://127.0.0.1:8081/wafl.InstanceService/Get
```

`Stop`도 같은 `ref`를 사용합니다. 이미 실행 중인 Instance의 `Start`는 재사용하고, 활성 요청이 있는 `Stop`은 `FAILED_PRECONDITION`으로 거절합니다. 중지해도 manifest의 ID와 tenant는 유지되며 다음 앱 요청에서 다시 기동합니다. `generation`은 runner 프로세스 내의 시작 시도 횟수입니다.

`Get`에서 `select`를 생략하거나 `all: true`로 지정하면 상태 전체를 반환합니다. 명시적 select는 선택한 필드와 ID만 반환합니다. tenant 관계는 ID만 보유합니다. `tenant: {}`는 이 참조를 선택하고, tenant 이름·alias 등 메타데이터 확장은 `UNIMPLEMENTED`입니다.

`List`는 ID순입니다. `filters: [{"ref":{"id":"..."}}]`로 선택하며 여러 필터는 OR로 결합합니다. 빈 필터 메시지와 64개를 초과하는 필터는 거절합니다. size가 0 이하이면 기본 100, 1,000보다 크면 1,000으로 제한합니다. `reverse: true`는 역순입니다. 응답의 `next`가 있으면 같은 필터와 방향으로 다음 요청의 `after`에 전달합니다. cursor를 다른 tenant나 방향에서 사용하면 `INVALID_ARGUMENT`입니다.

이전 prototype의 앱 이름 바이트 ID, `filters.name`, `WAFL_TOKEN`은 교체됐습니다. manifest에 Instance/Tenant ID를 명시하고 `WAFL_TOKENS`를 설정한 뒤 새 `List` 결과로 참조와 cursor를 갱신합니다.

## 시나리오

### 트래픽이 드문 두 HTTP 앱

`examples/runner.json`의 두 앱은 같은 WASM 파일과 포트 8080을 사용합니다. 각 앱을 번갈아 호출하면 이름과 요청 카운터가 독립적입니다. 5초 이상 쉬고 호출하면 해당 앱의 카운터가 1부터 시작합니다. 브라우저나 gateway의 HTTP keep-alive 연결이 남아 있어도 완료된 요청은 회수를 막지 않습니다.

### Robot과 Job의 gRPC 스트림

앱이 WASI 소켓에서 HTTP/2 gRPC 서버를 열도록 빌드하고 `protocol: "h2c"`로 등록합니다. runner의 외부 listener도 HTTP/2를 받습니다. 양방향 스트림이 열려 있으면 데이터가 잠시 흐르지 않아도 활성 요청이므로 유휴 회수를 하지 않습니다. 클라이언트 취소는 upstream 연결 종료로 전달됩니다.

통합 테스트의 `tests/guests/src/bin/grpc.rs`는 실제 WASM 안에서 `h2`를 사용해 gRPC 프레임을 echo하며 데이터, trailers, 취소를 검증합니다. 테스트용 단순 executor를 포함하며 일반 앱용 웹 프레임워크를 제공하는 예제는 아닙니다. Tonic, Go gRPC 등 개별 라이브러리의 WASIp2 호환성은 별도 검증 대상입니다.

### 앱마다 독립적인 DB 연결

앱 코드가 `127.0.0.1:5432`로 접속하면 해당 앱의 `bindings`에 있는 `target`으로 연결합니다. 다른 앱도 똑같은 guest 주소를 사용하면서 다른 DB에 연결할 수 있습니다. 로컬 개발에서는 해당 주소에 로컬 DB를 띄우면 됩니다.

이 runner는 TCP 바이트를 전달합니다. DB 계정·스키마 생성, 인증정보 배포, TLS 설정, 연결 풀, 백업은 아직 구현하지 않았습니다. 테스트는 서로 다른 TCP backend로 연결되는지 검증하며 실제 PostgreSQL 드라이버 호환성을 검증한 것은 아닙니다.

### 기존 gateway 뒤에 배치하기

Traefik/Envoy의 upstream을 runner의 `listen`에 연결하고 원래 Host/`:authority`를 보존합니다. 앱 WASM 수가 0이어도 runner 주소는 유지합니다. TLS 인증서와 외부 도메인 설정은 gateway가 담당하고 첫 요청의 인스턴스 기동은 runner가 담당합니다.

현재는 앱마다 runner 하나당 인스턴스 하나입니다. 여러 runner의 배치 조정, manifest 변경 반영, 아티팩트 업로드, Kubernetes controller와 설치 chart는 후속 작업입니다. runner 종료 시 새 연결 수락을 중단하고 기존 HTTP 연결을 최대 5초 동안 drain한 다음 남은 인스턴스를 종료합니다.

## 개발과 코드 생성

`generated/`는 직접 수정하지 않습니다. paydirt와 codegen 도구 버전을 고정한 설치·생성 스크립트를 사용합니다.

`paydirt.schema.json`은 일반 entity 생성 설정을 사용합니다. `entities: false` 옵션은 제거됐습니다. 기본 entity와 저장 코드도 생성하지만, runner는 메시지·서비스·`generated/server.rs`만 포함하고 DB Sink를 선택하지 않습니다. 필드는 `proto/wafl/instance.proto`에서, 추가 동작은 `proto/ext/wafl/instance_svc.ext.proto`에서 수정합니다. 생성된 `Get/List` 선언을 손으로 작성하지 않습니다.

```sh
bash scripts/codegen.sh
bash scripts/codegen.sh --check
bash scripts/test.sh
cargo clippy --locked --all-targets -- -D warnings
```

코드 생성에는 Rust, Go, Buf가 필요합니다. 일반 runner 빌드는 저장소에 포함된 생성 파일을 사용하므로 Go와 Buf를 요구하지 않습니다. 테스트 스크립트는 WASM fixture를 먼저 빌드한 뒤 실제 runner 통합 테스트를 실행합니다.
