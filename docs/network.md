# 외부 연결과 DB

앱은 표준 WASI TCP 소켓으로 외부 서비스에 직접 연결합니다. IPv4·IPv6 주소와 DNS 이름을 지원하며, `DATABASE_URL` 같은 환경변수는 `manifest.env` 또는 CLI의 `--env`로 전달합니다. runner는 connection string을 해석하지 않습니다. 앱의 클라이언트 라이브러리가 주소 해석, DB 프로토콜과 필요한 TLS 처리를 담당합니다.

## 주소와 네트워크 범위

- 발신 TCP와 이름 조회는 runner 호스트·Pod의 네트워크와 resolver를 사용합니다. runner의 별도 목적지 allowlist는 없습니다. OS·네트워크 정책이 허용하는 목적지에 연결할 수 있습니다.
- `localhost`와 loopback IP는 runner 호스트·Pod를 뜻합니다. 원격 runner에 제출한 앱에서 개발자 컴퓨터를 가리키지 않으며, 각 WASM 인스턴스의 사설 네트워크도 아닙니다.
- 선택적인 `bindings`는 선언한 IPv4 loopback 포트를 먼저 변환합니다. 예를 들어 `--binding 5432=10.0.0.20:5432`가 있으면 `127.0.0.1:5432`를 그 DB로 연결합니다. 그 밖의 TCP 목적지는 그대로 사용합니다.
- 서버의 수신 포트는 계속 가상화합니다. 각 인스턴스는 선언한 IPv4 서버 포트 하나를 사용하며, 추가 listen 포트는 허용하지 않습니다. 클라이언트의 포트 0 bind는 IPv4·IPv6 wildcard/loopback을 보존합니다.
- UDP는 지원하지 않습니다. DNS는 호스트 resolver가 수행하므로 게스트 UDP 지원이 필요하지 않습니다. HTTPS나 DB TLS를 사용하려면 앱의 WASI 호환 TLS 구현과 신뢰할 CA 구성이 필요합니다. `wasi:http` 전용 클라이언트 인터페이스는 제공하지 않습니다.

이 구성은 한 runner의 인스턴스들이 발신 네트워크 범위를 공유합니다. loopback에서 동작하는 runner 관리 API 등도 연결 대상이 될 수 있습니다. [Kubernetes NetworkPolicy](https://kubernetes.io/docs/concepts/services-networking/network-policies/)는 runner Pod 단위로 적용되며 같은 Pod 안의 WASM 인스턴스를 구분하지 않습니다. 서로 다른 네트워크 신뢰 범위가 필요한 앱은 별도 runner Pod에 배치해야 합니다. `bindings`는 접근 제어 기능이 아닙니다.

## 로컬과 WASM에서 같은 DB 클라이언트 사용

[`examples/redis`](../examples/redis/src/main.rs)는 일반 `redis` crate의 동기 클라이언트를 사용하는 작은 HTTP 서버입니다. 요청마다 Redis의 카운터를 증가시키고 결과를 반환합니다. WAFL 전용 SDK나 WASI 전용 연결 어댑터를 사용하지 않습니다. 이 예제는 TLS 없는 로컬 개발 연결을 사용합니다.

먼저 개발용 Redis를 실행합니다.

```sh
docker run --rm -d --name wafl-redis-example \
  -p 127.0.0.1:6379:6379 redis:7-alpine
```

WASM을 빌드하기 전에 같은 소스를 네이티브로 실행할 수 있습니다.

```sh
DATABASE_URL=redis://localhost:6379/0 \
  cargo run --locked -p wafl-example-redis
# 다른 터미널
curl http://127.0.0.1:8080/
```

네이티브 앱을 Ctrl-C로 종료한 뒤 번들을 만들고 runner를 실행합니다.

```sh
cargo install --path . --locked --bin wafl
bash scripts/bundle.sh /tmp/wafl-example/redis.tar.gz redis
wafl serve runner
```

다른 터미널에서 인스턴스를 추가합니다. DB 연결을 위한 `--binding`은 필요하지 않습니다.

```sh
wafl instance add file:///tmp/wafl-example/redis.tar.gz \
  --alias counter --env DATABASE_URL=redis://localhost:6379/0
instance_id=$(wafl instance get @counter | cut -f1)
curl -H "wafl-instance: $instance_id" http://127.0.0.1:8080/

wafl instance stop @counter
wafl instance start @counter
curl -H "wafl-instance: $instance_id" http://127.0.0.1:8080/
```

카운터는 네이티브 앱과 WASM 인스턴스가 같은 Redis에 저장하므로 증가한 값을 유지합니다. WASM의 메모리와 DB 연결은 재시작할 때 새로 생성됩니다. 예제의 기본 key는 `wafl:visits`이고 `--env COUNTER_KEY=another-counter`로 바꿀 수 있습니다. 별도 인스턴스에서 같은 URL과 key를 사용하면 같은 카운터를 봅니다.

열린 DB 연결이나 앱의 발신 작업만으로 유휴 회수가 연장되지는 않습니다. runner의 활성 요청 수는 들어온 HTTP/gRPC 요청을 기준으로 계산합니다. 백그라운드 작업을 계속 유지하는 앱은 현재 유휴 회수 정책을 고려해야 합니다.

일반적으로 플랫폼이 DB·계정을 준비하고 URL을 주입하며, runner는 앱 실행을 담당합니다. 현재 runner가 DB를 생성하거나 Secret을 관리하지는 않습니다. `env`는 일반 manifest 필드여서 관리 API로 조회됩니다.

실습을 마치면 `wafl instance erase @counter`와 `docker stop wafl-redis-example`로 정리합니다. 위 개발용 컨테이너는 영구 볼륨을 사용하지 않으므로 컨테이너 제거 시 Redis 데이터도 사라집니다.

## Kubernetes와 다른 DB

runner Pod에서 접근 가능한 Service DNS 이름을 사용하면 됩니다. 예를 들어 Redis 앱에는 `redis://redis.default.svc.cluster.local:6379/0`을, PostgreSQL 드라이버를 사용하는 앱에는 다음 환경변수를 전달할 수 있습니다.

```json
{
  "env": {
    "DATABASE_URL": "postgresql://app@postgres.default.svc.cluster.local:5432/app"
  }
}
```

이 JSON은 manifest의 일부입니다. 사용할 DB의 드라이버와 TLS 설정이 WASI 타깃에서 동작해야 합니다. Redis 예제는 실제 WASM으로 검증하며, PostgreSQL 등 다른 드라이버의 호환성을 대신 보장하지 않습니다. HTTP API 호출도 동일한 발신 TCP/DNS를 사용하며 해당 클라이언트의 WASI 지원이 필요합니다.

## 검증

`bash scripts/test.sh`는 외부 인터넷 없이 로컬 backend를 사용해 IPv4·IPv6 직접 연결, 이름 조회, 명시적인 client bind, 선택적인 포트 매핑과 수신 제한을 검증합니다. 실제 Redis 테스트는 DB가 준비된 환경에서 실행합니다.

```sh
WAFL_TEST_DATABASE_URL=redis://localhost:6379/0 \
  bash scripts/test.sh --test runner \
  redis_connection_string_keeps_state_across_instances_and_restarts -- --ignored
```

CI에서는 Redis service를 띄우고 이 테스트도 실행합니다. 일반 DB 클라이언트가 `DATABASE_URL`만 받아 연결하는지, 두 인스턴스가 상태를 공유하는지, Stop 후 다시 실행되어도 값이 이어지는지 확인합니다.
