# Controller와 Traefik으로 앱 실행하기

`wafl serve controller`는 App의 Manifest와 replica 수를 PostgreSQL에 저장하고, runner를 선택해 Instance를 만듭니다. Traefik은 controller의 `/traefik`을 주기적으로 읽어 도메인별 backend를 갱신합니다. 앱 요청은 **Traefik → runner가 배정한 WASM 소켓**으로 전달됩니다. controller와 runner의 HTTP 중계는 요청 경로에 없습니다.

```text
wafl app add/patch ─→ controller replicas ─→ PostgreSQL: App, Placement
                          │
                          └→ runner 관리 RPC → WASM Instance
Traefik ── 설정 조회 ─────→ GET /traefik
사용자 ── Host/authority → Traefik ─────────→ Instance.endpoint
```

이 문서는 고정 replica 운영을 설명합니다. 고정 방식의 `replicas=0`에는 503을 반환합니다. **YAML 등록, 요청 부하 기반 자동 증감과 최초 요청 기동은 [자동 증감 튜토리얼](autoscaling.md)**을 참고하세요. 앱은 Manifest의 env로 전달받은 DB에 직접 연결합니다. Kubernetes runner 발견과 앱 DB 자동 제공은 후속 범위입니다.

## 로컬에서 끝까지 실행

Rust 툴체인 외에 PostgreSQL 클라이언트 개발 라이브러리와 Traefik v3가 필요합니다. Debian/Ubuntu에서는 `libpq-dev`를 설치합니다. PostgreSQL은 기존 서버를 사용하거나 다음과 같이 로컬 Docker로 시작할 수 있습니다. 예제 비밀번호는 로컬 실습용입니다.

```sh
docker run --name wafl-postgres --rm \
  -e POSTGRES_PASSWORD=wafl-example -e POSTGRES_DB=wafl \
  -p 127.0.0.1:5432:5432 postgres:17-alpine
```

별도 터미널에서 저장소 루트를 기준으로 준비합니다.

```sh
cargo install --path . --locked --bin wafl
bash scripts/bundle.sh /tmp/wafl-example/hello.tar.gz
# Linux amd64용 고정 버전 설치 helper. 이미 설치한 Traefik v3도 사용 가능.
bash scripts/install-traefik.sh /tmp/wafl-traefik
```

runner 두 개를 각각 별도 터미널에서 실행합니다. 같은 호스트 IP이므로 **동일한 direct 상태 디렉터리와 포트 풀**을 사용합니다. bundle cache도 공유하지만 두 디렉터리는 서로 분리합니다.

```sh
wafl serve runner /tmp/wafl-cache --config examples/runner-direct.json \
  --control 127.0.0.1:8081
```

```sh
wafl serve runner /tmp/wafl-cache --config examples/runner-direct.json \
  --control 127.0.0.1:9081
```

또 다른 터미널 두 개에서 controller와 gateway를 실행합니다. 각각 종료 신호를 받을 때까지 블로킹합니다.

```sh
wafl serve controller --config examples/controller.json
```

```sh
/tmp/wafl-traefik/traefik --configFile=examples/traefik-controller.yaml
```

이제 CLI의 대상을 controller로 정하고 앱을 만듭니다.

```sh
export WAFL_ENDPOINT=http://127.0.0.1:8082
wafl app add file:///tmp/wafl-example/hello.tar.gz \
  --alias hello --domain hello.localhost --replicas 2 --env GREETING=Hello
wafl app get @hello
wafl app list
wafl placement list --app @hello
curl -H 'Host: hello.localhost' http://127.0.0.1:8080/
curl -H 'Host: hello.localhost' http://127.0.0.1:8080/index.html
```

`app add`는 원하는 상태를 저장한 뒤 반환합니다. 별도의 조정 루프가 다운로드·실행을 진행하므로 `placement list`에서 `PLACEMENT_PHASE_READY`가 될 때까지 기다립니다. gateway 설정 반영에는 provider polling 간격도 필요합니다. App 기본 출력은 UUID, `@alias`, domain, replicas, revision이고 Placement 출력은 UUID, phase, runner URL, 실행 주소, error입니다. `--json`은 protobuf JSON을 출력하며 List는 페이지를 모두 조회합니다.

