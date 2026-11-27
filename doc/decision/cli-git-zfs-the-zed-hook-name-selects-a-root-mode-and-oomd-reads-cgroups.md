# The ZED hook's name selects a root mode, and systemd-oomd may read the cgroup tree

## Status

Decided by the owner on 2026-11-24,
through the question tool,
in one round of two questions.
Recorded by the session.

## The start name, for the ZED hook only

### The question

The one-program design
(`doc/planning/cli-git-zfs-root-modes-in-the-one-wrapper-executable-design.md`)
recommended that the wrapper read the name it was started under,
because ZED starts a hook by its file name with no arguments.
The repository says otherwise:
`package/git-policy/cli/SPEC.md` ("the executable does not read the name it was started under"),
the source guards `main_does_not_dispatch_on_its_start_name`
(`package/git-policy/cli/src/native/source_guard/hook_self_invocation_tests.rs`)
and the `argv0-dispatch` scan rule
(`source_guard/no_self_invocation_rules_tests.rs`),
and `doc/planning/cli-git-resident-service.md`,
which rejected a name-based dispatch.
The design's gate review
(`~/temp/agent/gate-review-zfs-root-modes-in-the-one-wrapper-executable-2026-11-24/`
(scratch,
deleted on 2026-11-27;
what this record uses from it is stated here))
asked this first,
and ran the failure the wrapper's own name in root's folder allows:
a start of the copy under a name outside the family reached Git,
which answered `git: 'boot-reconcile' is not a git command. See 'git --help'.`
and exited 1.
The options were to lift the rule for the ZED hook only,
to keep it and recognise ZED's start by ZED's own variables,
or to keep it and ship a hook script.

### The answer

"Lift it for the ZED hook only (Recommended)"

### What follows

Only the zedlet's file name,
`mount-git-worktree-clone-datasets`
(or the name the design settles for the hook),
selects a root mode,
compared as the whole last path component;
the boot and shutdown units,
adoption and a person reach the root modes by words in the wrapper's own namespace,
as `service run` does.
`SPEC.md`'s sentence and the two guards change to allow exactly that one name and nothing else.
The gate review also asked that the root-owned regular file carry the mode's name rather than the wrapper's,
so that a root start under any other name fails with no such file instead of running Git as root;
that is the design's to adopt.

## systemd-oomd reads the cgroup tree

### The question

During the night's gate,
AppArmor's profile for `systemd-oomd` denied it reading the cgroup folders it watches
(224 denials for each folder in the window),
its log repeated "Considered 0 cgroups for killing",
and `oomctl` reported `app.slice`'s swap as 0 B while 47.5 GiB of swap was in use,
with 1,964 MiB of memory available.
The question offered a two-line local rule,
`/sys/fs/cgroup/ r,` and `/sys/fs/cgroup/** r,` in `/etc/apparmor.d/local/systemd-oomd`,
which the profile includes,
or to leave it.

### The answer

"Add the read rule now (Recommended)"

### What was done

The two lines were appended to `/etc/apparmor.d/local/systemd-oomd`,
with a comment naming this answer,
and the profile was reloaded with `apparmor_parser --replace` at 2026-11-24T00:49:55-05:00.
Afterwards `oomctl` listed the account's app cgroups one by one,
where before it listed only `/system.slice` and `app.slice`.
The account of the denial and the rule is in `doc/troubleshooting/cachyos-apparmor-enforcement-workload-tuning.md`.
