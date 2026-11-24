# Wafl

작은 HTTP/gRPC 앱을 위한 자체 운영 WASM 앱 플랫폼. 전체 구상은 [이슈 #1](https://github.com/lesomnus/wafl/issues/1)에 정리되어 있습니다.

현재 구현은 **paydirt와 Wasmtime을 사용하는 runner**입니다. 하나의 프로세스에서 여러 WASI Preview 2 소켓 서버를 실행합니다. 플랫폼은 생성된 `InstanceService.Add(InstanceAddRequest)`에 `tenant`, `name`, `manifest`를 제출합니다. runner가 `manifest.bundle.url`의 `tar.gz`를 다운로드하여 인스턴스를 시작하고, 준비된 `Instance`를 반환합니다.

```text
InstanceAddRequest { tenant, name, manifest: { bundle: { url }, port, ... } }
  → 다운로드·압축 해제 → main.wasm 컴파일·실행 → Instance { id, manifest, phase, ... }

gateway → runner (wafl-instance: <Instance UUID>) → WASM 서버 소켓
```

번들에는 루트의 `main.wasm`과 선택적인 `data/`가 있습니다. `data/`는 게스트의 읽기 전용 `/data`로 제공됩니다. 유휴 인스턴스는 실행을 종료하고 다음 요청에서 다시 시작합니다. `Erase`는 인스턴스와 보관한 번들을 제거합니다.

앱은 일반 `wasi:cli/run` 프로그램이며 직접 TCP 소켓을 엽니다. Wafl 핸들러 SDK는 없습니다. 예제는 Rust의 `std::net`과 `std::fs`를 사용하여 로컬과 `wasm32-wasip2`에서 같은 소스를 실행합니다.

```sh
# 번들 생성 후 HTTP로 제공 (이 터미널에서 계속 실행)
bash scripts/bundle.sh /tmp/wafl-example/hello.tar.gz
python3 -m http.server 9000 --bind 127.0.0.1 --directory /tmp/wafl-example
```

다른 터미널에서 빈 runner를 실행합니다.

```sh
cargo run --locked -- examples/runner.json
```

다른 터미널에서 인스턴스를 만들고 호출합니다.

```sh
curl --fail-with-body -H 'Content-Type: application/json' \
  -H 'Connect-Protocol-Version: 1' --data-binary @examples/instance.json \
  http://127.0.0.1:8081/wafl.InstanceService/Add

curl -H 'wafl-instance: 00000000-0000-8000-8007-000000000001' \
  http://127.0.0.1:8080/
curl -H 'wafl-instance: 00000000-0000-8000-8007-000000000001' \
  http://127.0.0.1:8080/index.html
```

`runner.json`은 `listen`과 `control`만 설정합니다. 앱·도메인·배치 관리는 플랫폼의 역할이며, runner는 제출된 인스턴스를 관리합니다. 관리 RPC에는 인증이 없으며 `tenant`는 entity 소속 정보입니다. Kubernetes 제어평면, gateway 설정, 업로드 저장소와 DB 프로비저닝은 아직 포함하지 않습니다.

[튜토리얼과 시나리오](docs/runner.md) · [번들 규격](docs/bundle.md) · [Instance](proto/wafl/instance.proto) · [Manifest](proto/wafl/manifest.proto) · [생성된 RPC 계약](generated/proto/wafl/instance_svc.g.proto)

```sh
bash scripts/test.sh
cargo clippy --locked --all-targets -- -D warnings
```

실제 WASM 실행으로 동적 생성, 파일·포트·메모리 격리, 기동 동시성, 취소·실패 정리, 유휴 회수, TCP 서비스 매핑과 HTTP/2 gRPC 스트리밍을 검증합니다.
