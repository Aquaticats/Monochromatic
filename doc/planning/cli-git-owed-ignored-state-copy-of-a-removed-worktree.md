# What the wrapper should do with an ignored-state copy it owes to a worktree that no longer exists

Status:
a proposal,
unapproved and non-final,
written by the Pi round on branch
`docs/planning-issue-6557-proposal-for-an-owed-copy-whose-worktree-is-gone`
on 2026-11-27T04:00:00-05:00,
for issue 6557.
Nothing here is decided,
no code,
test or `SPEC.md` text is changed by it,
and no record,
stage or worktree was settled or deleted by this round.
What is not yet known:
what the owner wants a stage folder to cost a person who did not watch the add;
whether a stage may ever be removed without a person's word;
whether the launcher's adds should stop leaving claims at all
(issues 6418 and 6339),
which decides how much of this matters;
and how the notices in issue 6557's own repository came to stop,
which the record there does not say and this round could not reproduce,
since the record that issue names still stands in this checkout.

## The two lines, and what keeps them coming

The wrapper names one interrupted add with two lines,
and both come from the recovery that runs before a forwarded command.

The first names the worktree and says the copy is still owed:

```text
// package/git-policy/cli/src/native/worktree_copying/worktree_copy_owed_copy_notices.rs:36
pub fn owed_in_another_worktree_notice(destination: &Path) -> String {
    let shown: String = quote(destination.to_string_lossy().as_ref());
    return format!(
        "monochromatic-git-policy-cli: the ignored-state copy an interrupted `git worktree add` owed {shown} is still owed; the first command run in that worktree makes it, and this command does not wait for it."
    );
}
```

The second names the stage folder the add left:

```text
// package/git-policy/cli/src/native/worktree_copying/worktree_copy_stage_reaping_proof_and_words.rs:206
pub(crate) fn kept_stage_words(
    container: &Path,
    reason: &str,
) -> String {
    return format!(
        "monochromatic-git-policy-cli: the stage {} was left by an interrupted `git worktree add` and was not changed: {reason}; remove it yourself once nothing needs it (`btrfs subvolume delete` for a snapshot, `rm -r` otherwise).",
        quote(container.to_string_lossy().as_ref())
    );
}
```

The first line is printed by the pass that finishes an owed copy,
which decides it only against the worktree the command runs in:

```text
// package/git-policy/cli/src/native/worktree_copying/worktree_copy_owed_copy.rs:551
        // The copy fills one worktree, and the command that runs in it is the one that waits: a command in another
        // worktree does not pay for the source's ignored state, and is told which worktree does.
        if !crate::worktree_copy_recovery::finishes_destination(
            destinations,
            add.destination.as_path(),
        ) {
            notices.push(owed_in_another_worktree_notice(add.destination.as_path()));
            continue;
        }
```

That test asks whether the recorded destination is this command's own worktree.
It never asks whether the destination still exists,
so a claim whose worktree is gone prints the same line as one whose worktree stands.

A claim outlives a whole worktree because of what the reaping does with the stage it names:

```text
// package/git-policy/cli/src/native/worktree_copying/worktree_copy_stage_claim_reaping.rs:461
            // While the stage stands its claim is what names it, so the record, the marker and the claim stay and the next
            //  command reports them again; a settled worktree keeps its records for the same reason.
            if settled.keep || stage_stays {
                continue;
            }
            remove_made(claim.as_path())?;
            remove_file_if_present(record.as_path())?;
            remove_file_if_present(claim.with_extension(&COMPLETE_SUFFIX[1..]).as_path())?;
            remove_file_if_present(claim.as_path())?;
```

`settled.keep` is the settlement of the add's destination record,
and it answers `false` for a worktree that is gone
(`worktree_copy_interrupted_add_settlement.rs:506` for a planned record,
`:553` for a bound one).
`stage_stays` is what keeps the claim after that.

## The stage is never provable for the route that copies files

The stage of an add is removed only with a record beside the claim:

```markdown
// package/git-policy/cli/SPEC.md:6392
A claim with a destination record beside it was made by an add,
and its stage is removed only with both proofs:
the record `<identifier>.made` beside the claim names the container whose creation it recorded,
```

The rule a person reads says the same,
and the last three of its clauses are what this issue found missing:

