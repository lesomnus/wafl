# Controller API와 CLI

`wafl app`과 `wafl traffic`은 controller API를 호출합니다. `wafl instance`, `wafl runner`, `wafl reservation`은 같은 RPC를 controller 또는 개별 runner에 호출할 수 있습니다. controller는 감독하는 runner들의 결과를 모으고, runner는 자기 범위의 결과를 반환합니다. CLI는 Kubernetes API, kubeconfig, `kubectl`, Docker를 사용하지 않습니다. Kubernetes 저장소와 runner 위치를 해석하는 일은 controller가 담당합니다.

```sh
export WAFL_ENDPOINT=http://127.0.0.1:8082
wafl app add app.yaml
wafl app get @hello
wafl app list
wafl app patch @hello --file patch.yaml
wafl app logs -f @hello
wafl app erase @hello
wafl runner list
wafl instance list
wafl reservation list

# 다른 tenant 또는 개별 runner
wafl --endpoint https://control.example.com --tenant demo app get @hello
wafl --endpoint http://runner.example.com:8081 instance list
```

App과 Traffic 명령의 기본 endpoint는 `http://127.0.0.1:8082`, runner 명령은 `http://127.0.0.1:8081`입니다. `--endpoint` 또는 `WAFL_ENDPOINT`가 있으면 그 주소를 사용합니다. 관리 API는 운영자가 접근 가능한 주소로 제공합니다. 앱 트래픽을 받는 Traefik 주소와 관리 API 주소를 구분합니다.

## Runner·Instance·Reservation 집계

controller는 `--system-namespace`에 속한 virtual Node의 runner를 조회합니다. 준비 중이거나 drain 중인 runner도 조회 대상입니다. 이 세 API는 해당 runner들이 호스팅하는 모든 tenant의 운영 정보를 반환합니다. App API의 `wafl-tenant` 필터를 이 목록에 적용하지 않습니다. 별도 리소스나 CRUD RPC를 만들지 않고 기존 entity에서 생성한 `Get/List`와 수명 관리 RPC를 그대로 제공합니다.

```sh
export WAFL_ENDPOINT=http://127.0.0.1:8082
wafl runner list
# 출력에서 ID와 @alias를 선택
wafl runner get @wafl-runner-example
wafl runner get "$runner_id"
wafl --session "$runner_id" instance list
wafl instance get "$instance_id"
wafl instance logs -f "$instance_id"
wafl reservation get "$address"

# 목록 데이터는 stdout, 연결 경고는 stderr
wafl --json instance list >instances.jsonl 2>runner-warnings.log
```

runner에 직접 연결하면 `runner get`의 기본 `@self`와 `runner list`는 현재 프로세스 하나를 반환합니다. controller에서는 `runner list`에서 얻은 ID 또는 virtual Node 이름인 `@alias`를 지정합니다. `@self`는 controller에서 사용할 수 없습니다. `Runner.id`는 재시작마다 바뀌는 프로세스 세션이고 `Instance.session`과 일치합니다. controller의 `Runner.control`은 controller가 접속한 runner URL입니다. 이 내부 주소에 CLI가 직접 접속할 필요는 없습니다.

연결되지 않거나 제한 시간 안에 응답하지 않는 runner는 controller 로그와 응답의 `wafl-warning` metadata에 경고를 남깁니다. CLI는 경고를 stderr에 출력합니다. 다른 runner의 결과와 이미 받은 페이지는 계속 반환하며 목록 명령의 종료 코드는 0입니다. **모든 runner가 실패해도 빈 목록과 경고를 반환합니다.** 빈 결과만으로 전체 자원이 없다고 판단하지 마세요. Kubernetes에서 runner 목록 자체를 읽지 못하거나 사용자가 요청을 취소한 경우는 요청 오류입니다.

`wafl serve controller --runner-timeout 3s --runner-parallel 16`이 기본 설정입니다. 제한 시간은 runner 하나의 세션 확인과 페이지 조회 전체에 적용하고, 동시에 조회하는 runner 수를 제한합니다. `logs -f`의 수명은 이 조회 제한 시간과 무관하며 클라이언트 취소 시 runner 스트림도 닫습니다.

