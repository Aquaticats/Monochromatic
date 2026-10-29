/**
 Tests for the leaf shapes one consolidated slice is built from.

 WHY THE SHIPPED FIELD IS STRICT AND THE OTHERS ARE NOT. Every other field
 here is evidence ABOUT a decision; `shipped` is the decision's OUTPUT, and a
 consumer writes its text into the document. So the terminal and the shipped
 kind are checked against each other rather than read independently. A record
 disagreeing with itself about that would either ship a passage nobody settled
 on or silently drop one that was, and both are wrong at the page rather than
 in a report.

 FOUR CASES MAKE THAT A TABLE rather than a rule: consolidated-with-text and
 unchanged-without are the two agreements, and the two crossings are the two
 refusals. A check reading either field alone would accept all four.

 WHY THE BALLOT'S EVIDENCE FIELDS ARE CHOICES AND NOT PROSE. The gate was once
 found shipping a rendering its own ballots named faultier, because nothing
 counted them. A name outside the three would be counted as nothing and would
 weaken that evidence silently, so the lists are parsed as names rather than
 as strings.

 `artifact-two-lane-read-consolidate.ts` is the only caller, and it asks these
 about whole valid artifacts. Every refusal here is a branch no valid fixture
 reaches.

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
import {
  type ArtifactConsolidationTerminal,
  parseGateBallot,
  parseShipped,
  parseVerdict,
} from '../../dist/final/node/index.mjs';
import { SEAT_HYPER_OPENROUTER_VISION_EDITOR, } from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';

/**
 Dotted path the cases hand in, standing for one slice's field.
 */
const SLICE_PATH = 'consolidation.slices[0].shipped';

/**
 Wording a consolidated slice ships.
 */
const SHIPPED_SILL = 'The cat naps on the windowsill.';

/**
 Terminal saying this slice replaces what stood, which is the ONLY one that
 carries text.
 */
const CONSOLIDATED: ArtifactConsolidationTerminal = 'consolidated';

/**
 Terminal saying the slate was judged and the standing text kept, which is one
 of the eight that carry none.
 */
const KEPT_STANDING: ArtifactConsolidationTerminal = 'slate-endorsed-standing';

/**
 Model that wrote a verdict, from the roster rather than invented.
 */
const VERDICT_MODEL = SEAT_HYPER_OPENROUTER_VISION_EDITOR;

/**
 Ballot every gate case departs from one field at a time.

 @returns Ballot as the gate records one

 @example
 ```ts
 const ballot = validBallot();
 ```
 */
