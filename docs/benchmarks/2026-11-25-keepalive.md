# Traefik을 포함한 Wasmtime 개별 실행과 runner 직결: Keep-alive 비교

앱과 클라이언트가 모두 HTTP/1.1 연결을 재사용할 때, Wasmtime CLI 두 프로세스와 runner
직결의 CPU·응답 지연은 이번 측정에서 비슷했다. **Traefik 포함 CPU는 19.39% 대 19.19%,
PSS는 159.46 대 145.72 MiB**였다. 이 조건에서는 runner 직결의 주된 이점이 메모리 절감으로 나타났다.

동일한 WASM 두 개를 다음 두 경로로 실행했다. 모든 경우에 **Traefik을 포함**한다.

```text
cli:    클라이언트 → Traefik → Wasmtime CLI 두 프로세스의 WASM
direct: 클라이언트 → Traefik → runner 한 프로세스 안의 WASM 두 인스턴스
```

앱이 HTTP/1.1 Keep-alive를 지원하는지와 클라이언트가 연결을 재사용하는지를 독립적으로
바꾸었다. 두 실행 방식 × 앱 지원 유무 × 클라이언트 재사용 유무, 총 8개 조건이다.

## HTTP 500 req/s 결과

표는 조건별 **3회 중앙값**, 괄호는 최소–최대다. **CPU 100% = 코어 1개**이며, 두 앱의
backend PID와 Traefik PID를 합산한다. 부하 생성기 CPU는 제외한다. 부하는 두 앱 합계 500 req/s다.

| 클라이언트 재사용 | 앱 지원 | Wasmtime CLI × 2 + Traefik | Runner 직결 × 1 + Traefik |
| --- | --- | ---: | ---: |
| 무 | 무 | 48.58% (47.28–49.77) | 46.58% (42.56–48.08) |
| 무 | 유 | 30.28% (26.78–33.97) | 25.78% (25.59–25.79) |
| 유 | 무 | 41.16% (39.98–46.68) | 38.87% (38.38–39.17) |
| 유 | 유 | 19.39% (18.49–19.39) | 19.19% (17.39–19.49) |

클라이언트 연결 재사용을 켜 둔 상태에서 앱도 Keep-alive를 지원하면, 전체 CPU는 CLI에서
**41.16% → 19.39%, 52.9% 감소**, 직결에서 **38.87% → 19.19%, 50.6% 감소**했다.
이 부하에서는 앱의 연결 재사용 효과가 실행 방식 사이의 차이보다 컸다.

양쪽 Keep-alive 조건에서 두 실행 방식의 CPU 중앙값 차이는 약 1%이고 범위가 겹친다.
이 결과로 runner가 CPU 면에서 항상 유리하다고 단정하지 않는다. 연결 종료 조건에서도
반복 간 변동이 있으므로 중앙값의 절감률을 고정된 비용 차이로 해석하지 않는다.

| 클라이언트 재사용 | 앱 지원 | CLI 합계 PSS | 직결 합계 PSS | CLI 지연 p95 | 직결 지연 p95 |
| --- | --- | ---: | ---: | ---: | ---: |
| 무 | 무 | 162.35 MiB | 148.06 MiB | 1.54 ms | 1.44 ms |
| 무 | 유 | 162.98 MiB | 148.28 MiB | 1.25 ms | 1.06 ms |
| 유 | 무 | 163.47 MiB | 148.74 MiB | 0.94 ms | 0.82 ms |
| 유 | 유 | 159.46 MiB | 145.72 MiB | 0.55 ms | 0.53 ms |

양쪽 Keep-alive 조건에서 합계 PSS는 직결이 **13.73 MiB, 8.6% 적었다**.
PSS 범위는 CLI **159.14–161.20 MiB**, 직결 **144.41–149.12 MiB**다.
지연 p95 범위는 각각 **0.530–0.550 ms**, **0.533–0.548 ms**로 겹친다.
두 앱의 결과이며 앱 수 증가에 따른 절감률로 외삽하지 않는다.

## Backend와 Traefik 비용 분리

아래는 **클라이언트가 연결을 재사용하는 조건**이다. Backend는 CLI 두 프로세스의 합계 또는
runner 한 프로세스다. 각 셀은 개별 중앙값이므로 항목별 중앙값의 합이 합계 중앙값과 항상 같지는 않다.

