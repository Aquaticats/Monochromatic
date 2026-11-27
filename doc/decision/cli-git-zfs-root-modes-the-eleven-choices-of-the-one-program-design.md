# The eleven choices of the root modes design in the one wrapper executable

## Status

Decided by the owner through the question tool,
answered between 2026-11-24T23:00-05:00 and 2026-11-24T23:33:55-05:00.
Recorded by the session.
The proposal these choices belong to is
`doc/planning/cli-git-zfs-root-modes-in-the-one-wrapper-executable-design.md`,
asked at its sixth revision as merged in `a0dad3adb7`;
a revision writes these answers into it,
together with the repairs its sixth read listed,
and the build of the root modes starts from that revision.
The proposal numbers its choices,
and this record keeps those numbers so the two can be read side by side.

## What came before

The owner decided that the ZFS route's root modes join the wrapper's executable
(`doc/decision/cli-git-zfs-one-program-only-the-root-modes-join-the-wrapper.md`)
and that only the ZED hook's file name selects a root mode
(`doc/decision/cli-git-zfs-the-zed-hook-name-selects-a-root-mode-and-oomd-reads-cgroups.md`).
The proposal designed how,
through six revisions,
each read by a Claude Sonnet second pass.
The sixth read,
`read-of-the-sixth-one-program-revision.md`
in `~/temp/agent/gate-review-zfs-root-modes-sixth-revision-2026-11-24/keep/`
(scratch,
deleted on 2026-11-27;
what this record uses from it is stated here),
found no high finding and judged the choices ready after five text repairs that change no command,
no ranking order and no boot behaviour;
the questions carried its corrected blocks:
choice 4's pair of B over C stated as a tie by delivery,
choice 5's pair of A over B given its reason,
and choice 5's line noted to carry choice 2's word.
The two repairs not named here were choice 9's printed usage line,
which named both the kernel's token and the record's token for `--replace`,
and choice 9's stale-record window con for two options,
which rests on what the boot refusal prints rather than on the option itself.

In every command this record quotes,
`<file>` is `/usr/libexec/monochromatic-git-policy-cli/history_event-mount_git_worktree_clone_datasets`,
the root-owned file under the ZED hook's name.

## The answers

### What the hook-named file accepts (choice 1)

Asked what the file under the ZED hook's name accepts beyond ZED's start with no argument.
Ranked A over B over C.

The answer: "A: also --help, --version (Recommended)".

So `<file>` alone runs the clone-mount sweep,
`<file> --help` and `<file> --version` print,
every other word under that name is refused with exit 2 before Git can run,
and `monochromatic-git-policy-cli zfs-root <mode>` selects a mode on any file.

### The word after the program's name (choice 2)

Asked the word every unit line,
adoption command and person's root-mode command spells after `monochromatic-git-policy-cli`.
Ranked `zfs-root` over `clone-mounts` over `mount` over `root`.

The answer: "A: zfs-root (Recommended)".

### What root's install folder holds (choice 3)

Asked what `/usr/libexec/monochromatic-git-policy-cli/` holds,
and so what a stray or mistyped start of it does as root.
Ranked one file under the hook's name,
over one file named `mount-git-worktree-clone-datasets`,
over one file named for the wrapper,
over a copy or link per name.

The answer: "A: one file, hook's name (Recommended)".

So the folder holds one root-owned regular file under the hook's name,
ZED's entry `/etc/zfs/zed.d/<that name>` is a link to it,
and no file there is named for the wrapper or the modes.

### Where the dispatch sits (choice 4)

Asked where in the program's start the root-mode dispatch sits.
Ranked the first statements of `main`,
over after the panic hook and the environment,
over the top of `run_process`,
over inside `plan_invocation`.

The answer: "A: first lines of main (Recommended)".

### What `--version` and the refusals say (choice 5)

Asked the line `<file> --version` prints and the name a root-mode refusal opens with.
Ranked the typed words,
over keeping `mount-git-worktree-clone-datasets`,
over the wrapper's name.

The answer: "C: the typed words (Recommended)".

So `--version` prints `monochromatic-git-policy-cli zfs-root` and the version,
every root-mode refusal opens with `monochromatic-git-policy-cli zfs-root:`,
and the family name `mount-git-worktree-clone-datasets` is no longer printed;
the line carries choice 2's word.

### A short name on the account's `PATH` (choice 6)

Asked whether the account's `PATH` keeps a short name for root modes.
Ranked none over a name the toolchain links.

The answer: "A: none (Recommended)".

So `doctor` prints the root-owned path and the words to type.

### The four test-only variables (choice 7)

Asked whether the release keeps reading the start probe's test switch,
its keep-descriptor switch,
`MONOCHROMATIC_VERBOSE` and `MONOCHROMATIC_WARN`.
Ranked keeping them over compiling them out.

The answer: "A: keep them (Recommended)".

### The record mode's word (choice 8)

Asked the word of the mode that records which dataset is the checkout
(the root command of choice 1,
option E,
in `doc/decision/cli-git-zfs-step-10-reach-plan-text-grants-and-migration-timing.md`).
Ranked `record-checkout` over `record-route` over `checkout-record`.

The answer: "A: record-checkout (Recommended)".

### What replaces a checkout's record after a restore (choice 9)

Asked what root types to replace the recorded checkout after a legitimate restore gives the checkout dataset a new GUID,
while the nested dataset's boot unit refuses and leaves it unmounted.
Ranked `--replace <the kernel's GUID>`,
over `--replace <the record's GUID>`,
over a bare `--replace`,
over the ordinary command replacing,
over deleting the record and recording again.

The answer: "E: --replace <kernel's GUID> (Recommended)".

So `record-checkout --checkout <path>` refuses `checkout-record-differs` while the record differs,
printing both GUIDs and the exact `--replace <the kernel's GUID>` to run;
`--replace <that GUID>` writes the record and prints the old and the new GUID;
a value that is not the kernel's GUID is refused `replace-token-differs`.

### The panic hook a root mode installs (choice 10)

Asked what a root-mode start prints if the program panics,
when a panic message can hold a dataset name or a checkout path and lands in root's journal.
Ranked the wrapper's payload-free hook over Rust's default hook.

The answer: "A: payload-free hook (Recommended)".

### The probe's deadline variable in the release (choice 11)

Asked whether the release honours `CLI_GIT_TEST_ONLY_START_PROBE_DEADLINE_MS`,
which today makes any positive number of milliseconds the start probe's bound.
Ranked compiling it out,
over keeping it and recording a raised bound in the verdict,
over keeping it as it is.

The answer: "A: compile it out (Recommended)".

So the release uses the shipped bound whatever the variable says,
and the guest arms that raise it build the probe with a Cargo feature.

## What follows

- The proposal's next revision writes each answer into its text,
  removes the options not chosen from what the build follows,
  and folds the sixth read's five repairs and its list of repairs after the answers.
- The build of the root modes,
  which the next alpha release waits for,
  starts from that revision.
- Step 19's migration and its `canmount=noauto` change,
  which the session runs with sudo after removal (plan steps 14 to 16) exists,
  use the words and the `record-checkout` mode decided here.
