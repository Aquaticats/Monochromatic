/**
 Tests that the width probe REFUSES a slice whose panel accepted nothing.

 WHY THIS MATTERS. The repair lane edits accepted issues and nothing else, so
 a slice whose critics filed claims the panel then threw out has no work in it
 either. It looks different from a silent slice, though: claims were filed, a
 panel round was bought, and the only thing separating the two is what the
 panel decided. The refusal has to name THAT, because a reader of the probe's
 rows is trying to tell a corpus with nothing wrong in it from a critic roster
 whose claims never survive adjudication, and those are opposite findings.

 WHAT WAS MEASURED. On 2026-08-25, inverting the test that keeps only accepted
 issues, so REJECTED ones are the ones an envelope may be cut from, failed no
 test in this package. Under that inversion this slice stops being refused for
 the reason it was refused, and the probe reports the wrong wall.

 NO NETWORK. The critics file one anchorable claim and every panelist votes it
 unsupported. Any other stage is refused by name, so a guard that let rejected
 issues through would be caught reaching for the editors.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import { tagged, } from '@monochromatic-dev/logger-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import type {
  ChatJsonOutcome,
  ChatJsonRequest,
  ModelClient,
} from '@monochromatic-dev/translation-repair-provider';
import {
  type BenchSlice,
  gatherWidthInput,
} from '../../dist/final/node/index.mjs';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import { SCRIPTED_STAGE_SETTINGS, } from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';

/**
 Logger for the probe under test.
 */
const l = tagged({ tag: 'editor-width-input-panel-test', },);

//region Fixtures

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
 in the translation so it locates without ambiguity.
 */
const ANCHOR = 'dozes on the windowsill';

/**
 Answers per stage, so an unexpected round is refused by name rather than
 silently served something shaped like an answer.
 */
const SCRIPT: Readonly<Record<string, unknown>> = {
  critic_report: {
    issues: [
      {
        category: 'accuracy/omission',
        severity: 'major',
        summary: '尾巴那句没有翻译。',
        targetQuote: ANCHOR,
      },
    ],
  },
  panel_ballot: {
    verdicts: [
      {
        claim: 1,
        vote: 'unsupported',
      },
    ],
  },
};

/**
 Client filing one claim and then voting it down.
 */
const CLIENT: ModelClient = {
  chatText: async () => {
    throw new Error('chatText unused while gathering width probe input',);
  },
  chatJson: async <ValueT,>(
    request: ChatJsonRequest<ValueT>,
  ): Promise<ChatJsonOutcome<ValueT>> => {
    /**
     Stage name from the structured-output constraint.
     */
    const stage = request.responseFormat
      ?.json_schema
      .name
      ?? '';

    /**
     Scripted answer for that stage, absent when the probe asked for a round
     a slice with no accepted work has no business buying.
     */
    const scripted = SCRIPT[stage];

    if (scripted === undefined) {
      throw new Error(
        `a slice whose panel accepted nothing must stop at the panel, and this asked ${stage}`,
      );
    }

    if (!request.validate(scripted,))
      throw new Error(`scripted ${stage} answer failed its own guard`,);

    return {
      kind: 'ok',
      value: scripted,
      rawText: JSON.stringify(scripted,),
    };
  },
  quotas: async () => {
    throw new Error('quotas unused while gathering width probe input',);
  },
};

//endregion Fixtures

await describe({
  name: gatherWidthInput.name,
  children: [
    it({
      name: 'SKIPS a slice whose panel rejected every claim, naming adjudication rather than silence '
        + 'as the wall it hit, since a corpus with nothing wrong in it and a critic roster whose '
        + 'claims never survive the panel are opposite findings that would otherwise print alike',
      fn: async () => {
        /**
         What the probe made of a slice whose claims were all voted down.
         */
        const outcome = await gatherWidthInput({
          client: CLIENT,
          slice: SLICE,
          signal: AbortSignal.timeout(HANG_STOP_MS,),
          l,
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);

        if (outcome.kind !== 'skipped')
          throw new Error('a slice whose panel accepted nothing must be skipped, not carried forward',);

        // NOT `no-claims`: claims were filed and a panel round was paid for.
        // NOT `no-envelopes`: nothing reached the point of cutting one.
        expect(outcome.refusal,).toBe('no-accepted-issues',);
        expect(outcome.entryId,).toBe('mittens-window',);
        expect(outcome.sliceIndex,).toBe(7,);
      },
    },),
  ],
},);
