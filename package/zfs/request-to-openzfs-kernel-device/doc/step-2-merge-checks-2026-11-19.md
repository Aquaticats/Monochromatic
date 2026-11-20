# Step 2's merge checks: the gate image, the fuzz smoke and the guest run

**Non-final,
unapproved.**
This records what an agent ran and what each check answered;
it is not the project's
acceptance of step 2,
and the owner alone decides that.

It records the checks the plan's "How it merges" names for step 2
([`cli-git-zfs-worktree-implementation-plan.md`](../../../../doc/planning/cli-git-zfs-worktree-implementation-plan.md),
"How it merges" and "Step 2"):
the tests and Clippy in the package's gate image,
`file-enforcer write` and `check`,
the bounded fuzz smoke,
and the repository linter;
then the guest recipe that is the step's own acceptance.
The fuzz smoke is the one the earlier integration pass could not run,
so this round closes the merge sentence
"the bounded fuzz smoke passes" with its own run.
The live arms' measured values are in
[`guest-measurements-2026-11-19.md`](guest-measurements-2026-11-19.md);
this file records the checks around them.

The logs live outside the repository,
in this round's `keep/round-evidence/` folder beside the
brief that asked for the run,
which is
`/home/user/temp/agent/zfs-step-2-integration-pass-findings-fixed-2026-11-19/keep/round-evidence/`,
and this file cites them by name.

## The gate image and its checks

Image:
`localhost/zfs-request-to-openzfs-kernel-device-gate:zfs-step-2-live-requests-2026-11-19`
(ID `19b0b4b5bcf9`).
It was neither rebuilt nor retagged this round.

Every check ran as
`podman run --rm --init --network none --cpus 4 --memory 4g --pids-limit 512
--blkio-weight 10`,
with the worktree mounted read-only at `/work`,
a scratch `CARGO_TARGET_DIR`,
and `CARGO_NET_OFFLINE=true`.
The two `cargo fmt --check` checks ran with `--cpus 2`;
the `file-enforcer write` check ran on a writable tracked-only copy instead of
the read-only mount.
The driver recorded each real exit status in
`container-checks-summary.log`,
and each check's whole output is `container-<check>.log`:

1.  `cargo nextest run --release --locked --offline --status-level all`:
    exit 0,
    `177 tests run: 177 passed, 30 skipped`
    (the 30 are the `#[ignore]`d live tests).
2.  `cargo clippy --release --locked --offline --all-targets -- -D warnings`:
    exit 0.
3.  `cargo fmt --check` (crate):
    exit 0,
    no output.
4.  `cargo fmt --check` (sidecar):
    exit 0,
    no output.
5.  `RUSTDOCFLAGS=-Dwarnings cargo doc --no-deps --locked --offline`:
    exit 0.
6.  Sidecar `cargo check --locked --offline`:
    exit 0.
7.  Sidecar `cargo clippy --locked --offline -- -D warnings`:
    exit 0.
8.  `file-enforcer check` on the read-only worktree mount:
    exit 0 in this round's own worktree,
    `check-clean`,
    `drifted: 0`.
    That result is not reproducible from a fresh worktree:
    the check reads `.config/mise/conf.d/10-bin.local.toml`,
    a gitignored local file the wrapper copies into a new worktree,
    and a copy that does not match `mise-bin-list` drifts there.
    The lows round measured exit 1 on that one file in a fresh worktree,
    and the same drift in the main checkout (check 9),
    so the branch's own result is check 10.
9.  `file-enforcer check` on the main checkout,
    read-only,
    same image:
    exit 1,
    the same single drift,
    `.config/mise/conf.d/10-bin.local.toml`
    with rule `mise-bin-list`,
    so it is not the branch's.
10. `file-enforcer write` on a tracked-only copy of `HEAD`:
    exit 0,
    29 files written,
    every one of them gitignored and none tracked;
    `git diff --stat` and `git ls-files --others --exclude-standard` are empty,
    and `file-enforcer check` on the copy afterwards is
    exit 0 with `drifted: 0`.
    This copy is the branch's result.
11. `monochromatic-lint package/zfs/request-to-openzfs-kernel-device
    package/zfs/request-to-openzfs-kernel-device.fuzz`:
    exit 0,
    one `warn`:
    `rust/avoidable-closure skipped ... cargo-dylint is not installed`,
    as for every Rust package here,
    so closures in this package were not checked by that rule.
12. The package's type gate on the host,
    `mise run //package/zfs/request-to-openzfs-kernel-device:lint:types`:
    exit 0,
    all three halves:
    `cargo check`,
    `cargo clippy --release --all-targets -- -D warnings`,
    and `tsc --noEmit --project tsconfig.json`.

## The lows round's rerun

The round that answered this review's lows reran the gate checks on its branch,
in the same image,
neither rebuilt nor retagged,
with the same mounts and caps,
one container per check,
each named `zfs-step-2-lows-2026-11-20-<check>`.
Its logs are in this round's `keep/` folder,
`/home/user/temp/agent/zfs-step-2-merge-lows-fixed-2026-11-20/keep/`,
as `container-<check>.log` and `container-checks-summary.log`,
taken between
2026-11-20T01:16:13-05:00 and
2026-11-20T01:16:38-05:00,
with the linter rerun at
2026-11-20T01:17:59-05:00.

