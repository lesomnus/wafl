# 실행 번들 v1

runner의 입력은 protobuf `InstanceAddRequest`입니다. `Instance`의 `Manifest` 필드가 실행 설정을 담고, `manifest.bundle`의 `Asset.url`이 번들 위치를 가리킵니다. manifest 자체를 tar 안에 넣지 않습니다.

```text
hello.tar.gz
├── main.wasm          # 필수: WASIp2 CLI component
└── data/              # 선택: 게스트의 /data
    ├── index.html
    └── static/style.css
```

## 전달

`Asset`은 `oneof kind { string url = 1; }`과 선택적인 `string digest = 2`를 가집니다. HTTP(S) URL과 로컬 `file:/path`·`file:///path`를 지원합니다. URL은 **runner에서 접근 가능한 주소**여야 합니다. file URL은 runner 호스트의 파일이며 CLI가 파일을 업로드하지는 않습니다. HTTP 예제의 `127.0.0.1`은 runner와 파일 서버가 같은 호스트에서 실행될 때 사용합니다. Pod에서는 해당 Pod가 접근할 수 있는 서비스 주소, object storage 또는 마운트한 파일 경로를 사용합니다.

query를 포함한 서명 URL과 사내 HTTP 저장소를 사용할 수 있습니다. URL userinfo와 fragment는 받지 않으며, 별도의 다운로드 인증 헤더 설정은 없습니다. URL은 `Instance.manifest`에 유지됩니다. HTTP redirect는 최대 5번, 연결 제한은 5초, 다운로드 전체 제한은 30초입니다. HTTP 2xx 응답의 본문 자체가 gzip 압축된 tar여야 합니다.

기본 한도는 압축 다운로드 64 MiB, 압축 해제한 tar 스트림 256 MiB(메타데이터·패딩 포함), 실제 파일·디렉터리 항목 4,096개입니다. runner 설정의 `bundle.download_bytes`, `bundle.expanded_bytes`, `bundle.entries`로 변경합니다. 모두 양수이며 `expanded_bytes`는 `u64::MAX`보다 작아야 합니다. 크기를 미리 알 수 없는 응답도 다운로드 도중 제한하고, cache hit도 receipt의 실제 크기·항목 수를 현재 한도와 비교합니다. 공유 캐시 총량은 별도의 `cache.capacity_bytes`입니다. [설정과 GC 계약](store.md)을 참고하세요.

## 파일 규칙

- 루트의 비어 있지 않은 일반 파일 `main.wasm`이 필수입니다. 현재 Wasmtime으로 실행 가능한 WASI Preview 2 `wasi:cli/run` component를 사용합니다.
- 루트에는 `main.wasm`과 `data/`만 허용합니다. `data/`가 없으면 빈 디렉터리를 제공합니다.
- UTF-8 상대 경로를 사용합니다. 일반적인 `./` 접두사와 GNU long-name/PAX local 메타데이터를 처리합니다. 경로 길이는 최대 4,096바이트입니다.
- 절대 경로, `..`, 역슬래시, 중복 경로, symlink·hardlink·device·FIFO는 거절합니다. 파일과 디렉터리만 추출하며 원래 owner·권한 비트는 적용하지 않습니다.
- gzip checksum과 footer까지 확인합니다. tar 종료 뒤에는 0 패딩만 허용합니다.

예제 번들은 다음 명령으로 만듭니다. 빌드 결과 경로는 `cargo metadata`로 읽으므로 `CARGO_TARGET_DIR`도 적용됩니다.

```sh
bash scripts/bundle.sh /tmp/wafl-example/hello.tar.gz
tar -tzf /tmp/wafl-example/hello.tar.gz
```

직접 묶을 때는 staging 디렉터리에 `main.wasm`과 `data/`를 놓고 `tar -czf bundle.tar.gz -C staging main.wasm data`를 실행합니다. 둘을 감싸는 상위 디렉터리는 넣지 않습니다.

## 실행 수명

`Add`는 캐시 조회, 필요 시 다운로드·검증·압축 해제, 컴파일과 선언된 TCP 포트의 listen 완료까지 기다립니다. HTTP health check를 보내거나 앱 응답 내용까지 검증하지는 않습니다. 진행 중에는 Get/List에 PREPARING/STARTING 상태를 표시합니다. 실패하거나 생성 중 요청이 취소되면 미완성 Instance를 목록에서 제거하고 임시 파일과 실행을 정리합니다. 이미 검증·공개된 캐시 object는 유지할 수 있습니다. 압축 해제·컴파일 작업이 이미 실행 중이면 해당 작업이 끝난 뒤 임시 파일을 회수합니다.

추출된 파일은 [공유 저장소](store.md)에 내용 hash로 보관하고, 컴파일 결과는 동일 코드끼리 프로세스 안에서 공유합니다. [컴파일 보관 한도](operations.md#수락-제한)를 적용합니다. `/data`는 읽기 전용이며 앱의 변경 가능한 상태는 [외부 DB에 직접 연결](network.md)해 저장합니다. 호스트의 다른 디렉터리는 게스트에 제공하지 않습니다.

`Stop`과 유휴 회수는 WASM 실행·메모리·소켓만 종료합니다. 다음 요청이나 `Start`는 보관한 번들로 다시 실행하므로 URL이 만료되어도 재시작할 수 있습니다. 같은 URL을 다시 다운로드하거나 실행 중 변경을 감지하지 않습니다. 새 코드는 새 Instance로 `Add`하고 gateway의 대상을 변경합니다.

`Erase`는 인스턴스·alias·로그와 실행 메모리를 제거합니다. 다른 runner도 사용할 수 있는 공유 파일 캐시는 삭제하지 않습니다. runner 종료 후에도 캐시는 남지만 인스턴스 목록은 사라지므로 플랫폼이 필요 시 다시 제출해야 합니다.

URL reference는 cache-first이며 같은 URL의 내용 변경을 자동 감지하지 않습니다. 새 내용을 적용하려면 새 URL 또는 tar.gz의 SHA-256인 `Asset.digest`를 전달합니다. Add 응답의 `manifest.bundle.digest`에는 실제 사용한 digest가 채워집니다. 현재 ORAS adapter와 credential은 제공하지 않습니다.
