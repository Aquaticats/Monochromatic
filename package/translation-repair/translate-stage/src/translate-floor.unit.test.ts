/**
 Guards class sixty-six (XingZ615, 2026-09-19): the translate lane's slate
 carried a candidate the deterministic publication rule refused after its
 repair turn (the closing poem's attribution moved out of the quote, three
 blocks against the original's two), the judges chose it over two valid
 renderings, the lane contest flagged the winner "retryable" and moved on,
 and the consolidation withheld it as the standing. A candidate the rule
 refuses cannot ship, so it is withheld from the judges after its turn and
 kept as a finding. Cat-themed invention throughout; no corpus content
 appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract';
import { floorTranslateVoices, } from '../dist/final/node/index.mjs';
import { capturingLogger, } from '@monochromatic-dev/translation-repair-test-fixture/ts/capturing-logger.test-fixture.ts';

/**
 Original: a heading and a paragraph.
 */
const SOURCE_TEXT = '## 猫猫的一天\n\n它在窗台上打盹。';

/**
 Rendering keeping the shape.
 */
const GOOD_TEXT = '## A Day in the Cat\'s Life\n\nIt dozes on the windowsill.';

/**
 Rendering that merged the heading away.
 */
const MERGED_TEXT = 'A day in the cat\'s life: it dozes on the windowsill.';

/**
 One heard voice.

 @param modelId - who answered

 @param translation - what it rendered

 @returns Voice as the gather reports it

 @example
 ```ts
 const voice = heard({ modelId: 'hf:cat/Cat-A', translation: GOOD_TEXT, },);
 ```
 */
function heard(
  {
    modelId,
    translation,
  }: {
    readonly modelId: string;
    readonly translation: string;
  },
): Parameters<typeof floorTranslateVoices>[0]['voices'][number] {
  return {
    modelId: modelId as unknown as RosterModelId,
    value: { translation, },
  };
}

await describe({
  name: 'the translate slate carries only candidates the rule accepts (class sixty-six, XingZ615 slice 91)',
  children: [
    it({
      name: 'WITHHOLDS the refused candidate and names it in a finding',
      fn: async () => {
        const floored = floorTranslateVoices({
          voices: [
            heard({
              modelId: 'hf:cat/Cat-A',
              translation: GOOD_TEXT,
            },),
            heard({
              modelId: 'hf:cat/Cat-B',
              translation: MERGED_TEXT,
            },),
          ],
          sourceText: SOURCE_TEXT,
          incumbentText: '',
          lineStructured: false,
          l: capturingLogger({ messages: [], },),
        },);
        expect(floored.voices
          .map(function idOf(voice,): string {
            return voice.modelId;
          },),).toEqual(['hf:cat/Cat-A',],);
        expect(floored.findings
          .some(function named(finding,): boolean {
            return finding.startsWith('translate-candidate-refused (hf:cat/Cat-B)',);
          },),).toBe(true,);
      },
    },),
    it({
      name: 'KEEPS every candidate the rule accepts, with no finding',
      fn: async () => {
        const floored = floorTranslateVoices({
          voices: [
            heard({
              modelId: 'hf:cat/Cat-A',
              translation: GOOD_TEXT,
            },),
          ],
          sourceText: SOURCE_TEXT,
          incumbentText: '',
          lineStructured: false,
          l: capturingLogger({ messages: [], },),
        },);
        expect(floored.voices.length,).toBe(1,);
        expect(floored.findings,).toEqual([],);
      },
    },),
    it({
      name: 'SAYS IN THE RUN LOG which candidate it withheld and why (ledger E5): the finding reaches '
        + 'only the artifact, and a slice whose slate thinned read in the log as judges choosing among '
        + 'fewer voices with nothing naming the rule that removed one',
      fn: async () => {
        /**
         Lines the floor wrote.
         */
        const said: string[] = [];
        const floored = floorTranslateVoices({
          voices: [
            heard({
              modelId: 'hf:cat/Cat-A',
              translation: GOOD_TEXT,
            },),
            heard({
              modelId: 'hf:cat/Cat-B',
              translation: MERGED_TEXT,
            },),
          ],
          sourceText: SOURCE_TEXT,
          incumbentText: '',
          lineStructured: false,
          l: capturingLogger({ messages: said, },),
        },);
        /**
         The rule's reason as the artifact's finding carries it.
         */
        const [finding,] = floored.findings;
        /**
         Prefix the finding names the withheld model under.
         */
        const prefix = 'translate-candidate-refused (hf:cat/Cat-B): ';
        expect(finding?.startsWith(prefix,),).toBe(true,);
        expect(finding?.length,).toBeGreaterThan(prefix.length,);
        expect(said,).toHaveLength(1,);
        expect(said[0],).toContain('withheld hf:cat/Cat-B',);
        expect(said[0],).toContain(finding?.slice(prefix.length,),);
      },
    },),
  ],
},);
