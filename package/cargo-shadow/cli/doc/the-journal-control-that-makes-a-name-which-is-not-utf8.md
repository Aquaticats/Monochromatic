# The journal control that makes a name which is not UTF-8

Recorded 2026-11-27T03:17:19-05:00 by the round
`issue-6550-non-utf8-directory-control-runs-on-a-utf8only-build-folder`,
on branch
`fix/cargo-shadow-cli-issue-6550-non-utf8-directory-control-runs-on-a-utf8only-build-folder`,
from `main` `2939b47565`.
Not approved and not final.

## What failed, and why

`journal_tests::a_manifest_path_that_is_not_utf8_is_recovered_exactly` makes a directory named
`odd-\xff-dir` under the fixture its case builds,
and the case's fixture lives in `test-scratch` beside the test executable
(`src/test_scratch.rs`,
three levels above the binary).
Cargo here writes artifacts under `/home/user/.cargo/build/{workspace-path-hash}`
(`/home/user/.cargo/config.toml`),
so that fixture root is
`/home/user/.cargo/build/d0/f9fc13fabb04d0/debug/build/monochromatic-cargo-shadow/test-scratch`
for a debug build of this worktree,
on the dataset `zpcachyos/nosnap/cargo-build`
(`findmnt --target <that folder> --output TARGET,SOURCE,FSTYPE`),
whose `utf8only` is on.
That dataset refuses a name that is not valid UTF-8:

```text
$ python3 -c 'import os; os.mkdir(b"/home/user/.cargo/build/probe-name-refusal-issue-6550-\xff")'
errno 84 Invalid or incomplete multibyte or wide character
$ mkdir -- /home/user/.cargo/build/probe-name-refusal-issue-6550-plain
PLAIN: made (exit 0)
```

The second command is the positive control:
the same folder makes a plain name,
so the probe can show a difference there.
`zfs get utf8only` was not run,
because reading that property opens `/dev/zfs`,
which this machine's rules leave to the coordinating session;
the dataset name comes from the mount table and the refusal from the kernel itself,
which is the fact the control needs.

Red on `main`'s code,
one process for the one test,
on 2026-11-27:

```text
FAIL [   0.017s] (1/1) monochromatic-cargo-shadow journal_tests::a_manifest_path_that_is_not_utf8_is_recovered_exactly
thread 'journal_tests::a_manifest_path_that_is_not_utf8_is_recovered_exactly' panicked at src/journal_tests.rs:1329:30:
a directory whose name is not UTF-8: Os { code: 84, kind: Uncategorized, message: "Invalid or incomplete multibyte or wide character" }
Summary [   0.263s] 1 test run: 0 passed, 1 failed, 892 skipped
```

## The answer

`src/test_scratch/fixture_root_taking_any_name.rs` asks the fixture root whether it takes the name,
with a probe named after this process and made with `File::create_new`.
Where the root takes it,
the control runs as it always did,
in the test's own process.
Where it does not,
the same test binary is started again for that one test
(`--exact <the running thread's name>`)
inside `unshare --user --map-root-user --mount --propagation private`,
and the inner half mounts a tmpfs at the fixture root
after proving from `/proc/self/uid_map` that its id 0 stands for another id outside.
The mount lives in that namespace alone:
nothing on the host is mounted,
and the fixture root stays the run's own rather than `/tmp` or `/dev/shm`.

This is the answer two sibling packages wrote for the same refusal:
`package/git-policy/cli/src/native/testing/fixture_root_taking_any_name.rs` (issue 6448) and
`package/forbidden-strings/cli/tests/support/fixture_root_taking_any_name.rs` (issue 6491).
Both mount with `libc`;
this crate has no `libc` and already depends on `rustix`,
so the mount is `rustix::mount::mount` (the `mount` feature is on for that reason)
and the refusal is `Errno::ILSEQ`.
Unlike the wrapper's copy it sets no `SECURE_NOROOT`:
the only program this control starts is its own test binary.

Where the host refuses both the name and an unprivileged user namespace,
the control panics with this package's own line for a test that did not run:

