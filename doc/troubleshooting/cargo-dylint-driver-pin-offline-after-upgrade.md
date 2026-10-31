# cargo-dylint 6.3.0 pins `dylint_driver = "=6.3.0"`; the offline lint run exits 2 after an upgrade

Evidence date 2026-10-30,
host with `cargo-dylint` 6.3.0 and `dylint-link` 6.3.0 installed
while `toolchain.lock.json` records 6.3.1,
cargo registry cache holding only `dylint_driver` 6.2.0,
and `rustc 1.101.0-nightly (19833af72 2026-10-20)` as the default `nightly`.
The OpenSnitch rule that blocks nightly `cargo` network access is a different cause
(`cargo-nightly-1-101-cannot-reach-crates-io.md`);
the prefetch advice in `dylint-library-tests-and-shared-build-dir.md` is the same mechanism seen from the other side.

## Symptom

`monochromatic-lint` runs `rust/avoidable-closure` through `cargo dylint` for every Rust package it selects.
On the packages `package/xml-edit/core-rust` and `package/dependency-catalog/core-native`
the run ends with exit status 2 and one JSONL finding:

```text
# monochromatic-lint package/xml-edit/core-rust (stderr of cargo, quoted inside the finding message)
Building driver for toolchain `nightly-x86_64-unknown-linux-gnu`
error: failed to select a version for the requirement `dylint_driver = "=6.3.0"`
candidate versions found which didn't match: 6.2.0
location searched: crates.io index
required by package `dylint_driver-nightly-x86_64-unknown-linux-gnu v0.1.0 (/tmp/.tmpq3Pdif)`
note: offline mode (via `--offline`) can sometimes cause surprising resolution failures
help: if this error is too confusing you may wish to retry without `--offline`
```

```json
{"message":"rust/avoidable-closure could not check this package: `cargo dylint` exited with exit status: 1. ...",
 "code":"core/processing-failure","severity":"error","filename":"package/xml-edit/core-rust/Cargo.toml"}
```

Two agents reported it on 2026-10-30;
one saw a later run pass and the other never did.
The failing step does not depend on the package (only `xml-edit/core-rust` was re-run here),
because it is the driver build that precedes every check.

## Root cause

Three facts combine.

### The pin is the version of `cargo-dylint` itself

`cargo dylint` needs a driver binary per toolchain,
cached as `~/.dylint_drivers/<toolchain>/dylint-driver`.
It builds a missing driver by writing a throwaway crate whose manifest pins `dylint_driver` to the version of the
`dylint` library crate that is compiled into the running `cargo-dylint`
([`driver_builder.rs#L183`][pin],
[`#L219-L222`][manifest]):

```rust
// dylint/src/driver_builder.rs:183 (tag v6.3.0)
let version_spec = concat!("version = \"=", env!("CARGO_PKG_VERSION"), "\"");
```

```rust
// dylint/src/driver_builder.rs:219-222 (tag v6.3.0)
[dependencies]
anyhow = "1.0"
env_logger = "0.11"
dylint_driver = {{ {dylint_driver_spec} }}
```

The installed binaries are 6.3.0 (`~/.cargo/.crates.toml:30-31`:
`"cargo-dylint 6.3.0 (registry+https://github.com/rust-lang/crates.io-index)"` and the same for `dylint-link`),
so the pin is `=6.3.0`.
Nothing in `package/linter/monochromatic-lint-closure` or `rust-catalog.toml` pins `dylint_driver`;
their `dylint_linting` and `dylint_testing` requirements are `>=6.2` and resolve to 6.2.0 in the lockfile.
`dylint-link` never resolves `dylint_driver`.

### Every driver built by an older `cargo-dylint` counts as outdated

An existing driver is reused only if its own version is not lower than the running `cargo-dylint`
([`driver_builder.rs#L59-L61`][reuse],
[`#L105-L107`][outdated]):

```rust
// dylint/src/driver_builder.rs:59-61 (tag v6.3.0)
let driver = driver_dir.join("dylint-driver");
if !driver.exists() || is_outdated(opts, toolchain, &driver)? {
    build(opts, toolchain, &driver_dir)?;
```

