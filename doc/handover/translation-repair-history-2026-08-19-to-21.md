# Translation repair history: 2026-08-19 to 2026-08-21

Part of the [translation repair history index](translation-repair-history.md).

## Contents

This record is kept as one file per subject or span,
each of at most 1,000 lines and named for what it holds.
New text goes into the file whose subject it belongs to;
a file that would pass the limit gives up a subject to a new file,
named for that subject and listed here.

- [Translation repair history 2026-08-19 to 2026-08-21: the reading stage's first runs, the OCR gate and the provider's limits](translation-repair-history-2026-08-19-to-21-reading-stage-first-runs-ocr-gate-and-provider-limits.md):
  The four defects the first real command-line runs of the reading stage found,
  the deterministic OCR gate in front of the model readers,
  the provider's refusal of AVIF,
  the byte cap that turned out to be self-imposed,
  and the silent defects behind it:
  a store guard that rejected a reading kind on resume,
  a stopped run that returned a verdict,
  and a test file shaped so that it registered nothing.
  Sections:
  - [2026-08-19: what the first real CLI runs of the reading stage found](translation-repair-history-2026-08-19-to-21-reading-stage-first-runs-ocr-gate-and-provider-limits.md#2026-08-19-what-the-first-real-cli-runs-of-the-reading-stage-found)
  - [2026-08-19, later: the deterministic reader, and what the provider will not take](translation-repair-history-2026-08-19-to-21-reading-stage-first-runs-ocr-gate-and-provider-limits.md#2026-08-19-later-the-deterministic-reader-and-what-the-provider-will-not-take)
  - [2026-08-19, later still: the cap was ours, and two silent defects behind it](translation-repair-history-2026-08-19-to-21-reading-stage-first-runs-ocr-gate-and-provider-limits.md#2026-08-19-later-still-the-cap-was-ours-and-two-silent-defects-behind-it)
- [Translation repair history 2026-08-19 to 2026-08-21: the reading stage's re-ask, its first settled entry and the reading census](translation-repair-history-2026-08-19-to-21-reading-stage-re-ask-settled-entry-and-census.md):
  Asking a declining reader again,
  the resume proof on the reading kind that needed it,
  the guard bite and the state of the lane,
  the first picture-bearing entry settled end to end with its cost,
  and the separate census of the reading lane,
  whose first null result was a bug.
  Sections:
  - [2026-08-19, last: a declined reading is asked again](translation-repair-history-2026-08-19-to-21-reading-stage-re-ask-settled-entry-and-census.md#2026-08-19-last-a-declined-reading-is-asked-again)
  - [2026-08-19, closing: the entry settles, and the reading lane is measured separately](translation-repair-history-2026-08-19-to-21-reading-stage-re-ask-settled-entry-and-census.md#2026-08-19-closing-the-entry-settles-and-the-reading-lane-is-measured-separately)
- [Translation repair history 2026-08-19 to 2026-08-21: the end-to-end corpus pass launched, then stopped on purpose](translation-repair-history-2026-08-19-to-21-corpus-pass-stopped-on-purpose.md):
  The launch of a full pass at 23:00Z and its stop eight minutes later,
  the reasons for stopping,
  the standing rule to stop and restart,
  what gated the relaunch,
  and the shell mechanics learned while killing the runner.
- [Translation repair history 2026-08-19 to 2026-08-21: the window wired into the repair lane and the instruments for its verify](translation-repair-history-2026-08-19-to-21-window-in-repair-lane-and-verify-instruments.md):
  The neighbouring-source window wired into every repair-lane stage that needs it,
  the test that proves the sheets carry it,
  what the five-entry verify had to show,
  the decision not to window the checker,
  the gate reader and the false pass it produced first,
  and the probe that proves the window reaches production.
- [Translation repair history 2026-08-19 to 2026-08-21: the critic voice-loss alarm and why code work was blocked](translation-repair-history-2026-08-19-to-21-critic-voice-loss-alarm-and-blocked-code-work.md):
  The early observation that the window seemed to cost critic voices,
  the explanations that were measured and falsified,
  the matched baseline that shrank the anomaly,
  what was left as a question,
  and the attempt to do code work in a separate worktree while the verify ran.
- [Translation repair history 2026-08-19 to 2026-08-21: the scope correction that removed the full corpus pass](translation-repair-history-2026-08-19-to-21-scope-correction-no-full-corpus-pass.md):
  The owner's instruction of 2026-08-20 that there is no full corpus pass,
  the rule that replaces it of sizing a run to its question,
  what the earlier stops cost,
  and the two-entry validation that started.
  The sections its warning says size their work against a corpus-wide pass are not in this file;
  the ones that gate or time a full pass or its relaunch are in these files of this group:
  `translation-repair-history-2026-08-19-to-21-corpus-pass-stopped-on-purpose.md` for the stop and the relaunch gates,
  `translation-repair-history-2026-08-19-to-21-reading-stage-re-ask-settled-entry-and-census.md` for the gate on a pass
  and the cost figure to read before planning one,
  and `translation-repair-history-2026-08-19-to-21-window-in-repair-lane-and-verify-instruments.md` for the fix that lands
  before the 92-entry relaunch.
- [Translation repair history 2026-08-19 to 2026-08-21: damage instruments that see past one slice](translation-repair-history-2026-08-19-to-21-damage-instruments-beyond-one-slice.md):
  The assembly-level repetition check,
  wired into both lanes,
  the introduced-defect probe given the window,
  what is verified and what is only predicted,
  the consistency test that replaced an unsound prediction,
  and a documentation trap hit twice.
- [Translation repair history 2026-08-19 to 2026-08-21: what the window did to the repair lane](translation-repair-history-2026-08-19-to-21-window-effect-on-repair-lane-and-lost-resolutions.md):
  The first entry settled under the window,
  with resolved issues falling from 23 to 6,
  the reading of the 17 lost resolutions and the prediction recorded before the re-run,
  the correction that a missing region means the panel said no,
  and the panel's own votes with the roster controlled.
  Sections:
  - [`saurikissa` settled under the window, and the repair lane got much quieter](translation-repair-history-2026-08-19-to-21-window-effect-on-repair-lane-and-lost-resolutions.md#saurikissa-settled-under-the-window-and-the-repair-lane-got-much-quieter)
  - [What the 17 lost resolutions were, and what the re-run should do to them](translation-repair-history-2026-08-19-to-21-window-effect-on-repair-lane-and-lost-resolutions.md#what-the-17-lost-resolutions-were-and-what-the-re-run-should-do-to-them)
  - [CORRECTION: `no-region` means the panel said no, not that a quote failed to anchor](translation-repair-history-2026-08-19-to-21-window-effect-on-repair-lane-and-lost-resolutions.md#correction-no-region-means-the-panel-said-no-not-that-a-quote-failed-to-anchor)
- [Translation repair history 2026-08-19 to 2026-08-21: the panel-rule run and the window as a trade](translation-repair-history-2026-08-19-to-21-panel-rule-run-and-the-window-trade.md):
  The reading pre-registered before the panel-rule run,
  the negative outcome that rewording the rule is not the lever,
  the evidence that the window trades repairs for restraint,
  and what stays undecided without reading the removed passages.
  Sections:
  - [Pre-registered reading for the panel-rule run](translation-repair-history-2026-08-19-to-21-panel-rule-run-and-the-window-trade.md#pre-registered-reading-for-the-panel-rule-run)
  - [The panel-rule run settled: the wording is not the lever, and the window is a TRADE](translation-repair-history-2026-08-19-to-21-panel-rule-run-and-the-window-trade.md#the-panel-rule-run-settled-the-wording-is-not-the-lever-and-the-window-is-a-trade)
- [Translation repair history 2026-08-19 to 2026-08-21: whether the judge can notice content loss](translation-repair-history-2026-08-19-to-21-judge-content-loss-before-decision-time.md):
  Two findings that together raise the question of whether anything catches content loss at decision time,
  and the pre-registered reading of the fidelity probe built to answer it.

## Current stop condition

Historical evidence only.
Candidate M failed on 2026-09-01.
No historical next-action list authorizes a corpus run,
model call,
spent-prompt retry,
or successor implementation.
