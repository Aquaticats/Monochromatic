/**
 Guards class ninety-seven (2026-09-23): a candidate that renders
 a passage the original addresses in the second person (你, 您) with no
 second-person pronoun and a third-person one instead is refused before any
 judge reads it, since a pronoun the original writes is rendered as written
 where it stands (class eighty-one). A greeting (你好) is no address, and a
 rendering carrying no pronoun at all is left to the judges. Cat-themed
 invention throughout; no corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  validateTranslatedSlice,
  type SliceValidation,
} from '../dist/final/node/index.mjs';

/**
 Original wish addressing the cat directly.
 */
const ADDRESSED = '明天早上，你还要陪我去院子里追那只黄蝴蝶哦！';

/**
 Rendering that keeps the address.
 */
const KEPT = 'Tomorrow morning, you still have to come chase that yellow butterfly in the yard with me!';

/**
 Rendering that turned the address into narration.
 */
const NARRATED = 'Tomorrow morning, she still has to come chase that yellow butterfly in the yard with me!';

/**
 Verdict on a candidate no floor refuses, with no page behind it.
 */
const VALID: SliceValidation = {
  kind: 'valid',
  pageGrammar: 'absent',
};

/**
 Verdict refusing a candidate that turns the address into narration.

 @param times - how often the original addresses someone, as the finding
 words it

 @param quoted - rendering's third-person pronouns, quoted and joined as
 the finding joins them

 @param rendered - how many third-person pronouns the rendering carries

 @param written - how many the original writes in the same blocks

 @returns Whole verdict, so a check reads every word of the finding

 @example
 ```ts
 switchRefusal({ times: 'once', quoted: '"she"', rendered: 1, written: 0, },);
 ```
 */
function switchRefusal(
  {
    times,
    quoted,
    rendered,
    written,
  }: {
    readonly times: string;
    readonly quoted: string;
    readonly rendered: number;
    readonly written: number;
  },
): SliceValidation {
  return {
    kind: 'invalid',
    findings: [
      'Your translation drops the address in the second person the ORIGINAL carries: where the ORIGINAL writes 你 '
        + `or 您 ${times}, your translation carries no "you" and more third-person pronouns than the ORIGINAL writes `
        + `there (${quoted}: ${String(rendered,)} against ${String(written,)}), so a pronoun stands where the address `
        + 'stood. A pronoun the ORIGINAL writes is rendered as written where it stands: address the person the ORIGINAL '
        + 'addresses.',
    ],
  };
}

/**
 Verdict on a rendering against an original, with no page behind it.

 @param sourceText - original passage

 @param candidateText - rendering under the floor

 @returns Whole verdict

 @example
 ```ts
 verdictOf({ sourceText: ADDRESSED, candidateText: KEPT, },); // VALID
 ```
 */
function verdictOf(
  {
    sourceText,
    candidateText,
  }: {
    readonly sourceText: string;
    readonly candidateText: string;
  },
): SliceValidation {
  return validateTranslatedSlice({
    sourceText,
    candidateText,
  },);
}