목록은 ID 순으로 합쳐 페이지를 나누며 `reverse`를 지원합니다. 같은 Instance ID가 여러 runner에 있으면 세션으로 순서를 구분하고 모두 반환합니다. 같은 영속 포트 풀을 공유하는 runner들의 Reservation은 ID로 중복을 제거합니다. 서로 다른 관측값은 경고하며, 충돌하는 `Reservation.Get`은 다시 조회하거나 `wafl-session`으로 범위를 지정하도록 오류를 반환합니다. token을 임의로 최신 값으로 바꾸지 않습니다.

페이지는 실시간 관측이며 고정된 snapshot이 아닙니다. 장애 복구 후 누락 항목까지 확인하려면 새 목록 조회를 시작합니다. controller 커서는 API 종류·감독 namespace·필터·선택 세션·정렬 방향에 묶이며 runner 커서와 교환할 수 없습니다.

## 특정 실행의 조회와 명령

`Get`과 `Tail`은 일치하는 실행 하나를 찾아 같은 세션에 요청합니다. 여러 runner에서 같은 alias나 ID를 찾으면 `--session`으로 구분해야 합니다. 대상을 찾지 못했고 조회 실패가 있었다면 `UNAVAILABLE`이며, 실패를 `NOT_FOUND`로 바꾸지 않습니다. 선택한 runner가 정상 응답하여 대상이 없음을 확인한 경우에는 다른 runner의 장애와 무관하게 `NOT_FOUND`입니다.

```sh
# runner_id는 runner list에서 선택한 현재 Runner ID
wafl --session "$runner_id" instance get @worker
wafl --session "$runner_id" instance start @worker
wafl --session "$runner_id" --revision "$revision" instance stop "$instance_id"

# 특정 runner에 저수준 실행 추가: App 배포와는 별도
wafl --session "$runner_id" instance add https://files.example.com/job.tar.gz
```

controller는 Instance 명령의 대상을 한 번 정한 뒤 그 ID와 runner 세션에 고정해 한 번만 전달합니다. caller가 지정한 revision을 유지하며 실패한 명령을 다른 runner로 재시도하지 않습니다. alias로 변경 명령을 내릴 때는 `--session`이 필수입니다. ID 명령에서 생략한 세션은 controller가 조회한 값으로 고정합니다. runner의 `Get/List/Tail`도 지정된 세션이 재시작 전 값이면 `FAILED_PRECONDITION`을 반환합니다.

`Instance.Add/Seal`과 `Reservation.Release`는 controller에서 반드시 `wafl-session`으로 **요청을 받을 현재 runner**를 지정합니다. `Instance.Release`에는 기존대로 session과 revision이 모두 필요합니다. 예약의 과거 소유 세션을 현재 수신 세션으로 사용하지 않으며, token과 `withdrawn` 확인은 그대로 전달합니다. 회수 전 외부 경로·연결을 정리하는 절차는 [예약 복구 계약](reservations.md)을 따릅니다.

앱 배포·수량·자동 배치는 `wafl app`으로 관리합니다. `instance add`는 선택한 runner에만 실행을 만들며 Kubernetes App이나 Pod을 생성하지 않습니다. Kubernetes가 관리하는 Instance를 직접 중지·삭제하면 provider가 App의 선언에 따라 다시 실행할 수 있습니다.

## 생성 RPC

App의 CRUD는 entity 선언에서 생성한 `AppService.Add/Get/List/Patch/Apply/Erase`를 사용합니다. 로그용 `Tail`만 확장 RPC입니다. Go controller와 Rust runner의 계약은 같은 protobuf에서 생성합니다.

- `Add(AppAddRequest) App`: 선언과 쓰기 전용 `secret_values`를 받습니다. 이 저장소에서는 ID를 서버가 부여하므로 `id`는 생략합니다. 고정 `replicas=0`은 실행 중지 상태이며 CLI YAML에서 replicas/scaling을 모두 생략하면 CLI가 1을 지정합니다.
- `Get`과 `List`: App protobuf를 반환합니다. `status`는 준비된 실행 수와 적용 상태, `revision`은 선언의 버전입니다. Secret 값과 Kubernetes 리소스를 반환하지 않습니다. `List`는 생성 계약의 필터·커서·역순 조회를 지원합니다.
- `Patch`: 생성된 필드 setter 계약입니다. `Apply`는 기존 protobuf patch 문서를 적용하며, read-only 필드 변경을 거절합니다. 한 요청은 Kubernetes resourceVersion CAS로 반영합니다.
- `Erase`: 해당 App과 종속 배포를 삭제합니다. 실행의 drain과 포트 회수는 기존 계약대로 완료합니다.
- `Tail`: controller가 선택한 runner의 `InstanceService.Tail`을 중계합니다. 응답 metadata의 `wafl-instance`가 선택된 실행입니다. cursor로 재개할 때는 `--instance ID`를 함께 지정합니다. 스트림은 선택한 실행에 고정되며 replica나 배포 버전을 자동으로 넘나들지 않습니다.

