# Kubernetes App provider

`wafl.opt.td/v1alpha1`의 namespaced `App`을 `kubectl apply`로 등록할 수 있습니다. controller 설정에 `providers.kubernetes`를 추가하면 해당 namespace의 리소스를 감시하여 기존 WAFL App·Placement·Instance 흐름으로 실행합니다. 설정을 생략하면 Kubernetes에 접속하지 않습니다. 같은 이미지와 `wafl serve controller` 명령을 사용합니다.

Kubernetes에는 앱의 선언과 상태 요약을, PostgreSQL에는 배치·트래픽 관측·회수 상태를 저장합니다. 요청 경로는 기존 **Traefik → WASM 소켓**, 0개에서 기동할 때는 **Traefik → activator → WASM 소켓**입니다. 요청마다 Kubernetes API에 접근하지 않습니다.

## 설치와 설정

기존 [controller·runner·Traefik 구성](autoscaling.md)에 provider를 추가합니다. 아래 파일은 CRD와 앱 감시 권한의 설치 예제입니다. PostgreSQL·runner·gateway Pod를 생성하는 전체 플랫폼 설치 파일은 아닙니다.

```sh
kubectl apply -f deploy/kubernetes/apps.json
kubectl apply -f deploy/kubernetes/rbac.yaml
```

`apps.json`은 protobuf descriptor에서 생성한 CRD이며 `kubectl`이 JSON을 그대로 읽습니다. 예제 RBAC는 `wafl-system/wafl-controller` ServiceAccount에 `demo` namespace의 App 조회·감시·patch, status patch와 Secret `get`을 허용합니다. provider는 App patch를 finalizer 변경에 사용하지만, RBAC 자체가 변경 가능한 필드를 제한하지는 않습니다. controller Pod의 `spec.serviceAccountName`을 `wafl-controller`로 지정합니다. 감시 namespace를 늘리려면 해당 namespace에도 Role/RoleBinding을 설치하세요.

controller의 기존 JSON 설정에 다음 필드를 추가합니다.

```json
{
  "providers": {
    "kubernetes": {
      "cluster": "development",
      "namespaces": {
        "demo": "00000000-0000-8000-8001-000000000001"
      },
      "interval_ms": 2000
    }
  }
}
```

이 블록은 전체 설정이 아닙니다. 기존 `database`, 개별 `runners`, `traffic` 등을 함께 유지합니다. **`traffic` 설정과 WAFL 미들웨어가 포함된 Traefik이 필요합니다.** 고정 replica CRD도 삭제 시 스트림과 포트를 자동 회수하기 위해 관측 gate를 사용합니다.

`cluster`는 이 Kubernetes 클러스터를 나타내는 운영자가 정한 고정 이름입니다. controller replicas 모두 같은 값을 사용하며 kubeconfig context 이름과는 별개입니다. `namespaces`는 감시 범위와 **기존 WAFL Tenant UUID**의 명시적인 매핑입니다. 위 UUID는 controller가 초기화하는 로컬 개발 tenant입니다. tenant가 없으면 controller 시작이 실패합니다. namespace·tenant를 자동 발견하거나 생성하지 않습니다.

Pod에서는 ServiceAccount의 자격 증명을 사용합니다. 클러스터 밖에서는 `KUBECONFIG` 또는 provider의 `kubeconfig`와 `context` 옵션을 사용할 수 있습니다. 파일 경로는 controller 프로세스 기준입니다.

```json
"kubeconfig": "/home/me/.kube/config",
"context": "kind-wafl"
```

기존 DB에 provider를 처음 추가할 때는 모든 controller replicas를 종료한 뒤 한 개를 다음과 같이 시작합니다. 나머지는 같은 공유 설정으로 `--configure` 없이 시작합니다. kubeconfig 경로·context·상태 갱신 주기는 replica마다 달라도 됩니다.

```sh
wafl serve controller --config controller.json --configure
```

`cluster`와 tenant 매핑은 기존 앱이 있는 동안 변경하지 않는 계약입니다. UID가 같아도 소유 tenant가 바뀌는 변경은 거절합니다. 다른 클러스터 이름으로 바꾸려면 먼저 기존 provider로 앱을 정상 삭제하고 새로 등록하세요.

