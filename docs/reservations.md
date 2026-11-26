# 장애 후 직결 포트 예약 회수

runner 프로세스가 종료되면 OS가 WASM의 소켓을 닫지만, 공개한 포트의 예약 파일은 남습니다. gateway에 남은 이전 경로가 다른 앱으로 연결되는 것을 막기 위해서입니다. 같은 `direct.directory`·advertise IP·포트 범위로 실행한 runner에서 `ReservationService`로 조회하고, 외부 경로 정리 후 회수할 수 있습니다.

`wafl --endpoint http://127.0.0.1:8082 reservation list`로 controller가 감독하는 runner들의 예약을 모아 볼 수 있습니다. 공유 풀의 중복 관측은 ID로 합치고 실패한 runner는 stderr 경고로 남깁니다. 전체 runner가 실패해도 목록 명령은 빈 결과로 성공하므로 경고를 함께 확인합니다. controller를 통한 `reservation release`는 `--session <현재 수신 Runner ID>`가 필수이며 token·외부 경로 철회 확인·활성 소유자 보호는 그대로 적용합니다. [집계 API](controller-api.md)에 자세한 범위와 오류 계약이 있습니다.

| 상황 | 정리 방법 |
| --- | --- |
| WASM만 종료되고 runner는 살아 있음 | 기존 Instance를 `Drain` → 외부 경로·연결 정리 → `InstanceService.Release` |
| runner 프로세스 종료, 새 runner 또는 같은 상태를 공유하는 다른 runner가 있음 | `ReservationService.Get/List` → 외부 경로·연결 정리 → `ReservationService.Release` |
| 이전 버전의 예약 또는 손상된 상태 파일 | 조회에는 표시. 자동 회수는 거절하며 아래 업그레이드 절차 적용 |

runner가 살아 있으면 이미 알고 있는 Instance의 수명을 마무리합니다. runner가 사라진 경우에만 이번 예약 회수 API를 사용합니다. WASM 실행 자원은 게스트 종료 시 반환되므로 포트 예약이 남아 있다고 게스트 메모리를 계속 점유하는 것은 아닙니다.

## 조회와 식별

[직결 튜토리얼](direct.md)의 설정으로 runner를 실행한 상태에서:

```sh
wafl reservation list
wafl reservation get 127.0.0.1:20000
wafl --json reservation get 127.0.0.1:20000 > /tmp/wafl-reservation.json
```

주소는 예시이며 실제 `Instance.endpoint.address`를 사용합니다. 일반 출력의 탭 구분 열은 `ID, address, status, token, session, instance, generation, error`입니다. `get`과 `release`는 domain-9 Reservation UUID 또는 `IPv4:port`를 받습니다. List는 설정된 IP·포트 범위의 모든 세션을 읽고, 다음 페이지를 자동 조회합니다.

`Reservation.id`는 주소에서 파생한 안정적인 ID입니다. 같은 주소를 다른 앱에 배정해도 ID는 같습니다. `token`은 **한 번의 배정**을 식별하며 재배정하면 바뀝니다. 인증 credential이 아닙니다. `session`, `instance`, `generation`은 그 배정의 과거 소유자이며 현재 Instance 목록에 존재하지 않아도 조회됩니다.

| status | 의미 | 예약 API로 Release |
| --- | --- | --- |
| `ACTIVE` | 소유 세션의 OS 잠금이 유지됨. 게스트 실행 상태를 뜻하지 않음 | 거절. 해당 runner의 Instance API 사용 |
| `ORPHAN` | 지원하는 저널과 세션 증거가 있고 소유 잠금이 풀림 | 외부 철회 확인 및 정확한 token 필요 |
| `UNKNOWN` | 이전 저널 버전, 세션 증거 누락·손상, 잠금 확인 실패 | 거절 |
| `CORRUPT` | 예약 레코드를 읽거나 검증할 수 없음 | 거절 |

protobuf JSON에서는 `RESERVATION_STATUS_ORPHAN`처럼 enum의 전체 이름을 사용합니다. `error`에 확인 실패 이유를 제공합니다. 조회 결과는 관측 시점의 상태이며 Release는 잠금 안에서 다시 검사합니다.