`file:` URL은 **선택된 runner의 파일 경로**입니다. 여러 호스트에서는 모든 runner가 접근할 HTTP/HTTPS 번들 URL을 사용하세요. 첫 정상 실행에서 관측한 SHA-256을 App Manifest에 고정한 뒤 추가 replica를 만듭니다. 처음부터 `--digest sha256:…`를 지정하는 것도 가능합니다. URL의 내용만 바꾸는 방식으로 배포하지 않습니다.

도메인은 하나의 정확한 소문자 ASCII hostname입니다. wildcard, port, Traefik rule 표현식은 받지 않습니다. Unicode 도메인은 ASCII 형태로 변환해 제출합니다. 도메인이 없는 요청은 Traefik에서 404, 등록된 앱에 준비된 backend가 없으면 503을 받습니다. TLS 인증서와 public entrypoint 구성은 기존 Traefik 배포에서 담당합니다.

## 앱 변경, gRPC, 로그

```sh
wafl app patch @hello --replicas 3
wafl --json app get @hello
wafl --json placement list --app @hello

# 전체 Manifest JSON을 새 배포 입력으로 사용
wafl app patch @hello --manifest /path/to/manifest.json --expect-revision 1

# 실제 gRPC 소켓 서버가 들어 있는 bundle
wafl app add https://files.example.com/grpc-v1.tar.gz \
  --alias grpc --domain grpc.example.com --protocol h2c
```

Manifest 변경은 App revision을 증가시킵니다. replica 수나 표시 이름만 바꾸면 revision은 유지됩니다. `--expect-revision`은 Manifest revision의 비교 조건이므로 replica 수 변경끼리의 충돌을 감지하는 전체 resource version은 아닙니다. Manifest는 전체 교체이며 URL, env, memory 등 필요한 필드를 포함합니다. direct 모드에 맞게 `idleMs`·`maxActive`는 생략하거나 0이어야 합니다.

기존 revision의 Placement는 철회 대상으로 바뀌고 새 revision이 배치됩니다. 현재는 준비 완료를 기다리는 rolling update 계약이 아니므로 변경 중 503이 발생할 수 있습니다. 이전 실행은 외부 연결 철회 확인 전까지 자원을 점유하므로 새 실행을 위한 여유 용량이 필요합니다.

Instance 로그는 Placement가 가리키는 runner에서 읽습니다. `wafl --json placement get <Placement UUID>`의 `runner`, `instance`를 사용합니다. JSON의 bytes ID는 base64이므로 UUID 변환이 필요합니다.

```sh
wafl --json placement list --app @hello > /tmp/hello-placements.jsonl
python3 - <<'PY'
import base64, json, uuid
for line in open('/tmp/hello-placements.jsonl'):
    p = json.loads(line)
    print(p['runner'], uuid.UUID(bytes=base64.b64decode(p['instance'])))
PY
wafl --endpoint http://127.0.0.1:8081 instance logs -f <Instance UUID>
```

controller가 소유한 Instance를 수동 Start/Erase로 변경하지 않습니다. 같은 ID가 다른 generation으로 실행되면 controller는 그 실행을 경로에 공개하지 않습니다.

## 여러 controller와 배치 결정

동일한 설정 파일과 PostgreSQL을 사용하고 관리 listen 주소만 바꾸면 controller를 추가할 수 있습니다.

```sh
wafl serve controller --config examples/controller.json --listen 127.0.0.1:9082
```

App/Placement는 PostgreSQL의 Paydirt Store에 저장됩니다. 시작 시 생성된 모델을 설치하고 로컬 개발 Tenant를 생성합니다. 서비스 계약은 entity에서 생성한 `AppService.Add/Get/List/Patch/Apply/Erase`, `PlacementService.Get/List`를 사용하며, 외부 철회 확인은 별도 동사 RPC `PlacementService.Release`입니다. controller는 서버 레이어에서 검증·조정과 Store를 연결합니다. runner의 Instance는 계속 프로세스 메모리에서 관리합니다.

