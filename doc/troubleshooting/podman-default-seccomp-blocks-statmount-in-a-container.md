# Podman 6.2's default seccomp profile answers `ENOSYS` for `statmount(2)` and `listmount(2)`, so a containerized test reads a kernel that is not there

## Symptom

The whole container gate on `main` at `8519c599d0`
(run `native-bNr3O6`,
2026-11-24)
failed one test of the wrapper's standing-mount sweep,
in the half it runs inside `unshare`:

```text
test worktree_clone_mount_handler::sweep_standing_mount_source::tests::the_answer_reads_its_strings_at_the_offsets_it_names ... FAILED
thread '...' panicked at src/native/worktree_clone_mount_handler/sweep_standing_mount_source_tests.rs:118:5:
the call answers on this kernel: Function not implemented (os error 38)
```

The host runs Linux `7.3.5-2-cachyos`,
which has `statmount(2)` since 6.8 and a mount's source (`STATMOUNT_SB_SOURCE`) since 6.13,
so `Function not implemented` names something other than the kernel.

The same process answers one way on the host and another way in a container.
A four-line perl probe of the two calls,
with `getpid` as the control that the syscall path itself works,
prints:

```text
$ perl probe.pl                       # on the host
getpid ret=2313629 errno=0
statmount ret=-1 errno=22 msg=Invalid argument
listmount ret=-1 errno=14 msg=Bad address

$ podman run --rm --cpus=1 --memory=512m --pids-limit=64 --network=none \
    --volume "$PWD/probe.pl:/probe.pl:ro" localhost/git-policy-native-test-base:e3fcdbe008bd-c04fca4bdd85 \
    perl /probe.pl
getpid ret=1 errno=0
statmount ret=-1 errno=38 msg=Function not implemented
listmount ret=-1 errno=38 msg=Function not implemented
```

Every container the gate starts answers this way,
so any test that asks the kernel about a mount,
and any product code path that asks the same thing inside a container,
sees `ENOSYS` on this host.

The failure read as a kernel-version problem first,
and the sweep's other control accepted it as one:
the test beside this one passed in the same container because its answer for "no answer" accepted any
reason whose text named `statmount` or 6.13
(`package/git-policy/cli/src/native/worktree_clone_mount_handler/sweep_standing_mount_source_tests.rs:79` at
`8519c599d0`):

```rust
MountSourceAnswer::NoAnswer(why) => {
    // A kernel before 6.13 reports no source at all, and the sweep then reads the live table; the string test below
    // is what proves the layout on a kernel that does answer.
    assert!(why.contains("6.13") || why.contains("statmount"), "the answer says why it could not name the source: {why}");
}
```

That condition is satisfied by a refused call as well,
which is why one control in the file reported `ok` while the other failed,
and why the refusal went unnoticed until a control demanded the call.

## Root cause

Podman installs a seccomp filter in every container it starts,
and by default the filter is the containers-common profile at `/usr/share/containers/seccomp.json`
(`/usr/share/containers/containers.conf:299-302`):

```text
# Path to the seccomp.json profile which is used as the default seccomp profile
# for the runtime.
#
#seccomp_profile = "/usr/share/containers/seccomp.json"
```

`podman info` reports the same file as the one it applies:

```text
$ podman info --format '{{json .Host.Security}}'
{"apparmorEnabled":false,"capabilities":"CAP_CHOWN,...","rootless":true,"seccompEnabled":true,"seccompProfilePath":"/usr/share/containers/seccomp.json","selinuxEnabled":false}
```

The profile's decision for a call it does not list is an error with errno 38
(`/usr/share/containers/seccomp.json:2-4`):

```json
	"defaultAction": "SCMP_ACT_ERRNO",
	"defaultErrnoRet": 38,
	"defaultErrno": "ENOSYS",
```

That errno is exactly the one a system call the kernel does not implement answers with,
so a refused call and an absent call are the same number to the caller.

The single `SCMP_ACT_ALLOW` rule lists the mount family but neither of the two calls that read a mount back
(`/usr/share/containers/seccomp.json:271-273`):

```json
				"mount",
				"mount_setattr",
				"move_mount",
```

