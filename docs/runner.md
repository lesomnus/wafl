# Runner 사용하기

runner는 `InstanceService.Add(InstanceAddRequest)`로 인스턴스를 만들고 ID로 전달된 HTTP/gRPC 요청을 해당 WASM 소켓에 연결합니다. 앱 등록·배포·도메인 선택은 플랫폼이 담당합니다.

## 1. 로컬에서 앱 개발하기

[`examples/hello/src/main.rs`](../examples/hello/src/main.rs)는 `std::net::TcpListener`로 HTTP를 받고 `/index.html` 요청에는 파일을 읽어 응답합니다. 일반 네이티브 프로그램으로 먼저 실행할 수 있습니다.

```sh
PORT=9001 GREETING=Local DATA=examples/hello/data \
  cargo run --locked -p wafl-example-hello
```

다른 터미널에서 `curl http://127.0.0.1:9001/`와 `curl http://127.0.0.1:9001/index.html`로 확인한 후 서버를 종료합니다. Wafl SDK나 별도 핸들러 등록은 없습니다.

Rust 1.100.0과 Wasmtime 51.0.0을 고정했습니다. `wasm32-wasip2`로 같은 소스를 빌드하면 실행 가능한 WASIp2 CLI component를 생성합니다. 현재 runtime은 core WASM, WASIp1, WASIX나 `wasi:http/incoming-handler` 전용 component를 지원하지 않습니다. 다른 언어·웹 프레임워크의 WASIp2 CLI 및 소켓 지원 여부는 각각 확인해야 합니다.

## 2. 번들 만들고 URL로 제공하기

```sh
bash scripts/bundle.sh /tmp/wafl-example/hello.tar.gz
python3 -m http.server 9000 --bind 127.0.0.1 --directory /tmp/wafl-example
```

스크립트는 예제 WASM을 빌드하고 루트의 `main.wasm`과 `data/index.html`을 `tar.gz`로 묶습니다. 파일 서버를 켜둡니다. 플랫폼에서는 이 자리에 object storage나 내부 HTTP 저장소를 사용할 수 있습니다. 자세한 형식과 제한은 [번들 규격](bundle.md)에 있습니다.

## 3. 빈 runner 실행하기

다른 터미널에서 실행합니다.

```sh
cargo run --locked -- examples/runner.json
```

```json
{
  "listen": "127.0.0.1:8080",
  "control": "127.0.0.1:8081"
}
```

이 파일은 프로세스 설정입니다. `listen`은 HTTP/1.1·HTTP/2 cleartext 앱 트래픽, `control`은 Connect/gRPC 관리 RPC 주소입니다. 아직 인스턴스는 없습니다. 기존 `apps`, `artifact`, `domain` 설정은 사용하지 않습니다.

## 4. 생성된 Add RPC 호출하기

[`examples/instance.json`](../examples/instance.json)은 생성된 `InstanceAddRequest`의 protobuf JSON입니다.

```json
{
  "id": "AAAAAAAAgACABwAAAAAAAQ==",
  "tenant": {"id": "AAAAAAAAgACAAQAAAAAAAQ=="},
  "name": "hello",
  "manifest": {
    "runtime": "wasmtime",
    "abi": "wasip2",
    "bundle": {"url": "http://127.0.0.1:9000/hello.tar.gz"},
    "port": 8080,
    "protocol": "http",
    "idleMs": "5000",
    "env": {"GREETING": "Hello"}
  }
}
```

```sh
curl --fail-with-body -H 'Content-Type: application/json' \
  -H 'Connect-Protocol-Version: 1' --data-binary @examples/instance.json \
  http://127.0.0.1:8081/wafl.InstanceService/Add
```

`Add`는 다운로드·컴파일 후 선언된 포트의 listen 완료까지 기다리고 `Instance`를 반환합니다. 응답의 `manifest`에는 기본값이 채워집니다. tenant는 domain 1 Payday ID, instance는 domain 7 Payday ID입니다. protobuf의 bytes ID는 JSON에서 base64입니다. `tenant.id`는 필수이며 alias 조회는 제공하지 않습니다.