```markdown
// package/git-policy/cli/README.md:762
A stage a killed `git worktree add` left beside its destination is removed by the next command
  that runs the recovery,
  with one line saying what went,
  when the record written beside the stage's claim names the object still standing at that name
  and the folder holds nothing that record cannot account for;
  a stage nothing proves is named,
  left whole,
  and keeps its claim,
  with the reason it was kept,
  so a folder holding something you added there is never removed.
```

`SPEC.md` states the promise this issue measured to be false,
and the reason the promise is printed again on every command:

```markdown
// package/git-policy/cli/SPEC.md:3373
prints the stage it did not remove,
and keeps the record,
the marker and the claim while the worktree is
not proven whole and while that stage still stands:
so every later command that runs the recovery prints the same report again,
```

```markdown
// package/git-policy/cli/SPEC.md:3549
The cost:
an interrupted or failed add keeps naming the same worktree and stage in every command that
runs the recovery,
until the person finishes or removes it.
```

The `.made` record is written by the snapshot rung and by nothing else.
Its two release call sites are in the btrfs snapshot route:

```text
// package/git-policy/cli/src/native/worktree/worktree_snapshot.rs:149
        crate::worktree_copy_stage_made_object::record_made(
```

```text
// package/git-policy/cli/src/native/worktree/worktree_snapshot.rs:200
    if let Err(error) = crate::worktree_copy_stage_made_object::record_creating_at(
```

The route that copies the ignored state file by file,
which is the route every filesystem without snapshots takes and the route this machine takes,
creates the container and records the registration it fills,
and writes no `.made`:

```text
// package/git-policy/cli/src/native/worktree_copying/worktree_copy_stage.rs:219
    create_private_directory(stage_container).map_err(raw_failure_of)?;
    // The registration goes in before the first byte of the payload, so a container killed at any later instruction says
    // which copy it belongs to.
    if let Some(bound) = binding
        && let Err(error) =
            crate::worktree_copy_stage_binding::record_stage_binding(stage_container, bound)
    {
        std::fs::remove_dir_all(stage_container).map_err(raw_failure_of)?;
```

So `provenance_of` reads no record and keeps the folder for ever,
with exactly the reason the issue quotes:

```text
// package/git-policy/cli/src/native/worktree_copying/worktree_copy_stage_reaping_proof_and_words.rs:145
    let Some(made) = read_made(claim) else {
        let stands: bool =
            std::fs::symlink_metadata(made_record_beside_claim(claim).as_path()).is_ok();
        // A file that stands there and does not read is damage, which is said differently from an absent record, so the
        // person knows whether to look at a file of their own or at nothing.
        return Ok(Provenance::Unproved(match stands {
            true => String::from(
                "the record of the object its creation made does not read as this release writes it",
            ),
            false => {
                String::from("no record of the object its creation made stands beside its claim")
            }
        }));
    };
```

The proof that would remove such a folder is one `SPEC.md` already describes
for a container no add recorded:

```markdown
// package/git-policy/cli/SPEC.md:3458
A copy's own container is not held to that record,
because a copy creates one and records the registration it fills as the step after it:
a container whose binding names the destination it fills is the copy's own and goes,
```

The reaping never applies that sentence to an add's stage.
It branches on whether a destination record stands beside the claim
(`worktree_copy_stage_claim_reaping.rs:402`),
and an add always writes one,
so a copy-route stage is judged by the snapshot's rule,
which it can never satisfy.

## What `git worktree remove` does with an owed record today

The wrapper's removal takes the linked-worktree lifecycle,
because the ladder serves a removal from any checkout:

```text
// package/git-policy/cli/src/native/worktree_copying/worktree_copy_forward.rs:63
/// when a `git worktree add` or `git worktree remove` that the ladder
///  serves may also run in the main checkout.
/// Why:
///   The ladder snapshots or clones from the creating worktree,
/// and the main checkout is one;
/// a removal is served from
///  any checkout.
```

So a `git worktree remove` run in the main checkout prints both notices first,
naming the worktree it is about to remove,
removes the worktree,
and changes no record.
The line the removal leaves its own work by is about its units,
never about the copy records:

```markdown
// package/git-policy/cli/README.md:2180
When the wrapper serves `git worktree remove <path>` (the subvolume destroy,
or Git's own removal run as a child),
it reads the worktree's registration first,
and once the removal ended well and the registration is gone it stops that worktree's socket unit and then its service
unit through the user manager,
never by killing a process by name.
```

Every other command in the main checkout runs no recovery at all,
so the checkout where the person removed the worktree is silent from then on,
and every linked worktree keeps printing the pair:

```text
// package/git-policy/cli/src/native/worktree_copying/worktree_copy_forward.rs:83
    match identity {
        WorktreeIdentity::OutsideWorktree => return false,
        WorktreeIdentity::MainWorktree {
            ..
        } => {
            return serves_ladder || creates;
        }
```

Both remedies either line prints leave the record where it is.
The second says to remove the folder by hand,
and the first says the next command in that worktree makes the copy
or the same `git worktree add` line makes it again.
Measured below:
the re-add copies the ignored state into a new worktree and the old claim and stage stand on.

## The one record that does end with its worktree

The record of a worktree whose copy *ended* settles on the proof that the worktree is gone,
which is the answer issue 6557 asks for and the mechanism the other record lacks:

```text
// package/git-policy/cli/src/native/worktree_copying/worktree_copy_unfilled_record.rs:496
        // `match` on where the worktree stands: one that is gone or was replaced settles the record, one that still
        // stands is named where it is now, which a `git worktree move` changed from the path the record wrote, and one
        // whose registration cannot be read is kept for a later pass rather than settled on a guess.
        match standing_at(common_dir, &copy) {
            Standing::Nowhere => {
                remove_file_if_present(path.as_path())?;
            }
            Standing::At(destination) => {
                notices.push(unfilled_notice(&copy, destination.as_path()))
            }
            Standing::Unproven => {}
        }
```

`standing_at` asks the recorded path's object,
then the repository's own registrations,
and its three answers are kept apart on purpose
(`worktree_copy_unfilled_record.rs:378`).
The owed copy's pass has no such step.

## What was measured

The subject was current `main`'s code,
built as `monochromatic-git-policy-cli 0.6.1-alpha.47` from this worktree's branch,
which held no commit of its own when the build ran.
Every add and removal named below ran through that build,
as a `git` first on `PATH`,
in one disposable repository under `${HOME}/temp/agent`,
with a `HOME`,
an `XDG_RUNTIME_DIR` and a `TMPDIR` of its own,
no `DBUS_SESSION_BUS_ADDRESS`,
and every command inside a capped `systemd-run --user --scope --slice=agent-work.slice`.