```rust
// dylint/src/driver_builder.rs:105-107 (tag v6.3.0)
let our_version = Version::parse(env!("CARGO_PKG_VERSION"))?;

Ok(their_version < our_version)
```

A failure to run the existing driver also counts as outdated (`#L109-L112`,
`Ok(true)` after a warning).
The three drivers on this host (`nightly-2026-10-18-...`,
`nightly-2026-10-19-...`,
`nightly-...`)
all embed `dylint_driver-6.2.0`,
so after the upgrade to 6.3.0 each of them is rebuilt on its next use.
The call site is `dylint/src/lib.rs:346` (`let driver = driver_builder::get(opts, toolchain)?;` in `check_or_fix`),
which runs for every `cargo dylint` check.

### The linter forbids the network and the cache has no 6.3.0

`package/linter/monochromatic-lint/src/closure_cargo.rs:290` sets `CARGO_NET_OFFLINE=true` on the `cargo dylint` child,
which passes it to the `cargo build` of the throwaway crate.
The host registry has `dylint-6.2.0`,
`dylint_driver-6.2.0`,
`dylint_internal-6.2.0`,
`dylint_linting-6.2.0` and
`dylint_testing-6.2.0` (all downloaded 2026-10-18) and nothing newer.
The sparse-index entry `~/.cargo/registry/index/index.crates.io-1949cf8c6b5b557f/.cache/dy/li/dylint_driver`
was last written 2026-10-18 and its last version is 6.2.0,
hence "candidate versions found which didn't match:
6.2.0".
Upstream published 6.3.0 on 2026-10-28,
6.3.1 on 2026-10-29 and 6.3.2 on 2026-10-30
(`gh release list --repo trailofbits/dylint`),
so a registry refreshed on 2026-10-18 cannot know any of them.

### Why this surfaces as a failure and not as the documented skip

The linter's contract (`package/linter/monochromatic-lint/README.md`,
"The linter never fetches") is that a cold cache
skips the package with the command that fills it.
The readiness probes do that for the linted package only:
`dependencies_missing` (`closure_cargo.rs:215`) runs `cargo metadata --offline --locked` on the package's manifest,
and `dylint_available` runs `cargo dylint --version`.
The dependencies of the driver crate belong to no package lockfile and no probe reads them,
so the gap bypasses the skip and the first `run_dylint` call turns it into `core/processing-failure` and exit 2
(`closure_step.rs`,
`check_package`,
the `!run.success` branch).
The README of `package/linter/monochromatic-lint-closure` says the driver cache lives in `~/.dylint_drivers`
and the planning notes record "the first-build cost and its network use were not measured"
(`doc/planning/closure-lint-everywhere.md:760`):
the network need was never written down.

### Why the host is on 6.3.0 while the lock says 6.3.1

`toolchain.lock.json` moved `cargo:cargo-dylint` and `cargo:dylint-link` from 6.2.0 to 6.3.0 in `d7ee4ac109`
(2026-10-29 03:11) and to 6.3.1 in `db50050155` (2026-10-29 20:56),
both with `request.track = "latest"`.
`mise run toolchain:status` on 2026-10-30:

```text
drift   cargo:cargo-dylint  locked=6.3.1  observed=6.3.0
drift   cargo:dylint-link  locked=6.3.1  observed=6.3.0
```

The mtime of `~/.cargo/bin/cargo-dylint` (2026-10-28 10:21 EDT) precedes the 6.3.0 release (14:26 UTC),
so it is most likely the build time kept from the archive and does not date the install;
the install happened after `d7ee4ac109` and before the first report.
The upstream pin logic is identical in `v6.2.0`,
`v6.3.0` and `v6.3.2` (`cmp` of `driver_builder.rs` between 6.2.0 and
6.3.2 is empty),
so every future `cargo-dylint` bump repeats this until the driver is rebuilt once with network access.

### The intermittency, as far as it was measured