예제는 호출할 ID를 쉽게 알 수 있도록 지정했습니다. `id`를 생략하면 runner가 발급합니다. 같은 ID가 존재하거나 생성 중이면 `ALREADY_EXISTS`이며 기존 인스턴스를 변경하지 않습니다. `name`은 선택적인 표시 이름으로, 중복을 허용합니다. `phase`, `generation`, `active`, `error`는 서버 상태이므로 Add 입력에서 생략합니다. 생성된 메시지에 필드가 있더라도 기본값 이외의 값을 제출하면 거절합니다.

RPC 응답 유실 때문에 성공 여부가 불분명하면 지정한 ID로 `Get`하여 확인할 수 있습니다. ID를 생략한 Add를 재호출하면 별도 인스턴스가 생성됩니다.

## 5. 인스턴스로 요청 보내기

```sh
curl -H 'Host: hello.localhost' \
  -H 'wafl-instance: 00000000-0000-8000-8007-000000000001' \
  http://127.0.0.1:8080/

curl -H 'Host: hello.localhost' \
  -H 'wafl-instance: 00000000-0000-8000-8007-000000000001' \
  http://127.0.0.1:8080/index.html
```

`wafl-instance` 값은 base64가 아닌 UUID 문자열입니다. 이 예제의 `AAAAAAAAgACABwAAAAAAAQ==`에 해당합니다. 발급받은 ID는 Python 표준 라이브러리의 `str(uuid.UUID(bytes=base64.b64decode(id)))` 등으로 변환할 수 있습니다.

gateway는 외부 도메인을 해석해 runner와 Instance ID를 선택하고 이 헤더를 설정합니다. runner는 헤더를 제거한 뒤 원래 Host·경로·HTTP/2 authority를 유지하여 게스트에 전달합니다. 헤더 누락·잘못된 ID는 400, 없는 Instance는 404입니다. 같은 Host로도 다른 Instance를 선택할 수 있습니다.

예제는 유휴 시간이 5초입니다. 요청이 끝난 뒤 5초 이상 쉬고 `/`를 호출하면 요청 카운터가 1부터 다시 시작합니다. `/index.html`은 재시작 후에도 번들의 동일 파일을 읽습니다.

## Manifest 필드

아래는 proto 필드 이름입니다. protobuf JSON에서는 `idleMs`, `startupMs`, `memoryBytes`, `maxActive`를 사용하며 uint64는 보통 문자열로 표현합니다.

| 필드 | 의미 |
| --- | --- |
| `runtime`, `abi` | 현재 `wasmtime`, `wasip2`. 생략하면 이 기본값 사용 |
| `bundle.url` | runner가 다운로드할 HTTP(S) tar.gz URL. 필수 |
| `port` | 게스트 IPv4 TCP listen 포트, 1–65535. `PORT` 환경 변수로도 전달 |
| `protocol` | `http`(기본) 또는 HTTP/2 prior knowledge용 `h2c` |
| `idle_ms` | 활성 요청이 0이 된 뒤 회수까지의 시간. 기본 60초, 최대 7일 |
| `startup_ms` | 실행 시작부터 포트 listen까지 제한. 기본 10초, 최대 10분. 다운로드·컴파일 시간 제외 |
| `memory_bytes` | 인스턴스의 단일 linear memory 상한. 기본 64 MiB, 최소 1 MiB |
| `max_active` | 시작 대기 요청까지 포함한 인스턴스별 동시 요청 수. 기본 128 |
| `env` | 게스트 환경 변수. 호스트 환경은 상속하지 않음. `PORT` 덮어쓰기 불가 |
| `bindings` | 게스트 loopback TCP 포트를 runner가 접근 가능한 IPv4 주소로 매핑 |

수치 필드의 0은 기본값을 선택합니다. runner는 생성 중인 항목을 포함해 최대 128개 Instance, 동시에 최대 4개 Add를 받습니다. 초과 시 `RESOURCE_EXHAUSTED`입니다.

## 관리 API와 수명

| RPC | 입력 → 출력 | 동작 |
| --- | --- | --- |
| `Add` | `InstanceAddRequest` → `Instance` | manifest로 번들을 받아 실행 준비 |
| `Get` | `InstanceGetRequest` → `Instance` | 상태와 manifest 조회, `select` 지원 |
| `List` | `InstanceListRequest` → `InstanceListResponse` | 이 runner의 인스턴스를 ID순 페이지 조회 |
| `Erase` | `InstanceRef` → `InstanceEraseResponse` | 인스턴스 제거 및 번들 해제. 없으면 `erased: false` |
| `Start` | `InstanceStartRequest` → `Instance` | 보관한 인스턴스 실행 시작 또는 재사용 |
| `Stop` | `InstanceStopRequest` → `Instance` | 실행 종료. 인스턴스 ID·manifest·번들은 보관 |