await describe({
  name: 'a second-person address the original passage carries (class ninety-seven)',
  children: [
    it({
      name: 'REFUSES a candidate that renders 你 with no second-person pronoun and a third-person one instead',
      fn: async () => {
        expect(verdictOf({
          sourceText: ADDRESSED,
          candidateText: NARRATED,
        },),).toEqual(switchRefusal({
          times: 'once',
          quoted: '"she"',
          rendered: 1,
          written: 0,
        },),);
      },
    },),
    it({
      name: 'ACCEPTS the address kept, a greeting rendered as a greeting, and a rendering carrying no pronoun at all',
      fn: async () => {
        expect(verdictOf({
          sourceText: ADDRESSED,
          candidateText: KEPT,
        },),).toEqual(VALID,);
        expect(verdictOf({
          sourceText: '她说「你好。再见。」',
          candidateText: 'She said “Hello. Goodbye.”',
        },),).toEqual(VALID,);
        expect(verdictOf({
          sourceText: '啊，干干你的',
          candidateText: 'Ah, wanna play?',
        },),).toEqual(VALID,);
        expect(verdictOf({
          sourceText: '<!-- 你 --> 猫猫睡了。',
          candidateText: 'The cat slept, and she dreamed.',
        },),).toEqual(VALID,);
      },
    },),
    it({
      name: 'COUNTS NOTHING in a comment left unclosed, which runs to the end of the passage, while the same 你 '
        + 'outside it is still an address',
      fn: async () => {
        expect(verdictOf({
          sourceText: '猫猫睡了。\n\n<!-- 愿你安好',
          candidateText: 'The cat slept, and she dreamed.\n\n<!-- 愿你安好',
        },),).toEqual(VALID,);
        expect(verdictOf({
          sourceText: '猫猫睡了。愿你安好。',
          candidateText: 'The cat slept, and she dreamed.',
        },),).toEqual(switchRefusal({
          times: 'once',
          quoted: '"she"',
          rendered: 1,
          written: 0,
        },),);
      },
    },),
    it({
      name: 'ACCEPTS 你看 and 你瞧 opening a clause as the imperative "look", which English writes without "you" '
        + '(class one hundred eighty-five: the rendering\'s "she" rendered the original\'s own 她)',
      fn: async () => {
        expect(verdictOf({
          sourceText: '所以她是只温柔的猫吧。你看窗台上，她在最小的角落里睡出了最甜的梦。',
          candidateText: 'So she was a gentle cat. Look at the windowsill: in the smallest corner she dreamed '
            + 'the sweetest dream.',
        },),).toEqual(VALID,);
        expect(verdictOf({
          sourceText: '「你瞧你瞧，她又醒了！」',
          candidateText: '“Look, look, she’s awake again!”',
        },),).toEqual(VALID,);
        expect(verdictOf({
          sourceText: '你看窗台上的小猫。',
          candidateText: 'Look at the little cat on the windowsill: she is asleep.',
        },),).toEqual(VALID,);
      },
    },),
    it({
      name: 'STILL REFUSES 你看 as a verb with its object or complement (被你看到, 你看到了) where the address drops',
      fn: async () => {
        expect(verdictOf({
          sourceText: '那只猫偷吃鱼干的样子，全被你看到了。',
          candidateText: 'Even the cat sneaking dried fish was seen by her.',
        },),).toEqual(switchRefusal({
          times: 'once',
          quoted: '"her"',
          rendered: 1,
          written: 0,
        },),);
        expect(verdictOf({
          sourceText: '你看到她睡着了吗？',
          candidateText: 'Did she see her fall asleep?',
        },),).toEqual(switchRefusal({
          times: 'once',
          quoted: '"she", "her"',
          rendered: 2,
          written: 1,
        },),);
      },
    },),
    it({
      name: 'ACCEPTS third-person pronouns that render the original\'s own, block by block, and 你 that addresses '
        + 'nobody (ledger F-1: the archive-against-itself replay refused correct English in seven slices)',
      fn: async () => {
        // SHAPES, each invented: a closing wish rendered as an imperative beside
        // pronouns for the original's own 他; the address in one block and a
        // pronoun the English adds in another; 其 rendered "she"; a cry whose
        // "you" English drops beside pronouns for the original's own 她; 迷你
        // (mini); 你们好 (hello, everyone); an idiom (你追我赶); the romanised
        // ta rendered "they".
        /**
         Pairs a correct rendering answers.
         */
        const accepted: readonly (readonly [string, string,])[] = [
          [
            '他说，「猫咪早已走远。」我们不会忘记他，感谢他留下的爪印。咪咪，愿你安睡。',
            'He said, “The cat has long gone.” We will not forget him, and we are grateful for the pawprints '
              + 'he left.\nSleep well, Mimi.',
          ],
          [
            '> 啊，玩你的毛线球\n\n确是一只小猫呢',
            '> Ah, go play with the yarn.\n\nA little cat she is~',
          ],
          [
            '> 当你看到此消息。\n>\n> 「纪念其在猫窝留下的足迹。」',
            '> When this message appeared:\n>\n> “In memory of the pawprints she left in the cat bed.”',
          ],
          [
            '「喵！你怎么了！」我不断地呼喊着她。',
            '“Meow! What’s wrong!” I kept calling out to her.',
          ],
          [
            '她买了一个迷你猫窝。',
            'She bought a mini cat bed.',
          ],
          [
            '你们好，她是小橘。',
            'Hello, everyone: she is Little Orange.',
          ],
          [
            '她和小黑在院子里你追我赶。',
            'She and Little Black chased each other around the yard.',
          ],
          [
            '你走后，ta 一直在等。',
            'After the cat left, they kept waiting.',
          ],
        ];
        expect(accepted.map(function verdictOfPair([sourceText, candidateText,],): SliceValidation {
          return verdictOf({
            sourceText,
            candidateText,
          },);
        },),).toEqual(accepted.map(function valid(): SliceValidation {
          return VALID;
        },),);
      },
    },),
    it({
      name: 'STILL REFUSES a person switch: a pronoun more than the original writes where its address stood, counting '
        + 'the romanised ta and not the 其 of 其他',
      fn: async () => {
        expect(verdictOf({
          sourceText: '可惜这一切戛然而止，她睡着了。\n\n愿在你的下一个世界，你还有同样的好奇心。',
          candidateText: 'It is a pity all this stopped when she fell asleep.\n\nMay she still have the same '
            + 'curiosity in her next world.',
        },),).toEqual(switchRefusal({
          times: '2 times',
          quoted: '"she", "her"',
          rendered: 2,
          written: 0,
        },),);
        expect(verdictOf({
          sourceText: '橘子，那天晚上你请我吃的鱼干真好吃。',
          candidateText: 'The dried fish she treated me to that night was delicious, Orange.',
        },),).toEqual(switchRefusal({
          times: 'once',
          quoted: '"she"',
          rendered: 1,
          written: 0,
        },),);
        expect(verdictOf({
          sourceText: '我很遗憾没有多为 ta 拍照。\n\n希望你去往没有雷雨的世界。',
          candidateText: 'I regret not taking more photos of them.\n\nI hope they have found a world without '
            + 'thunderstorms.',
        },),).toEqual(switchRefusal({
          times: 'once',
          quoted: '"they"',
          rendered: 1,
          written: 0,
        },),);
        expect(verdictOf({
          sourceText: '她说她很累。你要好好的。',
          candidateText: 'She said she was tired. May she be well.',
        },),).toEqual(switchRefusal({
          times: 'once',
          quoted: '"she"',
          rendered: 3,
          written: 2,
        },),);
        expect(verdictOf({
          sourceText: '你走后，ta 一直在等。',
          candidateText: 'After she left, they kept waiting.',
        },),).toEqual(switchRefusal({
          times: 'once',
          quoted: '"she", "they"',
          rendered: 2,
          written: 1,
        },),);
        expect(verdictOf({
          sourceText: '你和其他猫一起玩。',
          candidateText: 'She played with the other cats.',
        },),).toEqual(switchRefusal({
          times: 'once',
          quoted: '"she"',
          rendered: 1,
          written: 0,
        },),);
      },
    },),
    it({
      name: 'READS an accented name as one word, so Heřmánek and Åhe hold no "he", while a real "she" beside one is '
        + 'still refused (ledger B18)',
      fn: async () => {
        // With ASCII letters only, `Heřmánek` read as `he`, `m` and `nek`, and
        // `Åhe` as `he` past its first letter, and the floor refused a
        // rendering that addressed the cat by name.
        expect(verdictOf({
          sourceText: ADDRESSED,
          candidateText: 'Tomorrow morning, Heřmánek still has to come chase that yellow butterfly in the yard with me!',
        },),).toEqual(VALID,);
        expect(verdictOf({
          sourceText: ADDRESSED,
          candidateText: 'Tomorrow morning, Åhe still has to come chase that yellow butterfly in the yard with me!',
        },),).toEqual(VALID,);
        expect(verdictOf({
          sourceText: ADDRESSED,
          candidateText: 'Tomorrow morning, she and Heřmánek still have to come chase that yellow butterfly in the yard '
            + 'with me!',
        },),).toEqual(switchRefusal({
          times: 'once',
          quoted: '"she"',
          rendered: 1,
          written: 0,
        },),);
      },
    },),
  ],
},);
