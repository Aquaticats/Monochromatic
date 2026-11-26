# Pi round start failures, and what would stop them

## Status

Proposal,
written 2026-11-26T18:05:00-05:00 by the coordinating Claude Code session,
after the owner asked:
"What more bad things would I expect to see about how you use Pi?
How do we prevent them from ever happening again?"
Unapproved and non-final:
nothing in "Proposed prevention" is built,
and the owner decides each item.
A Claude Sonnet read of this proposal is pending.

## What has gone wrong, by cause

### Limits an agent made up, later read as the owner's

On 2026-11-26 the coordinator held writer rounds to five at once,
three minutes apart,
behind a memory test.
The owner:
"Just because the 1st pool is 5 at a time doesn't mean you gotta limit yourself."
and
"I didn't tell you to apply the 3 minutes apart and memory test rules."
The spacing and the memory test came from a session's own reading under the owner's "Up to 64 rounds at once."
(`doc/decision/zfs-route-later-pieces-built-ahead-unwired-and-up-to-64-rounds-at-once.md`, "What it means")
and from the coordinator's handover section "Memory, and how many rounds build at once"
(`doc/handover/cli-git-rust-implementation.md`).
The coordinator's compaction summary then listed both among the owner's standing directions,
and the next context applied them as the owner's.

The same shape is on record before:

- `doc/handover/agents-at-once-on-this-machine.md`:
  the agent cap of 20 read first as no cap and then as a cap,
  and the owner's "We talked about this at least 3 times."
- `doc/decision/pi-driver-and-provider-owner-answers-2026-11-09-to-2026-11-11.md`:
  "Why did you downgrade everything to 6 and 7?"
- `doc/decision/cli-git-commit-protocol-owner-answers-of-2026-11-03-and-2026-11-04.md`:
  "Why did you cap the commit service to the magic number of 1GB of memory w/o evidence?"

`AGENTS.md` already asks a wait to name the failure it prevents,
and a brief to carry only what the owner decided;
neither stopped this,
because nothing marks an agent's choice as an agent's once it is written down.

### Provider choice held in private launchers

- 2026-11-23:
  launchers that call `drive-pi-subagent.ts` directly put writer rounds on standard Neuralwatt,
  which the owner had closed
  ("This shouldn't happen.",
  `.claude/skills/pi-as-subagent/SKILL.md`).
- 2026-11-26 about 02:35:
  three rounds pinned to opencode-go ended on its spent monthly quota while Qwen and the virtual writer sat unused
  ("What? Don't you have a virtual writer model in Pi? And what of my Qwen quota?",
  `doc/decision/writer-rounds-take-qwen-first-through-the-virtual-writer.md`).
- 2026-11-26 about 17:31:
  two rounds the coordinator pinned to opencode-go as overflow each died on their first request with
  `429: {"type":"GoUsageLimitError"`,
  a refusal the virtual writer passes on to flex because it comes before any output
  (`package/pi-plugin/rate-limit-fallback/README.md`).

The launchers live in `~/temp/agent/zfs-open-item-design-rounds-launch-2026-11-22/`,
outside the repository,
with no tests and no review,
and other sessions never see them.
`writer-pool-for-a-round.ts` there still says opencode-go comes first and still offers `WRITER_POOL` pinning.
Calling the driver directly also skips its pool gate,
so the driver's own count never applied to these rounds.

### Runs waiting on things that are not limits

- Each split chain ran one round at a time for hours:
  a shell loop started each round's starter only after the round before it ended.
- A question dialog held the coordinating session for about three and a half hours on 2026-11-26,
  while rounds ended and their merges waited.
- A finished round merges only when the coordinator reads it and runs the merge check,
  so a busy or blocked coordinator delays every merge.

### What rounds do inside

Rounds repeat their test runs to show a failure or a fix:
issue 6518's round ran one module 60, 30 and 20 times,
and issue 6520's round ran 30-run loops,
although each brief quoted the synthetic-load ban.

### Briefs that state the coordinator's design as the project's

