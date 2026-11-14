# Removing the whole-manager reload from the Git wrapper's commit path

Proposal and measurements,
2026-11-12T08:20:00-05:00.

Status at 2026-11-12T10:30:00-05:00:
**built and measured**,
on branch `fix/git-policy-cli-wrapper-never-asks-the-user-manager-for-a-manager-wide-reload`,
as option 1 with one addition that document did not list:
the shared template names a per-user program link,
and a unit the manager already holds at an older text is dropped and started again instead of reloaded.
Nothing on a Git command's path asks the manager to re-read its units any more,
so the question "which of the three directions" is answered by the measurements below rather than by a preference.
What was measured,
what was chosen,
and what the migration from the units an earlier release left behind costs are recorded under
"What was built,
and what it measured".
The proposal itself is kept as it was written.

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

## What was built, and what it measured

Measured on 2026-11-12 between 09:00 and 10:25-05:00,
in containers of their own,
with a private session bus (`dbus-daemon`),
a `systemd --user` of systemd `262.1-1-arch`,
and an unrelated `Type=dbus` service (`dbus-test-tool echo`) bound by `BindsTo=` to a target,
which is how the session's own units are bound to a window manager.

### The alternation, on the code as it was

Two wrapper executables at two paths (`/opt/release-a/git`,
`/opt/release-b/git`) in one machine,
each commit of one worktree through one of them in turn,
then a second worktree,
then four callers at once,
with the manager holding 4,059 loaded units.

With the code this branch started from (`main`,
release `0.6.1-alpha.13`):
the manager was asked to re-read its units **seven times over those steps**,
once per commit that ran through the other executable,
each reload taking 976 to 1,245 ms,
and the unrelated `Type=dbus` service and its target ended at **the first** of them and stayed ended.
Every commit still landed.

With the code on this branch,
on the same steps:
**no reload at all**,
and the service kept its bus name and its target stayed active after every step.

The proof is the container test
`mise run //package/git-policy/cli:test:service-manager`,
whose arm from `main` has to end the unrelated service and its arm from this branch has to keep it.
It also runs the arm from this branch on a machine that the release from `main` provisioned first,
which is the migration,
and that arm asks for no reload either.

### What the merge with main kept, and why

Both sides removed the reload from a release change,
and both answered it with a program link:
main's is `service_provision_program_pointer.rs`
(`$XDG_RUNTIME_DIR/systemd/user/monochromatic-git-policy-cli-worktree-notes/service-program`),
and this branch's is `program_link` in `service_instance.rs`
(`$XDG_RUNTIME_DIR/monochromatic-git-policy-cli-bin`).
One stays.

The branch's stays,
for two reasons.
It is the one the container proof above measured:
seven reloads over the steps on `main`'s code of that morning and none on this branch's,
and the arm that keeps the unrelated desktop service alive is that code.
And it writes the link through `link_atomically`,
the same temporary-name-and-rename path the unit files are written through,
so one file of this repository owns how anything the manager reads is replaced.
Main's module and its controls went;
the one control they held that the branch's did not
(a replacement killed between its temporary link and its rename leaves the old link whole)
was ported to the surviving link.

From main,
kept because they are better than what the branch had:

- A manager that does not answer within one exchange never fails the commit.
  The units stay written,
  one line says what the wait was for,
  and the next commit asks again.
