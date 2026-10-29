# Mutation-testing image for a deepmerge-ts checkout (the `mutation:*` mise tasks).
# StrykerJS runs upstream's own Vitest suite here, never in this repo (user decision in
# doc/handover/deepmerge-ts-hardening.md); the checkout is mounted read-only at run time
# and copied into /upstream by src/mutation-score.ts.
FROM docker.io/library/node:26-slim

# Stryker's child-process cleanup calls `ps`.
RUN apt-get update && apt-get install --yes --no-install-recommends procps && rm -rf /var/lib/apt/lists/*

WORKDIR /upstream

# Pinned so mutant ids and scores stay comparable with
# doc/audit/deepmerge-ts-mutation-2026-09-24.md, which used exactly these versions.
RUN printf '{"name":"deepmerge-ts-mutation","private":true,"type":"module"}\n' > package.json \
 && npm install --no-audit --no-fund --save-dev \
    @stryker-mutator/core@10.0.0 @stryker-mutator/vitest-runner@10.0.0 \
    vitest@4.1.10 vite@8.0.10 vite-tsconfig-paths@6.1.1 \
    typescript@6.0.2 esbuild@0.28.2
