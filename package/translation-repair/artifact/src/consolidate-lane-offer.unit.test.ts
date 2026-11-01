/**
 Tests for which lane texts a consolidation slate offers (class forty,
 2026-09-17).

 WHY THIS FILE EXISTS. The consolidation is the retry of a declined contest,
 and on Mio24 slice 17 the retry never saw the two texts the contest was
 about: the archive's one-line rendering stood, the deterministic rule
 withheld it, and the slate carried only the writers' fresh proposals. The
 lane texts join the slate exactly when the standing is not a text the
 contest endorsed and the rule admits, each only when it passes the rule
 itself and differs from the standing.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { laneTextsForSlate, } from '../dist/final/node/index.mjs';

//region Fixtures

/**
 Original carrying a link the archive's line drops.
 */
const SOURCE_TEXT = '猫在[窗边](https://example.invalid/cat)睡着了。';

/**
 The archive's rendering, which drops the link and so fails the rule.
 */
const INCUMBENT_TEXT = 'The cat slept by the window.';

/**
 What the repair lane would ship, which keeps the link.
 */
const REPAIR_TEXT = 'The cat fell asleep by the [window](https://example.invalid/cat).';

/**
 What the translate lane would ship, which keeps the link too.
 */
const TRANSLATE_TEXT = 'The cat had fallen asleep beside the [window](https://example.invalid/cat).';

/**
 The slice every case reads, which each varies.
 */
const SLICE = {
  sourceText: SOURCE_TEXT,
  incumbentText: INCUMBENT_TEXT,
  repairText: REPAIR_TEXT,
  translateText: TRANSLATE_TEXT,
  standingText: INCUMBENT_TEXT,
  standingMayShip: false,
  standingEligible: false,
};

//endregion Fixtures

