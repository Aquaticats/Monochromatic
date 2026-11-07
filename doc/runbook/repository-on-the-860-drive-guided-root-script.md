# Guided root script for moving the repository to the 860 drive

Written 2026-11-07T13:35:00-05:00.
This runbook is the manual for `doc/runbook/repository-on-the-860-drive-guided-root-script.sh`,
a guided bash script that the owner runs.
It carries out the steps of `doc/planning/repository-on-the-860-drive.md`
(sections "The copy", "The symlink design", "Fixture test before the real run", "Verification of the real copy",
"Cut-over" and "Rollback") in the plan's order.
The plan owns the design and the reasons;
this file says how to run the script and what each stage proves.

Owner words that bind this work, verbatim:

- Option B, a symbolic link at the old path ("take care of dangling links"), for all worktrees.
- 2026-11-07T12:46:40-05:00: "No need to wait for me. Do it whenever you see fit."
- "never discard user data"
- "nothing should ever wait forever or indefinitely"
- "Commits should never fail."
- "I didn't mean to let you make your own choices, especially when the choices could be potentially destructive."

The go-ahead covers timing only.
Every root step is still run by the owner, because agents never run `sudo`.

## What this proves

The script moves the main checkout, its worktree subvolumes and the external worktrees to the 860 with extent sharing
kept, verifies the copy against a frozen baseline, and only then points the old paths at the copy.
It never deletes anything:
where a deletion is needed it prints the command for the owner.
Each stage shows its exact command, runs it only after the owner answers **y**, checks the result,
and stops at the first failure with what completed and how to resume.
A **n** answer also stops the script;
it never skips a stage, because a skipped root stage breaks every stage after it.

The repository rule SCR forbids bash scripts.
The request for this work asked for the form the `wizard` skill prescribes, which is bash,
so the script follows the request.
The library part of the script (the part the skill says never to edit) is the skill's `template.sh`, byte for byte.

### Bridges tried before handing this to the owner

- `sudo`: agents never run it, so no root command was run, including the fixture on loop devices.
- User-level fixtures: the plan already used a user namespace with tmpfs and the owner's own scratch subvolumes;
  they cannot exercise `btrfs send`, `btrfs receive`, `chattr +i` on a mountpoint, or loop mounts.
- A pseudo-terminal driver to type the password: rejected, because the script asks for the password itself
  (`sudo --validate`) and a driver would hold the credential.
- Nested compositor or browser: not relevant, nothing here has a graphical step.

What was run without root is listed in the section "What was verified without root".

## Setup

Status:
TODO

1.  Pick the moment for the cut-over.
    The `prepare` command needs no quiet moment;
    the `cutover` command needs the coordinator to have said that no agent works in any worktree.
2.  Copy the script out of the repository, because bash holds the running script open
    and the holder scan reports every process with a file open inside the repository:

    ```bash
    # /var/home/user/repository-move-wizard.sh: a copy outside the repository.
    cp /var/home/user/Monochromatic/doc/runbook/repository-on-the-860-drive-guided-root-script.sh /var/home/user/repository-move-wizard.sh
    chmod +x /var/home/user/repository-move-wizard.sh
    ```

    Expected: the copy exists and is executable.
    The script refuses to run from inside the repository.
3.  Open a terminal whose current folder is not inside the repository
    (`cd /var/home/user`), and a normal user shell, not root.
    The script refuses to run as root or from a folder inside the repository.
4.  Check that the two scratch scripts the plan names exist:
    `/var/home/user/temp/agent/dangling/move-repository.ts` and
    `/var/home/user/temp/agent/dangling/list-uncommitted-work.ts`.
    The script checks this itself and stops with the missing path.
    They sit under `~/temp`, which is a link onto the 860, so they are on the drive that receives the copy.
5.  Optional dry run, which shows every command and runs nothing, records nothing and asks nothing:

    ```bash
    WIZARD_DRY_RUN=1 /var/home/user/repository-move-wizard.sh prepare
    ```

    Expected: every stage prints `dry run: this step is shown, not executed, and not recorded`.

## Steps

Status:
TODO

