# Runner 수명·용량 운영 튜토리얼

이 문서는 단독 runner 운영과 이후 제어평면이 사용할 계약을 설명합니다. 앱 등록·domain 라우팅·배치 결정을 구현하는 제어평면은 아직 없습니다. runner는 생성된 `InstanceService.Add`로 제출받은 Manifest를 실행합니다.

아래 트래픽·Drain·자동 재기동 설명은 기본 proxy 모드 기준입니다. [직결 모드](direct.md)에서는 `Instance.endpoint`로 연결하며 HTTP 요청·유휴 상태는 외부에서 판단합니다. `Drain`은 표시만 바꾸고 실행을 유지하며, gateway 철회·연결 정리 후 `Release`로 포트를 반환합니다. 실행·메모리·준비 한도는 동일하게 적용합니다.

## 시작과 관측

```sh
wafl serve runner /tmp/wafl-cache --config examples/runner-limits.json
# 다른 터미널
wafl runner get
wafl --json runner get
```

`Runner`는 domain 8의 일반 Payday entity입니다. 여러 tenant를 호스팅하는 프로세스의 정보이므로 `global: {}`를 명시하며 tenant를 임의로 붙이지 않습니다. 생성된 `RunnerService.Get`/`List`를 runner의 메모리 layer가 구현합니다. `Get`의 ref는 현재 Runner ID 또는 `{"alias":"self"}`입니다. `List`는 현재 프로세스 한 개만 반환하며 다른 runner를 검색하지 않습니다. Get select는 ID와 선택한 필드를 반환합니다.

ID는 **실행 세션**이며 재시작마다 바뀝니다. Instance의 `session`과 OTLP `service.instance.id`가 같은 세션을 가리킵니다. `runtime`, `abi`, `version`, `capabilities`, `ingress`, `control`, `accepting`, `capacity`, `usage`를 조회할 수 있습니다. 주소는 bind 주소이며 외부에서 접근 가능한 advertise 주소나 gateway 경로를 자동 계산하지 않습니다. `0.0.0.0`을 접속 대상으로 사용하면 안 됩니다.

## 수락 제한

runner JSON 설정의 `capacity`는 프로세스 전체 한도입니다. 생략한 값에는 기본값을 적용합니다. Instance가 갖는 `Manifest.memory_bytes`, `max_active`는 이 전체 한도와 함께 적용합니다.

| 설정 | 기본값 | 의미 |
| --- | ---: | --- |
| `instances` | 128 | 생성 중 항목을 포함한 등록 Instance 수 |
| `preparations` | 4 | 다운로드·압축 해제·컴파일·초기 기동을 포함한 Add 슬롯 |
| `startups` | 4 | WASM 실행을 시작해 선언 포트의 TCP listen을 기다리는 시도 수 |
| `running` | 128 | 기동 대기·기동 중·실행 중인 실행 예약 수 |
| `memory_bytes` | 8589934592 | 실행 예약들의 선언된 linear-memory 상한 합계 |
| `requests` | 1024 | 전체 실행 lease: HTTP/gRPC 요청과 Add/Start의 준비 대기 포함 |
| `waiting` | 128 | startup 슬롯을 기다리는 시도 수. 같은 Instance 요청은 하나로 합침 |
| `queue_ms` | 10000 | startup 슬롯 대기 기한 |
| `drain_ms` | 30000 | Drain RPC의 기본 기한 및 프로세스 ingress 종료 대기 |
| `compilations` | 32 | Instance가 참조하지 않는 컴파일 결과의 보관 수 |

모두 양수이고 시간 값은 최대 600000 ms입니다. 메모리 합계는 최소 1 MiB입니다. 실행 수와 메모리 예약은 한 원자적 결정으로 수락하며, 초과하면 `RESOURCE_EXHAUSTED`입니다. startup 슬롯만 제한된 FIFO 대기를 제공하고, 대기열이 가득 차면 즉시 거절하며 기한 초과는 `DEADLINE_EXCEEDED`입니다. 요청자는 기동 대기를 포함해 `requests`와 Instance의 `max_active`를 소비합니다.

