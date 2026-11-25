# App secret 환경 변수

`App.secrets`는 **source를 읽어 지정한 환경 변수에 넣는 참조 목록**입니다. 앱은 `std::env::var("DATABASE_URL")` 등 언어의 환경 변수 API를 사용합니다. `DATABASE_URL`은 예제 이름이며 WAFL의 예약 이름이 아닙니다.

일반 `app.yaml`은 파일을, Kubernetes App CRD는 같은 namespace의 `Secret` key를 참조합니다. App·Placement에는 참조만 저장합니다. controller가 새 Instance를 생성할 때 source를 읽고 runner의 메모리에 전달합니다.

## 파일: 로컬 개발·자체 운영

파일은 **controller 호스트에 있는 파일**입니다. CLI가 업로드하거나 runner의 bundle에 포함하지 않습니다. [controller 구성](controller.md)이 실행 가능한 상태에서 파일을 준비합니다.

```sh
install -d -m 700 "$HOME/.config/wafl/secrets"
printf '%s' 'Hello from a secret' > "$HOME/.config/wafl/secrets/greeting"
chmod 600 "$HOME/.config/wafl/secrets/greeting"
```

controller JSON 설정에 tenant별 허용 디렉터리를 추가합니다. 아래 경로는 실제 **절대 경로**로 바꿉니다. 설정에서 `$HOME`이나 `~`를 확장하지 않습니다.

```json
"secrets": {
  "files": {
    "00000000-0000-8000-8001-000000000001": "/home/me/.config/wafl/secrets"
  }
}
```

이 UUID는 기본 로컬 tenant입니다. 디렉터리는 controller 시작 전에 존재해야 합니다. 컨테이너에서는 읽기 전용 mount를 연결합니다. 여러 controller replicas의 디렉터리 경로는 달라도 되지만 같은 tenant의 같은 상대 경로에서 동일한 내용을 읽을 수 있어야 합니다. 허용 tenant 목록은 공유 설정 fingerprint에 포함됩니다. 기존 DB에 목록을 추가·변경할 때는 모든 controller를 중지한 뒤 한 개를 `wafl serve controller --config controller.json --configure`로 시작하고 나머지를 같은 공유 설정으로 시작합니다.

`app.yaml`:

```yaml
alias: hello
domain: hello.example.com
manifest:
  bundle:
    url: https://assets.example.com/hello.tar.gz
  env:
    LOG_LEVEL: info
secrets:
  - env: GREETING
    file: greeting
replicas: 1
```

```sh
wafl --endpoint http://localhost:8082 app add app.yaml
wafl --endpoint http://localhost:8082 --json app get @hello
```

조회에는 `env`와 `file` 참조만 나옵니다. 환경 변수에는 파일 전체 내용이 들어갑니다. 빈 파일은 빈 문자열이며 마지막 개행도 유지합니다. trim, 템플릿 치환, dotenv 파싱은 하지 않습니다. `file`은 허용 디렉터리 기준 상대 경로입니다. 절대 경로·`..`·디렉터리 밖으로 향하는 symlink는 거절하며 일반 파일만 지원합니다.

DB URL도 같은 방식입니다. URL 내용이 있는 `db-url` 파일을 준비하고 `env: DATABASE_URL`, `file: db-url`로 참조합니다. 앱이 DB 클라이언트로 직접 연결하며 WAFL이 DB를 생성하지는 않습니다.

## Kubernetes: 기존 Secret 참조

[Kubernetes provider](kubernetes.md)를 설정하고 controller ServiceAccount에 해당 namespace의 `secrets/get` 권한을 부여합니다. [RBAC 예제](../deploy/kubernetes/rbac.yaml)에 포함되어 있으며 Secret `list`·`watch`·쓰기 권한은 필요하지 않습니다. 앱 작성자는 해당 namespace의 Secret을 앱에 제공할 수 있으므로 namespace와 tenant의 매핑도 그 권한에 맞게 설정합니다.

기존 Secret의 `name`과 `key`를 지정합니다. 아래는 실행 예제를 위한 가짜 값입니다. 실제 값은 앱 YAML이나 Git 저장소에 넣지 않습니다.

```sh
kubectl create secret generic greeting -n demo \
  --from-literal=value='Hello from Kubernetes'
```

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
      url: https://assets.example.com/hello.tar.gz
  secrets:
    - env: GREETING
      secretKeyRef:
        name: greeting
        key: value
    - env: OPTIONAL_TOKEN
      secretKeyRef:
        name: optional-service
        key: token
        optional: true
  replicas: 1