The script has five commands:
`prepare`, `cutover`, `thaw`, `rollback` and `status`.
Press **Enter** at the pause after each stage;
**Ctrl-C** is safe at any prompt and between stages.
Rerun the same command to resume:
completed stages are skipped (the progress files are under `/var/home/user/repository-move-state/wizard/done/`),
and the stage that failed is offered again.
`--redo=<stage id>` offers a completed stage again.

### The prepare command

Run `/var/home/user/repository-move-wizard.sh prepare`.
The repository stays writable.
Nothing is frozen and no commit is refused.

1.  Root login, `sudo --validate`.
    Check: `sudo --non-interactive true` succeeds.
    A background loop then renews the login so no stage stalls on a prompt inside a timeout; it ends with the script.
    Rollback: nothing changed.
2.  Fixture F1a to F8 of the plan, on two 3G loop images in a fresh folder under `/var/home/user/temp/agent`.
    Stages: folder, loop images and mounts, the source tree (a subvolume with two worktree subvolumes sharing a reflink),
    print mode of the copy script against it, the first pass, a change, the final pass with the freeze,
    the comparison, the deadline case, the leftover-receive case, the corrupted-stream case,
    the staged verification in a private mount namespace, and the mount over an immutable folder.
    Checks, each with its exact string: `inventory agrees: 2 nested subvolumes`, `pass first finished`,
    `pass final finished`, a `received` and a `placed` record for `f000`, `f001`, `f002`,
    empty `rsync` output, `ro=true`, a received UUID equal to the sent UUID,
    a copy footprint within a quarter margin of the source (sharing kept),
    `exists without a completion record`, `receive refused the corrupted stream`,
    and an `i` flag on the folder after unmount but not while mounted.
    Rollback: the images are throwaway;
    each stage names its own undo, and the last fixture stage prints the removals for the owner to run.
3.  Inventory R0: `sudo btrfs subvolume list /var/home` into `inventory-nested.txt` and a scan for mounts inside the tree.
    Check: the list names the checkout and the output says `no mount inside the tree`.
4.  Reconcile: the copy script in print mode reads the inventory and compares it with Git's registered worktrees.
    Check: `inventory agrees: N nested subvolumes`.
    On a difference the script stops and names every subvolume Git does not know and every registered one that is not a
    subvolume; each needs a written disposition before the run continues.
5.  Pre-flight space check, three stages:
    the footprint of the checkout (`btrfs filesystem du --summarize --raw` as root),
    the size of each external worktree (`du`),
    and the rule from the plan's section "Capacity policy":
    the 860's free space after the estimate plus a quarter margin must stay at least 150 GiB,
    and the NVMe at least 100 GiB.
    Check: `pre-flight passed`.
    A `REFUSED` line is the rule working;
    free space or the estimate changed since the plan was written.
6.  Folders: the snapshot folder on the NVMe, and on the 860 the repository folder (0700, owned by the user),
    the received folder and the external-worktrees folder.
    Check: all four are `user:user` mode 700.
    Rollback: `rmdir` of the empty folders, for the owner to run.
7.  First pass in print mode: the copy script prints every step to `first-pass-print.txt`.
    The owner reads it.
    The stage shows the first and last lines and the counts of snapshots and sends.
8.  First pass for real, after the owner confirms the print was read.
    The tree stays writable.
    Each transfer has the copy script's own deadline (14400 s) and its own systemd scope.
    The copy script has no total limit, only that per-transfer one;
    **Ctrl-C** is safe and a rerun resumes from the state records.
    Check: `pass first finished` and one `received` record per row of the manifest.
    The script adds the elapsed time to `first-pass-seconds`, which feeds the window maximum.
    Rollback: the NVMe tree only gained snapshots;
    a partial receive is named by the copy script with the exact `btrfs subvolume delete` for the owner.
9.  External worktrees, pass one: a new subvolume per folder under `~/worktrees` and a root `rsync` into it.
    Check: every folder has a copy.
10. Size table of every subvolume, for the verification of the six largest and six smallest.

Then the script prints a clear stop and exits.
It does not start the cut-over.

### The cutover command

Run `/var/home/user/repository-move-wizard.sh cutover` only when the coordinator says no agent works in a worktree.
It refuses unless the first pass, the external pass one and the size table are recorded as done.

Before the window (nothing is refused yet):

