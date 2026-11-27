# The embedding control on a file system that refuses a name that is not UTF-8

Recorded 2026-11-27T01:29:00-05:00 by the round
`issue-6491-forbidden-strings-embedding-control-runs-on-a-utf8only-home`,
on branch `fix/forbidden-strings-cli-issue-6491-embedding-control-runs-on-a-utf8only-home`,
from `main` `f1957916b0`.
Not approved and not final.

## What failed, and why

`tests/embedding.rs`'s `native_rule_path` control makes a rule file named `rules-\xff.txt`
under `CARGO_TARGET_TMPDIR` and proves its native path keeps those bytes through the cache repair.
On this host that folder is `target/tmp` in the checkout,
which `cargo-shadow` redirects into its own build
folder,
and both sit on `zpcachyos/ROOT/cos/home`,
whose `utf8only=on` answers `EILSEQ` (`errno` 84) for a
name that is not valid UTF-8.
So the control never ran here,
and the package's `test` task was red for a reason that has nothing to do
with the scanner (issue 6491).
Cause A of `doc/troubleshooting/git-policy-cli-container-gate-on-a-zfs-host.md` holds the file system's
side of this refusal.

Red on `main`'s code,
one process for the one test:

```text
thread 'embedded_consumer_probe' panicked at tests/embedding.rs:90:51:
native rule file: Os { code: 84, kind: Uncategorized, message: "Invalid or incomplete multibyte or wide character" }
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1 filtered out
```

## The answer

`tests/support/fixture_root_taking_any_name.rs` asks the fixture root whether it takes the name,
and where it does not,
starts the same test binary again for the same test
(`public_embedding_runs_without_real_user_state`) inside
`unshare --user --map-root-user --mount --propagation private`,
mounts a tmpfs at that root,
and lets the inner half run the control.
This is the answer the Git wrapper's crate wrote for the same refusal in issue 6448
(`package/git-policy/cli/src/native/testing/fixture_root_taking_any_name.rs`),
copied into this package because this crate does not depend on the wrapper's.

What the copy decides for itself:

- The mount lives only in the namespace the inner half made,
  so nothing on the host is mounted and no fixture
  is made under `/tmp` or `/dev/shm`.
- Where the fixture root already takes the name,
  which is what the container gates give it,
  no namespace is
  started and the control runs as it always did.
- There is no acknowledgement variable.
  A host that refuses both the name and an unprivileged user namespace
  fails the control and names it,
  because a run in which this control skipped would pass while proving
  nothing,
  and this package's `test:container` task gives it a fixture root that takes the name.
- `SECURE_NOROOT` is not set,
  unlike the wrapper's copy:
  that bit answers a fixture's Git asking the
  capability module for a capability its confinement refuses (issue 6459),
  and the only program this control
  starts is its own test binary.
- The probe name carries this process's number and is made with `File::create_new`,
  so a probe left by another
  run,
  or by a run that had this very process number,
  fails the control instead of being overwritten and
  removed as though this run had made it.
- Before mounting,
  the inner half proves with `/proc/self/uid_map` that its id 0 stands for another id outside:
  the variable alone would mount in whatever namespace a process that inherited it runs in.
- The inner half is waited for with a deadline and ended if it never ends;
  the control itself took under 0.1 s
  in every run here in a release build.

On a Unix platform with no unprivileged user namespace the control fails and names itself when the root
refuses the name,
and on a platform whose paths are not byte strings the control and this guard do not exist
at all.

## The hold on a quiet skip

`native_rule_path` prints one line,
`NATIVE RULE-FILE NAME CONTROL RAN`,
once it has run,
and the driver
requires that line in its child's output.
A control that returned early on a refusing file system therefore fails the run:
with a stand-in quiet skip in
the control,
the driver failed with

```text
the native rule-file control did not run in the child, so a file system that refuses its name would go unnoticed
```

## Green after

```text
PASS [   0.016s] (1/1) forbidden-strings::embedding public_embedding_runs_without_real_user_state
the control public_embedding_runs_without_real_user_state ran inside a user and mount namespace of its own,
with a tmpfs at ~/.cargo/build/toolchain/nightly-2026-11-16/44/7fdd6ab15b6390/tmp, because the file system
there refused a name that is not UTF-8
```

Both lines rest on real invocations of the test binary on this host on 2026-11-27.

## Checks

Run on 2026-11-27 in `package/forbidden-strings/cli` on the branch after its last merge of `origin/main`,
each once,
each inside its own `systemd-run --user --scope --slice=agent-work.slice` with `MemoryMax`,
`CPUQuota`,
`TasksMax`,
`IOWeight=1` and `RuntimeMaxSec`:

- `cargo fmt --check`:
  no output,
  exit 0.
- `cargo check --locked --offline --all-targets`:
  exit 0.
- `cargo clippy --locked --offline --all-targets -- -D warnings`:
  exit 0,
  no warning.
- `mise run //package/forbidden-strings/cli:test`:
  `365 tests run: 365 passed, 0 skipped`.
- `/home/user/Monochromatic/.cache/toolchain-bin/monochromatic-lint package/forbidden-strings/cli`,
  from the
  repository root:
  exit 0.
  Its one line is a `rust/avoidable-closure` warning,
  not a finding:
  it says the Dylint driver for this toolchain is absent and that nothing in this package was checked by that
  rule,
  so the run says nothing about closures.
  Its `--fix` pass rewrapped the prose in this package and removed the two pre-existing
  `markdown/semantic-line-breaks` findings the package's README carried.

Not run,
and why:

- `mise run //package/forbidden-strings/cli:lint:clippy:windows`:
  no `x86_64-pc-windows-gnu` target is installed
  on this machine,
  so the not-Unix side of this change is reasoned and not compiled here.
- `mise run //package/forbidden-strings/cli:verify:markdown`:
  it passed on this tree before the last merges and
  fails after them with `Expected one installed unified renderer, found 0`,
  because this worktree's
  `node_modules/.pnpm` holds only `lock.yaml`;
  a `pnpm install` in a worktree is not this round's to run.
  The repository linter's CommonMark pipeline still parsed this document and the README.
- The package's container tasks,
  its mutation tasks and its fuzz tasks:
  outside this change's subject.
