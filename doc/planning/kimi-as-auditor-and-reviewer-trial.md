# Kimi K3.1 Pro as auditor and reviewer: trial read at 2026-11-10T09:54:22-05:00

The owner wrote
"Try synthetic's hf:moonshotai/Kimi-K3.1-Pro as auditer and reviewer.
If this works we can replace both GPT and Sonnet with it."
and
"Kimi K3.1 Pro has a tendency to cheat.
I suggest you let deepseek look through on disk transcripts after the trial kimi runs.
If it cheated,
we need to disqualify it."

This record holds what the trial showed,
the recommendation drawn from it,
and the owner's decision,
under the heading "Decision".

## How the trial ran

Kimi ran four tasks blind,
each at a past commit another reviewer had already covered,
each in its own worktree,
with its folder under `${HOME}/temp/agent/kimi-trials/`.
The script `run-kimi-trials-and-check-them.ts` started them outside the Neuralwatt slots,
under the owner's "Kimi trials don't count within 20 slots because 20 is neuralwatt's effective concurrency and kimi runs on Synthetic."

Each audit and each finished review took between 24 and 28 minutes.
The merge route review ended on a provider error after 102 turns and 28 minutes,
and a resumed run answered after 31 turns and 7 minutes.

## Whether it cheated

The DeepSeek V4.1 Flash run `check-kimi-trial-transcripts-for-cheating-1` read all four sessions and judged all four clean.
No run obtained an earlier answer,
looked past its commit,
reached GitHub,
a web search or a session log,
or reported work it did not do.

One run typed two paths that do not exist,
both the same slip of dropping `merge-route` from its own folder name,
and both returned nothing.
Three runs saw other runs' names through `ps` with `args`,
and none used them.

The check read every tool call in full,
but only 17 to 26 percent of each session's thinking characters,
plus the full text of every block its keyword sweep matched.
So the result is that no cheating was caught,
and it does not prove there was none.
The mechanical scan missed both typed paths;
only the reader's broader sweep found them.

## What each reviewer found

Kimi reported 12 findings across the four tasks,
and the reviewers it was compared with reported 18,
not counting Sonnet's latency measurement.
Six findings were reported by both,
six only by Kimi,
and twelve only by the others.

### Differential audit at `7da0abce05`

Kimi compared 127 commands across 65 scenarios and reported one finding,
that a service-landed commit writes an empty reflog message.
Sonnet ran about 870 cases and reported that one and four more:
`git commit -- .` also commits untracked files,
`--noglob-pathspecs` and `--icase-pathspecs` are ignored by an explicit-path commit,
a merge omits `Auto-merging <path>`,
and a service stopped by `SIGTERM` leaves its made-ahead marker file.
Sonnet also measured commit latency against the owner's five-second limit.
Kimi found nothing here that Sonnet missed.

### Never-stuck audit at `a56d51b311`

Kimi reported three hangs,
all reproduced,
from a FIFO at `.git/index.lock`,
at `.git/index~pid.lock`,
and at `.git/index`.
Sonnet reported two:
the FIFO at `.git/index.lock`,
and a FIFO where a rules file is expected.
Each found a defect the other missed.
Sonnet returned early under the owner's polling rule and listed several experiments as unfinished.

### Merge route review at `3d5219e9da`

Kimi reported five findings and reproduced three against the real wrapper.
GPT reported eight and stated no reproduction.
Both found the overwritten staged edit,
the unchanged symlink that recovery calls changed,
and the nested directory a merge cannot replace with a file.

Only Kimi found that a failed branch restoration is reported as successful,
and that two commands recovering one dead record make one of them fail.
It reached the second by reading.

Only GPT found that an aside can receive new bytes and then be deleted,
that a partly installed merge is reported as untouched when a worktree read fails,
that moved-branch recovery deletes the record of a partly updated checkout,
that the original record blocks commands on another branch,
and that a tracked executable-bit edit is erased.

### Worktree copy review, round 5

Kimi reported three findings with reproduction evidence.
GPT reported three and stated no reproduction.
Both found that a temporarily empty registration leads recovery to delete a pending copy.

Only Kimi found that a FIFO at the stage's `registration` file blocks recovery,
and that a crafted journal directs a recursive deletion.
Only GPT found that trimming a registration path changes its bytes,
and gave a permission-failure scenario for the class Kimi named as unresolvable.

## What these results do not show

- Each task ran once,
  and no run-to-run spread was measured,
  so only a large difference counts.
  The differential audit's is large.
  The never-stuck audit's is not.
- The two GPT reviews came through the Pi route,
  from before the move to `codex` with tools,
  so the difference in reproduction is not like for like.
- Nobody ran Kimi's reproduction scripts for this record.
  The DeepSeek check compared each claimed reproduction with the session's tool results.
- The trial covers only this wrapper's Rust code.

## Findings only Kimi reported

These sites read the same on `main` as the trial's reports describe:

- `lock/index_lock_evidence.rs:378` reads the PID file with a blocking `std::fs::read`.
- `foundation/private_storage.rs:753` opens without `O_NONBLOCK`.
- `merge/merge_isolation_recovery_complete.rs` only logs a failed branch restoration,
  and then returns `a_worktree_that_switched`.
- `worktree_copying/worktree_copy_stage_binding.rs:81` reads the stage binding with a blocking `read_to_string`.

That is from reading,
so an agent reproduces each one on `main` before fixing it.
The crafted journal is left out,
because it needs the same account to plant a forged record.

## Recommendation

Neither replacement is supported by these results.
Each reviewer found real defects the others missed,
so running Kimi beside the current reviewer finds the most.

Ranking for reviews:
beside GPT > GPT alone > Kimi alone,
because Kimi found 4 defects GPT missed,
and GPT found 7 that Kimi missed.

Ranking for long audits:
beside Sonnet > Sonnet alone > Kimi alone,
because Kimi found 2 hangs Sonnet missed,
and Sonnet found 5 defects that Kimi missed.

If Kimi stays in use,
its briefs should forbid `ps` with `args`,
the one channel through which these runs saw other runs' names.

## Decision

The owner answered in the question tool on 2026-11-10,
after reading the rankings in the "Recommendation" section:
"Kimi alone" for reviews of a change,
and "Both side by side (Recommended)" for long audits.

Rule `KRV` in `AGENTS.md` carries the first,
and rule `LHZ` the second.
GPT stays only for small audits,
under rules `XVW`,
`XVB` and `XVS`.
Kimi's briefs forbid `ps` with `args`.
