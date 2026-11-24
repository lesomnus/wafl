# Wafl

작은 HTTP/gRPC 앱을 위한 자체 운영 WASM 앱 플랫폼. 전체 구상은 [이슈 #1](https://github.com/lesomnus/wafl/issues/1)에 있습니다.

현재 runner는 paydirt와 Wasmtime으로 한 프로세스에서 여러 WASIp2 소켓 서버를 실행합니다. 앱은 표준 TCP·파일 API를 사용하며 별도 핸들러 SDK가 필요하지 않습니다. `Instance`는 Manifest를 가진 일반 entity이고 생성된 `InstanceService.Add(InstanceAddRequest)`로 만듭니다.

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

앱·도메인·배치는 플랫폼이 관리하고 gateway가 `wafl-instance` 헤더로 인스턴스를 선택합니다. runner 관리 API는 인증 없이 동작하며 tenant는 소속 메타데이터입니다. Kubernetes 제어평면, credential, ORAS 다운로드와 원격 파일 지연 읽기는 아직 구현하지 않았습니다.

[CLI 튜토리얼](docs/cli.md) · [외부 연결과 DB](docs/network.md) · [RPC와 시나리오](docs/runner.md) · [번들 규격](docs/bundle.md) · [공유 저장소 계약과 원격 읽기 확장](docs/store.md) · [Instance](proto/wafl/instance.proto) · [생성된 RPC](generated/proto/wafl/instance_svc.g.proto)

```sh
bash scripts/test.sh
cargo clippy --locked --workspace --all-targets -- -D warnings
```

실제 WASM과 별도 runner/CLI 프로세스로 ID·alias 참조, 로그 스트리밍, 동시 다운로드, 강제 종료 복구, 파일 접근 제한·수신 포트 가상화·발신 TCP/DNS와 gRPC 스트리밍을 검증합니다. CI는 Redis를 실행해 DB 클라이언트의 직접 연결과 인스턴스 재시작 후 상태 유지도 검증합니다.
