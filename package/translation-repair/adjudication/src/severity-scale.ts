//region Severity scale
// LEDGER L5. Critics filed, and panelists re-graded, on neutral, minor, major
// and critical with no definition on either sheet, and two deterministic
// rules read the result: the archive dispute reads an accuracy claim the panel
// settled at major or worse (`archive-dispute.ts`), and the tally holds an
// acceptance settled at neutral for a human (`tally-votes.ts`). Over every
// run, 41 accepted issues had settled at neutral, 28 shipped an edit, and 8
// were claims whose own summary called the rendering accurate or correct,
// while real small losses of nuance were filed neutral too and fixed.
//
// MINOR, MAJOR AND CRITICAL ARE MQM'S, adapted to a memorial page
// (https://www.themqm.org/guidance/values-and-scores/, read 2026-09-28): minor
// "has a limited impact on, for example, accuracy, stylistic quality,
// consistency, fluency, clarity ... but it does not seriously impede the
// usability, understandability, or reliability of the content"; major
// "seriously affects the understandability, reliability, or usability ... for
// instance due to a significant loss or change in meaning or because the error
// appears in a highly visible or important part of the content"; critical
// "renders the entire content unfit for purpose or poses the risk for serious
// physical, financial, or reputational harm".
//
// NEUTRAL IS THIS PIPELINE'S OWN, NOT MQM'S. MQM's neutral marks a spot where
// "a different solution is warranted" without a penalty; here a merely
// possible alternative is no defect at all (the panel's translation policy),
// and neutral is what `issue-taxonomy.ts` says it is: a finding a human
// should see that asserts no defect in the TRANSLATION, an interpretive
// ambiguity or a suspected error in the ORIGINAL.

/**
 Severity definitions both the critic and the panel sheets carry, one line
 per severity in `ISSUE_SEVERITIES` order.

 @example
 ```ts
 const sheet = `Report each defect with a severity.\n\n${SEVERITY_SCALE}`;
 ```
 */
export const SEVERITY_SCALE: string = `SEVERITY SCALE:
- neutral: not a defect in the TRANSLATION at all, only an interpretive ambiguity or a suspected error in the ORIGINAL that a human should see; it asks for no edit
- minor: a real defect of limited impact on accuracy, style, consistency, fluency or clarity, however small, that does not seriously impede understanding
- major: a defect that seriously affects understanding or reliability, for instance a significant loss or change of meaning, or one in a prominent place such as a heading or a person's name
- critical: a defect that makes the page unfit for its purpose or risks serious harm, including to the reputation or dignity of a person the page is about`;

/**
 Critic rule refusing a report that finds nothing wrong.

 @example
 ```ts
 const rules = `${SEVERITY_SCALE}\n${CRITIC_NOTHING_WRONG_RULE}`;
 ```
 */
export const CRITIC_NOTHING_WRONG_RULE: string = 'Never report a rendering you find correct: a claim that calls a '
  + 'rendering accurate, correct or acceptable and names nothing wrong with it is not an issue.';

/**
 Panel rule voting down a claim that finds nothing wrong, with the two
 legitimate no-defect findings kept outside it.

 @example
 ```ts
 const policy = `- ${PANEL_NOTHING_WRONG_RULE}`;
 ```
 */
export const PANEL_NOTHING_WRONG_RULE: string = 'A claim that calls the rendering accurate, correct or acceptable '
  + 'and names nothing wrong with it reports no defect: vote unsupported. An extension/interpretive-ambiguity or '
  + 'extension/suspected-source-error finding is not such a claim.';

/**
 Panel re-grade rule lifting a real defect filed neutral, the one path that
 keeps its fix now that an acceptance settled at neutral asks for no edit.

 @example
 ```ts
 const regrade = `Optionally re-grade a supported claim's severity. ${PANEL_NEUTRAL_REGRADE_RULE}`;
 ```
 */
export const PANEL_NEUTRAL_REGRADE_RULE: string = 'A supported claim filed neutral that describes a real defect in '
  + 'the TRANSLATION, however small, is at least minor: re-grade it, since a claim that settles at neutral asks for '
  + 'no edit.';

//endregion Severity scale
