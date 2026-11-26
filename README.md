# Wafl

작은 HTTP/gRPC·TCP 앱을 위한 자체 운영 WASM 앱 플랫폼. 전체 구상은 [이슈 #1](https://github.com/lesomnus/wafl/issues/1)에 있습니다.

플랫폼은 Kubernetes를 사용하며 `App → Deployment → ReplicaSet → Pod → runner Instance`로 실행합니다. 실제 runner Pod 하나가 가상 Node 하나를 제공하고, runner 프로세스 내부에서 여러 WASM Instance를 실행합니다. 앱은 표준 소켓으로 서버를 열며 별도 SDK가 필요 없습니다.

**[Kubernetes native 시작하기](docs/kubernetes-native.md)**: 운영자는 `wafl-kube bootstrap dev`로 로컬 kind 플랫폼을 만들거나 `wafl-kube bootstrap install`로 기존 클러스터에 설치합니다. 앱 사용자는 controller 주소만 지정해 `wafl app add app.yaml`로 앱을 올립니다. `wafl` 클라이언트는 Kubernetes API와 kubeconfig를 사용하지 않습니다. 준비 상태를 확인한 rolling update, Secret 교체, 첫 요청으로 0개에서 기동하기와 부하에 따른 증감을 지원합니다. 플랫폼 상태는 Kubernetes에 저장합니다.

이미지는 `ghcr.io/lesomnus/wafl:latest`와 관측 플러그인을 포함한 `ghcr.io/lesomnus/wafl-traefik:latest`입니다. [자동 증감](docs/autoscaling.md) · [Secret](docs/secrets.md) · [Controller 구조](docs/controller.md) · [컨테이너](docs/container.md)

단독 runner와 Instance CLI는 Kubernetes 없이 사용할 수 있습니다. `Instance`는 Manifest를 가진 Payday entity이고, paydirt로 생성한 `InstanceService.Add(InstanceAddRequest)`로 만듭니다. Wasmtime으로 WASIp2 소켓 서버를 실행합니다.

**[일반 TCP 튜토리얼](docs/tcp.md)**: 여러 이름 있는 소켓과 공개 포트, 첫 연결 기동, 열린 연결의 부하에 따른 증감과 rolling update를 지원합니다. TCP는 Traefik → controller 연결 중계 → WASM 경로를 사용하고 UDP는 아직 지원하지 않습니다.

**L7 proxy에서 WASM 소켓에 직접 연결**하려면 [직결 튜토리얼](docs/direct.md)의 `direct` 설정을 사용합니다. `Instance.endpoint`로 반환한 IP:port를 Traefik/Envoy backend로 지정하며 runner HTTP 중계는 없습니다. 포트 공유·재시작 예약과 `Drain → 외부 연결 정리 → Release` 계약을 포함합니다. 아래 빠른 시작은 기본 proxy 모드입니다.

프로세스 장애로 남은 직결 예약은 `wafl reservation list/get`으로 조회하고, 외부 경로 정리 후 관측한 token으로 `release`합니다. [예약 복구 튜토리얼](docs/reservations.md)은 OS 잠금에 의한 소유자 확인, 재배정 보호와 업그레이드 절차를 설명합니다.

```sh
cargo install --path . --locked --bin wafl
bash scripts/bundle.sh /tmp/wafl-example/hello.tar.gz

# 이 터미널에서 블로킹 실행. 경로 생략 시 XDG cache 또는 ~/.cache/wafl
wafl serve runner /tmp/wafl-cache
```

다른 터미널에서:

```sh
wafl instance add file:///tmp/wafl-example/hello.tar.gz \
  --alias hello --env GREETING=Hello --idle-ms 5000
# <UUID>    @hello

wafl instance get @hello
wafl instance stop @hello
wafl instance start @hello
wafl instance logs -f @hello
```

또 다른 터미널에서 앱을 호출하면 로그가 출력됩니다.

```sh
instance_id=$(wafl instance get @hello | cut -f1)
curl -H "wafl-instance: $instance_id" http://127.0.0.1:8080/
curl -H "wafl-instance: $instance_id" http://127.0.0.1:8080/index.html
```

Bundle URL은 HTTP·HTTPS·file을 지원합니다. tar.gz 루트의 `main.wasm`을 실행하고 선택적인 `data/`를 읽기 전용 `/data`로 제공합니다. 기본 앱 포트는 8080, 관리 endpoint는 `http://127.0.0.1:8081`입니다. `--endpoint`, `--port` 등으로 변경할 수 있습니다.

`Add`는 캐시 조회/다운로드·검증·실행 준비 후 ID와 alias를 반환합니다. 준비된 번들은 SHA-256으로 보관하고 프로세스 간 잠금과 atomic rename으로 여러 runner가 같은 디렉터리를 재사용합니다. 유휴 인스턴스는 실행을 종료하고 다음 요청에서 다시 시작합니다. `Erase`는 인스턴스를 제거하고 캐시 pin을 해제합니다. 새 번들 공간이 필요하면 미참조 캐시를 LRU 순서로 회수하며, 등록된 Instance의 번들은 Stop 상태도 보호합니다. [한도·GC 설정](docs/store.md)을 참고하세요.

앱은 DNS와 IPv4·IPv6 TCP로 외부 서비스에 직접 연결할 수 있습니다. 플랫폼이 `DATABASE_URL` 등의 환경변수를 전달하면 앱의 DB 클라이언트가 연결하며, 별도 DB 포트 매핑은 선택 사항입니다. 발신 연결은 runner 호스트·Pod의 네트워크 범위를 공유합니다.

runner 설정의 `metrics.endpoint`를 지정하면 실행 상태·활성 lease와 Linux 프로세스 CPU/RSS를 OTLP/HTTP protobuf로 전송합니다. 설정을 생략하면 비활성화됩니다. [메트릭 튜토리얼](docs/metrics.md)에 Collector와 함께 실행하는 예제가 있습니다.

[Runner 운영 튜토리얼](docs/operations.md)은 생성 중 조회, Drain/Start, 전체 자원 수락 제한, 컴파일 공유, 세션·revision 조건과 운영 시나리오를 설명합니다.

[직결 성능 비교](docs/benchmarks/2026-11-25-direct.md)는 같은 WASM 두 개의 Wasmtime CLI·runner proxy·직결과 Traefik을 포함한 전체 비용을 비교합니다. 측정 스크립트와 원본 결과를 함께 제공합니다.

[Keep-alive 비교](docs/benchmarks/2026-11-25-keepalive.md)는 Wasmtime 개별 실행과 runner 직결 양쪽에 Traefik을 두고, 앱의 연결 유지 지원과 클라이언트의 연결 재사용을 독립적으로 비교합니다.

단독 runner의 기본 proxy 모드는 `wafl-instance` 헤더를 사용합니다. HTTP/gRPC Kubernetes 경로는 Traefik이 `Host`로 앱을 선택하고 준비된 WASM 소켓에 직접 전달합니다. gateway의 수신 중단과 스트림 종료를 확인한 뒤 포트를 회수합니다. 관리 CLI는 controller 주소 하나로 runner·Instance·Reservation을 조회할 수 있으며, 연결에 실패한 runner는 경고와 함께 건너뜁니다. [집계 API와 사용 예](docs/controller-api.md)를 참고하세요. runner 관리 API에는 아직 인증이 없습니다. 앱 DB 자동 제공, 번들 credential, ORAS 다운로드와 원격 파일 지연 읽기는 아직 구현하지 않았습니다.

[CLI 튜토리얼](docs/cli.md) · [외부 연결과 DB](docs/network.md) · [RPC와 시나리오](docs/runner.md) · [번들 규격](docs/bundle.md) · [공유 저장소 계약과 원격 읽기 확장](docs/store.md) · [Instance](proto/wafl/instance.proto) · [생성된 RPC](generated/proto/wafl/instance_svc.g.proto)

```sh
bash scripts/test.sh
cargo clippy --locked --workspace --all-targets -- -D warnings
```

실제 WASM으로 runner·CLI·gRPC·외부 DB 연결을 검증합니다. Kubernetes 종단 테스트는 kind에 controller 2개·runner 2개·실제 Traefik을 설치하고 readiness 실패, Secret rolling update 중 열린 gRPC 스트림, 첫 요청 기동, 부하 증감과 runner 장애 복구를 확인합니다. `scripts/test-kubernetes.sh`를 참고하세요.