| 앱 지원 | 실행 방식 | Backend CPU | Traefik CPU | Backend PSS | 합계 요청당 CPU |
| --- | --- | ---: | ---: | ---: | ---: |
| 무 | CLI | 9.69% | 31.47% | 39.78 MiB | 0.824 ms |
| 무 | 직결 | 7.80% | 31.08% | 23.97 MiB | 0.778 ms |
| 유 | CLI | 3.80% | 15.49% | 38.49 MiB | 0.388 ms |
| 유 | 직결 | 3.70% | 15.49% | 23.74 MiB | 0.384 ms |

양쪽 Keep-alive 조건에서 backend PSS는 **38.49 → 23.74 MiB, 38.3% 감소**했다.
Traefik 자체의 PSS가 커서 배포 합계의 절감률은 8.6%다. 연결을 재사용하면 backend와
Traefik의 CPU 모두 줄었으며, 비용이 한쪽으로 이동한 것만으로 설명되지 않는다.

연산 응답은 요청마다 xorshift 200,000회를 수행하고 합계 200 req/s로 측정했다.
양쪽 Keep-alive 조건의 CPU 합계는 CLI **20.29%**, 직결 **20.59%**, 요청당 CPU는
각각 **1.015 ms**, **1.030 ms**였다. 이 조건에서도 두 실행 방식은 비슷했다.

## 부하 품질과 재측정

최초 3회 반복의 마지막 반복에서 CLI/앱 미지원/클라이언트 재사용 없음 조건의 요청 생성
지연 p95가 **164.500 ms**로 커졌다. 평균 처리율은 499.76 req/s, 오류는 0이었지만,
늦어진 요청이 몰려 들어간 구간이므로 정상적인 일정 부하의 비교에 사용하지 않았다.

해당 trial 하나만 빼는 대신 **그 반복의 8개 조건 전체를 같은 순서로 재측정**했다.
재측정 채택 기준은 오류 없음, 목표 처리율 99% 이상, 요청 생성 지연 p95 ≤ 5 ms로 정했다.
첫 두 반복은 전 구간에서 이 기준을 통과해 유지했다. CPU나 응답 latency가 높다는 이유로
제외하지 않았다. 지연 원인 전체를 확정하는 profiler 분석은 수행하지 않았다.

최초·재측정의 측정 구간 요청 **224,000개는 모두 성공**했다. 최종 집계에 채택한 24개 trial은
**168,000개 요청, 실패 0개**, HTTP 실제 처리율 **499.70–499.94 req/s**, 요청 생성 지연 p95
**1.04–1.08 ms**였다. 채택한 측정 구간의 cgroup CPU throttling은 없었다.
선택 목록과 최초 원본도 보존하며 `select.py`가 완전한 반복·동일 조건·부하 품질을 검사한다.

## 연결 재사용 확인

Traefik의 앞단과 뒷단은 연결 수명이 독립적이다. 앱이 Keep-alive를 지원하지 않아도
클라이언트→Traefik 연결은 유지될 수 있고, 클라이언트가 요청마다 연결을 닫더라도
Traefik→앱 연결은 재사용될 수 있다.

측정 전에 각 backend에 두 연결을 열어, 첫 연결이 idle인 동안 두 번째 연결이 응답하고
첫 연결도 다시 응답하는지 검증한다. 설정과 실제 연결 재사용이 다르면 측정기가 실패한다.

별도 연결 진단은 100 req/s, 2초 HTTP 구간으로 실행했다. 아래 값은 Wasmtime과 runner
양쪽 모두 동일했다. 본 측정의 CPU/메모리 집계에 진단 실행은 포함하지 않는다.

| 클라이언트 재사용 | 앱 지원 | 클라이언트 신규 / 재사용 | Backend 새 TCP 주소 쌍 |
| --- | --- | ---: | ---: |
| 무 | 무 | 200 / 0 | 200 |
| 무 | 유 | 200 / 0 | 0 |
| 유 | 무 | 0 / 200 | 200 |
| 유 | 유 | 0 / 200 | 0 |

Backend 값은 `/proc/net/tcp`에서 구간 전후 새로 관찰된 주소 쌍 수다.
ESTABLISHED·TIME_WAIT 등을 포함하고 LISTEN은 제외한다. tuple 재사용·만료로 누락될 수
있으므로 정확한 connect syscall 횟수와 같지는 않다. 클라이언트 값은 측정기 pool의 실제 계수다.

## 측정 방법과 해석 범위

