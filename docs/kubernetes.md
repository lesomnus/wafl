# Kubernetes App

Kubernetes는 WAFL 플랫폼의 필수 제어평면입니다. 이전 선택적 App importer와 PostgreSQL 상태 저장소를 대체했습니다. 설치와 전체 예제는 [Kubernetes native 튜토리얼](kubernetes-native.md)을 참고하세요.

```sh
wafl install
kubectl create namespace demo
# URL과 압축 파일의 SHA-256 digest를 수정한 뒤 제출합니다.
kubectl apply -f deploy/kubernetes/app.yaml
kubectl -n demo get apps.wafl.opt.td,deployments,pods
kubectl -n demo describe app hello
wafl --namespace demo app logs hello
```

App은 `wafl.opt.td/v1alpha1`의 namespaced CRD입니다. `spec`은 공통 protobuf 계약에서 생성하며 manifest·replicas 또는 scaling·secrets·readiness·rollout을 받습니다. 앱의 domain은 생성 후 바꿀 수 없습니다. 같은 domain을 여러 App이 요청하면 먼저 만들어진 App만 조정합니다.

`status.observedGeneration`과 condition의 `observedGeneration`을 현재 `metadata.generation`과 비교하세요. `Accepted=False`는 source 또는 선언을 적용할 수 없다는 뜻입니다. 이전 정상 Deployment는 유지됩니다. `Ready=True`는 최신 Deployment의 목표 개수와 준비 상태가 충족됐다는 뜻입니다. 0개인 자동 관리 App에는 `ScaledToZero` 사유를 표시합니다.

`kubectl delete app`은 소유한 Deployment·Pod를 정리합니다. provider의 Pod finalizer가 새 요청 중단, 기존 스트림 종료, 실행·포트 회수를 기다립니다. 강제 Pod 삭제나 provider 재시작에도 별도 실행 receipt로 회수 작업을 이어갑니다. 사용자 지정 finalizer를 수동 제거하면 이 절차를 우회하므로 운영 복구에는 [관측 회수 명령](kubernetes-native.md)을 사용하세요.
