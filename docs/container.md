# 컨테이너로 실행

앱 등록부터 자동 증감까지 실행하려면 [Kubernetes native 튜토리얼](kubernetes-native.md)을 사용하세요. Pingora 프록시도 같은 WAFL 이미지에 포함됩니다.

동일한 `ghcr.io/lesomnus/wafl` 이미지에서 `serve runner`, `serve controller`, `serve node`, `serve proxy`와 CLI를 실행합니다. 현재 발행 플랫폼은 `linux/amd64`입니다. 이미지는 UID/GID 10001로 실행하며, WASM은 runner 프로세스 내부에서 실행됩니다.

```sh
docker pull ghcr.io/lesomnus/wafl:latest
docker run --rm ghcr.io/lesomnus/wafl:latest --help
```

main 변경은 이미지 빌드와 실행 검증을 통과한 뒤 `latest`와 `sha-<전체 Git commit SHA>` 태그로 발행됩니다. 재현 가능한 배포에는 SHA 태그 또는 registry digest를 사용합니다. 소스에서 빌드하려면 저장소 루트에서 `docker build -t wafl .`을 실행합니다.

GitHub Container Registry의 새 패키지는 기본적으로 private입니다. 패키지가 private이면 `read:packages` 권한으로 GHCR에 로그인한 뒤 pull합니다. 공개 배포는 GitHub 패키지 설정에서 visibility를 Public으로 변경한 뒤 anonymous pull로 확인합니다. public repository의 visibility만으로 container package까지 공개되지는 않습니다.

## Runner

```sh
docker volume create wafl-cache
docker run --rm --name wafl-runner \
  -p 8080:8080 -p 8081:8081 \
  -v wafl-cache:/var/lib/wafl/cache \
  ghcr.io/lesomnus/wafl:latest serve runner /var/lib/wafl/cache \
  --listen 0.0.0.0:8080 --control 0.0.0.0:8081
```

위는 proxy 모드 예제입니다. 직결 모드에서는 gateway가 runner의 실제 IP와 배정한 포트 범위에 접근할 수 있어야 합니다. Kubernetes에서는 Pod IP를 `direct.advertise`로 설정하고, 같은 IP의 예약을 관리하는 모든 runner가 별도의 영속 `direct.directory`를 공유합니다. [직결 계약](direct.md)을 따릅니다.

bind mount의 cache/예약 디렉터리는 UID 10001이 쓸 수 있어야 합니다. bundle cache는 지울 수 있는 객체 저장소이고, direct 예약 디렉터리는 재시작 후에도 보존할 상태입니다. 서로 같은 디렉터리나 하위 디렉터리로 설정하지 않습니다.

## 플랫폼

Kubernetes에 전체 플랫폼을 설치합니다. 동일 이미지에 Rust runner/CLI와 Go Virtual Kubelet/controller 실행파일이 들어 있습니다. `wafl`이 명령에 맞는 실행파일로 연결합니다.

```sh
wafl-kube bootstrap install --image ghcr.io/lesomnus/wafl:sha-COMMIT
```

controller 2개, runner Pod 2개, Pingora proxy과 RBAC·Service·CRD를 설치합니다. runner Pod 안의 node provider가 가상 Node를 등록합니다. 기본 데이터 디렉터리는 Pod의 `emptyDir`이며 runner 컨테이너 재시작 동안 캐시와 포트 예약을 유지합니다. 물리 Pod를 교체하면 새 IP/Node에서 실행을 배정합니다. 클러스터마다 runner capacity와 requests/limits, cache 용량, gateway 노출 방식을 조정하세요.

운영 예제의 실제 설치 원본은 [platform.yaml](../kubernetes/install/platform.yaml)입니다. 별도 PostgreSQL은 필요하지 않습니다. 플랫폼 API는 내부 Service로 사용하고 외부 앱 요청은 Pingora proxy으로 받습니다. 관리 API와 앱 트래픽 주소를 구분합니다.
