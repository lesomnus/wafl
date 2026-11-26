# Controller와 Virtual Kubelet

플랫폼의 설치·로컬 개발은 [Kubernetes native 튜토리얼](kubernetes-native.md)을 따릅니다. PostgreSQL 기반 controller와 별도 runner 목록 설정은 제거했습니다.

```mermaid
flowchart LR
  CLI[wafl app] --> RPC[Controller AppService]
  RPC --> API[Kubernetes App CRD]
  Operator[kubectl / bootstrap] --> API
  API --> C[WAFL controller]
  C --> D[Deployment / ReplicaSet]
  D --> P[논리 Pod]
  P --> S[Kubernetes scheduler]
  S --> V[runner별 Virtual Kubelet]
  V --> R[한 runner의 여러 WASM Instance]
  T[Traefik + WAFL plugin] --> R
  T --> A[첫 요청 activator]
  A --> R
  C -->|동적 경로| T
```

controller는 App에서 Deployment를 생성하고 Secret을 불변 snapshot으로 변환합니다. Kubernetes scheduler가 가상 Node를 선택하고 ReplicaSet이 실행 개수를 유지합니다. Virtual Kubelet provider는 생성된 Instance RPC로 runner를 제어하고 readiness와 종료 상태를 Pod status로 보고합니다. 앱마다 Linux 컨테이너를 새로 띄우지 않습니다.

controller는 여러 replica로 실행할 수 있습니다. Kubernetes Lease 소유자만 배포·증감·회수를 조정하고 모든 replica가 App/Traffic RPC, 로그 중계, Traefik 설정과 activator 요청을 처리합니다. API 변경은 resourceVersion으로 충돌을 확인합니다. 조정 leader가 바뀌어도 App·Deployment·관측 기록·실행 receipt는 Kubernetes에 남습니다.

| 구성요소 | 기본 주소 / 명령 |
| --- | --- |
| controller | `wafl serve controller`, `:8082`의 `/health`, `/traefik`, 생성된 AppService/TrafficService RPC |
| activator | 같은 프로세스의 `:8083`, 앱 HTTP/gRPC 요청만 처리 |
| node provider | `wafl serve node --node NAME --runner http://RUNNER:8081` |
| runner | `wafl serve runner`, 생성된 Instance/Runner/Reservation RPC |
| gateway | 플러그인이 포함된 Traefik, `Host` 규칙과 guest IP:port 사용 |

controller의 `--control`, `--activate`에는 gateway에서 접근할 수 있는 내부 주소를 사용합니다. 설치 파일은 Kubernetes Service 주소를 설정합니다. App Service도 Traefik을 통하므로 내부 호출과 외부 호출이 같은 수신·drain 계약을 따릅니다. 준비된 앱의 요청마다 Kubernetes API를 호출하지는 않습니다.

가상 Node와 논리 Pod는 WAFL의 제한된 실행 계약입니다. 일반 kubelet의 Linux 컨테이너·exec·CSI·Pod별 CNI를 구현하지 않습니다. 상세 지원 범위와 장애 후 예약 회수는 [플랫폼 계약](kubernetes-native.md)에 있습니다.

클라이언트 계약과 Secret·번들 전달은 [controller API](controller-api.md)에 있습니다. Kubernetes credentials는 controller/node 서버와 별도 운영자 도구만 사용합니다. `AppService.Tail`은 runner의 스트림을 중계하며 클라이언트는 runner Pod IP나 Kubernetes port-forward를 알 필요가 없습니다.