Add·명시적 Start·트래픽에 의한 자동 재기동이 같은 startup/실행/메모리 예약을 사용합니다. 같은 Instance의 동시 cold request는 한 기동을 기다립니다. 첫 요청이 취소돼도 공유 기동은 계속되며, startup/queue 기한 또는 유휴 회수가 수명을 제한합니다. Stop·Erase·기동 실패·게스트 종료에서 실행 메모리와 소켓을 정리한 뒤 예약을 반환합니다. 취소된 blocking 컴파일이 이미 실행 중이면 실제 종료까지 Add 슬롯을 보유합니다.

`memory_bytes` 예약은 process RSS 상한이 아닙니다. Wasmtime 코드·메타데이터·테이블·소켓·프록시·임시 버퍼는 linear memory 밖에 있습니다. 실제 프로세스 메모리는 OTLP RSS로 관측하고 필요한 경우 Pod 제한을 별도로 적용합니다. CPU/fuel 실행량 제한은 하지 않습니다.

컴파일 key는 `main.wasm` bytes의 SHA-256이며 같은 engine을 쓰는 프로세스 안에서 공유합니다. manifest나 `/data`가 달라도 코드는 공유하고 실행 상태는 분리합니다. 동시 컴파일도 하나로 합칩니다. Instance가 사용하는 결과는 보유하고, 마지막 참조가 끝나면 미참조 결과를 LRU로 최대 `compilations`개 유지합니다. 이 수는 정확한 메모리 바이트 한도가 아니며 사용 중인 결과는 Instance 수와 preparation 슬롯으로 별도 제한됩니다. 프로세스 간 JIT 결과는 공유하지 않습니다.

## 생성 중 조회와 응답 유실

```sh
# 재시도에 쓸 ID를 먼저 정할 때는 domain-7 Payday ID를 사용합니다.
wafl instance add file:///tmp/wafl-example/hello.tar.gz \
  --id 00000000-0000-8000-8007-000000000001 --alias hello
# Add 진행 중 별도 터미널에서
wafl --json instance get @hello
wafl --json instance list
```

검증된 요청의 ID·alias를 예약하면 `PHASE_PREPARING`으로 조회됩니다. 다운로드·추출·컴파일이 끝나면 `PHASE_STARTING`, TCP listen이 끝나면 `PHASE_RUNNING`이 됩니다. 준비 중에는 Get/List만 사용하고 Start/Stop/Drain/Erase는 `FAILED_PRECONDITION`입니다. ingress는 Add 완료 전 트래픽을 `503`, `wafl-error: preparing`으로 거절합니다. TCP listen은 준비 기준이며 HTTP health check나 DB 연결 성공까지 확인하지 않습니다.

Add는 준비 완료까지 기다립니다. 실패·취소 시 아직 완료되지 않은 항목은 제거하고 ID·alias를 반환합니다. 성공과 취소가 경합하거나 응답이 유실되면 이미 완료된 Instance가 있을 수 있습니다. caller ID 또는 alias로 Get하여 확인합니다. 동일 ID의 Add는 `ALREADY_EXISTS`이며 기존 Instance를 수정하거나 새로 실행하지 않습니다. Get 결과의 manifest/tenant를 확인한 뒤 기존 결과를 사용하고, 없으면 같은 ID로 다시 제출합니다. 응답 유실 뒤 새 ID를 발급해 반복하면 중복 실행을 만들 수 있습니다.

재시작 후 Instance 목록은 유지되지 않습니다. 캐시만 남으며 플랫폼이 같은 manifest를 다시 제출합니다. Add가 발급한 ID만 사용하고 그 응답을 잃으면 ID로 직접 조회할 수 없으므로, 응답 유실 복구가 필요한 호출자는 ID 또는 유일한 alias를 지정합니다.

## Drain과 명시적 재개

```sh
wafl instance drain @hello --timeout-ms 30000
wafl --json instance get @hello
# 필요하면 명시적으로 다시 수락
wafl instance start @hello
# 종료 기한 뒤 남은 요청도 끊기로 결정한 경우
wafl instance drain @hello --timeout-ms 5000 --force
wafl instance erase @hello
```