1.  Root login.
2.  Pre-window checks: the checkout is a real writable folder (`ro=false`), the 860 is mounted,
    and no leftover `f###` objects or final folder exist.
3.  Table of every worktree before the freeze, timed (`scan-seconds`).
    Check: no row is missing, failing, mid-operation or holding `index.lock`.
4.  Window maximum, from measured numbers (section "The maximum duration of the window").
5.  Holder scan as root: any process with a working folder or open file in the repository, the external worktrees,
    or a worktree elsewhere.
    Check: `holders-before.txt` is empty.
    Nothing is ever signalled.
6.  Wrapper state: nothing in `busy/` or `push-retries/` for any commit service instance.
7.  R0 again, because worktrees come and go.
    The copy script's print mode reconciles it with Git and prints the freeze lines,
    which become `freeze-commands.txt` and the matching `thaw-commands.txt`.

The window, which the script times from the first stage that stops the units:

8.  Stop the user units (`cli-git@*` and `claude-cpu-watch.service`).
    The window clock starts here and commits fail from here.
    The script asks whether the coordinator has said no agent is working and whether commits may be refused.
9.  Freeze: `btrfs property set -ts <path> ro true` on the checkout and every nested subvolume,
    and a read-only bind mount over each external worktree.
    Check: every path reads `ro=true` and every external folder is a read-only mount.
10. Holder scan again.
    A holder now has a descriptor the freeze cannot stop.
    Check: `holders-frozen.txt` is empty; otherwise run `thaw`.
11. Baseline of the frozen tree: the table of every worktree, the hash of all refs and the worktree list.
12. Final pass: `move-repository.ts --run --pass=final`, deadline per transfer the smaller of 14400 s and the window's
    remainder.
    Check: `pass final finished`, a `placed` record for every final snapshot, the placed checkout is `ro=false`.
13. External worktrees, pass two: the same `rsync` from the read-only mounts.
14. Verification 1 to 3 in a private mount namespace with the copy bound over the real paths:
    `git fsck --full`, the table equals the baseline byte for byte, the refs hash and the worktree list are equal.
    Git and the scan run as the user, never as root.
15. Verification 4: choose the subvolumes to compare by content (the checkout, every worktree with a non-clean
    baseline row, the six largest and six smallest), then `rsync --checksum --dry-run` of each source snapshot against
    its placed copy must print nothing.
16. Verification 5: the copy's footprint is within a quarter margin of the source's (sharing kept).
17. Verification 6: file counts equal for every subvolume.
18. Verification 7: counts of root-owned `*.log.jsonl` files are equal on source and copy.
19. Verification 8: each external worktree against its copy, with `--delete` in dry-run so an extra file in the copy is
    reported.
20. Mountpoint guard (plan R1 and R2): in a private mount namespace the script binds the NVMe `/var`,
    requires zero entries under the hidden mountpoint, then sets root ownership, mode 0755 and `chattr +i` there.
    Check: the `after guard:` line shows the `i` flag and the `mounted drive:` line does not.
21. Swap: rename the old checkout and the old `~/worktrees` aside as `.nvme-before-860-<timestamp>`
    (renames, no copy, no deletion), then `ln --symbolic` the old paths to the 860 copies.
    Check: both paths are links with the exact target text, and `.git` resolves through the checkout link.
22. Verification 9: `git fsck --full`, the table and the worktree list through the real link equal the baseline.
23. Release: start `claude-cpu-watch.service` again, close the window, print the window length.
    After this the fast rollback no longer applies.

Rollback from the stop of the units through the mountpoint guard is `thaw`.
Rollback for the swap and the verification through the link is `rollback`.
The script numbers its stages one higher than this list from the content comparison on,
because it shows the choice of subvolumes and the comparison as two stages.

### The thaw and rollback commands

`thaw` is for a stop before the swap:
it sets `ro=false` on every subvolume the script froze, unmounts the external binds,
and restarts `claude-cpu-watch.service`.
It then renames the cut-over progress and the window clock aside (renames, never deletions)
and prints the leftovers of the aborted attempt that the owner must delete before trying again,
because the copy script refuses to reuse an `f###` snapshot, received copy or final folder.

