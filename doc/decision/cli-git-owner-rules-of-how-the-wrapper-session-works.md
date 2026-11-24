# Cli git owner rules of how the wrapper session works

## Status

Moved on 2026-11-23 from the old handover, verbatim;
not yet edited into a record of its own.

## Moved from the old handover

From lines 47035 to 47037 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

  Round 2 still merges after gate r7 (it narrows the 0.5.31 and 0.5.32 races;
  owner rule "Treat reviews as async.");
  every finding went to the agent as round 3.

From lines 47860 to 47873 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

### Issues the owner raised are active work

The owner,
in the session,
before commit `25c4b39f58` (2026-11-03,
the commit that first recorded it):
"Why are you still not doing gh issue 6260 and 6261?"
#6260 was fixed and merged into the 0.5.28 base but the issue had no comment;
#6261 had been queued by the coordinator behind 0.5.28 without asking.
Both are now active:
#6260 has a status comment,
and #6261 runs on `refactor/git-policy-cli-subsystem-directories` (from `f89fa63f10`) in slices,
families no running branch touches first.

From lines 49147 to 49152 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

The commits that reached origin/main without a gate,
2026-11-04:
"keep,
gate first (Recommended)",
so the one-executable branch was gated before its merge and before 0.5.35.

From lines 49172 to 49176 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

How a question reaches him,
2026-11-05:
he asked for it as plain prose in the session ("async"),
not through the question tool.

From lines 49270 to 49295 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

### Reviews go to Kimi K3.1 Pro while the Qwen quota is dry (2026-11-14, answered by 18:19-05:00)

The owner wrote "My Qwen quota has ran dry."
Asked through the question tool where reviews of a change go until Qwen is back,
with four courses
(Kimi K3.1 Pro;
hold until Qwen returns;
Claude Sonnet subagents;
GPT through `codex`),
the owner answered "Kimi K3.1 Pro (Recommended)".

- The standing rule (`AGENTS.md`, rule `KRV`:
  reviews of a change go to `qwen4-max-preview` alone)
  and the driver package's `src/reviewer-of-a-change.ts` are unchanged.
  The exception lasts until the owner says the quota is back.
- It is carried by one file:
  `~/temp/agent/runs-of-2026-11-12-ended-by-the-full-drive/keep/reviews-go-to-kimi-k3-1-pro-while-this-file-exists.md`.
  While it exists the queue's starter starts each range review on provider `neuralwatt`,
  model `kimi-k3.1-pro-flex`
  (the package's pool `neuralwatt-kimi-flex`, limit 20),
  holds any entry that names the Qwen model or that the package's own review command would start,
  and the script that puts quota-ended reviews back does nothing.
- To end it:
  delete that file.
  Reviews going on Kimi at that moment run to their end.

From lines 49461 to 49461 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

- QFL was not added to `AGENTS.md`: "Do not add it", 2026-11-07T02:03:01-05:00.

From lines 49488 to 49490 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

- Cleanup is the session's own judgement: "Cleanup whenever it makes sense on your own." and "It covers
  deleting stale detached worktrees.", 2026-11-08T23:47:13-05:00. A worktree goes only when its commits are
  on a branch and it has no uncommitted work; branches stay.

From lines 19368 to 19369 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

The owner wrote "Continue." at about 08:38 after a login;
the session had been logged out since about 03:09.

From lines 34215 to 34224 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

- The owner on the rule for non-trivial commands:
  "Change the "non-trival command" rule to also require running as a process (in harness terminology)."
  and
  "And also check dsh."
  That is rule `RXI` in `AGENTS.md`,
  worded as a harness-managed background process because each harness names it differently:
  Claude Code's `Bash` with `run_in_background`,
  Pi's `process` tool from `@aliou/pi-processes`,
  and DSH's managed background jobs,
  `run_in_background` in `@deepseek-ai/dsh-tool-terminal` and `dsh-tool-pwsh` 0.2.2-rc.1.
