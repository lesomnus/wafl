# 첫 요청 기동과 부하 증감

[일반 TCP App](tcp.md)도 같은 증감 정책을 사용합니다. TCP의 `target`·`active`·`started`·`busyMs`는 요청 대신 연결 동시성·연결 시작·누적 연결 시간이며, 조용한 열린 연결도 활성입니다. 아래 HTTP 응답 코드와 plugin 경로는 HTTP/gRPC에 해당합니다.

먼저 [Kubernetes native 튜토리얼](kubernetes-native.md)로 플랫폼을 설치합니다. 로컬 전체 플랫폼은 `wafl-kube bootstrap dev`, 기존 클러스터는 `wafl-kube bootstrap install`을 사용합니다.

```yaml
alias: hello
domain: hello.localhost
manifest:
  bundle:
    url: file:./hello.tar.gz
  env:
    GREETING: Hello
readiness:
  http: /
scaling:
  min: 0
  max: 4
  target: 8
  upMs: 1000
  downMs: 10000
  idleMs: 60000
  waiting: 128
  waitMs: 30000
```

```sh
wafl app add app.yaml
kubectl get apps.wafl.opt.td,pods
curl -H 'Host: hello.localhost' http://127.0.0.1:8080/
```

`min: 0`이면 요청이 오기 전에는 실행을 만들지 않습니다. Traefik은 준비된 backend가 없을 때 activator로 보냅니다. activator는 대기 기록을 저장하고 Deployment의 replicas를 0에서 1로 바꾼 뒤 준비된 실행이 생길 때까지 요청을 잡아 둡니다. 동시에 여러 controller가 첫 요청을 받아도 resourceVersion 조건으로 첫 replica 요청을 합칩니다. 이후의 증가는 부하 조정 주기를 따릅니다.

| 필드 | 의미 |
| --- | --- |
| min / max | 최소·최대 실행 수 |
| target | 실행당 목표 요청 동시성 |
| upMs / downMs | 한 단계 증감 사이의 최소 간격 |
| idleMs | 새 요청이나 활성 요청 없이 0개까지 줄이기 전 대기 시간 |
| waiting | 앱 전체 cold-start 대기 요청 상한 |
| waitMs | 실행 준비를 기다리는 요청의 최대 시간 |

Traefik plugin은 실행별 gate에서 활성 요청·시작 횟수·누적 요청 시간을 집계해 약 500ms마다 보고합니다. controller는 동시성과 구간 평균 부하를 사용하며 한 번에 replica 하나씩 변경합니다. Kubernetes scheduler가 실제 runner를 선택합니다. runner 자체 Pod 개수는 이 정책이 늘리지 않습니다. 운영자는 runner Deployment를 별도로 확장합니다.

gRPC는 `manifest.protocol: h2c`와 필요한 경우 `readiness.grpc`를 지정합니다. 스트림은 닫힐 때까지 활성 요청입니다. `grpc-timeout`과 클라이언트 취소도 첫 요청 대기에 적용합니다. 대기 한도 초과는 HTTP 429/gRPC RESOURCE_EXHAUSTED, 기동 제한은 HTTP 504/gRPC DEADLINE_EXCEEDED입니다. 전달을 시작한 앱 요청을 자동 재전송하지 않습니다.

축소·rolling update·삭제 모두 gateway가 새 요청을 막고 기존 연결이 끝났음을 보고한 뒤 포트를 회수합니다. 유예 시간이 끝나면 WASM 실행은 종료하지만 주소 예약은 관측 확인까지 유지합니다. 관측 heartbeat가 끊기면 자동 축소/회수는 보수적으로 멈출 수 있습니다. 실제 gateway 프로세스와 연결 종료를 확인한 뒤 `wafl traffic list`, `wafl traffic close CONFIGMAP --withdrawn`을 사용합니다.