`wafl-tenant` RPC metadata는 tenant alias 또는 ID를 선택합니다. 기본값은 `default`이고 Kubernetes 저장소가 namespace에 대응시킵니다. 요청의 `tenant`가 있으면 이 범위와 일치해야 합니다. 기존 Kubernetes App도 같은 API로 조회할 수 있으며 App ID는 Kubernetes UID에서 파생한 domain-10 UUID입니다. alias와 domain은 기존 선언을 가리키는 불변 식별자입니다.

`wafl-revision` metadata로 `Patch/Apply/Erase`를 조건부 실행할 수 있습니다. CLI의 merge-style YAML patch는 Get으로 읽은 버전을 함께 보내므로 동시 수정 내용을 덮어쓰지 않습니다. 충돌하면 다시 읽어 수정합니다. 예를 들어 `replicas: 0`, `scaling: null`, `secrets: []`도 명시적으로 전달합니다.

Traffic 조회와 운영 회수는 기존 `TrafficService.List/Close`를 사용합니다. `wafl traffic close ID --withdrawn`은 관측자와 그 연결들이 실제로 종료됐다는 운영자의 확인을 전달합니다. ConfigMap 이름을 알아야 할 필요가 없습니다.

## 파일과 Secret

CLI는 `secrets[].file`을 YAML 기준 상대 경로에서 읽고 파일 경로를 제거한 뒤 `secret_values`로 보냅니다. controller가 불변 Secret source를 만들고 App에는 참조만 저장합니다. `Add/Patch/Apply/Get/List` 응답에는 값이 없습니다. 파일 변경은 자동 감시하지 않으며 새 파일을 지정한 patch를 다시 제출합니다. 저장 도중 응답이 유실되어도 사용 중인 source를 지우지 않습니다. 참조되지 않는 source는 기존 Secret GC가 수거합니다.

로컬 `file:` 번들은 CLI가 해시를 계산한 뒤 `PUT /bundles/<64자리 SHA-256>`으로 controller에 스트리밍합니다. 최대 크기는 1 GiB입니다. controller는 운영자가 설정한 `--assets` 저장소로 중계하고, 원자적으로 게시된 번들의 `Asset` protobuf JSON을 반환합니다. 임시 파일은 해시 검증 전에는 공개되지 않습니다. HTTP(S) URL의 digest가 생략되면 controller가 고정하며, 다른 URL 형식에는 digest를 지정합니다.

업로드 저장소는 선택 사항입니다. local bootstrap은 공유 개발용 asset 서버를 설치합니다. 운영 환경에서는 공개한 HTTP(S) 번들 URL을 쓰거나, controller replicas가 공유하고 runner가 읽을 수 있는 같은 업로드 저장소를 설정합니다. 개발용 hostPath 서버의 보관 정책을 운영용 객체 저장소로 간주하지 않습니다.

## 설치는 운영자 도구

아직 controller가 없는 클러스터에 설치할 때는 별도 `wafl-kube bootstrap` 도구가 Kubernetes를 사용합니다. 이 도구는 앱 CLI 명령이 아닙니다.

```sh
bash scripts/build.sh
export PATH="$PWD/target/debug:$PATH"
# Docker와 kind가 필요한 로컬 플랫폼 설치
wafl-kube bootstrap dev
# 기존 클러스터에 운영자 kubeconfig로 설치
wafl-kube bootstrap --context production install
```

로컬 gateway는 기본 `:8080`, controller는 `:8082`입니다. 운영자용 kubeconfig는 클러스터 관리에만 사용하며, 이후 `wafl app`에는 controller 주소만 전달합니다. Kubernetes에 직접 CRD/Secret을 적용하는 운영자 작업도 계속 지원합니다.
