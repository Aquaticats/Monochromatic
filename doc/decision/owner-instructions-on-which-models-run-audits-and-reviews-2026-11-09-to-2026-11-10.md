# Owner instructions on which models run audits and reviews 2026 11 09 to 2026 11 10

## Status

Moved on 2026-11-23 from the old handover, verbatim;
not yet edited into a record of its own.

## Moved from the old handover

From lines 33216 to 33226 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

### Trying Kimi K3.1 Pro as auditor and reviewer (2026-11-10T05:53:55-05:00)

The owner:
"Try synthetic's hf:moonshotai/Kimi-K3.1-Pro as auditor and reviewer.
If this works we can replace both GPT and Sonnet with it."
And:
"Kimi K3.1 Pro has a tendency to cheat.
I suggest you let deepseek look through on disk transcripts after the trial kimi runs.
If it cheated,
we need to disqualify it."

From lines 34148 to 34153 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

- The `gpt-6.1-sol` audit of the invocation route ended with the provider's cybersecurity flag.
  It was not restarted or reworded.
  What it wrote is `dbb8087dfe` on `docs/planning-cli-git-invocation-and-argument-audit`,
  and its worktree keeps one uncommitted test file of the audit's.
  The owner answered "New Claude Sonnet audit (Recommended)", and it ran as `invocation-and-arguments`.
  Fixing the findings it already wrote is ordinary work and waits for a slot.

From lines 34179 to 34214 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

- The owner at about 2026-11-09T05:40-05:00:
  "Please use gpt-6.1-sol with the openai provider for audits and reviews.
  Those don't count within the 20 limit."
  The owner later moved them off Pi:
  "Use `codex` to run openAI reviews and audits instead.
  `--dangerously-bypass-approvals-and-sandbox` has to be passed to codex because codex's auto mode has tons of issues which I'm not listing here."
  and
  "Still pass in xhigh explicitly for codex."
  The command is rule `XVW` in `AGENTS.md`,
  a probe on `gpt-6.1-sol` with codex-cli 0.167.0 answered,
  and Pi's own OpenAI login had begun to fail with `invalid_grant`.
  The owner then wrote:
  "I think we can go more ambitious with codex reviews and audits now that I've effectively approved it running with tools."
  The session's suggestion,
  revisable,
  is that a finding counts only when it was reproduced.
  The rules are in `${HOME}/temp/agent/pi-runs/shared-rules-for-codex-audits.md`,
  and runs write to `${HOME}/temp/agent/codex-runs`.
  The owner then wrote:
  "Keep GPT runs small.
  GPT models cannot be trusted on any long horizon work."
  That is rule `XVS` in `AGENTS.md`.
  The session had just started broad audits of up to three hours and stopped them within minutes.
  It replaces them with runs of one narrow question against one prebuilt binary,
  and it reruns every reported reproduction before acting on it.
  The owner then wrote:
  "Use Claude Sonnet on long horizon reviews and audits.
  I'm approving you to use Claude Code native subagents since now."
  That lifts "Drive pi as subagents exclusively from now on for this session."
  for reviews and audits at least.
  The owner on the model being the point and not the harness:
  "I specifically said "Use Claude Sonnet" so that every harness can do long horizon reviews and audits."
  That is rule `LHZ` in `AGENTS.md`.
  In this session the long audits run as native subagents under `${HOME}/temp/agent/pi-runs/shared-rules-for-sonnet-audits.md`,
  writing to `${HOME}/temp/agent/sonnet-audits`,
  and `codex` keeps the small checks.
