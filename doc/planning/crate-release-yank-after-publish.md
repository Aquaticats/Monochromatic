# Yanking older crate releases right after a new one is published

Status: the one-time yank finished at 2026-11-05T00:07:16Z;
every one of our crates on crates.io now has exactly one unyanked version, its newest.
The automatic step in `package/crate-release/cli` is not built,
and is on hold behind the owner's answer in `doc/planning/crates-io-footprint.md`:
yanking keeps every version stored on crates.io,
so it does nothing for the owner's later question "Can we avoid polluting crates.io too much?".

## What the owner asked

2026-11-04, verbatim:
"For every crate we publish except forbidden-strings and forbidden-regex, add a note in README to say
\"EXTREMELY UNSTABLE AND UNFINISHED DO NOT USE\" and yank old releases immediately after new releases are published."
Then: "I cargo login ed again with publish-new, trusted-publishing, and yank permissions."

The crates are the 14 the publish workflow `.github/workflows/cargo-publish.yml` releases other than
`forbidden-strings` (`package/forbidden-strings/cli`) and `forbidden-regex` (`package/forbidden-strings/core`):
`monochromatic-browserslist`, `monochromatic-cargo-shadow`, `monochromatic-catalog-specifier`,
`monochromatic-dependency-catalog`, `monochromatic-file-enforcer-cli`, `monochromatic-file-enforcer`,
`monochromatic-git-policy-cli`, `monochromatic-jsonc-edit`, `monochromatic-lint`, `monochromatic-mpm-shadow`,
`monochromatic-nested-wayland-session`, `monochromatic-pnpm-shadow`, `monochromatic-search-shadow`,
`monochromatic-xml-edit`.
The README notice landed in `1694ce1816`, as a plain paragraph under each title
(a bold-only line is reported by `markdown/no-emphasis-as-heading`);
crates.io shows it from each crate's next release.

## Where the yank can run

- CI publishes with crates.io Trusted Publishing (`rust-lang/crates-io-auth-action`, `cargo-publish.yml`).
  That token cannot yank:
  crates.io's yank endpoint checks an API token with the yank scope
  (`src/controllers/version/update.rs` at `30c4ed7fd9b79eba10e42d5c173e9bdd8aab1b4c`:
  `AuthCheck::default().with_endpoint_scope(EndpointScope::Yank)`),
  and the trusted-publishing access token is accepted only by the publish, exchange and revoke controllers
  (`src/controllers/krate/publish.rs`, `src/controllers/trustpub/tokens/`).
- So the yank runs on this machine, with the owner's `cargo login` token (yank scope),
  which `cargo yank` reads from Cargo's own credentials file; no tool reads, passes or prints it.
  The natural place is `crate-release`'s wait command, which already follows the publish run to the new version on
  crates.io (`package/crate-release/cli/README.md`, section "Wait command").

## What yanking does to installs

- cargo-binstall never picks a yanked version:
  `crates/binstalk-registry/src/common.rs` skips index entries with `yanked` set,
  and `crates_io_registry.rs` returns nothing for an exact version that is yanked
  (cargo-binstall at `c3664f981c04ab538bd76b0ba83680ee5bab631a`).
- So a toolchain lock row that names a yanked version can no longer be installed fresh;
  copies already installed keep working.
  Between the yank and the commit that locks the new version, a fresh install from the old lock fails
  (other worktrees on an older lock, CI on an older commit).
  Rows that carry an archive and checksum would not depend on the registry; the wrapper's row has none today.
- An exact pin between our crates (`monochromatic-file-enforcer = { path = "../core-rust", version = "=0.1.1" }`)
  to a yanked version leaves the dependent's newest release resolvable only from its packaged lock file,
  so a library release yanks cleanly only when its dependents are released in the same run.

## One-time yank of releases already superseded

Measured on crates.io on 2026-11-04: of the 14 crates, only four had unyanked versions older than the newest:
`monochromatic-git-policy-cli` (40), `monochromatic-file-enforcer-cli` (2), `monochromatic-file-enforcer` (1),
`monochromatic-dependency-catalog` (1); the other ten already had every older version yanked.
None of the 44 is named by `toolchain.lock.json` (which names `monochromatic-git-policy-cli` 0.5.34 and
`monochromatic-file-enforcer-cli` 0.1.2, the newest) or by an exact pin in a newest release
(`monochromatic-dependency-catalog` `=0.2.0`, `monochromatic-file-enforcer` `=0.1.1`, the newest).
The owner then asked for these right now, naming the seven crates that carry older versions or are new.
They are yanked one at a time by `~/temp/agent/issue-6261/yank-older-releases.ts`
(log `~/temp/agent/issue-6261/yank-log.txt`); undo with `cargo yank --undo --version <v> <crate>`.

Result, from the log:
nine versions were yanked
(`monochromatic-dependency-catalog` 0.1.0, `monochromatic-file-enforcer` 0.1.0,
`monochromatic-file-enforcer-cli` 0.1.1 and 0.1.0,
`monochromatic-git-policy-cli` 0.5.18, 0.5.17, 0.5.16, 0.5.15 and 0.5.14).
When the run read `monochromatic-git-policy-cli` at 2026-11-05T00:01:26Z,
only five of the 40 older versions counted earlier were still unyanked;
this session did not yank the other 35,
and nothing here records who did.
A read of all 16 of our crates on crates.io at about 2026-11-05T00:10Z
(`~/temp/agent/issue-6261/crates-io-footprint.ts`)
found one unyanked version each, the newest.

## The automatic step (to build)

In `crate-release`'s wait command, after the new version is confirmed on crates.io:
list the crate's versions, yank every unyanked one older than the new version, one at a time,
for every crate except `forbidden-strings` and `forbidden-regex`;
report each yank; a failure (no login, missing scope, rate limit) is reported with the remedy and fails the command.
Proposed order, open to veto: the step runs immediately after the publish, as asked,
and the wait command then names the lock update as the next step, because a lock still naming the yanked version
cannot be installed fresh.

## Rate limits met on 2026-11-04

The first yank answered 429:
"You have yanked or unyanked too many versions in a short period of time."
crates.io allows a burst of 100 yanks per account and then one a minute
(`src/rate_limiter.rs` at the commit read on 2026-11-04: `YankUnyank => 60` seconds, burst `100`),
so the account had used its burst before this run,
and each yank after the burst waits about one minute.
A second form of 429 also came back: an empty body from the cache in front of the registry
(`via: 1.1 varnish`, `content-length: 0`), naming no time;
the script waits 65 seconds on that one.
The read API answered 429 too after the version counts, so the script waits out both,
retrying at the time each answer names, within a two-hour deadline
(systemd unit `crates-io-yank-older-releases`, log `~/temp/agent/issue-6261/yank-log.txt`).
The automatic step must do the same: one release yanks its older versions within the burst,
but a run after a large batch of yanks waits a minute per version.