## 앱 등록·조회·수정

[예제 App](../deploy/kubernetes/app.yaml)의 `spec.domain`과 `manifest.bundle.url`을 실제 값으로 수정합니다. `wafl.opt.td`는 **Kubernetes API 그룹**이고, 앱의 HTTP Host는 `spec.domain`으로 따로 정합니다. DNS는 gateway를 가리키게 설정합니다.

```sh
kubectl apply -f deploy/kubernetes/app.yaml
kubectl get apps.wafl.opt.td -n demo
kubectl describe app.wafl.opt.td hello -n demo
kubectl wait -n demo --for=condition=Accepted app.wafl.opt.td/hello --timeout=60s
curl -H 'Host: hello.example.com' http://<gateway-address>/
```

`spec`은 App의 `name`, `domain`, `manifest`, `replicas`, `scaling`, `secrets`를 받습니다. `manifest.port` 기본값은 8080, `scaling`이 없으면 기본 replica는 1입니다. CRD는 protobuf JSON의 **camelCase**를 사용합니다(`memoryBytes`, `idleMs` 등). 기존 `wafl app add app.yaml` 형식에는 `apiVersion/kind/metadata/spec`가 없으므로 두 파일 형식을 구분하세요.

`status.app`은 WAFL App UUID, `status.alias`는 CLI 조회용 alias입니다. 별도의 UUID나 alias를 YAML에서 정할 필요가 없습니다.

```sh
alias=$(kubectl get app.wafl.opt.td hello -n demo -o jsonpath='{.status.alias}')
wafl --endpoint http://<controller>:8082 --json app get "@$alias"
wafl --endpoint http://<controller>:8082 placement list --app "@$alias"
kubectl patch app.wafl.opt.td hello -n demo --type merge \
  -p '{"spec":{"manifest":{"env":{"GREETING":"Updated"}}}}'
```

CRD가 소유한 App은 WAFL `Patch`·`Apply`·`Erase`로 변경할 수 없습니다. Kubernetes에서 수정·삭제합니다. CLI로 등록한 일반 App은 기존 방식으로 계속 사용할 수 있습니다. 도메인은 App 생성 뒤 변경할 수 없고 다른 App과 중복될 수도 없습니다. 도메인 중복 등 WAFL 검증 실패는 `Accepted=False`와 `ReconcileFailed` 사유로 status에 표시됩니다.

`status.observedGeneration`은 PostgreSQL에 반영한 spec 세대입니다. `desired`와 `ready`는 목표 실행 수와 현재 revision의 정상 실행 수입니다. `Ready=True`는 목표 상태에 도달했다는 뜻이며, 목표 0개이면 `ScaledToZero` 사유로 True입니다. 첫 요청 기동은 별도 동작입니다. 전환 시 condition의 `observedGeneration`도 현재 `metadata.generation`과 비교하세요.

## 사용 시나리오

**간헐적 HTTP 앱:** 예제처럼 `scaling.min: 0`을 사용합니다. 첫 요청은 기동을 기다리고 이후 요청은 직결됩니다. 요청/스트림 동시성으로 증감하며 유휴 기간이 지나면 0개로 줄어듭니다. 고정 방식의 `replicas: 0`은 자동 기동하지 않습니다.

**상시 2개 실행:** `scaling`을 제거하고 `replicas: 2`를 넣습니다. merge patch에서는 필드를 실제로 지우도록 `null`을 사용합니다.

```sh
kubectl patch app.wafl.opt.td hello -n demo --type merge \
  -p '{"spec":{"scaling":null,"replicas":2}}'
```

**gRPC 스트리밍:** `manifest.protocol: h2c`로 HTTP/2 서버를 실행합니다. 배포 변경·삭제 중에도 이미 수락한 스트림은 끝까지 유지합니다. gateway timeout과 클라이언트 deadline은 서비스 특성에 맞게 정합니다.

**번들 갱신:** 첫 성공 실행의 digest를 WAFL App에 고정합니다. 같은 URL의 YAML에서 이름·메모리 등을 수정해도 이 digest를 보존합니다. 배포할 코드를 바꾸려면 새 URL 또는 명시적인 새 digest를 선언하세요. URL의 파일만 덮어쓰고 같은 YAML을 재적용하는 것은 새 배포가 아닙니다. 번들을 다운로드하는 주체는 runner이고 `file:`도 runner의 로컬 경로입니다.

