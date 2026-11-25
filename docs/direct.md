# WASM 소켓에 직접 연결하기

`direct` 설정을 켜면 앞단 L7 proxy가 각 WASM의 실제 TCP 소켓에 연결합니다. runner는 WASI 소켓의 bind 주소를 바꾸고 수명을 관리하며 HTTP 요청을 중계하지 않습니다. 앱은 기존처럼 `0.0.0.0:8080`을 listen하고 `InstanceService.Add`로 실행합니다. SDK나 앱 수정은 필요하지 않습니다.

```text
클라이언트 → Traefik/Envoy → runner Pod IP:배정 포트 → WASM 서버
제어평면  → runner 관리 API → 실행·조회·Drain·Release
```

`direct`를 생략한 기본 proxy 모드는 기존 `wafl-instance` 라우팅·요청 계측·유휴 재기동을 유지합니다. 아래 기능은 runner의 직결 계약입니다. 자동 배치, gateway 설정 배포, scale-to-zero 뒤 첫 요청을 받는 activator는 [제어평면 #10](https://github.com/lesomnus/wafl/issues/10)에서 이어서 구현합니다.

## 로컬에서 두 인스턴스 실행

```sh
cargo install --path . --locked --bin wafl
bash scripts/bundle.sh /tmp/wafl-example/hello.tar.gz
wafl serve runner /tmp/wafl-direct-cache --config examples/runner-direct.json
```

예제는 localhost 전용입니다. `direct`가 있으면 최상위 `listen`의 HTTP ingress를 열지 않습니다. 관리 API는 `control`에서 계속 제공합니다. 다른 터미널에서 실행합니다.

```sh
wafl instance add file:///tmp/wafl-example/hello.tar.gz --alias one --env GREETING=One
wafl instance add file:///tmp/wafl-example/hello.tar.gz --alias two --env GREETING=Two
# <UUID>    @one    127.0.0.1:20000
# <UUID>    @two    127.0.0.1:20001

wafl --json instance get @one
address=$(wafl instance get @one | cut -f3)
curl "http://$address/"
curl "http://$address/index.html"
wafl instance logs -f @one
```

포트는 예시입니다. 실제 반환값을 사용합니다. 두 게스트는 모두 가상 포트 8080을 사용하지만 실제 주소는 다릅니다. Host/authority나 `wafl-instance` 헤더로 runner가 대상을 다시 선택하지 않습니다. 도메인 선택·TLS 종료는 앞단 L7의 역할입니다.

`Instance.endpoint`는 `{address, protocol}`이며 `protocol`은 Manifest의 `http` 또는 `h2c`입니다. `direct: true`, `phase: PHASE_RUNNING`, `draining: false`, endpoint 존재를 모두 확인한 뒤 경로에 추가합니다. 준비 중에는 endpoint를 공개하지 않습니다. 실패·draining 상태에는 **철회할 주소를 알 수 있도록 endpoint가 남으므로**, 필드 존재만 보고 트래픽을 보내면 안 됩니다. 준비 완료는 TCP listen 기준이며 앱 health check는 별도로 수행합니다.

endpoint의 식별자는 `(session, id, generation)`입니다. IP:port는 안정적인 인스턴스 ID가 아니며 Release 뒤 다른 실행에 재사용할 수 있습니다.

## Traefik에 연결

Traefik v3의 [file provider](https://doc.traefik.io/traefik/reference/install-configuration/providers/others/file/)와 [service backend URL](https://doc.traefik.io/traefik/reference/routing-configuration/http/load-balancing/service/)을 이용하는 로컬 예제입니다. `traefik` 실행파일을 설치한 환경에서 위 두 인스턴스를 준비한 뒤 실행합니다.

```sh
mkdir -p /tmp/wafl-traefik
wafl --json instance get @one > /tmp/wafl-one.json
wafl --json instance get @two > /tmp/wafl-two.json
python3 - <<'PY'
import json, pathlib
lines = []
for alias in ('one', 'two'):
    app = json.loads(pathlib.Path(f'/tmp/wafl-{alias}.json').read_text())
    assert app['direct'] and app['phase'] == 'PHASE_RUNNING' and not app.get('draining', False)
    endpoint = app['endpoint']
    lines.extend([
        f'[http.routers.{alias}]',
        f'rule = "Host(`{alias}.localhost`)"',
        'entryPoints = ["web"]',
        f'service = "{alias}"',
        f'[[http.services.{alias}.loadBalancer.servers]]',
        f'url = "{endpoint["protocol"]}://{endpoint["address"]}"',
    ])
directory = pathlib.Path('/tmp/wafl-traefik')
draft = directory / 'apps.tmp'
draft.write_text('\n'.join(lines) + '\n')
draft.replace(directory / 'apps.toml')
PY
traefik --entrypoints.web.address=127.0.0.1:8080 \
  --providers.file.directory=/tmp/wafl-traefik --providers.file.watch=true
```

별도 터미널에서 도메인별 응답을 확인합니다. 이 8080 포트는 Traefik이 열며 runner는 HTTP ingress를 열지 않습니다.

```sh
curl -H 'Host: one.localhost' http://127.0.0.1:8080/
curl -H 'Host: two.localhost' http://127.0.0.1:8080/
```

HTTP/2 gRPC 서버를 Add할 때는 `--protocol h2c`를 사용합니다. [Traefik gRPC 설정](https://doc.traefik.io/traefik/master/user-guides/grpc/)에 따라 backend URL을 `h2c://<endpoint.address>`로 만듭니다. 게스트가 받는 HTTP/2 연결과 stream은 L7·앱이 관리합니다. runner에 HTTP/2 중계나 요청마다 새 upstream을 여는 단계는 없습니다.

이 예제는 설정 파일을 직접 작성합니다. 제어평면의 discovery·자동 갱신이나 gateway의 철회 확인을 구현한 예제가 아닙니다. 파일 변경 이벤트 또는 설정 수신 확인만으로 모든 기존 요청·연결이 끝났다고 판단하면 안 됩니다.

## 안전하게 내리고 포트 반환하기

직결에서는 runner가 진행 중인 HTTP 요청이나 gRPC stream을 알 수 없습니다. 순서는 다음과 같습니다.

1. `Drain`으로 `draining=true`를 설정합니다. 즉시 반환하고 WASM 소켓은 유지합니다.
2. **모든** gateway에서 해당 `(session, id, generation)`을 대상에서 빼고, 이전 backend connection pool을 폐기합니다. 이미 진행 중인 응답·stream을 마치거나 외부 정책에 따라 종료합니다. HTTP/2 연결 하나에서 새 stream이 생기는 경우도 막아야 합니다.
3. 철회와 연결 정리를 확인한 주체가 정확한 세션·revision·generation으로 `Release`를 호출합니다. runner는 WASM 실행과 소켓 종료를 기다린 다음 포트 예약을 반환합니다.
4. `Erase`로 등록 정보를 지우거나 `Start`로 새 generation을 시작할 수 있습니다. 새 endpoint를 다시 조회·공개합니다.

Drain만으로 직접 소켓에 오는 새 요청이 차단되지는 않습니다. `active=0`이나 TCP 연결 수만으로 drain 완료를 판정할 수도 없습니다. Release는 외부 정리가 완료됐다는 호출자의 명시적인 확인입니다. 먼저 Release하면 남은 stream을 끊거나 오래된 경로가 이후 다른 인스턴스에 연결될 수 있습니다.

로컬 튜토리얼에서는 다음처럼 직접 요청을 마치고 Traefik을 Ctrl-C로 **정상 종료하여 프로세스 종료까지 확인**하면 유일한 gateway의 연결이 정리됩니다. 다른 gateway나 직접 연결 클라이언트가 있으면 함께 정리해야 합니다.

```sh
wafl --json instance drain @one > /tmp/wafl-drained.json
# 이제 Traefik 터미널에서 Ctrl-C 후 종료를 확인하고 직접 연결도 종료합니다.
# 다음 명령은 그 확인이 끝난 뒤 실행합니다.
python3 - <<'PY'
import base64, json, pathlib, subprocess, uuid
app = json.loads(pathlib.Path('/tmp/wafl-drained.json').read_text())
session = str(uuid.UUID(bytes=base64.b64decode(app['session'])))
subprocess.run([
    'wafl', '--session', session, '--revision', str(app['revision']),
    'instance', 'release', '@one', '--generation', str(app['generation']),
], check=True)
PY
wafl instance erase @one
```

`Release`의 입력은 `{ref, generation}`이고 `wafl-session`·`wafl-revision` RPC metadata가 필수입니다. CLI는 `--session`, `--revision`, `--generation`으로 전달합니다. 오래된 조건은 거절합니다. 오류 뒤 무조건 최신 값을 넣어 재시도하지 말고, 현재 generation이 실제 철회한 실행인지 다시 확인합니다. RPC 연결이 끊겨도 이미 수락된 Release의 정리는 계속됩니다. 응답이 유실되면 Get으로 확인합니다. 같은 절차로 `@two`도 정리합니다.

예약이 있는 동안 일반 `Stop`·`Erase`와 draining/실패 실행의 `Start`는 거절합니다. direct Drain에는 `--timeout-ms`·`--force`를 지정하지 않습니다. 정상 runner 종료도 외부 drain을 대신하지 않습니다. 전체 종료 전 위 절차를 완료하며, SIGTERM/SIGKILL·충돌로 종료한 실행의 예약은 보존합니다.

## Pod 주소·공유 상태·재시작

```json
{
  "listen": "127.0.0.1:8080",
  "control": "0.0.0.0:8081",
  "direct": {
    "listen": "0.0.0.0",
    "advertise": "10.244.1.5",
    "directory": "/var/lib/wafl/endpoints",
    "first": 20000,
    "last": 29999
  }
}
```

`advertise`에는 gateway가 접근할 실제 runner Pod IPv4를 넣습니다. `listen`은 그 IP 또는 `0.0.0.0`만 허용하며 wildcard 주소를 endpoint로 반환하지 않습니다. Pod IP 주입은 배포 구성이 생성한 JSON으로 처리합니다. runner 설정에 환경변수 문자열을 넣어 자동 치환하는 기능은 없습니다. 현재 직결 수신은 IPv4이고 기존 발신 IPv4·IPv6 TCP/DNS는 유지합니다.

gateway는 **개별 runner Pod IP:배정 포트**를 backend로 사용해야 합니다. 임의 runner로 분산하는 단일 Service IP:port를 넣으면 다른 Pod로 갈 수 있습니다. `containerPort`를 하나씩 생성하는 방식으로 포워딩하지 않습니다. 실제 소켓과 Pod 네트워크가 수신하며 배포의 네트워크 정책에서 해당 범위를 접근 가능하게 합니다.

`direct.directory`는 **절대 경로의 영속 상태**이며 삭제 가능한 bundle cache와 서로 포함되지 않는 별도 디렉터리여야 합니다. 같은 advertise IP를 사용하는 모든 runner 프로세스는 같은 상태 디렉터리를 공유합니다. 포트 풀은 WAFL 전용으로 운영하고 OS ephemeral/client 포트·다른 서버의 listener 범위와 겹치지 않게 배치합니다. runner가 OS 네트워크 설정을 변경하지는 않습니다.

예약은 `v1/<advertise-IP>/<port>.json`에 version/token/session/instance/generation을 저장합니다. 풀의 `catalog.lock`과 `create_new`로 프로세스 간 배정·반환을 조정하고 파일·디렉터리를 동기화한 뒤 실제 WASI 소켓이 bind합니다. OS가 사용 중인 포트는 다른 후보를 시도합니다. probe 후 소켓을 닫고 재bind하는 방식은 쓰지 않습니다. 세션별 파일의 OS 잠금으로 소유자의 생존 여부를 확인합니다. 상태 저장소는 로컬 POSIX 파일 생성·동기화·잠금 의미를 제공해야 합니다. NFS/object storage 분산 잠금은 이 계약에 포함하지 않습니다.

공개된 예약은 시간 경과, 죽은 PID, runner 재시작, bundle GC로 회수하지 않습니다. IP가 재사용될 수 있는 Pod 교체도 이전 IP의 예약 상태를 잃으면 안 됩니다. Kubernetes에서는 IP 재사용과 gateway 철회를 제어평면이 조정하고 상태 볼륨의 수명을 그에 맞춰야 합니다. 단순 `emptyDir`만으로 재생성된 Pod까지 보호하지 못합니다.

충돌 후 Instance 목록은 복원되지 않습니다. `ReservationService.Get/List`로 이전 예약을 조회하고, 모든 경로·연결을 철회한 뒤 `ReservationService.Release`에 관측한 token과 `withdrawn:true`를 제출합니다. 살아 있는 세션이나 새 소유자로 바뀐 예약의 회수는 거절합니다. CLI 명령은 `wafl reservation list/get/release`이며 [장애 복구 튜토리얼](reservations.md)에 정상·장애·동시 회수와 이전 저널의 업그레이드 절차가 있습니다. 재시작마다 상태 디렉터리를 비우지 않습니다. 자동 조정 루프와 gateway 확인 프로토콜은 #10의 후속 범위입니다. 풀이 소진되면 Add/Start가 실패하고 다른 앱의 주소를 임의로 재사용하지 않습니다.

## 제한과 메트릭

| 기능 | 직결 모드 |
| --- | --- |
| guest memory / runner 실행·메모리 예약 / 준비·기동 한도 | 계속 적용 |
| bundle 검증·캐시 GC·컴파일 공유·Tail·OTLP 프로세스 CPU/RSS | 계속 적용 |
| Manifest `idle_ms`, `max_active` | Add에서 생략 또는 0만 허용. 정규화된 manifest도 0 |
| Instance `active`, runner request lease | Add/Start 관리 대기만 나타냄. 앱 요청 수 아님 |
| `wafl.instance.active` gauge | 의미 없는 0을 보내지 않고 생략 |
| 유휴 회수·요청 한도·HTTP/gRPC 기한·재시도 | L7·제어평면·앱에서 처리 |
| `wafl-instance`, `wafl-session` 앱 트래픽 헤더 | 선택 기능 없음. 필요한 외부 헤더 정책은 L7에서 처리 |
| scale-to-zero의 첫 요청 | 외부 activator가 실행을 준비하고 endpoint를 공개한 뒤 전달해야 함 |

`Runner.ingress`는 빈 문자열이고 capabilities에 `direct-endpoint-v1`, `external-drain-v1`, `reservation-recovery-v1`을 제공합니다. 기본 proxy 모드는 `proxy-ingress`, `drain`을 제공합니다. runner의 `capacity.requests`와 `usage.requests`는 직결 HTTP 부하 제한·관측으로 사용할 수 없습니다. CPU/fuel quota를 추가하지 않았으며 메모리 제한도 process RSS 전체 제한과는 다릅니다.

## 검증 시나리오

```sh
bash scripts/test.sh --test runner direct::
bash scripts/test.sh --test cli direct::
cargo test --locked --lib endpoint::
```

- 같은 가상 포트로 여러 WASM을 실행하고 non-loopback 실제 주소로 헤더 없이 호출합니다.
- HTTP 응답 body·gRPC stream 도중 Drain해도 연결이 유지되고 Release 뒤 포트가 반환됩니다.
- session/revision/generation이 다른 Release를 거절하고, 공유 상태를 쓰는 별도 runner 프로세스가 주소를 중복 배정하지 않습니다.
- runner를 강제 종료·재시작해도 이전 주소를 새 앱에 재사용하지 않습니다.
- 이전 세션의 예약을 조건부 회수한 뒤 같은 포트를 재사용하고, 오래된 token의 재시도가 새 앱을 건드리지 않는지 검증합니다.

Traefik 예제는 실제 file provider와 두 backend로 수동 검증합니다. CI는 외부 프록시 다운로드 없이 runner/CLI와 실제 WASM 소켓 계약을 검증합니다.
