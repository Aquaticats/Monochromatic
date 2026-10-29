/**
 Tests for the line a settled entry prints once both lanes have run.

 WHAT THESE PIN is that the line reports SETTLEMENT rather than a lane. The
 version 1 line named one lane's status as the run's status, because there was
 one lane; keeping that shape and appending translate counts beside it would
 tell every later log reader that the repair lane is the outcome and the other
 is commentary, which is the question this generation exists to leave open.

 They also pin that every number comes off the artifact rather than being
 recounted beside it, since a log line that disagrees with the file it
 describes is worse than no log line.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  ARTIFACT_SCHEMA_VERSION_V2,
  type SettledArtifact,
} from '@monochromatic-dev/translation-repair-artifact';
import { settledTallyLine, } from '../../dist/final/node/index.mjs';

/**
 Instant the injected clock reads, so no case waits on the real one.
 */
const PRINTED_AT = new Date('2026-10-22T08:10:11.123Z',);

/**
 Attempt the cases print the line for: the entry's second, the one that began under a fresh cap.
 */
const SECOND_ATTEMPT = 2;

/**
 Clock that reads the same instant every time it is asked.

 @returns Instant the line is printed at

 @example
 ```ts
 const printed = settledTallyLine({ artifact, attempt: SECOND_ATTEMPT, wallClock: fixedWallClock, },);
 ```
 */
function fixedWallClock(): Date {
  return PRINTED_AT;
}

/**
 Archive wording of the slice both lanes worked on.
 */
const ARCHIVE_NAP = 'The cat sleeps on the sill.';

/**
 Wording the repair lane shipped.
 */
const MENDED_NAP = 'The cat is asleep on the windowsill.';

/**
 Wording the translate lane shipped for the gap.
 */
const FRESH_BOWL = 'The cat has a bowl of its own.';

/**
 Archive wording with the page's curly apostrophe.
 */
const ARCHIVE_BOWL = 'The cat\u2019s bowl is full.';

/**
 Same wording with a straight apostrophe, as a model writes it.
 */
const STRAIGHT_BOWL = 'The cat\'s bowl is full.';

/**
 Wording that changes what the archive says.
 */
const EMPTY_BOWL = 'The cat\'s bowl is empty.';

/**
 Artifact these cases render.

 Built as a literal rather than through the builder, because what is under
 test is the RENDERING: a fixture assembled by the thing whose output it
 describes could hide a field the renderer reads from the wrong place.

 @returns Artifact of a two-slice entry where the lanes differ

 @example
 ```ts
 const artifact = catArtifact();
 ```
 */