controller들은 짧은 DB transaction으로 배치 결정을 직렬화하고 그 밖에서 runner를 호출합니다. Placement에 runner URL, session, 고정 Instance ID와 Manifest revision을 먼저 저장합니다. Add 응답이 유실되거나 두 controller가 동시에 호출해도 같은 ID로 재시도하고 Get으로 복구합니다. 준비 중이거나 철회 중인 실행도 예약한 용량에 포함합니다. 정상 실행과 최신 관측이 모두 확인된 주소만 gateway 설정에 포함합니다.

같은 App을 이미 실행하는 runner를 우선하고, 그다음 실행·예약 수가 적은 runner를 선택합니다. runner의 수락 여부와 실행·메모리·등록 수 한도도 확인합니다. 앱별로 준비 중인 Placement 하나, 전체적으로 `max_pending`개까지 만들 수 있습니다. 이는 분산 장애 영역이나 정밀 CPU 부하를 고려하는 scheduler는 아닙니다. 실제 수락 여부는 runner가 최종 판단합니다.

이미 생성된 실행이 FAILED가 되면 해당 revision의 새로운 배치를 멈춥니다. 원인을 확인해 Manifest를 변경하거나 실패한 Placement를 철회·회수한 뒤 재시도합니다. 실행 전 Add 실패로 runner에 Instance가 남지 않으면 같은 Placement/Instance ID로 재시도하며, DB에 저장한 `failures`·`date_retry`에 따라 1초부터 최대 60초까지 지수형 대기를 적용합니다. runner 연결 상실은 `fresh_ms` 이후 이전 배치를 철회하고 살아 있는 runner에 대체 배치를 만듭니다. 철회 확인이 누적되어 앱별 `max_retiring`에 이르면 새 배치를 멈추므로 반복 장애가 실행을 무한히 늘리지 않습니다.

`runners`는 **개별 프로세스의 안정적인 관리 URL** 목록입니다. 여러 runner에 임의 분산하는 Service 주소를 넣지 않습니다. 같은 runner가 재시작하면 같은 URL에서 새 session을 관측합니다. 이 초기 버전에서는 topology, freshness, 배치 한도와 entrypoint를 DB에 설정 fingerprint로 고정하며 서로 다른 설정의 controller 시작을 거절합니다. 운영 중 runner 목록 변경·Kubernetes 발견·설정 migration API는 후속 작업입니다. 임의 DB 수정이나 초기화로 살아 있는 배치 기록을 지우지 않습니다.

controller replicas 앞에는 기존 load balancer를 둘 수 있습니다. Traefik의 provider endpoint를 그 주소로 지정하면 어느 controller에서든 같은 DB의 설정을 받습니다. 현재 한 DB 조정 행이 결정들을 직렬화하므로 controller replica 증가는 가용성과 작업 병렬성을 제공하며, DB 결정 처리량을 선형으로 늘리지는 않습니다.

## 축소와 장애 후 포트 회수

```sh
wafl app patch @hello --replicas 0
wafl placement list --app @hello
```

controller는 Placement를 RETIRING으로 만들고 `/traefik`에서 제외한 다음 원래 runner에 Drain을 요청합니다. direct Drain은 이미 열린 WASM 소켓과 기존 연결을 즉시 종료하지 않습니다. Traefik이 새 설정을 가져갔다는 사실이나 일정 시간 경과만으로는 모든 gateway의 이전 연결이 끝났음을 증명할 수 없습니다.

로컬 실습에서는 **유일한 Traefik을 종료하고 프로세스 종료까지 기다린 다음** 각 RETIRING Placement를 확인합니다. gateway가 여러 개이면 모두에서 이전 경로, 연결 풀, 장기 스트림이 더는 사용되지 않아야 합니다. 그 사실을 확인한 운영자나 외부 gateway 조정 코드가 다음 명령을 실행합니다.

