# 일반 TCP 앱 실행하기

HTTP/gRPC 외의 TCP 서버도 WASIp2 소켓으로 실행할 수 있습니다. 앱은 전용 SDK 없이 listen/read/write하며, 한 runner 안의 여러 인스턴스가 같은 게스트 포트를 사용해도 실제 호스트 포트는 각각 배정됩니다. 현재 UDP는 지원하지 않으며 선언 시 거부합니다.

## 소켓과 공개 포트

`Manifest.sockets`는 앱이 여는 소켓, `App.listeners`는 외부에 공개할 포트입니다. 아래 예제는 두 TCP 소켓을 엽니다.

```yaml
alias: echo
manifest:
  bundle:
    url: file:./tcp.tar.gz
  sockets:
    - name: echo
      port: 8080
      transport: tcp
    - name: admin
      port: 8081
      transport: tcp
  env:
    GREETING: hello
    SECOND_PORT: "8081"
listeners:
  - socket: echo
    port: 10000
  - socket: admin
    port: 10001
scaling:
  min: 0
  max: 4
  target: 16
  idleMs: 60000
rollout:
  surge: 1
  unavailable: 0
  drainMs: 30000
```

| 값 | 의미 |
| --- | --- |
| `sockets[].name` | 고유한 이름, 소문자로 시작하고 영문 소문자·숫자·하이픈을 사용. 끝은 영문자/숫자, 최대 15자 |
| `sockets[].port` | 게스트가 listen하는 IPv4 포트, 1–65535. 인스턴스 내 중복 불가 |
| `sockets[].transport` | 현재 `tcp`만 지원, 생략해도 TCP |
| `sockets[].protocol` | 생략하면 바이트 스트림, `http`/`h2c`이면 HTTP 앱 |
| `listeners[].socket` | 연결할 소켓 이름 |
| `listeners[].port` | 플랫폼 전체에서 독점하는 공개 포트, 현재 10000–10015 |

TCP App은 `domain`을 생략합니다. 일반 TCP에는 Host 헤더가 없으므로 도메인을 읽어 앱을 구분하지 않습니다. DNS 이름을 붙여도 같은 IP의 TCP 앱 구분은 포트로 합니다. 암호화된 바이트도 해석하지 않고 전달하며, TLS를 쓰면 앱이 처리합니다. SNI 기반 포트 공유와 플랫폼의 TCP TLS 종료는 지원하지 않습니다.

최대 16개 소켓을 선언할 수 있습니다. TCP App은 모든 소켓을 `listeners`로 공개해야 하고, HTTP App은 하나의 `http` 또는 `h2c` 소켓과 `domain`을 사용합니다. HTTP 전용 라우팅과 일반 TCP 소켓을 한 App에 혼합하는 선언은 아직 거부합니다. `listeners`는 App 생성 후 변경할 수 없습니다. 중복 공개 포트는 먼저 생성한 App이 사용하고 다른 App은 `status.accepted: false`가 됩니다.

단일 소켓은 `manifest.port`/`protocol`로 선언할 수 있습니다. 소켓 목록이 없으면 기본값은 `8080`/`http`이며, `protocol: tcp`는 이름 `app`인 TCP 소켓 하나를 뜻합니다. 소켓 목록과 이 필드를 함께 쓰면 첫 소켓과 일치해야 합니다. `PORT`에는 첫 소켓 포트가 들어갑니다. 추가 소켓의 환경변수 이름은 앱이 정합니다. 위 `SECOND_PORT`는 예제 앱의 설정이며 플랫폼의 예약 변수는 아닙니다.

## 로컬 실행과 배포

[`examples/tcp`](../examples/tcp/src/main.rs)는 서버가 먼저 `hello:8080` 같은 인사말을 보내고, 받은 바이트를 그대로 돌려줍니다. 클라이언트가 송신만 종료하면 서버는 `done`을 응답하고 종료합니다. 표준 라이브러리만 사용하는 작은 polling 예제이며 네이티브와 WASM으로 같은 소스를 빌드합니다.

```sh
cargo run --locked -p wafl-example-tcp
# 다른 터미널: 먼저 서버 인사말이 출력됩니다.
nc 127.0.0.1 8080
```

네이티브 앱을 종료한 뒤 번들을 만들고 [로컬 플랫폼](kubernetes-native.md)을 설치합니다.

```sh
bash scripts/build.sh
export PATH="$PWD/target/debug:$PATH"
bash scripts/bundle.sh "$PWD/examples/tcp/tcp.tar.gz" tcp
wafl-kube bootstrap dev
export WAFL_ENDPOINT=http://127.0.0.1:8082
wafl app add examples/tcp/app.yaml
wafl app get @echo
nc 127.0.0.1 10000
nc 127.0.0.1 10001
```

`wafl`은 controller API만 사용합니다. `bootstrap dev`는 운영자 도구이며 kind 클러스터를 만들 때 공개 포트 10000–10015를 호스트에 연결합니다. `--tcp-port 22000`이면 호스트 22000–22015에 연결하되 App의 포트 선언은 10000–10015를 유지합니다. 포트 매핑은 클러스터 생성 시 결정하며, 같은 이름의 클러스터를 재사용하면 변경되지 않습니다.

원격 Docker를 사용하면 출력된 gateway 호스트에 연결합니다. 기존 Kubernetes에서는 proxy Service의 포트 10000–10015, NodePort 31000–31015를 운영 환경에 맞게 외부에 노출합니다. 공개 범위를 바꾸려면 현재는 설치 파일과 proxy의 포트 범위를 함께 변경해야 합니다.

## 첫 연결, 증감과 업데이트

```text
클라이언트 TCP 연결
  → wafl proxy의 포트별 TCP 수신
  → runner Pod의 배정된 포트
  → WASM 서버 소켓
```

