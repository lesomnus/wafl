# App 환경 변수와 Secret

일반 설정은 `manifest.env`, 비밀 값은 App의 `secrets`에서 지정합니다. 목적지 환경 변수 이름은 앱이 정합니다. `DATABASE_URL`은 WAFL 전용 필드가 아닙니다.

로컬 YAML 파일은 CLI가 읽습니다. 상대 경로는 YAML 파일이 있는 디렉터리를 기준으로 해석하며, 내용을 생성된 App API의 쓰기 전용 `secretValues`로 controller에 전달합니다. controller가 Kubernetes Secret으로 저장하고 App에는 참조만 남깁니다. CLI는 Kubernetes에 직접 접근하지 않습니다.

```yaml
alias: hello
domain: hello.localhost
manifest:
  bundle:
    url: file:./hello.tar.gz
secrets:
  - env: GREETING
    file: ./greeting
```

```sh
printf 'Hello' > greeting
wafl app add app.yaml
```

Kubernetes 리소스를 직접 선언하거나 이미 존재하는 Secret을 쓰려면 같은 namespace의 `secretKeyRef`를 지정합니다.

```yaml
apiVersion: wafl.opt.td/v1alpha1
kind: App
metadata:
  name: hello
  namespace: demo
spec:
  domain: hello.example.com
  manifest:
    bundle:
      url: https://files.example.com/hello-v1.tar.gz
      digest: sha256:REPLACE_WITH_ARCHIVE_SHA256
  secrets:
    - env: DATABASE_URL
      secretKeyRef:
        name: database
        key: url
        optional: false
```

source Secret의 UID/resourceVersion이 바뀌면 controller는 불변 snapshot을 만들고 Deployment를 갱신합니다. 새 실행이 준비되면 새 요청이 이동하고 이전 스트림이 끝난 뒤 이전 실행을 정리합니다. 이전 ReplicaSet은 이전 snapshot을 계속 참조하므로 실행 재생성 도중 값이 뒤섞이지 않습니다. 참조되지 않는 오래된 snapshot은 GC하며 보관된 rollback ReplicaSet이 쓰는 snapshot은 남깁니다.

source가 없거나 필수 key가 빠지면 `Accepted=False`로 보고하고 이전 정상 Deployment를 유지합니다. `optional: true`이면 없는 source/key는 환경 변수에서 생략합니다. source의 파일 변경을 감시하지는 않습니다. 자동 교체에는 Kubernetes Secret을 갱신하세요. CLI 파일 입력을 바꾸려면 새 파일을 가리키는 `secrets` YAML을 작성하고 `wafl app patch @hello --file secrets.yaml`로 제출합니다. controller가 새 Secret을 만들고 참조를 교체합니다.

값은 UTF-8이어야 하며 NUL은 허용하지 않습니다. 파일 끝의 개행도 그대로 보존합니다. 바인딩은 최대 64개, 값 하나는 16KiB, 환경 변수 이름과 값을 합친 총량은 64KiB입니다. 중복 이름, `PORT`, `manifest.env`와의 충돌은 거절합니다.

App·Pod template·Instance 응답에는 해석한 값을 넣지 않습니다. source와 snapshot은 Kubernetes Secret 저장·접근 정책을 따르며 runner 메모리에는 실행 동안 값이 존재합니다. 앱이 stdout/stderr에 값을 출력하면 로그에도 남습니다. 번들 다운로드 credential은 별도 기능이며 아직 지원하지 않습니다.
