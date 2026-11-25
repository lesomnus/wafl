# Runner proxy와 직결 성능 측정

같은 WASIp2 HTTP 서버 두 개를 Wasmtime CLI 두 프로세스, WAFL proxy 한 프로세스,
WAFL direct 한 프로세스로 실행한다. 게스트는 이전 리소스 비교에 사용한 소스를
그대로 사용하며 앱마다 독립적인 memory와 TCP listener를 갖는다.

## 빌드와 실행

저장소에 고정한 Rust toolchain과 [Wasmtime 51.0.0 CLI](https://github.com/bytecodealliance/wasmtime/releases/tag/v51.0.0)를 사용한다.
측정 전에 모든 빌드를 완료하고, 측정 중 다른 빌드·테스트·부하 실험을 실행하지 않는다.
`CARGO_TARGET_DIR`을 지정했다면 아래 `target`을 그 경로로 바꾼다.

```sh
cargo build --locked --release --bin wafl
mkdir -p /tmp/wafl-performance
rustc --edition 2024 --target wasm32-wasip2 -O \
  scripts/benchmarks/guest.rs -o /tmp/wafl-performance/main.wasm

python3 scripts/benchmarks/resources.py \
  --wasmtime /path/to/wasmtime --wafl target/release/wafl \
  --guest /tmp/wafl-performance/main.wasm \
  --output /tmp/wafl-performance/backend.json
```

기본값은 각 구성을 새 프로세스로 3회 반복하고 실행 순서를 회전한다.
모든 요청은 두 앱에 50:50으로 보내며 응답 status/body를 검사한다.
각 부하 앞에 2초 warmup을 수행한다.

| 구간 | 조건 |
| --- | --- |
| idle | 준비 완료·2초 안정화 후 10초 |
| http | `ok\n` 응답, 합계 500 req/s, 10초 |
| work | 요청마다 xorshift 200,000회 후 같은 응답, 합계 200 req/s, 10초 |

`--repeat`, `--seconds`, `--rps`, `--work-rps`, `--work`로 변경할 수 있다.
`--memory 0 32`는 각 게스트가 추가로 할당·접촉·보유하는 heap 크기(MiB)별 실행이다.
기본 0도 guest stack·기본 heap을 사용한다. 인스턴스 linear memory 한도는 모두 64 MiB다.

## 앞단 L7을 포함한 비교

[Traefik](https://doc.traefik.io/traefik/reference/install-configuration/providers/others/file/) 바이너리를 준비하고 별도 결과에 측정한다.

```sh
python3 scripts/benchmarks/resources.py \
  --wasmtime /path/to/wasmtime --wafl target/release/wafl \
  --guest /tmp/wafl-performance/main.wasm \
  --traefik /path/to/traefik --gateway --cases proxy direct \
  --output /tmp/wafl-performance/gateway.json
```

proxy 구성은 `클라이언트 → Traefik → runner ingress → WASM`이고,
direct 구성은 `클라이언트 → Traefik → WASM`이다. Traefik이 Host로 두 앱을 구분하며
proxy 구성에는 `wafl-instance` 헤더를 주입한다. 별도 서비스의 두 backend로 구성해
같은 도메인 라우팅 작업을 수행한다. CLI도 `--cases cli proxy direct`로 포함할 수 있다.

각 phase의 최상위 CPU/PSS는 **backend + gateway** 합계다. `groups.backend`와
`groups.gateway`에도 별도 수치를 기록하므로 비용이 앞단으로 이동했는지 구분할 수 있다.
측정기·부하 생성기 CPU는 포함하지 않는다. 모든 구간 뒤 gateway부터 종료하고 완전히
종료한 후 backend를 종료한다. 다음 trial에 이전 경로나 연결 pool을 남기지 않는다.

## 공정한 조건과 해석 범위

- proxy/direct는 동일한 runner release 바이너리다. 차이는 `direct` 설정, endpoint 선택,
  그 모드의 요청 계측·lifecycle 경로다. 이 차이 전체를 비교하며 함수별 profiling은 아니다.
- CLI도 같은 Wasmtime 버전과 게스트를 사용한다. runner와 같은 on-demand allocator,
  디스크 컴파일 cache 없음, parallel compilation 없음으로 설정한다. 공식 CLI의 host 기능과
  executor가 runner와 완전히 같지는 않으며 두 프로세스 대 한 프로세스의 비교다.
- 프로세스마다 `TOKIO_WORKER_THREADS=2`, `RAYON_NUM_THREADS=2`, `GOMAXPROCS=2`를
  사용한다. CPU/fuel quota는 없고, runner는 취소를 위한 기존 10 ms epoch clock을 유지한다.
- 모든 수신 주소는 loopback이다. 각 요청은 새 TCP 연결이며 guest는 `Connection: close`로
  응답한다. 이 조건은 이전 실험과 같고 keep-alive, HTTP/2, gRPC, TLS, 큰 body, 실제 Pod 간
  네트워크나 최대 처리량을 대표하지 않는다. 직결 mode의 gRPC 동작은 별도 통합 테스트에서 검증한다.
- OTLP export·DB·앱별 요청 로그는 끈다. compilation·bundle 다운로드·Add는 측정 구간 밖이다.
  JSON의 startup_seconds는 준비 시간을 참고로 기록한 값이며 cold-start 비교에 사용하지 않는다.
- CPU는 `/proc/<pid>/stat`의 user+system 시간 합계로 **100% = 코어 1개**다. 10 ms tick
  해상도 때문에 작은 idle 사용량이 0%로 관측될 수 있다.
- 메모리는 `/proc/<pid>/smaps_rollup`의 PSS를 주 지표로 사용한다. RSS·private memory도 기록한다.
  별도 CLI 프로세스도 실행 파일 페이지를 공유하므로 RSS 합계만으로 절감률을 계산하지 않는다.
  파일 cache·kernel socket buffer 등 cgroup 전체 메모리는 포함하지 않는다.
- 고정 도착률로 요청을 생성하며 task 시작 지연 `schedule_p95_ms`, 실제 처리율과 실패를 기록한다.
  응답 latency는 task 시작부터 body 완료까지다. shared host의 다른 부하와 scheduler 영향을 받는다.
  `cgroup_cpu_start/end`는 이 실험 밖 프로세스도 포함한 cgroup 상태로, 서버 CPU와 분리해서 해석한다.
- `/proc` 표본은 별도 thread에서 읽어 kernel memory accounting 지연이 요청 생성 event loop를
  멈추지 않게 한다. `sampling_ms`에 각 표본 수집 소요 시간을 남긴다. 목표 처리율을 유지하지
  못하거나 task 시작 지연이 큰 구간은 정상 부하의 latency 비교로 사용하지 않는다.
  CPU/time의 구간 경계는 빠른 `/proc/stat` counter로 별도로 잡고 마지막 memory 표본 대기를 포함하지 않는다.

## 결과 보존

JSON에 각 반복의 명령, 바이너리·게스트·측정기 SHA-256, runner source revision, 환경,
0.5초 간격 표본, traffic 오류와 지연을 기록한다. 각 표의 요약값은 반복별 결과의 중앙값이며
단 한 번의 최저 CPU/지연을 고르지 않는다.

```sh
python3 scripts/benchmarks/summarize.py \
  /tmp/wafl-performance/backend.json /tmp/wafl-performance/gateway.json \
  > /tmp/wafl-performance/summary.json
```

`--output` 옆 `.files` 디렉터리에 bundle, runner 설정, 포트 예약과 process 로그를 보존한다.
동일 output directory가 이미 있으면 실패하며 과거 결과를 덮어쓰지 않는다.
이 파일들은 격리된 벤치마크 실행의 자료다. 실제 서비스의 포트 예약을 이 측정기로 정리하지 않는다.

이 스크립트는 제품 코드를 바꾸지 않는다. 짧은 smoke run으로 구성·응답 검증을 한 뒤
정식 측정을 실행하며, smoke run의 수치는 성능 결론에 사용하지 않는다.
