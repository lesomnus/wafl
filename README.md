# Wafl

작은 HTTP/gRPC 앱을 위한 자체 운영 WASM 앱 플랫폼. 전체 구상은 [이슈 #1](https://github.com/lesomnus/wafl/issues/1)에 정리되어 있습니다.

현재 구현은 **paydirt와 Wasmtime을 사용하는 초기 runner**입니다. 하나의 프로세스가 여러 WASI Preview 2 서버를 실행하고, 요청의 도메인에 따라 앱 소켓으로 전달합니다. 첫 요청에 인스턴스를 시작하고 이후 요청에 재사용하며, 활성 요청이 없는 인스턴스는 유휴 시간이 지나면 종료합니다.

앱은 일반 `wasi:cli/run` 프로그램이며 직접 TCP 소켓을 엽니다. Wafl 핸들러 SDK는 없습니다. 예제는 Rust의 `std::net`만 사용하여 로컬과 `wasm32-wasip2`에서 같은 코드를 실행합니다.

```sh
# rustup이 rust-toolchain.toml에 지정된 도구와 타깃을 설치합니다.
cargo build --locked -p wafl-example-hello --target wasm32-wasip2
cargo run --locked -- examples/runner.json
```

다른 터미널에서:

```sh
curl -H 'Host: hello.localhost' http://127.0.0.1:8080/
curl -H 'Host: bye.localhost' http://127.0.0.1:8080/
curl -H 'Content-Type: application/json' -H 'Connect-Protocol-Version: 1' \
  -d '{}' http://127.0.0.1:8081/wafl.InstanceService/List
```

두 앱 모두 내부 포트 `8080`을 사용하지만 메모리, 환경 변수, 소켓은 각각 독립적입니다. 예제의 유휴 시간은 5초이며 회수 후 다음 호출에서 요청 카운터가 다시 1이 됩니다.

[사용법과 시나리오](docs/runner.md) · [Instance 리소스](proto/wafl/instance.proto) · [생성된 관리 계약](generated/proto/wafl/instance_svc.g.proto)

검증:

```sh
bash scripts/test.sh
cargo clippy --locked --all-targets -- -D warnings
```

WASM 앱을 실제로 실행하여 같은 포트 격리, 기동 동시성, 실패·취소, 유휴 회수, TCP 서비스 매핑, HTTP/2 gRPC 양방향 데이터와 trailers를 검증합니다. Kubernetes 제어평면, 외부 gateway 설정, 아티팩트 업로드, DB 프로비저닝은 아직 포함하지 않습니다.
A self-hosted app platform for HTTP/gRPC services, powered by WebAssembly and managed infrastructure.
