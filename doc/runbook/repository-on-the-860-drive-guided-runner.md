# Guided runner for moving the repository to the 860 drive

Written 2026-11-07T14:55:39-05:00,
replacing the guided bash script of `4980c200ea`.
The coordinator runs `package/repository-move/cli/src/run-repository-move.ts`,
the TypeScript runner that carries out the steps of `doc/planning/repository-on-the-860-drive.md`
(sections "The copy",
"The symlink design",
"Fixture test before the real run",
"Verification of the real copy",
"Cut-over" and "Rollback") in the plan's order.
The plan owns the design and the reasons;
this file says how to run the runner and what each stage proves.

Owner words that bind this work,
verbatim:

- Option B,
  a symbolic link at the old path ("take care of dangling links"),
  for all worktrees.
- 2026-11-07T12:46:40-05:00:
  "No need to wait for me.
  Do it whenever you see fit."
  This covers timing only.
- "Wizard writes TypeScript,
  and you just do the move yourself,
  no need to delegate to me."
  (2026-11-07,
  relayed by the coordinator.)
  So the runner is TypeScript and the coordinator runs it,
  not the owner.
- "never discard user data"
- "nothing should ever wait forever or indefinitely"
- "Commits should never fail."
- "I didn't mean to let you make your own choices,
  especially when the choices could be potentially destructive."

## What this proves

The runner moves the main checkout,
its worktree subvolumes and the external worktrees to the 860 with extent sharing kept,
verifies the copy against a frozen baseline,
and only then points the old paths at the copy.
It never deletes anything:
where a deletion is needed it prints the command for the coordinator.
Each stage shows its command,
runs it,
checks the result,
and stops at the first failure with what completed and how to resume.
Stages that state a fact only a person can know
(that no agent works in any worktree,
that the printed first pass was read,
that the last chance for the fast rollback is accepted)
do not run until the caller passes `--assert=<stage id>`.

Every stage says whether it uses root and which unprivileged route exists,
so root is used only where nothing else works.
The runner never runs `sudo` itself:
a stage that needs root has `sudo` in its own shown command,
and `sudo` is called with the absolute path of node,
because `sudo node` fails when node comes from the version manager
(measured in the plan,
section "The copy").

## Setup

Status:
TODO

1.  Pick the moment for the cut-over.
    The `prepare` command needs no quiet moment;
    the `cutover` command needs no agent working in any worktree.
2.  Open a shell in the coordinator's normal user session,
    not root,
    with a current folder outside the repository (`cd /var/home/user`).
    The runner refuses to start as root or from inside the repository.
3.  The runner runs from the repository's TypeScript source,
    which imports only node's own modules,
    so nothing needs building and nothing outside the repository is needed.
    Run it with the absolute path of the node that `mise` resolves,
    because the runner passes that same path to every root step:

    ```bash
    # Run from a folder outside the repository.
    NODE="$(readlink -f "$(command -v node)")"
    "$NODE" /var/home/user/Monochromatic/package/repository-move/cli/src/run-repository-move.ts status
    ```

4.  Optional dry run,
    which shows every command and runs nothing,
    records nothing and creates no folder:

    ```bash
    "$NODE" /var/home/user/Monochromatic/package/repository-move/cli/src/run-repository-move.ts prepare --dry-run
    ```

    Expected:
    every stage prints `dry run: this stage is shown, not executed, and not recorded`.

## Steps

Status:
TODO

The runner has five commands:
`prepare`,
`cutover`,
`thaw`,
`rollback` and `status`.
Rerun the same command to resume:
completed stages are skipped (the progress files are under `/var/home/user/repository-move-state/wizard/done/`),
and the stage that failed is offered again.
`--redo=<stage id>` offers a completed stage again.

### The prepare command

The repository stays writable.
Nothing is frozen and no commit is refused.
The stages,
with root marked,
in order:

1.  `sudo-prepare` (root):
    `sudo --non-interactive true` must succeed.
2.  `i1-install-link-check`:
    builds the repository link check (work item W1) and copies its bins to
    `~/.local/share/monochromatic-dev/repository-link/`,
    outside the repository,
    so the check never dangles.
    The check afterwards is that the installed file starts with a node shebang.
3.  `i2-record-before`:
    records the identity file in phase `before` while the repository is still a real folder on the NVMe.
