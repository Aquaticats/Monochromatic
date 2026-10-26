FROM ubuntu:latest@sha256:f3d28607ddd78734bb7f71f117f3c6706c666b8b76cbff7c9ff6e5718d46ff64
RUN apt-get update \
 && apt-get install -y --no-install-recommends ca-certificates apt-utils\
 && rm -rf /var/lib/apt/lists/*
RUN sed -i 's|http://|https://|g' /etc/apt/sources.list.d/ubuntu.sources
RUN apt-get update \
 && apt-get install -y --no-install-recommends podman git unzip curl bash build-essential
RUN curl https://mise.run/bash | sh
ENV PATH="/root/.local/bin:$PATH"
# vfs driver: avoids overlay mount issues inside Docker volumes.
# Performance is acceptable: the canary pulls one small image infrequently.
RUN printf '[storage]\ndriver = "vfs"\nrunroot = "/run/containers/storage"\ngraphroot = "/var/lib/containers/storage"\n' \
      > /etc/containers/storage.conf
RUN echo 'root:100000:65536' > /etc/subuid \
 && echo 'root:100000:65536' > /etc/subgid

WORKDIR /app
COPY . .

RUN ls -la

RUN mise trust --all

# Tool versions resolve from mise.toml/mise.lock through `mise install` until the
# cutover. `toolchain.lock.json` (repository root) is the source of truth for every
# tool version (doc/decision/mpm-toolchain-adoption.md); `mise run toolchain:sync`
# replaces this install step and reads that lock.
RUN export $(cat .env.local | xargs) && mise install

RUN mise run prepare