`rollback` is the fast path of the plan, valid only until the release stage:
it checks the old trees are still frozen, renames the two links aside, renames the old folders back,
and then does what `thaw` does.
After the release a rollback is the plan's full reverse migration, which this script does not run.

`status` lists what has completed in each phase.

## The maximum duration of the window

The plan says the window length "is not known" and that the runbook states the maximum after the first pass is timed.
The script derives it from times it measured itself, and refuses to exceed it:

- the first pass, measured in `prepare` (`first-pass-seconds`):
  the final pass sends only what changed since then, so a full first pass is its upper bound;
- two scans of every worktree, each assumed to take as long as the one measured in `cutover` before the window
  (`scan-seconds`):
  one for the frozen baseline and one for the staged table;
- two footprint measurements of the checkout, each assumed to take as long as the one measured in `prepare`
  (`du-seconds`):
  one for the source and one for the copy.

The maximum is the sum, printed with its parts at the "Window maximum" stage.
The owner may shorten it with `MOVE_WINDOW_MAX_SECONDS`, never lengthen it.
The fsck and the content comparisons have no measured counterpart;
they are absorbed by the slack between the final pass and its upper bound, which is an assumption to watch.

Enforcement:
every window stage checks the clock before it is offered,
runs under `timeout` with the window's remainder,
and the final pass gets that remainder as its per-transfer deadline.
When the clock is exceeded the script runs nothing further and says what to run:
`thaw` before the swap, `rollback` after it.
A timeout cannot end a task stuck in uninterruptible kernel I/O;
the script then reports a failure and treats the stage as not done, and the stuck process is the owner's to find.

## What was verified without root

Run on 2026-11-07 between 13:00:00-05:00 and 13:40:00-05:00, each non-trivial command in a bounded
`systemd-run --user --scope`:

- `bash -n` on the script, and its library part compared byte for byte with the skill's template.
- A full dry run of `prepare` and of `cutover`: every command printed, nothing executed.
- The step engine against stand-in steps:
  a failing stand-in stopped with exit 1, listed what completed, named its rollback and the resume command;
  a rerun skipped the completed stage;
  a check that fails stopped the stage;
  an answer of **n** stopped with exit 2 and ran nothing;
  an exceeded window stopped with exit 3 and ran nothing;
  a stage past its timeout was reported as a failure (the command exited 124) and stopped the script;
  an expected exit code of 1 was accepted and an unexpected 0 was refused.
- The copy script in print mode against the real repository path, 263 subvolumes (the checkout and 262 nested),
  with a stand-in inventory built from the real inode-256 folders:
  it agreed, it agreed with unrelated lines in the list, and it refused with the exact differences when one line was
  added and one removed.
  The real inventory file does not exist yet (it needs root), so this proves the reconciliation and the path mapping
  for the plan's `home/` prefix only against a list built from the same data.
- The final pass in print mode with a stand-in manifest: 263 freeze lines for 263 sends, 262 stub removals.
- The helper computations (`__preflight`, `__window-maximum`, `__select-verify`) with prepared inputs.

Not verified: every real root command, the real `btrfs send` and `receive`, the mount guard, the freeze,
the bind mounts, and the behavior of the commit service and the wrapper while frozen.

## Where the plan no longer matches the repository

Measured 2026-11-07 between 13:00:00-05:00 and 13:30:00-05:00.

