# Kubernetes native WAFL

WAFL 플랫폼은 Kubernetes를 필수로 사용합니다. 앱은 일반 HTTP/gRPC·TCP 서버이며 별도 SDK가 필요 없습니다. 단독 `wafl serve runner`와 `wafl instance` 명령은 Kubernetes와 `wafl-kube` 없이 계속 사용할 수 있습니다.

## 책임과 실행 단위

App CRD → Deployment → ReplicaSet → Pod → Virtual Kubelet provider → runner Instance 순서로 실행합니다. runner 프로세스 하나가 여러 WASM Instance를 실행합니다. 앱 Pod는 컨테이너 프로세스가 추가되는 Pod가 아니라 실행 상태를 나타내는 Kubernetes 리소스입니다. 실제 runner Pod 하나마다 가상 Node 하나가 등록됩니다.

Kubernetes가 복제본 유지와 rolling update를 담당하고, WAFL이 WASM 실행·readiness·도메인 라우팅·첫 요청 대기·요청 관측과 drain을 담당합니다. controller replicas의 조정 작업은 Kubernetes Lease로 leader를 선출하며, 모든 replica가 요청과 관측 API를 처리할 수 있습니다. PostgreSQL은 플랫폼 상태 저장소로 사용하지 않습니다. 앱이 사용하는 DB는 별개입니다.

## 로컬 개발과 설치

`wafl app`·`wafl traffic`은 controller API를 호출하고, `wafl instance`·`wafl runner`·`wafl reservation`은 controller의 집계 API 또는 개별 runner API를 호출합니다. 앱 사용자에게 Kubernetes 접근 권한을 요구하지 않습니다. 설치 전에 controller가 존재할 수 없으므로 최초 클러스터 설치는 별도 운영자 도구가 맡습니다. [API 계약](controller-api.md)을 참고하세요.

앱 코드는 기존 언어 도구로 실행하고 디버깅합니다. WASM 번들만 확인하려면 단독 runner를 사용합니다. 전체 플랫폼을 로컬에서 검증할 때는 Docker와 kind를 설치하고 `wafl-kube bootstrap dev`를 실행합니다. 이 명령은 전용 kind 클러스터를 만들거나 재사용하고 controller·runner·Traefik을 설치합니다. 클러스터는 명령 종료 뒤에도 유지됩니다.

```sh
bash scripts/build.sh
export PATH="$PWD/target/debug:$PATH"
bash scripts/bundle.sh "$PWD/examples/local/hello.tar.gz"
wafl-kube bootstrap dev examples/local/app.yaml
# CLI에는 출력된 controller 주소만 필요합니다.
export WAFL_ENDPOINT="http://127.0.0.1:8082"
wafl app get @hello
curl -H 'Host: hello.localhost' http://127.0.0.1:8080/
```

다른 Kubernetes 클러스터에는 `wafl-kube bootstrap install`로 설치합니다. 클러스터당 WAFL 플랫폼 하나를 설치하고 controller replicas를 늘려 가용성을 확보합니다. 기본 인프라 namespace는 `wafl-system`입니다. 게이트웨이 Service는 NodePort 30080을 사용하며 운영 환경에 맞게 LoadBalancer나 기존 외부 프록시에 연결할 수 있습니다. CLI가 이미지를 내려받아 호스트에서 실행하지는 않습니다. 설치한 Deployment가 이미지를 사용합니다.

```sh
wafl-kube bootstrap --context production install
# 운영자가 제공한 controller 관리 API 주소로 연결합니다.
wafl --endpoint https://control.example.com --tenant demo app add app.yaml
```

[일반 TCP 튜토리얼](tcp.md)은 `domain` 대신 공개 포트를 사용하는 App, 복수 소켓과 TCP 연결 단위 증감을 설명합니다.

## App 예제

```yaml
alias: hello
domain: hello.localhost
manifest:
  bundle:
    url: file:./hello.tar.gz
  port: 8080
  env:
    GREETING: Hello
readiness:
  http: /
  periodSeconds: 1
  timeoutSeconds: 1
  failures: 3
scaling:
  min: 0
  max: 4
  target: 8
  idleMs: 60000
rollout:
  surge: 1
  unavailable: 0
  drainMs: 30000
secrets:
  - env: DATABASE_URL
    file: ./database-url
```