function catArtifact(): SettledArtifact {
  return {
    artifactSchemaVersion: ARTIFACT_SCHEMA_VERSION_V2,
    id: 'CatEntry1',
    tip: 'a'.repeat(40,),
    pipelineDigest: `sha256-tree-v1:${'c'.repeat(64,)}`,
    corpusSha: 'b'.repeat(40,),
    callConfig: {},
    durationMs: 1_234,
    timestamp: '2026-08-16T00:00:00.000Z',
    preparation: {
      identity: `sha256-preparation-v1:${'d'.repeat(64,)}`,
      archiveText: ARCHIVE_NAP,
      sliceCount: 2,
      sourceChars: 18,
      targetChars: ARCHIVE_NAP.length,
      sourceBytes: 54,
      alignmentPairCount: 1,
      alignmentFindings: ['alignment structure-mismatch',],
    },
    lanes: {
      repair: {
        result: {
          status: 'repaired',
          issues: [
            {
              issue: { status: 'accepted', },
              resolved: true,
            },
            {
              issue: { status: 'accepted', },
              resolved: false,
            },
            {
              issue: { status: 'rejected', },
              resolved: false,
            },
          ],
          findings: [
            'alignment structure-mismatch',
            'slice 1 had no archive wording',
          ],
        },
        delivery: [
          {
            sliceIndex: 0,
            sourceText: '猫猫在窗台上睡觉。',
            incumbentKind: 'present',
            incumbentText: ARCHIVE_NAP,
            outcome: {
              kind: 'decided',
              acceptedText: MENDED_NAP,
            },
            shippedText: MENDED_NAP,
            delivery: { kind: 'replacement-shipped', },
          },
          {
            sliceIndex: 1,
            sourceText: '猫猫有自己的碗。',
            incumbentKind: 'absent',
            incumbentText: '',
            outcome: { kind: 'not-applicable', },
            shippedText: '',
            delivery: { kind: 'gap-remains', },
          },
        ],
      },
      translate: {
        result: { status: 'complete', },
        delivery: [
          {
            sliceIndex: 0,
            sourceText: '猫猫在窗台上睡觉。',
            incumbentKind: 'present',
            incumbentText: ARCHIVE_NAP,
            outcome: {
              kind: 'decided',
              acceptedText: ARCHIVE_NAP,
            },
            shippedText: ARCHIVE_NAP,
            delivery: { kind: 'incumbent-retained', },
          },
          {
            sliceIndex: 1,
            sourceText: '猫猫有自己的碗。',
            incumbentKind: 'absent',
            incumbentText: '',
            outcome: {
              kind: 'decided',
              acceptedText: FRESH_BOWL,
            },
            shippedText: FRESH_BOWL,
            delivery: { kind: 'replacement-shipped', },
          },
        ],
      },
    },
    comparison: [
      {
        sliceIndex: 0,
        incumbentKind: 'present',
        incumbentText: ARCHIVE_NAP,
        repairText: MENDED_NAP,
        translateText: ARCHIVE_NAP,
        verdict: 'repair-only',
        repairOutcome: {
          kind: 'decided',
          acceptedText: MENDED_NAP,
        },
        translateOutcome: {
          kind: 'decided',
          acceptedText: ARCHIVE_NAP,
        },
        decisionComparison: {
          kind: 'comparable',
          verdict: 'different',
        },
        repairDelivery: { kind: 'replacement-shipped', },
        translateDelivery: { kind: 'incumbent-retained', },
      },
      {
        sliceIndex: 1,
        incumbentKind: 'absent',
        incumbentText: '',
        repairText: '',
        translateText: FRESH_BOWL,
        verdict: 'translate-only',
        repairOutcome: { kind: 'not-applicable', },
        translateOutcome: {
          kind: 'decided',
          acceptedText: FRESH_BOWL,
        },
        decisionComparison: {
          kind: 'not-comparable',
          undecidedLanes: ['repair',],
        },
        repairDelivery: { kind: 'gap-remains', },
        translateDelivery: { kind: 'replacement-shipped', },
      },
    ],
    laneSelection: { kind: 'pending-human-decision', },

    // A STATED ABSENCE, which the contract requires and this fixture's type cast was
    // hiding: leaving it out made this fixture claim a shape no pipeline
    // writes, and the first reader to reach for the field found undefined.
    consolidation: { kind: 'not-run', },
    pageAssembly: {
      trimmed: [],
      withdrawn: [],
      findings: [],
    },
  } as unknown as SettledArtifact;
}

/**
 The same entry after a page-assembly pass rewrote its first slice, over an
 archive that writes curly apostrophes.

 @param trimmedText - wording the pass recorded for that slice

 @returns Artifact whose first slice the page-assembly record decides

 @example
 ```ts
 const artifact = trimmedArtifact({ trimmedText: STRAIGHT_BOWL, },);
 ```
 */
function trimmedArtifact({ trimmedText, }: { readonly trimmedText: string; },): SettledArtifact {
  /**
   Entry the pass ran over.
   */
  const base = catArtifact();
  return {
    ...base,
    preparation: {
      ...base.preparation,
      archiveText: ARCHIVE_BOWL,
      targetChars: ARCHIVE_BOWL.length,
    },
    comparison: base.comparison.map(function withBowl(row,): SettledArtifact['comparison'][number] {
      return (row.sliceIndex === 0)
        ? {
          ...row,
          incumbentText: ARCHIVE_BOWL,
          translateText: ARCHIVE_BOWL,
        }
        : row;
    },),
    pageAssembly: {
      trimmed: [
        {
          sliceIndex: 0,
          replacementText: trimmedText,
        },
      ],
      withdrawn: [],
      findings: ['bowl pass: slice 0 rewritten',],
    },
  };
}

