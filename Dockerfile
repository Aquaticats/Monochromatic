FROM ubuntu:latest@sha256:f3d28607ddd78734bb7f71f117f3c6706c666b8b76cbff7c9ff6e5718d46ff64
RUN apt-get update \
 && apt-get install -y --no-install-recommends ca-certificates apt-utils\
 && rm -rf /var/lib/apt/lists/*
RUN sed -i 's|http://|https://|g' /etc/apt/sources.list.d/ubuntu.sources
RUN apt-get update \
 && apt-get install -y --no-install-recommends podman uidmap git unzip curl bash build-essential jq tar zstd python3
RUN curl https://mise.run/bash | sh
ENV PATH="/root/.local/bin:$PATH"
# The plan's tools need two more programs from the image: `tar --zstd` unpacks the `.conda` package of bubblewrap
# (zstd), and `uv tool install` builds the Python tools (slopo) with the image's own Python (python3), because the
# newest Python uv would download is one that some of their dependencies (fastuuid, through PyO3) do not support yet.
ENV UV_PYTHON_PREFERENCE=only-system
# vfs driver: avoids overlay mount issues inside Docker volumes.
# Performance is acceptable: the canary pulls one small image infrequently.
RUN printf '[storage]\ndriver = "vfs"\nrunroot = "/run/containers/storage"\ngraphroot = "/var/lib/containers/storage"\n' \
      > /etc/containers/storage.conf
# `mise run prepare` builds the Playwright image with podman inside this build (`prepare:playwright`), so podman needs
# what it needs to run inside a container: `newuidmap` (the uidmap package), a subordinate ID range that lies inside the
# build container's own ID range (root:100000:65536 lies outside the 65537 IDs a rootless `podman build` gives it, and
# newuidmap refuses it), and the inner settings the podman-in-podman image uses (host namespaces; a bind of /proc instead
# of mounting a new one, which a container without CAP_SYS_ADMIN cannot do). Measured with `podman build`, 2026-10-31.
RUN echo 'root:1:65535' > /etc/subuid \
 && echo 'root:1:65535' > /etc/subgid
RUN printf '[containers]\nnetns = "host"\nuserns = "host"\nipcns = "host"\nutsns = "host"\ncgroupns = "host"\ncgroups = "disabled"\nlog_driver = "k8s-file"\nvolumes = ["/proc:/proc"]\ndefault_sysctls = []\n[engine]\ncgroup_manager = "cgroupfs"\nevents_logger = "file"\n' \
      > /etc/containers/containers.conf

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

# The plan's `rust` entry runs `rustup toolchain install`, and every cargo-binstall tool runs `cargo binstall`, a cargo
# subcommand, so a machine needs rustup before `mpm-shadow install` (the plan does not provision rustup itself; the
# retired TypeScript sync needed it the same way). rustup-init is checked against the checksum rust-lang publishes
# beside it, and installs no toolchain (`rustup toolchain install` of the plan does) but the `cargo` and `rustc`
# proxies that need one. The `rust` entry's RUSTUP_TOOLCHAIN then selects the locked nightly for them.
RUN rustup_url=https://static.rust-lang.org/rustup/dist/x86_64-unknown-linux-gnu \
 && curl --fail --silent --show-error --location --output /tmp/rustup-init "${rustup_url}/rustup-init" \
 && curl --fail --silent --show-error --location --output /tmp/rustup-init.sha256 "${rustup_url}/rustup-init.sha256" \
 && echo "$(cut --delimiter=' ' --fields=1 /tmp/rustup-init.sha256)  /tmp/rustup-init" | sha256sum --check \
 && chmod +x /tmp/rustup-init \
 && /tmp/rustup-init -y --no-modify-path --profile minimal --default-toolchain none \
 && rm /tmp/rustup-init /tmp/rustup-init.sha256
ENV PATH="/root/.cargo/bin:${PATH}"

# Every tool converges to `toolchain.lock.json`; mise runs only the repository's tasks afterwards
# (`mise run toolchain:install` is the same entry).
RUN export $(cat .env.local | xargs) && mpm-shadow install --frozen-lockfile

RUN mise run prepare
