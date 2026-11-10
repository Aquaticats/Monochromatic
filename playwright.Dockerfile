# Named playwright.Dockerfile (not Dockerfile.playwright) because nvim-web-devicons
# matches icons by file extension: "Dockerfile" is a recognized extension.
#
# Playwright test runner for immutable OSes (Bazzite, Fedora Atomic, etc.)
# that lack apt-get for installing browser dependencies.
#
# Only browsers and system libraries live in this container.
# @playwright/test stays in the host catalog for type checking and IDE support
# (test files import { test, expect } from it). Do not remove it from package.json.
#
# The version tag MUST match the @playwright/test resolution in pnpm-lock.yaml.
# Find available tags: https://mcr.microsoft.com/v2/playwright/tags/list
FROM mcr.microsoft.com/playwright:v1.65.0-noble@sha256:6e3fb709361521eb27161606e6da6d2674e5e89fa16516cb37d6fdd91372273d

RUN apt-get update && apt-get install -y unzip && rm -rf /var/lib/apt/lists/*

# bun comes from the release archive `toolchain.lock.json` records for it (`.tools.bun`), fetched and checked
# against the SHA-256 in that row, the way package/mutation-test/cli/runtime/Containerfile installs node and pnpm.
# The version and the two digests restate the lock row because this file has no COPY step to read the lock from,
# and root-build-file-toolchain.unit.test.ts fails when they stop matching it. The installer script bun.sh serves is
# not used: this build must not run whatever that endpoint answers at build time.
ARG BUN_VERSION=1.4.4
ARG BUN_SHA256_LINUX_X64=bf8ccbfb2b0b80159e0f9a6d15c55317add8077534dbc6130a3edb52fccb395c
ARG BUN_SHA256_LINUX_ARM64=baa6bb36f51d6e58561e02bbea825038a9df454f673d6d776e8bbf86f8c7850e
ENV BUN_INSTALL="/usr/local"
RUN set -eux; \
    case "$(uname --machine)" in \
      x86_64) archive_platform=linux-x64; archive_sha256="${BUN_SHA256_LINUX_X64}" ;; \
      aarch64) archive_platform=linux-arm64; archive_sha256="${BUN_SHA256_LINUX_ARM64}" ;; \
      *) echo "no bun archive for $(uname --machine)" >&2; exit 1 ;; \
    esac; \
    curl --fail --silent --show-error --location --output /tmp/bun.zip \
      "https://github.com/oven-sh/bun/releases/download/bun-v${BUN_VERSION}/bun-${archive_platform}.zip"; \
    printf '%s  %s\n' "${archive_sha256}" /tmp/bun.zip | sha256sum --check --strict; \
    mkdir --parents /tmp/bun; \
    unzip -q /tmp/bun.zip -d /tmp/bun; \
    find /tmp/bun -type f -name bun -exec install --mode 0755 {} /usr/local/bin/bun \; ; \
    rm --recursive --force /tmp/bun /tmp/bun.zip; \
    bun --version

WORKDIR /work
