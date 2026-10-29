# Release forbidden-strings for Windows arm64

## Status

Accepted,
2026-09-16.

## Context

`.github/workflows/cargo-publish.yml` has published prebuilt `forbidden-strings` archives
since commit `ef6e3ac25` (2026-05-17).
Its `build-binary` matrix covered Linux gnu and musl on x86_64 and aarch64,
macOS on x86_64 and aarch64,
and Windows only on x86_64 (`x86_64-pc-windows-msvc`).
Releases through `forbidden-strings-v0.4.0` therefore shipped no Windows arm64 binary.

That commit recorded no reason for leaving out Windows arm64.
The actual reason was a mistaken belief held by the repository owner:
GitHub has no hosted Windows arm64 runner.
Nobody checked that belief against GitHub's documentation when the matrix was written.

GitHub's documentation contradicted that belief before `forbidden-strings` existed:

- [Windows arm64 hosted runners now available in public preview][preview] (2025-04-14)
  introduced the `windows-11-arm` label,
  free for public repositories.
- [arm64 hosted runners for public repositories are now generally available][ga] (2025-08-07)
  made `windows-11-arm` generally available alongside `ubuntu-24.04-arm` and `ubuntu-22.04-arm`.
  Commit `ef6e3ac25` cites this same August 2025 availability for its Linux arm runners.
- The first `forbidden-strings` crate commit is `e15f79198` (2026-05-02),
  about nine months after general availability.

## Decision

Add `aarch64-pc-windows-msvc` to the `build-binary` matrix on the native `windows-11-arm` runner.
Ship it immediately with patch releases:
`forbidden-regex` 0.1.1 first,
then `forbidden-strings` 0.4.1 requiring `forbidden-regex` 0.1.1.
GitHub releases are immutable,
so `forbidden-strings-v0.4.0` cannot receive the new archive.

Evidence checked on 2026-09-16 before the change:

- The repository is public (`gh repo view --json visibility`).
- The [`windows-11-arm` image][image] ships Rust 1.98.1,
  rustup,
  Bash,
  and 7-Zip,
  which covers every tool the existing Windows packaging step uses.
- `package/forbidden-strings/cli/Cargo.lock` contains no C-compiling build dependency
  (`cc`,
  `ring`,
  `openssl-sys`,
  and similar);
  its only Windows binding crate is `windows-sys` 0.61.2.
- The build itself was not reproduced locally;
  the 0.4.1 release run was the first real build for this target.

The `forbidden-regex` bump exists because `cargo package` resolves the path dependency from crates.io.
Published `forbidden-regex` 0.1.0 (checksum `ef4a63a5`) predates the multithreaded rule compile,
so a `forbidden-strings` crate built against it would differ from the release binaries.
The source diff against the published crate adds only private items,
so a patch version is semver-compatible.

## Outcome

Verified 2026-09-17:

- Commit `43404709a` published `forbidden-regex` 0.1.1 to crates.io
  (sparse index checksum `2df22dea`)
  after `mise run //package/forbidden-strings/core:test` passed.
- Commit `207bed93f` published `forbidden-strings` 0.4.1 to crates.io
  (sparse index checksum `724eb91c`,
  `forbidden-regex` requirement `^0.1.1` for normal and build dependencies)
  after `mise run //package/forbidden-strings/cli:test` passed.
- Workflow run 35178616343 succeeded on every job,
  including `build-binary (aarch64-pc-windows-msvc, windows-11-arm)`.
- Release `forbidden-strings-v0.4.1` carries eight archives,
  including `forbidden-strings-0.4.1-aarch64-pc-windows-msvc.zip`.
- `gh attestation verify` binds that archive to `cargo-publish.yml` at `207bed93f`,
  and `file` reports its `forbidden-strings.exe` as a PE32+ ARM64 console executable.
- The binary has not been executed on Windows arm64 hardware.

## Alternatives considered

Cross-compiling from the x86_64 `windows-latest` runner was rejected.
It depends on that image carrying the arm64 MSVC libraries,
which was not verified,
and it diverges from the native-runner pattern every other matrix entry follows.

Waiting for the next substantive version bump was rejected by the repository owner
in favor of shipping the Windows arm64 binary now.

[preview]: https://github.blog/changelog/2025-04-14-windows-arm64-hosted-runners-now-available-in-public-preview/
[ga]: https://github.blog/changelog/2025-08-07-arm64-hosted-runners-for-public-repositories-are-now-generally-available/
[image]: https://github.com/actions/runner-images/blob/main/images/windows/Windows11-Arm64-Readme.md
