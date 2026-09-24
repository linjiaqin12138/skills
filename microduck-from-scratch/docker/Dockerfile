FROM docker.m.daocloud.io/library/rust:1-bookworm

ARG UID=1000
ARG GID=1000

RUN groupadd -o -g ${GID} dev \
    && useradd -o -m -u ${UID} -g ${GID} -s /bin/bash dev \
    && mkdir -p /cargo \
    && chown -R dev:dev /cargo \
    && apt-get update \
    && apt-get install -y --no-install-recommends python3-onnx \
    && rm -rf /var/lib/apt/lists/*

# 镜像里的 rustc/cargo 在 /usr/local/cargo/bin；依赖缓存单独放到 /cargo
ENV CARGO_HOME=/cargo
ENV PATH=/usr/local/cargo/bin:/cargo/bin:${PATH}

USER dev
WORKDIR /work
CMD ["sleep", "infinity"]
