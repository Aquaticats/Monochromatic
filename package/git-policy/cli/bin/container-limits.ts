/**
 Memory, CPU and process limits of every container the verification runners start, with the Podman arguments they become.

 The implementation and its tests live in the shared cargo-mutants package, because the mutation campaigns of every Rust
 product read the same `--memory`, `--cpus` and `--pids-limit` options; this module keeps the runners' import path.
 */
export * from '@monochromatic-dev/mutation-test-cli-cargo-mutants/ts/container-limits.ts';
