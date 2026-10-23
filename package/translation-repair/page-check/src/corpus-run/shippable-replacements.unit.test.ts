/**
 Tests for what each slice contributes to its entry's page, the replacements the page is spliced from.

 APART FROM `publish-fixed.unit.test.ts` DELIBERATELY, for the reason `publish-fixed-replacements.unit.test.ts` gives:
 a file is abandoned once any describe in it fails, so cheap cases sharing a file with expensive ones hide them
 exactly when something has broken.

 NOTHING HERE TOUCHES A DISK. The subject is a total function of an artifact.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  shippableReplacements,
  type WouldShipSource,
} from '../../dist/final/node/index.mjs';
import {
  sourceShippedArtifact,
  type SourceShippedRow,
} from './source-shipped.test-fixture.ts';
import { artifactWithAnUnfilledAnchor, } from './would-ship-unfilled-anchor.test-fixture.ts';

/**
 An archive paragraph and two anchors past its end, whose original the artifact records.
 */
const ROWS_WITH_A_TAIL: readonly SourceShippedRow[] = [
  {
    incumbentText: 'A tabby.\n',
    sourceText: '虎斑猫。\n',
  },
  {
    incumbentText: '',
    sourceText: '## 后记\n',
  },
  {
    incumbentText: '',
    sourceText: '[^1]: 注\n',
  },
];

/**
 Archive's own wording at the slice every case reads.
 */
const ARCHIVE_MIDDLE = '\nShe slept on the counter by the till.\n';

/**
 Wording a decider settled on for it.
 */
const DECIDED_MIDDLE = '\nShe slept on the counter beside the till, in the sun.\n';

/**
 Builds an artifact whose one slice reads as the wording given.

 GOES THROUGH THE CONTEST rather than the consolidation, because the contest
 is the shortest path to a chosen wording and this file is about what a
 settled wording becomes, not about which decider settled it. Which stage is
 read is `would-ship-text.unit.test.ts`.

 @param translateText - wording the translate lane offered and the contest picked

 @returns Artifact the builder reads

 @example
 ```ts
 const artifact = artifactShipping({ translateText: DECIDED_MIDDLE, },);
 ```
 */
function artifactShipping(
  { translateText, }: { readonly translateText: string; },
): WouldShipSource {
  return {
    comparison: [
      {
        sliceIndex: 1,
        incumbentKind: 'present',
        incumbentText: ARCHIVE_MIDDLE,
        repairText: ARCHIVE_MIDDLE,
        translateText,
        laneRelation: 'both-differ',
        repairOutcome: {
          kind: 'decided',
          acceptedText: ARCHIVE_MIDDLE,
        },
        translateOutcome: {
          kind: 'decided',
          acceptedText: translateText,
        },
        decisionComparison: {
          kind: 'comparable',
          verdict: 'different',
        },
        repairDelivery: { kind: 'replacement-shipped', },
        translateDelivery: { kind: 'replacement-shipped', },
      },
    ],
    consolidation: { kind: 'not-run', },
    laneSelection: {
      kind: 'contested',
      slices: [
        {
          sliceIndex: 1,
          verdict: {
            kind: 'lane-won',
            lane: 'translate',
          },
          ballots: [],
          usable: 3,
        },
      ],
    },
  };
}

/**
 Builds an artifact whose one slice is a CONTENT span both lanes emptied.

 REACHES THE SILENCE THROUGH LANES THAT AGREED, which is the only reading that
 can be silent over an archive that speaks: the two contest paths both name a
 silent archive in their own reason strings, so a slice the contest never saw
 is the way here. The contest names no slice at all for that reason.

 @returns Artifact whose one slice removes wording the archive holds

 @example
 ```ts
 const artifact = artifactWhoseLanesRemovedTheWording();
 ```
 */
function artifactWhoseLanesRemovedTheWording(): WouldShipSource {
  return {
    comparison: [
      {
        sliceIndex: 1,
        incumbentKind: 'present',
        incumbentText: ARCHIVE_MIDDLE,
        repairText: '',
        translateText: '',
        laneRelation: 'both-differ',
        repairOutcome: {
          kind: 'decided',
          acceptedText: '',
        },
        translateOutcome: {
          kind: 'decided',
          acceptedText: '',
        },
        decisionComparison: {
          kind: 'comparable',
          verdict: 'same',
        },
        repairDelivery: { kind: 'replacement-shipped', },
        translateDelivery: { kind: 'replacement-shipped', },
      },
    ],
    consolidation: { kind: 'not-run', },
    laneSelection: {
      kind: 'contested',
      slices: [],
    },
  };
}

