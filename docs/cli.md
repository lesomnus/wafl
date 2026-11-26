# wafl CLI

`wafl app`과 `wafl traffic`은 controller API를 사용합니다. `wafl instance`·`wafl runner`·`wafl reservation`은 controller 또는 개별 runner에 연결합니다. `--endpoint`/`WAFL_ENDPOINT`로 주소를 지정하며 Kubernetes API, kubeconfig, kubectl을 사용하지 않습니다. 설치는 운영자 도구 `wafl-kube bootstrap`이 담당합니다. 전체 과정은 [Kubernetes native 튜토리얼](kubernetes-native.md)에 있습니다. 아래 예제는 단독 runner를 사용합니다.

이 문서의 기본 예제는 proxy 모드입니다. L7에서 WASM 소켓에 직접 연결하는 설정, endpoint 출력과 `release` 명령은 [직결 튜토리얼](direct.md)을 참고하세요. 직결 출력은 `<UUID>\t@alias\t<endpoint.address>`이며 `--idle-ms`·`--max-active`는 생략 또는 0만 허용합니다.

runner 장애로 남은 직결 포트는 `wafl reservation list`, `wafl reservation get <ID|IP:PORT>`로 조회합니다. 소유 세션이 종료되고 외부 경로·연결을 모두 정리했으면 `wafl reservation release <ID|IP:PORT> --token <관측한 token> --withdrawn`으로 회수합니다. token을 자동 갱신하지 않으며 활성 세션은 거절합니다. 전체 절차와 출력 열은 [예약 복구 튜토리얼](reservations.md)에 있습니다.

## 시작하기

저장소 루트에서 CLI를 설치합니다. Rust와 WASIp2 target은 `rust-toolchain.toml`에 고정되어 있습니다.

플랫폼 CLI까지 빌드하려면 `bash scripts/build.sh`를 사용하고 생성된 `wafl`과 `wafl-kube`를 같은 디렉터리에 둡니다. 아래 `cargo install`은 단독 runner CLI만 설치합니다.

```sh
cargo install --path . --locked --bin wafl
bash scripts/bundle.sh /tmp/wafl-example/hello.tar.gz
```

설치 없이 `cargo run --locked -- <명령>`으로도 실행할 수 있습니다. 기본 실행파일은 `wafl`입니다.

한 터미널에서 runner를 실행합니다. SIGINT/Ctrl-C 또는 SIGTERM을 받을 때까지 블로킹합니다.

```sh
wafl serve runner

# 디렉터리 지정: 아래 두 형태 모두 지원
wafl serve runner /srv/wafl-cache
wafl serve runner --directory /srv/wafl-cache
```

기본 디렉터리는 `$XDG_CACHE_HOME/wafl`이고, XDG 값이 없거나 상대 경로이면 `$HOME/.cache/wafl`입니다. 여러 명령을 동시에 실행하는 예시가 아니라, 사용할 명령 하나를 고릅니다.

다른 터미널에서 인스턴스를 추가합니다.

```sh
wafl instance add file:///tmp/wafl-example/hello.tar.gz \
  --alias hello --env GREETING=Hello --idle-ms 5000
```

성공하면 `<UUID>\t@hello` 한 줄을 반환합니다. `--alias`를 생략하면 `i-<UUID>` 형태의 alias를 발급합니다. 추가된 인스턴스는 이미 선언된 포트를 열고 실행 중입니다. `--port` 기본값은 8080입니다.

```sh
wafl instance list
wafl instance get @hello
wafl instance stop @hello
wafl instance start @hello
wafl instance logs @hello
wafl instance logs -f @hello
```

ID도 그대로 받습니다. 접두사 없는 이름은 UUID로 해석하고, alias는 반드시 `@`로 시작합니다. alias는 runner 안에서 유일하며 소문자 영문으로 시작하는 1–63자의 영문·숫자·하이픈입니다. API 메시지의 alias 값에는 `@`를 넣지 않습니다.

앱에 요청하려면 조회한 UUID를 사용합니다.

```sh
instance_id=$(wafl instance get @hello | cut -f1)
curl -H "wafl-instance: $instance_id" http://127.0.0.1:8080/
curl -H "wafl-instance: $instance_id" http://127.0.0.1:8080/index.html
```

hello 예제는 시작과 요청을 stdout에 기록합니다. `logs -f` 터미널에서 새 요청 로그를 확인할 수 있습니다.