- Linux x86-64, AMD Ryzen 7 5825U, cgroup CPU 4개 상당·메모리 8 GiB. 공유 호스트, affinity 미고정.
- Wasmtime 51.0.0, Traefik 3.8.0, Rust 1.100.0 release build. Runner 바이너리는 이전 보고서와
  동일한 SHA-256이며 제품 구현 `b07dd01547107a164e9f00195394b3ef0522d917`을 사용했다.
- 모든 조건에서 동일한 `bench-http.wasm`을 사용했다. WASI `poll`로 여러 idle 연결을 함께
  기다리며, 환경 변수 `BENCH_KEEPALIVE`의 0/1만 바꾼다. 주기적 sleep·busy loop·guest thread는 없다.
  작은 응답의 write/flush는 blocking이다. 느린 클라이언트나 큰 payload를 위한 범용 서버가 아니다.
- CLI는 on-demand allocator, disk compilation cache 없음, parallel compilation 없음으로 실행한다.
  두 구성 모두 게스트당 linear memory 한도 64 MiB, resource table 1,024개다. 별도 큰 heap을
  할당하지 않지만 stack·기본 heap은 사용한다. CPU/fuel quota는 없고 runner의 10 ms epoch clock은 유지한다.
- 프로세스마다 Tokio/Rayon worker 2개, Traefik GOMAXPROCS 2다. CLI의 host feature·executor가
  runner와 완전히 같지는 않다. 이 수치는 선택한 배포 구성의 비용 비교이며, 프로세스 통합 하나의
  순수한 효과나 모든 Wasmtime embedding의 일반적인 성능을 뜻하지 않는다.
- 두 Host route와 두 backend를 사용하고 트래픽은 50:50으로 보낸다. 실제 WASM socket으로
  연결하며 runner HTTP ingress를 통과하지 않는다. 모든 네트워크는 loopback, HTTP/1.1, TLS 없음이다.
- 각 trial마다 새 서버·Traefik 프로세스와 새 runner cache를 사용한다. 준비 후 2초 안정화,
  idle 10초, HTTP 500 req/s 10초, xorshift 200,000회 연산 응답 200 req/s 10초다.
  부하 앞에 각각 2초 warmup을 수행한다. 클라이언트 pool은 warmup부터 측정 끝까지 유지한다.
- 조건마다 3회 반복하고 순서를 회전·반전한다. 모든 응답의 status·body를 검증하며 자동 재시도는 없다.
  클라이언트 pool은 Host당 동시 연결 최대 32개, pipelining 없음이다. 빌드·다른 부하 실험과 겹치지 않았다.
- CPU는 서버 PID의 user+system 시간 합계로 **100% = 코어 1개**다. 부하 생성기 CPU는 제외한다.
  메모리는 공유 페이지를 나눠 계산한 process PSS이며 file cache·kernel socket buffer 등 전체 cgroup
  메모리는 포함하지 않는다. `/proc` 메모리 표본은 별도 thread로 수집한다.
- 이전 보고서는 단일 연결을 처리하고 응답마다 종료하는 `guest.rs`를 사용했다. 이번 fixture와
  HTTP 구현이 다르므로 **이전 보고서 수치를 이번 표의 분모로 섞지 않는다**.
- HTTP/2·gRPC, TLS, 큰 응답, 실제 Pod 간 네트워크, cold start·최대 처리량은 측정하지 않았다.

## 재현 자료

본 측정기는 `3a61e3b`, 연결 진단 측정기는 `ab81e34`의 소스다. 각 원본의
`script_sha256`을 해당 커밋과 대조했다. 최종 표는 선택 목록으로 만든 파생 데이터이며,
원본을 수정하지 않고 각 trial의 원본 파일·반복 번호를 보존한다.

모든 측정에 사용한 게스트 SHA-256은
`35353eba1921647867ce1ed582f14f1b152fc80c0ba918dae3d7672cf3912c0c`다.

[재현 명령](../../scripts/benchmarks/README.md#wasmtime-직접-실행과-runner-직결-keep-alive-비교) ·
[HTTP fixture](../../scripts/benchmarks/http-guest.rs) ·
[측정기](../../scripts/benchmarks/resources.py) ·
[집계기](../../scripts/benchmarks/summarize.py) ·
[선택 목록](2026-11-25-keepalive-selection.json) ·
[선택기](../../scripts/benchmarks/select.py) ·
[최초 측정 원본](2026-11-25-keepalive-initial.json) ·
[마지막 반복 재측정 원본](2026-11-25-keepalive-repeat2.json) ·
[중앙값·범위](2026-11-25-keepalive-summary.json) ·
[연결 진단 원본](2026-11-25-keepalive-connections.json)