- The line a commit prints when the manager has not answered its provisioning within a second.
- The one ending of a service of another release that outlasts the bound it was given
  (`RestartUnit` of this worktree's own unit),
  which is what keeps a change of release from ending uncommitted.

What that port cost,
and found:
the record of the unit texts the manager was told about is now written only by a call that proved what the manager holds
(the new `Unloaded::Dropped` of `service_provision_instance_unloaded.rs`),
because main's control for a manager that does not answer showed the branch writing it after a question the manager
never answered.
A start on its own is no proof either:
a manager that could not say what it holds may have started the unit from a text an upgrade left behind.

What this means for a machine that installed `0.6.1-alpha.14`:
its template already names a link,
but a different one,
and its record is the older one-file shape (`units-the-manager-was-told`) that this release never reads.
So its first commit of this release rewrites the text and drops this worktree's unit once,
exactly as a machine carrying `0.6.1-alpha.13`'s template does;
neither pays a reload.
Keeping main's link path would have spared the rewrite but not the drop,
because the record's shape changed with it.

### Where a slow reload spends its time

A reload of a disposable user manager,
three samples per point,
the manager holding loaded transient services (each one active,
no process):

- 59 loaded units:
  61,
  68,
  63 ms.
- 559:
  194,
  297,
  199 ms.
- 1,059:
  355,
  346,
  358 ms.
- 2,059:
  646,
  639,
  610 ms.
- 4,059:
  1,142,
  1,061,
  1,086 ms.

That is about **0.26 ms per loaded unit plus 50 ms** of fixed cost,
and the same manager with a user generator that sleeps 500 ms reloaded in
1,651,
1,597 and 1,649 ms:
**the generator's whole running time is spent inside the reload**,
because the manager waits for it on its own event loop,
and with the generator removed the same manager took 1,068 to 1,106 ms again.

So the number of units the manager holds does set a floor for a reload,
and the machines' own generators and its storage sit on top of it.
The install's reload on this machine at 08:40:00-05:00 was 487 ms with 319 units,
which is 3.7 times what the 0.26 ms per unit of a disposable manager predicts,
so a real manager's reload is not unit count alone.
The 139,770 ms of the incident would need about 540,000 units at that rate,
so it came from something else on top of the count,
and this measurement cannot say what:
the journal has no per-phase timings for it.

What the count points at for the rest of this repository:
every agent command that runs in a transient scope of its own makes every later reload of that manager more expensive,
so a reload asked for by one tool is paid by all of them.

### What was chosen

The template names `$XDG_RUNTIME_DIR/monochromatic-git-policy-cli-bin`,
a link each commit points at the release it is running,
so an upgrade,
a downgrade and two releases used in turn write no text the manager read.
That is option 1.

What option 1 alone does not answer is a machine whose manager already holds the earlier text,
which is every machine that has an earlier release installed,
and that is where the fourth direction was looked at:

- A transient unit (`StartTransientUnit`,
  what `systemd-run` does) needs no file and no reload.
  What it would not give,
  read in systemd's own source and measured in the same container:
  - `StartTransientUnit` refuses a name the manager already holds or has a fragment for
    (`org.freedesktop.systemd1.UnitExists`),
    so the earlier release's installed unit has to be stopped and dropped before a transient unit of the same name can
    exist,
    which is the same work the chosen direction does;
  - a transient socket unit needs its service to be loadable
    (`Socket service X not loaded, refusing`),
    so the two cannot be created independently,
    and the file-based template is what makes a socket's service always resolvable;
  - `systemctl --user status` shows a transient unit as well as a file one,
    but nothing of a transient unit survives a restart of the manager,
    so the wrapper would have to create it again on the next command.
  It removes the reload but not the work of getting rid of the earlier release's unit,
  and it trades a file the manager reads for a file the manager writes under `systemd/transient/`.
  It was not built.
- Option 2 (one unit file per worktree) still needs the manager to read a new text for a worktree whose release path
  changed,
  and the link makes that unnecessary for every worktree at once.
- Option 3 (move only the waits) leaves the manager-wide stall that ended the session.

### The migration from the units an earlier release left behind

A machine with an earlier release's template installed has a service unit whose text is that release's own:
the executable of the release that wrote it (`0.6.1-alpha.13`),
or a link of that release
(`0.6.1-alpha.14`),
and the manager may hold it.
The first commit of this release rewrites the template,
asks a running service to retire (so its batch is finished),
stops this worktree's socket (which stops the service that requires it),
asks the manager what it holds of the worktree's units until it holds neither,
and starts the socket again,
which is what makes the next start read the file as it is now.
Measured in the disposable manager holding 4,059 units:
the manager held neither unit **11 ms after the stop was answered**.

Cost:
one stop and one start of this worktree's own socket,
and one retirement of a service that was going to be retired anyway when its release answered a command of this one.
It happens once per worktree on the first commit after the change,
and again only for a release that changes the unit text itself,
which the link exists to prevent.

### What the merged code measured

On 2026-11-13 between 12:18 and 12:23-05:00,
load average 33,
in the gate's own containers.
The gate image of the day before was gone with the machine's restart,
so the image is the gate's base
(`mise run //package/git-policy/cli:test:git-image`,
`localhost/git-policy-native-test-base:bb4ebaceda61-97221c29742b`),
which carries systemd `262.1`,
`dbus-daemon`
and Git.

**The proof,
twice**
(`mise run //package/git-policy/cli:test:service-manager -- --units 1000 --probe-deadline 100ms …`,
each machine holding 1,059 units),
with the control arm running the release from `main` (`0.6.1-alpha.15`)
and the other arm this branch's build:

- A machine carrying `0.6.1-alpha.13`'s template
  (`ExecStart=/opt/release-seed/git …`,
  the executable of the release that wrote it):
  the arm from `main` asked for **one reload of 304 ms**,
  and the unrelated `Type=dbus` service and its target **ended at the first step after it and stayed ended**;
  this branch's arm asked for **no reload**,
  the stand-in **never ended**,
  and every commit landed.
- A machine carrying `0.6.1-alpha.14`'s template
  (`ExecStart=/run/user/0/systemd/user/monochromatic-git-policy-cli-worktree-notes/service-program …`,
  main's own link):
  the same readings,
  one reload of 308 ms and the stand-in ended on `main`,
  no reload and no death here.

The seeding of the control arm is the older template,
because the release from `main` asks the manager to read every unit only when a unit *text* changes:
that is the first commit after a machine carries an earlier release's template,
and never a release change on a machine that release provisioned itself
(the measurement below shows a release change asking for no reload on `main` too).
A control arm on a fresh machine therefore reproduces nothing,
and the proof says so instead of calling such a run a pass.

**A service that never greets,**
on this branch's build (a stand-in program that holds this worktree's socket and says nothing on it):
the commit took **121,788 ms** and exited 1,
asked the manager for no reload,
and printed,
in order:

```text
nothing has answered at …/1d64ab6f….sock yet (the service manager says …@1d64ab6f….service is active)
the commit service of this worktree has answered nothing for 60 seconds, so it is stopped inside its own process; it is asked to restart (…@1d64ab6f….service), and this commit goes on once it answers
The commit service did not start, so this commit was not made. Its log shows why: `journalctl --user --unit 'monochromatic-git-policy-cli@*.service'`. Run the commit again after fixing that. Nothing in the repository changed.
```

That is the trade the three-second rule's removal makes:
two whole start ceilings (120 s) before a commit gives up on a service that is truly wedged,
where the releases before this one ended such a commit in about three seconds
and,
on 2026-11-12,
asked for the reload that ended a desktop session.

**The first commit after a release change,**
on a disposable repository in the same containers,
against the release from `main`:

- A release change on a machine the release provisioned itself
  (the same build at a second path,
  which is a build in a worktree):
  **58 ms** and no reload on this branch,
  **51 ms** and no reload on `main`.
  Both print only the commit's own summary line.
  Main's program link is why this case asks for nothing on both arms.
- The first commit of a release on a machine an earlier release set up:
  **162 ms** and **no reload** on this branch,
  **557 ms** and **one reload (150 ms)** on `main`,
  which printed `Updating the commit service...` before its summary.
  This is the incident's own step,
  and the manager's own record shows the difference:
  one reload in the machine's whole life here (the fixture's own,
  before any arm ran),
  two on `main`.