```

`secretKeyRef`는 Kubernetes의 [SecretKeySelector 방식](https://kubernetes.io/docs/tasks/inject-data-application/distribute-credentials-secure/)을 따릅니다. namespace는 App의 `metadata.namespace`로 고정됩니다. 다른 namespace 참조나 CRD의 로컬 `file` 참조는 지원하지 않습니다.

`optional` 기본값은 `false`입니다. 필수 Secret이나 key가 없으면 App 선언을 유지하고 Instance 생성을 재시도합니다. `Placement.error`와 CRD Ready 상태에서 실패를 확인할 수 있습니다. `optional: true`는 Secret/key가 없을 때 환경 변수를 생략합니다. 권한 부족, API 장애, 잘못된 값은 optional이어도 실패합니다. 같은 Secret의 여러 key는 한 번 읽은 객체에서 가져옵니다. 서로 다른 Secret이나 파일 사이의 원자적 snapshot은 보장하지 않습니다.

## 변경·삭제와 적용 시점

값은 **Instance 생성 시점의 snapshot**입니다. source 내용만 바꾸면 기존 Instance는 기존 값을 유지합니다. 같은 Instance를 Stop/Start해도 다시 읽지 않습니다. 이후 확장·재배치·0개에서 기동으로 생성하는 새 Instance는 최신 값을 읽습니다. Kubernetes의 [Secret 환경 변수](https://kubernetes.io/docs/concepts/configuration/secret/)처럼 실행 중 환경 변수를 갱신하지 않으며, WAFL의 재조회 단위는 새 Instance 생성입니다.

전체 배포를 전환하려면 새 파일명 또는 Secret 이름/key로 참조를 바꿉니다. 참조 변경은 App revision을 증가시켜 기존 배포 전환·drain 절차를 따릅니다. source 내용 변경 감시나 자동 재시작은 하지 않습니다.

일반 App은 목록 파일로 참조를 교체합니다. `--secrets`는 다른 patch 옵션과 함께 사용하지 않습니다.

```yaml
# secrets.yaml
- env: DATABASE_URL
  file: db-url-v2
```

```sh
wafl --endpoint http://localhost:8082 app patch @hello --secrets secrets.yaml
printf '[]\n' > empty-secrets.yaml
wafl --endpoint http://localhost:8082 app patch @hello --secrets empty-secrets.yaml
```

CLI는 생성된 `AppService.Patch`로 목록을 교체하고 `[]`는 `AppService.Apply`의 빈 목록 대입으로 명시적으로 삭제합니다. 일반 App의 고정 replica는 [기존 Placement 회수 계약](controller.md)을 따릅니다. 자동 증감 App과 CRD App은 gateway 관측을 통해 drain·회수합니다. Kubernetes에서는 `spec.secrets`를 수정하거나 `secrets: []`로 비웁니다. CRD 소유 App은 WAFL CLI로 수정할 수 없습니다.

## 한도와 값의 취급

이 기능 이전 버전에서 업그레이드할 때는 runner를 먼저 업데이트하고, 기존 controller replicas를 모두 중지한 뒤 새 controller 한 개를 `--configure`로 시작합니다. 나머지 replicas도 새 버전으로 시작합니다. 공유 계약 버전을 올려 이전 controller가 secret 필드를 무시한 채 배치하는 것을 막습니다. 기존 App과 Placement에는 빈 참조 목록이 자동으로 추가됩니다.

- 최대 64개 binding, 값 하나는 최대 16 KiB, 이름과 값의 합은 최대 64 KiB입니다.
- 값은 UTF-8 문자열이며 NUL을 포함할 수 없습니다. binary Secret은 지원하지 않습니다.
- `env`는 최대 128자의 `[A-Za-z_][A-Za-z0-9_]*`입니다. `PORT`, 중복 이름, `manifest.env`에 이미 있는 이름은 거절합니다.
- 생성된 `InstanceService.Add`의 write-only `secrets` map으로 값을 전달합니다. Get/List/Start/Stop 등 응답이나 public Manifest에는 넣지 않습니다. controller는 `secret-env-v1` capability가 있는 runner에만 secret을 사용하는 App을 배치합니다.
- App·Placement·audit 기록과 캐시에는 해석한 값을 저장하지 않습니다. source 오류는 값을 포함하지 않으며, secret을 가진 guest의 실행 실패 메시지도 stderr를 관리 메타데이터에 복사하지 않습니다.

앱 코드가 값을 stdout/stderr나 HTTP 응답에 넣으면 해당 로그/응답에는 값이 드러납니다. WAFL은 앱 출력 전체를 필터링하지 않습니다. 관리 API의 기존 인증·네트워크 계약도 그대로이므로 controller→runner 통신은 신뢰할 수 있는 관리망에서 운영합니다. 이 기능은 별도의 secret vault, 저장소 암호화, bundle 다운로드 credential 기능을 제공하지 않습니다.
