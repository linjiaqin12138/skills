FROM docker.m.daocloud.io/library/rust:1-bookworm

ARG UID=1000
ARG GID=1000

RUN groupadd -o -g ${GID} dev \
    && useradd -o -m -u ${UID} -g ${GID} -s /bin/bash dev \
    && mkdir -p /cargo \
    && chown -R dev:dev /cargo \
    && apt-get update \
    && apt-get install -y --no-install-recommends python3-onnx python3-pip \
        libosmesa6 libgl1 \
        xvfb x11vnc novnc websockify \
    && rm -rf /var/lib/apt/lists/*

# M4 仿真体（sim/duck_body.py）与网页画面（MJPEG viewer，osmesa 离屏渲染；
# --viewer 模式走 Xvfb + noVNC 浏览器远程桌面）
RUN python3 -m pip install --break-system-packages --no-cache-dir mujoco pillow

# 镜像里的 rustc/cargo 在 /usr/local/cargo/bin；依赖缓存单独放到 /cargo
ENV CARGO_HOME=/cargo
ENV PATH=/usr/local/cargo/bin:/cargo/bin:${PATH}

USER dev
WORKDIR /work
CMD ["sleep", "infinity"]