await describe({
  name: shippableReplacements.name,
  concurrency: DEFAULT_CONCURRENCY,
  children: [
    it({
      name:
        'NAMES AN UNFILLED ANCHOR NOWHERE WHEN THE ARTIFACT RECORDS NO ORIGINAL, because a row carrying blank '
          + 'text there claims a rendering was written where none was, and `spliceSlices` refuses that '
          + 'outright. It used '
          + 'to emit one, and the refusal landed in `publishFixedPage`, which runs before the '
          + 'artifact is written: `XIEPT2` recorded the slice unfilled three and a half hours in and '
          + 'then lost the whole entry at the last step, four hours and forty-eight minutes of calls '
          + 'with no page and no artifact kept',
      fn: async () => {
        expect(shippableReplacements({ artifact: artifactWithAnUnfilledAnchor(), },),).toEqual([],);
      },
    },),

    it({
      name:
        'WRITES THE ORIGINAL AT AN UNFILLED ANCHOR THE ARTIFACT RECORDS IT FOR, with the note before the first '
          + 'slice of the run only (the owner, 2026-10-23, "Source text + note")',
      fn: async () => {
        expect(shippableReplacements({
          artifact: sourceShippedArtifact({ rows: ROWS_WITH_A_TAIL, },),
          includeSource: true,
        },),).toEqual([
          {
            sliceIndex: 0,
            replacementText: 'A tabby.\n',
          },
          {
            sliceIndex: 1,
            replacementText: '> This passage has not been translated yet. The original text follows.\n\n## 后记',
          },
          {
            sliceIndex: 2,
            replacementText: '[^1]: 注',
          },
        ],);
      },
    },),

    it({
      name:
        'WRITES THE ENGLISH ALONE WHEN NOT TOLD OTHERWISE, so a caller that never heard of the original (the '
          + 'consistency pass hands an artifact with no record of it today, and would hand one that has it the '
          + 'day its composed artifact gains `lanes`) cannot put source text into the English it rewrites',
      fn: async () => {
        expect(shippableReplacements({
          artifact: sourceShippedArtifact({ rows: ROWS_WITH_A_TAIL, },),
        },),).toEqual([
          {
            sliceIndex: 0,
            replacementText: 'A tabby.\n',
          },
        ],);
      },
    },),

    it({
      name:
        'WRITES THE ENGLISH ALONE WHEN ASKED, so the passes that rewrite English and the guards that weigh the '
          + 'page against the archive never meet the original',
      fn: async () => {
        expect(shippableReplacements({
          artifact: sourceShippedArtifact({ rows: ROWS_WITH_A_TAIL, },),
          includeSource: false,
        },),).toEqual([
          {
            sliceIndex: 0,
            replacementText: 'A tabby.\n',
          },
        ],);
      },
    },),

    it({
      name:
        'STILL HANDS A SILENT CONTENT SPAN THE EMPTY STRING, which is what makes the anchor case '
          + 'evidence about anchors rather than about silence. Both lanes removing wording the archive '
          + 'holds is a decision, and a slice the assembler is never told about keeps whatever the '
          + 'archive had, so dropping this row would republish that wording and undo them',
      fn: async () => {
        expect(shippableReplacements({ artifact: artifactWhoseLanesRemovedTheWording(), },),).toEqual([
          {
            sliceIndex: 1,
            replacementText: '',
          },
        ],);
      },
    },),

    it({
      name:
        'CARRIES THE DECIDED WORDING for a slice that settled on some, which is what makes the "STILL HANDS A '
          + 'SILENT CONTENT SPAN THE EMPTY STRING" case evidence: a builder that emitted the empty string for '
          + 'everything would satisfy it just as well and would publish an empty page',
      fn: async () => {
        expect(shippableReplacements({
          artifact: artifactShipping({ translateText: DECIDED_MIDDLE, },),
        },),).toEqual([
          {
            sliceIndex: 1,
            replacementText: DECIDED_MIDDLE,
          },
        ],);
      },
    },),
  ],
},);