4.  The fixture stages `f1a` to `f8` (root where marked),
    the plan's F1 to F8 on two sparse loop images under `~/temp/agent`,
    each sized in `package/repository-move/cli/src/constants.ts` (`fixtureImageGiB`) from the data the stages write:
    the populated data of every subvolume in every copy set the destination keeps,
    plus the free-space guard minimum,
    with the plan's quarter margin,
    so the guard that stops a run below its minimum free space never stops a later fixture stage for want of room.
    Its message names the folder whose filesystem it measured:
    the loop mount,
    send and receive,
    the staged mount namespace and the mount over an immutable folder need root.
    The freeze of the fixture runs as the user.
    Checks,
    each with its exact string:
    `inventory agrees: 2 nested subvolumes`,
    `pass first finished`,
    `freeze finished`,
    `pass final finished`,
    a `received` and a `placed` record for `f000`,
    `f001`,
    `f002`,
    empty `rsync` output,
    `ro=true`,
    a received UUID equal to the sent UUID,
    a copy footprint within a quarter margin of the source (sharing kept),
    `exists without a completion record`,
    `receive refused the corrupted stream`,
    and an `i` flag on the folder after unmount but not while mounted.
5.  `r0-inventory` (root):
    `sudo btrfs subvolume list -u /var/home`,
    which is root-only.
    The copy tool rebuilds each full path from the list's `top level` (parent ID) column,
    because the `path` column is relative to the parent subvolume
    (measured by the coordinator with root on 2026-11-07:
    the checkout's row reads `top level 257 ... path user/Monochromatic`).
6.  `r0-reconcile`:
    the copy tool in print mode compares the list with Git's registered worktrees.
    Check:
    `inventory agrees: N nested subvolumes`.
    On a difference the tool stops and names every subvolume Git does not know and every registered one that is not a
    subvolume;
    each needs a written disposition before the run continues.
7.  `p3a-footprint` (root),
    `p3b-external`,
    `p3c-rule`:
    the space rule of the plan's section "Capacity policy":
    the 860's free space after the estimate plus a quarter margin must stay at least 150 GiB,
    and the NVMe at least 100 GiB.
    Check:
    `pre-flight passed`.
    A `REFUSED` line is the rule working.
8.  `p4a-folders` (root for the first folder only):
    the snapshot folder on the NVMe,
    and on the 860 the repository folder (0700,
    owned by the user),
    the received folder and the external-worktrees folder.
9.  `m1a-unit-edit` and `m1b-mount-option` (root):
    the 860 mount gains `user_subvol_rm_allowed`.
    The first stage writes an edited copy of `/etc/systemd/system/mount-860.service`
    (and keeps the unit as it is,
    both under `wizard/`),
    changing the one `mount -o compress=zstd:3,noatime` line and refusing anything else.
    The second installs it,
    runs `systemctl daemon-reload` and `mount -o remount,user_subvol_rm_allowed /var/mnt/encrypted`.
    Check:
    the unit file,
    `systemctl cat mount-860.service` and `findmnt --output OPTIONS` all name the option.
    Rollback:
    copy `mount-860.service.before` back and reload.
    No unprivileged route exists for a remount or a file under `/etc/systemd/system`.
10. `p4b-print`:
    the copy tool prints every step of the first pass to `first-pass-print.txt`.
11. `p4c-first` (root,
    needs `--assert=p4c-first`):
    the first pass for real.
    The inventory is taken again at the start of the stage.
    The tree stays writable.
    Each transfer has its own deadline (14400 s) and its own systemd scope,
    and the pass has a total deadline (86400 s,
    the author's bound).
    Records are keyed by subvolume UUID plus path:
    a rerun after worktrees were added,
    removed,
    moved or replaced continues each subvolume's own records and prints what changed,
    and never reuses a name.
    Check:
    `pass first finished` and a received record for every manifest row.
12. `p4d-external-first` (root),
    `p4e-sizes` (root),
    `p4f-verify-first` (root):
    a new subvolume per external folder with a root `rsync` into it,
    and the size table for the cut-over's choice of subvolumes to compare.
    The last compares every first-pass copy with the read-only snapshot it was sent from,
    by path,
    type,
    size,
    time and
    mode (`rsync --dry-run`,
    no data read),
    excusing exactly the inventoried nested subvolumes.
    A difference stops the run and names the copy.

The runner then stops.
It does not start the cut-over.

### The cutover command

Run it only when no agent works in a worktree.
It refuses unless the first pass,
the external pass one,
the size table and the mount option are recorded as done.
Stages marked window run inside the window and are refused once its maximum has passed.

1.  `c00` to `c05`,
    before the window:
    pre-window checks,
    the table of every worktree (timed),
    the window maximum,
    the root holder scan,
    the wrapper state,
    and a fresh `btrfs subvolume list -u` with the print mode of the freeze and the final pass.
2.  `c06-stop-units` (needs `--assert=c06-stop-units`):
    stops the user units.
    The window clock starts here and commits fail from here.
3.  `c07-freeze`:
    the freeze pass of the copy tool as the user
    (the owner of a subvolume sets `ro` on it without root,
    measured in the plan),
    which writes a manifest of what it froze and reads each value back,
    then root read-only bind mounts over the external worktrees.
4.  `c08-holders-frozen` (root),
    `c09-baseline`:
    holder scan again,
    then the baseline table of the frozen tree.
5.  `c10-final-pass` (root):
    the final pass.
    The tool refuses unless every subvolume it sends is in the freeze manifest and still reads `ro=true`.
    A worktree with no first-pass copy gets a full send and the tool says so.
    Deadline per transfer the smaller of 14400 s and the window's remainder;
    total deadline the window's remainder.
    For each subvolume with a first-pass copy it first asks btrfs what changed
    (an incremental `btrfs send --no-data` between the two snapshots) and records the answer.
    `c10b-verify-final` (root,
    in the window) then compares the changed final copies,
    and the ones with no first-pass copy,
    with their snapshots by path,
    type,
    size,
    time and mode.
6.  `c11-external-final` (root):
    the second `rsync` of each external worktree.
    It deletes nothing from the copy.
    A dry-run `rsync --delete` then lists every file deleted at the source after pass one in
    `external-<name>.deleted.txt`;
    the copy keeps those files.
7.  `c12` to `c17`:
    verification in a private mount namespace with the copy bound over the real paths (`git fsck --full`,
    the table equals the baseline,
    the refs hash),
    the content comparison of the chosen subvolumes,
    the footprint,
    file counts,
    root-owned files,
    and the external worktrees
    (a `*deleting` line is reported and does not fail the stage;
    any other difference does).
8.  `c18-guard` (root):
    the mountpoint guard on the hidden `/var/mnt/encrypted`.
9.  `c19-swap`:
    renames the old checkout and `~/worktrees` aside as `.nvme-before-860-<timestamp>`
    and makes the two links.
    No root needed:
    the home folder is user-owned.
10. `c20-through-link`,
    then `c20b-record-after`:
    the same checks through the link,
    then the identity file is recorded in phase `after` as the user
    (link path,
    exact link text,
    canonical root,
    860 UUID,
    first commit of `main`,
    and the marker `.git/monochromatic-root-commit`),
    and the installed `repository-link-check` must print `ready (on the 860)`.
11. `c21-release` (needs `--assert=c21-release`):
    starts the stopped commit sockets and `claude-cpu-watch.service` and closes the window.
    After this the fast rollback no longer applies.
12. `c22-verify-final-checksum` (root,
    outside the window):
    the same comparison as `c10b-verify-final` for the same final copies,
    reading every byte.
    The old tree (`.nvme-before-860-<timestamp>`) stays for the rollback until this stage is recorded done;
    the `status` command prints `DO NOT DELETE` until then and says the tree may be deleted by the owner after it.

### The thaw and rollback commands

`thaw` is for a stop before the swap:
it sets `ro=false` on exactly the subvolumes the freeze manifest lists,
as the user,
and does not read the inventory,
so worktrees that changed since the freeze cannot block it.
It unmounts the external binds (root),
restarts the stopped units,
renames the cut-over progress and the window clock aside,
and prints the leftovers the coordinator must delete before another attempt,
because the copy tool refuses to reuse an `f###` snapshot,
received copy or final folder.

`rollback` is the fast path of the plan,
valid only until the release stage:
it checks the old trees are still frozen,
renames the two links aside,
renames the old folders back,
renames the identity file aside and records phase `before` again,
and then does what `thaw` does.
After the release a rollback is the plan's full reverse migration,
which this runner does not run.

`status` lists what has completed in each phase.

## Which stages need root

Root is needed only where the kernel requires it.
Each stage prints its own line `root: yes|no; unprivileged route: ...`;
the stage data test fails when a stage that uses `sudo` is not marked `yes`,
or a stage marked `yes` has no `sudo`.

- No unprivileged route exists for:
  the loop mounts,
  `btrfs send` and `btrfs receive` (the plan's method M2,
  an unprivileged copy plus per-worktree snapshots,
  needs no root but writes every extent again,
  so worktrees would stop sharing space),
  `btrfs subvolume list`,
  `btrfs filesystem du`,
  every bind mount and the mountpoint guard (`chattr +i`),
  the holder scans that must see other users' and containers' processes,
  and the remount and the unit file of the 860 mount option.
- An unprivileged route exists and is used for:
  the freeze and the thaw (the owner of a subvolume sets `ro`),
  the swap and the links (the home folder is user-owned),
  the identity file,
  the installs,
  and every print mode.
- An unprivileged route exists and root is kept to cover files the user cannot read:
  the external `rsync` passes,
  the size table,
  the content and count comparisons.

## The maximum duration of the window

The plan says the window length "is not known" and that the runbook states the maximum after the first pass is timed.
The runner derives it from times it measured itself plus one allowance,
and refuses to exceed it:

- the first pass,
  measured in `prepare` (`first-pass-seconds`):
  the final pass sends only what changed since then,
  so a full first pass is its upper bound;
- two scans of every worktree,
  each assumed to take as long as the one measured in `cutover` before the window
  (`scan-seconds`):
  one for the frozen baseline and one for the staged table;
- two footprint measurements of the checkout,
  each assumed to take as long as the one measured in `prepare`
  (`du-seconds`):
  one for the source and one for the copy;
- a content-check allowance of one more first-pass duration,
  the author's choice with no measurement behind it:
  the fsck,
  the checksum comparisons,
  the file counts and external pass two read the copy about once.

The maximum is the sum,
printed with its parts at the `c02-window-maximum` stage.
`MOVE_WINDOW_MAX_SECONDS` may shorten it and never lengthen it.
The allowance is the weakest part of the bound:
a maximum that is too small makes the cut-over stop in the verification and the coordinator starts over,
so the first real run should be read as a measurement of it.

Enforcement:
every window stage checks the clock before it runs,
runs under `timeout` with the window's remainder,
and the final pass gets that remainder as its total deadline and the smaller of it and 14400 s as its per-transfer deadline.
When the clock is exceeded the runner runs nothing further and says what to run:
`thaw` before the swap,
`rollback` after it.
A transfer that `timeout` ended leaves its systemd scope running until its own deadline:
wait for `systemctl list-units 'run-*.scope'` to show no `btrfs send` before deleting a partial receive.
A timeout cannot end a task stuck in uninterruptible kernel I/O;
the runner then reports a failure and treats the stage as not done,
and the stuck process is the coordinator's to find.

## What was verified without root

Run on 2026-11-07,
each non-trivial command in a bounded `systemd-run --user --scope`,
and no `sudo` command by the preparing agent:

- The copy tool against a btrfs test double (`src/btrfs-test-double.ts`):
  both passes,
  the four defects of the first script as tests
  (records keyed by position,
  freeze joined to the copy,
  no total deadline,
  an external second pass that could not report deletions),
  and the thaw that does not depend on the inventory.
- The inventory parser against the exact rows the coordinator printed with root
  (the `top level` column rebuilt into full paths).
- The repository link check:
  every answer of the plan's contract against an in-memory probe,
  the real probe,
  the three bins in a disposable home,
  the deadline against a probe that never answers,
  and the copy outside the repository running with nothing of the repository reachable.
- The runner's engine against stand-in stages:
  a failing stage stopped with exit 1 and named its rollback;
  a rerun skipped the completed stage;
  a failing check stopped the stage;
  a listed non-zero status was accepted and an unexpected zero refused;
  a missing assertion stopped with exit 2 and ran nothing;
  a stage past its timeout was reported as a failure;
  an exceeded window stopped with exit 3 and ran nothing;
  a dry run executed and recorded nothing.
- The stage data:
  every stage names its root use and unprivileged route,
  every stage that uses `sudo` is marked and no other stage uses it,
  every root run of the tools names the absolute path of node,
  and the order freeze,
  baseline,
  final pass,
  swap,
  identity record,
  release holds.
- A dry run of every command through the built bin:
  every command printed,
  nothing executed,
  no folder created.

Not verified:
every real root command,
the real `btrfs send` and `receive`,
the mountpoint guard,
the freeze on the real subvolumes (the owner route was measured on a scratch subvolume in the plan),
the bind mounts,
the format of `btrfs subvolume list -u` for a subvolume nested in another nested subvolume
(only the first rows and the checkout's own row were printed),
and the behavior of the commit service and the wrapper while frozen.

## Where the plan no longer matches the repository

Measured 2026-11-07 between 13:00:00-05:00 and 13:30:00-05:00.

- Registered worktrees:
  371 (including this task's own worktree),
  the plan counts 340.
  Nested under `.claude/worktrees`:
  277,
  of which 262 are subvolumes and 15 are plain directories;
  the plan counts 246,
  of which 242 and 4.
  External:
  16 under `~/worktrees`,
  71 on the 860,
  6 spelled `~/temp/agent`;
  these match the plan.
- Free space:
  the 860 had between 571 and 576 GiB available (plan:
  662),
  the NVMe between 300 and 303 GiB (plan:
  350).
  With the plan's own estimate (263.6 GiB nested plus 74.3 GiB external) the pre-flight rule leaves about 160 GiB on
  the 860 against the required 150 GiB,
  so a live measurement that comes out a little larger will be refused.
- `btrfs subvolume list` prints the `path` column relative to the parent named by `top level`,
  not to the filesystem's top level
  (measured by the coordinator with root on 2026-11-07).
  The plan's `--inventory-main=home/user/Monochromatic` and the prefix `home/` still hold,
  because the copy tool rebuilds the full path from the parent chain,
  but the plan's check `grep ' path home/user/Monochromatic'` on the raw list would find nothing.
- The wrapper prints a notice on every command that `.git` holds both `cli-git*` and `monochromatic-git-policy-cli*`
  files;
  they move with `.git`,
  and the plan does not mention them.
- Work items W2 to W15 are not built;
  W1 is,
  in `package/repository-link/check`,
  and its install is the `i1` stage.

Defects in the plan's own commands that the runner or the tool fixed:

- `sudo node` fails when node comes from the version manager;
  the runner passes the absolute path of the node binary.
- The plan's F5 reused the snapshot,
  received and state folders of F3,
  and its flipped-byte test seeks 100000 bytes into the stream of `s001`,
  which is probably a short clone stream (not measured).
  The runner uses fresh folders per case,
  a deterministic leftover (a subvolume named `s000` with no record),
  and flips a byte in the stream of `s000`.
- The plan's F2 inventory used `btrfs subvolume list -o`,
  which the plan itself says is not recursive;
  the runner lists the whole fixture filesystem with `-u`.
- `move-repository.ts --freeze` froze and copied in one run;
  the freeze is now its own pass,
  and the final pass refuses to run unless everything it sends is frozen.
- Pass two of the external worktrees had no `--delete`,
  so a file removed after pass one stayed in the copy unreported;
  the second pass now reports every such file and still deletes nothing.
- The copy tool keyed its records by position,
  so a resume after a worktree was added or removed would reuse a record for a different path;
  records are now keyed by UUID plus path.
- A root `git` in the staged verification can rewrite the index as root;
  the runner runs Git as the user.
- The plan does not state the window maximum;
  the section "The maximum duration of the window" derives it.

## What to check

Status:
TODO

After `prepare`:

1.  The last line says `prepare finished.`
2.  `status` lists every prepare stage as completed.
3.  `ls /var/home/user/repository-move-state/wizard` shows `first-pass-seconds`,
    `du-seconds`,
    `subvolume-sizes.txt` and `first-pass-print.txt`.

After `cutover`:

1.  The last line says `cutover finished.`
2.  `ls -ld /var/home/user/Monochromatic` prints a line starting with `l` and ending with
    `-> /var/mnt/encrypted/repository/Monochromatic`.
3.  `ls /var/home/user/repository-move-state/wizard/window-end` exists and the release stage printed `window length:`.
4.  `cmp /var/home/user/repository-move-state/wizard/baseline-frozen.tsv /var/home/user/repository-move-state/wizard/through-link.tsv`
    prints nothing.
5.  `~/.local/share/monochromatic-dev/repository-link/repository-link-check` prints `ready (on the 860)`.

Still the coordinator's,
not run by the runner:
the plan's phase 5 checks (a test commit in a throwaway worktree,
which the wrapper may push),
`mise trust --all` then `mise trust --show`,
a started `created` container,
a fresh worktree's `pnpm install` then `find node_modules -xtype l` printing nothing,
the probe session for the harness key and the harness migration,
and the removal of the old trees and snapshots after the validation window.

## Restore

Status:
TODO

- Before the swap:
  the `thaw` command.
  Expected:
  every frozen subvolume reads `ro=false`,
  no external folder is a mount,
  `claude-cpu-watch.service` is active again.
- After the swap,
  before the release stage:
  the `rollback` command.
  Expected:
  `/var/home/user/Monochromatic` is a real folder again,
  the links sit beside it as
  `Monochromatic.link-rolled-back-<timestamp>` and `worktrees.link-rolled-back-<timestamp>`,
  the identity file says phase `before`,
  and the thaw checks pass.
- After the release stage:
  the plan's full reverse migration,
  section "Rollback".
- The runner deletes nothing.
  What it leaves for the coordinator to delete,
  with the commands it prints:
  the fixture images and mount folders,
  a partial received subvolume after an interrupted transfer,
  the aborted attempt's `f###` snapshots,
  received copies and final folder,
  and later the old trees,
  the source snapshots and the received first-pass copies.
