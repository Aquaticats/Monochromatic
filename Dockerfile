FROM ubuntu:latest@sha256:f3d28607ddd78734bb7f71f117f3c6706c666b8b76cbff7c9ff6e5718d46ff64
RUN apt-get update \
 && apt-get install -y --no-install-recommends ca-certificates apt-utils\
 && rm -rf /var/lib/apt/lists/*
RUN sed -i 's|http://|https://|g' /etc/apt/sources.list.d/ubuntu.sources
RUN apt-get update \
 && apt-get install -y --no-install-recommends podman git unzip curl bash build-essential jq tar
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

# Node and pnpm come from the archived artifacts `toolchain.lock.json` pins (repository root: the source of truth
# for every tool version, doc/decision/mpm-toolchain-adoption.md). `jq` reads the `linux-x64` rows, each archive is
# fetched and its SHA-256 verified against the lock before it is linked. Every version here comes from that lock.
RUN node_url="$(jq --raw-output '.tools.node.artifacts["linux-x64"].url' toolchain.lock.json)" \
 && node_sum="$(jq --raw-output '.tools.node.artifacts["linux-x64"].sha256' toolchain.lock.json)" \
 && curl --fail --silent --show-error --location --output /tmp/node.tar.gz "$node_url" \
 && echo "$node_sum  /tmp/node.tar.gz" | sha256sum --check \
 && mkdir /opt/node \
 && tar --extract --file /tmp/node.tar.gz --directory /opt/node --strip-components=1 \
 && ln --symbolic /opt/node/bin/node /usr/local/bin/node \
 && ln --symbolic /opt/node/bin/npm /usr/local/bin/npm \
 && ln --symbolic /opt/node/bin/npx /usr/local/bin/npx \
 && pnpm_url="$(jq --raw-output '.tools.pnpm.artifacts["linux-x64"].url' toolchain.lock.json)" \
 && pnpm_sum="$(jq --raw-output '.tools.pnpm.artifacts["linux-x64"].sha256' toolchain.lock.json)" \
 && curl --fail --silent --show-error --location --output /tmp/pnpm.tar.gz "$pnpm_url" \
 && echo "$pnpm_sum  /tmp/pnpm.tar.gz" | sha256sum --check \
 && mkdir /opt/pnpm \
 && tar --extract --file /tmp/pnpm.tar.gz --directory /opt/pnpm \
 && ln --symbolic /opt/pnpm/pnpm /usr/local/bin/pnpm

# The workspace dependencies the tasks import (`mise run prepare` below needs `node_modules`).
RUN pnpm install --frozen-lockfile

# `mpm-shadow` is the one prebuilt binary a fresh machine needs: the archive row of the `mpm-shadow` tool in
# `toolchain.lock.json` (URL and SHA-256, the same pattern as Node and pnpm above) is fetched and checked before it is
# unpacked. It then installs every other tool, `file-enforcer` among them.
RUN ms_url="$(jq --raw-output '.tools["mpm-shadow"].artifacts["linux-x64"].url' toolchain.lock.json)" \
 && ms_sum="$(jq --raw-output '.tools["mpm-shadow"].artifacts["linux-x64"].sha256' toolchain.lock.json)" \
 && curl --fail --silent --show-error --location --output /tmp/mpm-shadow.tar.gz "$ms_url" \
 && echo "$ms_sum  /tmp/mpm-shadow.tar.gz" | sha256sum --check \
 && tar --extract --gzip --file /tmp/mpm-shadow.tar.gz --wildcards --strip-components=1 --directory /usr/local/bin '*/mpm-shadow' \
 && mpm-shadow --version

# Every tool converges to `toolchain.lock.json`; mise runs only the repository's tasks afterwards
# (`mise run toolchain:install` is the same entry).
RUN export $(cat .env.local | xargs) && mpm-shadow install --frozen-lockfile

RUN mise run prepare