```sh
wafl placement release <Placement UUID> --withdrawn
wafl placement get <Placement UUID>
# 모든 Placement가 RELEASED가 된 뒤
wafl app erase @hello
```

Release RPC는 철회 확인을 저장하고 반환합니다. worker가 같은 session의 Instance.Release를 호출하거나, runner가 재시작했다면 기존 Reservation을 조회한 token으로 조건부 회수합니다. 살아 있는 소유 세션, 잘못된 소유 정보와 오래된 token은 runner가 거절합니다. runner가 계속 접근 불가능하면 회수도 기다립니다. 이전 주소를 관측하기 전에 Add 응답과 runner를 동시에 잃은 경우에는 자동으로 안전을 추정하지 않고 error를 남기므로 [예약 복구 절차](reservations.md)로 원래 풀을 조사해야 합니다.

RELEASED 뒤 runner의 Stopped Instance 등록은 유지합니다. 늦게 도착한 중복 Add가 같은 ID를 새 실행으로 되살리지 못하게 하는 기록입니다. 등록 수와 cache pin을 계속 사용하므로, 해당 App을 지우고 모든 controller를 정지해 진행 중인 RPC도 끝난 것이 확인된 유지보수 시점에 필요하면 runner에서 이 기록을 Erase합니다. 자동 tombstone GC는 후속 계약입니다.

## 설정과 검증

| 설정 | 기본값 | 의미 |
| --- | --- | --- |
| `database` | 필수 | controller 전용 PostgreSQL URL |
| `runners` | 필수 | 1–64개 HTTP 관리 origin |
| `listen` | `127.0.0.1:8082` | 관리 RPC와 `/traefik` |
| `interval_ms` | 1000 | 조정 주기 |
| `probe_ms` | 2000 | runner 관측 제한 |
| `operation_ms` | 120000 | Add/Release 등 실행 작업 제한 |
| `fresh_ms` | 10000 | 경로 공개에 허용하는 관측 수명 |
| `max_pending` | 4 | 전체 준비 중인 배치 수 |
| `max_retiring` | 32 | 앱별 철회 대기 한도. 도달하면 추가 배치 중지 |
| `max_replicas` | 32 | 앱별 목표 수 상한 |
| `entrypoint` | `web` | 기존 Traefik HTTP entrypoint 이름 |

관측은 runner 8개씩 병렬로 수행합니다. `fresh_ms`는 `3 × (interval_ms + probe_ms × ceil(runner 수 / 8))` 이상이어야 합니다. 현재 관리 API에는 인증이 없으며 trusted 관리망에서 사용합니다. PostgreSQL URL·Manifest env와 RPC를 public Traefik 앱 entrypoint에 함께 노출하지 않습니다.

Traefik은 [HTTP provider](https://doc.traefik.io/traefik/reference/install-configuration/providers/others/http/)의 polling으로 설정을 받습니다. controller의 DB 조회가 실패하면 503을 반환하며 Traefik은 직전 설정을 유지할 수 있습니다. 모든 controller/DB 장애에서 즉시 경로가 제거된다고 가정하지 않습니다. 이 경우에도 철회 확인 없이 포트를 재사용하지 않습니다. [service backend](https://doc.traefik.io/traefik/reference/routing-configuration/http/load-balancing/service/)에는 `http://` 또는 `h2c://` 주소를 쓰고 원래 Host를 보존합니다.

CI는 PostgreSQL, controller 2개, runner 2개, 실제 Traefik 3.8.0과 WASI HTTP/gRPC 앱을 실행합니다. 중복 배치 방지, controller 재시작, runner 강제 종료·교체, 도메인별 응답, Host 유지, gRPC framing/trailer와 철회 후 회수를 검증합니다.

```sh
WAFL_TEST_CONTROLLER_DATABASE_URL=postgresql://postgres:wafl-example@127.0.0.1:5432/wafl \
WAFL_TEST_TRAEFIK=/tmp/wafl-traefik/traefik \
  bash scripts/test.sh --test cli controller:: -- --include-ignored
```