The driver round's brief
(`~/temp/agent/subagent-driver-counts-a-virtual-writer-run-once-against-its-members-combined-room-2026-11-26/prompt.md`)
tells it to count a virtual-writer run once against its members' combined room.
The owner asked not to be limited;
the combined-room count is the coordinator's design,
which the brief states as the requirement.

### Bookkeeping

The session task list showed rounds as pending while they ran (6521, 6522, the follow-up refusals round).

## What to expect next if nothing changes

- A new wait or cap after the next compaction,
  because summaries carry agent choices beside the owner's words.
- Another round on a spent or closed pool,
  while any launcher can pin one.
- Finished rounds left unmerged while the coordinator waits on a dialog or on its own reading.
- More repeated test runs inside rounds.
- Merge conflicts between split rounds working in one folder at once;
  the briefs tell a round to leave a file another branch changes,
  so these are a cost of running them together,
  not a failure.
- Memory:
  64 rounds compiling at once on 63 GiB;
  the desktop freezes in `doc/troubleshooting/desktop-freezes-while-many-agent-runs-thrash-zram-swap.md` came from memory.
  No start waits on memory now,
  by the owner's word;
  the coordinator reports the readings.

## Proposed prevention

Each item is enforced by code or a hook where it can be,
because the prose rules already existed and were broken.
They are complements,
not alternatives;
the order is the proposed build order.

### One start command in the driver package

A command in `package/pi-shared/subagent-driver` that does what the private launchers do:
make the worktree and round folder,
start the run as its own capped unit,
keep its report.
It starts every writer run on the virtual writer and has no option to pin one,
starts at once and detached,
and refuses only past the owner's 64 at once.
The private launchers are deleted,
and a Claude Code hook refuses a direct `drive-pi-subagent.ts` call outside the package's own commands.
This removes the pinning,
the drifting launcher text
and the serial chains in one change,
and puts the start under the package's tests.

### Merge when a round ends

The same command,
when a round ends idle,
runs the round's merge check and merges a branch whose failures are none of its own,
the way `check_one` does by hand today,
and writes what it did beside the round.
A blocked or busy coordinator then delays no merge.
The coordinator still reads every report.

### The guardrail refuses repeated test runs

`package/pi-plugin/guardrail` already refuses Bash calls inside Pi.
It would also refuse a loop or repetition around a test command
(`for`, `while`, `seq`, `--stress-count` around `cargo test`, `cargo nextest` or `mise run`),
answering with the synthetic-load ban and how to make a rare failure common instead.

### Owner words and agent choices kept apart in writing

An `AGENTS.md` change:
a limit or wait an agent adds is labelled as the agent's wherever it is written,
beside the failure it prevents,
and a handover or summary lists the owner's quoted words apart from agent choices;
a standing direction is only an owner quote with its date.

### No dialog while work is ready

An `AGENTS.md` change:
before opening a question dialog,
start every ready run and merge,
since the dialog blocks the session.

### Briefs keep the coordinator's design as a suggestion

An `AGENTS.md` tightening:
a brief's requirements quote a decision,
and the coordinator's own design goes under a heading that names it as a suggestion,
or is asked first.

### Ranking

One start command > merge when a round ends,
because the start command is where both the pinning and the serial waits live,
and merging on end is built inside it.
Merge when a round ends > guardrail,
because unmerged rounds cost the whole queue,
while a test loop costs one round's time and host load.
Guardrail > owner words apart,
because the guardrail refuses mechanically,
while the writing rule depends on every session keeping it.
Owner words apart > no dialog while work is ready,
because made-up limits recur across every kind of work,
while the dialog case is one tool.
No dialog while work is ready > briefs keep design as suggestion,
because the dialog delayed every merge for hours,
while a brief's design reaches one round.

## Open question for the owner

How the driver's pool gate counts a run on the virtual writer,
for starts that pass through it:

- No count:
  the router moves a refused request on by itself,
  so the gate counts only runs pinned to one pool.
- Once against the open members' combined room
  (the driver round's current brief).
- As now:
  a slot in every member,
  which stops at Qwen's 5.
