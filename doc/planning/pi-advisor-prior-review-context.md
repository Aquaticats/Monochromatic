# Reassessing Advisor prior-review context

## Status and scope

[Issue #407][issue] was withdrawn at the user's explicit request and closed as not planned.
The user initially questioned whether excluding prior Advisor outputs by default is desirable.
The assessment did not change implementation or configuration;
the subsequent withdrawal preserves the current default and existing opt-out.
No replacement design has been accepted.

## Recommendation

Do not implement #407 as written.
Keep the current default while separating independent judgment from a review that cannot see previous conclusions.
The existing configuration opt-out remains useful,
but its name describes artifact filtering rather than a guarantee of blind review.

The issue treats prior Advisor output as unrelated by provenance.
A previous review can instead be evidence for the current task:

- Has the revision addressed the earlier finding?
- Was the earlier review mistaken?
- Which of the conflicting recommendations follows the user's requirements?

These examples establish distinct purposes,
not measured frequencies of actual usage.
Exclusion can suit a fresh assessment,
but automatic inclusion is useful for follow-up and synthesis.
Neither setting has a demonstrated review-quality advantage in the issue's cited incident.

## Evidence

### Repository records include iterative review and a preference against guessed evidence selection

`doc/handover/pi-advisor-413-grilling.md:273` records an initial Advisor review
followed by another review checking the user's correction and withdrawal of a recommendation at line 282.
This is a related review sequence,
not evidence that every prior review is unrelated.
It does not prove that the second review required the original tool result.

`doc/planning/pi-advisor-recovery-policy.md:140` records rejection of task-scoped evidence selection:

> Its possible relevance and input savings do not outweigh the risk of omitting a requirement,
> earlier decision,
> or cross-task dependency.

That decision concerns inferred task boundaries,
not this specific prior-review toggle.
It nevertheless argues against introducing selective context as an assumed improvement without evidence.
The unaccepted answer-substitution proposal also ranks full context ahead of selected or blind context in
`doc/handover/pi-advisor-answer-substitution.md:854`.
It is supporting design analysis,
not an accepted decision about #407.

### The current switch removes original reviews but keeps their possible restatements

`package/pi-plugin/advisor/src/config.ts:44` still sets:

```typescript
// package/pi-plugin/advisor/src/config.ts
includePriorAdvisorResults: true,
```

`package/pi-plugin/advisor/src/context.ts:375` excludes manual review messages by `customType`.
`package/pi-plugin/advisor/src/context.ts:418` excludes tool results by tool name:

```typescript
// package/pi-plugin/advisor/src/context.ts
if ((!includePriorAdvisorResults) && (message.toolName
  === ADVISOR_TOOL_NAME))
  return MESSAGE_EXCLUDED;
```

The same file preserves compaction summaries at line 353,
branch summaries at line 363,
and ordinary messages at line 424.
Its assistant filter at line 436 only removes the active Advisor tool-call placeholder.
Previous call arguments and assistant prose remain.
Consequently,
removing an original review need not remove its conclusions from the reviewer's input.
It can also remove the evidence needed to challenge a main-agent paraphrase.

### Call purpose cannot select history policy today

`package/pi-plugin/advisor/src/tool-params.ts:125` accepts only `model` and `question`:

```typescript
// package/pi-plugin/advisor/src/tool-params.ts
return (key !== 'model') && (key !== 'question');
```

This expression identifies rejected extra keys.
The history policy is configuration-wide,
not a parameter selected for a particular follow-up or fresh assessment.
The issue's configuration opt-in does not resolve this mismatch between call purposes.

### The incident does not establish a quality benefit

The [provider-failure diagnosis][incident] records `1512228` serialized characters with prior reviews
and `1503716` without them on its historical raw branch.
Arithmetic on those recorded values gives `8512` characters,
approximately `0.56%` of that input.
This is not a new measurement of current sessions or of the compacted branch.

The same incident records `363290` characters when using compaction-aware entries.
Current preparation already calls `buildContextEntries()` in
`package/pi-plugin/advisor/src/run-preparation.ts:77`.
The default-history proposal is therefore a separate review-policy question,
not an established remedy for the diagnosed context duplication.

## Verification

A disposable Node `v26.8.2` probe imported the current source
`package/pi-plugin/advisor/src/context.ts`
and called `buildAdvisorContext` on synthetic entries,
without truncation,
provider calls,
or real-session mutations.
It exercised `includePriorAdvisorResults: true` first as the positive control,
then `false` on the same fixture.

Observed results:

- Advisor tool-result and manual-review sentinels were present with `true` and absent with `false`.
- Prior call-question,
  assistant-restatement,
  user-reference,
  compaction-summary,
  branch-summary,
  and unrelated-custom-message sentinels were present under both settings.

This verifies the filter's information boundary,
not the effect of that filtering on model judgment.
The existing custom-message test is in
`package/pi-plugin/advisor/src/context.unit.test.ts`.
No model-quality experiment was performed.

## Alternatives and ranking

1.  Keep the default and withdraw the automatic-flip proposal.
     Benefit:
     retains evidence for follow-up reviews without adding another interface.
     Cost:
     direct exposure to earlier conclusions remains,
     so anchoring is still a plausible risk.
2.  Design an explicit call-level fresh-assessment facility if actual usage establishes that need.
     Benefit:
     distinguishes fresh judgment from critique or continuation of an earlier review.
     Cost:
     requires an evidence-selection contract,
     including treatment of paraphrases and summaries;
     merely relabelling the existing filter as independent would overpromise.
3.  Flip the configuration default as #407 proposes.
     Benefit:
     removes direct exposure to original Advisor outputs unless configured otherwise.
     Cost:
     discards relevant evidence for follow-ups while retaining other channels for earlier conclusions.

Ranking:
retaining the default precedes a new facility because this review has not established a need for another interface.
A purpose-specific facility precedes blanket exclusion because the context policy would match the requested task.
This ranking does not claim the present default is empirically optimal.

## Next decision

The user selected withdrawal,
and #407 is closed as not planned.
Its acceptance criteria are not implementation work.
If evaluating a fresh-assessment facility,
compare the same evidence under blind and informed review,
including follow-ups and mistaken earlier findings.
Judge valid new findings,
missed defects,
and unsupported repetition rather than filtering mechanics alone.
No replacement design has been accepted.

[issue]: https://github.com/Aquaticat/Monochromatic/issues/407
[incident]: ../troubleshooting/pi-advisor-long-session-provider-failure.md