On this host's default paths the failure is deterministic:
no `dylint_driver-6.3.0.crate`,
no 6.3.0 driver and no 6.3.0 index entry exist,
and two consecutive real `monochromatic-lint package/xml-edit/core-rust` runs on 2026-10-30 both exited 2 with the
message quoted in the symptom section.
The checkout is not a factor:
`.cache/toolchain-bin/cargo-dylint` is a symlink to `~/.cargo/bin/cargo-dylint`,
and both the driver cache and the registry are per user.
A pass therefore needs a run that either reached the network (without `CARGO_NET_OFFLINE`) or used another
`CARGO_HOME` or `DYLINT_DRIVER_PATH`;
arms D and E of the verification section are exactly those two conditions.
Which of them the passing agent used was not recoverable and is not guessed here.
Two further conditions make a run skip the Dylint step without failing,
and were read in source,
not tested:
`--fix` skips the closure step (`closure_step.rs`,
doc of `run_closure_step`),
and a run whose selected files include no Rust file of the package never reaches `check_package`.

## Verification

Versions under test:
`cargo-dylint` 6.3.0 (`sha256` prefix `3d628fd98ba079536743`) and `dylint-link` 6.3.0
(`a0b9b12a405af3720503`) from `~/.cargo/bin`;
upstream tag `v6.3.0` = `39f72bb3b6c5baf402ae0ae865852d11868a5a0c`;
toolchain `nightly-2026-10-30-x86_64-unknown-linux-gnu` selected through `RUSTUP_TOOLCHAIN`
(cargo-dylint strips that variable for the driver build,
so the driver directory was
`nightly-x86_64-unknown-linux-gnu`,
the rustup default,
in every arm);
the lint library `package/linter/monochromatic-lint-closure` and the crate `package/xml-edit/core-rust`,
both copied into a scratch directory.
The permalinks in the root cause section were loaded in `agent-browser`,
and the highlighted lines matched the quoted text.

Real-tool run,
in the repository,
with the host environment:

```text
# cd /var/home/user/Monochromatic
$ monochromatic-lint package/xml-edit/core-rust
... "code":"core/processing-failure" ...  (exit 2, twice)
```

Harness for the arms:
a scratch `CARGO_HOME` holding copies of `registry/index` and `registry/cache`
(no `build-dir` from the host `~/.cargo/config.toml`),
a scratch `DYLINT_DRIVER_PATH` and `CARGO_TARGET_DIR`,
`PATH` with `~/.cargo/bin`,
and `RUSTUP_AUTO_INSTALL=0`.
Each arm ran as a capped unit
(`systemd-run --user --collect --wait --pipe --property=MemoryMax=4G --property=CPUQuota=200%`).

```ts
// scratch run.ts: node run.ts <scratch> <driver-dir> <offline|online> <cargo-home>
import { spawnSync } from 'node:child_process';
const [scr, drivers, offline, cargoHome] = process.argv.slice(2);
const home = process.env.HOME!;
const env = {
  PATH: `${home}/.cargo/bin:/usr/local/bin:/usr/bin:/bin`,
  HOME: home,
  RUSTUP_HOME: `${home}/.rustup`,
  RUSTUP_TOOLCHAIN: 'nightly-2026-10-30-x86_64-unknown-linux-gnu',
  RUSTUP_AUTO_INSTALL: '0',
  CARGO_HOME: cargoHome,
  DYLINT_DRIVER_PATH: drivers,
  CARGO_TARGET_DIR: `${scr}/target`,
  DYLINT_RUSTFLAGS: '-W avoidable_closure',
  ...(offline === 'offline' ? { CARGO_NET_OFFLINE: 'true' } : {}),
};
const result = spawnSync('cargo', ['dylint', '--path', `${scr}/pkg/lint-closure`,
  '--manifest-path', `${scr}/pkg/xml-edit-core-rust/Cargo.toml`, '--', '--locked'], { env, stdio: 'inherit' });
console.log(`exit ${result.status}`);
```

Failing catalog (the exact symptom,
`=6.3.0`,
candidate `6.2.0`):

- Arm A:
  empty driver directory,
  registry copy with 6.2.0 only,
  offline.
- Arm B:
  the host's real 6.2.0 driver copied into the driver directory,
  same registry,
  offline.
  The existing driver did not help,
  matching the `their_version < our_version` rule.
- The two real `monochromatic-lint` runs on the host.

Working catalog (control,
exit 0,
`Checking monochromatic-xml-edit`):

- Arm C:
  empty driver directory,
  online,
  fresh registry copy.
  The registry gained 6.3.0 and the driver build took 24.6 s
  (service runtime 1 min 26 s,
  peak memory 1.1 GB,
  including the library build).
  The new driver embeds `dylint_driver-6.3.0`.
