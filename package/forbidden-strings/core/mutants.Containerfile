# What:  baked image that runs mutation testing for forbidden-regex.
# Why:   cargo-mutants compiles and runs MUTATED source, i.e. arbitrary and
#        unpredictable code, which must never execute on the host. This image is the
#        disposable sandbox; the `test:mutation` mise task runs cargo-mutants only here.
# The official image only supplies rustup; the toolchain is the nightly toolchain.lock.json pins
# (`.tools.rust.version`), because the repository builds everything with that nightly. The build
# context is not the repository root, so this default is kept in sync with that key, and
# `--build-arg RUST_TOOLCHAIN=` overrides it.
FROM docker.io/library/rust:1
ARG RUST_TOOLCHAIN=nightly-2026-10-29
RUN rustup toolchain install "${RUST_TOOLCHAIN}" --profile minimal \
    && rustup default "${RUST_TOOLCHAIN}"

# Bake cargo-mutants so a run is one `podman run` with no in-container install step.
# `--locked` pins its own dependency versions for a reproducible tool build.
RUN cargo install cargo-mutants --locked
# The campaign runs the baseline and every mutant through nextest, as the `test` task does.
RUN cargo install cargo-nextest --locked
