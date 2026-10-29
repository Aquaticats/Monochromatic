/**
 Tests for reading the tense candidates a slice records.

 WHAT IS UNDER TEST is what the reader refuses. The finding is an audit trail
 for a page read, so a record that the writer could not have produced must stop
 the read at its own path rather than reach a count as a candidate: a list with
 no group, a group with no candidate, and a label the slice never offers its
 judges, repeated or out of the order the sheet prints them in.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  caught,
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { ArtifactParseError, } from '@monochromatic-dev/translation-repair-artifact-pool';
import { parseRecordedTenseCandidates, } from '../../dist/final/node/index.mjs';

/**
 Path every message under test is built against.
 */
const TENSE_PATH = 'CatEntry1.tenseCandidates';

/**
 Labels a contest slice offers its judges, in the order the sheet prints them.
 */
const CONTEST_LABELS = [
  'ARCHIVE RENDERING',
  'CANDIDATE "repair"',
  'CANDIDATE "translate"',
];

/**
 One candidate as the writer records it.
 */
const NAPS = {
  sentence: 'Mittens naps in the bookshop attic.',
  matched: 'Mittens naps',
  rule: 's-ending-verb',
};

/**
 Reads a value and returns the refusal it raised.

 @param value - recorded value handed to the reader

 @param labels - labels the slice offers, the contest's unless a case says otherwise

 @returns What the reader threw

 @example
 ```ts
 const refusal = refusalOf({ value: 'naps', },);
 ```
 */
function refusalOf(
  {
    value,
    labels = CONTEST_LABELS,
  }: {
    readonly value: unknown;
    readonly labels?: readonly string[];
  },
): unknown {
  return caught(function readRecorded() {
    return parseRecordedTenseCandidates({
      value,
      path: TENSE_PATH,
      labels,
    },);
  },);
}

/**
 Reads a value and returns the text of the refusal it raised.

 @param value - recorded value handed to the reader

 @param labels - labels the slice offers, the contest's unless a case says otherwise

 @returns Refusal as text, class name first

 @example
 ```ts
 const text = refusalTextOf({ value: 'naps', },);
 ```
 */
function refusalTextOf(
  {
    value,
    labels = CONTEST_LABELS,
  }: {
    readonly value: unknown;
    readonly labels?: readonly string[];
  },
): string {
  return String(refusalOf({
    value,
    labels,
  },),);
}

/**
 What a refusal of a label reads, for the contest's labels.
 */
const CONTEST_LABEL_REASON = 'a label from ARCHIVE RENDERING, CANDIDATE "repair", CANDIDATE "translate", each once and in that order';