- Arm D:
  the driver from arm C,
  the old registry copy with 6.2.0 only,
  offline.
  A current driver alone makes the offline run pass,
  so the cache that matters is the driver,
  not the registry.
- Arm E:
  empty driver directory,
  the warmed registry copy from arm C,
  offline.
  A warm registry alone makes the offline run build the driver and pass.

Not run:
`cargo-dylint` 6.3.1 against these caches.
The prebuilt 6.3.1 archive (`sha256` `e5ea1643...3b14d78`,
matches its `.sha256`) was downloaded into scratch,
but running it was refused by the session's permission classifier and it was not retried.
The 6.3.1 behaviour is inferred from the source identity of `driver_builder.rs` across 6.2.0,
6.3.0 and 6.3.2.

## Verified workarounds

### Warm the caches once with network access (works today, arms C to E)

Run one `cargo dylint` check without `CARGO_NET_OFFLINE`,
for example:

```text
# cd /var/home/user/Monochromatic
$ env --unset=CARGO_NET_OFFLINE cargo dylint --path package/linter/monochromatic-lint-closure \
    --manifest-path package/xml-edit/core-rust/Cargo.toml -- --locked
```

It refreshes the sparse index,
downloads the 6.3.0 crates of `dylint_driver`,
builds
`~/.dylint_drivers/nightly-x86_64-unknown-linux-gnu/dylint-driver` and checks the crate.
A cargo whose executable path has no OpenSnitch allow rule hits the block described in
`cargo-nightly-1-101-cannot-reach-crates-io.md`,
so use a toolchain `cargo` that has one.
Tradeoffs:
it writes the real `~/.cargo/registry` and `~/.dylint_drivers`,
so it needs the owner's go-ahead here;
it warms the version of the installed `cargo-dylint` (6.3.0 today),
so after `mise run toolchain:sync` installs 6.3.1
the pin becomes `=6.3.1` and the failure returns until the same step runs again;
and the registry part is shared by every toolchain directory while the driver part is per toolchain directory,
so a changed rustup default needs only an offline rebuild once the registry is warm (arm E).

### Converge the host to the lock first, then warm (recommended order)

`mise run toolchain:sync` installs `cargo-dylint` and `dylint-link` 6.3.1 (the drift in the root cause section),
and the warm step then targets 6.3.1.
Installing tools is the owner's decision;
this investigation did not run any installer.
Tradeoff:
warming before the sync wastes the work,
and syncing alone changes the pin to `=6.3.1`
and fixes nothing,
because 6.3.1 is not in the registry either.

## Options for a durable fix

Ranking:
B > C > A > D.
B beats C because it repairs the unchecked cache the linter's own contract says it probes,
without giving `toolchain:sync` a network step.
C beats A because it removes the manual step on every bump,
at the price of widening what sync writes.
A beats D because it costs nothing in the repository and fixes the failure today,
while D only slows the bumps and leaves every bump to be warmed.
The immediate unblock is A (owner-approved);
the durable repair is B.

- A,
  warm by hand after each `cargo-dylint` bump,
  with the command in the first workaround.
  Pros:
  no repository change.
  Cons:
  the failure returns on every bump,
  and 6.3.0,
  6.3.1 and 6.3.2 came out within three days.
- B,
  make `monochromatic-lint` treat a missing or outdated driver as a skip.
  A probe before `run_dylint` would compare the version printed by `cargo dylint --version` with the driver
  (or run `cargo metadata --offline` on a scratch manifest that pins `dylint_driver = "=<version>"`)
  and,
  on a cold cache,
  push `skip_message(...)` naming the warm command,
  as `dependencies_missing` does.
  Files:
  `package/linter/monochromatic-lint/src/closure_cargo.rs` and `closure_step.rs`,
  their `*_tests.rs`,
  and the README paragraph "The linter never fetches".
  Pros:
  honours the documented never-fetch contract,
  and a skip is not a finding,
  so exit status stays 0.
  Cons:
  Rust change with tests;
  a skip hides the missing check until somebody warms.