`wafl app add`는 로컬 파일 번들의 digest를 계산한 뒤 controller의 업로드 API로 보냅니다. 로컬 bootstrap은 업로드용 asset 서버를 설정합니다. HTTP(S) URL에 digest가 없으면 controller가 다운로드해 digest를 고정합니다. 원격 클러스터에서는 runner가 접근할 수 있는 HTTP(S) URL을 사용하세요. Kubernetes에 직접 제출하는 App에는 `manifest.bundle.digest`가 필요합니다. mutable URL의 내용을 바꿔도 기존 배포의 실행 코드가 달라지지 않습니다.

일반 YAML의 secret 파일은 **CLI가 YAML 파일 기준 상대 경로에서 읽어** 쓰기 전용 값으로 controller에 보냅니다. controller가 Kubernetes Secret을 저장하며 로컬 경로를 읽지는 않습니다. Kubernetes App의 `spec.secrets`에서는 같은 namespace의 `secretKeyRef`를 사용합니다. 파일의 개행을 포함한 UTF-8 내용을 그대로 전달합니다.

controller는 source Secret의 UID/resourceVersion과 값을 한 번 읽고 불변 Secret snapshot을 만듭니다. source 변경은 새 Pod template을 만들며 이전 ReplicaSet은 이전 snapshot을 계속 참조합니다. 보관된 ReplicaSet·Pod·Deployment 또는 App source가 참조하지 않는 오래된 snapshot은 GC합니다. 값은 App·Pod annotation·Instance 응답에 기록하지 않습니다. 원래 Kubernetes Secret과 snapshot은 Kubernetes Secret의 저장·접근 정책을 따릅니다.

## 배포와 종료

App의 manifest, readiness, secret source 변경은 Deployment의 Pod template에 반영됩니다. 기본 전략은 `maxSurge: 1`, `maxUnavailable: 0`입니다. 새 Pod가 ready가 되기 전까지 이전 Pod는 계속 요청을 처리합니다. 준비에 실패한 새 버전이 기존 정상 버전을 먼저 철회하지 않습니다.

readiness를 생략하면 TCP listen을 확인합니다. HTTP path 또는 gRPC Health service를 지정하면 앱 준비 상태를 확인합니다. gRPC 예시는 `readiness: {grpc: ""}`입니다. 빈 service 이름은 gRPC Health의 전체 서버 상태입니다.

삭제되는 Pod는 새 요청을 받지 않고 기존 요청·스트림을 마무리합니다. 종료 유예 시간이 지나면 runner의 `Halt`가 실행을 종료하지만 **주소 예약은 유지**합니다. 모든 gateway observer가 닫혔고 활성 요청이 0임을 확인한 뒤 `Release`와 `Erase`를 수행합니다. 누락된 heartbeat는 연결 종료의 증거로 사용하지 않습니다. 관측이 유실되면 Pod finalizer 또는 실행 receipt가 남을 수 있으며, 해당 gateway와 연결이 실제로 종료됐음을 확인한 뒤 `wafl traffic close ID --withdrawn`으로 회수합니다.

Traefik은 준비된 guest socket으로 직접 요청을 전달합니다. App용 ClusterIP Service도 Traefik을 통해 같은 관측 경로를 사용합니다. 정상 실행 중인 앱에 대한 요청마다 Kubernetes API를 호출하지 않습니다. guest 주소로 직접 요청하면 이 drain 계약에 포함되지 않으므로 플랫폼 앱 트래픽은 gateway 또는 App Service를 사용해야 합니다.

## 지원하는 Pod 계약

provider는 App에서 생성한 단일 WASM 실행만 지원합니다. Linux image, exec, sidecar/init container, PVC/CSI, host namespace, Linux securityContext, CPU limit은 지원하지 않습니다. Pod template에 이러한 설정을 넣으면 실행을 거절합니다. 번들은 기존 WAFL tar.gz 규격을 사용하며 내부 image 문자열은 provider가 식별하는 불변 실행 참조입니다.

가상 Node의 OS/architecture label은 `wasi`/`wasm32`입니다. Linux Node로 표시하지 않으므로 Linux용 kube-proxy 같은 DaemonSet이 배정되지 않습니다. 추가 시스템 DaemonSet도 실제로 지원하는 Node의 OS/architecture를 선택하도록 설정합니다.