- nextest exits 0,
  `178 tests run: 178 passed, 30 skipped`;
  the one more than check 1 is the new device-free test of a failed write's reply.
- Clippy exits 0.
- Both `cargo fmt --check` runs exit 0 with no output.
- `cargo doc` with `-Dwarnings` exits 0.
- The sidecar's `cargo check` and its Clippy exit 0.
- `file-enforcer check` on the worktree mount exits 1 with one drift,
  the gitignored local file check 8 describes;
  the main checkout's copy drifts the same way.
- `file-enforcer write` on a tracked-only copy of the branch exits 0,
  29 files written,
  every one gitignored,
  and `file-enforcer check` on the copy afterwards exits 0 with `drifted: 0`.
- The linter exits 0 with the same single `warn`.
  Its first run in this pass reported two `markdown/semantic-line-breaks` errors on
  lines the lows round had just written;
  both were fixed and its later runs are green.

The red for the new test is
`red-6-failed-write-reply-decode-error-replaces-the-kernel-error.patch`,
also in that folder:
with it,
nextest fails on that test
(`left: None`,
`right: Some(7)`)
and Clippy exits 0.

The recipe changed in this round,
so it ran once in a fresh guest on the `libvirt` backend,
by the route `e2e/README.md` documents,
in the user unit `zfs-step-2-lows-guest-run`
(`MemoryMax=8G`,
`CPUQuota=400%`,
`TasksMax=1024`,
`IOWeight=1`,
`RuntimeMaxSec=3000`):
`all --name zfs-step-2-lows-2026-11-20`.
It exited 0 in 5 min 21 s,
2026-11-20T01:08:44-05:00 to 2026-11-20T01:14:05-05:00.
Its logs are `guest-run/` in that folder:
root `199 tests run: 199 passed, 9 skipped`,
account `9 tests run: 9 passed, 199 skipped`,
no `^SKIP live_` line,
the documented teardown,
and `VM zfs-step-2-lows-2026-11-20 destroyed`.

## The bounded fuzz smoke

`mise run //package/zfs/request-to-openzfs-kernel-device.fuzz:smoke`,
with `ZFS_BINDING_IMAGE_TAG` set so each run built its own image.
It ran twice:
first on the tree as it stood before the last documentation commits
(`fuzz-smoke.log`,
image
`localhost/zfs-request-to-openzfs-kernel-device-fuzz:zfs-step-2-integration-pass-findings-fixed-2026-11-19`),
then on the shipped tree (`fuzz-smoke-shipped.log`,
image
`localhost/zfs-request-to-openzfs-kernel-device-fuzz:zfs-step-2-findings-fixed-shipped-2026-11-19`).
Both passed,
and the shipped tree leaves `src/nvlist/` and the sidecar byte-identical to
`08c9dea25f`,
the tip the first run built
(`git diff 08c9dea25f..HEAD -- src/nvlist request-to-openzfs-kernel-device.fuzz`
is empty).
Each run built the image from the wrapper's gate base
`localhost/git-policy-native-test-base:6fa67475da3a-97221c29742b`,
and ran the target for its 30-second bound with no network,
no home folder,
2 CPUs and 2 GiB.
The shipped run exited 0 and printed:

```text
cargo 1.102.0-nightly (ede6cefea 2026-11-11)
rustc 1.102.0-nightly (5f4effd57 2026-11-13)
cargo-fuzz 0.13.2
Done 1673755 runs in 31 second(s)
```

The first run printed the same versions and
`Done 2152978 runs in 31 second(s)`.
Neither run found a failing input;
the growing corpus and any artifacts stay in
`node_modules/.monochromatic/zfs-binding-fuzz/` of the checkout that ran them,
which is gitignored.
The image's nightly is the one `toolchain.lock.json`'s `rust` row names,
and both crates' locked dependencies were fetched at image-build time,
so the fuzz run itself needed no network.

## The guest run

Run once on the fix round's tree,
by the route `e2e/README.md` documents:

```text
mise run //package/zfs/request-to-openzfs-kernel-device:e2e -- all
  --name zfs-step-2-findings-final --log-dir <keep>/final-guest-logs
```

in a user unit (`zfs-step-2-findings-guest-final`,
`MemoryMax=12G`,
`CPUQuota=400%`,
`TasksMax=1024`,
`IOWeight=1`),
`mvm` `libvirt` backend only.
The recipe exited 0 and printed
`zfs step 2 fixture: step 'all' finished`.
It took 5 min 30 s,
from `mvm-create.log` to `mvm-list-after-destroy.log`.

- Guest:
  kernel `6.8.0-147-generic`,
  OpenZFS userspace and module `2.4.4-1`,
  Git `2.43.0`,
  `pahole v1.25`,
  `strace 6.8`,
  rustc and cargo `1.100.0`,
  `cargo-nextest 0.9.152` (`final-guest-logs/versions.log`).
  The kernel release is asserted by `prepare`,
  and both toolchain versions are the constants the recipe installs.
