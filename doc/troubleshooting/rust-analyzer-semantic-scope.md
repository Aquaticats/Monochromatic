# rust-analyzer 0.0.336 direct semantic queries require an attached database

## Symptom

A disposable consumer compiled,
loaded its Cargo fixture,
then panicked on semantic queries:

```text
# ra_ap_hir_ty 0.0.336
Try to use attached db, but not db is attached
```

This was a missing caller-side scope,
not a failure to distinguish generic methods.

A separate fixture later called `AbsPath::exists()`.
rustc warned that the method is deprecated,
then the test panicked with `not implemented`.
The supported filesystem boundary is `std::fs::metadata`.

## Root cause

The installed `ra_ap_hir_ty` 0.0.336 source and the read-only checkout at
`~/temp/agent/rust-analyzer-0336-20261004`
share revision `7ea2b259ca3fa0e97d0e8e2ec4c3f902f049cd76` for the inspected source.
Both copies of `crates/hir-ty/src/next_solver/interner.rs`
have SHA-256
`69c59ab2eac404af92454e534ed772ab4fef24612530c5b8b18131744745bd4d`.
This comparison covers that file,
not every publication edit.

The accessor at `crates/hir-ty/src/next_solver/interner.rs:2439-2443`
requires an existing attachment:

```rust
// crates/hir-ty/src/next_solver/interner.rs:2439-2443
fn with<R>(&self, op: impl FnOnce(&dyn HirDatabase) -> R) -> R {
    let db = self.database.get().expect("Try to use attached db, but not db is attached");
    op(unsafe { db.as_ref() })
}
```

The public scope at lines 2452-2453 installs that attachment around its callback:

```rust
// crates/hir-ty/src/next_solver/interner.rs:2452-2453
pub fn attach_db<R>(db: &dyn HirDatabase, op: impl FnOnce() -> R) -> R {
    GLOBAL_DB.with(|global_db| global_db.attach(db, op))
}
```

The separate filesystem failure is explicit in `crates/paths/src/lib.rs:301-305`:

```rust
// crates/paths/src/lib.rs:301-305, relevant declaration
#[deprecated(note = "use std::fs::metadata().is_ok() instead")]
pub fn exists(&self) -> ! {
    unimplemented!()
}
```

The caller incorrectly assumed an ordinary Path-like filesystem method.

## Verification

The HIR type crate's crates.io checksum is
`59b7f949a3e5409372d019ca4d828cffa7e5a22f785119736263d57aa31b2ea4`.
Cargo recorded it in `package/linter/monochromatic-lint/Cargo.lock`.

Private harness:
`~/temp/agent/monochromatic-semantic-probe-20261004`.
Its `EXECUTION.md` records the inspected build-script paths,
compiler image,
and isolation boundaries.
No upstream source was edited.

Working controls:

- Named functions through `attach_db` and `with_attached_db` returned method arities
  `[1, 0, 1]` for identical `parse` names on generic/nongeneric receivers
  (`proc_46ae`).
- A typed Salsa input selected the file without a captured callback or parallel request-global store
  (`proc_39fb`).
- The standard-library user example resolved `parse`,
  `map`,
  `collect`,
  `iter`,
  and `clone` correctly
  (`proc_e5d4`).
- The actual linter rule accepted that example as a Rust library consumer
  (`proc_7d2f`).

Failing controls:

- Omitting attachment compiled but panicked
  (`proc_8ffa`).
- Calling deprecated `AbsPath::exists()` prevented the semantic conformance catalog from starting
  (`proc_ae2d`);
  the other 93 tests passed.
  That result is not semantic-conformance evidence.

The current probe can be run through its owning task:

```bash
# ~/temp/agent/monochromatic-semantic-probe-20261004
mise run probe:std
```

The package-level conformance path is:

```bash
# Repository root
mise run //package/linter/monochromatic-lint:lint:container
```

## Verified workaround and remaining verification

Use the backend's attachment API and its typed per-database request storage.
The tradeoff is an explicit semantic query scope;
it does not require anonymous application callbacks,
a second mutable request-global store,
or an upstream patch.

Use `std::fs::metadata(&path)` instead of the intentionally disabled path method.
That fixture correction is under package verification;
do not describe its rerun as passed until the process result is recorded.

## What does not work

- Successful compilation does not establish that the database scope exists at runtime.
- A separately parsed syntax tree is not automatically registered in the semantic database.
- The path wrapper's resemblance to `std::path::Path` does not make every inherited-looking method usable.
- Assuming semantic crate version 0.0.335 exists because syntax 0.0.335 exists:
  the freshly fetched HIR sparse index contained 292 versions but no 0.0.335 entry.
  Both 0.0.334 and 0.0.336 were present and not yanked.
  Direct crates.io HTTP returned 403,
  and the 0.0.335 docs.rs page returned 404;
  the sparse index and Cargo resolver provided the decisive publication evidence.

## Upstream filing decision

No matching rust-analyzer exemption was present in the inspected `.out-of-scope/` filenames.
No issue or comment is proposed,
so no external mutation was performed.

1.  Upstream fault: no.
    The consumer omitted a required scope and called an explicitly deprecated,
    disabled filesystem method.
2.  Fixability: the supported APIs already provide the needed behavior.
3.  Supported use: the loader explicitly documents external library consumers;
    its source and the semantic APIs were inspected.
4.  Contribution policy: not evaluated for filing because there is no upstream defect claim.
5.  Expected upstream action: none is requested.
6.  Upstream prototype: not applicable;
    the named-callback consumer correction was run,
    and the filesystem-call correction is being reverified locally.

Upstream filing artifact:
nothing to add.
The durable action is correcting the consumer,
not filing a report about its misuse.