```text
cargo-shadow tests: not run [<the test>]: the file system at <root> refuses a name that is not UTF-8, and this host lets no unprivileged user namespace to mount a tmpfs there (<what unshare said>); it runs where the file system behind the fixture root takes any name, or where this account may run `unshare --user --map-root-user --mount --propagation private`
```

The `test` task reads that line (`src/cgroup_test_support.rs` holds its prefix),
so such a run reports the control as one that did not run instead of as a defect of the
recovery,
and its degraded run leaves the control out of its totals.

## Green on this host

```text
PASS [   0.072s] (1/1) monochromatic-cargo-shadow journal_tests::a_manifest_path_that_is_not_utf8_is_recovered_exactly
Summary [   0.072s] 1 test run: 1 passed, 892 skipped
the control journal_tests::a_manifest_path_that_is_not_utf8_is_recovered_exactly ran inside a user and mount namespace of its own, with a tmpfs at /home/user/.cargo/build/d0/f9fc13fabb04d0/debug/build/monochromatic-cargo-shadow/test-scratch, because the file system there refused a name that is not UTF-8
```

## The other side of the fact

A root that already takes the name is the arm the gate containers give,
and it was measured here by mounting a tmpfs at the fixture root inside a private namespace
first and running the test binary there:

```text
tmpfs tmpfs
test journal_tests::a_manifest_path_that_is_not_utf8_is_recovered_exactly ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 849 filtered out; finished in 0.01s
```

No `ran inside a user and mount namespace` line was printed,
which is what says the control took the direct route,
and the case finished in 0.01 s rather than starting a child.
The same command for the new control below printed the same shape.

## The release code held the same assumption

`journal_unready_verdict::unready_verdict` returned the live files to make durable as
`Vec<String>`,
and `journal_unready::recover_unready` handed each one to `make_durable` through `Path::new`.
A manifest whose name is not UTF-8 therefore reached the barrier as the text a lossy
conversion spells (`odd-\u{FFFD}-dir`).
The barrier then opened a file that is not the live manifest:
it skipped it where that name is absent,
or synced another file where a directory with the spellable name exists,
and the journal was removed either way,
so the only durable copy of the manifest's text went while the live one was never synced.

Both items carry `PathBuf` now,
so the barrier is asked for the path the journal recorded,
byte for byte.

The control is
`journal_recover_tests::an_unready_journal_of_a_manifest_that_is_not_utf8_is_kept_when_that_manifest_cannot_be_made_durable`.
It arms the manifest's own data barrier (`failpoints::arm_output_file_sync_of`) and also holds
a directory under the spellable name with the same manifest text,
so a recovery that carried the name as text would find a file there and call the barrier done.
Red on the code before the fix:

```text
FAIL [   0.204s] (1/1) monochromatic-cargo-shadow journal_recover_tests::an_unready_journal_of_a_manifest_that_is_not_utf8_is_kept_when_that_manifest_cannot_be_made_durable
thread 'journal_recover_tests::an_unready_journal_of_a_manifest_that_is_not_utf8_is_kept_when_that_manifest_cannot_be_made_durable' panicked at src/journal_recover_tests.rs:236:5:
the live manifest's own barrier was never reached, so the recovery asked for a name this file does not have: []
```

The empty message list is the harm:
the journal was removed with nothing said about it.
Green after:

```text
PASS [   0.153s] (1/1) monochromatic-cargo-shadow journal_recover_tests::an_unready_journal_of_a_manifest_that_is_not_utf8_is_kept_when_that_manifest_cannot_be_made_durable
```

A second shape was measured too,
with no directory under the spellable name:
there the barrier skipped the absent file and failed on the mangled parent directory,
so the journal was kept,
but by a message naming a file that does not exist
(`... odd-<U+FFFD>-dir/Cargo.toml could not be made durable (No such file or directory (os error 2))`)
and never by a barrier on the live manifest.
Both shapes fail the armed-barrier assertion above,
which is why the case asserts that and not only the message.