## 시나리오: runner 강제 종료 후 복구

1. 같은 영속 상태와 advertise IP를 사용하는 runner를 준비합니다. 이미 동작 중인 다른 runner도 가능합니다. `InstanceService.List`는 새 프로세스의 목록이지만 `reservation list`는 이전 세션의 예약까지 보여줍니다.
2. 대상 예약을 위 명령으로 파일에 저장합니다. 해당 배정의 ID·token·주소·소유자를 고정합니다.
3. **모든 gateway**에서 그 배정의 경로를 제거하고 기존 backend connection pool을 폐기합니다. 진행 중인 연결·stream을 종료하거나 완료시킵니다. 직접 연결하는 클라이언트도 포함합니다. 설정 수신 ACK만으로는 이 단계가 완료되지 않습니다.
4. 완료를 확인한 주체가 저장한 token으로 Release합니다.

로컬 실습에서 유일한 gateway를 정상 종료하고 프로세스 종료까지 확인한 뒤 직접 연결도 모두 닫았다면 다음 명령을 사용할 수 있습니다.

```sh
# 위 3단계 완료 후 실행. 조회 파일을 최신 token으로 덮어쓰지 않습니다.
python3 - <<'PY'
import json, pathlib, subprocess
r = json.loads(pathlib.Path('/tmp/wafl-reservation.json').read_text())
assert r['status'] == 'RESERVATION_STATUS_ORPHAN'
subprocess.run([
    'wafl', 'reservation', 'release', r['address'],
    '--token', r['token'], '--withdrawn',
], check=True)
PY
```

`--endpoint` 또는 `WAFL_ENDPOINT`로 복구를 담당하는 runner의 관리 주소를 선택합니다. `--withdrawn`과 `--token`은 필수입니다. CLI는 token을 대신 조회하거나 갱신하지 않습니다. 성공하면 `true`, 이미 없는 예약이면 `false`를 출력합니다. `--json`은 `ReservationReleaseResponse`를 출력하며 protobuf 기본값인 false는 생략될 수 있습니다.

회수 후 새로운 Add/Start가 포트를 배정받을 수 있습니다. 이전 Instance가 복원되거나 이전 앱의 트래픽이 자동 복구되는 기능은 아닙니다. 새 Instance를 준비하고 새 endpoint를 gateway에 공개하는 절차는 플랫폼이 담당합니다.

## 시나리오: 중복 실행과 응답 유실

두 제어평면 replica가 같은 token으로 회수해도, 파일 잠금 안에서 한 번만 삭제합니다. 다른 배정이 없으면 나중 호출은 `released=false`입니다. RPC가 취소되어도 이미 시작한 회수 작업은 계속됩니다. 응답을 잃으면 Get으로 확인하거나 **동일한 token**으로 재시도합니다.

그 사이 같은 포트가 새 앱에 배정되었다면 `ABORTED`입니다. 최신 token을 다시 읽어 자동 재시도하면 철회한 적 없는 새 앱의 포트를 건드리게 됩니다. 원래 작업은 종료하고 새 배정은 별도 작업으로 취급합니다. runner가 살아 있거나 소유 증거가 불충분하면 `FAILED_PRECONDITION`이며 강제 우회 옵션은 없습니다.

## RPC 계약

`Reservation`은 domain 9의 전역 Payday entity입니다. `Get/List`와 조회 메시지는 paydirt/get-service로 생성하고, 동작 RPC `Release`만 확장 proto에 선언합니다. DB Sink 없이 저널을 읽는 레이어를 조합합니다.

| RPC | 입력 → 출력 |
| --- | --- |
| `ReservationService.Get` | `ReservationGetRequest` → `Reservation` |
| `ReservationService.List` | `ReservationListRequest` → `ReservationListResponse` |
| `ReservationService.Release` | `ReservationReleaseRequest` → `ReservationReleaseResponse` |

```json
{
  "ref": {"address": "127.0.0.1:20000"},
  "token": "<조회 때 저장한 allocation token>",
  "withdrawn": true
}
```