프록시는 공개 포트로 App을 찾고 관측 gate를 등록합니다. 실행이 0개이면 대기 관측을 등록하고 controller에 Wake를 요청한 뒤 준비될 때까지 연결을 유지합니다. 여러 프록시의 동시 기동 요청은 controller가 Kubernetes resourceVersion으로 합칩니다. 서버가 먼저 말하는 프로토콜도 클라이언트 데이터를 기다리지 않고 실행을 시작합니다. 모든 선언 소켓이 listen하고 readiness를 통과해야 새 연결을 받습니다.

한번 연결한 TCP 스트림은 끝날 때까지 같은 인스턴스에 갑니다. 새 연결은 준비된 인스턴스들에 분산합니다. `scaling.target`은 인스턴스당 열린 연결 수이며, HTTP의 요청 동시성과 단위가 다릅니다. 조용히 열려 있는 연결도 활성으로 계산하므로 idle 회수를 막습니다. 모든 연결이 닫힌 뒤 `idleMs`가 지나야 0개로 줄일 수 있습니다. `scaling` 없이 `replicas: 0`을 지정하면 중지 상태이며 새 연결로 기동하지 않습니다.

버전을 바꾸려면 새 번들 URL/digest 또는 환경변수를 `wafl app patch`로 제출합니다.

```sh
cat > /tmp/tcp-update.yaml <<'YAML'
manifest:
  env:
    GREETING: updated
YAML
wafl app patch @echo --file /tmp/tcp-update.yaml
```

새 실행이 준비되면 기존 실행을 종료 대상으로 바꿉니다. 이후 새 연결은 준비된 실행으로 가고, 기존 연결은 이전 실행에서 끝납니다. 전환 중에는 준비된 두 버전이 잠시 함께 새 연결을 받을 수 있습니다. `drainMs`가 끝나면 기존 WASM을 종료하며, 관측 기록이 연결 종료를 확인한 뒤 모든 endpoint 예약을 회수합니다. 포트 회수 조건은 [직결 계약](direct.md)과 같습니다.

클라이언트 송신 EOF는 upstream의 송신 반쪽만 닫으며 응답 방향은 계속 읽습니다. 양쪽 EOF가 끝나거나 연결 오류가 발생하면 연결 전체를 닫습니다. 정상 proxy 종료는 drain 유예 시간 동안 기존 연결을 기다리고 종료 상태를 보고합니다.

## 한도와 운영 범위

- App 전체 준비 대기는 `scaling.waiting`/`waitMs`를 사용합니다. 기본 128개/30초입니다. proxy replica당 HTTP 요청과 대기·활성 TCP 연결 합계는 기본 1024개이며 `--capacity`로 설정합니다. 한도나 기동 시간 초과 시 연결을 닫습니다. 임의 TCP 프로토콜에 HTTP 오류나 별도 에러 바이트를 삽입하지 않습니다.
- TCP 중계는 양방향 복사와 소켓 backpressure를 사용합니다. 전체 요청을 메모리에 모으거나 payload를 해석하지 않습니다. backend 연결 후 실패한 데이터를 재전송하지 않습니다.
- HTTP/gRPC와 TCP 모두 Pingora proxy에서 WASM 소켓으로 직결합니다.
- 준비된 실행은 proxy/placement별 관측 gate에서 연결을 집계하고 약 500ms마다 보고합니다. 실행 기동을 기다리는 연결은 개별 대기 기록을 사용합니다.
- proxy가 사라지면 해당 연결은 끊깁니다. 다른 replica가 새 연결을 받을 수 있으나 기존 스트림을 옮기지는 않습니다. controller 재시작은 이미 전달 중인 TCP 데이터 경로를 끊지 않습니다. 강제 proxy 종료나 API 장애로 종료 확인이 남으면 자동 회수를 보수적으로 멈춥니다. 프로세스·연결 종료를 확인한 뒤 [관측 복구](autoscaling.md)의 `wafl traffic close ... --withdrawn` 절차를 사용합니다.
- 소켓 App을 실행하는 runner는 `named-sockets-v1` capability를 제공해야 합니다.

## 단독 runner

Kubernetes 없이 여러 TCP 소켓을 실행할 때는 direct runner에 JSON Manifest를 제출합니다.

```sh
wafl serve runner /tmp/wafl-tcp-cache --config examples/runner-direct.json
# 다른 터미널
wafl instance add "file:$PWD/examples/tcp/tcp.tar.gz" \
  --alias echo --manifest examples/tcp/manifest.json
wafl --json instance get @echo
```

`Instance.endpoints`에 소켓별 `{name, address, transport, protocol}`이 반환되고 `endpoint`는 첫 소켓을 가리킵니다. 단독 runner에는 App 공개 포트나 자동 배치·TCP 연결 관측이 없습니다. 반환 주소로 직접 접속하며 외부 호출자가 연결 철회와 `Drain → Release`를 책임집니다. 기본 runner HTTP proxy 모드에서는 일반 TCP와 복수 소켓을 거부합니다.

## 검증

`scripts/test.sh`는 동일 게스트 포트의 두 인스턴스·복수 소켓·서버 선행 응답·바이너리 데이터·TCP 반닫힘·모든 소켓의 준비 대기·예약 회수를 실제 WASM으로 검사합니다. `scripts/test-kubernetes.sh`는 별도 kind에 실제 Pingora proxy를 두고 TCP 최초 연결 기동, 공개 포트 두 개, 업데이트 중 기존 연결 유지, 연결 부하에 따른 증감, idle 0개 회수와 재기동, fixed-zero 중지를 검사합니다. TCP FIN 검증은 kubectl port-forward가 아닌 실제 NodePort 경로를 통과합니다.
