# 공유 번들 저장소

[직결 모드](direct.md)의 `direct.directory`는 이 캐시와 분리한 영속 포트 예약 저장소입니다. 서로 포함되는 경로는 거절하며 bundle GC가 endpoint 예약을 지우지 않습니다.

같은 로컬 파일시스템의 디렉터리를 여러 runner 프로세스가 공유합니다. OS 파일 잠금과 같은 파일시스템의 atomic rename을 요구하며 Linux 로컬 파일시스템에서 검증합니다. NFS·분산 파일시스템은 지원 계약에 포함하지 않습니다. 디렉터리는 신뢰된 runner와 운영자만 수정해야 합니다.

## 위치와 설정

`wafl serve runner [DIRECTORY]` 또는 `--directory`로 경로를 지정합니다. 생략하면 JSON 설정의 `directory`, 절대 경로인 `$XDG_CACHE_HOME/wafl`, `$HOME/.cache/wafl` 순입니다.

```json
{
  "listen": "127.0.0.1:8080",
  "control": "127.0.0.1:8081",
  "bundle": {
    "download_bytes": 67108864,
    "expanded_bytes": 268435456,
    "entries": 4096
  },
  "cache": { "capacity_bytes": 10737418240 }
}
```

표시한 값이 기본값입니다. 모든 한도는 양수입니다. `expanded_bytes`는 `u64::MAX`보다 작아야 합니다. `/data`는 읽기 전용이며 게스트가 쓸 수 있는 임시 FS는 제공하지 않습니다. `expanded_bytes`가 개별 번들의 FS 입력 상한이고 `cache.capacity_bytes`가 **공개된 v2 번들의 일반 파일 논리 크기 합계** 상한입니다. 같은 digest는 한 번만 셉니다.

파일시스템의 실제 블록·메타데이터, 압축 다운로드/추출 중 staging, 관리 경로 밖의 파일, 컴파일·실행 메모리는 캐시 상한에 포함하지 않습니다. 따라서 디스크 전체 hard quota는 아닙니다. staging은 Add마다 최대 `download_bytes + expanded_bytes`가 필요할 수 있고, 여러 프로세스의 동시 Add 수만큼 여유 공간을 확보해야 합니다. 실행 memory는 별도의 `Manifest.memory_bytes`로 제한하며 CPU/fuel 제한은 하지 않습니다.

공유 디렉터리의 살아 있는 runner는 같은 `cache.capacity_bytes`를 써야 합니다. 다르면 시작을 거절합니다. 번들별 한도는 runner마다 달라도 됩니다. cache hit에서도 기록한 실제 크기·항목 수를 검사합니다. 용량을 변경하려면 모든 관련 runner를 종료하고 새 값으로 시작합니다. 줄어든 상한을 맞추기 위한 시작 시 GC는 아래 Add 시 GC의 유일한 예외입니다.

## 레이아웃과 잠금

```text
<directory>/v2/
├── policy.json                    # 공유 capacity_bytes
├── catalog.lock                   # pin/공개/GC 직렬화
├── users.lock                     # 살아 있는 Store/Bundle의 공유 잠금
├── objects/sha256/<bundle hash>/
│   ├── main.wasm
│   ├── data/
│   ├── ready.json
│   └── lease                      # reader pin과 최근 사용 시각
├── refs/sha256/<source URL hash>.json
├── locks/source-<00..ff>           # 최대 256개 stripe
├── tmp/<source URL hash>-<random>/
└── trash/<bundle hash>/
```

Bundle hash는 원본 tar.gz 전체의 SHA-256입니다. URL hash는 정규화된 URL 문자열 UTF-8의 SHA-256입니다. 소문자 hex를 쓰며 URL 원문은 인덱스에 저장하지 않습니다. `ready.json`과 reference는 다음 정보를 갖습니다.

```json
{"version":2,"digest":"sha256:<64 lowercase hex>","download_bytes":123,"expanded_bytes":2048,"entries":2,"payload_bytes":12}
```

Receipt는 검증·추출 완료를 나타내며 컴파일/실행 준비 상태를 뜻하지 않습니다. 파일은 immutable이고 매 cache hit에서 전체 내용을 다시 hash하지 않습니다. 캐시의 저장 형식은 v2이며 내려받는 tar.gz의 형식은 [실행 번들 v1](bundle.md)입니다.

## 수명과 GC

