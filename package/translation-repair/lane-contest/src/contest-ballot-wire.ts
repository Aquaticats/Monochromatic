import {
  CANDIDATE_DECLARED_NAMES_RULES,
  JUDGE_POLICY_BLOCK,
} from '@monochromatic-dev/translation-repair-house-rule/ts';
import type { JsonSchemaResponseFormat, } from '@monochromatic-dev/translation-repair-provider/ts';
import { SIZE_NOTE_POLICY, } from './contest-size-note.ts';
import { continuesLatinWord, } from '@monochromatic-dev/translation-repair-markup/ts';
import {
  APPARATUS_KINDS,
  NARRATIVE_DETAIL_IS_NOT_APPARATUS,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';

//region Contest ballot wire
// The parts every two-way contest shares: the question, the reply schema, and
// the reading that narrows a judge's findings to the candidates they blame.
//
// EXTRACTED RATHER THAN COPIED when the consolidation gate needed the same
// question over a different pair of candidate names. The lane contest measured
// 10 of 13 against the eight-entry reading where a general-preference control
// managed 8; a second copy of that wording would drift from the one that number
// describes, and the number would go on being quoted.
//
// THE POLICY NAMES NO CANDIDATE. It speaks of "two English renderings" and
// "each candidate" throughout, which is what makes it shareable: the names live
// in the user message and in the reply instruction, where the caller puts them.

/**
 Answer a judge gives when neither candidate is better.

 A VERDICT, not a failure to answer. Two candidates that differ only in
 wording have no better one, and a judge forced to pick would be inventing a
 preference the evidence does not carry.
 */
export const CONTEST_REFUSAL = 'neither';

/**
 What every two-way contest tells its judges.

 THE ORIGINAL IS THE STANDARD, per `doc/decision/translation-repair-output-goal.md`.
 The archive rendering is shown as EVIDENCE about what the original says and a
 starting point worth keeping where it is right, never as the thing a
 candidate is scored against. Accurate detail the archive adds is kept rather
 than stripped, which is the one place a candidate may exceed the original.
 */
export const CONTEST_POLICY: string = [
  'You are choosing which of two English renderings of a Chinese passage should be published.',
  '',
  'THE ORIGINAL IS THE STANDARD. Judge each candidate against the Chinese, never against the archive rendering.',
  'The archive rendering is shown only as evidence about what the original says, and as wording worth keeping where it is right.',
  '',
  'Answer two questions about each candidate first, and let the choice follow from them.',
  '',
  'UNSUPPORTED: does the candidate state something the Chinese does not say?',
  'An invented time period, an invented characterization, a strengthened claim: all unsupported.',
  `Apparatus the archive supplies (${APPARATUS_KINDS}) is NOT unsupported: keeping it is correct.`,
  NARRATIVE_DETAIL_IS_NOT_APPARATUS,
  '',
  'DROPPED: does the candidate omit something the Chinese does say?',
  'A clause, a qualifier, a named object, a speaker aside: all dropped.',
  '',
  'DROPPED ALSO COVERS WHAT THE ARCHIVE CARRIES AND THE CHINESE DOES NOT SAY.',
  'The archive rendering is the page as it stands, not only evidence about the original, so a candidate leaving out something it carries deletes that from the page.',
  'Where the Chinese is SILENT rather than contradicting, dropping page apparatus the archive carries, or a whole region of the page, is a fault of this kind, and keeping it is correct; a narrative detail the archive adds to a passage is not kept on the archive\'s word, since what happened is never apparatus.',
  'THE SEVEREST CASE IS A WHOLE REGION. Where the archive carries a quoted letter, a list, a details section or any block the candidate does not, that is dropped content, NOT a candidate being faithful to a shorter original.',
  'A passage the Chinese states in one line can stand beside a page region the archive spells out at length, and the shorter candidate is the one that lost something.',
  '',
  SIZE_NOTE_POLICY,
  '',
  CANDIDATE_DECLARED_NAMES_RULES,
  '',
  JUDGE_POLICY_BLOCK,
  '',
  'THEN CHOOSE. Prefer the candidate with no unsupported statements. If both are clean, prefer the one that drops nothing.',
  `Answer "${CONTEST_REFUSAL}" when they differ only in wording and neither is more faithful, which is a real verdict rather than a failure to answer.`,
  `Answer "${CONTEST_REFUSAL}" also when both are equally unfaithful.`,
].join('\n',);

/**
 The part of a contest's ask that says it judges no archive rendering.
 */
type ContestWithoutArchive = {
  /**
   False: no archive verdict is asked.
   */
  readonly asksArchive: false;

  /**
   Absent: there is no archive vocabulary to list.
   */
  readonly archiveVerdicts?: undefined;
};

/**
 The part of a contest's ask that says it judges the archive rendering, with the vocabulary of that verdict.
 */
type ContestWithArchive = {
  /**
   True: an archive verdict is asked.
   */
  readonly asksArchive: true;

  /**
   Every verdict a ballot may give the archive rendering, the guard's own list.
   */
  readonly archiveVerdicts: readonly string[];
};

/**
 Builds the reply schema one contest asks for.

 REASON FIRST (owner, 2026-10-21, "Reason first everywhere"), then the two
 findings lists, then the choice and the archive answer, which the reason
 precedes as the panel's does (owner, 2026-09-27, "Reason before vote"). The
 reply is read by property name, so the order changes only what a model
 writes first.

 THE KEYS ARE NAMED SO THAT THEY SORT IN THAT ORDER (ledger B762): `assessment`
 is the reason, `dropped` and `unsupported` the two findings lists, `verdict`
 the choice and `verdictOnArchive` the archive answer. Charm Hyper returns a
 tool call's keys alphabetically, so a schema declared in another order gives
 the reason first on three providers and last on one. The ballot a reader
 builds from the reply keeps the names `reason`, `choice` and `archive` it is
 stored under.

 NAMED PER CONTEST, because the schema name is what a provider log and a
 degradation finding call this stage, and two stages sharing one name are two
 stages nobody can tell apart afterwards.

 @param schemaName - name this contest's replies are recorded under

 @param verdicts - every choice a ballot may cast, the candidate names and the refusal; the guard's own list, so the
 schema's `enum` and the guard cannot drift (P1 of the second review of the Claude route; the lead's ruling of
 2026-10-15 lists a reply's labels as an `enum`)

 @param asksArchive - whether this contest also judges the archive rendering

 @param archiveVerdicts - every verdict a ballot may give the archive rendering, present exactly when the contest asks
 for one

 @returns Response format for the round

 @example
 ```ts
 const format = contestResponseFormat({
   schemaName: 'lane_contest',
   verdicts: CANDIDATE_NAMES,
   asksArchive: true,
   archiveVerdicts: ARCHIVE_VERDICTS,
 },);
 ```
 */
export function contestResponseFormat(
  {
    schemaName,
    verdicts,
    asksArchive,
    archiveVerdicts,
  }: {
    readonly schemaName: string;
    readonly verdicts: readonly string[];
  } & (ContestWithoutArchive | ContestWithArchive),
): JsonSchemaResponseFormat {
  /**
   Archive property, present only for the contest that asks about it.

   NOT DEFAULTED. Both contests state the answer at their own call site,
   so a third contest added later cannot inherit a choice nobody made.
   */
  const archiveProperty = asksArchive
    ? {
      verdictOnArchive: {
        type: 'string',
        enum: [...archiveVerdicts,],
      },
    }
    : {};

  /**
   Archive entry in the required list, matching the property.

   REQUIRED HERE THOUGH THE LANE GUARD READS ITS ABSENCE LENIENTLY. The
   schema is how a compliant model is told to answer; the guard is how a
   model that ignores it is kept from losing its lane ballot too.
   */
  const archiveRequired: readonly string[] = asksArchive
    ? ['verdictOnArchive',]
    : [];

  return {
    type: 'json_schema',
    json_schema: {
      name: schemaName,
      schema: {
        type: 'object',
        // DECLARED IN THE ORDER THE KEYS SORT (ledger B762), which is the order the owner ruled on 2026-10-21
        // ("Reason first everywhere"): the assessment, the two findings lists, the verdict, then the verdict on the
        // archive where this contest asks for one. `verdictOnArchive` sorts after `verdict`, so a provider that
        // returns a tool call's keys alphabetically keeps the archive answer behind the reason and the choice. The
        // scan of `source-scan` passes over the spread member, so `contest-ballot-wire.unit.test.ts` pins its place.
        properties: {
          assessment: { type: 'string', },
          dropped: {
            type: 'array',
            items: { type: 'string', },
          },
          unsupported: {
            type: 'array',
            items: { type: 'string', },
          },
          // THE FINDINGS STAY PLAIN STRINGS: a finding may annotate the candidate it names, and the reader takes the
          // name from the start of the finding (`readCandidateNames`). THE VERDICTS ARE THE GUARD'S OWN LISTS, as an
          // enum: a live Claude call under a plain string answered "Consistent", which the guard refused.
          verdict: {
            type: 'string',
            enum: [...verdicts,],
          },
          ...archiveProperty,
        },
        required: [
          'assessment',
          'dropped',
          'unsupported',
          'verdict',
          ...archiveRequired,
        ],
        // CLOSED, as Anthropic's structured outputs require of every object and the other routes accept.
        additionalProperties: false,
      },
    },
  };
}

/**
 Whether a value is a list of strings, whatever those strings say.

 SHAPE, NOT VOCABULARY. An earlier form of the lane contest's guard demanded
 that every member name a candidate, so a judge that filled the findings with
 the offending phrases instead lost its whole ballot, choice included. Two of
 the first sixty calibration voices went that way, both carrying a usable
 choice. The choice is the thing a contest counts, and no wording of a finding
 may cost a voice.

 NOR DOES SHAPE COST ONE (ledger B46): no ballot guard asks this of a findings
 field any more; each reader takes the field through {@link findingsOrNone}.

 @param value - list from a reply

 @returns Whether it is a list of strings

 @example
 ```ts
 const shaped = isStringList([ 'repair', ],);
 ```
 */
export function isStringList(value: unknown,): value is readonly string[] {
  return Array.isArray(value,)
    && value.every(function isText(member: unknown,): boolean {
      return ((typeof member) === 'string');
    },);
}

/**
 A ballot's findings field as the list of strings it is, or no findings.

 A FINDING'S SHAPE NEVER COSTS A VOICE (ledger B46). A judge that left a
 findings field out, wrote `null`, or wrote a bare phrase where a list goes
 still cast its choice, and the choice is what a contest counts. Before
 2026-09-30 the lane contest refused such a ballot whole, and the
 consolidation and polish gates refused one with the field left out while
 keeping one with the field set to anything; all three readers now take the
 field through here, and their guards ask only for a readable choice and
 reason.

 @param value - findings field as the reply carried it, absent included

 @returns Its strings, or none when it is not a list of strings

 @example
 ```ts
 const unsupported = findingsOrNone(wire.unsupported,);
 ```
 */
export function findingsOrNone(value: unknown,): readonly string[] {
  return isStringList(value,) ? value : [];
}

/**
 Whether a value is one of the names this contest allows.

 RETURNS A PLAIN BOOLEAN rather than narrowing, because a type predicate has
 to name a parameter and this one reads a property of a destructured object.
 Each contest wraps it in its own one-line predicate over its own union.

 @param value - candidate name from a reply

 @param names - names this contest allows

 @returns Whether the value is one of them

 @example
 ```ts
 const allowed = namesOneOf({ value: 'repair', names: [ 'repair', 'translate', ], },);
 ```
 */
export function namesOneOf(
  {
    value,
    names,
  }: {
    readonly value: unknown;
    readonly names: readonly string[];
  },
): boolean {
  return ((typeof value) === 'string')
    && names.includes(value,);
}

/**
 Whether the character at one offset could continue a word.

 READS PAST THE END SAFELY, because `charAt` answers an empty string beyond
 the last index and an empty string continues nothing. A candidate name
 filling a finding entirely therefore needs no separate case.

 @param text - finding being read

 @param at - offset just past a candidate name

 @returns Whether the character there extends that name into a longer word

 @example
 ```ts
 const continues = continuesWord({ text: 'repairing', at: 'repair'.length, },);
 ```
 */
function continuesWord(
  {
    text,
    at,
  }: {
    readonly text: string;
    readonly at: number;
  },
): boolean {
  // A Latin letter, accented or not, a digit or a combining mark goes on with
  // the name; the test took ASCII letters and digits only until ledger B18.
  return continuesLatinWord({ character: text.charAt(at,), },);
}

/**
 Whether one finding blames one candidate.

 ANNOTATION IS NOT REFUSAL. Judges write `repair`, and they write
 `repair (changes the bottle to a can)`, and both name the same candidate.
 A finding naming a phrase rather than a candidate blames nobody, and says so
 by matching none of them.

 @param finding - what a judge wrote

 @param name - candidate to test for

 @returns Whether this finding names this candidate

 @example
 ```ts
 const blamed = namesCandidate({ finding: 'repair (adds a season)', name: 'repair', },);
 ```
 */
function namesCandidate(
  {
    finding,
    name,
  }: {
    readonly finding: string;
    readonly name: string;
  },
): boolean {
  /**
   Finding with surrounding space and case removed.
   */
  const folded = finding
    .trim()
    .toLowerCase();
  return (folded.startsWith(name,)
    && (!continuesWord({
      text: folded,
      at: name.length,
    },)));
}

/**
 Reads which candidates a list of findings blames.

 @param findings - findings exactly as a judge wrote them

 @param names - candidate names this contest allows, in canonical order

 @returns Names blamed, in that order and without repeats

 @example
 ```ts
 const blamed = readCandidateNames({ findings: [ 'repair (adds a season)', ], names, },);
 ```
 */
export function readCandidateNames<NameT extends string,>(
  {
    findings,
    names,
  }: {
    readonly findings: readonly string[];
    readonly names: readonly NameT[];
  },
): readonly NameT[] {
  return names.filter(function blamed(name,): boolean {
    return findings.some(function namesIt(finding,): boolean {
      return namesCandidate({
        finding,
        name,
      },);
    },);
  },);
}

//endregion Contest ballot wire
