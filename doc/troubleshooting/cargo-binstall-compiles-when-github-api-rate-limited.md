# cargo-binstall 1.26.1 compiles from source when an exhausted unauthenticated GitHub API quota stalls release resolution past its 15 second timeout

The case here is `monochromatic-mpm-shadow` 0.1.0,
installed through the repository's mpm extension
(`mpm --config mpm.managers.toml --cargo-binstall-monochromatic-mpm-shadow install monochromatic-mpm-shadow`),
which runs `cargo binstall --no-confirm --version 0.1.0 monochromatic-mpm-shadow`.
The release `monochromatic-mpm-shadow-v0.1.0` existed with both archives,
and the same command had installed the archive's binary minutes earlier.

## Symptom

The install finished with `Installed 1/1 packages` after 2 minutes 10 seconds,
where the earlier run took 6.8 seconds.
The installed binary had SHA-256 `285cd6c2…` while the archive's binary has `0a2bbf92…`,
so it was built from source.
While it ran,
`ps` showed `cargo install monochromatic-mpm-shadow --versi…` as a child of `cargo-binstall`,
and the process was in uninterruptible sleep (`Dl`) during the build.
Neither mpm nor binstall printed anything that names a reason.

Run directly with `--disable-strategies compile`,
the same discovery prints:

```text
cargo-binstall:  WARN resolve: Timeout reached while checking fetcher invalid url: deadline has elapsed (x4 repeated lines)
cargo-binstall: ERROR Fatal error:
  × For crate monochromatic-mpm-shadow: Fallback to cargo-install is disabled
  ╰─▶ Fallback to cargo-install is disabled
```

With a token in the environment,
the identical command succeeds in 6.3 seconds:

```text
cargo-binstall:  WARN The package monochromatic-mpm-shadow v0.1.0 (x86_64-unknown-linux-gnu) has been downloaded from github.com
```

## Root cause

All paths are relative to a shallow clone of `cargo-bins/cargo-binstall` at commit
`81df5c23b0dc775b009efdfc9895c5965b7a4e40` (release v1.26.1),
the version installed here (`cargo-binstall -V` prints `1.26.1`).

Each fetcher check runs inside a timeout,
and a timeout is not an error but a warning followed by the next fetcher
(`crates/binstalk/src/ops/resolve.rs:171-175` and `:239-244`):

```rust
for fetcher in &handles {
    match timeout(
        opts.maximum_resolution_timeout,
        AutoAbortJoinHandle::new(fetcher.clone().find()).flattened_join(),
    )
    // ...
    Err(err) => {
        warn!(
            "Timeout reached while checking fetcher {}: {}",
            fetcher.source_name(),
            err
        );
```

The timeout defaults to 15 seconds (`crates/bin/src/args.rs:241`):

```rust
default_value_t = NonZeroU16::new(15).unwrap(),
```

For a `pkg-url` that points at a GitHub release,
the fetcher asks the GitHub API first and,
on a rate limit,
sleeps for the remaining time inside the same future the timeout wraps
(`crates/binstalk-fetchers/src/gh_crate_meta.rs:119-120`):

```rust
Err(GhApiError::RateLimit { retry_after }) => {
    sleep(retry_after.unwrap_or(DEFAULT_GH_API_RETRY_DURATION)).await;
}
```

So a rate limit longer than 15 seconds surfaces as `Timeout reached while checking fetcher`,
and when no fetcher resolves,
the fallback to compiling is on unless the `compile` strategy is disabled
(`crates/binstalk/src/ops/resolve.rs:257-258`):

```rust
if !opts.cargo_install_fallback {
    return Err(BinstallError::NoFallbackToCargoInstall);
}
```

The research pass for this record also reported,
without this record re-reading the lines,
that the sleep length comes from `Retry-After` or `x-ratelimit-reset` capped at 120 seconds per wait,
that the default sleep is 10 minutes,
and that the token is read from `--github-token` or `GITHUB_TOKEN`,
then `GH_TOKEN`,
then `~/.git-credentials`,
then `gh auth token` unless `--no-discover-github-token`
(`crates/bin/src/args.rs:426-440`, `crates/binstalk-git-repo-api`, `crates/binstalk-downloader/src/remote.rs`).

### Evidence that the quota was the trigger

- `curl -I -L` to the archive URL answered HTTP 200 in 0.66 to 1.3 seconds three times in a row,
  so github.com itself was healthy.
- `https://api.github.com/rate_limit` without a token reported `limit 60, remaining 12`,
  resetting in 37 minutes,
  while the logged-in `gh` reported `limit 5000, remaining 4964`.
  Several sessions on this machine share the unauthenticated quota.
- No `GITHUB_TOKEN` was set,
  and the install scripts ran with `PATH` limited to `.cache/toolchain-bin`, `~/.cargo/bin`, `/usr/bin` and `/bin`.
  `gh` lives in `~/.local/share/monochromatic-dev/toolchain-global/exposure`,
  which that `PATH` omitted,
  so binstall's own `gh auth token` discovery could not have found a token.
- The positive control above:
  the same discovery command with `GITHUB_TOKEN="$(gh auth token)"` resolved the archive in 6.3 seconds.

