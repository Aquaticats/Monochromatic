/**
 Keeps a distorted rendering of a documented event separate from unrelated added information.
 The category is an input to repair, so duplicate addition diagnoses can steer
 an author to delete an event that instead needs its participant corrected.
 Genuine addition and omission controls remain actionable under this policy.

 @example
 ```ts
 const criticRule = ACCURACY_CATEGORY_SCOPE;
 ```
 */
export const ACCURACY_CATEGORY_SCOPE =
  'Accuracy category scope: use accuracy/mistranslation when a source-grounded event or statement is rendered with the wrong actor, recipient, timing, relationship, negation, certainty or quantity. Its meaning needs correction; the source-grounded event is not disposable. Use accuracy/addition for independent extra information with no counterpart in the supplied source evidence. Do not additionally label the same distorted rendering as addition or omission merely because its incorrect assertion is not stated verbatim in the source. Genuine independent added information remains an addition even when another part of its sentence is a mistranslation.';

/**
 Requires category validity as well as a real defect before a claim gains repair authority.
 This is review guidance, not deterministic claim suppression or reclassification.

 @example
 ```ts
 const panelRule = ADJUDICATION_CATEGORY_CHECK;
 ```
 */
export const ADJUDICATION_CATEGORY_CHECK =
  'A supported defect must fit its claimed category as well as identify a real problem. If the source supplies the corresponding event but the target distorts its actor or other relation, support a properly stated mistranslation claim; vote unsupported on an addition diagnosis of that same distortion. Do not endorse an incorrect diagnosis merely because some error exists at that location.';
