/**
 Tests that the width probe hands the editors a slice whose panel accepted an
 issue with a target-side span, and refuses one whose accepted issue names
 nothing in the translation to cut an envelope from.

 WHY THE SECOND REFUSAL IS ITS OWN. The probe compares editor rosters on the
 words the editors may change. An accepted issue that quotes only the
 original (an omission the translation shows no trace of) leaves no span in
 the translation, so no envelope exists to hand an editor, and a comparison
 run over it would measure two rosters on an empty input. The refusal names
 that wall apart from a panel that accepted nothing.

 NO NETWORK. The critics and the panel answer from a script, the same report
 from every critic and the same supporting verdict from every panelist, and
 any other stage is refused by name.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import { tagged, } from '@monochromatic-dev/logger-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  type BenchSlice,
  gatherWidthInput,
} from '../../dist/final/node/index.mjs';

import { envelopeSpansOf, } from '@monochromatic-dev/translation-repair-shared-contract/ts/envelope-span.test-fixture.ts';
import { scriptedClient, } from '@monochromatic-dev/translation-repair-provider/ts/corpus-run/scripted-width-client.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import { SCRIPTED_STAGE_SETTINGS, } from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';

/**
 Logger for the probe under test.
 */
const l = tagged({ tag: 'editor-width-input-work-test', },);

/**
 Slice drawn for the bench, with a translation already in the archive.
 */
const SLICE: BenchSlice = {
  entryId: 'mittens-window',
  index: 7,
  sourceText: '小猫在窗台上打盹。它的尾巴垂在地板上。',
  incumbentText: 'The kitten dozes on the windowsill.',
  lineStructured: false,
};

/**
 Target-side wording the critics anchor their claim on, present exactly once
 in the translation.
 */
const ANCHOR = 'dozes on the windowsill';

/**
 The one verdict every panelist casts: the claim stands, with a reason.
 */
const SUPPORTED = {
  verdicts: [
    {
      claim: 1,
      vote: 'supported',
      reason: 'The sentence about the tail is missing from the translation.',
    },
  ],
};

/**
 What the probe makes of a slice whose critics file one issue with the given
 quotes and whose panel supports it.

 @param quote - quote field the issue carries, `targetQuote` or `sourceQuote`

 @returns Probe's outcome

 @example
 ```ts
 const outcome = await gathered({ quote: { targetQuote: ANCHOR, }, },);
 ```
 */
async function gathered(
  { quote, }: { readonly quote: Readonly<Record<string, string>>; },
): ReturnType<typeof gatherWidthInput> {
  return await gatherWidthInput({
    client: scriptedClient({
      script: {
        critic_report: {
          issues: [
            {
              category: 'accuracy/omission',
              severity: 'major',
              summary: '尾巴那句没有翻译。',
              ...quote,
            },
          ],
        },
        panel_ballot: SUPPORTED,
      },
    },),
    slice: SLICE,
    signal: AbortSignal.timeout(HANG_STOP_MS,),
    l,
    stageSettings: SCRIPTED_STAGE_SETTINGS,
  },);
}

await describe({
  name: gatherWidthInput.name,
  children: [
    it({
      name: 'HANDS ON A SLICE whose panel accepted an issue quoting the translation, with the accepted issue and '
        + 'the envelope cut over the quoted words',
      fn: async () => {
        /**
         What the probe made of a slice with accepted work in it.
         */
        const outcome = await gathered({ quote: { targetQuote: ANCHOR, }, },);

        if (outcome.kind !== 'ready')
          throw new Error('a slice with an accepted issue over the translation must be carried forward',);

        expect({
          entryId: outcome.input.entryId,
          sliceIndex: outcome.input.sliceIndex,
          sourceText: outcome.input.sourceText,
          targetText: outcome.input.targetText,
          statuses: outcome.input.issues.map(function statusOf(issue,): string {
            return issue.status;
          },),
          envelopes: envelopeSpansOf(outcome.input.envelopes,),
          issueIds: outcome.input.issues.map(function idOf(issue,): string {
            return issue.issueId;
          },),
          findings: outcome.input.findings,
        },).toEqual({
          entryId: 'mittens-window',
          sliceIndex: 7,
          sourceText: SLICE.sourceText,
          targetText: SLICE.incumbentText,
          statuses: ['accepted',],
          envelopes: [{
            startOffset: SLICE.incumbentText.indexOf(ANCHOR,),
            endOffset: SLICE.incumbentText.indexOf(ANCHOR,) + ANCHOR.length,
            baseText: ANCHOR,
            issueIds: outcome.input.issues.map(function idOf(issue,): string {
              return issue.issueId;
            },),
          },],
          issueIds: outcome.input.issues.map(function idOf(issue,): string {
            return issue.issueId;
          },),
          findings: [],
        },);
      },
    },),
    it({
      name: 'SKIPS a slice whose accepted issue quotes only the original, naming that no envelope can be cut, '
        + 'where a panel that accepted nothing is another refusal',
      fn: async () => {
        /**
         What the probe made of a slice whose accepted issue has no target span.
         */
        const outcome = await gathered({ quote: { sourceQuote: '它的尾巴垂在地板上', }, },);

        expect(outcome,).toEqual({
          kind: 'skipped',
          refusal: 'no-envelopes',
          entryId: 'mittens-window',
          sliceIndex: 7,
        },);
      },
    },),
  ],
},);
