# Wrapper release and reviewer owner answers of 2026 11 11

## Status

Moved on 2026-11-23 from the old handover, verbatim;
not yet edited into a record of its own.

## Moved from the old handover

From lines 29575 to 29575 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

alpha.11 shipped before this verdict, under the owner's "Run it, don't wait (Recommended)".

From lines 30113 to 30131 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

### Reviews go to qwen4 instead of Kimi, and pass 14 ended (2026-11-11T19:27:00-05:00)

The owner, at about 19:27: "Make the Qwen 4 pool 18, and use it for reviews INSTEAD of Kimi."

What that changes:

- The `qwen-token-plan` pool (`qwen-token-plan-individual`, `qwen4-max-preview`) is 18 in the package
  (`package/pi-shared/subagent-driver/src/pool-limits.ts`, with the owner's words beside it).
  The scratch gate still says 4 and is not edited, so only a start through the package's gate sees 18.
- Every new review of a change is a qwen4 run. No new Kimi review is started.
- The Kimi reviews already running (round 231 and a few of round 210 and 219) are left to finish.
  The owner, 19:31, through the question tool (options: let them finish, stop and re-run on qwen4, stop without re-running):
  "Let them finish (Recommended)".
- Long audits are unchanged: Kimi and Sonnet side by side, and qwen4 "Not used for audits" (the owner, 17:58).
- `AGENTS.md` rules KRV and PFW now name qwen4.
  The owner, 19:31, through the question tool (options: rewrite both, this session only): "Rewrite KRV and PFW (Recommended)".
  KRV reads "Reviews of a change go to qwen4-max-preview alone: Pi with
  `--provider qwen-token-plan-individual --model qwen4-max-preview`, in a worktree of its own, on the qwen4 pool.";
  PFW says "a qwen4 review". LHZ (long audits) is untouched.

From lines 30366 to 30368 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

With the open-file guard also confirmed, all six distinct qwen audit findings are confirmed and none refuted;
the owner's "Not used for audits" stands, and the notes in `~/temp/agent/qwen-trials/comparison-notes.md` say so.

From lines 30424 to 30441 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

The owner's answers of 2026-11-11T17:58-05:00, through the question tool, each asked with its options and a ranking:

- qwen4-max-preview in long audits: "Not used for audits".
  The result put to him: on Kimi's own audit prompts over three areas of the alpha.10 wrapper code,
  Kimi reported 22 findings and Sonnet 20 (on its own briefs), every one confirmed by a fix agent;
  qwen reported 6 distinct findings over four runs (none on the commit path, where Kimi had 9),
  4 confirmed, 2 with fix agents to reproduce, none refuted;
  it read nothing forbidden and no other model helped it (`~/temp/agent/qwen-trials/comparison-notes.md`).
  So long audits stay Kimi and Sonnet side by side.
- qwen4-max-preview on change reviews, which it had never done: "Trial beside Kimi (Recommended)".
  Kimi reviews resume, which ends the hold of 16:5x ("Before the Qwen 4 review verdict, avoid starting more reviews.");
  qwen4 reviews three changes Kimi already reviewed, without seeing Kimi's review, and both are scored against what the fix agents confirm.
- The btrfs control run before an alpha: "Skip for alphas (Recommended)".
  The 2026-11-10 choice "Before each release (Recommended)" holds for every release but an alpha.
- The full test gate (`native:gate`) before an alpha: "Run it, don't wait (Recommended)".
  The gate starts on the release commit, the alpha is released and installed at once,
  and what the gate finds is fix work for the next alpha.

From lines 30485 to 30532 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

### The owner's answers of 2026-11-11T16:35-05:00, through the question tool

Eight choices, each asked with its options, pros, cons and a ranking; the owner's picks, verbatim:

- The `qwen-token-plan-individual` plan's DeepSeek models as a fourth pool for regular agents: "Keep it for qwen only".
  The slot script's `qwen-token-plan` pool takes only `qwen4-max-preview`.
- `refactor/git-policy-cli-source-layout` (`671eb1c6d1`) and `integrate/git-policy-cli-deadline-stop-and-conversion-real-git` (`2ac6da873e`): "Delete outright"
  (over pinning their tips under a kept ref, and over keeping them).
  The coordinator removed the clean worktree each was checked out in and deleted both, and the first on `origin` too.
- Issue #3024 (a vendored clone under the Cargo build directory refused by the worktree policy): "Exempt Cargo build dir (Recommended)",
  whose text was "Read CARGO_BUILD_DIR, then CARGO_TARGET_DIR, add it beside uv and mise, and state the exemption's limit in the function's doc."
  Round 220, `fix/git-policy-cli-vendored-clone-under-the-cargo-build-dir-is-exempt`.
- The stale decision records of issues #1698, #3258, #3275, #3276 and #3745: "Rewrite the stale parts" ("Replace old text with current facts."),
  over dated notes beside the old text and over leaving them.
  That is the owner's approval to edit those sentences of `doc/decision/`, and for them it replaces rule `WR6`'s keep-the-original-plus-a-note.
  Round 220, `docs/decision-records-stale-parts-rewritten`.
- Issue #1768 (a session whose `git` is not the wrapper loses every protection silently): "Launcher preflight only",
  over also fixing the shell-command analyzer's PATH-prefix hole.
  Round 220, `feat/pi-shared-subagent-driver-preflight-finds-a-session-without-the-wrapper`.
- Round 187's adopted choice (a worktree add installs a copy taken ahead and names the ignored entries that changed after it): vetoed, "Fresh snapshot every add"
  (measured 5.4 s at the median and 26 s at worst in that agent's fixture).
  Sent to the round 218 agent fixing that branch's review findings, and to the round 212 worktree-add agent.
- Round 187's adopted removal of an orphan snapshot whose identity was never recorded: "Keep removing, fix the check"
  (over naming such snapshots and never removing them automatically);
  the round 218 agent fixes the Kimi finding that the check can match a different snapshot holding the only copy of data.
- Issues #492, #574, #1717, #3796 and #630, whose options are not yet researched: "Defer until alpha.11 (Recommended)".

Two more from the owner the same hour:

- "Before the Qwen 4 review verdict, avoid starting more reviews."
  The coordinator stopped every queued Kimi review that had not started (12: ten of round 219, one of round 210, and round 221's rerun of the driver-port review) and the starters of rounds 219 and 221;
  the 22 reviews already running finish.
  No hourly fill starts a review until the qwen4-max-preview verdict is delivered.
- "I want desktop notifications."
  The Pi extension `package/pi-plugin/agent-settled-notification` failed in 100 of the 373 agent runs started that day
  ("settled-agent desktop notification unavailable: Command failed with exit code 1: notify-send ..."),
  with `notify-send`'s standard error discarded and a one-second bound (`NOTIFICATION_TIMEOUT_MS = 1_000`);
  the same command from a transient user unit exited 0.
  Round 222 (`fix/pi-plugin-agent-settled-notification-delivers-under-load`) finds the cause and makes delivery work under the host's load.
  The shared brief rules now say desktop notifications are the one thing allowed on the desktop and never to be disabled.

The driver-port review (round 212) ended `model-error` on Neuralwatt's
"429 ... lowpri_budget_exceeded ... You've hit your concurrent flex request limit for moonshotai/Kimi-K3.1-Pro (max 22 in flight at once)",
with the Kimi pool at its limit of 22: the first measured statement of that limit.

Still to ask: the shadow pool claim's rename once round 212's proposal lands,
and Sonnet's finding that an add can stay quiet for 600 s against "60s is fine for adding worktrees" once it is framed with options.

From lines 30555 to 30558 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

The owner, 2026-11-11T16:3x-05:00: "qwen-token-plan-individual  qwen4-max-preview is now available in Pi. Check if it can replace Kimi or Sonnet."
`pi --list-models qwen4` lists it (1M context, 393.2K output, thinking, images);
the same plan also lists `deepseek-v4.1-flash`, `deepseek-v4.1-flash-1017`, `deepseek-v4-pro`, `deepseek-v4-pro-0813` and `deepseek-v4-flash-0731`.
A one-line `pi --print` answered `ready`.
