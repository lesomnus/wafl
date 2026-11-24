# 공유 번들 저장소 v1

이 계약은 같은 로컬 파일시스템의 디렉터리를 사용하는 여러 runner 프로세스에 적용합니다. 프로세스 간 파일 잠금과 같은 파일시스템의 atomic rename을 요구합니다. 현재 Linux의 로컬 파일시스템에서 검증하며 NFS·분산 파일시스템의 동작까지 보장하지 않습니다. 쓰기 주체는 이 계약을 따르는 신뢰된 runner/운영자입니다.

## 위치와 레이아웃

`wafl serve runner [DIRECTORY]` 또는 `--directory`가 경로를 지정합니다. 생략하면 JSON 설정의 `directory`, 절대 경로인 `$XDG_CACHE_HOME/wafl`, `$HOME/.cache/wafl` 순으로 결정합니다. XDG 기본값은 [XDG Base Directory Specification](https://specifications.freedesktop.org/basedir/)을 따릅니다.

```text
<directory>/
└── v1/
    ├── objects/sha256/<bundle hash>/
    │   ├── main.wasm
    │   ├── data/
    │   └── ready.json
    ├── refs/sha256/<source URL hash>.json
    ├── locks/source-<source URL hash>
    ├── locks/object-<bundle hash>
    └── tmp/<source URL hash>-<random>/
```

Bundle hash는 원본 tar.gz 전체의 SHA-256입니다. URL hash는 파서가 정규화한 URL 문자열 UTF-8의 SHA-256이며 URL 원문을 인덱스에 저장하지 않습니다. hex는 소문자 64자입니다.

`ready.json`과 source reference의 형식은 아래와 같습니다.

```json
{"version":1,"digest":"sha256:<bundle hash>"}
```

`ready.json`은 번들이 검증·추출되었음을 뜻합니다. WASM 컴파일이나 앱의 실행 준비 상태를 나타내지는 않습니다. Instance 상태는 각 runner가 관리합니다. 향후 호환되지 않는 레이아웃은 다른 버전 디렉터리를 사용합니다.

## 읽기·공개 순서

1. 요청의 `Asset.digest`가 있고 완성된 해당 object가 있으면 그대로 사용합니다.
2. 정규화된 URL의 source lock을 독점 획득합니다. 기다리는 요청은 취소할 수 있습니다. 잠금 후 캐시를 다시 확인합니다.
3. digest가 없는 요청은 URL reference가 가리키는 완성된 object를 사용합니다. object가 없으면 다운로드합니다. 잘못된 receipt는 오류로 처리합니다.
4. 해당 source lock을 보유한 상태에서 그 URL에 속한 버려진 staging만 정리합니다. 고유한 `v1/tmp/<source hash>-...`에 파일을 받아 크기·digest를 확인하고 추출 규칙을 검증합니다.
5. 내용 hash의 object lock을 획득합니다. 다른 URL을 통해 같은 내용을 받은 프로세스도 이 잠금으로 공개를 직렬화합니다.
6. 다른 프로세스가 이미 완성한 같은 object가 있으면 재사용합니다. 없으면 `ready.json`을 작성한 staging 디렉터리를 최종 object 경로로 한 번에 rename합니다. 이미 공개된 object를 수정하거나 덮어쓰지 않습니다.
7. source reference도 임시 파일을 작성한 뒤 rename하여 교체합니다. 잠금을 해제합니다.

독자는 공개된 object만 읽습니다. 같은 URL의 동시 cache miss는 하나만 다운로드하며, URL이 달라도 같은 내용은 object 하나로 합쳐집니다. **staging을 캐시 파일시스템 안에 두므로 `/tmp`가 다른 mount여도 atomic rename이 유지됩니다.** 같은 URL의 잠금 → object 잠금 순서를 바꾸지 않습니다.

잠금은 OS 파일 잠금이며 PID 파일의 존재 여부로 판단하지 않습니다. 프로세스가 죽으면 OS가 잠금을 해제합니다. `locks/` 파일은 삭제하지 않습니다. 삭제 후 다시 만들면 같은 이름을 서로 다른 inode로 잠그는 문제가 생깁니다.

취소된 압축 해제·공개 작업이 이미 blocking worker에서 실행 중이면 그 작업은 끝날 때까지 잠금과 동시성 슬롯을 유지합니다. 정상 취소는 staging을 정리하고, 강제 종료로 남은 staging은 다음 source lock 소유자가 회수합니다. 공개된 object만 남고 reference 기록 전 프로세스가 죽으면 이후 요청이 내용을 다시 받아 같은 object에 합칠 수 있습니다. 이 버전은 프로세스 중단 중 공개의 원자성을 보장하며 전원 장애에 대한 완전한 디스크 내구성까지 약속하지 않습니다.

## URL과 digest

URL reference는 **cache-first**입니다. 파일 서버가 사라졌거나 file URL의 원본이 없어도 캐시에 있는 내용을 사용합니다. 동일 URL의 서버 내용 변경을 ETag·mtime으로 자동 감지하지 않습니다.

배포 내용이 바뀌면 버전이 다른 URL을 쓰거나 새 번들의 `Asset.digest`를 명시합니다. 명시한 digest는 URL reference보다 우선하며, 다운로드 bytes와 다르면 인스턴스를 생성하지 않습니다. Add 응답에는 실제 사용한 digest를 `Instance.manifest.bundle.digest`에 채웁니다. 이미 같은 digest가 있으면 다른 URL로 요청해도 해당 object를 사용할 수 있습니다.

```sh
bundle_digest="sha256:$(sha256sum /tmp/wafl-example/hello.tar.gz | cut -d' ' -f1)"
wafl instance add file:///tmp/wafl-example/hello.tar.gz --digest "$bundle_digest"
```

캐시는 로컬의 신뢰된 immutable 저장소입니다. cache hit는 receipt와 기본 파일 구조를 확인하며 매번 추출된 모든 파일의 hash를 재계산하지 않습니다. 외부 프로세스가 공개된 파일을 수정하는 것은 이 계약에 포함되지 않습니다.

`Stop`은 실행만 중지합니다. `Erase`는 runner의 Instance·alias·로그를 제거하며 공유 object는 보존합니다. runner 종료 후에도 캐시는 남지만 인스턴스 목록은 남지 않습니다. 자동 GC와 실행 중 cache 삭제는 제공하지 않습니다. 캐시 정리는 해당 디렉터리를 쓰는 모든 runner를 중지한 상태에서 수행합니다. 컴파일 결과는 프로세스/인스턴스 메모리에 있으며 디스크 캐시에는 원본 WASM과 data를 보관합니다.

## 원격 data를 필요할 때 읽는 확장

현재 실행기는 완전히 추출된 로컬 `/data`를 WASI에 preopen합니다. 아직 원격 지연 읽기를 수행하지 않습니다. 코드상 소스 fetch, bundle 검증/저장(`store.rs`, `bundle.rs`), WASI 실행/파일 노출(`runtime.rs`)을 분리했으므로 아래 세 부분을 각자 확장할 수 있습니다.

1. **파일 인덱스:** 경로, 파일 크기, 파일 또는 chunk별 digest, 원격 위치/offset을 담는 immutable manifest. 전체 tar를 순회하지 않고 파일을 찾을 수 있어야 합니다.
2. **부분 fetch와 캐시:** 먼저 main.wasm 및 인덱스를 받고, data는 open/read 시 필요한 파일·chunk만 가져옵니다. 검증된 chunk를 내용 hash로 공유하며, v1과 별도 레이아웃/버전을 사용합니다.
3. **게스트 파일 API 연결:** 앱은 계속 `std::fs` 같은 WASI 파일 API를 씁니다. Wasmtime의 파일 호스트 구현 또는 FUSE 같은 mount 계층에서 읽기를 가로채 fetch·검증·캐시에 연결해야 합니다. 현재의 로컬 디렉터리 preopen만으로 자동 해결되지는 않습니다.

구현 후보는 파일별 object와 인덱스, 또는 seek 가능한 압축 이미지입니다. [eStargz 규격](https://github.com/containerd/stargz-snapshotter/blob/main/docs/estargz.md)은 tar.gz 호환 형식에 색인·chunk 검증을 추가해 필요한 파일 부분만 가져옵니다. 따라서 파일 확장자를 반드시 바꿀 필요는 없지만, 일반 gzip tar에 없는 인덱스와 부분 검증 계약은 필요합니다. OCI registry에 일반 tar.gz blob을 올리는 것만으로 지연 읽기가 생기지는 않습니다.

WAFL의 원격 파일 형식, Wasmtime 파일 계층 구현, cache GC와 ORAS fetch adapter는 후속 범위이며 이번 v1 계약에 지원된 것으로 포함하지 않습니다. Credential 작업도 보류 상태입니다.