이 JSON을 `/wafl.ReservationService/Release`에 제출합니다. `ref.id`를 쓰면 UUID bytes의 base64입니다. 선택적인 `wafl-session` metadata는 **요청을 받는 현재 runner**를 가리킵니다. 종료된 소유 세션을 넣지 않습니다. `wafl-revision`은 받지 않으며 배정 조건은 token입니다.

Get은 select 생략/`all:true`이면 전체, select 지정 시 ID와 선택한 필드를 반환합니다. List는 ID순이며 기본 100개, 최대 1,000개입니다. `filters:[{"ref":{"id":"..."}},{"ref":{"address":"..."}}]`는 OR, 최대 64개이며 빈 ref는 거절합니다. `next`를 같은 범위·필터·방향의 요청 `after`로 넘깁니다. 페이지 간 스냅샷 일관성은 보장하지 않으므로 조정 루프는 반복 조회합니다. 범위 밖 주소는 `INVALID_ARGUMENT`, 없는 ID/주소의 Get은 `NOT_FOUND`입니다. proxy 모드에서는 `FAILED_PRECONDITION`입니다.

## 저장소와 업그레이드

상태 파일의 위치는 다음과 같습니다.

```text
<direct.directory>/v1/<advertise-IP>/
  catalog.lock
  <port>.json                 # version:2, token, session, instance, generation
  sessions/<session>.json     # version:1, session; 소유자가 OS 잠금을 유지
```

풀 단위 `catalog.lock`은 배정·조회·반환을 직렬화합니다. 각 runner는 새 세션 파일을 배타 생성하고 프로세스/게스트 참조 수명 동안 잠급니다. 프로세스가 죽으면 OS가 잠금을 해제합니다. PID 재사용, 시간 경과 또는 heartbeat 누락을 종료 증거로 사용하지 않습니다. 회수는 catalog와 소유 세션 잠금을 잡은 상태에서 token을 검사하고 삭제·디렉터리 fsync를 마칩니다.

새 runner 시작 시 **어떤 예약도 참조하지 않는 종료된 세션 증거만** 정리합니다. 살아 있는 세션, 예약이 참조하는 증거, 확인 불가능한 증거는 보존합니다. 예약 레코드가 손상되어 있으면 이 증거 정리도 건너뜁니다. 공개된 포트 예약은 시작 시나 TTL·bundle GC로 자동 삭제하지 않습니다.

파일 위치의 `v1`은 기존 배정과의 충돌 방지를 위해 유지하고 새 레코드에 `version:2`를 적습니다. 기존 무버전 저널은 종료를 증명할 세션 잠금이 없으므로 `UNKNOWN`입니다. 업그레이드 전에 이전 runner의 정상 `Drain/Release`로 비우는 것이 권장 절차입니다. 이미 이전 runner가 사라졌거나 상태가 손상됐다면, 해당 IP의 모든 관련 runner를 정지하고 외부 경로·연결을 철회한 상태에서 운영자가 정확한 예약 파일을 오프라인으로 정리해야 합니다. 증거 파일을 임의로 만들어 `ORPHAN`으로 보이게 하지 않습니다.

동일 advertise IP를 쓰는 프로세스는 같은 영속 디렉터리를 공유해야 합니다. 저장소는 로컬 POSIX 파일 생성·fsync·프로세스 간 OS 잠금을 지원해야 하며 NFS/object storage 분산 잠금은 지원하지 않습니다. 실행 중 `catalog.lock`이나 세션 파일을 삭제·교체하면 잠금 증거가 깨집니다. Pod IP 교체/재사용과 볼륨 수명 조정은 여전히 제어평면의 책임이며 단순 emptyDir 삭제로 안전하게 회수되지 않습니다. 다른 advertise IP의 예약을 현재 풀 API로 회수하지 않습니다.

인증과 자동 gateway 철회 프로토콜은 추가하지 않았습니다. 관리 API는 신뢰된 플랫폼 경로로 제공하며, 이 기능은 그 플랫폼이 사용할 조회·조건부 회수 수단입니다.
