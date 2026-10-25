/**
 Tests for how much of the ORIGINAL a fidelity judge is shown.

 Two things are pinned here, and they are the two halves of one claim. First,
 that {@link neighbouringSource} takes one section each way and nothing more,
 including at both ends of a document where there is no such section. Second,
 that the trial's sheet carries the surrounding text ONLY when a caller
 supplied it, which is what lets a narrow arm and a wide arm be compared as
 differing in exactly one thing. That second claim was argued in a comment and
 enforced nowhere, so the measured 12-of-16 against 15-of-16 rested on it
 without evidence.

 Fixtures are cat-themed invention mirroring corpus structure only.

 @module
 */

import { tagged, } from '@monochromatic-dev/module-logger/ts';
import {
  caught,
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract';
import {
  neighbouringIncumbent,
  neighbouringSource,
} from '@monochromatic-dev/translation-repair-repair';
import type { ChunkPair, } from '@monochromatic-dev/translation-repair-slice-reading';
import {
  type ChatJsonOutcome,
  type ChatJsonRequest,
  messageText,
  type ModelClient,
} from '@monochromatic-dev/translation-repair-provider';
import {
  type FidelityOutcome,
  runFidelityTrial,
} from '../dist/final/node/index.mjs';
import { sliceOf, } from '@monochromatic-dev/translation-repair-slice-reading/ts/content-slice-of.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import { SCRIPTED_STAGE_SETTINGS, } from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';

/**
 Logger the trial writes its progress to.
 */
const l = tagged({ tag: 'fidelity-window-test', },);

/**
 Label the sheet gives the neighbouring sections.

 Matched on its distinctive opening rather than in full, so rewording the
 explanatory tail does not fail a test that is about presence.
 */
const SURROUNDING_LABEL = 'SURROUNDING ORIGINAL';

/**
 Roster the sheets go to.

 THREE RATHER THAN ONE because selection requires two ballots: a lone judge
 backing a candidate, whatever its weight, still falls short, and the trial
 declines. A one-model roster would make every sheet assertion here read
 a declined run.
 */
const ROSTER = [
  'hf:cat/Cat-A',
  'hf:cat/Cat-B',
  'hf:cat/Cat-C',
].map(function toModelId(id,): RosterModelId {
  return id as unknown as RosterModelId;
},);

/**
 Three-slice document, so a middle slice has a neighbour either way.
 */
const SLICES: readonly ChunkPair[] = [
  '小猫在窗台上睡觉。\n',
  '她看着外面的鸟。\n',
  '傍晚她回到炉火旁。\n',
].map(function toSlice(
  text,
  sliceIndex,
) {
  return sliceOf({
    text,
    sliceIndex,
  },);
},);

/**
 Every sheet a run sent, in call order.
 */
type SentSheets = {
  /**
   Sheets captured so far.
   */
  readonly sheets: string[];

  /**
   Client that records them and votes for the first candidate.
   */
  readonly client: ModelClient;
};

/**
 Builds a client that records each sheet and always backs candidate one.

 WHAT IT DOES NOT DO is judge. The vote is fixed because this test is about
 what the judges are SHOWN; `judge-fidelity.unit.test.ts` covers what the
 trial makes of what they say.

 @returns Recorder and the client writing into it

 @example
 ```ts
 const recorder = recordingClient();
 ```
 */
function recordingClient(): SentSheets {
  /**
   Sheets this client was sent.
   */
  const sheets: string[] = [];
  return {
    sheets,
    client: {
      chatText: async () => {
        throw new Error('chatText unused by selection',);
      },
      quotas: async () => {
        throw new Error('quotas unused by selection',);
      },
      chatJson: async <ValueT,>(
        request: ChatJsonRequest<ValueT>,
      ): Promise<ChatJsonOutcome<ValueT>> => {
        sheets.push(request.messages
          .map(function toContent(message,) {
            return messageText({ message, },);
          },)
          .join('\n',),);

        /**
         Wire value backing the first candidate.
         */
        const value: unknown = {
          best: 1,
          assessment: 'fixture',
        };
        if (!request.validate(value,)) {
          return {
            kind: 'schema-mismatch',
            rawText: JSON.stringify(value,),
            detail: 'reply failed the wire guard',
          };
        }
        return {
          kind: 'ok',
          value: value as ValueT,
          rawText: JSON.stringify(value,),
        };
      },
    },
  };
}

/**
 Runs one trial and answers every sheet it sent, with what the trial made of
 the replies.

 The verdict comes back so a test can show the sheets were captured from a
 COMPLETED trial: a run that failed every call would also record sheets, and
 an assertion that some label is absent would pass on the wreckage.

 @param contextText - surrounding original, empty for a narrow run

 @returns Sheets the judges were sent, and the trial's verdict

 @example
 ```ts
 const run = await sheetsFor({ contextText: '', },);
 ```
 */
async function sheetsFor(
  { contextText, }: { readonly contextText: string; },
): Promise<{
  readonly sheets: readonly string[];
  readonly verdict: FidelityOutcome['verdict'];
}> {
  /**
   Recorder capturing this run.
   */
  const recorder = recordingClient();

  /**
   What the trial made of a roster backing the first candidate.
   */
  const outcome = await runFidelityTrial({
    client: recorder.client,
    trial: {
      trialId: 'cat/1',
      direction: 'preserve',
      damageKind: 'alteration',
      sourceText: '小猫在二零二三年五月搬到窗台上。\n',
      contextText,
      cleanText: 'The cat moved to the windowsill in May 2023.\n',
      damagedText: 'The cat moved to the windowsill in May 2024.\n',
      cleanFirst: true,
    },
    judgeModelIds: ROSTER,
    signal: AbortSignal.timeout(HANG_STOP_MS,),
    perCallTimeoutMs: HANG_STOP_MS,
    l,
    stageSettings: SCRIPTED_STAGE_SETTINGS,
  },);
  return {
    sheets: recorder.sheets,
    verdict: outcome.verdict,
  };
}

/**
 Pairs carrying wording on BOTH sides, which `SLICES` deliberately does not:
 that fixture leaves the target empty because the source window is all it
 needs, and reusing it here would assert against the empty string and pass for
 a function that returned nothing.

 @param text - original of this slice

 @param wording - archive English of this slice

 @param sliceIndex - slice index

 @returns Pair with both sides populated

 @example
 ```ts
 const pair = pairOf({ text: '猫。', wording: 'A cat.', sliceIndex: 0, },);
 ```
 */
function pairOf(
  {
    text,
    wording,
    sliceIndex,
  }: {
    readonly text: string;
    readonly wording: string;
    readonly sliceIndex: number;
  },
): ChunkPair {
  return {
    source: {
      sliceIndex,
      nodes: [],
      startOffset: 0,
      endOffset: text.length,
      text,
    },
    target: {
      sliceIndex,
      nodes: [],
      startOffset: 0,
      endOffset: wording.length,
      text: wording,
    },
  };
}

/**
 Three slices with an archive translation on every one.
 */
const TRANSLATED: readonly ChunkPair[] = [
  {
    text: '小猫在窗台上睡觉。',
    wording: 'The kitten sleeps on the window sill.',
  },
  {
    text: '她看着外面的鸟。',
    wording: 'She watches the birds outside.',
  },
  {
    text: '傍晚她回到炉火旁。',
    wording: 'By evening she is back beside the stove.',
  },
].map(function toPair(
  { text, wording, },
  sliceIndex,
): ChunkPair {
  return pairOf({
    text,
    wording,
    sliceIndex,
  },);
},);

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: neighbouringSource.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'joins ONE section each way for a slice in the middle, which is the window the relocation finding '
            + 'says would fix a passage carried across a single boundary',
          fn: async () => {
            expect(neighbouringSource({
              slices: SLICES,
              slicePosition: 1,
            },),).toBe('小猫在窗台上睡觉。\n\n\n傍晚她回到炉火旁。\n',);
          },
        },),
        it({
          name: 'gives the FIRST slice only its follower, since asking for index minus one at the '
            + 'start of a document must not read the end of the array',
          fn: async () => {
            expect(neighbouringSource({
              slices: SLICES,
              slicePosition: 0,
            },),).toBe('她看着外面的鸟。\n',);
          },
        },),
        it({
          name: 'gives the LAST slice only its predecessor, and reaches ONE section back rather than '
            + 'to the start of the document',
          fn: async () => {
            expect(neighbouringSource({
              slices: SLICES,
              slicePosition: 2,
            },),).toBe('她看着外面的鸟。\n',);
          },
        },),
        it({
          name: 'answers EMPTY for a lone slice, which is the value the trial reads as "render the '
            + 'narrow sheet" rather than an empty context block',
          fn: async () => {
            expect(neighbouringSource({
              slices: [sliceOf({
                text: '小猫在窗台上睡觉。\n',
                sliceIndex: 0,
              },),],
              slicePosition: 0,
            },),).toBe('',);
          },
        },),
        it({
          name: 'THROWS on an index past the end rather than answering empty, because empty is what a '
            + 'lone slice answers: a wide arm handed a stray index would send the narrow sheet, and '
            + 'the comparison would report the window as making no difference',
          fn: async () => {
            expect(function askPastEnd() {
              return neighbouringSource({
                slices: SLICES,
                slicePosition: SLICES.length,
              },);
            },).toThrow(RangeError,);
          },
        },),
        it({
          name: 'THROWS on a STAMPED chunk index that is not this array position, which is the live '
            + 'mistake already recorded once: the same number names three different things, and two of them '
            + 'silently read the wrong neighbours or none',
          fn: async () => {
            /**
             Two slices of one section, stamped with the document-wide indices they
             would carry in an entry whose earlier sections were not sliced.

             Passing `sliceIndex` here rather than the array position is the whole
             hazard: `11` and `12` are ordinary stamps, and both are outside a
             two-element array.
             */
            const stamped: readonly ChunkPair[] = [
              sliceOf({
                text: '小猫在窗台上睡觉。\n',
                sliceIndex: 11,
              },),
              sliceOf({
                text: '她看着外面的鸟。\n',
                sliceIndex: 12,
              },),
            ];
            /**
             What askByStamp raised, read for its class as well as its wording.
             */
            const refusalOfAskByStamp = caught(function askByStamp() {
              return neighbouringSource({
                slices: stamped,
                slicePosition: stamped[0]?.source
                  .sliceIndex ?? 0,
              },);
            },);

            expect(refusalOfAskByStamp,).toBeInstanceOf(RangeError,);
            expect((refusalOfAskByStamp as Error).message,).toContain('not a position in this entry',);
          },
        },),
        it({
          name: 'THROWS on a negative index rather than reading the end of the array, since a caller '
            + 'that already subtracted one would otherwise be handed the LAST slice as a neighbour of '
            + 'the first',
          fn: async () => {
            expect(function askBeforeStart() {
              return neighbouringSource({
                slices: SLICES,
                slicePosition: -1,
              },);
            },).toThrow(RangeError,);
          },
        },),
      ],
    },),

    describe({
      name: `${runFidelityTrial.name} evidence`,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'POSITIVE CONTROL: the narrow run reaches a verdict, so the sheets the other cases '
            + 'read were captured from a trial that completed rather than one that failed every call',
          fn: async () => {
            const run = await sheetsFor({ contextText: '', },);
            expect(run.verdict,).toBe('clean',);
            expect(run.sheets
              .length,).toBe(ROSTER.length,);
          },
        },),
        it({
          name: 'renders NO surrounding block on a narrow run, in EVERY judge sheet, so the arm '
            + 'measured before the window was widened is the sheet that was always sent',
          fn: async () => {
            const run = await sheetsFor({ contextText: '', },);
            expect(run.sheets
              .filter(function carriesLabel(sheet,) {
                return sheet.includes(SURROUNDING_LABEL,);
              },)
              .length,).toBe(0,);
          },
        },),
        it({
          name: 'renders the surrounding block AND its context-only caveat when a caller supplies '
            + 'one, which is the single difference between the narrow and wide arms',
          fn: async () => {
            const run = await sheetsFor({ contextText: '她看着外面的鸟。\n', },);
            expect(run.verdict,).toBe('clean',);
            expect(run.sheets
              .filter(function carriesContext(sheet,) {
                if (!sheet.includes(SURROUNDING_LABEL,))
                  return false;
                if (!sheet.includes('她看着外面的鸟。',))
                  return false;
                return sheet.includes('not expected to render this',);
              },)
              .length,).toBe(ROSTER.length,);
          },
        },),
        it({
          name: 'carries the slice ORIGINAL either way, since widening ADDS evidence rather than '
            + 'replacing what the judges were already reading',
          fn: async () => {
            /**
             Both arms, run together because neither reads the other.
             */
            const runs = await Promise.all([
              '',
              '她看着外面的鸟。\n',
            ].map(async function toRun(contextText,) {
              return await sheetsFor({ contextText, },);
            },),);
            for (const run of runs) {
              expect(run.sheets
                .filter(function carriesOriginal(sheet,) {
                  return sheet.includes('小猫在二零二三年五月搬到窗台上。',);
                },)
                .length,).toBe(ROSTER.length,);
            }
          },
        },),
      ],
    },),

    describe({
      name: neighbouringIncumbent.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'TAKES THE ARCHIVE ONE SECTION EACH WAY, which is the half of the window that shows a '
            + 'relocation: the original says each thing once in its own place, while the English can '
            + 'say it next door, and only the English side reveals where a missing passage went',
          fn: async () => {
            /**
             Archive wording either side of the middle slice.
             */
            const beside = neighbouringIncumbent({
              slices: TRANSLATED,
              slicePosition: 1,
            },);

            expect(beside.includes('window sill',),).toBe(true,);
            expect(beside.includes('the stove',),).toBe(true,);
          },
        },),

        it({
          name: 'GIVES THE FIRST SLICE ONLY ITS FOLLOWER, since asking for index minus one at the '
            + 'start of a document must not read the end of the array',
          fn: async () => {
            /**
             Archive wording beside the opening slice.
             */
            const beside = neighbouringIncumbent({
              slices: TRANSLATED,
              slicePosition: 0,
            },);

            expect(beside.includes('the stove',),).toBe(false,);
          },
        },),

        it({
          name: 'REFUSES AN INDEX THAT IS NOT A POSITION rather than returning nothing, because empty '
            + 'means a lone slice with no neighbours and a stamped index arriving here would report a '
            + 'measured null instead of the mistake it is',
          fn: async () => {
            /**
             What the call did, as a value, since the throw is what is asserted.
             */
            const outcome = (() => {
              try {
                neighbouringIncumbent({
                  slices: TRANSLATED,
                  slicePosition: TRANSLATED.length,
                },);
                return 'returned';
              }
              catch (error) {
                return (error instanceof RangeError) ? 'refused' : `threw ${String(error,)}`;
              }
            })();

            expect(outcome,).toBe('refused',);
          },
        },),
      ],
    },),
  ],
},);
