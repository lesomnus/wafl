# Runner OTLP 메트릭

runner는 프로세스 설정의 `metrics.endpoint`로 OTLP/HTTP protobuf 메트릭을 주기적으로 전송합니다. 앱의 Manifest나 게스트 SDK 설정은 필요하지 않습니다. endpoint가 있는 `metrics` 객체를 생략하거나 `null`로 지정하면 exporter·수집 스레드를 만들지 않습니다.

이 기능은 runner에서 수집기로 보내는 경로를 제공합니다. 제어평면의 메트릭 소비 방식, 스케줄링 정책과 상태 보고 RPC는 후속 계약입니다.

## 설정

```json
{
  "listen": "127.0.0.1:8080",
  "control": "127.0.0.1:8081",
  "metrics": {
    "endpoint": "http://otel-collector:4318/v1/metrics",
    "interval_ms": 10000,
    "timeout_ms": 3000
  }
}
```

| 필드 | 계약 |
| --- | --- |
| `endpoint` | 필수. HTTP(S)의 **완전한 metrics URL**. 경로를 추가하지 않고 그대로 사용. 기본 Collector는 `/v1/metrics` 사용 |
| `interval_ms` | 전송 주기. 생략 시 10,000 ms. 허용 범위 100–86,400,000 ms |
| `timeout_ms` | HTTP 전송 제한 시간. 생략 시 3,000 ms. 허용 범위 1–5,000 ms |

전송 형식은 OTLP/HTTP protobuf로 고정하며 `POST`와 `application/x-protobuf`를 사용합니다. HTTPS는 시스템 신뢰 인증서를 사용하고, HTTP redirect는 따라가지 않습니다. OTLP/gRPC 전송은 아직 제공하지 않습니다.

설정 파일의 endpoint·전송 주기·타임아웃은 명시적으로 적용합니다. `OTEL_EXPORTER_OTLP_ENDPOINT` 등의 환경변수만으로 export를 켜지는 않습니다. 내부 OpenTelemetry SDK가 지원하는 기타 환경변수 설정은 SDK 규칙을 따르지만, WAFL의 인증·credential 설정 계약은 이번 범위에 포함하지 않습니다. URL userinfo와 fragment는 거절합니다.

```sh
wafl serve runner /srv/wafl-cache --config runner.json
```

## 로컬 수집기에서 확인

