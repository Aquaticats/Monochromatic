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
    `p3a-footprint` runs `measure-footprint.ts`,
    which walks the whole checkout with `btrfs filesystem du` (sharing counted once).
    A worktree an agent removes during the walk fails that walk,
    so the walk is made again,
    at most three times,
    and only when every line btrfs printed says an entry was not there;
    the first line of `main-du.txt` says which attempt measured and which entries were removed meanwhile.
    Any other error stops the stage at once.
    The measurement is not split by subvolume:
    the worktrees share extents,
    so one measurement per subvolume would count the shared data once per worktree
    (the plan's per-worktree figure is 3753 GiB against about 264 GiB counted once)
    and the space rule would refuse for a reason that is not real.
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
    excusing the empty placeholder folders a snapshot shows for subvolumes:
    those of the inventoried nested subvolumes,
    and any other (inode 2) whose live path holds the same placeholder or nothing,
    each named in the output
    (`doc/troubleshooting/btrfs-snapshot-keeps-an-empty-placeholder-for-a-nested-subvolume.md`).
    A placeholder whose live path holds a real folder or subvolume stops the run:
    the inventory missed it.
    A difference stops the run and names the copy.
    The window's content compare (`c13b-rsync`) applies the same rule.

The runner then stops.
It does not start the cut-over.

### The cutover command

Run it only when no agent works in a worktree.
Stop `kopia-snapshot-all.timer` first and start it again afterwards:
`c03-holders-before` refuses the hourly kopia run and `c06-stop-units` does not stop it.
One run takes about 3 to 5 minutes here.
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
    the wrapper state
    (no `busy/` or `push-retries/` file in either commit service state folder:
    `~/.local/state/cli-git/service` of the releases before the rename
    and `~/.local/state/monochromatic-git-policy-cli/service` of the renamed wrapper),
    and a fresh `btrfs subvolume list -u` with the print mode of the freeze and the final pass.
    A registration of a worktree whose folder is gone fails `c01-pre-freeze-table`:
    `git worktree prune` settles them
    (the first attempt met 36,
    all under scratch folders that had been deleted).
2.  `c06-stop-units` (needs `--assert=c06-stop-units`):
    stops the user units:
    every `cli-git@*` and `monochromatic-git-policy-cli@*` socket and service
    (the earlier sockets keep listening after the rename)
    and `claude-cpu-watch.service`.
    A rerun merges into the list of stopped units instead of replacing it,
    and the check fails when `systemctl` itself cannot answer.
    The window clock starts here and commits fail from here.
    Nothing stops a new activation between this stage and the freeze;
    `c08-holders-frozen` scans again after the freeze and stops the window when it finds a holder.
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
    The table and the worktree list are compared with the frozen baseline with exactly one spelling allowed:
    Git prints the main worktree under its resolved path through the link,
    so that one line is spelled as the 860 path there and as the link path in the baseline,
    both spellings are required where they belong,
    and a difference on any other line still fails the stage.
11. `c20c-refresh-checkouts`:
    a plain `git status` in every registered checkout,
    which takes the index lock and rewrites the index,
    then `mise exec -- pnpm install --offline --frozen-lockfile` in every checkout whose `node_modules` holds a link
    that points nowhere.
    The move gives every file a new inode,
    so an index keeps the stat data of the files as they were before the move:
    a commit there stops at the wrapper's 3-second scratch ceiling,
    again on every retry,
    until a status has rewritten it
    (`doc/handover/cli-git-rust-implementation.md`,
    section "A commit stopped at the scratch ceiling on every retry after the 860 move").
    The relative links of `node_modules` reach the pnpm store through the checkout's physical path,
    so after the move they point nowhere:
    of 12 checkouts sampled the main one held 19 and 2 worktrees held some,
    the main checkout's broke `pi`,
    and 5 checkouts had no `node_modules` at all.
    `mise exec -- pnpm install --offline --frozen-lockfile` repaired the main checkout in 7.6 s.
    `c25-pnpm-links` checks only a fresh worktree and cannot catch this,
    so this stage is what repairs the existing ones.
    It runs after `c20-through-link`,
    whose table holds the index hashes,
    and before the release,
    because commits resume there.
    A `--no-optional-locks` status does not rewrite the index (measured 2026-11-08),
    so this stage runs Git without that flag.
    A checkout with no `node_modules` is named and left as it is.
12. `c21-release` (needs `--assert=c21-release`):
    starts the stopped commit sockets and `claude-cpu-watch.service` and closes the window.
    After this the fast rollback no longer applies.
13. `c22-verify-final-checksum` (root,
    outside the window):
    the same comparison as `c10b-verify-final` for the same final copies,
    reading every byte.
    The old tree (`.nvme-before-860-<timestamp>`) stays for the rollback until this stage is recorded done;
    the `status` command prints `DO NOT DELETE` until then and says the tree may be deleted by the owner after it.
14. `c23-link-check` to `c28-ide-through-link` (as the user,
    outside the window):
    the checks after the cut-over of the plan's items W12 to W15.
    `c23-link-check` runs the installed check again (the wrapper's `doctor` prints the same line);
    `c24-mise-trust` runs `mise trust --all` and shows the trust of the root,
    a package and a worktree;
    `c25-pnpm-links` installs in a fresh worktree and lists every link of `node_modules` that points nowhere,
    which must be none (the worktree and branch stay for you to remove);
    `c26-kopia-snapshot` makes the first snapshot of the repository source and lists the last ones to read their size;
    `c27-container-bind` starts a container with the repository as a read-only bind spelled through the link,
    on the first image already on the host with the memory,
    processor and process counts capped;
    `c28-ide-through-link` needs `--assert=c28-ide-through-link`,
    because only you can see the IDE read the files through the link.

#### What stopped the first cut-over attempt (2026-11-08T06:33:36-05:00)

`repository-move-cutover-1` stopped at `c01-pre-freeze-table` before the window:
36 of 375 rows were `missing`,
registrations in `.git/worktrees` whose folders had been deleted
(34 under `/var/mnt/encrypted/temp/agent`,
removed with the old agent scratch the owner chose to delete,
and 2 under `/var/home/user/temp/agent`).
`git worktree prune --dry-run --verbose` listed exactly those 36 as "gitdir file points to non-existent location";
with the encrypted drive confirmed mounted and the folders confirmed absent,
`git worktree prune` removed them and a second dry run listed none.

A holder scan run by hand after that found processes that `c03-holders-before` would refuse:
the coordinator's own Claude Code session and its MCP servers (started in the repository),
terminal shells (one running `atop`),
Zed and its extension host,
a Codex session with three app-server daemons,
a DeepSeek `dsh` session with its search-fetch server,
the `adb` server,
and the Git wrapper's commit service holding a descriptor on
`.git/index.lock.cli-git-marker-<pid>.<pid namespace>-<batch>-<uuid>`.
That last one is by design:
when a batch ends the service makes the lock file of its next batch ahead of time
(`package/git-policy/cli/src/native/landing/service_index_prestage.rs`)
and keeps it open while idle,
so a running instance that has landed a commit is always a holder.
Measured on 2026-11-08:
the file of batch `1794117366015935` was born about 10 ms after batch `1794117366015934` wrote the index.
The number in the name is the batch,
not a time.

`c03-holders-before` runs before `c06-stop-units`,
so stop the running instance first
(`systemctl --user stop 'monochromatic-git-policy-cli@<id>.service'`,
after its last push is done);
its socket starts it again on the next commit.
Start the session that drives the runner from a folder outside the repository
(`--add-dir` the repository),
and keep that session's shell out of the repository:
a `cd` into it carries over to the next command and makes the shell a holder.

#### What the cut-over of 2026-11-08T07:19:28-05:00 to 2026-11-08T11:03:25-05:00 met

The window opened at 07:19:28 and the release stage finished at 11:03:25,
about 11 minutes before its maximum.
The source of the move tool could not change while the tree was frozen,
so two stages were recorded by hand and one was helped by a labelled script.
The scripts named here are in `${HOME}/temp/agent/860-move/`.

The holders found across the attempts,
in order:
this session and its shells,
Codex,
`dsh`,
`adb`,
Zed,
16 terminal shells,
`atop`,
the commit service's made-ahead lock file,
two missed shells,
and kopia.
The owner on the second-to-last of those and the last:
"Sorry I forgot to quit the 2nd Claude Code session.
You can temporarily disable the kopia job if you need to."

Measured stage times,
in minutes:
`c10-final-pass` about 10 per attempt,
`c10b-verify-final` 13.5,
`c11-external-final` 7,
`c12-verify-staged` 29.5,
`c13b-rsync` 63.5,
`c14-sharing` 28.5,
`c15-counts` 3,
`c16-root-owned` 2,
`c17-externals` 25,
`c20-through-link` 32.

`c05-fresh-inventory` compared the freeze lines with every `btrfs send` line,
which counted the `--no-data` comparison beside each real send.
Fixed in `e388bcf2ed`,
with a test shown failing both ways
(a correct plan rejected;
the comparison removed).

`c06-stop-units` called `xargs --arguments-file`.
The option is `--arg-file`,
so the stage stopped nothing,
and `|| echo` hid that from its test.
Fixed in `dd88b4def4`,
with one unit per `systemctl` call,
because `systemctl` refuses a whole list when any unit in it is not loaded.
The failed attempt left `window-start`,
which was renamed `window-start.aborted-c06-xargs` so the clock restarted.
A sweep of every long option in the dry-run text against each tool's `--help` found no other wrong option;
short options and option values were not checked.

`c10-final-pass` stopped with status 141 (SIGPIPE) at `f210` and at later subvolumes,
in the `btrfs send --no-data | btrfs receive --dump` of `package/repository-move/cli/src/final-diff-step.ts`.
The first explanation given to the owner,
that the diff exceeded the runner's output limit,
was wrong:
the limit is 64 MiB (`CHILD_OUTPUT_MIB` in `package/repository-move/cli/src/constants.ts`),
the `f210` diff was 3,035,324 bytes,
and the same step succeeded for `f210` on the next run.
So the stop is intermittent and its cause is not established.
Every stop was on a diff of roughly 9,000 to 16,500 commands;
small diffs never stopped.
The candidate to test on the snapshots with `PIPESTATUS`:
`receive --dump` exits at the end-of-stream marker while `send` is still writing,
and `pipefail` reports that as a failure.
The workaround,
`resume-final-pass.ts`,
counted the commands again in a separate pipeline on each stop,
wrote `fNNN.diff.json` with `changed: yes` and the real count,
and reran the stage.
A record saying `changed: yes` can only add verification,
because a copy is skipped only for `changed: no`.
The records it wrote,
with their command counts:
`f216` 16,317,
`f221` 8,895,
`f235` 16,302,
`f238` 16,533,
`f246` 16,317,
`f263` 15,932,
`f267` 16,311.
The tool wrote the record of `f210` itself on the retry (15,967 commands).
The status was later measured on throwaway subvolumes and the step fixed;
the section "What landed after the window" says how.

The check of `c17-externals` can never pass:
under `bash -e -o pipefail`,
`grep --invert-match` exits 1 when it selects no lines,
which is what an empty compare or one holding only `*deleting` lines gives,
and a real difference fails it by design.
On throwaway compare files of those three kinds the shipped check exited 1,
1 and 1,
and the corrected one (`{ grep ... || true; } | head`) exited 0,
0 and 1.
Rerunning the 25-minute compare did not fit the window,
so `finish-c17-record.ts` wrote `done/cutover/c17-externals` by hand,
and the record's third line says so,
after these held:
the stage's compare file was empty for its logged `rsync` runs,
all with status 0;
`tr-s406` was missing from the logged `rsync` lines,
so it was compared separately,
with status 0 and no lines;
and a positive control with the same options on real trees
(`tr-w400/doc` against the copy of `tr-s406/doc`)
gave status 0 and 2,574 lines,
2 of them content differences.
Not established:
whether the stage's own loop compared `tr-s406`
(17 folders exist and 16 `sudo` lines were logged).
A control on a scratch pair failed with `rsync` status 3,
because `rsync` under `sudo` refuses the symlink at `/var/home/user/temp`;
it says nothing about the move.

The check of `c20-through-link` can never pass through a link:
it is a plain `cmp` against the frozen baseline,
and Git prints the main worktree under its resolved path.
Both pairs differed by exactly one line,
the main worktree's,
by spelling alone:
same commit `dd88b4def4`,
same state hash,
same counts.
`git fsck --full` exited 0 and printed only `dangling` lines and the scan's own lines.
The stage took 32 minutes and 12 remained,
so `finish-c20-record.ts` wrote `done/cutover/c20-through-link` by hand,
and the record's third line says so,
after a comparison allowing only that exact spelling passed
and a planted difference on another line was caught in each file.

`c27-container-bind` took its image with `grep ... | head --lines=1` under `pipefail`,
which exited 141 on this host's list of 631 images (reproduced on the real list),
so the stage could not pass here.
It now reads the whole listing with `awk 'NR == 1'`,
and `guided-container-bind-limits.unit.test.ts` runs that line on a listing longer than a pipe holds.
The other stages' commands and checks in `guided-cutover-stages.data.ts` were read for this shell trap
and nothing else was found
(a second pass by a Claude Code native subagent,
not an independent review).

#### What landed after the window

Each of these was fixed after the window,
on branch `fix/repository-move-cli-checks-after-the-860-move`
in `.claude/worktrees/pi-repository-move-tool-fixes`,
with a test that was run against the shipped code first
(same file,
same command,
watch the status):

- The check of `c17-externals`
  (`642f8e148c` the test,
  `f395b88b8f` the fix,
  `guided-external-compare-check.unit.test.ts`)
  runs over the empty compare,
  the compare holding only `*deleting` lines,
  a real difference,
  and a compare longer than the 20 lines the report shows.
  The shipped check exited 1 on the first two
  (`expected 1 to equal +0` for each);
  the fix wraps the `grep --invert-match` in a group that keeps its status.
  No other check in `guided-cutover-stages.data.ts`,
  `guided-prepare-stages.data.ts`,
  `guided-loop-image-stages.data.ts` or `guided-undo-stages.data.ts`
  has a `grep` whose empty selection is a legitimate pass:
  every other `grep` there either must match to pass,
  sits in a command substitution whose status `test` ignores,
  or is guarded by `|| true`.
- The check of `c20-through-link`
  (`42a4d409ef` the test,
  `f395b88b8f` the fix,
  `guided-through-link-check.unit.test.ts`)
  allows exactly the main worktree's moved spelling.
  Against the shipped plain `cmp` the real pair failed
  (`differ: byte 69, line 2`)
  and a planted difference on another line must still fail,
  which the test holds as well:
  another table line,
  a nested worktree spelled under the 860 path,
  a list that still spells the link path,
  and a planted difference in the list.
- `final-diff-step.ts` no longer stops on a complete dump
  (`39ef139236` the test,
  `a8cf2df158` the fix,
  `final-diff-step.unit.test.ts`).
  The status 141 was measured first,
  on throwaway subvolumes in `${HOME}/temp/agent/repository-move-pipe-measure`
  with `btrfs-progs` 7.1,
  not on the move's snapshots:
  a `--no-data` stream ends exactly at its end record
  (bisected:
  946,688 of 946,688 bytes are needed for a dump to succeed,
  16,802 commands),
  bytes appended after the record are ignored by `--dump`,
  and a stream cut short makes `--dump` exit 1 with
  `ERROR: short read from stream`
  (cut at 1,024,
  40,000 and 400,000 bytes).
  The shipped shape did not fail in 20 sequential and 24 concurrent runs on that stream,
  while the same shape with a reader that stops early exits 141 every time,
  so the harness can see the failure and the trigger is timing rather than size alone.
  In the real stops the receiver therefore exited 0,
  because `pipefail` reports the rightmost non-zero status
  and a truncated stream cannot leave it 0.
  The command now drains the rest of the stream after the dump,
  so the sender always finishes,
  and the group exits with the receiver's own status,
  so a dump cut short still fails the step.
  A test of the printed command was updated with it
  (`29062faa43`):
  `move-repository.unit.test.ts` expects the drained pipeline in print mode.
- A stage after the swap refreshes the moved checkouts
  (`4f6f3c80fd`,
  stage `c20c-refresh-checkouts`,
  `guided-refresh-checkouts.unit.test.ts`),
  which is item 11 of the cutover list above.
  It also covers the `node_modules` relink,
  which `c25-pnpm-links` cannot:
  that stage only installs in a fresh worktree.

Still open:

- The workaround scripts in `${HOME}/temp/agent/860-move/` can be deleted now:
  `resume-final-pass.ts`,
  `finish-c17-record.ts`,
  `finish-c20-record.ts`,
  `check-c17-evidence.ts` and
  `read-c20-differences.ts`.
- The stage times above are from the shipped tool.
  `c20c-refresh-checkouts` has not run on the real tree:
  its cost is one status per registered checkout
  (375 rows at the last table)
  plus an install in each checkout whose links dangle,
  which took 7.6 s for the main checkout,
  and it is inside the window.
  Until it runs,
  the existing checkouts other than the repaired main one are not relinked.

The copy tool's unit tests import the built package:
after a source change run `mise run //package/repository-move/cli:buildAndTest`,
not `test:unit`,
or the suite tests the previous build.
On this branch that task and `mise run //package/repository-move/cli:lint` both pass
(the lint reports 0 warnings and 0 errors,
with no rule loosened;
`ddab440fa4` clears the findings the new tests raised).

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
4.  `/var/home/user/repository-move-state/wizard/baseline-frozen.tsv` and `through-link.tsv` in the same folder
    differ in exactly one line,
    the main worktree's,
    and only by the spelling of its path
    (`/var/home/user/Monochromatic` in the baseline,
    `/var/mnt/encrypted/repository/Monochromatic` through the link).
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
