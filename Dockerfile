# syntax=docker/dockerfile:1
FROM rust:1.100-bookworm AS build
RUN apt-get update && apt-get install -y --no-install-recommends cmake \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /src
COPY . .
ENV CARGO_BUILD_JOBS=2
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/src/target \
    cargo build --locked --release --bin wafl && cp target/release/wafl /wafl

FROM golang:1.26-bookworm AS kubernetes
WORKDIR /src
COPY kubernetes/go.mod kubernetes/go.sum ./
RUN --mount=type=cache,target=/go/pkg/mod go mod download
COPY kubernetes/ .
RUN --mount=type=cache,target=/go/pkg/mod \
    --mount=type=cache,target=/root/.cache/go-build \
    CGO_ENABLED=0 go build -trimpath -ldflags='-s -w' -o /wafl-kube ./cmd/wafl-kube

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates libgcc-s1 \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --uid 10001 --create-home wafl \
    && mkdir -p /var/lib/wafl/cache /var/lib/wafl/endpoints \
    && chown -R 10001:10001 /var/lib/wafl
COPY --from=build /wafl /usr/local/bin/wafl
COPY --from=kubernetes /wafl-kube /usr/local/bin/wafl-kube
LABEL org.opencontainers.image.source="https://github.com/lesomnus/wafl" \
      org.opencontainers.image.description="WAFL WASM application controller and runner"
USER 10001:10001
WORKDIR /var/lib/wafl
EXPOSE 8080 8081 8082 8083
ENTRYPOINT ["/usr/local/bin/wafl"]
CMD ["--help"]