논리적인 앱 Pod에는 독립 Pod IP를 할당하지 않습니다. 실제 실행 주소는 runner의 IP와 예약한 포트이며 WAFL이 Endpoint 정보를 관리합니다. 일반 CNI/NetworkPolicy가 앱 Pod마다 격리해 주지는 않습니다. 격리는 runner Pod 및 WASM sandbox 단위입니다. `kubectl logs/exec`의 일반 kubelet API는 제공하지 않으며 로그는 `wafl app logs` 또는 `wafl instance logs`로 조회합니다. 두 명령 모두 controller 주소를 사용할 수 있습니다.

runner Pod가 사라져도 이전 관측자들의 종료 확인 전에는 해당 가상 Node의 주소 소유권을 보존합니다. CNI가 그 IP를 새 Pod에 재사용하면 새 provider는 이전 소유권이 정리될 때까지 실행 배정을 기다립니다. 가상 Node를 임의로 삭제하거나 포트 예약을 비우면 이 보호를 우회합니다.

가상 Node의 `wafl.opt.td/slots`, `wafl.opt.td/memory`는 예약할 실행 수와 linear-memory 상한 합계입니다. 컨테이너 전체 RSS/CPU 한도와 같지 않습니다. runner Pod의 Kubernetes resources는 별도로 설정합니다. 실행 실패는 sandbox 실패로 보고하며 10초의 대기 후 ReplicaSet이 새 Pod를 만듭니다. 일반 컨테이너의 모든 restartPolicy 동작을 모방하지 않습니다.

## 운영 예시

고정 실행 수가 필요하면 `scaling`을 생략하고 `replicas: 2`를 사용합니다. 자동 증감으로 바꾸려면 `replicas`를 제거하거나 0으로 설정하고 `scaling`을 추가합니다. CRD의 `status.deployment`와 `status.service`로 생성한 리소스 이름을 확인할 수 있습니다.

```sh
wafl app get hello
kubectl get app hello -o jsonpath='{.status.deployment}'
# 앱의 Secret 교체: 같은 namespace의 source Secret을 갱신합니다.
kubectl -n demo create secret generic database --from-file=url=./database-url \
  --dry-run=client -o yaml | kubectl apply -f -
kubectl -n demo get pods -w
```

새 bundle을 배포할 때는 URL과 digest를 함께 바꿉니다. readiness가 실패하면 기존 정상 실행이 유지됩니다. declaration을 수정할 때는 App을 수정하세요. 생성된 Deployment를 직접 변경하면 다음 reconcile이 App의 선언으로 되돌립니다.

runner를 늘리는 작업은 `kubectl -n wafl-system scale deployment/wafl-runner --replicas=3`입니다. runner Pod의 종료 hook은 먼저 가상 Node를 cordon하고 실행들을 drain합니다. 예제 runner Pod의 종료 유예는 60초이므로 더 긴 앱 스트림을 정상 완료시키려면 Pod 유예와 hook 대기 시간도 함께 조정해야 합니다. 강제 종료나 host 장애 시 진행 중이던 요청 자체를 재실행하지는 않습니다.

runner 이미지 교체는 앱 복제본이 2개 이상이어도 모든 실행의 재기동이 겹칠 수 있습니다. runner Pod의 readiness는 앱의 readiness와 다릅니다.
준비된 실행이 일시적으로 없으면 고정 복제본 App도 activator에서 새 실행을 기다립니다. 기본 대기 한도는 30초이며 gRPC deadline이나 클라이언트 취소가 먼저 발생하면 대기를 끝냅니다.
이는 짧은 응답 시간을 보장하는 업데이트가 아닙니다. `replicas: 0`으로 중지한 고정 복제본 App은 요청으로 기동하지 않습니다.

`wafl-kube bootstrap dev --load --image LOCAL_TAG --gateway-image GATEWAY_TAG`는 로컬 Docker 이미지 두 개를 kind에 적재합니다. 같은 태그를 다시 빌드한 경우 image ID가 바뀌면 새 Pod template으로 갱신합니다. 로컬 클러스터 삭제는 `kind delete cluster --name wafl`입니다. 원격 Docker daemon을 사용하는 개발 환경은 `DOCKER_HOST` 주소로 접속하며 kubeconfig의 TLS 검증을 유지합니다.