HTTP 경로는 `/wafl.InstanceService/<동사>`입니다. 모두 인증 없이 호출하며 tenant에 따른 접근 범위를 제한하지 않습니다. `tenant`는 entity 소속 정보입니다. 관리 RPC와 앱 트래픽 모두 별도의 토큰 설정이 없습니다.

```sh
curl -H 'Content-Type: application/json' -H 'Connect-Protocol-Version: 1' \
  -d '{}' http://127.0.0.1:8081/wafl.InstanceService/List

curl -H 'Content-Type: application/json' -H 'Connect-Protocol-Version: 1' \
  -d '{"ref":{"id":"AAAAAAAAgACABwAAAAAAAQ=="},"select":{"phase":true,"manifest":true}}' \
  http://127.0.0.1:8081/wafl.InstanceService/Get

curl -H 'Content-Type: application/json' -H 'Connect-Protocol-Version: 1' \
  -d '{"ref":{"id":"AAAAAAAAgACABwAAAAAAAQ=="}}' \
  http://127.0.0.1:8081/wafl.InstanceService/Stop

# Start도 같은 {"ref":{"id":...}}를 사용합니다.
# Erase의 입력은 InstanceRef 자체입니다.
curl -H 'Content-Type: application/json' -H 'Connect-Protocol-Version: 1' \
  -d '{"id":"AAAAAAAAgACABwAAAAAAAQ=="}' \
  http://127.0.0.1:8081/wafl.InstanceService/Erase
```

활성 요청이 있는 `Stop`·`Erase`는 `FAILED_PRECONDITION`입니다. 응답 body·trailers가 끝나거나 클라이언트가 취소할 때까지 활성 요청으로 유지합니다. 열린 gRPC 스트림도 유휴 회수를 막습니다. HTTP keep-alive 연결만 남아 있는 경우는 회수를 막지 않습니다.

중지된 Instance에 요청이 오면 한 번만 기동하고 동시 요청들이 같은 준비 결과를 기다립니다. 첫 호출자가 취소해도 다른 요청의 기동은 계속됩니다. 시작 실패는 503, upstream 연결 실패는 502이며 전송을 시작한 요청을 자동 재시도하지 않습니다. 실패한 인스턴스에는 약 1초의 재시도 간격을 둡니다. `generation`은 시작 시도 횟수이고 `error`는 최근 실패 진단입니다.

`Get`은 select 생략 또는 `all: true`이면 전체 상태를 반환합니다. 명시적 select는 ID와 선택한 필드를 반환합니다. tenant 관계에는 ID만 보유하므로 `tenant: {}`로 선택할 수 있지만 tenant 이름·alias 등 확장은 `UNIMPLEMENTED`입니다.

`List`의 `filters: [{"ref":{"id":"..."}}]`는 OR이며 최대 64개입니다. 빈 필터는 거절합니다. size 0 이하는 기본 100, 최대 1,000입니다. `reverse`로 방향을 정하고 `next`를 같은 필터·방향의 다음 요청 `after`로 전달합니다. cursor는 ID 위치이므로 해당 인스턴스가 삭제되어도 다음 페이지를 조회할 수 있습니다. 반대 방향 cursor는 거절하며, 페이지 전체에 대한 스냅샷 일관성은 제공하지 않습니다.

## 시나리오

### 같은 번들로 여러 인스턴스 실행

예제 요청의 `id`를 생략하고 `name`과 `env.GREETING`을 바꾸어 Add를 여러 번 호출합니다. 같은 URL·같은 게스트 포트 8080이어도 ID, 메모리, 환경 변수, 소켓과 요청 카운터는 독립적입니다. 각 ID를 `wafl-instance`로 선택합니다. runner에는 별도의 앱 등록 객체가 없습니다.

### HTML과 WASM을 함께 배포

