# Controller API와 CLI

`wafl app`과 `wafl traffic`은 controller API만 호출합니다. `wafl instance`, `wafl runner`, `wafl reservation`은 runner API를 호출합니다. CLI는 Kubernetes API, kubeconfig, `kubectl`, Docker를 사용하지 않습니다. Kubernetes 저장소와 runner 위치를 해석하는 일은 controller가 담당합니다.

```sh
export WAFL_ENDPOINT=http://127.0.0.1:8082
wafl app add app.yaml
wafl app get @hello
wafl app list
wafl app patch @hello --file patch.yaml
wafl app logs -f @hello
wafl app erase @hello

# 다른 tenant 또는 개별 runner
wafl --endpoint https://control.example.com --tenant demo app get @hello
wafl --endpoint http://runner.example.com:8081 instance list
```

App과 Traffic 명령의 기본 endpoint는 `http://127.0.0.1:8082`, runner 명령은 `http://127.0.0.1:8081`입니다. `--endpoint` 또는 `WAFL_ENDPOINT`가 있으면 그 주소를 사용합니다. 관리 API는 운영자가 접근 가능한 주소로 제공합니다. 앱 트래픽을 받는 Traefik 주소와 관리 API 주소를 구분합니다.

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