/**
 The same entry after the contest and the consolidation ran, with the contest backing no lane at the first slice and
 the translate lane at the second.

 @param firstTerminal - how the consolidation ended at the first slice
 @param firstShipped - what the consolidation shipped at the first slice
 @param firstVerdict - what the contest recorded at the first slice

 @returns Artifact whose consolidation record the unendorsed standing count reads

 @example
 ```ts
 const artifact = consolidatedArtifact({
   firstTerminal: 'slate-declined-standing',
   firstShipped: { kind: 'unchanged', },
   firstVerdict: { kind: 'quorum-not-met', },
 },);
 ```
 */
function consolidatedArtifact(
  {
    firstTerminal,
    firstShipped,
    firstVerdict,
  }: {
    readonly firstTerminal: string;
    readonly firstShipped: Record<string, unknown>;
    readonly firstVerdict: Record<string, unknown>;
  },
): SettledArtifact {
  return {
    ...catArtifact(),
    laneSelection: {
      kind: 'contested',
      slices: [
        {
          sliceIndex: 0,
          verdict: firstVerdict,
          ballots: [],
          usable: 1,
        },
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
    consolidation: {
      kind: 'settled',
      slices: [
        {
          sliceIndex: 0,
          terminal: firstTerminal,
          shipped: firstShipped,
        },
        {
          sliceIndex: 1,
          terminal: 'gate-kept-standing',
          shipped: { kind: 'unchanged', },
        },
      ],
    },
  } as unknown as SettledArtifact;
}

/**
 Splits the line into its `key=value` pairs.

 @returns Every field of the rendered line, keyed

 @example
 ```ts
 const fields = renderedFields();
 ```
 */
function renderedFields(): Record<string, string> {
  return fieldsOf({ artifact: catArtifact(), },);
}

/**
 Splits one artifact's line into its `key=value` pairs.

 @param artifact - entry whose line is rendered

 @returns Every field of the rendered line, keyed

 @example
 ```ts
 const fields = fieldsOf({ artifact: catArtifact(), },);
 ```
 */
function fieldsOf({ artifact, }: { readonly artifact: SettledArtifact; },): Record<string, string> {
  return Object.fromEntries(
    settledTallyLine({
      artifact,
      attempt: SECOND_ATTEMPT,
      wallClock: fixedWallClock,
    },)
      .split(' ',)
      .filter(function isField(token,): boolean {
        return token.includes('=',);
      },)
      .map(function toPair(token,): readonly [string, string,] {
        /**
         Key and value either side of the first separator.
         */
        const cut = token.indexOf('=',);
        return [
          token.slice(
            0,
            cut,
          ),
          token.slice(cut + 1,),
        ];
      },),
  );
}

await describe({
  name: settledTallyLine.name,
  children: [
    it({
      name:
        'reports SETTLEMENT as the top-level status rather than either lane`s, since a line reading '
          + 'status=repaired would name the repair lane as the run`s outcome in every log anyone greps, '
          + 'which is the question nobody has answered yet',
      fn: async () => {
        /**
         Rendered fields of a two-lane entry.
         */
        const fields = renderedFields();
        expect(fields.status,).toBe('SETTLED',);
        expect(fields.selection,).toBe('pending-human-decision',);

        // Neither lane's status is reachable without its lane in the key.
        expect(fields.repairStatus,).toBe('repaired',);
        expect(fields.translateStatus,).toBe('complete',);
      },
    },),
    it({
      name:
        'STAMPS THE LINE WITH THE CLOCK TIME IT WAS PRINTED AT AND THE ATTEMPT IT BELONGS TO, straight after the status '
          + 'the runbook greps by, because a reader windowing a log by time had no way to place a tally and no way to '
          + 'tell which attempt of a re-attempted entry it closed (2026-10-22, XingZ60)',
      fn: async () => {
        /**
         Rendered fields of the entry's second attempt.
         */
        const fields = renderedFields();
        expect(fields.at,).toBe('2026-10-22T08:10:11.123Z',);
        expect(fields.attempt,).toBe('2',);

        /**
         Whole rendered line.
         */
        const line = settledTallyLine({
          artifact: catArtifact(),
          attempt: SECOND_ATTEMPT,
          wallClock: fixedWallClock,
        },);
        expect(line.startsWith(
          'TALLY CatEntry1 status=SETTLED at=2026-10-22T08:10:11.123Z attempt=2 slices=2 ',
        ),).toBe(true,);
      },
    },),
    it({
      name:
        'names the lane on every lane measurement, so a count that belongs to one lane can never be '
          + 'read as the entry`s: the translate lane files no issues at all, and an unprefixed issues= '
          + 'would be read as the entry having had three',
      fn: async () => {
        /**
         Rendered fields of the same entry.
         */
        const fields = renderedFields();
        expect(fields.repairIssues,).toBe('3',);
        expect(fields.repairAccepted,).toBe('2',);
        expect(fields.repairResolved,).toBe('1',);
        expect(fields.repairFindings,).toBe('2',);

        // The old unprefixed keys are gone rather than kept as aliases.
        for (const bare of [
          'issues',
          'accepted',
          'resolved',
          'findings',
        ]) {
          expect(Object.hasOwn(
            fields,
            bare,
          ),).toBe(false,);
        }
      },
    },),
    it({
      name:
        'counts what each document CARRIES per lane, and how many slices the two documents ended up '
          + 'disagreeing about, which is the number that says how much of an entry the open question '
          + 'actually covers',
      fn: async () => {
        /**
         Rendered fields of an entry the lanes split on.
         */
        const fields = renderedFields();

        // The repair lane mended the first slice and had no work at the anchor;
        // the translate lane kept the archive and filled the gap. So each lane
        // carries exactly one change, and they are not the same slice.
        expect(fields.repairChanged,).toBe('1',);
        expect(fields.translateChanged,).toBe('1',);
        expect(fields.documentsDiffer,).toBe('2',);
      },
    },),
    it({
      name:
        'reports the preparation`s alignment findings ONCE rather than per lane, because both lanes ran '
          + 'over one alignment and a per-lane count would report one defect in the archive twice',
      fn: async () => {
        /**
         Rendered fields, whose repair lane repeats the finding in its own
         list.
         */
        const fields = renderedFields();
        expect(fields.alignmentFindings,).toBe('1',);

        // The repair lane's own findings include that same alignment finding
        // plus one of its own, which is exactly why the two are counted apart.
        expect(fields.repairFindings,).toBe('2',);
      },
    },),
    it({
      name:
        'reads the entry id, the slice count and the duration off the ARTIFACT rather than taking them '
          + 'beside it, so a line cannot describe an entry the file does not',
      fn: async () => {
        /**
         Whole rendered line.
         */
        const line = settledTallyLine({
          artifact: catArtifact(),
          attempt: SECOND_ATTEMPT,
          wallClock: fixedWallClock,
        },);
        expect(line.startsWith('TALLY CatEntry1 status=SETTLED ',),).toBe(true,);

        /**
         Its fields.
         */
        const fields = renderedFields();
        expect(fields.slices,).toBe('2',);
        expect(fields.ms,).toBe('1234',);
      },
    },),
    it({
      name:
        'COUNTS WHAT A DOCUMENT WOULD CARRY BESIDE what each lane proposed, and reports ZERO here '
          + 'ON PURPOSE. This entry is undecided with no consolidation, so the archive stands at every '
          + 'slice that has one: two lanes proposed changes and, as things stand, a document would '
          + 'carry none of them. Read beside selection=pending-human-decision on the same line, that '
          + 'is the pending decision stated in the log rather than left to inference',
      fn: async () => {
        /**
         Its fields.
         */
        const fields = renderedFields();

        // The lane counts are untouched: they say what was PROPOSED.
        expect(fields.translateChanged,).toBe('1',);

        expect(fields.pageChanged,).toBe('0',);

        // The slice with no incumbent at all: no English stands there, which is neither a change nor a
        // retention and would be invisible inside either. The artifact records its original, so the page
        // carries that under a note (the owner, 2026-10-23, "Source text + note") and the slice is counted
        // as carrying the original, not as silent.
        expect(fields.pageSilent,).toBe('0',);
        expect(fields.pageSourceShipped,).toBe('1',);

        // Positive control: the same artifact with the original of that slice recorded blank has no passage to
        // carry, and the slice is silent again.
        const base = catArtifact();
        const silent = fieldsOf({
          artifact: {
            ...base,
            lanes: {
              ...base.lanes,
              translate: {
                ...base.lanes.translate,
                delivery: base.lanes.translate.delivery.map(function blanked(row,) {
                  return (row.sliceIndex === 1) ? { ...row, sourceText: '', } : row;
                },),
              },
            },
          },
        },);
        expect(silent.pageSilent,).toBe('1',);
        expect(silent.pageSourceShipped,).toBe('0',);
      },
    },),
    it({
      name: 'COUNTS the slices whose consolidation kept a standing text the contest did not endorse, read off the '
        + 'record and not off the stage\'s log lines (XingZ6020: 58 log lines, 56 slices)',
      fn: async () => {
        // Nothing consolidated yet: no standing was kept, so nothing is unendorsed.
        expect(renderedFields().standingUnendorsed,).toBe('0',);

        // The contest backed no lane at the first slice and the consolidation kept the standing; it backed the
        // translate lane at the second, whose standing is endorsed.
        expect(fieldsOf({
          artifact: consolidatedArtifact({
            firstTerminal: 'slate-declined-standing',
            firstShipped: { kind: 'unchanged', },
            firstVerdict: { kind: 'quorum-not-met', },
          },),
        },).standingUnendorsed,).toBe('1',);
      },
    },),
    it({
      name: 'LEAVES OUT a standing the contest endorsed, a slice the consolidation rewrote, and a slice that kept the '
        + 'archive for want of a valid wording, none of which remains retryable on an unendorsed standing',
      fn: async () => {
        // The contest settled on neither lane and endorsed the archive: the standing may ship as it is.
        expect(fieldsOf({
          artifact: consolidatedArtifact({
            firstTerminal: 'slate-declined-standing',
            firstShipped: { kind: 'unchanged', },
            firstVerdict: {
              kind: 'settled-neither',
              archive: 'endorsed',
            },
          },),
        },).standingUnendorsed,).toBe('0',);

        // The settled-neither verdict that declined the archive is unendorsed, the positive control for the
        // endorsed-standing case.
        expect(fieldsOf({
          artifact: consolidatedArtifact({
            firstTerminal: 'slate-declined-standing',
            firstShipped: { kind: 'unchanged', },
            firstVerdict: {
              kind: 'settled-neither',
              archive: 'declined',
            },
          },),
        },).standingUnendorsed,).toBe('1',);

        // The consolidation wrote fresh wording, so no unendorsed standing was kept.
        expect(fieldsOf({
          artifact: consolidatedArtifact({
            firstTerminal: 'consolidated',
            firstShipped: {
              kind: 'consolidated',
              text: MENDED_NAP,
            },
            firstVerdict: { kind: 'quorum-not-met', },
          },),
        },).standingUnendorsed,).toBe('0',);

        // No wording passed the deterministic rule, so the archive kept the slice: the defects line reports it.
        expect(fieldsOf({
          artifact: consolidatedArtifact({
            firstTerminal: 'incumbent-only',
            firstShipped: { kind: 'archive', },
            firstVerdict: { kind: 'quorum-not-met', },
          },),
        },).standingUnendorsed,).toBe('0',);
      },
    },),
    it({
      name: 'COUNTS A SLICE WHOSE SHIPPED KIND IS INCUMBENT, since the artifact does not say whether the standing it '
        + 'stood in for was ineligible (no line) or valid under an archive dispute (the line), so the count can exceed '
        + 'the stage lines by the first kind',
      fn: async () => {
        expect(fieldsOf({
          artifact: consolidatedArtifact({
            firstTerminal: 'slate-declined-standing',
            firstShipped: {
              kind: 'incumbent',
              text: ARCHIVE_NAP,
            },
            firstVerdict: { kind: 'quorum-not-met', },
          },),
        },).standingUnendorsed,).toBe('1',);
      },
    },),
    it({
      name: 'COUNTS a slice by what the page carries after typography, so a wording differing from the '
        + 'archive only in apostrophe style is no change (ledger A10)',
      fn: async () => {
        expect(fieldsOf({ artifact: trimmedArtifact({ trimmedText: STRAIGHT_BOWL, },), },).pageChanged,)
          .toBe('0',);

        // Positive control: a wording that changes the words still counts.
        expect(fieldsOf({ artifact: trimmedArtifact({ trimmedText: EMPTY_BOWL, },), },).pageChanged,)
          .toBe('1',);
      },
    },),
  ],
},);
