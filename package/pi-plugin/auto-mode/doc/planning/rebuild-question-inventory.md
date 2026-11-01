# Complete current question inventory

Recomputed on 2026-10-19 after the user required individual judgment of every tool call.
This replaces the incomplete handoff that presented only Q7.
The [visual review](ground-up-rebuild-review.html#frontier) presents every current choice together.
The [working understanding](rebuild-understanding.md) records meanings and evidence.
Implementation remains stopped.

## Settled boundary

Every outer tool invocation has its own prepared input, rule answers, decision, review allowance, and human approval.
Parallel emission does not combine calls.
`codemode` counts as one tool call even when its body contains multiple operations.
Parse and assess that body as a whole; do not give its internal operations separate approval units or budgets.
Pi owns execution scheduling.

This removes batch-member attribution, shared verdict reduction, joint survivor selection,
shared review budgets, and consolidated batch approval from the design tree.
It does not remove conflicts among rules governing the same call.

## Questions answerable now

The original question numbers remain stable.
Q10 is withdrawn because the existing limit governs Luna requests; shadow requests are separately accounted.

### Q7: How should one call's Deny and Ask answers combine?

Example: an upload contains customer names and emails.
“Never publish raw customer fixtures” gives Deny; “Ask before public uploads” gives Ask.
A separate test call has a separate judgment and is unaffected.
The menu also covers the same upload receiving Approve from “Publish the requested report” plus the privacy Deny,
or that Deny plus an unresolved “Use the approved storage region” assessment when the destination region is missing.
In each case A blocks the upload; B reconciles the mixed answers within the remaining allowance.
These rules are illustrative, not proposed built-ins.

- A: Block the upload. This honors the admitted prohibition immediately,
  but does not give a mistaken interpretation another model attempt.
- B: Reconcile the upload's mixed answers within its remaining budget.
  This can correct interpretation, but delays a valid denial and may turn a clear case into a human interruption.

Ranking: A > B, because a confirmation requirement does not itself remove a prohibition.
The [full reduction options](rebuild-review-round-2.md#q7-for-the-same-call-how-should-deny-and-ask-combine)
also describe the related Approve, unresolved, and not-applicable combinations.

### Q8: Can complete all-not-applicable coverage approve a call?

Example: a read receives a valid not-applicable answer from every effective rule.
No rule is missing, tied, Deny, or Ask.

- A: Allow it. This preserves autonomy when no applicable restriction was found,
  but an erroneous relevance judgment could miss a restriction.
- B: Require at least one Approve. This demands positive permission evidence,
  but adds a new permission requirement and can interrupt routine work.

Ranking: A > B, because complete not-applicable coverage is resolved evidence under the autonomy baseline.

### Q9: How do focused follow-ups resolve an uncertain rule answer?

Example: one test call's first answer ties on a rule requiring disposable fixture writes and no external transfer.
Follow-ups distinguish fixture confinement from external transfer.

- A: Gather those focused findings, then reconcile the original rule using them.
  The evidence-to-rule connection is explicit, but reconciliation consumes the final batch in this example.
- B: Code combines an explicitly complete decomposition.
  This can finish after the focused batch, but arbitrary prose cannot silently become an exhaustive checklist.
- C: Each focused follow-up still decides the whole original rule.
  This leaves another batch available, but needs a clear update contract so new answers do not erase old evidence.

Ranking: A > C > B.
A makes subsidiary facts and final interpretation separately inspectable.
C handles arbitrary rules without the exhaustive code-owned decomposition B needs.

### Q11: Should unfinished shadow comparisons resume after Pi restarts?

Example: Pi closes after Luna finishes but before Span 02 returns.

- A: Persist and replay the unfinished comparison.
  This preserves evidence across restarts, but a later start can transmit the earlier session's captured input.
- B: Cancel it and record the gap without replay.
  This ends its delivery lifetime with the session, but permanently loses the comparison.

Ranking: A > B, because replay preserves the intended comparison evidence without delaying the tool decision.
Provider failures can still prevent completion.

### Q12: Should complete decision evidence expire by age?

Example: an old decision needs investigation long after its session ended.

- A: Retain until explicitly removed, using compression or archiving.
  This preserves reconstructable history, but also preserves sensitive context and accumulated storage.
- B: Expire by an explicitly selected age.
  This limits data lifetime, but can permanently remove the evidence needed for a later investigation.

Ranking: A > B, because the recorded diagnostic preference favors preserving evidence.
The duration, if B is selected, is a later choice informed by record-volume measurements.

### Q13: At full evidence storage, preserve old records or new capture?

Example: compression has been attempted, the store is full, and a new review completes.
Neither option blocks the tool merely because logging is full.

- A: Preserve existing records and visibly report new capture gaps.
  This avoids destroying retained evidence, but new decisions lack full records until space is available.
- B: Remove oldest inactive records to keep capturing.
  This preserves recent investigations, but destroys older evidence that might still be needed.

Ranking: A > B, because automatic destruction is a greater departure from the current preservation preference.
The byte limit remains a measurement task, not an invented number in this question.

### Q14: What behavior is the Span 02 comparison intended to study?

Example: startup relevance wrongly excludes a governing rule.
Later operational review never asks that rule, so comparing only operational answers cannot reveal that exclusion.

- A: Compare startup relevance and operational judgments, reporting them as separate tasks.
  This exposes both rule-selection and decision disagreements,
  but expands the study beyond four-choice operational judgments and retains startup comparison records too.
- B: Study operational judgments only.
  This keeps the study focused on approve/deny/ask/not-applicable answers,
  but deliberately leaves startup relevance disagreements unobserved.

Ranking: A > B, because the startup filter affects every subsequent review.
Extra request price alone is not a reason to select B under the quality-over-cost guidance.
This question concerns the intended study scope.

### Q15: Preserve older genuine human messages outside the short review window?

Example: the human says “Never upload raw customer fixtures,” then later says “Inspect the logs.”
After 12 assistant output blocks, the original prohibition is outside the historical window
and is not the latest-human-request field.

- A: Add an ordered, verbatim genuine-human-message history outside the bounded recent-history field.
  Preserve later corrections alongside the original messages so precedence remains visible.
  This preserves the evidence without a generated constraint summary,
  but grows the payload and requires capacity and compaction-boundary qualification.
- B: Keep only the specified recent window and latest-human field.
  This keeps that payload bounded, but explicitly accepts losing older human instructions from the reviewer input
  when no other included source carries them.

Ranking: A > B, because a newer request does not automatically expire earlier constraints.
A changes the snapshot contract explicitly; it is not silently implemented by this inventory.
The full body of each historical message remains data to interpret with its provenance and later corrections,
not a claim that every sentence is a permanent instruction.

## Dependent questions and their prerequisites

These remain visible but are not premature mechanism menus.
Their concrete options depend on answers or evidence not yet available.

- If Q7 B: how will admitted conflicting answers be reconciled without silently replacing earlier evidence?
  Develop examples where an exception was missed and where the rules are compatible.
- After Q7 and Q8: which incomplete response sets can establish a terminal result?
  Derive and test the reducer truth table; ask again only if a new semantic choice remains.
- After Q9: how are focused questions generated from the particular unresolved distinction?
  Demonstrate generation on arbitrary authored rules, not only a fixed example catalogue.
- After Q9: how are child findings bound to the original rule,
  and how are conflicting follow-up findings reconciled within the existing allowance?
- After the follow-up design: how will every Ask explain evidence and required judgment?
  The Decisions contract supplies choice probabilities, not free-form reasons.
  Qualify a grounded explanation mechanism before offering model/API alternatives.
- If Q11 A: when is replay eligible after a restart, and how are cancelled-session requests distinguished?
  Develop delivery identities, duplicate-attempt handling, and cancellation examples first.
- If Q11 A: how should unfinished delivery compete with ordinary diagnostic records at storage pressure?
  Keep replay promises and ordinary retention separate; determine concrete ownership and bounds.
- If Q12 B: what age limit fits the user's investigation horizon and measured storage growth?
- After Q12 and Q13: what storage ceiling and archival behavior fit measured complete-record volumes?
- If Q15 A: how will genuine human provenance survive compaction, branches, and session resume?
  Measure capacity before proposing any truncation or semantic summarization.
- After measured implementation and dependency inventories: do the provisional source and deployment budgets fit?
  Any proposed tradeoff must identify a real exceeded limit and a concrete alternative.

## Facts and engineering work, not questions for the user

- Exercise one independent review for each parallel call through public Pi hooks.
  Exercise `codemode` as one outer review despite multiple operations in its body.
- Qualify prepared inputs and host cancellation without private hooks or copied Pi components.
  The user already accepts later extensions mutating reviewed arguments as unsupported.
- Preserve uncertain startup applicability under “could be potentially relevant.”
  Failed filtering with fresh parsed rules retains all those rules.
  Do not invent a new exclusion threshold or target-count truncation.
- Measure final request sizes and actual endpoint capacity before partition sizes or timeout values.
  Each call retains every-rule first-pass coverage and its own counted review allowance.
- Preserve valid answers when another answer is absent or malformed; missing evidence does not fabricate a verdict.
- Validate observed instruction/input changes and invalidate affected evidence.
  Resolve the allowance's identity against the individual invocation before introducing a budget-reset option.
- Verify optional-reason omission, stale prompt cancellation, model-visible feedback, and reload/resume behavior.
  Omission preserves the selected decision; a conditional human instruction must not be ignored as mere commentary.
- Measure observer load, cancellation, queues, and logger behavior before selecting resource limits.
- Verify the shared logger's actual RPC/TUI output and failure reporting.
  Logging ownership is already settled; a second subsystem is not an option introduced by this inventory.
- Built-in rule contents remain explicitly deferred.
  They are not smuggled into current questions or adopted examples.

## Evidence for the newly exposed context question

The public-SDK fixture is `~/temp/agent/pi-human-context-probe-20261019/`.
`mise run lint:types` and `mise run probe` pass; `results.json` records the controls.
It uses actual `createAgentSession`, a public `before_agent_start` handler, and a scripted local provider.
No remote model is called.

Both cases retain the older human instruction in Pi's full provider context and session branch.
Neither promotes it into `ctx.getSystemPrompt()`.
The positive control retains the instruction in the proposed short snapshot.
After 12 intervening assistant outputs and a new human request, the reconstructed short snapshot omits it.
This exercises plain output units with compaction disabled and no third-party context transformations.
It does not claim every configured extension has identical context behavior.

## Coverage correction

The complete review passes browser interaction checks for all 25 options and 18 navigation links,
the scenario and manual-choice controls, keyboard operation, and 390-pixel layout in light and dark themes.
All 12 diagrams pass geometry and narrow-screen keyboard scrolling in both themes.
No page or console errors were reported.
The retained [question overview](ground-up-rebuild-review/complete-review-frontier-dark.png)
and [individual-call diagram](ground-up-rebuild-review/individual-chart-grouping-dark.png)
show the revised scope; corresponding light captures and every question panel are retained beside them.
The Markdown renderer passes this inventory and the other affected documents.

The missed handoff was a process error: presenting one question concealed other independent choices.
Proposed replacement for the existing `QGR` guidance:

“Adopt settled answers unasked. Ask every open, independently answerable choice together,
including veto-open adoptions; record later choices with their prerequisites.”

This is a proposal, not an additional standing-rule edit.
The current inventory implements that practice for this review.
