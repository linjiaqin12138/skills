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
        curl ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# M4 仿真体（系统 python3.11 + mujoco）：--no-bam / 旧验收脚本仍可用。
RUN python3 -m pip install --break-system-packages --no-cache-dir mujoco pillow

# M8 BAM：Python 3.12 venv（better-actuator-models 要求 <3.13；系统 3.11
# 装不了/不跑 bam）。python 装到 /opt/uv-python，venv 在 /opt/sim-venv，
# 权限对所有用户可读可执行（compose exec 默认 USER=dev）。
# bam 钉 microduck_rl uv.lock 的 mjlab_frictionloss@62bd8ce（带 vin_drop_gain；
# PyPI 1.0.2 是 vin_drop_resistance，与训练蓝图不一致）。
RUN curl -LsSf https://astral.sh/uv/install.sh | sh \
    && export PATH="/root/.local/bin:${PATH}" \
    && UV_PYTHON_INSTALL_DIR=/opt/uv-python uv python install 3.12 \
    && uv venv /opt/sim-venv --python /opt/uv-python/cpython-3.12-linux-x86_64-gnu/bin/python3.12 \
    && uv pip install --python /opt/sim-venv/bin/python \
        mujoco pillow \
        "better-actuator-models @ git+https://github.com/Rhoban/bam.git@62bd8ce12154340be97e06f7f41a0ca8f116d967" \
    && chmod -R a+rX /opt/sim-venv /opt/uv-python

# 镜像里的 rustc/cargo 在 /usr/local/cargo/bin；依赖缓存单独放到 /cargo
ENV CARGO_HOME=/cargo
ENV PATH=/usr/local/cargo/bin:/cargo/bin:${PATH}
ENV SIM_PYTHON=/opt/sim-venv/bin/python

USER dev
WORKDIR /work
CMD ["sleep", "infinity"]