## 연결과 실행 설정

기본 관리 endpoint는 `http://127.0.0.1:8081`입니다. `--endpoint` 또는 `WAFL_ENDPOINT`로 변경합니다. CLI의 관리 연결은 현재 HTTP Connect transport를 사용합니다. 번들 소스는 HTTP·HTTPS·file을 지원합니다.

플랫폼 전체를 조회할 때는 `WAFL_ENDPOINT=http://127.0.0.1:8082`로 controller를 지정합니다. `wafl runner list`는 runner별 세션·alias·주소를, `wafl runner get <ID|@alias>`는 선택한 runner를 반환합니다. 표 출력은 `<UUID>\t@alias\tingress=...\tcontrol=...\taccepting=...`이고 `--json`으로 capacity·usage 등 전체 protobuf 필드를 볼 수 있습니다. 단독 runner의 `runner get`은 인자를 생략하면 `@self`입니다.

controller에서는 연결에 실패한 runner의 경고를 stderr에 출력하면서 정상 목록을 반환합니다. 목록 명령은 전체 runner가 실패해 빈 결과를 반환해도 성공합니다. `--session <Runner ID>`로 runner 하나의 범위를 선택하며, alias를 사용하는 변경 명령과 `instance add`, `reservation release`에는 이 선택이 필수입니다. 사용 예와 오류·커서 계약은 [controller 집계 API](controller-api.md#runnerinstancereservation-집계)에 있습니다.

```sh
wafl serve runner /srv/wafl-cache \
  --listen 127.0.0.1:9080 --control 127.0.0.1:9081

wafl --endpoint http://127.0.0.1:9081 instance add \
  https://files.example.com/hello-v1.tar.gz --alias hello
```

`file:`은 runner가 실행되는 호스트/Pod의 파일 경로입니다. 원격 runner를 호출하면 CLI 컴퓨터의 파일을 업로드하지 않습니다. 로컬 file URL은 인증 없는 관리 API를 호출할 수 있는 제어평면의 신뢰된 입력으로 취급합니다.

프로세스 JSON 설정도 사용할 수 있습니다. 명시한 CLI 값이 JSON 설정을 덮어씁니다. 디렉터리 상대 경로는 프로세스 작업 디렉터리 기준입니다.

```sh
wafl serve runner --config examples/runner.json
# 이전 실행파일도 유지
cargo run --locked --bin wafl-runner -- examples/runner.json
```

OTLP 전송은 JSON의 `metrics.endpoint`로 켭니다. `wafl serve runner --config examples/runner-metrics.json`과 수집기를 함께 실행하는 방법은 [메트릭 튜토리얼](metrics.md)을 참고하세요.

## Add 옵션

CLI는 생성된 `InstanceServiceClient`로 `InstanceAddRequest`를 제출합니다. 별도 등록·Deploy RPC는 없습니다.

| 옵션 | 의미 |
| --- | --- |
| `<URL>` | 필수 번들 위치. `file:/path`와 `file:///path` 모두 가능 |
| `--alias`, `--name` | 유일한 참조 이름과 선택적인 표시 이름 |
| `--tenant` / `WAFL_TENANT` | domain 1 Tenant UUID. 기본은 로컬 개발용 `00000000-0000-8000-8001-000000000001` |
| `--port`, `--protocol` | 게스트 포트와 `http` 또는 `h2c` |
| `--env KEY=VALUE` | 반복 가능. `PORT`는 runner가 설정 |
| `--binding PORT=IP:PORT` | 반복 가능. 선택적인 IPv4 loopback 포트 매핑 |
| `--idle-ms`, `--startup-ms` | 유휴 회수·시작 제한 |
| `--memory-bytes`, `--max-active` | 실행 메모리·동시 요청 제한 |
| `--digest sha256:…` | 압축된 번들의 SHA-256 고정·검증 |
| `--manifest manifest.json` | 전체 Manifest의 protobuf JSON을 기본값으로 사용 |

기본 tenant는 DB의 tenant를 생성하는 동작이 아닙니다. 현재 runner에서 보관하는 소속 메타데이터 값입니다. 플랫폼에서는 `--tenant`로 실제 소속을 전달합니다.

`--manifest`를 사용하면 URL과 명시적인 CLI 옵션만 해당 필드를 덮어씁니다. env는 key별로 병합하고 bindings는 추가합니다. `examples/instance.json`은 AddRequest 전체이므로, 이 옵션에는 그 안의 `manifest` 객체만 저장한 파일을 사용합니다.

```sh
wafl instance add https://files.example.com/jobs-v2.tar.gz \
  --alias jobs --port 8080 --protocol h2c \
  --env DATABASE_URL=postgresql://app@postgres.default.svc.cluster.local:5432/app

wafl --json instance get @jobs
wafl instance erase @jobs
```

`--json`은 protobuf JSON을 출력하므로 bytes ID는 base64, uint64는 문자열입니다. List는 인스턴스마다 한 줄을 출력하고 다음 페이지를 자동 조회합니다. 기본 출력은 UUID와 `@alias`입니다. 실패하면 0이 아닌 종료 코드를 반환합니다.

앱이 `DATABASE_URL`을 읽고 해당 DB의 WASI 호환 클라이언트로 연결해야 합니다. runner는 connection string을 해석하지 않습니다. DNS와 직접 발신 TCP를 지원하므로 일반 DB 연결에는 `--binding`이 필요하지 않습니다. 실행 가능한 Redis 예제, 로컬 개발과 Kubernetes 주소 사용법은 [외부 연결과 DB 튜토리얼](network.md)에 있습니다.

## 로그

`logs`는 `InstanceService.Tail`을 호출합니다. stdout·stderr의 최근 1 MiB, 최대 4,096개 레코드를 메모리에 보관합니다. 레코드는 최대 8 KiB의 바이트 조각이며 완성된 줄이나 UTF-8 문자 경계를 보장하지 않습니다.

- 기본 출력은 원본 bytes를 해당 CLI stdout/stderr에 씁니다. `--json`은 channel, sequence, generation, dateMs와 base64 data를 출력합니다.
- `logs`는 보관한 기록을 출력하고 종료합니다. `-f`는 같은 인스턴스의 Stop/Start를 가로질러 새 로그를 기다립니다.
- `--limit N`은 처음에 읽는 최근 레코드 수이며 0은 보관한 전체입니다. `--cursor N`은 sequence N 다음부터 이어 읽습니다.
- sequence는 실행 generation이 바뀌어도 증가합니다. 너무 오래된 cursor나 버퍼를 따라잡지 못한 구독은 `OUT_OF_RANGE`로 종료합니다. cursor 0으로 남아 있는 기록부터 다시 읽을 수 있습니다.
- 로그 구독은 앱의 활성 요청에 포함하지 않아 유휴 회수를 막지 않습니다. Erase 또는 runner 종료 시 스트림이 종료됩니다. Ctrl-C는 CLI의 구독을 취소합니다.

로그는 runner 프로세스 수명 동안만 유지합니다. 장기 로그 저장소는 아직 연결하지 않았습니다. 번들 credential 역시 추가하지 않았습니다.

## 여러 runner와 저장소

다른 listen/control 주소를 사용하면 같은 디렉터리를 동시에 사용할 수 있습니다.

```sh
wafl serve runner /srv/wafl-cache --listen 127.0.0.1:8080 --control 127.0.0.1:8081
# 다른 터미널
wafl serve runner /srv/wafl-cache --listen 127.0.0.1:9080 --control 127.0.0.1:9081
```

번들 파일만 공유합니다. 각 runner의 인스턴스·ID·alias·로그는 독립적이며, 공유 디렉터리에 인스턴스 목록을 저장하지 않습니다. `Erase`는 다른 프로세스도 쓸 수 있는 캐시 파일을 삭제하지 않습니다. 잠금·원자적 공개·URL 재사용 규칙은 [저장소 계약](store.md)을 따릅니다.

## Runner 조회·드레인·명령 조건

```sh
wafl --json runner get
wafl instance drain @hello --timeout-ms 30000
wafl instance start @hello
wafl instance drain @hello --timeout-ms 5000 --force
wafl instance erase @hello
```

Add는 `--id <domain-7 UUID>`로 호출자가 ID를 정할 수 있습니다. global `--session <Runner UUID>`와 `--revision <number>`로 오래된 명령을 거절할 수 있으며 revision은 session과 함께 사용합니다. Add에는 session만 사용합니다. 생성 중 조회·응답 유실 복구와 각 선택의 의미는 [운영 튜토리얼](operations.md)을 참고하세요.