and the file names neither call anywhere
(`grep -c 'statmount\|listmount' /usr/share/containers/seccomp.json` prints `0`).
The same is true upstream:
the default profile moved into Go source in the migrated repository
[`common/pkg/seccomp/default_linux.go`](https://github.com/containers/container-libs/blob/2306ffc968379ac7cc33ceaaf2bd3fd06f210f48/common/pkg/seccomp/default_linux.go#L270-L272),
where the allow list holds `"mount"`,
`"mount_setattr"` and `"move_mount"`,
and where the 1001-line file names neither `statmount` nor `listmount`.

The gate passed no seccomp profile of its own.
Its suite container's arguments at `8519c599d0` were the bounds,
this array,
and the image
(`package/git-policy/cli/bin/test-native-container.ts:283-291` at that commit):

```ts
const limits = [
  '--rm',
  '--init',
  '--network=none',
  ...runLimitArguments({
    limits: containerLimits,
    withPidsLimit: true,
  }),
];
```

and the only `--security-opt` in the file is the Clippy container's `label=disable`,
which names AppArmor labels and not seccomp.

So the chain is:
podman starts the gate's suite container with the default profile,
that profile's rule for an unlisted call answers `ENOSYS`,
`statmount(2)` is unlisted,
and the sweep reads a refusal that looks exactly like the kernel the fallback exists for.

### Why the fallback is right and the container is wrong

The sweep's fallback is correct and stays:
a Linux before 6.13 really does report no source,
and the sweep reads the live mount table there
(`doc/decision/cli-git-zfs-step-8-sweep-choices.md`,
"A standing mount's source is read by its mount number").
What is wrong is that the gate's container never lets the kernel answer,
so the gate never exercises the path the product takes on the host it is released for,
and a control that demanded the call failed with no way to tell which of the two it met.

## Verification

Versions under test:
`podman version 6.2.0`,
`container-libs 0.70.0` (the package that owns `/usr/share/containers/seccomp.json` on this host,
`/usr/share/containers/container-libs.version`),
kernel `7.3.5-2-cachyos`,
rootless podman on a ZFS host,
and upstream [`containers/container-libs`](https://github.com/containers/container-libs/tree/2306ffc968379ac7cc33ceaaf2bd3fd06f210f48)
at `2306ffc968379ac7cc33ceaaf2bd3fd06f210f48`.

The harness is the probe above plus the gate's own test binary,
built once on the host and started inside the gate's base image,
which is the same image the gate's suite container starts
(`localhost/git-policy-native-test-base:e3fcdbe008bd-c04fca4bdd85`):

```text
$ cargo test --locked --offline --lib --no-run
$ BIN=/home/user/.cargo/build/cf/fb5e846135e9ad/debug/build/monochromatic-git-policy-cli/0516cc94cb03a25b/out/git_policy_cli-0516cc94cb03a25b
$ TEST=worktree_clone_mount_handler::sweep_standing_mount_source::tests::the_answer_reads_its_strings_at_the_offsets_it_names
$ podman run --rm --cpus=2 --memory=2g --pids-limit=256 --network=none \
    --tmpfs /gate-fixtures:rw,size=256m,mode=1777 --env CLI_GIT_TEST_ROOT=/gate-fixtures \
    --volume "$BIN:/testbin:ro" localhost/git-policy-native-test-base:e3fcdbe008bd-c04fca4bdd85 \
    /testbin --exact "$TEST" --nocapture
```

### Patterns that fail

Podman's default profile,
which is what a container gets with no `--security-opt`:

```text
test ...the_answer_reads_its_strings_at_the_offsets_it_names ... FAILED
thread '...' panicked at src/native/worktree_clone_mount_handler/sweep_standing_mount_source_tests.rs:118:5:
the call answers on this kernel: Function not implemented (os error 38)
test result: FAILED. 0 passed; 1 failed; ...
```

The whole module under the same profile,
after the controls were made to name what they could not do:

```text
STATMOUNT CONTROL NOT RUN: ...the_answer_reads_its_strings_at_the_offsets_it_names (this run does not state that its
containers allow statmount(2), and the call answered nothing: the call answered Function not implemented (os error 38))
STATMOUNT CONTROL NOT RUN: ...the_kernel_names_the_source_of_one_mount_by_its_unique_number (...)
STATMOUNT CONTROL NOT RUN: ...a_call_the_filter_refuses_is_answered_from_the_live_table (...)
test result: ok. 14 passed; 0 failed; ...
```

The same run with the allowance stated and no profile passed fails three tests,
each naming the variable and the call's error,
which is the shape a mis-wired gate now takes.

### Patterns that work

The derived profile,
passed the way the gate now passes it:

```text
$ podman run --rm --cpus=2 --memory=2g --pids-limit=256 --network=none \
    --security-opt "seccomp=$PWD/evidence/gate-container-seccomp.json" \
    --env CLI_GIT_STATMOUNT_ALLOWED_IN_CONTAINERS=1 \
    --tmpfs /gate-fixtures:rw,size=256m,mode=1777 --env CLI_GIT_TEST_ROOT=/gate-fixtures \
    --volume "$BIN:/testbin:ro" localhost/git-policy-native-test-base:e3fcdbe008bd-c04fca4bdd85 \
    /testbin sweep_standing_mount --nocapture
test result: ok. 14 passed; 0 failed; ...
```

No `STATMOUNT CONTROL NOT RUN` line is in that log,
so all three controls read the kernel,
and the perl probe in the same container answers `errno=22` and `errno=14` as it does on the host.

## Verified workarounds

### 1. The gate derives the host's own profile with the two calls added, and states the allowance

`package/git-policy/cli/bin/native-gate-syscall-profile.ts` asks `podman info` for the profile podman applies,
reads it,
adds `statmount` and `listmount` to the unconditional allow rule that names `mount`,
writes the result beside the run's evidence,
and returns:

```text
--security-opt seccomp=<run evidence>/gate-container-seccomp.json
--env CLI_GIT_STATMOUNT_ALLOWED_IN_CONTAINERS=1
```

`package/git-policy/cli/bin/test-native-container.ts` spreads those arguments into the `limits` array it already gives
both of its containers and records in the manifest.

The derivation against this host adds exactly two lines and touches nothing else:

```diff
--- /usr/share/containers/seccomp.json
+++ <run evidence>/gate-container-seccomp.json
@@
 				"linkat",
+				"listmount",
 				"listen",
@@
 				"statfs",
 				"statfs64",
+				"statmount",
 				"statx",
```

Tradeoffs:
- The gate needs the host's profile to be readable.
  A host whose podman applies a filter and reports no profile file now refuses the gate with a message naming both
  facts,
  where it used to fail a control with `Function not implemented`.
- The derived file lives in the run's evidence directory,
  so the profile a run used is auditable beside its log and goes when the evidence retention takes the run.
- A host whose containers-common already allows the calls is given its own file unchanged,
  and the run's line says so.
- The profile is a host artifact the gate copies,
not a policy this repository owns;
  a host with a hand-written profile gets that profile plus the two calls,
  which is what its containers would apply anyway.

### 2. A copy of the profile committed to the repository

A checked-in `seccomp.json` with the two names added is reproducible and needs no host read.

Tradeoffs:
- It freezes every other rule of the profile at the commit's date,
  while the host's podman keeps moving (a newer syscall a newer glibc needs,
  the capability-narrowed rules,
  the argument checks the profile gains),
  so the gate would run under a policy nobody else runs.
- Upstream moved the default profile into Go source,
  so the copy is already one reorganisation behind.

Not taken.

### 3. `--security-opt seccomp=unconfined`

It removes the filter,
so both calls answer.

Tradeoffs:
- Every other rule the profile applies goes too,
  including the capability-narrowed allows (`perf_event_open` without `CAP_PERFMON`,
  `bpf` without `CAP_BPF`)
  and the deny rules with argument checks,
  so the gate would run its suite in a container that is less like the product's environment than the one with the
  derived profile.
- It also hides the very denial the fallback control needs.

Not taken.

## What does not work

- `--cap-add`,
in any combination:
  a capability cannot add an allow entry to a profile.
  The profile's answer for a call it does not name is the default action,
  and no capability changes that.
- Running the test under `unshare`:
  seccomp filters are inherited across `fork`,
`execve` and namespace creation,
  so the inner half of a control meets the same filter as its parent.
- Waiting for the kernel to lack the call:
  the host's 7.3 kernel has it,
  and a container cannot make its kernel older.
- Deciding from the kernel release what `ENOSYS` means:
  a release says what the kernel could answer,
  not what the filter did,
  and a backported call on an older release is the counter-example.
  The run's own statement about its containers is the only signal that separates the two.
- Reading the gate's own `--help` or `podman run` for an additive seccomp option:
  podman 6.2 takes a whole profile (`seccomp=<file>`) or none (`seccomp=unconfined`),
  and has no option that adds one call to the default.

## Upstream filing decision

Nothing is filed.

The default profile belongs to `containers/container-libs`,
not to `containers/common`,
which now auto-closes every new issue with a pointer to the new repository
([`.github/workflows/auto-close-issues.yml`](https://github.com/containers/common/blob/a5ccdae846b629b5ceaefa6ffd5c6511409c3487/.github/workflows/auto-close-issues.yml#L1-L16)):

```text
on:
  issues:
    types: [opened]
...
      run: gh issue close --repo "$REPO" --comment "This repository has been migrated to https://github.com/containers/container-libs. Please open your issue there." "$ISSUE"
```

`.out-of-scope/` holds no exemption for container seccomp profiles or for podman
(`.out-of-scope/cargo-workspace.md` mentions a Fedora Podman build boundary and is unrelated).

Searches,
run on 2026-11-24 against `containers/container-libs` at
`2306ffc968379ac7cc33ceaaf2bd3fd06f210f48`
and `containers/common` at `a5ccdae846b629b5ceaefa6ffd5c6511409c3487`:

- `gh issue list --repo containers/container-libs --state all --search statmount` and the same for `listmount`:
  no result.
- `gh pr list --repo containers/container-libs --state all --search statmount`:
  no result.
- `gh search issues --repo containers/common statmount`,
open and closed,
and `gh search prs` for `statmount` and
  `listmount`:
  no result.
- Recent commits on `common/pkg/seccomp/default_linux.go`:
  `seccomp: restrict socketpair to AF_UNIX and AF_TIPC` (2026-11-02),
  `seccomp: mask upper bits in socket deny rules` (2026-10-11),
  `seccomp: riscv: allow riscv_hwprobe` (2026-09-19),
  `seccomp: allow all futex_* syscalls` (2026-03-03).
  Maintainers are adding single calls to this list now,
  and no open work touches the two calls this report needs.

1.  Upstream's file is where the two names are missing,
    and the missing names are why a container refuses the calls.
    This is behavior,
    not wording,
    and not an architectural limit.
2.  Upstream can fix it:
    two names go into the allow list,
    which is the change `seccomp: riscv: allow riscv_hwprobe` already shows the shape of.
3.  The profile already allows the mount family the calls belong to
    (`mount`,
`mount_setattr`,
`move_mount`),
    so a container that may mount is meant to work;
    reading a mount back is the same use case.
    Nothing in the repository's docs asks for `statmount` by name,
    so this constraint is a soft yes.
4.  A contributed issue does not meet the project's own policy for this artefact.
    The organization's policy bans LLM-generated text verbatim in issues and pull requests
    ([`LLM_POLICY.md`](https://github.com/podman-container-tools/community/blob/8440423ff88e75f222f47c987b49416862ba626a/LLM_POLICY.md?plain=1#L6-L9)):

    ```text
    ## No LLM-Generated Direct Communication

    LLM output cannot be used verbatim in:
    - Issues, comments, or pull request content
    ```

    An issue body drafted here is LLM output,
    so it may not be posted as it stands;
    the person posting has to write it in their own words.
    Code contributions are allowed with that person responsible for them.
5.  Upstream is not leaning no.
    The maintainers add calls to this list as recently as 2026-11-02,
    and no document states a non-goal for the mount family.
6.  The minimal fix is prototyped and verified against the behaviour it changes:
    the derived profile in "Verified workarounds" adds the two names to the same allow list,
    the failing control passes under it,
    and the perl probe answers as it does on the host.
    The Go form of the same change,
    `"listmount",` between lines 247 and 248 and `"statmount",` between 424 and 426 of
    `common/pkg/seccomp/default_linux.go`,
    was not built or tested here,
    because that repository's own build and test suite were not run in this round.

Readiness before anyone files:

1.  Reproduced on this host's packaged profile (`container-libs 0.70.0`) and read on upstream's default branch at
    `2306ffc968379ac7cc33ceaaf2bd3fd06f210f48`.
    Older maintained branches were not read.
2.  Prior art read:
    no issue,
    no pull request and no maintainer statement about these two calls exists in either repository,
    and the recent commits above are the closest work.
3.  Intake read:
    the organization's contributing guide,
    which asks for a reproducer,
    a test that fails without the change,
    and a commit message that explains why
    ([`CONTRIBUTING.md`](https://github.com/podman-container-tools/community/blob/8440423ff88e75f222f47c987b49416862ba626a/CONTRIBUTING.md?plain=1#L105-L110)),
    and the organization's LLM policy above.
    `containers/container-libs/CONTRIBUTING.md` only points at that guide.
4.  What automation does to a filing was read for `containers/common`,
    which is the auto-close above,
    and not for `containers/container-libs`,
    whose workflows were not opened.
5.  The duplicate and work-in-progress searches above ran on 2026-11-24.
    A trust or blocklist system was searched for and not found in either repository's tree;
    the live repository was not re-read on a filing day that has not happened.
6.  The draft's reproduction was run as written,
in the shape recorded in "Verification".
7.  No reproduction repository was created.
    That needs the poster's own repository and permission to publish it,
    and nothing in this round's brief authorizes one.
8.  No recommendation to file is made,
    because item 4's automation check and item 7's reproduction repository are missing,
    and because constraint 4 requires the posting person to rewrite the text anyway.

The draft below is kept for the person who files,
and it is not fileable as it stands.

~~~md
Title: default seccomp profile denies statmount(2) and listmount(2), so containers read ENOSYS for a kernel that has them

What happens: with the default profile, statmount(2) and listmount(2) answer ENOSYS (errno 38) in a container even
where the kernel implements both, because neither call is in the allow list and the profile's default action is
SCMP_ACT_ERRNO with errno 38. Callers cannot tell that from an old kernel.

Reproduction (podman 6.2.0, kernel 7.3.5, rootless):
  perl -e 'my $r=pack("LLQQ",24,0,1,0x208); my $b="\0"x1536; print syscall(457,$r,$b,length $b,0)," ",0+$!,"\n"'
On the host this prints -1 22 (EINVAL, the call ran); in a default container it prints -1 38 (the filter).

Where: common/pkg/seccomp/default_linux.go, the SCMP_ACT_ALLOW rule that names mount, mount_setattr and move_mount.
"listmount" belongs between "listen" and "listxattr"; "statmount" belongs between "statfs64" and "statx".

Why it matters: a container that mounts inside its own mount namespace and then asks the kernel to read that mount
back (statmount, listmount) is refused on a kernel that supports both, and the container tools read the refusal as a
kernel without the calls.

Suggested fix: add the two names to that allow list, beside the mount family it already allows.
~~~

## References

- [The default profile's decision for an unlisted call](https://github.com/containers/common/blob/a5ccdae846b629b5ceaefa6ffd5c6511409c3487/pkg/seccomp/seccomp.json#L2-L4),
  in the repository the profile was published from before the migration.
- [The allow list that names the mount family and neither call](https://github.com/containers/common/blob/a5ccdae846b629b5ceaefa6ffd5c6511409c3487/pkg/seccomp/seccomp.json#L271-L273).
- [The same allow list as Go source after the migration](https://github.com/containers/container-libs/blob/2306ffc968379ac7cc33ceaaf2bd3fd06f210f48/common/pkg/seccomp/default_linux.go#L270-L272).
- [The issue auto-close that points new reports at the migrated repository](https://github.com/containers/common/blob/a5ccdae846b629b5ceaefa6ffd5c6511409c3487/.github/workflows/auto-close-issues.yml#L1-L16).
- [The organization's contributing guide](https://github.com/podman-container-tools/community/blob/8440423ff88e75f222f47c987b49416862ba626a/CONTRIBUTING.md?plain=1#L105-L110).
- [The organization's LLM policy](https://github.com/podman-container-tools/community/blob/8440423ff88e75f222f47c987b49416862ba626a/LLM_POLICY.md?plain=1#L6-L9).
