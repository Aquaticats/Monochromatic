# What:  baked image that runs mutation testing for forbidden-regex.
# Why:   cargo-mutants compiles and runs MUTATED source, i.e. arbitrary and
#        unpredictable code, which must never execute on the host. This image is the
#        disposable sandbox; the `test:mutation` mise task runs cargo-mutants only here.
# The official image only supplies rustup; the toolchain is the nightly toolchain.lock.json pins
# (`.tools.rust.version`), because the repository builds everything with that nightly. The build
# context is not the repository root, so the caller reads that key and passes it as
# `--build-arg RUST_TOOLCHAIN=` (package/test-harness/core-matrix/src/bin/locked-rust-toolchain.ts prints it); an empty value stops the build.
FROM docker.io/library/rust:1
ARG RUST_TOOLCHAIN
RUN test -n "${RUST_TOOLCHAIN}" \
    || { echo "RUST_TOOLCHAIN is empty: pass --build-arg RUST_TOOLCHAIN=<tools.rust.version of toolchain.lock.json>" >&2; exit 1; }
RUN rustup toolchain install "${RUST_TOOLCHAIN}" --profile minimal \
    && rustup default "${RUST_TOOLCHAIN}"

# Bake cargo-mutants so a run is one `podman run` with no in-container install step.
# `--locked` pins its own dependency versions for a reproducible tool build.
RUN cargo install cargo-mutants --locked
# The campaign runs the baseline and every mutant through nextest, as the `test` task does.
RUN cargo install cargo-nextest --locked