function validBallot(): Record<string, unknown> {
  return {
    choice: 'consolidated',
    unsupported: ['standing',],
    unsupportedRaw: ['the sill it never names',],
    dropped: ['neither',],
    droppedRaw: ['the windowsill',],
    reason: 'the consolidated rendering keeps the sill the original names',
  };
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: parseShipped.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'ACCEPTS a consolidated slice carrying the text it ships, which '
            + 'is one of the two ways the terminal and the shipped kind agree',
          fn: async () => {
            expect(parseShipped({
              value: {
                kind: 'consolidated',
                text: SHIPPED_SILL,
              },
              terminal: CONSOLIDATED,
              path: SLICE_PATH,
            },),)
              .toEqual({
                kind: 'consolidated',
                text: SHIPPED_SILL,
              },);
          },
        },),

        it({
          name: 'ACCEPTS an unchanged slice under a terminal that settled on no '
            + 'change, which is the other agreement',
          fn: async () => {
            expect(parseShipped({
              value: { kind: 'unchanged', },
              terminal: KEPT_STANDING,
              path: SLICE_PATH,
            },),)
              .toEqual({ kind: 'unchanged', },);
          },
        },),

        it({
          name: 'REFUSES an unchanged slice whose terminal says it consolidated, '
            + 'which would silently drop a passage somebody settled on',
          fn: async () => {
            const refusalOfSilentDrop = caught(function silentDrop() {
              parseShipped({
                value: { kind: 'unchanged', },
                terminal: CONSOLIDATED,
                path: SLICE_PATH,
              },);
            },);

            expect(refusalOfSilentDrop,).toBeInstanceOf(ArtifactParseError,);
            expect((refusalOfSilentDrop as Error).message,)
              .toBe(`artifact parse failed at ${SLICE_PATH}: expected the text a slice whose terminal is consolidated ships.`,);
          },
        },),

        it({
          name: 'REFUSES text from a slice whose terminal settled on no change, '
            + 'and NAMES that terminal, which would otherwise ship a passage '
            + 'nobody settled on',
          fn: async () => {
            const refusalOfStrayText = caught(function strayText() {
              parseShipped({
                value: {
                  kind: 'consolidated',
                  text: SHIPPED_SILL,
                },
                terminal: KEPT_STANDING,
                path: SLICE_PATH,
              },);
            },);

            expect(refusalOfStrayText,).toBeInstanceOf(ArtifactParseError,);
            expect((refusalOfStrayText as Error).message,)
              .toBe(
              `artifact parse failed at ${SLICE_PATH}: expected no text to ship, since a slice whose terminal is `
                + 'slate-endorsed-standing settled on no change.',
              );
          },
        },),

        it({
          name: 'REFUSES a shipped kind this version does not name, listing the '
            + 'ones it does',
          fn: async () => {
            const refusalOfThirdKind = caught(function thirdKind() {
              parseShipped({
                value: { kind: 'napped', },
                terminal: KEPT_STANDING,
                path: SLICE_PATH,
              },);
            },);

            expect(refusalOfThirdKind,).toBeInstanceOf(ArtifactParseError,);
            expect((refusalOfThirdKind as Error).message,).toBe(
              'artifact parse failed at consolidation.slices[0].shipped.kind: expected one of archive, consolidated, '
                + 'incumbent, unchanged.',
            );
          },
        },),

        it({
          name: 'REFUSES an unchanged slice carrying text anyway, since the field '
            + 'set is what says whether this slice replaces anything',
          fn: async () => {
            const refusalOfUnchangedText = caught(function unchangedText() {
              parseShipped({
                value: {
                  kind: 'unchanged',
                  text: SHIPPED_SILL,
                },
                terminal: KEPT_STANDING,
                path: SLICE_PATH,
              },);
            },);

            expect(refusalOfUnchangedText,).toBeInstanceOf(ArtifactParseError,);
            expect((refusalOfUnchangedText as Error).message,).toBe(
              'artifact parse failed at consolidation.slices[0].shipped.text: expected no key here beyond '
                + 'kind.',
            );
          },
        },),

        it({
          name: 'REFUSES a consolidated slice carrying a field beyond the two, '
            + 'since a reader that ignored it would drop what a writer meant',
          fn: async () => {
            const refusalOfExtraKey = caught(function extraKey() {
              parseShipped({
                value: {
                  kind: 'consolidated',
                  text: SHIPPED_SILL,
                  settledBy: 'the panel',
                },
                terminal: CONSOLIDATED,
                path: SLICE_PATH,
              },);
            },);

            expect(refusalOfExtraKey,).toBeInstanceOf(ArtifactParseError,);
            expect((refusalOfExtraKey as Error).message,).toBe(
              'artifact parse failed at consolidation.slices[0].shipped.settledBy: expected no key here '
                + 'beyond kind, text.',
            );
          },
        },),

        it({
          name: 'REFUSES shipped text that is not a string, which is the field a '
            + 'consumer writes into the document',
          fn: async () => {
            const refusalOfNonText = caught(function nonText() {
              parseShipped({
                value: {
                  kind: 'consolidated',
                  text: 42,
                },
                terminal: CONSOLIDATED,
                path: SLICE_PATH,
              },);
            },);

            expect(refusalOfNonText,).toBeInstanceOf(ArtifactParseError,);
            expect((refusalOfNonText as Error).message,).toBe(
              'artifact parse failed at consolidation.slices[0].shipped.text: expected a string.',
            );
          },
        },),

        it({
          name: 'REFUSES a shipped field that is not a record at all',
          fn: async () => {
            const refusalOfBareString = caught(function bareString() {
              parseShipped({
                value: SHIPPED_SILL,
                terminal: CONSOLIDATED,
                path: SLICE_PATH,
              },);
            },);

            expect(refusalOfBareString,).toBeInstanceOf(ArtifactParseError,);
          },
        },),
      ],
    },),

    describe({
      name: parseVerdict.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'ACCEPTS a verdict naming its author, its reading and what it '
            + 'found, which is the control the refusals depart from',
          fn: async () => {
            expect(parseVerdict({
              value: {
                modelId: VERDICT_MODEL,
                kind: 'invalid',
                findings: [
                  'the footnote marker is gone',
                  'the heading dropped a level',
                ],
              },
              path: 'consolidation.slices[0].verdicts[0]',
            },),)
              .toEqual({
                modelId: VERDICT_MODEL,
                kind: 'invalid',
                findings: [
                  'the footnote marker is gone',
                  'the heading dropped a level',
                ],
              },);
          },
        },),

        it({
          name: 'ACCEPTS a valid verdict that found nothing, since a clean '
            + 'reading is a reading and not a missing one',
          fn: async () => {
            expect(parseVerdict({
              value: {
                modelId: VERDICT_MODEL,
                kind: 'valid',
                findings: [],
              },
              path: 'consolidation.slices[0].verdicts[0]',
            },),)
              .toEqual({
                modelId: VERDICT_MODEL,
                kind: 'valid',
                findings: [],
              },);
          },
        },),

        it({
          name: 'REFUSES a verdict reading this version does not name, listing '
            + 'the two it does',
          fn: async () => {
            const refusalOfThirdKind = caught(function thirdKind() {
              parseVerdict({
                value: {
                  modelId: VERDICT_MODEL,
                  kind: 'unsure',
                  findings: [],
                },
                path: 'consolidation.slices[0].verdicts[0]',
              },);
            },);

            expect(refusalOfThirdKind,).toBeInstanceOf(ArtifactParseError,);
            expect((refusalOfThirdKind as Error).message,).toBe(
              'artifact parse failed at consolidation.slices[0].verdicts[0].kind: expected one of valid, '
                + 'invalid.',
            );
          },
        },),

        it({
          name: 'REFUSES a finding that is not a string, and NAMES ITS POSITION, '
            + 'so a reader is told which of several went wrong',
          fn: async () => {
            const refusalOfNonFinding = caught(function nonFinding() {
              parseVerdict({
                value: {
                  modelId: VERDICT_MODEL,
                  kind: 'invalid',
                  findings: [
                    'the footnote marker is gone',
                    7,
                  ],
                },
                path: 'consolidation.slices[0].verdicts[0]',
              },);
            },);

            expect(refusalOfNonFinding,).toBeInstanceOf(ArtifactParseError,);
            expect((refusalOfNonFinding as Error).message,).toBe(
              'artifact parse failed at consolidation.slices[0].verdicts[0].findings[1]: expected a string.',
            );
          },
        },),

        it({
          name: 'REFUSES a verdict carrying a field beyond the three',
          fn: async () => {
            const refusalOfExtraKey = caught(function extraKey() {
              parseVerdict({
                value: {
                  modelId: VERDICT_MODEL,
                  kind: 'valid',
                  findings: [],
                  confidence: 0.9,
                },
                path: 'consolidation.slices[0].verdicts[0]',
              },);
            },);

            expect(refusalOfExtraKey,).toBeInstanceOf(ArtifactParseError,);
            expect((refusalOfExtraKey as Error).message,).toBe(
              'artifact parse failed at consolidation.slices[0].verdicts[0].confidence: expected no key '
                + 'here beyond modelId, kind, findings.',
            );
          },
        },),

        it({
          name: 'REFUSES a verdict recording no author, since a verdict nobody '
            + 'wrote cannot be weighed against the writer it judges',
          fn: async () => {
            const refusalOfNoAuthor = caught(function noAuthor() {
              parseVerdict({
                value: {
                  kind: 'valid',
                  findings: [],
                },
                path: 'consolidation.slices[0].verdicts[0]',
              },);
            },);

            expect(refusalOfNoAuthor,).toBeInstanceOf(ArtifactParseError,);
            expect((refusalOfNoAuthor as Error).message,).toBe(
              'artifact parse failed at consolidation.slices[0].verdicts[0].modelId: expected a string.',
            );
          },
        },),
      ],
    },),

    describe({
      name: parseGateBallot.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'ACCEPTS a ballot naming a choice, both evidence lists in names '
            + 'and both in prose, and a reason',
          fn: async () => {
            expect(parseGateBallot({
              value: validBallot(),
              path: 'consolidation.slices[0].gate.ballots[0]',
            },),)
              .toEqual(validBallot(),);
          },
        },),

        it({
          name: 'REFUSES a choice naming a rendering this gate does not offer, '
            + 'listing the three it does',
          fn: async () => {
            const refusalOfStrayChoice = caught(function strayChoice() {
              parseGateBallot({
                value: {
                  ...validBallot(),
                  choice: 'the archive',
                },
                path: 'consolidation.slices[0].gate.ballots[0]',
              },);
            },);

            expect(refusalOfStrayChoice,).toBeInstanceOf(ArtifactParseError,);
            expect((refusalOfStrayChoice as Error).message,).toBe(
              'artifact parse failed at consolidation.slices[0].gate.ballots[0].choice: expected one of '
                + 'consolidated, standing, neither.',
            );
          },
        },),

        it({
          name: 'REFUSES an evidence list naming a rendering that does not '
            + 'exist, which is what counting the evidence turns on: a name outside the three '
            + 'would be counted as nothing, and the evidence would weaken in '
            + 'silence',
          fn: async () => {
            const refusalOfStrayName = caught(function strayName() {
              parseGateBallot({
                value: {
                  ...validBallot(),
                  unsupported: ['the archive',],
                },
                path: 'consolidation.slices[0].gate.ballots[0]',
              },);
            },);

            expect(refusalOfStrayName,).toBeInstanceOf(ArtifactParseError,);
            expect((refusalOfStrayName as Error).message,).toBe(
              'artifact parse failed at consolidation.slices[0].gate.ballots[0].unsupported[0]: expected '
                + 'one of consolidated, standing, neither.',
            );
          },
        },),

        it({
          name: 'REFUSES a prose list holding something that is not prose, on '
            + 'the raw side where any wording is allowed but a number is not',
          fn: async () => {
            const refusalOfNonProse = caught(function nonProse() {
              parseGateBallot({
                value: {
                  ...validBallot(),
                  droppedRaw: [
                    'the windowsill',
                    3,
                  ],
                },
                path: 'consolidation.slices[0].gate.ballots[0]',
              },);
            },);

            expect(refusalOfNonProse,).toBeInstanceOf(ArtifactParseError,);
            expect((refusalOfNonProse as Error).message,).toBe(
              'artifact parse failed at consolidation.slices[0].gate.ballots[0].droppedRaw[1]: expected a '
                + 'string.',
            );
          },
        },),

        it({
          name: 'REFUSES a ballot recording no reason, since the reason is what '
            + 'a reader weighs a choice by',
          fn: async () => {
            /**
             Ballot built without a reason, which the parser reads as absent
             rather than as blank.
             */
            const {
              reason,
              ...withoutReason
            } = validBallot();
            expect(typeof reason,).toBe('string',);

            const refusalOfNoReason = caught(function noReason() {
              parseGateBallot({
                value: withoutReason,
                path: 'consolidation.slices[0].gate.ballots[0]',
              },);
            },);

            expect(refusalOfNoReason,).toBeInstanceOf(ArtifactParseError,);
            expect((refusalOfNoReason as Error).message,).toBe(
              'artifact parse failed at consolidation.slices[0].gate.ballots[0].reason: expected a string.',
            );
          },
        },),

        it({
          name: 'REFUSES a ballot carrying a field beyond the six, since a '
            + 'reader that ignored it would weigh less evidence than the gate '
            + 'recorded',
          fn: async () => {
            const refusalOfExtraKey = caught(function extraKey() {
              parseGateBallot({
                value: {
                  ...validBallot(),
                  weight: 0.5,
                },
                path: 'consolidation.slices[0].gate.ballots[0]',
              },);
            },);

            expect(refusalOfExtraKey,).toBeInstanceOf(ArtifactParseError,);
            expect((refusalOfExtraKey as Error).message,).toBe(
              'artifact parse failed at consolidation.slices[0].gate.ballots[0].weight: expected no key '
                + 'here beyond choice, unsupported, unsupportedRaw, dropped, droppedRaw, reason.',
            );
          },
        },),
      ],
    },),
  ],
},);