앱은 `/data/index.html`을 일반 파일 API로 읽습니다. 로컬 개발에서는 예제처럼 `DATA`로 파일 위치를 지정하고, 번들에서는 `/data` 기본값을 사용합니다. 읽기 전용 파일과 실행 코드는 Instance 수명 동안 동일하게 유지됩니다. 새 번들로 새 Instance를 만들고 준비가 완료되면 gateway 대상을 바꾼 뒤 이전 Instance를 Erase합니다. 현재 runner가 이 배포 전환을 자동 수행하지는 않습니다.

### Robot과 Job의 gRPC 스트림

앱이 WASI TCP 소켓에서 HTTP/2 gRPC 서버를 열고 `protocol: "h2c"`를 제출합니다. gateway는 authority를 보존하고 Instance ID 헤더를 설정합니다. 스트림이 열려 있으면 데이터가 잠시 없어도 회수하지 않으며, 취소는 upstream 연결 종료로 전달합니다. 실제 WASM의 `h2` 기반 fixture로 데이터·trailers·취소를 검증합니다. Tonic이나 Go gRPC 등 개별 프레임워크 호환성을 보장하는 것은 아닙니다.

### 인스턴스별 DB 연결

Manifest에 `"bindings": [{"port": 5432, "target": "10.0.0.20:5432"}]`를 넣으면 앱의 `127.0.0.1:5432` 연결을 해당 주소로 매핑합니다. 다른 Instance는 같은 게스트 포트를 다른 DB에 연결할 수 있습니다. 로컬에서는 동일한 포트에 개발 DB를 실행하면 됩니다. runner는 TCP 연결만 제공하며 DB 계정·스키마·백업·TLS·연결 풀을 관리하지 않습니다.

### Kubernetes gateway와 연동

Traefik/Envoy 등의 upstream을 runner의 `listen`으로 연결하고, 플랫폼 제어평면이 runner와 Instance ID를 선택해 `wafl-instance`를 설정합니다. 외부 클라이언트의 해당 헤더는 gateway가 선택한 값으로 덮어씁니다. 도메인과 TLS는 gateway가 관리합니다. 현재 runner 간 배치·확장, gateway 동적 설정, controller와 chart는 후속 범위입니다.

## 실행 제약과 개발

게스트는 IPv4 `0.0.0.0` 또는 loopback에 bind하며 실제 호스트 포트는 인스턴스마다 다릅니다. WASI 호출 수준의 소켓 가상화이며 OS network namespace를 생성하지는 않습니다. `/data` 이외의 파일시스템 preopen, 호스트 환경 상속, DNS, UDP와 선언하지 않은 outbound 연결은 제공하지 않습니다.

WASI resource table은 인스턴스당 1,024개입니다. 공통 epoch clock이 CPU 루프를 주기적으로 yield하여 취소·시작 제한을 처리하지만 CPU quota는 아닙니다. stdout은 버리고 stderr는 최대 16 KiB를 실패 진단에 보관합니다. 별도 로그 API는 없습니다. 요청별 upstream TCP 연결을 사용하며 WebSocket/Upgrade·CONNECT는 501입니다. 프로세스 종료 시 HTTP 연결을 최대 5초 drain한 뒤 나머지 실행을 종료합니다.

`Instance`는 `(orm.message)`·`(payday.entity)`가 선언된 일반 entity입니다. `tenant = 2`, `manifest = 17`이며 이전 `domain = 12`, `port = 16`은 예약했습니다. 포트는 `manifest.port`에 있습니다. **Add/Get/List/Erase와 요청 메시지는 codegen이 생성**하고, Start/Stop만 확장 proto에 선언합니다. custom Deploy RPC는 없습니다.

서버는 생성된 `Unimplemented → Instances` 레이어를 조합합니다. 메모리 구현을 선택하고 DB Sink를 넣지 않으므로 DB 연결·마이그레이션은 실행하지 않습니다. runner 재시작 후에는 플랫폼이 Instance를 다시 제출해야 합니다.

```sh
bash scripts/codegen.sh
bash scripts/codegen.sh --check
bash scripts/test.sh
cargo clippy --locked --all-targets -- -D warnings
```

`generated/`를 직접 수정하지 않습니다. 코드 생성에는 Rust, Go와 Buf가 필요하고 일반 빌드는 포함된 생성 파일을 사용합니다. 테스트 스크립트는 WASM fixture를 먼저 빌드한 뒤 실제 runner를 검증합니다.
