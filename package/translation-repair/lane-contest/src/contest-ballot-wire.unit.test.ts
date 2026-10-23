/**
 Tests for the pieces every two-way contest shares.

 WHAT THIS FILE EXISTS TO STOP. These were private to the lane contest until a
 second contest needed the same question over a different pair of names.
 Sharing them is only safe while the reading stays vocabulary-agnostic, so
 these exercise it on a vocabulary the lane contest never sees.

 Fixtures are cat-themed invention mirroring corpus structure only.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';
import {
  CONTEST_POLICY,
  CONTEST_REFUSAL,
  contestResponseFormat,
  findingsOrNone,
  isStringList,
  namesOneOf,
  readCandidateNames,
} from '../dist/final/node/index.mjs';

/**
 A vocabulary neither shipped contest uses.
 */
const NAMES = [
  'tabby',
  'calico',
] as const;

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: 'CONTEST_POLICY',
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'REFUSES to name any candidate, which is what makes it shareable',
          fn: async () => {
            for (const name of [ 'repair', 'translate', 'consolidated', 'standing', ])
              expect(CONTEST_POLICY.includes(`"${name}"`,),).toBe(false,);
          },
        },),
        it({
          name: 'states that the original rather than the archive is the standard',
          fn: async () => {
            expect(CONTEST_POLICY,).toContain('THE ORIGINAL IS THE STANDARD',);
            expect(CONTEST_POLICY,).toContain('never against the archive rendering',);
          },
        },),
        it({
          name: 'REFUSES to call an unnamed person a dropped proposition',
          fn: async () => {
            // The claim this replaced said a candidate omitting a declared name HAS
            // dropped something, which is the wording that made a judge abstain
            // over a declared location the passage never mentions and let a note
            // left by a FRIEND be signed with the dead person's name. The selection
            // sheet dropped it; this sheet kept it, and both decide what ships.
            expect(CONTEST_POLICY.includes('omitting one HAS dropped something',),).toBe(false,);
            expect(CONTEST_POLICY.includes('has dropped nothing',),).toBe(true,);
            expect(CONTEST_POLICY.includes('attributing the passage to someone ELSE',),).toBe(true,);
          },
        },),
        it({
          name: 'RANKS the declared spelling above the archive rendering',
          fn: async () => {
            expect(CONTEST_POLICY.includes('OUTRANK the archive rendering',),).toBe(true,);
          },
        },),
        it({
          name: 'TELLS both deciders what this corpus is written under',
          fn: async () => {
            // These two stages choose what ships and neither had the house rules.
            expect(CONTEST_POLICY.includes('Reader protection outranks completeness',),).toBe(true,);
            expect(CONTEST_POLICY.includes('Chinese marks no tense',),).toBe(true,);
            expect(CONTEST_POLICY.includes('THE HOUSE RULE WINS',),).toBe(true,);
            // Ledger S6: the precedence line once named only a criterion, and this
            // sheet asks questions and has no criteria.
            expect(CONTEST_POLICY,).toContain('A CRITERION OR ANY OTHER RULE YOU HAVE BEEN GIVEN',);
          },
        },),
        it({
          name: 'states that a declared name is not an unsupported statement',
          fn: async () => {
            expect(CONTEST_POLICY,).toContain('DECLARED NAMES ARE ATTESTED FACTS',);
          },
        },),
        it({
          name: 'offers the refusal as a verdict rather than as a failure to answer',
          fn: async () => {
            expect(CONTEST_POLICY,).toContain(`"${CONTEST_REFUSAL}"`,);
            expect(CONTEST_POLICY,).toContain('a real verdict rather than a failure to answer',);
          },
        },),
      ],
    },),

    describe({
      name: contestResponseFormat.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'names each contest separately, so two stages stay distinguishable',
          fn: async () => {
            expect(contestResponseFormat({
              schemaName: 'cat_contest',
              verdicts: ['tabby', CONTEST_REFUSAL,],
              asksArchive: false,
            },)
              .json_schema
              .name,).toBe('cat_contest',);
          },
        },),
        it({
          name: 'requires every field a ballot is read from',
          fn: async () => {
            expect(contestResponseFormat({
              schemaName: 'cat_contest',
              verdicts: ['tabby', CONTEST_REFUSAL,],
              asksArchive: false,
            },)
              .json_schema
              .schema
              .required,).toEqual([ 'assessment', 'dropped', 'unsupported', 'verdict', ],);
          },
        },),
        it({
          name: 'LISTS THE VERDICTS A BALLOT MAY CAST AS AN enum (P1), the vocabulary the contest\'s guard reads and the '
            + 'archive vocabulary where it asks one, and leaves the findings as plain strings, since a finding may '
            + 'annotate the candidate it names ("repair (adds a season)") and the reader takes the name from its start',
          fn: async () => {
            expect(contestResponseFormat({
              schemaName: 'cat_contest',
              verdicts: ['tabby', 'tortoiseshell', CONTEST_REFUSAL,],
              asksArchive: true,
              archiveVerdicts: ['publishable', 'flawed',],
            },)
              .json_schema
              .schema
              .properties,).toEqual({
                assessment: { type: 'string', },
                dropped: { type: 'array', items: { type: 'string', }, },
                unsupported: { type: 'array', items: { type: 'string', }, },
                verdict: { type: 'string', enum: ['tabby', 'tortoiseshell', CONTEST_REFUSAL,], },
                verdictOnArchive: { type: 'string', enum: ['publishable', 'flawed',], },
              },);
            expect(contestResponseFormat({
              schemaName: 'cat_contest',
              verdicts: ['tabby', CONTEST_REFUSAL,],
              asksArchive: false,
            },)
              .json_schema
              .schema
              .properties,).toEqual({
                assessment: { type: 'string', },
                dropped: { type: 'array', items: { type: 'string', }, },
                unsupported: { type: 'array', items: { type: 'string', }, },
                verdict: { type: 'string', enum: ['tabby', CONTEST_REFUSAL,], },
              },);
          },
        },),
        it({
          name: 'ASKS FOR THE ARCHIVE VERDICT ONLY WHERE THE CONTEST WANTS ONE, so the gate is not made to answer a question whose answer its own choice already carries',
          fn: async () => {
            expect(contestResponseFormat({
              schemaName: 'cat_contest',
              verdicts: ['tabby', CONTEST_REFUSAL,],
              asksArchive: true,
              archiveVerdicts: ['publishable', 'flawed',],
            },)
              .json_schema
              .schema
              .required,).toEqual([ 'assessment', 'dropped', 'unsupported', 'verdict', 'verdictOnArchive', ],);
          },
        },),
      ],
    },),

    describe({
      name: isStringList.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'ACCEPTS an empty list, since finding nothing is an answer',
          fn: async () => {
            expect(isStringList([],),).toBe(true,);
          },
        },),
        it({
          name: 'ACCEPTS strings saying anything at all, which is the whole point',
          fn: async () => {
            expect(isStringList([ 'napping in the sun', ],),).toBe(true,);
          },
        },),
        it({
          name: 'REFUSES a list carrying anything but strings',
          fn: async () => {
            expect(isStringList([ 'tabby', 1, ],),).toBe(false,);
          },
        },),
        it({
          name: 'REFUSES a value that is not a list',
          fn: async () => {
            expect(isStringList('tabby',),).toBe(false,);
          },
        },),
      ],
    },),

    describe({
      name: findingsOrNone.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'KEEPS a list of strings as it stands, and READS a field left out, null, a bare phrase or a list '
            + 'carrying anything but strings as no findings (ledger B46)',
          fn: async () => {
            /** A list of strings. */
            const listed = [ 'napping in the sun', ];
            expect([
              findingsOrNone(listed,),
              findingsOrNone(undefined,),
              findingsOrNone(null,),
              findingsOrNone('napping in the sun',),
              findingsOrNone([ 'tabby', 1, ],),
            ],).toEqual([ listed, [], [], [], [], ],);
          },
        },),
      ],
    },),

    describe({
      name: namesOneOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'ACCEPTS a name this contest allows',
          fn: async () => {
            expect(namesOneOf({
              value: 'calico',
              names: NAMES,
            },),).toBe(true,);
          },
        },),
        it({
          name: 'REFUSES a name from another contest',
          fn: async () => {
            expect(namesOneOf({
              value: 'repair',
              names: NAMES,
            },),).toBe(false,);
          },
        },),
        it({
          name: 'REFUSES a value that is not text',
          fn: async () => {
            expect(namesOneOf({
              value: 1,
              names: NAMES,
            },),).toBe(false,);
          },
        },),
      ],
    },),

    describe({
      name: readCandidateNames.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'reads a bare name as naming that candidate',
          fn: async () => {
            expect(readCandidateNames({
              findings: [ 'calico', ],
              names: NAMES,
            },),).toEqual([ 'calico', ],);
          },
        },),
        it({
          name: 'reads an annotated name as naming that candidate',
          fn: async () => {
            expect(readCandidateNames({
              findings: [ 'tabby (invents an afternoon)', ],
              names: NAMES,
            },),).toEqual([ 'tabby', ],);
          },
        },),
        it({
          name: 'REFUSES a longer word that merely begins with a name',
          fn: async () => {
            expect(readCandidateNames({
              findings: [ 'tabbyish wording', ],
              names: NAMES,
            },),).toEqual([],);
          },
        },),
        it({
          name: 'REFUSES a longer word whose next letter is accented, composed or combining, which an ASCII test '
            + 'read as the end of the name (ledger B18)',
          fn: async () => {
            expect(readCandidateNames({
              findings: [ 'tabbyé wording', 'calico\u{0301} wording', ],
              names: NAMES,
            },),).toEqual([],);
          },
        },),
        it({
          name: 'ACCEPTS a finding naming nobody by blaming nobody',
          fn: async () => {
            expect(readCandidateNames({
              findings: [ 'the second bowl', ],
              names: NAMES,
            },),).toEqual([],);
          },
        },),
        it({
          name: 'returns names in the order the contest fixed, without repeats',
          fn: async () => {
            expect(readCandidateNames({
              findings: [ 'calico drops it', 'tabby adds it', 'calico again', ],
              names: NAMES,
            },),).toEqual([ 'tabby', 'calico', ],);
          },
        },),
        it({
          name: 'ACCEPTS surrounding space and mixed case around a name',
          fn: async () => {
            expect(readCandidateNames({
              findings: [ '  Calico is missing the ledge  ', ],
              names: NAMES,
            },),).toEqual([ 'calico', ],);
          },
        },),
      ],
    },),
  ],
},);
