FROM debian:trixie-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates libgcc-s1 \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --uid 10001 --create-home wafl \
    && mkdir -p /var/lib/wafl/cache /var/lib/wafl/endpoints \
    && chown -R 10001:10001 /var/lib/wafl
COPY wafl wafl-kube /usr/local/bin/
USER 10001:10001
ENTRYPOINT ["/usr/local/bin/wafl"]
