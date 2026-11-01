# Rust 1.101 nightly requires full metadata beside a standalone library link archive

## Symptom

The frozen source-linter campaign uses Rust `1.101.0-nightly`,
commit `5d8fec0669a64607db9ccbf663e7c23125bf4748`.
Its actual library builds and 415 Nextest cases pass,
but a separate direct `rustc` consumer given only the produced `.rlib` fails.
The diagnostic contains these exact clauses:

```text
// existing-processor-consumer-build.log
only metadata stub found for `rlib` dependency `monochromatic_lint`
please provide path to the corresponding .rmeta file with full metadata
```

The consumer does not execute after that compilation failure.
This is distinct from the later research harness's `ENOENT` directory lookup:
that lookup assumed `debug/deps` existed and stopped before invoking `rustc`.

## Root cause

The exact compiler source was inspected in a separate read-only shallow checkout.
In `compiler/rustc_metadata/src/locator.rs:676`,
the loader rejects stub archive metadata when no full-metadata slot has already been resolved:

```rust
//! `rustc_metadata/src/locator.rs:676`
//! Compiler revision `5d8fec0669a64607db9ccbf663e7c23125bf4748`.
if metadata.get_header().is_stub {
    assert_ne!(flavor, CrateFlavor::Rmeta);
    if slot.is_none() {
        return Err(CrateError::FullMetadataNotFound(self.crate_name, flavor));
    }
} else {
    *slot = Some((hash, metadata, lib.clone(), flavor));
}
```

The adjacent compiler comment states that `.rmeta` files resolve before `.rlib` and dynamic-library files.
The missing-metadata diagnostic is declared by `FullMetadataNotFound`
in `compiler/rustc_metadata/src/diagnostics.rs:326`.
The failure does not establish an application-source defect or a provider-specific compiler failure.

The compiler's own `tests/run-make/embed-metadata/rmake.rs:54` tests the intended explicit pairing:

```rust
//! `tests/run-make/embed-metadata/rmake.rs:60`
//! Same compiler revision.
let mut rustc = rustc();
kind.add_extern(&mut rustc, "dep1", "deps");
rustc.extern_("dep1", path("deps").join("libdep1.rmeta"));
rustc.input("foo.rs").run();
```

The neighboring `lookup_rmeta_missing` control at line 68 omits the second `--extern`
and requires the diagnostic substring `only metadata stub found`.
The primary evidence therefore supports a caller-side artifact-discovery correction.
It does not support changing the library source or attributing the result to an incorrect output flag.

## Verification

The [Rust campaign ledger](../audit/depot-rust-coverage-2026-10-18.evidence.json)
binds source,
tool hashes,
raw command logs and JUnit.

- `j50kf2jlqx` builds the real linter,
  passes 415 tests with 1 skipped out of 416 enumerated,
  and rejects the archive-only direct consumer with the stated diagnostic.
- `jf0c9llb1v` repeats the same application test counts,
  retains the tested CLI executable,
  and stops at the harness's fixed `debug/deps` lookup before attempting the revised consumer.
- `zxlt43t7g1` discovers the library and dependency filenames
  from Cargo's `compiler-artifact` JSON,
  admits co-produced archive and metadata hashes,
  compiles the unchanged `src/processors_consumer.rs`,
  and executes it successfully with `processor artifact consumer passed`.
  Its harness is `ee873be6f753c611b15cbfd0e925d0048a631dbb`.
  Artifact `01a14e95-7e1f-70e2-a82d-b8ea65ea6fd7` passed the provider upload digest and ZIP CRC checks.

The passing catalog currently includes application compilation,
the full Nextest boundary and actual CLI help/rule enumeration.
The failing catalog includes the archive-only direct invocation
and the separate fixed-directory harness assumption.
The recovered ordinary consumer's command logs and artifact hashes are retained in that ledger.

## Verified workaround

Preserve the link archive and full metadata from the same Cargo build.
Pass both paths explicitly to direct `rustc`,
using dependency search directories obtained from that build's compiler-artifact messages.
The admitted command shape is:

```text
// Research harness substitutes the Cargo-reported absolute paths.
rustc --edition=2024 --extern monochromatic_lint=LIBRARY.rlib
  --extern monochromatic_lint=LIBRARY.rmeta -L dependency=DEPENDENCY_DIRECTORY
  src/processors_consumer.rs --out-dir OWNED_OUTPUT_DIRECTORY
```

This is an argument-vector illustration,
not a multiline shell script.
The tradeoff is that standalone transport must retain matching compiler metadata and dependency artifacts;
a CLI executable archive has a different consumer boundary.
No upstream source edit is required by the inspected compiler control.

## What does not work

- Passing only the archive reproduces the compiler's missing-full-metadata boundary.
- Finding files by a presumed `debug/deps` location caused the separate harness failure.
  Cargo's actual artifact messages must decide the paths.
- Replacing actual execution evidence with a manifest,
  a successful library build or the application's passing unit tests does not verify the public consumer.

## Upstream filing decision

The repository's `.out-of-scope/cargo-workspace.md` concerns workspace architecture,
not this compiler invocation.
No relevant filing exemption was identified there.
Upstream communication is outside this research campaign's authorized scope.

1.  Upstream fault is not established:
    the compiler intentionally tests both the successful pairing and missing-metadata rejection.
2.  A caller correction is possible;
    no architectural impossibility or upstream source patch has been established.
3.  The paired-artifact use case is supported by the exact compiler run-make test.
4.  No contribution is proposed,
    so contribution-policy acceptance is not claimed or used to justify filing.
5.  The existing [metadata-embedding tracking issue](https://github.com/rust-lang/rust/issues/139165)
    covers the unstable option and links its implementation and Cargo integration.
    Its instructions exclude using the tracking thread for bug reports or discussion.
6.  The consumer-side pairing prototype passed its actual execution in `zxlt43t7g1`;
    no upstream source patch was necessary.

There is nothing additive to file about the intentional archive-only rejection.
No new issue or comment was posted,
and no upstream fix is claimed.
