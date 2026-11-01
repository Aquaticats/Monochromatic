/**
 Tests for wrapping consolidation proposals before the slate is built.

 THE DEFECT THIS FILE EXISTS TO STOP is both deciders approving bytes the run
 then changes. `wrapConsolidation` used to be the only wrap on this path, and
 it runs AFTER the slate judges have chosen a rendering and the gate has
 approved it, so what shipped was not what either of them read. Measured over
 the two most recent runs of the band pair's six entries, 15 of the 16
 consolidations that shipped came back from that wrap altered.

 WHAT THIS FILE COVERS is the wrapper alone, as a function. What the STAGE does
 with it is `consolidate-slate-carries-shipping-text.unit.test.ts`, and the
 two are apart for a diagnostic reason rather than a tidiness one: the runner
 abandons a whole FILE once a describe in it fails, so end-to-end cases sharing
 a file with unit cases go unrun exactly when something has broken and their
 answer matters most.

 THE INVARIANT BOTH FILES SERVE: a candidate on the slate is the text that
 ships if it wins. Proposals arrive wrapped, the incumbent arrives untouched,
 and a governed slice arrives exactly as its producers wrote it.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { wrapReplacementText, } from '@monochromatic-dev/translation-repair-typography';
import { wrapConsolidationProposals, } from '../dist/final/node/index.mjs';
import {
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';

/**
 Roster of three, matching the sibling settle tests.
 */
const ROSTER = [
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
] as const;

/**
 One seated voice, narrowed to the roster so a fixture cannot invent a model
 the provider catalogue does not carry.
 */
type FixtureModelId = (typeof ROSTER)[number];

/**
 A proposal that changes the content rather than the line breaks, emitted on
 one line as producers do.
 */
const FRESH = 'The cat fell asleep beside the window. She woke at four in the afternoon.';

/**
 Builds one voice as the producing half hands them over.

 @param modelId - voice that wrote it

 @param translation - wording it proposed

 @returns Voice shaped as the gather round returns one

 @example
 ```ts
 const voice = voiceOf({ modelId: ROSTER[0], translation: FRESH, },);
 ```
 */
function voiceOf(
  {
    modelId,
    translation,
  }: {
    readonly modelId: FixtureModelId;
    readonly translation: string;
  },
) {
  return {
    modelId,
    value: { translation, },
  };
}

await describe({
  name: wrapConsolidationProposals.name,
  children: [
    it({
      name: 'REWRITES A FLAT PROPOSAL INTO THE LINES IT WOULD SHIP ON, and leaves a governed slice\'s '
        + 'proposals exactly as their producer wrote them. Both halves in one case because the '
        + 'second is what makes the first evidence: a wrapper that ran unconditionally would '
        + 'satisfy the first just as well',
      fn: async () => {
        const voices = ROSTER.map(function toVoice(modelId,) {
          return voiceOf({ modelId, translation: FRESH, },);
        },);

        const ungoverned = wrapConsolidationProposals({ voices, lineStructured: false, },);
        const governed = wrapConsolidationProposals({ voices, lineStructured: true, },);

        // The wrap must actually move this fixture, or neither half means
        // anything. That is this case's own positive control.
        expect(wrapReplacementText({ text: FRESH, },),).not
          .toBe(FRESH,);

        expect(ungoverned.every(function carriesWrapped(voice,): boolean {
          return voice.value
            .translation === wrapReplacementText({ text: FRESH, },);
        },),).toBe(true,);

        expect(governed.every(function carriesEmitted(voice,): boolean {
          return voice.value
            .translation === FRESH;
        },),).toBe(true,);
      },
    },),

    it({
      name: 'HANDS BACK THE VERY ARRAY IT WAS GIVEN FOR A GOVERNED SLICE, by identity rather than '
        + 'rebuilt. A verse slice must reach the slate as it did before this existed, and an '
        + 'equal-but-rebuilt array would let a later edit change what governed slices are shown '
        + 'while every value assertion still passed',
      fn: async () => {
        const voices = ROSTER.map(function toVoice(modelId,) {
          return voiceOf({ modelId, translation: FRESH, },);
        },);

        expect(wrapConsolidationProposals({ voices, lineStructured: true, },),).toBe(voices,);
      },
    },),
  ],
},);
