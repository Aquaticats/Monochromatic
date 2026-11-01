import { BILINGUAL_LINE_CLAUSE, } from './bilingual-line-clause.ts';
import {
  DECLARED_NAME_REFERENCE_EXEMPTION,
  NAME_FORM_SCOPE_RULE,
} from './name-form-policy.ts';
import {
  APPARATUS_KINDS,
  NARRATIVE_DETAIL_IS_NOT_APPARATUS,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';

//region Candidate judge rules
// Rules every sheet that weighs named candidates against the ORIGINAL and the
// archive rendering reads, worded in that vocabulary: candidates, the archive
// rendering, unsupported and dropped content. The lane contest and the
// consolidate gate read them inside `CONTEST_POLICY`; the polish gate read none
// of them (ledger S12), and no candidate-weighing sheet read a line rule
// (ledger S15).

/**
 How a judge reads the DECLARED NAMES block, as the contest has always said it.

 MOVED, NOT REWORDED, out of `CONTEST_POLICY` so the polish gate reads the
 same lines; the contest's text is unchanged.

 @example
 ```ts
 const system = `${policy}\n\n${CANDIDATE_DECLARED_NAMES_RULES}`;
 ```
 */
export const CANDIDATE_DECLARED_NAMES_RULES: string = [
  'DECLARED NAMES ARE ATTESTED FACTS about this person, taken from the documents\' own front matter.',
  DECLARED_NAME_REFERENCE_EXEMPTION,
  'When referring to a person or place, declared names settle HOW to spell that reference and OUTRANK the archive rendering where it uses another spelling.',
  NAME_FORM_SCOPE_RULE,
  'Declared identities are NOT extra content a passage owes: a candidate that does not name this person has dropped nothing, and a line attributing the passage to someone ELSE never takes this person\'s name.',
].join('\n',);

/**
 Page apparatus in a candidate sheet's own words.

 NOT `PAGE_APPARATUS_IS_KEPT`, which speaks of "the existing translation", a
 text these sheets call the archive rendering; a rule about a term the sheet
 never shows reads as addressed to someone else.

 @example
 ```ts
 const system = `${policy}\n\n${CANDIDATE_APPARATUS_RULE}`;
 ```
 */
export const CANDIDATE_APPARATUS_RULE: string = `PAGE APPARATUS IS KEPT. Apparatus the archive rendering carries and the Chinese is silent about (${APPARATUS_KINDS}) is NOT unsupported: a candidate keeping it has added nothing, and a candidate leaving it out has dropped page content. ${NARRATIVE_DETAIL_IS_NOT_APPARATUS}`;

/**
 What a line-structured ORIGINAL asks of a candidate, for a judge.

 WRITTEN FOR JUDGES, NOT PRODUCERS. The producers' and editors' line rules
 tell a writer what to do; `TRANSLATE_LINE_STRUCTURE_CRITERION` names a
 numbered criterion and the existing translation, which these sheets lack. So
 a judge reading only "never weigh line breaks" or nothing at all preferred a
 candidate that merged the lines a producer had been told to keep.

 @example
 ```ts
 const system = lineStructured ? `${policy}\n\n${JUDGE_LINE_STRUCTURE_CLAUSE}` : policy;
 ```
 */
export const JUDGE_LINE_STRUCTURE_CLAUSE: string = 'THE ORIGINAL IS LINE-STRUCTURED: each original line is a unit. '
  + 'A candidate that merges two original lines into one, splits one across two, or drops a line has dropped content '
  + 'however well it reads, a candidate that invents a line is unsupported, and a candidate carrying one line per '
  + `original line is correct even where the archive rendering merged them. ${BILINGUAL_LINE_CLAUSE}`;

//endregion Candidate judge rules