- C,
  give `toolchain:sync` ownership of the warm step for `cargo:cargo-dylint`
  (`package/file-enforcer/core/src/toolchain/plan.tools.data.ts` and the sync executor).
  Pros:
  the component that installs the version also pays for its consequence.
  Cons:
  sync would need network and write `~/.cargo` and `~/.dylint_drivers`,
  which it does not touch today.
- D,
  replace `request: { track: 'latest' }` for the two Dylint tools with an exact request in
  `plan.tools.data.ts`,
  then regenerate `toolchain.lock.json` through the repository flow.
  Pros:
  bumps become deliberate.
  Cons:
  does not repair the missing probe,
  and any bump still needs the warm step.

Every option that runs `mise run toolchain:sync`,
`toolchain:update` or writes `~/.cargo` or `~/.dylint_drivers` needs
the owner's approval.
A and the warm part of the recommended order change no repository file;
B,
C and D change the files named in each.

## What does not work

- Concurrent runs racing on the driver directory:
  upstream pull request
  [trailofbits/dylint#2155][flock] ("`flock` Dylint drivers directory when installing drivers",
  opened 2026-10-31 UTC
  by the maintainer,
  open) concerns racing installs,
  while this error is a resolution failure in a fresh temporary directory before any install.
- A per-worktree cause:
  the driver cache and registry are per user,
  and the linter binary and `cargo-dylint` are
  shared through symlinks.
- A package-level `cargo fetch --locked`:
  it fills the package's dependencies and not those of the throwaway
  driver crate,
  so `dependencies_missing` passes and the failure remains.
- Syncing the host to 6.3.1 alone:
  it changes the pin to `=6.3.1`,
  which is also absent from the registry.
- Reading the default toolchain as the cause:
  the driver directory is `nightly-x86_64-unknown-linux-gnu`
  (the rustup default,
  `rustc 1.101.0-nightly (19833af72 2026-10-20)`),
  not the locked `nightly-2026-10-30`.
  This differs from the lock but does not produce the version error:
  drivers for all three toolchain directories
  are 6.2.0 and outdated alike.

## Upstream filing decision

Checked on 2026-10-30.

- `.out-of-scope/` holds `bun-install.md`,
  `cargo-workspace.md` and `claude-code-upstream-bugs.md`;
  `rg --files-with-matches --ignore-case "dylint|cargo-dylint"` over it returns nothing,
  so no exemption applies.
- Constraint 1 (really upstream's fault) fails.
  The exact pin keeps the driver in step with the binary on purpose,
  and upstream's README recommends caching
  `~/.dylint_drivers/` between runs.
  The offline policy and the missing probe are this repository's.
- Duplicate search:
  `gh search issues --repo trailofbits/dylint` for `offline driver`,
  `driver build network`,
  `=version dylint_driver` and `dylint_drivers outdated` returned no results;
  `gh search prs ... driver_builder` returned the merged #1298 only;
  open pull requests matching `driver` are #2155 (see the excluded causes)
  and #1985 (embedded source paths),
  neither related.
- Work in progress:
  the latest commit touching `dylint/src/driver_builder.rs` is `968fda312` (2026-09-18).
- No `CONTRIBUTING.md` trust or AI-assistance ban was found by `rg` for `AI`,
  `LLM`,
  `generated` and `agent`,
  and no `VOUCHED` file;
  this is moot because nothing is drafted.

Nothing to file or comment.
The consumer-side fix is a repository change (option B) and an owner-approved warm step.

[pin]: https://github.com/trailofbits/dylint/blob/39f72bb3b6c5baf402ae0ae865852d11868a5a0c/dylint/src/driver_builder.rs#L183
[manifest]: https://github.com/trailofbits/dylint/blob/39f72bb3b6c5baf402ae0ae865852d11868a5a0c/dylint/src/driver_builder.rs#L219-L222
[reuse]: https://github.com/trailofbits/dylint/blob/39f72bb3b6c5baf402ae0ae865852d11868a5a0c/dylint/src/driver_builder.rs#L59-L61
[outdated]: https://github.com/trailofbits/dylint/blob/39f72bb3b6c5baf402ae0ae865852d11868a5a0c/dylint/src/driver_builder.rs#L105-L107
[flock]: https://github.com/trailofbits/dylint/pull/2155