await describe({
  name: laneTextsForSlate.name,
  children: [
    it({
      name: 'OFFERS both lane texts when the contest declined both and the archive stands without '
        + 'endorsement, which is the Mio24 slice 17 shape',
      fn: async () => {
        expect(laneTextsForSlate(SLICE,),).toEqual([
          {
            lane: 'repair',
            text: REPAIR_TEXT,
          },
          {
            lane: 'translate',
            text: TRANSLATE_TEXT,
          },
        ],);
      },
    },),

    it({
      name: 'OFFERS both lane texts when the standing is eligible but the contest endorsed neither, '
        + 'since a declined contest is what the consolidation retries',
      fn: async () => {
        expect(laneTextsForSlate({
          ...SLICE,
          sourceText: '猫在窗边睡着了。',
          repairText: 'The cat fell asleep by the window.',
          translateText: 'The cat had fallen asleep beside the window.',
          standingEligible: true,
        },),).toHaveLength(2,);
      },
    },),

    it({
      name: 'OFFERS nothing when the standing is a lane the contest endorsed and the rule admits, '
        + 'because the contest already chose between the lanes',
      fn: async () => {
        expect(laneTextsForSlate({
          ...SLICE,
          standingText: REPAIR_TEXT,
          standingMayShip: true,
          standingEligible: true,
        },),).toEqual([],);
      },
    },),

    it({
      name: 'LEAVES OUT the lane whose text is the standing, so a withheld standing is not offered '
        + 'back under a lane name',
      fn: async () => {
        expect(laneTextsForSlate({
          ...SLICE,
          standingText: TRANSLATE_TEXT,
        },),).toEqual([
          {
            lane: 'repair',
            text: REPAIR_TEXT,
          },
        ],);
      },
    },),

    it({
      name: 'LEAVES OUT the lane whose text is the standing with its soft line breaks elsewhere, which the site '
        + 'renders as the same page, so a withheld standing is not offered back under a lane name (ledger B26)',
      fn: async () => {
        expect(laneTextsForSlate({
          ...SLICE,
          standingText: 'The cat fell asleep by the\n[window](https://example.invalid/cat).',
        },),).toEqual([
          {
            lane: 'translate',
            text: TRANSLATE_TEXT,
          },
        ],);
      },
    },),

    it({
      name: 'LEAVES OUT a lane text the deterministic publication rule refuses, since the slate '
        + 'floor would refuse it and a judge choosing it would choose nothing',
      fn: async () => {
        expect(laneTextsForSlate({
          ...SLICE,
          translateText: 'The cat had fallen asleep beside the window.',
        },),).toEqual([
          {
            lane: 'repair',
            text: REPAIR_TEXT,
          },
        ],);
      },
    },),

    it({
      name: 'LEAVES OUT, on a line-structured slice, a lane text carrying the Chinese line of a bilingual '
        + 'pair beside its English (class one hundred two, shi_Yumiaoya16, 2026-09-23): the standing was '
        + 'refused for that very shape and the offer must read the same rule',
      fn: async () => {
        /**
         Original giving the cat's greeting twice, in Chinese and in English.
         */
        const sourceText = '> 猫说：早安，午安，晚安。\n>\n> The cat said: good morning, good afternoon, and good night.'
          + '\n>\n> 出自《猫经》\n>\n> From *The Cat Sutra*';
        /**
         The archive carries the pair once.
         */
        const incumbentText = '> The cat said: good morning, good afternoon, and good night.\n>\n> From *The Cat Sutra*';
        expect(laneTextsForSlate({
          sourceText,
          incumbentText,
          repairText: '> 猫说：早安，午安，晚安。\n>\n> The cat said: good morning, good afternoon, and good night.'
            + '\n>\n> From *The Cat Sutra*',
          translateText: '> The cat said: good morning, good afternoon, and good night.\n>\n> From *The Cat Sutra*.',
          standingText: incumbentText,
          standingMayShip: false,
          standingEligible: true,
          lineStructured: true,
        },),).toEqual([
          {
            lane: 'translate',
            text: '> The cat said: good morning, good afternoon, and good night.\n>\n> From *The Cat Sutra*.',
          },
        ],);
      },
    },),

    it({
      name: 'LEAVES OUT a blank lane text, which is a lane that proposed nothing',
      fn: async () => {
        expect(laneTextsForSlate({
          ...SLICE,
          repairText: '',
        },),).toEqual([
          {
            lane: 'translate',
            text: TRANSLATE_TEXT,
          },
        ],);
      },
    },),

    it({
      name: 'LEAVES OUT a lane text of invisible characters alone, which the deterministic rule passes over a '
        + 'plain original and no reader sees (ledger B40)',
      fn: async () => {
        expect(laneTextsForSlate({
          ...SLICE,
          sourceText: '猫在窗边睡着了。',
          repairText: '\u{3164}\u{200B}',
          translateText: 'The cat had fallen asleep beside the window.',
          standingEligible: true,
        },),).toEqual([
          {
            lane: 'translate',
            text: 'The cat had fallen asleep beside the window.',
          },
        ],);
      },
    },),

    it({
      name: 'READS A FRONT-MATTER SLICE\'S LANE TEXTS UNDER THE FRONT-MATTER RULE, leaving out one that renames a '
        + 'YAML field, which the Markdown rule the same call makes without the syntax role offers',
      fn: async () => {
        /** The slice's metadata, whose name the alias repeats. */
        const sourceText = '---\nname: 猫猫\ninfo:\n  alias: 猫猫\n---\n';
        /** The archive's metadata, which stands. */
        const page = '---\nname: Maomao\ninfo:\n  alias: Maomao\n---\n';
        /** The slice as both calls read it, apart from the syntax role. */
        const slice = {
          sourceText,
          incumbentText: page,
          repairText: '---\ntitle: Maomao Cat\ninfo:\n  alias: Maomao Cat\n---\n',
          translateText: '---\nname: Maomao Cat\ninfo:\n  alias: Maomao Cat\n---\n',
          standingText: page,
          standingMayShip: false,
          standingEligible: true,
        };
        /**
         Lanes one call offers.

         @param offered - lane texts the call put on the slate

         @returns Their lanes, in slate order
         */
        function lanesOf(offered: readonly { readonly lane: string; }[],): readonly string[] {
          return offered.map(function toLane(laneText,): string {
            return laneText.lane;
          },);
        }
        expect([
          lanesOf(laneTextsForSlate(slice,),),
          lanesOf(laneTextsForSlate({
            ...slice,
            syntax: 'front-matter',
          },),),
        ],).toEqual([
          [ 'repair', 'translate', ],
          [ 'translate', ],
        ],);
      },
    },),
  ],
},);