Drain은 원자적으로 `draining: true`를 설정해 새 요청과 자동 재기동을 차단합니다. 이미 수락한 HTTP 응답 body, gRPC 데이터·trailers, 기동 대기 lease가 모두 끝나면 실행을 Stop합니다. 등록 정보·번들·로그는 유지하며 이후 요청은 계속 거절합니다. **Start가 명시적인 재개**입니다. 기존 Stop/유휴 회수는 수락을 닫지 않으므로 다음 트래픽이 재기동합니다.

`timeout_ms: 0`은 runner 기본값을 쓰고 최대는 10분입니다. 기본 `force: false`에서 기한을 넘기면 `DEADLINE_EXCEEDED`를 반환하고 수락은 닫힌 채 기존 요청을 계속 처리합니다. 다시 Drain하거나 Start로 재개할 수 있습니다. `force: true`이면 기한에 게스트를 종료하고 남은 스트림을 중단합니다. 기한 내 자연 종료하면 force 여부와 무관하게 정상 종료합니다.

Drain RPC를 취소해도 이미 닫힌 수락을 열거나 정리를 취소하지 않습니다. 같은 진행 중 Drain을 다시 호출하면 최초 작업의 기한/force를 공유합니다. 진행 중 Start/Stop/Erase는 거절하므로 완료를 조회하거나 Drain 결과를 기다립니다. Get의 `draining`은 수락 여부이며 작업 완료는 phase 및 Drain 결과와 함께 판단합니다. `generation`은 start 시도 번호이고 lifecycle `revision`과 다릅니다.

SIGINT/SIGTERM에서는 Runner `accepting`을 false로 바꿔 Add와 새 실행 lease를 거절하고 ingress 연결을 graceful shutdown합니다. 설정된 `drain_ms` 동안 기존 요청을 기다린 후 나머지 연결·게스트를 종료합니다. 프로세스는 재개할 수 없으며 새 프로세스는 새 세션입니다.

## 오래된 명령 거절

```sh
session=$(wafl runner get | cut -f1)
revision=$(wafl --json instance get @hello | python3 -c 'import json,sys; print(json.load(sys.stdin)["revision"])')
wafl --session "$session" --revision "$revision" instance drain @hello
```

관리 RPC metadata의 `wafl-session`은 domain-8 UUID 문자열이고 `wafl-revision`은 십진 uint64 문자열입니다. revision을 지정하려면 session도 필요합니다. Add는 session만 받고 Start/Stop/Drain/Erase는 둘 다 받습니다. 기존 generated Add/Erase 요청 모양을 유지하기 위해 metadata를 사용합니다. 생략한 호출은 기존대로 현재 상태를 대상으로 실행합니다.

다른 세션은 `FAILED_PRECONDITION`, 다른 revision은 `ABORTED`입니다. revision 검사와 변경은 같은 잠금 안에서 처리합니다. revision은 프로세스 전체에서 단조 증가하므로 같은 ID를 Erase 후 다시 Add해도 옛 명령이 새 Instance에 적용되지 않습니다. Get/List의 조회 값과 후속 명령 사이에 상태가 바뀌면 다시 조회하고 판단합니다. 요청 수의 변화마다 revision을 올리지는 않습니다.

이 조건은 runner의 현재 상태에 대한 낙관적 동시성 검사입니다. 제어평면 replica의 소유권·리더 임대·fencing token을 구현한 것은 아닙니다. 그 계약은 제어평면 단계에서 별도로 정합니다. 인증도 추가하지 않았습니다.

## 오류와 재전송

| ingress HTTP | `wafl-error` | 의미 |
| --- | --- | --- |
| 400 | `invalid_request` | authority/Instance ID/session 형식 오류 |
| 404 | `not_found` | 이 runner에 Instance가 없음 |
| 429 | `capacity` | Instance 또는 runner 수락 한도 초과 |
| 503 | `preparing` | Add가 아직 완료되지 않음 |
| 503 | `draining` | 수락 중단 또는 이전 실행 정리 중 |
| 503 | `session_mismatch` | gateway가 지정한 runner 세션과 다름 |
| 503 | `unavailable` | runner 종료 또는 게스트 시작/실행 실패 |
| 504 | `startup_timeout` | startup queue 또는 listen 준비 기한 초과 |
| 502 | `upstream` | 연결/응답 실패. 앱에 전달되었을 가능성이 있음 |
| 501 | `unsupported` | Upgrade 또는 CONNECT |

