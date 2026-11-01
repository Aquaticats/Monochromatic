# IDE build script cannot find the embedded runtime under nightly Cargo

## Symptom

The release workflow `ide-release.yml` failed in both build legs, at the step that runs `task-run //package/ide/app:bundle`.
The tasks that prepare the language runtime had succeeded (27 grammars fetched and built),
and the failing command was `cargo build --release --bin monochromatic-ide` in the build image:

```text
thread 'main' panicked at build.rs:207:23:
The application binary embeds its language runtime, but /work/target/release/build/runtime/manifest.json
cannot be read (No such file or directory (os error 2)). Prepare it with `mise run //package/ide/app:runtime` ...
```

The runtime directory the task prepared was `target/release/runtime`, one level up from where the script looked.

## Cause

`generate_embedded_runtime` in `package/ide/app/build.rs` located `target/<profile>` with `out_directory.ancestors().nth(3)`,
which is correct only for the older output layout `target/<profile>/build/ide-<hash>/out`.
The image toolchain moved from the floating stable channel to the pinned nightly (Cargo 1.101.0-nightly, commit `8652ba7d7f`).
Measured in `localhost/monochromatic/ide` with a probe crate whose build script printed `OUT_DIR`:

```text
/work/target/release/build/outdir-probe/c9206825fbebe4cc/out
```

The package name and the hash are now separate directories, so three levels up is `target/release/build`, not `target/release`.
The package move to `package/ide/app` did not change any build logic (a rename-aware diff shows only message text and task addresses).

## Fix

The script now takes the nearest ancestor of `OUT_DIR` named `build` and uses its parent.
Checked with the same expression in the nightly image against four paths:

- the real nightly `OUT_DIR` gave `target/release`;
- the older layout `target/release/build/ide-0123456789abcdef/out` gave `target/release`;
- a path with a `build` directory above the target (`/home/runner/build/work/target/debug/build/ide/0123/out`) gave `target/debug`;
- a path with no `build` ancestor gave nothing, which reaches the panic that names the path.

The old stable layout was not produced by a live Cargo in this check; that case rests on the synthetic path above
and on the release `ide-v0.1.3`, which built with the stable toolchain and the old `nth(3)` rule.

## Related

- `doc/troubleshooting/cargo-1-98-inherited-default-features.md` covers the other Cargo change that blocked IDE 0.1.3.
- `doc/handover/slint-ide-0x.md` records the release dry runs that found this.