[OpenTelemetry Collector](https://opentelemetry.io/docs/collector/installation/)의 `otelcol`을 설치한 환경에서 저장소 루트에서 실행합니다.

```sh
otelcol --config examples/otel-collector.yaml
```

예제 수집기는 `127.0.0.1:4318`에서 OTLP/HTTP를 받고 메트릭을 터미널에 출력합니다. 다른 터미널에서 runner를 시작합니다.

```sh
wafl serve runner /tmp/wafl-cache --config examples/runner-metrics.json
```

1초마다 `service.name: wafl-runner`, `wafl.runner.instances`, `wafl.runner.uptime` 등이 출력됩니다. 이 1초 주기는 예제 설정이며 기본값은 10초입니다.

별도 터미널에서 실제 앱을 추가하고 호출합니다.

```sh
bash scripts/bundle.sh /tmp/wafl-example/hello.tar.gz
wafl instance add file:///tmp/wafl-example/hello.tar.gz --alias hello
instance_id=$(wafl instance get @hello | cut -f1)
curl -H "wafl-instance: $instance_id" http://127.0.0.1:8080/

wafl instance stop @hello
wafl instance erase @hello
```

`wafl.runner.instances`의 `running`이 증가하고, Stop 이후 `stopped`로 이동합니다. Erase 이후 해당 Instance의 활성 lease gauge가 다음 수집에서 사라집니다. 수집기·저장소에 이미 기록한 과거 데이터는 별도로 남을 수 있습니다.

## 전송하는 값

| 이름 | 종류·단위 | 의미 |
| --- | --- | --- |
| `wafl.runner.instances` | Gauge, `{instance}` | 공개된 Instance 수. `wafl.instance.phase`로 stopped/starting/running/failed/stopping 구분. 각 phase는 0도 전송 |
| `wafl.runner.additions.pending` | Gauge, `{instance}` | Add가 예약하고 아직 공개하지 않은 Instance ID 수 |
| `wafl.runner.additions.active` | Gauge, `{operation}` | 사용 중인 Add 슬롯. 취소 이후 계속 정리 중인 컴파일 작업도 포함할 수 있음 |
| `wafl.instance.active` | Gauge, `{lease}` | Instance의 기존 `active` 값. `wafl.instance.id` UUID로 구분 |
| `wafl.runner.uptime` | Gauge, `s` | 이 runner 메트릭 세션의 경과 시간 |
| `wafl.runner.metrics.failures` | 누적 Counter, `{attempt}` | 이 세션의 OTLP export 실패 횟수. 수집기 복구 후 확인 가능 |
| `process.cpu.time` | 누적 Counter, `s` | Linux runner 프로세스 전체의 user+system CPU 시간 |
| `process.memory.usage` | Gauge, `By` | Linux runner 프로세스 전체의 RSS |

`active`는 기동 대기와 HTTP/gRPC 응답 body·trailers 전달이 끝날 때까지의 요청을 포함합니다. Add/Start가 실행 준비를 기다리는 임시 lease도 포함하므로 순수 HTTP 요청 수와 완전히 같지는 않습니다. 공개 전 Add의 lease는 Instance별 gauge에 나타나지 않습니다. 긴 gRPC 스트림은 활성 상태로 남고, 여러 요청이 샘플 사이에 완료되면 이 gauge만으로 처리량을 구할 수 없습니다.

위 요청 계측은 기본 proxy 모드 기준입니다. [직결 모드](direct.md)의 HTTP/gRPC 트래픽은 runner 계측을 지나지 않습니다. 이 모드에서는 `wafl.instance.active`를 생략하며, `Instance.active`와 `wafl.runner.requests`는 Add/Start 관리 대기만 나타냅니다. 실행 수·선언 메모리 예약·프로세스 CPU/RSS는 계속 전송합니다. 앱 요청의 부하와 유휴 여부는 앞단 L7에서 관측해야 합니다.

CPU와 RSS는 `/proc/self/stat`에서 읽습니다. CPU 시간은 백분율이나 할당량이 아니며, 모든 WASM·컴파일·프록시·메트릭 전송 비용을 포함합니다. RSS도 게스트별 linear memory나 Pod/cgroup 메모리 사용량과 다릅니다. 다른 OS이거나 해당 통계를 읽지 못하면 두 process 메트릭을 생략합니다. 0으로 대체하지 않습니다. 인스턴스별 CPU 계측, cgroup 용량·제한, 게스트 실제 메모리 사용량과 요청 지연 histogram은 후속 범위입니다. 선언된 메모리 예약 합계는 아래 새 gauge로 제공합니다.

각 export의 resource에는 `service.name=wafl-runner`, 빌드의 `service.version`, 서버 시작마다 새로 생성하는 `service.instance.id`가 있습니다. 이 ID는 메트릭 세션 식별자이며 Instance entity ID나 영속 runner ID가 아닙니다. runner 재시작 후의 시계열을 이전 실행과 구분할 수 있습니다. 계측 scope는 `wafl.runner`입니다.

값은 짧은 잠금으로 읽은 관측값이며, 한 batch 전체에 대한 원자적 상태 스냅샷이나 자원 예약 보장은 아닙니다. env·번들 URL·로그 내용은 이 메트릭에 넣지 않습니다.

## 수집기 장애와 종료

수집·전송은 SDK의 별도 스레드에서 실행합니다. 수집기 연결 성공을 기다리지 않고 runner를 시작하며, 장애가 Add·앱 트래픽을 대기시키지 않습니다. 잘못된 설정 형식이나 exporter 초기화 오류는 시작 단계에서 보고합니다.

요청이 실패하거나 타임아웃이면 실패를 계수하고 다음 주기에 최신 값을 보냅니다. 이전 gauge 샘플을 무제한 큐에 쌓거나 디스크에 보관하지 않습니다. 따라서 장애 구간의 순간 값은 유실될 수 있습니다. Counter는 누적값으로 보내며 같은 세션이 살아 있으면 이후 성공한 export에서 누적값을 볼 수 있습니다. 실패 진단은 stderr에 최대 분당 한 번 출력합니다.

정상 종료 시 마지막 수집·전송을 시도합니다. SDK 종료 대기는 제한되며 수집기 장애가 프로세스를 무기한 유지하지 않습니다. SIGKILL·프로세스 충돌 시 마지막 export는 보장하지 않습니다.

확인 시나리오:

- **수집기 중지/복구:** 수집기를 중지해도 앱 호출은 계속 처리됩니다. 다시 실행하면 최신 메트릭과 증가한 export 실패 횟수를 받습니다.
- **Runner 재시작:** 같은 캐시 디렉터리를 사용해도 `service.instance.id`는 바뀝니다. Instance 목록은 기존 계약대로 다시 제출해야 합니다.
- **Kubernetes:** endpoint를 접근 가능한 Collector Service 주소로 지정합니다. collector가 다른 Pod에 있다면 localhost 대신 Service DNS를 쓰고, Collector receiver도 Pod 외부 연결을 받는 주소로 설정합니다.

## 검증

```sh
bash scripts/test.sh --test cli metrics::
```

실제 `wafl serve runner` 프로세스의 HTTP POST를 로컬 테스트 수집기가 받아 OTLP protobuf로 디코딩합니다. 사용자 지정 URL 경로, resource/session, Instance 생성·중지·삭제, Linux CPU/RSS, 종료 시 마지막 export, 느린 수집기의 타임아웃과 복구를 검증합니다.

## 수락·기동·컴파일 관측

`service.instance.id`는 `Runner.id`/`Instance.session`의 UUID 문자열입니다. 프로세스 재시작 시 새 값이 되므로 runner 조회와 메트릭을 같은 세션으로 연결할 수 있습니다. pending Add는 Get/List에서 조회 가능하지만 `wafl.runner.instances`에서는 계속 완료된 항목만 세고 pending gauge로 별도 집계합니다.

| 이름 | 종류·단위 | 의미 |
| --- | --- | --- |
| `wafl.runner.startups` | Gauge, `{instance}` | 실행을 시작해 listen을 기다리는 시도 |
| `wafl.runner.executions` | Gauge, `{instance}` | 대기/기동/실행의 예약 수 |
| `wafl.runner.memory.reserved` | Gauge, `By` | 해당 예약의 Manifest memory_bytes 합계. RSS 아님 |
| `wafl.runner.requests` | Gauge, `{lease}` | runner 전체 실행 lease |
| `wafl.runner.waiting` | Gauge, `{instance}` | startup 슬롯 대기 시도 |
| `wafl.runner.compilations` | Gauge, `{component}` | 사용 중/미참조/컴파일 중인 코드 cache entry |
| `wafl.runner.compilation.requests` | Counter, `{operation}` | `result=compiled/reused`로 실제 컴파일·공유 횟수. 실패한 컴파일도 포함 |
| `wafl.runner.accepting` | Gauge, `1` | 새 실행 수락 가능 시 1, 프로세스 종료 시 0 |

설정 한도와 현재 usage는 `RunnerService.Get`으로도 조회합니다. 이 API는 배치 제어평면이나 metric 수집기를 구현하지 않으며, 제어평면의 메트릭 소비 방식은 후속으로 정합니다.