위 거절은 HTTP 레벨이며 관리 RPC는 대응하는 Connect/gRPC 오류 code를 사용합니다. readiness는 TCP listen이고 요청별 upstream 연결 제한은 5초입니다. 첫 전달 전에 거절된 요청과 이미 전달이 시작된 요청을 구분해야 합니다. runner는 요청 본문을 자동 재전송하지 않습니다. HTTP body·gRPC stream의 정상 종료/취소/실패가 lease를 반환하며, response 시작 뒤 오류는 스트림 실패로 나타날 수 있습니다. 게스트가 보낸 `wafl-error`는 제거해 runner 진단과 섞이지 않게 합니다.

Gateway는 Host로 runner 내부 라우트를 결정하지 않습니다. `wafl-instance: <Instance UUID>`를 주입하고 선택적으로 `wafl-session: <Runner UUID>`를 지정합니다. 둘은 게스트 전달 전에 제거하며 원래 Host/authority는 보존합니다. 제어평면이나 gateway 재시도 정책은 여기서 구현하지 않습니다.

## 검증 시나리오

- **동일 코드의 여러 앱:** 같은 main.wasm에 다른 `/data`와 환경 변수를 넣어 Add합니다. 실행 상태·응답 카운터는 분리되고 컴파일 결과만 공유합니다.
- **한도 회수:** `running: 1`로 설정한 뒤 두 번째 Instance를 Add하면 거절됩니다. 첫 Instance를 Stop하면 실행/메모리 예약을 반환하여 다음 Add가 성공합니다. 등록 수·캐시 pin은 Erase 전까지 남습니다.
- **배포 교체:** 새 Instance Add 완료 → gateway 대상 변경 → 이전 Instance Drain → Erase 순서로 호출합니다. runner는 gateway 규칙을 변경하지 않습니다.
- **긴 스트림:** gRPC 스트림을 연 뒤 Drain합니다. 새 스트림은 거절되지만 기존 스트림은 데이터와 trailers를 마칠 수 있습니다. 기한 정책은 위 force 계약을 따릅니다.
- **재시작·지연 명령:** 세션/revision을 저장하고 runner를 재시작한 뒤 옛 조건으로 명령하면 거절됩니다. 새 세션을 조회하고 manifest를 재제출합니다.
- **공유 캐시 용량:** [저장소 문서](store.md)의 두 runner·Stop pin·Erase·crash 예시를 적용합니다. 디스크 GC와 실행 예약은 독립적입니다.

### gRPC 요청 기한

[`grpc-timeout` 규격](https://github.com/grpc/grpc/blob/master/doc/PROTOCOL-HTTP2.md#requests)의 최대 8자리와 H/M/S/m/u/n 단위를 지원합니다. cold start·queue·upstream 응답·열린 스트림에 걸쳐 **하나의 절대 기한**을 적용하고, 게스트에 전달할 때 이미 기다린 시간을 차감합니다. 헤더가 없으면 요청 전체의 임의 기한을 추가하지 않습니다. startup/queue/연결 한도는 별도로 적용합니다.

요청 기한이 response 시작 전에 끝나면 gRPC trailers-only 응답(`HTTP 200`, `grpc-status: 4`, `wafl-error: deadline`)을 반환합니다. 응답 시작 후에는 deadline trailer를 내보내고 upstream과 lease를 정리합니다. 클라이언트가 응답을 읽지 않아 flow control로 전송이 멈춰도 별도 기한 작업이 upstream과 lease를 해제합니다. 클라이언트 취소도 upstream을 끊고 lease를 반환합니다. 공유 기동은 다른 호출자와 독립적으로 계속될 수 있으며 그 요청의 body를 나중에 다시 전달하지 않습니다. HTTP 요청 자체의 전체 기한은 클라이언트 취소로 제어하며 장기 스트림에 임의의 전체 timeout을 강제하지 않습니다.