- Registered worktrees: 371 (including this task's own worktree), the plan counts 340.
  Nested under `.claude/worktrees`: 277, of which 262 are subvolumes and 15 are plain directories;
  the plan counts 246, of which 242 and 4.
  External: 16 under `~/worktrees`, 71 on the 860, 6 spelled `~/temp/agent`;
  these match the plan.
- Free space: the 860 had between 571 and 576 GiB available (plan: 662), the NVMe between 300 and 303 GiB (plan: 350).
  With the plan's own estimate (263.6 GiB nested plus 74.3 GiB external) the pre-flight rule leaves about 160 GiB on
  the 860 against the required 150 GiB, so a live measurement that comes out a little larger will be refused.
- The built `repository-link-check` of work item W1 does not exist (`package/repository-link` is absent and
  `~/.config/monochromatic-dev/repository-link.json` is absent), so the identity file and its check, which the plan's
  phase 4 records, are not run and the final screen lists them as not done.
- The plan's phase 0 items that are not in the owner's list for this script are not in it:
  the `user_subvol_rm_allowed` remount and the `mount-860.service` edit (the 860 is still mounted without it),
  the work items W1 to W15, the `claude-cpu-watch.service` unit copy, and the harness migration.
- The wrapper prints a notice on every command that `.git` holds both `cli-git*` and `monochromatic-git-policy-cli*`
  files; they move with `.git`, and the plan does not mention them.

Defects in the plan's own commands, found while writing the script:

- `sudo node` fails when node comes from the version manager and is not on sudo's search path;
  the script passes the absolute path of the node binary.
- The plan's F5 reuses the snapshot, received and state folders of F3, so the copy script stops on existing names before
  it reaches the case under test, and its flipped-byte test seeks 100000 bytes into the stream of `s001`,
  which is probably a short clone stream (not measured).
  The script uses fresh folders per case, a deterministic leftover (a subvolume named `s000` with no record),
  and flips a byte in the stream of `s000`.
- The plan's F2 inventory uses `btrfs subvolume list -o`, which the plan itself says is not recursive;
  the script lists the whole fixture filesystem and passes the prefix `main/`.
- `move-repository.ts --freeze` freezes and copies in one run,
  so the plan's order (freeze, scan again, baseline, final pass) is not reachable with it;
  the script takes the freeze lines from print mode and runs the final pass without `--freeze`.
- The first inventory goes stale before the cut-over because worktrees are added and removed,
  and the copy script then refuses; the cut-over takes R0 again.
- Pass two of the external worktrees has no `--delete`, so a file removed after pass one stays in the copy,
  and the plan's check 8 has no `--delete` either; the script keeps the copy step as written
  (it deletes nothing) and adds `--delete` to the dry-run comparison so the extra file is reported for the owner.
- A root `git` in the staged verification can rewrite the index as root; the script runs Git as the user.
- The plan does not state the window maximum; the section "The maximum duration of the window" derives it.

## What to check

Status:
TODO

After `prepare`:

1.  The last screen says `Prepare is finished. Nothing was cut over.`
2.  `/var/home/user/repository-move-wizard.sh status` lists every prepare stage as completed.
3.  `ls /var/home/user/repository-move-state/wizard` shows `first-pass-seconds`, `du-seconds`,
    `subvolume-sizes.txt` and `first-pass-print.txt`.

After `cutover`:

1.  The last screen says `The repository is now at /var/mnt/encrypted/repository/Monochromatic through a link.`
2.  `ls -ld /var/home/user/Monochromatic` prints a line starting with `l` and ending with
    `-> /var/mnt/encrypted/repository/Monochromatic`.
3.  `ls /var/home/user/repository-move-state/wizard/window-end` exists and the release stage printed `window length:`.
4.  `cmp /var/home/user/repository-move-state/wizard/baseline-frozen.tsv /var/home/user/repository-move-state/wizard/through-link.tsv`
    prints nothing.

Still the owner's, as the final screen lists:
the plan's phase 5 checks (a test commit in a throwaway worktree, which the wrapper may push),
`mise trust --all` then `mise trust --show`,
a started `created` container,
a fresh worktree's `pnpm install` then `find node_modules -xtype l` printing nothing,
the probe session for the harness key and the harness migration,
and the removal of the old trees and snapshots after the validation window.

## Restore

Status:
TODO

- Before the swap: `/var/home/user/repository-move-wizard.sh thaw`.
  Expected: every frozen subvolume reads `ro=false`, no external folder is a mount,
  `claude-cpu-watch.service` is active again.
- After the swap, before the release stage: `/var/home/user/repository-move-wizard.sh rollback`.
  Expected: `/var/home/user/Monochromatic` is a real folder again, the links sit beside it as
  `Monochromatic.link-rolled-back-<timestamp>` and `worktrees.link-rolled-back-<timestamp>`, and the thaw checks pass.
- After the release stage: the plan's full reverse migration, section "Rollback".
- The script deletes nothing.
  What it leaves for the owner to delete, with the commands it prints:
  the fixture images and mount folders, a partial received subvolume after an interrupted transfer,
  the aborted attempt's `f###` snapshots, received copies and final folder,
  and later the old trees, the source snapshots and the received first-pass copies.
