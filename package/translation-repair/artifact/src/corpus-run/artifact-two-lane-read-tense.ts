import { requireExactKeys, } from '../artifact-exact-guard.ts';
import {
  ArtifactParseError,
  requireArray,
  requireRecord,
  requireString,
} from '@monochromatic-dev/translation-repair-artifact-pool/ts';
import type {
  PresentTenseCandidate,
  PresentTenseRule,
  TenseCandidateGroup,
} from '@monochromatic-dev/translation-repair-translate-stage/ts';

//region Recorded tense candidates reader
// READS THE TENSE CANDIDATES A SLICE RECORDS (`tenseCandidates`, the audit
// finding of a contest slice, `artifact-two-lane-contest.ts`, and of a
// consolidation slice that ships wording, `artifact-two-lane-consolidate.ts`).
// Stored values are checked against what the writers record
// (`lane-contest-tense.ts`, `consolidate-shipped-tense.ts`) before anything is
// returned, so a reader never hands a record it could not have written to the
// audit trail: a list holds at least one group, a group at least one
// candidate, and the labels are those the slice's sheet gave its texts, each
// once and in the order the sheet printed them. The finding is read back as it
// was written and feeds no verdict.

/**
 Rules a recorded candidate may name, as the detector writes them.
 */
const RECORDED_RULES: readonly PresentTenseRule[] = [
  'contraction',
  'copula-or-auxiliary',
  's-ending-verb',
  'bare-verb',
];

/**
 Reads one recorded rule name, refusing any name the detector never writes.

 @param value - recorded rule name

 @param path - artifact path of the value, for the refusal

 @returns The rule name

 @throws {@link ArtifactParseError} when the name is not a rule the detector writes

 @example
 ```ts
 const rule = parseRecordedRule({ value: 'bare-verb', path: 'x.rule', },);
 ```
 */
function parseRecordedRule(
  {
    value,
    path,
  }: {
    readonly value: unknown;
    readonly path: string;
  },
): PresentTenseRule {
  /**
   Rule name as recorded, before it is matched against those the detector writes.
   */
  const named = requireString({
    value,
    path,
  },);
  for (const rule of RECORDED_RULES) {
    if (rule === named)
      return rule;
  }
  throw new ArtifactParseError({
    path,
    reason: 'a present-tense rule the detector writes',
  },);
}

/**
 Reads one recorded candidate, checking each field's shape.

 @param value - recorded candidate

 @param path - artifact path of the value, for refusals

 @returns The candidate

 @throws {@link ArtifactParseError} when a field is missing or of the wrong shape

 @example
 ```ts
 const found = parseRecordedCandidate({ value, path: 'x.found[0]', },);
 ```
 */
function parseRecordedCandidate(
  {
    value,
    path,
  }: {
    readonly value: unknown;
    readonly path: string;
  },
): PresentTenseCandidate {
  /**
   Candidate as a record, before its fields are read.
   */
  const record = requireRecord({
    value,
    path,
  },);
  requireExactKeys({
    record,
    allowed: [
      'sentence',
      'matched',
      'rule',
    ],
    path,
  },);
  return {
    sentence: requireString({
      value: record.sentence,
      path: `${path}.sentence`,
    },),
    matched: requireString({
      value: record.matched,
      path: `${path}.matched`,
    },),
    rule: parseRecordedRule({
      value: record.rule,
      path: `${path}.rule`,
    },),
  };
}

/**
 Reads one recorded group: the label its sheet gave the text and the
 candidates found in it.

 @param group - recorded group

 @param path - artifact path of the group, for refusals

 @param labels - labels the slice's sheet gave its texts, in the order it printed them

 @param after - position in `labels` of the group read before this one, minus one for the first group

 @returns The group

 @throws {@link ArtifactParseError} when the group is not an object of exactly a label and a list of candidates, when
 its label is not one of `labels` later than the group before it, or when it holds no candidate

 @example
 ```ts
 const read = parseRecordedGroup({ group, path: 'x.tenseCandidates[0]', labels, after: -1, },);
 ```
 */
function parseRecordedGroup(
  {
    group,
    path,
    labels,
    after,
  }: {
    readonly group: unknown;
    readonly path: string;
    readonly labels: readonly string[];
    readonly after: number;
  },
): TenseCandidateGroup {
  /**
   Group as a record, before its fields are read.
   */
  const record = requireRecord({
    value: group,
    path,
  },);
  requireExactKeys({
    record,
    allowed: [
      'label',
      'found',
    ],
    path,
  },);
  /**
   Label the sheet gave the text.
   */
  const label = requireString({
    value: record.label,
    path: `${path}.label`,
  },);
  if (labels.indexOf(label,) <= after) {
    throw new ArtifactParseError({
      path: `${path}.label`,
      reason: `a label from ${labels.join(', ',)}, each once and in that order`,
    },);
  }
  /**
   Candidates found in the text, as recorded.
   */
  const recordedFound = requireArray({
    value: record.found,
    path: `${path}.found`,
  },);
  if (recordedFound.length === 0) {
    throw new ArtifactParseError({
      path: `${path}.found`,
      reason: 'at least one candidate, since a text without one contributes no group',
    },);
  }
  return {
    label,
    found: recordedFound.map(function readFound(
      found,
      at,
    ): PresentTenseCandidate {
      return parseRecordedCandidate({
        value: found,
        path: `${path}.found[${String(at,)}]`,
      },);
    },),
  };
}

/**
 Reads the tense-candidate groups a slice records, present only where the
 record carries the field.

 @param value - recorded `tenseCandidates` value

 @param path - artifact path of the value, for refusals

 @param labels - labels the slice's sheet gave its texts, in the order it printed them

 @returns Groups in recorded order

 @throws {@link ArtifactParseError} when the value is not a non-empty list of labelled, non-empty candidate lists under
 the labels the slice's sheet gave

 @example
 ```ts
 const groups = parseRecordedTenseCandidates({ value, path: 'x.tenseCandidates', labels: CONTEST_TENSE_LABELS, },);
 ```
 */
export function parseRecordedTenseCandidates(
  {
    value,
    path,
    labels,
  }: {
    readonly value: unknown;
    readonly path: string;
    readonly labels: readonly string[];
  },
): readonly TenseCandidateGroup[] {
  /**
   Groups as recorded, before any is read.
   */
  const groups = requireArray({
    value,
    path,
  },);
  if (groups.length === 0) {
    throw new ArtifactParseError({
      path,
      reason: 'at least one group, since a slice records the finding only where a text has a candidate',
    },);
  }
  /**
   Groups read so far, in recorded order.
   */
  const read: TenseCandidateGroup[] = [];
  /**
   Position in `labels` of the group read last, minus one before the first.
   */
  let after = -1;
  for (const [position, group,] of groups.entries()) {
    /**
     The group at this position, checked against the labels after the one before it.
     */
    const parsed = parseRecordedGroup({
      group,
      path: `${path}[${String(position,)}]`,
      labels,
      after,
    },);
    after = labels.indexOf(parsed.label,);
    read.push(parsed,);
  }
  return read;
}

//endregion Recorded tense candidates reader