//region Recorded tense candidates tests

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: parseRecordedTenseCandidates.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'READS groups of candidates as they were written, in recorded order, naming each of the four rules',
          fn: async () => {
            /**
             A group holding one candidate under each rule the detector writes.
             */
            const group = {
              label: 'CANDIDATE "repair"',
              found: [
                'contraction',
                'copula-or-auxiliary',
                's-ending-verb',
                'bare-verb',
              ].map(function candidateOf(rule,) {
                return {
                  ...NAPS,
                  rule,
                };
              },),
            };
            /**
             The archive's group, which the sheet prints before the lanes'.
             */
            const archiveGroup = {
              label: 'ARCHIVE RENDERING',
              found: [NAPS,],
            };
            expect(parseRecordedTenseCandidates({
              value: [
                archiveGroup,
                group,
              ],
              path: TENSE_PATH,
              labels: CONTEST_LABELS,
            },),).toEqual([
              archiveGroup,
              group,
            ],);
          },
        },),

        it({
          name: 'REFUSES a record that is not a list, naming the path',
          fn: async () => {
            expect(refusalOf({ value: 'naps', },),).toBeInstanceOf(ArtifactParseError,);
            expect(refusalTextOf({ value: 'naps', },),).toBe(
              'ArtifactParseError: artifact parse failed at CatEntry1.tenseCandidates: expected an array.',
            );
          },
        },),

        it({
          name: 'REFUSES an empty list, which the writer never records since a slice carries the finding only where a text has a candidate',
          fn: async () => {
            expect(refusalTextOf({ value: [], },),).toBe(
              'ArtifactParseError: artifact parse failed at CatEntry1.tenseCandidates: '
                + 'expected at least one group, since a slice records the finding only where a text has a candidate.',
            );
          },
        },),

        it({
          name: 'REFUSES a group that is not an object, and one without a text label',
          fn: async () => {
            expect(refusalTextOf({ value: ['naps',], },),).toBe(
              'ArtifactParseError: artifact parse failed at CatEntry1.tenseCandidates[0]: expected an object.',
            );
            expect(refusalTextOf({ value: [{ found: [], },], },),).toBe(
              'ArtifactParseError: artifact parse failed at CatEntry1.tenseCandidates[0].label: expected a string.',
            );
          },
        },),

        it({
          name: 'REFUSES a label the slice never offers its judges, one repeated and one out of the order the sheet prints them',
          fn: async () => {
            expect(refusalTextOf({
              value: [{
                label: 'CANDIDATE "consolidated"',
                found: [NAPS,],
              },],
            },),).toBe(
              `ArtifactParseError: artifact parse failed at CatEntry1.tenseCandidates[0].label: expected ${CONTEST_LABEL_REASON}.`,
            );
            expect(refusalTextOf({
              value: [
                {
                  label: 'ARCHIVE RENDERING',
                  found: [NAPS,],
                },
                {
                  label: 'ARCHIVE RENDERING',
                  found: [NAPS,],
                },
              ],
            },),).toBe(
              `ArtifactParseError: artifact parse failed at CatEntry1.tenseCandidates[1].label: expected ${CONTEST_LABEL_REASON}.`,
            );
            expect(refusalTextOf({
              value: [
                {
                  label: 'CANDIDATE "translate"',
                  found: [NAPS,],
                },
                {
                  label: 'CANDIDATE "repair"',
                  found: [NAPS,],
                },
              ],
            },),).toBe(
              `ArtifactParseError: artifact parse failed at CatEntry1.tenseCandidates[1].label: expected ${CONTEST_LABEL_REASON}.`,
            );
          },
        },),

        it({
          name: 'REFUSES a label of the other shipped wording on a slice that offers one label alone',
          fn: async () => {
            expect(refusalTextOf({
              value: [{
                label: 'ARCHIVE RENDERING',
                found: [NAPS,],
              },],
              labels: ['CANDIDATE "consolidated"',],
            },),).toBe(
              'ArtifactParseError: artifact parse failed at CatEntry1.tenseCandidates[0].label: '
                + 'expected a label from CANDIDATE "consolidated", each once and in that order.',
            );
          },
        },),

        it({
          name: 'REFUSES a group whose candidates are not a list, a list with no candidate, and a candidate that is not an object',
          fn: async () => {
            expect(refusalTextOf({
              value: [{
                label: 'ARCHIVE RENDERING',
                found: 'naps',
              },],
            },),).toBe(
              'ArtifactParseError: artifact parse failed at CatEntry1.tenseCandidates[0].found: expected an array.',
            );
            expect(refusalTextOf({
              value: [{
                label: 'ARCHIVE RENDERING',
                found: [],
              },],
            },),).toBe(
              'ArtifactParseError: artifact parse failed at CatEntry1.tenseCandidates[0].found: '
                + 'expected at least one candidate, since a text without one contributes no group.',
            );
            expect(refusalTextOf({
              value: [{
                label: 'ARCHIVE RENDERING',
                found: ['naps',],
              },],
            },),).toBe(
              'ArtifactParseError: artifact parse failed at CatEntry1.tenseCandidates[0].found[0]: expected an object.',
            );
          },
        },),

        it({
          name: 'REFUSES a candidate missing its sentence, or holding a matched text or a rule that is no string',
          fn: async () => {
            expect(refusalTextOf({
              value: [{
                label: 'ARCHIVE RENDERING',
                found: [{
                  matched: NAPS.matched,
                  rule: NAPS.rule,
                },],
              },],
            },),).toBe(
              'ArtifactParseError: artifact parse failed at CatEntry1.tenseCandidates[0].found[0].sentence: expected a string.',
            );
            expect(refusalTextOf({
              value: [{
                label: 'ARCHIVE RENDERING',
                found: [{
                  ...NAPS,
                  matched: 1,
                },],
              },],
            },),).toBe(
              'ArtifactParseError: artifact parse failed at CatEntry1.tenseCandidates[0].found[0].matched: expected a string.',
            );
            expect(refusalTextOf({
              value: [{
                label: 'ARCHIVE RENDERING',
                found: [{
                  ...NAPS,
                  rule: 1,
                },],
              },],
            },),).toBe(
              'ArtifactParseError: artifact parse failed at CatEntry1.tenseCandidates[0].found[0].rule: expected a string.',
            );
          },
        },),

        it({
          name: 'REFUSES a candidate naming a rule the detector never writes',
          fn: async () => {
            expect(refusalTextOf({
              value: [{
                label: 'ARCHIVE RENDERING',
                found: [{
                  ...NAPS,
                  rule: 'guess',
                },],
              },],
            },),).toBe(
              'ArtifactParseError: artifact parse failed at CatEntry1.tenseCandidates[0].found[0].rule: '
                + 'expected a present-tense rule the detector writes.',
            );
          },
        },),

        it({
          name: 'REFUSES a group or a candidate carrying a key the writer never records',
          fn: async () => {
            expect(refusalTextOf({
              value: [{
                label: 'ARCHIVE RENDERING',
                found: [],
                verdict: 'flawed',
              },],
            },),).toBe(
              'ArtifactParseError: artifact parse failed at CatEntry1.tenseCandidates[0].verdict: '
                + 'expected no key here beyond label, found.',
            );
            expect(refusalTextOf({
              value: [{
                label: 'ARCHIVE RENDERING',
                found: [{
                  ...NAPS,
                  severity: 'major',
                },],
              },],
            },),).toBe(
              'ArtifactParseError: artifact parse failed at CatEntry1.tenseCandidates[0].found[0].severity: '
                + 'expected no key here beyond sentence, matched, rule.',
            );
          },
        },),
      ],
    },),
  ],
},);

//endregion Recorded tense candidates tests