1. Add가 catalog 잠금 아래 digest 또는 URL reference를 조회합니다. 존재하면 object의 `lease`에 공유 잠금을 잡고 반환합니다.
2. Miss면 URL hash 앞 2자리의 source stripe를 독점 잠급니다. 동일 URL의 다운로드를 합치며, 드물게 다른 URL도 같은 stripe를 기다립니다. 대기는 취소할 수 있습니다. 잠금 후 캐시를 다시 확인합니다.
3. source 잠금을 보유한 staging과 잠금 없이 버려진 다른 stripe의 staging을 정리합니다. 다운로드·추출은 catalog 밖에서 하며, 취소된 blocking 작업은 끝날 때까지 잠금과 Add 슬롯을 보유합니다.
4. 검증 후 catalog 잠금 아래 같은 digest가 있으면 재사용합니다. 없다면 `payload_bytes`만큼 공간을 확보합니다. 충분하면 GC를 하지 않습니다.
5. 부족하면 `lease` 독점 잠금을 즉시 얻을 수 있는 object만 후보로 삼습니다. **등록된 Instance가 참조하는 번들은 Running·Stopped·Failed 여부와 무관하게 보호됩니다.** 컴파일 중에도 pin을 유지합니다. 미참조 번들은 마지막 획득/해제 시각의 오래된 순서(LRU)로 회수합니다. 사용자 우선순위 필드는 없습니다.
6. 후보 총량이 부족하거나 번들 하나가 용량을 넘으면 `RESOURCE_EXHAUSTED`로 거절합니다. 부족한데 일부만 지우지는 않습니다. 실행 중 Instance를 내리지 않습니다.
7. 회수 대상은 잠금 아래 `trash`로 rename한 뒤 삭제합니다. 중간에 죽어도 다음 공간 확보가 trash를 청소합니다. 삭제된 object를 가리키는 reference도 정리합니다. staging은 같은 파일시스템에서 최종 object 경로로 rename해 공개하고 reference도 임시 파일에서 rename합니다.

`lease`는 Instance 및 이를 사용하는 작업의 수명 동안 유지합니다. `Stop`은 pin을 해제하지 않습니다. `Erase`·runner 종료 후 마지막 사용자가 해제되면 다음 Add의 GC 후보가 됩니다. 별도 주기적 sweeper는 없습니다. 프로세스가 죽으면 OS가 잠금을 해제합니다. catalog 잠금이 pin 열기와 삭제를 직렬화하므로 GC는 object 내부의 lease 파일도 안전하게 제거할 수 있습니다. 고정 catalog/users/source 잠금 파일은 삭제하지 않습니다.

잠금 순서는 source → catalog → object lease입니다. 다른 source의 staging 정리는 nonblocking try-lock만 사용합니다. 네트워크 다운로드와 WASM 컴파일은 catalog를 잡지 않습니다. 프로세스 중단에 대한 공개 원자성을 제공하지만 전원 장애 시 완전한 디스크 내구성까지 보장하지는 않습니다.

## URL과 운영 예시

URL reference는 cache-first입니다. 원본 서버나 로컬 파일이 없어도 캐시에 있으면 사용하며 ETag·mtime으로 갱신하지 않습니다. 새 코드는 새 URL 또는 `Asset.digest`를 제출합니다. digest는 URL reference보다 우선하고 다운로드 bytes와 다르면 거절합니다. Add 응답에는 실제 digest를 채웁니다.

- **여러 runner:** 같은 설정과 디렉터리로 두 프로세스를 시작하고 서로 다른 ingress/control 포트를 씁니다. 동일 번들은 한 object로 공유하지만 Instance·alias·로그는 각 runner에 독립적입니다.
- **캐시 가득 참:** 여유가 없고 기존 Instance가 모든 번들을 참조하면 Add가 실패합니다. 필요 없는 Instance를 Stop 후 Erase하고 Add를 다시 요청하면 해당 캐시를 회수할 수 있습니다. Stop만으로는 회수하지 않습니다.
- **프로세스 강제 종료:** pin은 OS가 해제합니다. 다음 miss는 버려진 staging을 청소하고 해당 object를 GC 후보로 사용할 수 있습니다.
- **용량 변경:** 공유 runner를 전부 종료한 다음 같은 새 용량으로 다시 실행합니다. 살아 있는 runner 하나만 다른 용량으로 바꾸면 시작에 실패합니다.

원격 URL도 전체 번들을 내려받아 검증·추출한 뒤 로컬 `/data`를 WASI에 preopen합니다. 파일별 지연 다운로드는 지원하지 않습니다.