Unconfirmed:
the remaining count of 12 was measured after the failures,
so the exact moment the window reached zero is not recorded,
and no run captured the rate-limit warning text itself.
The explanation rests on the source trace and the token positive control,
not on a captured warning.

### An earlier reading that was wrong

An earlier note blamed cargo's package-cache file lock for the slow install.
The `Blocking waiting for file lock on package cache` lines came from `cargo publish`,
which is cargo's own lock.
The slow install was a source build:
the child `cargo install`, the 2:10 duration and the differing binary hash show it.
`cargo-binstall` does not take cargo's package-cache lock for resolution.

## Verification

Version under test:
`cargo-binstall` 1.26.1 (`.cache/toolchain-bin/cargo-binstall`),
`mpm` 8.2.0,
crate `monochromatic-mpm-shadow` 0.1.0 and 0.1.1.

Discovery only,
no install,
in a fresh cargo home:

```bash
CARGO_HOME="${HOME}/temp/agent/binstall-diag" \
  PATH="/var/home/user/Monochromatic/.cache/toolchain-bin:${HOME}/.cargo/bin:/usr/bin:/bin" \
  cargo binstall --dry-run --no-confirm --disable-strategies compile,quick-install \
  --version 0.1.0 monochromatic-mpm-shadow
```

Catalog that works:

- Same command with `GITHUB_TOKEN="$(gh auth token)"` added:
  `has been downloaded from github.com`, 6.3 seconds.
- `mpm remove` then `mpm install` through the extension with the token set:
  `Installed 1/1 packages` in 9.7 seconds,
  installed binary SHA-256 equal to the archive's.

Catalog that fails:

- The command above with no token:
  four `Timeout reached while checking fetcher` warnings,
  then `Fallback to cargo-install is disabled`.
- The extension install without the `--disable-strategies` flag and without a token:
  a 2:10 source build and a binary with a different hash.

## Verified workarounds

1. Give binstall a token.
   Export `GITHUB_TOKEN` (for example `GITHUB_TOKEN="$(gh auth token)"`) for the install,
   or keep `gh` on `PATH` so binstall discovers its login itself.
   Tradeoff:
   the token reaches every child of the install,
   although binstall removes `GITHUB_TOKEN` and `GH_TOKEN` from the `cargo install` fallback it starts
   (`crates/binstalk/src/ops/resolve/resolution.rs:176-183`, reported, not re-read).
2. Fail instead of compiling.
   Add `--disable-strategies compile` to the install arguments.
   In this repository the mpm extension bakes a fixed argument list per crate
   (`package/file-enforcer/core/src/toolchain/mpm-manifest.ts`, `installArgs`),
   so the flag could be baked for crates that publish archives.
   Tradeoff:
   a crate with no prebuilt archive,
   or any transient failure,
   then fails to install instead of compiling,
   and the other `cargo:` tools rely on the fallback.
   Not applied.
3. Raise `BINSTALL_MAXIMUM_RESOLUTION_TIMEOUT`.
   Tradeoff:
   the install waits through the rate-limit reset,
   which the source sleeps up to 10 minutes by default,
   so it trades a silent compile for a long silent wait.

## What does not work

- Cargo's package-cache lock as the cause (see the wrong reading above).
- A missing or late release as the cause:
  the archive answered HTTP 200 throughout,
  and an earlier install in a fresh cargo home used it.
- Reading the install record to tell the two outcomes apart:
  both write `~/.cargo/.crates.toml`.
  Compare the installed binary's SHA-256 with the archive's instead.

## Upstream filing decision

1. Is it upstream's fault?
   Partly.
   Compiling on a failed resolution is documented behavior with a flag to disable it,
   and a timeout is deliberately non-fatal.
   What reads as a defect is that a rate-limit sleep is counted against the resolution timeout
   and that nothing names the cause before the fallback.
   That is closer to diagnostics than to wrong behavior,
   so this constraint is not clearly met.
2. Can upstream fix it?
   Yes: the sleep and the timeout are ordinary code in two files.
3. Do they support this use case?
   Yes: GitHub-release `pkg-url` resolution is the main path.
4. Would the repository welcome a contribution?
   No policy found.
   The research pass found no `CONTRIBUTING.md`, issue template, pull request template, code of conduct or
   AI-assistance rule, and `README.md` invites contributions in general terms.
5. Will they likely fix it?
   No signal either way.
   Open issue `cargo-bins/cargo-binstall#1560` ("feature: Improve retrying logic") says binstall
   "often fails in our very own CI and fallback to cargo-install".
   Closed `#2705` is a malformed `GITHUB_TOKEN` silently disabling resolution and falling back to compiling.
   The research pass read only the openings of `#1560`, `#2705`, `#1229` and `#822`, not the threads.
6. Prototype?
   Not done.
   Constraint 1 is not clearly met,
   so the default applies: do not file.

`.out-of-scope/` has no entry for cargo-binstall.
Duplicate search found no issue stating that unauthenticated rate limiting makes binstall compile silently,
so nothing is drafted as a new issue.
If a future session wants to contribute,
read the full `#1560` thread first and comment there only with what it lacks:
the observation that `gh_crate_meta.rs:119-120` sleeps inside the future that
`resolve.rs:171-175` bounds at 15 seconds,
so a rate limit surfaces as a timeout followed by a silent compile.
