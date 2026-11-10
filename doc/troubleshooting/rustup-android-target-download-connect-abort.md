# rustup 1.29.0 Android target installation can stop at a Reqwest connect abort while curl reaches the artifact

## Symptom

The Android native build failed before compiling project code:

```text
error[E0463]: can't find crate for `core`
= note: the `aarch64-linux-android` target may not be installed
```

`mise exec rust -- rustup target list --installed` listed only
`x86_64-unknown-linux-gnu`.
The repository's recovery task then failed while downloading each Android standard library:

```text
error: component download failed for rust-std-aarch64-linux-android:
partially downloaded file was kept for resumption, please try again:
client error (Connect): tcp connect error: Software caused connection abort (os error 103)
```

The same error affected `rust-std-x86_64-linux-android` after the ARM artifact was cached.

## Root cause

The immediate build failure was a missing cross-target standard library.
Rustup's user guide at
`doc/user-guide/src/cross-compilation.md:27-52` says non-host targets must be added to the active toolchain
and that `rustup target add` installs their Rust standard library.
The Android NDK alone does not supply Rust's `core` or `std` crates.

The download failure occurred before any HTTP response reached rustup.
The installed rustup 1.29.0 reported a Reqwest TCP connect error,
while `curl --head` reached the same URL with HTTP 200.
The available evidence therefore isolates this incident to rustup's client connection path on this host,
not an unavailable distribution artifact.
It does not identify why the operating system aborted Reqwest's connection.

Rustup intentionally keeps resumable files after a client-side network failure.
Upstream commit `b1d4c1e5dc63ad98401162f5a26615a39742ed44` shows:

- `src/download/mod.rs:125-149` enables resume and retains the cached file for a resumable network failure;
- `src/download/mod.rs:290-291` sends an HTTP `Range` header when a partial file has content;
- `src/dist/download.rs:61-69` reuses a completed cache entry only after its SHA-256 matches the expected hash;
- `src/dist/download.rs:73-95` names the resumable file `<hash>.partial` and attaches
  `IncompletePartialFile` to a repeated network failure;
- `src/errors.rs:37-40` emits
  `partially downloaded file was kept for resumption, please try again`.

The first failed attempts left zero-byte partial files.
Downloading through curl produced complete artifacts,
but rustup still treated the `.partial` name as a resume candidate.
After independent hash verification and promotion to rustup's completed cache name,
rustup reused each artifact and installed both targets.

## Verification

Versions and source under test:

- rustup `1.29.0 (28d1352db 2026-03-05)`;
- rustc `1.100.0-nightly (c656540d6 2026-08-21)`;
- rustup source commit `b1d4c1e5dc63ad98401162f5a26615a39742ed44`;
- Fedora Linux host;
- ARM artifact size `30,424,296` bytes;
- x86_64 artifact size `30,285,512` bytes.

Failing command:

```sh
mise run prepare:android
```

Patterns that worked:

- `curl --head --location <artifact-url>` returned HTTP 200 with `accept-ranges: bytes`;
- `curl --location --continue-at - --output <hash>.partial <artifact-url>` completed;
- `sha256sum <hash>.partial` exactly matched the expected cache key;
- rerunning `mise run prepare:android` after cache promotion installed both targets.

Patterns that failed:

- the initial `rustup target add` invocation through `prepare:android`;
- an unchanged retry,
   which failed before receiving bytes;
- leaving a complete artifact named `.partial`,
   because rustup entered its resume path again.

Final proof:

```text
aarch64-linux-android
x86_64-linux-android
x86_64-unknown-linux-gnu
```

## Verified workaround

For each URL and expected SHA-256 printed by rustup:

1.  Download to rustup's reported `.partial` path with curl.
2.  Run `sha256sum` on the complete file.
3.  Continue only when the digest exactly equals the expected cache filename.
4.  Rename `<hash>.partial` to `<hash>` in `~/.rustup/downloads/`.
5.  Rerun `mise run prepare:android`.

Tradeoff:
this operates on rustup's internal cache convention and is specific to one dated nightly artifact.
Never promote a file without exact digest verification.
A later rustup release may change cache layout.

## What does not work

- Repeating `prepare:android` without changing the client path reproduced OS error 103.
- Installing only the NDK cannot fix Rust error E0463;
  the target-specific Rust standard library is separate.
- Treating the `.partial` suffix as a completed cache entry does not work.
  Rustup sends a range request from the existing length instead of taking the completed-cache branch.

## Upstream filing decision

No `.out-of-scope/` entry covers rustup downloads.
Searches of open and closed rustup issues and pull requests for the error,
partial-resume behavior,
and `static.rust-lang.org` download failures found no matching report.

1.  **Upstream fault:**
     not established.
     The artifact was available and curl succeeded,
    but the evidence does not distinguish Reqwest,
     host networking,
     or transient connection state.
2.  **Upstream can fix it:**
     unknown without a reproducible client-side cause.
3.  **Supported use case:**
     yes.
     Rustup documents Android target installation.
4.  **Contribution policy:**
     rustup's developer guide requires issue and pull-request communication in the
    contributor's own words and says AI should not generate maintainer comments.
5.  **Likely fix:**
     unknown because the cause below Reqwest's connect error is not isolated.
6.  **Compatible prototype:**
     not applicable without an upstream defect boundary.

There is nothing responsible to file upstream from this incident.
No issue or comment draft is retained.