## Files changed

- `src/test_scratch/fixture_root_taking_any_name.rs` (new):
  the question,
  the namespace re-execution,
  the tmpfs mount,
  and the not-run line.
- `src/test_scratch.rs`:
  `scratch_root` is `pub(crate)` so the question names the folder the fixtures are made in.
- `src/lib.rs`:
  declares the new module.
- `Cargo.toml`:
  `rustix` gains the `mount` feature.
- `src/journal_tests.rs`:
  the existing control begins with the question.
- `src/journal_unready_verdict.rs` and `src/journal_unready.rs`:
  live files are `PathBuf`,
  so the barrier sees the manifest's own bytes.
- `src/journal_recover_tests.rs`:
  the new control,
  and the shared journal helper takes the manifest path its index names.

## The whole suite, this branch beside `main`

Both arms ran on this host on 2026-11-27,
each once,
each inside its own `systemd-run --user --scope --slice=agent-work.slice` with `MemoryMax`,
`CPUQuota`,
`TasksMax`,
`IOWeight=1` and `RuntimeMaxSec`,
with the host's load average near 138 both times.

`main` is `origin/main` `2eb4228f0f`,
the split round for this package merged,
extracted with `git archive` into the round's scratch folder so the repository's own files
stand around the package,
and run with the task's own command (`cargo nextest run --release --no-fail-fast`):

```text
Summary [  68.712s] 872 tests run: 857 passed, 15 failed, 21 skipped
```

This branch,
merged with that `main` (`c86dfea9ba`) and then `d2c105d1cd`,
through the package's task (`mise run //package/cargo-shadow/cli:test`,
which is that same command):

```text
Summary [  66.115s] 873 tests run: 859 passed, 14 failed, 21 skipped
```

Every test that still fails here also fails on `main`,
and each of the fourteen is another issue's:
nine are issue 6549 (the controls read `/proc/<pid>/cgroup` as one cgroup v2 line and this
host's file holds a `1:net_cls:/` line first),
two are issue 6551 (proc-scan fixtures below a home folder of mode 700),
two are issue 6552 (ZFS reports a file's blocks only after its transaction group syncs),
and one is issue 6554 (the unrecorded launcher shim control hitting its own 20 s bound).
The two arms differ in one name:
`main` fails `journal_tests::a_manifest_path_that_is_not_utf8_is_recovered_exactly`,
which is this issue and passes here.
The 6553 control (an unsorted `read_dir` compared with a fixed order) failed in this branch's
pre-merge run and passed in this one,
which is that issue's own record:
it fails in some runs and passes in others.

## Checks

Run on 2026-11-27 in `package/cargo-shadow/cli`,
each once,
each inside its own `systemd-run --user --scope --slice=agent-work.slice` with
`MemoryMax`,
`CPUQuota`,
`TasksMax`,
`IOWeight=1` and `RuntimeMaxSec`,
on a host whose load average was about 138:

- the red runs quoted above,
  on `main`'s code and on the code before the barrier fix;
- `cargo fmt --check`:
  exit 0,
  no output;
- `cargo check --locked --offline --all-targets`:
  exit 0;
- `cargo clippy --locked --offline --all-targets -- -D warnings`:
  exit 0,
  no warning;
- `mise run //package/cargo-shadow/cli:test`:
  exit 100 on the summary line above,
  whose fourteen failures are all `main`'s and another issue's;
- `/home/user/Monochromatic/.cache/toolchain-bin/monochromatic-lint package/cargo-shadow/cli`
  from the worktree root:
  one warning,
  `rust/avoidable-closure`,
  which says the Dylint driver for this toolchain is absent and that nothing in this package
  was checked by that rule,
  and no finding on any line of this change.

What was not run,
and why:

- `zfs get utf8only`:
  see the first section,
  reading a ZFS property opens `/dev/zfs`.
- The tests that need a delegated control group outside this host:
  unchanged here.
- Windows and macOS:
  out of scope on this machine,
  and the crate refuses to build off Linux.