- Root suite:
  `Summary [   8.835s] 198 tests run: 198 passed, 9 skipped`
  (the 9 are the delegated arms,
  filtered out of the root run).
- Account suite,
  as `ubuntu`:
  `Summary [   0.171s] 9 tests run: 9 passed, 198 skipped`.
- No live arm skipped:
  the count of `^SKIP live_` lines is 0 in both suite logs.
- The markers the recipe requires are present in `root-suite.log`:
  `MEASUREMENT guest kernel=`,
  `layout zfs_cmd bytes=`,
  `MEASUREMENT promoted-batch ` and
  `MEASUREMENT snapshot-ioctl `;
  `account-suite.log` has the `MEASUREMENT account ` lines,
  including the two the new arms print.
- Teardown:
  `teardown.log` lists only the delegated pool `zfsstep2-account`,
  as documented;
  `mvm-destroy.log` records
  `VM zfs-step-2-findings-final destroyed`,
  and `mvm-list-after-destroy.log` reads `no VMs found`.

## The reds

Each new live arm was shown red first,
in a guest,
by a one-hunk patch on the committed code.
Two trees of the package came from `git archive HEAD`;
each patch file is in the same `round-evidence/` folder beside the suite log
it produced.

`red-a-guest-logs/` (guest `zfs-step-2-findings-red-roots`,
kept with `--keep-guest`,
its full root log pulled to `red-a-root-suite.log`)
ran three patches:

- `red-1-snapshot-props-ignored.patch`:
  `ZfsDevice::snapshot` passes `None` instead of the request's properties.
  `live_write_tests::a_snapshot_set_with_a_user_property_creates_each_and_answers_with_no_failure`
  failed:
  `the reply carries no 'monochromatic:request'`,
  so the arm guards the `props` member.
- `red-2-clone-props-ignored.patch`:
  `ZfsDevice::clone` passes `None`.
  `live_write_tests::a_creation_time_user_property_is_accepted_and_reads_back`
  failed with `the reply carries no 'monochromatic:desired-state'`.
- `red-3-refused-property-value-at-the-limit-minus-one.patch`:
  the refused arm sends one byte less than `ZAP_MAXVALUELEN`.
  `live_write_tests::a_creation_time_user_property_past_the_value_limit_is_refused_and_creates_nothing`
  failed because the kernel accepted the clone:
  `the kernel refuses a user property value past its limit: Some(NvList ...)`.
- The suite summary was
  `198 tests run: 195 passed, 3 failed, 9 skipped`,
  and those three were the only failures.

`red-b-guest-logs/` (guest `zfs-step-2-findings-red-account`)
ran two patches:

- `red-4-checkout-snapshot-grant-removed.patch`:
  the fixture no longer grants local `snapshot` on the checkout.
  `live_account_tests::the_account_snapshots_the_checkout_and_the_reply_carries_no_failure`
  failed with
  `Request { source: Call { operation: Snapshot, target: "zfsstep2-account", errno: 1 } }`.
- the clone hunk of `red-2-clone-props-ignored.patch`:
  `ZfsDevice::clone` passes `None`.
  `live_account_tests::the_account_clones_with_a_creation_time_user_property`
  failed with `the reply carries no 'monochromatic:desired-state'`.
- The root suite also hits the grants test and both creation-time property
  arms when these two hunks are in place,
  so the copy of the recipe the red run used narrows its root `--filterset` to
  leave those three out
  (`red-5-root-suite-filter-narrowed-for-the-account-red.patch`).
  The account suite itself ran unchanged.
- The summaries were
  `195 tests run: 195 passed, 12 skipped` for the root suite and
  `9 tests run: 7 passed, 2 failed, 198 skipped` for the account suite.

Two attempts before these are kept as well,
because each failed for a reason outside the arms:

- `red-a-aborted-attempt-logs/` holds the destroy logs of a red-A guest stopped
  after 10 minutes of an OpenZFS build.
  The cause was not established:
  the kept folder holds only the destroy output,
  nothing records where that attempt's qemu process ran,
  and a later run under the same caps
  (`MemoryMax=2G`,
  `CPUQuota=100%`)
  kept qemu outside its unit and finished the whole recipe,
  so the caps as such do not explain it.
  The caps were raised for every later run regardless.
- `red-b-aborted-attempt-logs/` holds a run that started the recipe from the
  worktree while passing the red tree only as `--crate`.
  The recipe is the worktree's,
  so its root filter was the shipped one and the
  root suite stopped before the account episode;
  the red copy's own recipe was then run instead.

## What this round did not do

- It did not rebuild or retag the gate image,
  and did not touch the fuzz sidecar's lockfiles.
- It never used the `mvm` `hetzner` backend,
  which rents an outside server (the design forbids it).
- It did not run `dprint check` or an oxlint sweep:
  the brief names neither,
  and the package has no task that covers its two `.ts` files.
- The main checkout's `file-enforcer check` drift (check 9) is recorded here but
  not repaired:
  that file belongs to whichever session is working in the main checkout.