`spec.secrets`의 `env`와 `secretKeyRef`로 같은 namespace의 기존 Secret key를 환경 변수에 주입합니다. 값은 App/status에 저장하지 않으며 새 Instance부터 적용합니다. [파일·Kubernetes secret 튜토리얼](secrets.md)에 예제와 갱신 계약이 있습니다.

## 삭제·복구와 controller replicas

```sh
kubectl delete app.wafl.opt.td hello -n demo
```

`wafl.opt.td/app` finalizer가 자동 증감을 해제하고 목표를 0으로 바꿉니다. 기존 gateway의 gate 종료와 활성 요청 0개를 확인한 뒤 runner 실행·직결 포트를 회수하고 WAFL App을 삭제합니다. 그 후 finalizer를 제거하여 Kubernetes 삭제를 완료합니다. 고정 replica도 같은 과정을 따릅니다.

gateway/activator가 장애로 마지막 종료 보고를 못 보내면 삭제는 `Terminating`에 남습니다. heartbeat timeout만으로 기존 스트림이 종료됐다고 추정하지 않습니다. 해당 프로세스와 연결이 실제 종료되었는지 확인한 뒤 [Traffic 복구 절차](autoscaling.md#안전한-축소와-복구)의 `wafl traffic close <UUID> --withdrawn`을 사용하세요. finalizer 강제 제거는 WAFL 정리 절차를 건너뜁니다.

controller 여러 개가 같은 namespace를 감시할 수 있습니다. PostgreSQL 트랜잭션과 CRD UID·generation으로 중복 반영과 오래된 spec 적용을 막고, status/finalizer patch는 Kubernetes resourceVersion으로 충돌을 확인합니다. 재시작하면 LIST/WATCH로 기존 리소스를 다시 읽습니다. 삭제 후 같은 namespace/name을 다시 만들면 새 UID에 대응하는 새 WAFL App이 됩니다.

`AppSource`는 Payday entity로 선언된 내부 소유권 기록입니다. App 삭제 후에도 삭제 표식을 보관하여 지연된 이전 reconcile이 앱을 재생성하지 못하게 합니다. 이 기록의 자동 GC는 없습니다. provider를 끄거나 감시 namespace를 제거하면 기존 App은 남고 finalizer 처리도 중단되므로, 제거 전에 기존 리소스를 정상 삭제해야 합니다. Kubernetes API 일시 장애 중에는 PostgreSQL에 마지막으로 반영한 앱과 기존 데이터 경로가 계속 동작합니다.

이번 provider는 App 선언을 읽습니다. runner Pod 생성·증감·자동 발견은 담당하지 않습니다. controller의 `runners`에는 개별 runner의 안정적인 관리 주소를, `direct.advertise`에는 gateway가 접근할 해당 runner의 IPv4를 계속 설정해야 합니다. 여러 runner를 분산 Service 주소 하나로 숨기지 않습니다. [직결 운영 계약](direct.md)을 참고하세요.

## 검증

CI는 일회용 kind 클러스터에 실제 CRD를 설치하고 controller 2개, PostgreSQL, Wasmtime runner, Traefik을 연결합니다. CRD 기본값·검증, 초기 LIST와 재시작, namespace 범위, 중복 방지, 변경 반영, 열린 gRPC 스트림을 기다리는 삭제, 동일 이름 재생성, 0개에서 첫 요청 기동을 확인합니다.

로컬 실행은 **테스트 전용 클러스터**의 kubeconfig를 명시합니다. 이 테스트는 CRD와 임시 namespace를 생성하며 개발자의 현재 kubeconfig를 자동 선택하지 않습니다.

```sh
WAFL_TEST_KUBECONFIG=/tmp/wafl-test-kubeconfig \
WAFL_TEST_CONTROLLER_DATABASE_URL=postgresql://postgres:wafl-test@localhost:5432/wafl \
WAFL_TEST_TRAEFIK=/path/to/traefik \
  bash scripts/test.sh kubernetes -- --include-ignored
```