## 이전 버전에서 전환

기존 PostgreSQL controller와 새 Kubernetes controller가 같은 runner를 동시에 관리하지 않도록 분리합니다. 기존 플랫폼에서 App YAML·secret 참조를 내보내고 새 클러스터에 등록하여 검증한 뒤 도메인의 게이트웨이를 전환합니다. 기존 요청을 drain한 뒤 이전 controller와 runner를 종료합니다. 기존 App/Placement UUID를 새 Pod의 UID로 재사용하지 않습니다.

기존 controller JSON·`--configure`·PlacementService 관리 API는 새 제어평면에서 제공하지 않습니다. `AppService`는 Kubernetes 저장소를 사용하는 controller API로 제공하며 `wafl app`은 endpoint/tenant만 사용합니다. kubeconfig와 namespace 기반 CLI 명령은 제거했고 설치 명령은 운영자 도구 `wafl-kube bootstrap install/dev`로 분리했습니다. `wafl instance`, `wafl runner`, `wafl reservation`은 같은 RPC로 controller의 집계 범위 또는 개별 runner 범위를 선택합니다. App 수정은 `kubectl apply` 또는 `wafl app patch NAME --file patch.yaml`로 수행합니다. 자동 migration이나 기존 PostgreSQL 데이터 삭제는 하지 않습니다.

## 실행 수명 통합 검증

Docker·kind·Rust·Go·kubectl이 있는 환경에서 다음 명령으로 전용 테스트 클러스터를 만듭니다.
현재 Kubernetes context를 사용하지 않습니다. CI도 같은 스크립트를 실행합니다.

```sh
bash scripts/test-kubernetes.sh
```

[기본 통합 테스트](../kubernetes/tests/native_test.go)는 실제 controller·Virtual Kubelet provider·Rust runner·WASM·Traefik을 사용합니다.
readiness 실패 시 기존 실행 유지, Secret 교체 중 gRPC 스트림 보존, 첫 요청 기동·부하 확장·유휴 회수, runner 장애 복구,
provider 재시작과 controller 리더 교체를 검사합니다.

[업데이트·runner 수명 테스트](../kubernetes/tests/lifecycle_test.go)는 다음을 추가로 검사합니다.

| 시나리오 | 확인 |
|---|---|
| 배정 전 취소 | runner 용량을 초과해 Pending인 Pod를 0개로 축소했을 때 provider 없이도 finalizer와 Pod가 정리되는지 확인합니다 |
| WASM A → B → A | 컴파일 시 버전이 다른 실행 파일과 tar.gz digest를 사용합니다. 새 요청의 응답에서 바이너리를 식별하고 기존 gRPC 스트림이 정상 종료된 뒤 이전 실행과 receipt가 제거되는지 확인합니다 |
| runner 2 → 3 → 2 | 실제 Deployment의 replicas를 바꿉니다. 가상 Node가 늘고 새 runner에 WASM이 배정되며, 축소할 때 기존 스트림을 완료하고 실행·가상 Node를 회수하는지 확인합니다 |
| runner/provider 이미지 교체 | 게시된 `2e463666` 이미지의 고정 digest에서 현재 소스로 빌드한 이미지로 두 컨테이너를 교체합니다. 실제 image ID 변경·앱 재배정·기존 스트림 완료·이전 실행 회수를 확인합니다 |

각 전환 동안 별도의 클라이언트가 요청을 계속 보내고 응답 본문과 gRPC trailer를 검사합니다. 오류를 재시도로 감추지 않으며,
요청 수·오류 수·최대 응답 시간을 테스트 로그에 남깁니다. 요청의 gRPC deadline은 기본 기동 대기 한도에 맞춘 30초입니다.
runner 증감·이미지 교체는 복제본을 서로 다른 runner에 배정하고 여유 용량을 둔 조건입니다.
단일 복제본이나 유예 시간을 넘기는 스트림, 임의의 과거 버전 조합에 대한 무중단 보장을 뜻하지 않습니다.

기본 baseline 이미지의 digest는 [테스트 스크립트](../scripts/test-kubernetes.sh)에 고정되어 있습니다.
테스트가 끝나면 `kind delete cluster --name wafl-test`로 전용 클러스터를 정리합니다.
