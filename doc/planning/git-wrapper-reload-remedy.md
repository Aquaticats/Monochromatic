# Removing the whole-manager reload from the Git wrapper's commit path

Proposal and measurements,
2026-11-12T08:20:00-05:00.

Status at that time:
no option is chosen.
Asked which of the three directions to build,
the owner answered that they were not sure and that this session's job was to investigate and write up,
with the work handed to another agent.
So nothing below is accepted,
and nothing is implemented.

Issue:
[GitHub issue 6286](https://github.com/Aquaticats/Monochromatic/issues/6286),
"global user-manager reload during service recovery can terminate the desktop session".
The evidence,
source excerpts,
and the finished reproductions are in
[the incident investigation](../troubleshooting/plasma-session-ended-after-service-manager-timeouts.md).

## What is established

- The wrapper asks the shared user manager for one global reload inside the commit path,
  from `ask_manager` in `package/git-policy/cli/src/native/service/service_provision.rs`.
  It is asked for only when a unit the manager may already hold changed or went away,
  or when the record of what the manager was told does not match the files.
- The reload re-reads every unit of the user manager,
  and the retained journal's 208 reloads have a median of 1635 ms,
  40 longer than 3000 ms,
  12 longer than 10000 ms,
  and the incident's 139770 ms longest.
  The requesters are the wrapper (161),
  `git` (41),
  and `systemctl` (6).
- A manager that is reading every unit can pass a unit's start deadline before reading a reply the bus already
  sent.
  sd-bus expires that reply,
  and the manager reads the expiry as a lost bus name,
  which stops a running `Type=dbus` unit.
  A disposable manager held inside its reload reproduced exactly that,
  with the successful subscription reply captured on the bus first.
- The wrapper's ceilings are sized against cold starts,
  not against a wait that includes a reload:
  `START_CEILING` is 60 seconds against 52 cold starts that greeted in at most 1041 ms,
  and `SERVICE_GREETING_SILENCE` is 3 seconds
  (`package/git-policy/cli/src/native/service/wait_ceilings.rs`).
  A reload longer than three seconds,
  which 40 of the 208 were,
  already outlives the silence the wrapper reads as a
  service stuck inside its own process.
- The shared template in `/run/user/1000/systemd/user/` embeds the absolute path of whichever executable ran the
  commit (`unit_texts`),
  so two releases,
  or a release and a worktree build,
  write different texts to the same file.

## Not established

- Whether different executables really alternate in practice on this machine.
  A fifteen-minute watch of the live template found no rewrite and no reload,
  so the fixture has to provoke the alternation instead of waiting for it.
  The live template names
  `/var/home/user/.local/share/monochromatic-dev/toolchain/cargo-monochromatic-git-policy-cli/0.6.1-alpha.12/...`.
- Which phase a slow reload spends its time in.
  The journal has no per-phase timings,
  and no measurement assigns any share to storage,
  generators,
  or enumeration.

## The options

### Option 1: name a stable program link in the shared template

`unit_texts` would name a per-user link,
for instance
`$XDG_RUNTIME_DIR/monochromatic-git-policy-cli-bin`,
which the wrapper points at the release it is running,
instead of the release's absolute path.

Pros:

- The shared text stops changing when a release is upgraded or when two callers resolve different release paths,
  so those paths stop reloading the manager at all.
- One file's text and one link update,
  no new unit files and no per-worktree bookkeeping.
- A running service keeps its process;
  the next start resolves the link to the new release.

Cons:

- The link becomes part of the wrapper's contract and needs its own update and failure path.
- A first install and the retirement of earlier releases' units can still ask for a reload.
- Two releases running at the same moment point the link at whichever wrote it last,
  so a start in that window runs the other release.
  That is the same executable the manager would otherwise have been reloaded for,
  but it is now implicit.

### Option 2: one unit file per worktree

Instead of one shared template plus per-worktree instances,
the wrapper would write `monochromatic-git-policy-cli@<instance>.service` per worktree.

Pros:

- No cross-worktree coupling by construction:
  one worktree's release never rewrites another worktree's unit.
- A new worktree needs no reload,
  because a unit file the manager never held is found when the unit is first started.

Cons:

- A file per worktree to create,
  own,
  and retire,
  on top of the sweep the wrapper already runs for removed worktrees.
- A worktree whose own release path changes still needs a reload for its own file.
- More code than option 1 for the same trigger.

### Option 3: keep the reload and fix only the waits

The reload stays,
and the wrapper's reading of a slow start changes:
the greeting silence becomes measured against the reload tail rather than against cold starts,
and the reload leaves the blocking part of the commit.

Pros:

- No unit layout change at all.
- The wrapper stops reading a slow start as a service stopped inside its own process.

Cons:

- The manager still stalls every other client while it re-reads the units,
  which is how KWin lost its bus name and how the session ended.
  It changes our reading of the symptom,
  not the trigger.

## Ranking

1 > 2 > 3.

- 1 over 2:
  it removes the reload trigger from the upgrade and alternation paths with one file's text,
  where 2 needs per-worktree file lifecycle work and still reloads for a worktree whose release changes.
- 2 over 3:
  2 ends the cross-worktree rewrite storms,
  and 3 leaves the manager-wide stall that stopped the desktop.

## What a decision would need

- The alternation fixture:
  two wrapper executables,
  one per path,
  driven against one isolated user manager,
  with the template sampled and the manager's reload records read afterwards.
  The harness for the systemd side already shows how to own a manager in a container;
  the wrapper side additionally needs a user manager,
  a session bus,
  and a scratch repository.
- A phase breakdown of a slow reload,
  measured in a disposable user manager with a representative unit set.
  Without it,
  no option can be costed against the reload's real cost.
- The owner's answer on whether the wrapper may keep a per-user link or must keep one file per worktree,
  which is a shape decision about the wrapper's own layout,
  not a bug fix.

## What this document does not propose

- No change to KDE's units and no change to how the desktop binds its session to a `Type=dbus` unit.
- No upstream systemd report.
  A candidate repair was verified in a disposable clone and is recorded in the incident document;
  filing it is unauthorized and would need the owner's word.