The repository held one commit,
one tracked file and a `.gitignore`,
and an ignored `build-state/` tree of 20 000 files
(313 MB by `--apparent-size`,
138 MB by `du`,
on this machine's compressed ZFS home).
A second linked worktree stood beside it from the start,
so a command could run where the recovery applies.
Through the build,
a whole `git worktree add` of that repository took 10.1 s,
and `git worktree remove` of the clean worktree it made took 1.2 s,
left no record and no stage.

Three kills were made,
by `SIGKILL` to the process group of the add,
which is what a caller's timeout does to a wrapper that outlives it:
during the copy's staging,
during its installation,
and after the worktree was removed.
Two more things were then watched:
a re-add at the same path,
and a command in the worktree that re-add made.

### A kill during staging reproduces issue 6557

The stage's binding appeared 0.846 s after the add started,
and the group was killed 0.4 s later.
The records then stood as the issue describes them:

- `<id>.add`,
  525 bytes,
  `phase=planned`,
  and `<id>.stage`,
  532 bytes,
  whose owner is dead;
- the stage folder with `registration` and `payload`,
  both made,
  no `.made` record beside the claim;
- the worktree registered,
  and its checkout whole.

`git worktree remove <path>` printed both notices,
naming the worktree it was removing,
exited 0,
and left every record standing.
Then 20 `git status --porcelain` commands ran in the second linked worktree:
**20 of 20 printed both lines**,
and the records stood after all 20.
Five of the same command in the main checkout printed neither of them,
which is where a person who removes a worktree works.

The same state was reached a second time
(the binding at 0.666 s,
the kill at 2.367 s)
and behaved the same,
so this is not one lucky interleaving.

### Both remedies the lines print fail

Running the same `git worktree add` line again at the same path,
which is what the first line names as the way to make the copy:

- copied 20 201 ignored entries into the new worktree and exited 0;
- printed the two notices before its own output;
- left the old claim,
  the old destination record and the old stage folder exactly where they were;
- and the next command in another worktree printed the pair again.

Removing the stage folder by hand,
which is what the second line names,
ended that claim on the next command:
one recovery settled the claim and its destination record,
the pair went from two lines to one,
and the remaining line was the other add's.
So the stage's existence is what holds its claim,
and a person who does only half of what the two lines say keeps the other half.

### A kill during installation settles by itself

Killed 2.0 s after the binding (0.188 s),
the add had already written its journal
and its record of an unfilled worktree,
and its stage held all 20 000 payload entries.
Three commands in the second worktree printed one owed line and one unfilled line each.
`git worktree remove` printed both.
Then five commands printed nothing,
the journal was recovered because the destination was no longer registered,
the stage went with it,
and the records and the stage folder were both empty.
That is the same code path behaving as designed,
and it is worth keeping in view:
the permanent case is the one where no journal exists yet.

### A command in the worktree is slow, and says nothing true about what it did

After the re-add,
one `git status --porcelain` in that worktree took **210.9 s**,
printed that it was finishing the owed copy and then
`finished the ignored-state copy ...: copied 0 ignored filesystem entries`,
and settled every record.
A command that reports a copy of nothing,
after three and a half minutes,
on the first command a person runs there,
is the shape of the same pass under load.

### This checkout's own state

The state folder is
`.git/monochromatic-git-policy-cli-worktree-copy/v1` in the main checkout,
and the stage folders stand beside the worktrees under `.claude/worktrees/`.
Read by listing only,
before this round ran any command,
it held 553 files:
477 pending journals,
35 claims,
35 destination records,
4 taking marks,
1 unfilled record and 1 journal,
with 36 stage folders beside them.
The same listing at 2026-11-27T03:42:38-05:00,
after other sessions' commands had run recoveries,
held 18 claims,
20 stage folders,
23 unfilled records,
477 pending journals,
and 62 registered worktrees.
**No claim in either listing had a `.made` record**,
and 17 of the 18 named a worktree that still stands.
The one that named a worktree that is gone was
`e34a3581-2b9a-4e93-91b4-8ac49576ba91`,
whose destination is
`.claude/worktrees/split-wrapper-worktree-copying-baseline-worktree-copying-2026-11-26`,
the very worktree issue 6557 is about,
and its record stood from 2026-11-26T21:26 until this round looked,
about six hours later,
through other sessions' recoveries.
Its stage folder holds `registration` and a `payload` of 43 entries.

The other 17 claims at that timestamp,
`0ae0dc7a`,
`0ae4afd5`,
`1294c9cf`,
`306c3089`,
`4f28e9cf`,
`54a80ea9`,
`554aafbf`,
`58455fab`,
`79d86db3`,
`7d38c492`,
`8225a120`,
`90772432`,
`ad0a4ae8`,
`b1dfbdd2`,
`eb834098`,
`edba1d9b` and
`f05a2383`,
each name a worktree that still exists and is still registered,
so none of them is a notice about a worktree that is gone.
One of those,
`edba1d9b`,
has no stage folder left,
so its owed line has nothing beside it for the second line to name.

The stage folders take disk too:
`du -s --block-size=1M --total` over the ones standing at 2026-11-27T03:56:10-05:00
came to 42 628 MiB in 16 folders,
the largest 5 337 MiB and the smallest 1 MiB,
and their number moves because other sessions' commands settle some of them while this one reads.

One `git status --porcelain` in a linked worktree here printed 62 notice lines:
27 owed lines naming 16 distinct worktrees,
25 stage lines,
and 10 unfilled lines.
Two owed lines for one worktree come from two claims,
since a retried add writes a second claim for the same destination.

This round's own first command,
a read-only `git merge-base` in its worktree,
ran that recovery and changed those records:
the claims fell from 35 to 30 and two stage folders went,
and a later command took them to 18.
The listings before and after are kept,
and the change is the wrapper's own doing,
not a deletion this round made.

### What could not be measured

- A stage the snapshot rung made.
  A btrfs subvolume and its `.made` record need a btrfs fixture root or a guest,
  and this home is ZFS.
  The snapshot route is the one case where the second line ends by itself after the folder's last write,
  and it is untested here.
- The ZFS clone route,
  which opens `/dev/zfs` on this host's pool and is not a run's to touch.
- A removal the wrapper serves with a subvolume destroy,
  which is what prints the extra line a destroy leaves.
- A worktree removed with `rm -rf` rather than `git worktree remove`.
  Only `git worktree prune` after the directory was gone was run,
  and a claim whose worktree went that way still printed,
  because the stage still stood.
- The load average issue 6557 reports,
  100 to 235.
  This host was loaded with other sessions,
  but no figure here is a comparison against a quiet host.
- Why the notices stopped in issue 6557's own repository.
  The record stands here now,
  so nothing this round could see makes them stop without a command run in that worktree or a person removing the stage.

## What the owner has to decide

Two questions,
asked as three,
because the second splits by who acts.

What ends the claim of a worktree that is gone:
a proof the wrapper can take by itself,
a bound after which no claim counts,
or a person's command.

What ends the stage folder,
given that the wrapper cannot prove a copy-route stage its own:
a comparison against the source,
a person's command,
never.

## The options

Each option is one answer to both questions.
Each is designed until its problems show,
and the effort to build each is stated,
not used to rank it.

### Option 1: the worktree's absence settles the claim, and the source settles the stage (Recommended)

What it does:

- The owed-copy pass asks where the worktree stands,
  with the same three answers `standing_at` already gives the unfilled record:
  at a path,
  nowhere,
  or unproven.
  `Nowhere` settles that claim,
  its destination record,
  its marker and its stage in one pass,
  and prints no line about the worktree.
- Before the stage goes,
  the wrapper proves it holds nothing the source does not:
  every entry of `payload/` must be at a path that source still holds with the same type,
  the same mode and the same bytes.
  One entry that differs keeps the folder,
  and the line names it.
  A source that cannot be read,
  or that is gone,
  keeps the folder and says so,
  because then the stage is the only copy left.
- `Unproven` behaves as it does today:
  kept,
  and drawn to the person's attention.

Pros:

- Both lines end with the worktree,
  with no command and no waiting,
  which is what the issue asks for and what `SPEC.md` already promises.
- The proof for the stage is the one thing that says whether a person can lose anything,
  so the folder goes only when every entry it holds is in the source too.
- The source's own state is the evidence,
  so a stale stage is removed a day later as easily as a minute later,
  and nothing new has to be remembered about when it became stale.
- One line remains for a stage that does hold something unique,
  and that line can name the entry and the reason,
  which is a line a person can act on instead of a line about a worktree that is gone.

Cons:

- A comparison of the stage against its source costs one walk of the folder when the obligation is already orphaned,
  and this host's stages hold up to 20 000 entries,
  so it is seconds,
  not milliseconds,
  on the first command after the removal.
  It must be bounded by the copy's own ceiling,
  and a stage of a whole checkout on btrfs is a snapshot,
  which has to be read through the same rules the accounting already has.
- The rule needs a definition of "the same bytes" that a file changing under it cannot break,
  which means a second look at an entry whose first look differed,
  the way the settling pass already works.
- It changes the answer for a claim whose worktree a person removed by mistake and re-added at the same path:
  the record of the earlier add settles,
  and the new worktree is filled by its own add,
  which is what the re-add does today anyway.

What a person sees,
after removing a worktree whose copy was interrupted:

```text
$ git -C checkouts/other status --porcelain
# (nothing on standard error)
```

and,
where the stage held something the source no longer has:

```text
$ git -C checkouts/other status --porcelain
monochromatic-git-policy-cli: the stage "/work/repo/.claude/worktrees/.cli-git-worktree-copy-e34a3581-2b9a-4e93-91b4-8ac49576ba91" was left by an interrupted `git worktree add` of a worktree that is now gone, and it is the only copy of 1 entry its source no longer holds: build-state/d042/f007.bin. It was not changed; `rm -r` removes it.
```

### Option 2: settle the claim, keep the stage, say it once, and give one command that removes them

What it does:

- The claim settles on the same proof as option 1,
  and the pair of lines about the worktree end there.
- The stage line keeps its present rule and its present refusal,
  and is printed once per stage per repository rather than on every command:
  a marker beside the claim records that the line was said,
  and the line names a wrapper command that lists every such folder,
  with its size,
  its age and the entries the wrapper cannot account for.
- That command is the only thing that removes a kept stage,
  it refuses a folder it cannot prove the wrapper's own,
  and it removes one only after naming what goes.

Pros:

- Neither line loses anything:
  the stage is a copy the wrapper made and cannot prove,
  and a person is told once and given one place to act.
- It is the smallest change that satisfies the first half of the issue,
  and it can be built and trusted before any comparison exists.
- A person who was not watching sees one line and a command,
  instead of the pair on every command.

Cons:

- The noise falls from a pair per command to one line per stage,
  and on this host that is 20 lines for 20 folders,
  printed once but standing until someone runs the command.
  Nothing removes them by itself,
  so a machine that nobody reads keeps the folders and their disk.
- It adds a durable "already said" mark and a management command,
  so it is more surface than the settlement alone,
  and the mark is one more record to be kept in step with the claim it describes.
- The person still has to be told what a folder holds before deciding,
  and the wrapper's own words cannot say whether it is safe to remove without the comparison of option 1.

What a person sees:

```text
$ git -C checkouts/other status --porcelain
monochromatic-git-policy-cli: 3 stage folders of interrupted `git worktree add` runs stand beside this repository, 1.1 GB, and nothing proves they hold anything the source no longer does: `git monochromatic-git-policy-cli worktree-copy --list` names them, and `--remove` removes the ones it can prove are its own.
```

### Option 3: the obligation lives where Git deletes it

What it does:

- The durable record of a copy still owed moves from the journal root into the worktree's own administrative directory,
  `.git/worktrees/<name>/`,
  as soon as Git registers the worktree,
  and the stage's name is recorded there too.
- Git's own removal deletes that directory,
  so the obligation dies with the worktree by construction,
  and no command has to take a proof about a worktree that is not there.
- The recovery reads the registrations of this repository instead of the journal root for this record.

Pros:

- It is the only option where the record cannot outlive the worktree,
  whatever removes it,
  so no proof about absence is needed at all.
- Nothing is compared and nothing is waited for;
  the deletion Git performs is the settlement.

Cons:

- Git's removal deletes the record *before* anything can read it,
  so the stage's name goes with it,
  and the folder becomes something nothing names:
  a stage of a whole checkout,
  with credentials in it,
  standing beside the worktrees with no record and no line.
  That is strictly worse than today for the folder.
- A worktree removed by `rm -rf` leaves its administrative directory behind until `git worktree prune`,
  so the record survives the folder and the promise is only kept for Git's own removals.
- The add writes its first record before Git runs,
  when the administrative directory does not exist,
  so an add needs two records and the hand-over between them,
  and that gap is one more window for a kill.

What a person sees:

```text
$ git -C checkouts/other status --porcelain
# (nothing on standard error, for ever)
$ ls /work/repo/.claude/worktrees
.cli-git-worktree-copy-e34a3581-2b9a-4e93-91b4-8ac49576ba91  other  …
# and no command of the wrapper ever names that folder again
```

### Option 4: expire what nothing has claimed for a bound

What it does:

- A claim whose owner is dead,
  whose destination is not a registration of this repository,
  and whose records are unchanged for longer than a bound,
  is settled on the next recovery,
  with the stage handled by option 1's comparison or left to option 2's command.
- The bound is a named constant with its reason beside it,
  in the shape Git uses for `gc.worktreePruneExpire`.

Pros:

- A removal by any means ends the obligation after a fixed time,
  so a person who removed a worktree and re-added one within the bound keeps the option of the copy.
- It needs no new proof about a worktree's absence,
  only the absence plus the passage of time.
- The bound is one figure to tune if a stage ever turns out to be worth keeping.

Cons:

- Every line keeps coming for the whole bound,
  which on this host is the failure the issue reports,
  only with an end date.
- It adds a durable instant that has to be written somewhere,
  and the file's own modification time is not one:
  a scan or a backup must not extend a stage's life,
  and nothing today records when the worktree went.
- With option 1's comparison available,
  the bound adds a wait to a question that can be answered now,
  and it still leaves the folder behind for that wait.

What a person sees,
on the day the bound passes:

```text
$ git -C checkouts/other status --porcelain
monochromatic-git-policy-cli: settled the interrupted `git worktree add` of "/work/repo/.claude/worktrees/gone-topic", whose worktree has been gone since 2026-10-30; its stage held nothing its source no longer does, so it was removed.
```

### Option 5: say less, and only what is true

What it does:

- The owed-copy pass settles a claim whose destination is gone or replaced,
  exactly as the unfilled record already settles,
  and prints nothing about a worktree that is not there.
- The stage line,
  its rule and its repetition stay exactly as they are,
  and the line gains one sentence saying the worktree it belongs to is gone,
  so a reader knows the folder is the only thing left.

Pros:

- The smallest change that makes both lines true,
  and the only one that needs no new proof,
  no comparison and no new command.
- The worktree line ends with the worktree,
  which is the first half of the issue,
  on the next command.

Cons:

- The stage line repeats on every command for as long as the folder stands,
  which is the louder half of the measurement:
  25 of the 62 lines one command printed here were stage lines,
  and nothing in this option ends them.
- It leaves the second half of the issue exactly as it is:
  a person is still told,
  on every command,
  to remove a folder by hand.

What a person sees:

```text
$ git -C checkouts/other status --porcelain
monochromatic-git-policy-cli: the stage "/work/repo/.claude/worktrees/.cli-git-worktree-copy-e34a3581-2b9a-4e93-91b4-8ac49576ba91" was left by an interrupted `git worktree add` whose worktree is now gone, and was not changed: no record of the object its creation made stands beside its claim; remove it yourself once nothing needs it (`btrfs subvolume delete` for a snapshot, `rm -r` otherwise).
```

## Ranking

Ranking:
1 > 2 > 5 > 3 > 4.

1 above 2:
both end the line about the worktree on the next command,
and 1 also ends the folders,
which is the half a person who was not watching cannot do for themselves.
2 leaves 20 folders,
42 628 MiB of them by `du` at 2026-11-27T03:56:10-05:00,
standing until someone notices a line that was printed once,
and what it costs on a loaded host is the same walk of the folder that 1 does,
moved into a command nobody has run.
For a person who did not watch the add,
a folder nothing removes is a folder nothing finds.

2 above 5:
both settle the claim on the same proof and cost the same there,
and 2 adds the one thing 5 lacks,
an end to the repeated line and a way to act on it.
5 leaves 25 lines per command on this host for ever,
which is what made the issue necessary in the first place.

5 above 3:
both end the line about the worktree without a person's word,
and 5 keeps every folder named and findable.
3 ends the record by letting Git delete it,
which also deletes the only name of the stage,
so the folders stand with no record and no line,
and the 42 GB become invisible rather than noisy.
A folder a person can read about and remove is better than one nothing names.

3 above 4:
both leave the folders where they are for now,
and 3 ends the line about the worktree at once,
while 4 prints it until the bound passes.
4 is 1 or 2 with a delay in front of it,
and the delay buys only the chance to re-add a worktree within the bound,
which is not the failure this issue reports.

## What only the owner can answer

1. Is a folder whose every entry its source still holds byte for byte safe to remove without asking?
   This is the line between option 1 and option 2,
   and it is the question issue 6296's reaping already raised once.
2. Should the stage line be printed once per folder,
   or on every command as it is today?
   The owner chose "prints again from every command" for the unfilled record;
   whether that applies to a folder a person must remove by hand is a different call.
3. Should the wrapper look for stage folders that no claim names?
   Today nothing does (`SPEC.md`,
   "A record that cannot be read":
   "The folder is not work:
   a command does not sweep for it"),
   so a folder whose claim was settled while a journal named it is invisible.
4. What should settle the claims and stage folders standing in this checkout now?
   They were 18 claims and 20 stage folders at 2026-11-27T03:42:38-05:00,
   and the figures move as other sessions run commands.
   Nothing here was deleted or settled by this round,
   and no release can safely guess about a folder whose source may hold the only other copy.
5. Does the launcher's add keep dying before it records its binding?
   The claims in this checkout's first listing were written between 2026-11-27T02:38 and 02:56,
   and every one of them was `phase=planned` with a dead owner,
   which is one more cause of the same line and belongs to issue 6418 and issue 6339.