### The review of the first change, and what it found

A review of `1cb47223f8` reproduced three defects against the real `provision_with`,
all of one family:
the unload of one worktree's units turned a manager that was slow,
or a service that was still landing,
into a
 failed commit,
where the design's own handling elsewhere let the commit go on.
Its reproduction is a crate that calls `provision_with` with a scripted `Manager`,
built from the reviewed source;
run against the fixed build it shows the healthy control passing and none of the three scenarios,
each one line on the wrapper's log instead.

- **A manager that answered the stop and then stopped answering** failed the commit with
  "This account has no user service manager ... `loginctl enable-linger`",
  which is false of a manager that answered a moment before.
  The same file already let the commit go on when the manager could not be asked for the two state questions,
  and the start phase already tolerated a timeout;
  the stop,
  the wait for the service to leave and the wait for the unit to be dropped did not.
  Now every one of them does:
  each answers that nothing was dropped,
  one line says so,
  the record stays unwritten so the next commit asks again,
  and the commit lands through whatever the manager holds.
- **A service landing a batch** was asked to retire and given the retirement wait (10 s) for it,
  while a landing may hold the index for `LANDING_LOCK_CEILING` (60 s);
  at the wait it failed the commit.
  The first commit of a worktree after an upgrade takes that path.
  Now a service still there at the wait is left running,
  the commit goes on through it,
  and the next commit drops the unit once the service is at rest.
  Nothing is stopped,
  so a landing is never cut,
  and the commit waits nothing extra for the landing:
  it is the service that lands it.
- **A program link that cannot be written** failed the commit,
  which its own doc promised it never would.
  Now it is one line naming the link,
  the units are still written,
  and a start runs whatever the link named before,
  which is this release's program while the link names it.

The one refusal kept is a manager that answered every question and still holds the unit `UNLOAD_WAIT` (5 s) after it
was stopped;
its line now says what is true of it,
that a start would read the text it loaded rather than the files,
and no longer that it "would run the release that was replaced":
the `ExecStart` of any release since `0.6.1-alpha.14` names the program link,
which every commit points before it touches the bus,
and the old text names the release that wrote it only when it is older than the link,
which the client's own repair for a service of another release answers.

The sweep of a removed worktree left the record of the unit texts behind while the record's own doc said the sweep
removes it;
it removes it now,
which is what keeps a worktree that is gone from leaving a file that says what the manager read.

**What the repair costs a commit,**
measured on the scripted manager with the shipped retirement wait (10 s) and,
for the
last row,
a wait of 1 s so the control reads fast:

- service leaves at once:
  **0 ms** and the commit goes on,
  before and after;
- a landing of 250 ms:
  **251 ms** and the commit goes on,
  before and after;
- a landing longer than the wait:
  **the wait itself (1,002 ms)**,
  and the commit failed before and goes on after.

So the fix adds no time to any path;
it changes what the end of a wait means.

### What is left

- The tile of `SERVICE_GREETING_SILENCE` that this document was written with is gone:
  a greeting that has not come is no longer read as a service stopped inside its own process
  before the whole start ceiling.
  The measurement behind it (52 cold starts,
  the slowest 1,041 ms,
  on an unloaded machine) is what the incident showed
  to be too narrow for a loaded one.
- Nothing here changes the desktop's own units,
  and no upstream systemd report is proposed.
