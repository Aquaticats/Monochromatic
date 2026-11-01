/**
 Tests for the restoration judge sheet and the verdict guard:
 seed ids bind by reference number, each deleted needle renders as a
 numbered reference beside the repaired text, and only listed
 verdicts pass the guard.
 Fixtures are cat-themed invention mirroring corpus structure only.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  buildRestorationJudgeMessages,
  isRestorationJudgeWire,
  isRestorationVerdict,
  resolveRestorationJudgment,
  RESTORATION_JUDGE_VERDICTS,
} from '../dist/final/node/index.mjs';
import { userText, } from '@monochromatic-dev/translation-repair-provider/ts/chat-message-reading.test-fixture.ts';

/**
 Invented zh source the judge anchors restoration against.
 */
const SOURCE_TEXT = '## 猫的日常\n\n小猫喜欢晒太阳。小猫也喜欢追蝴蝶。\n';

/**
 Repaired translation under judgment.
 */
const REPAIRED_TEXT =
  '## A cat\'s day\n\nThe kitten loves sunbathing. The kitten also chases butterflies.\n';

/**
 Deleted needles the judge checks for restoration.
 */
const REFERENCES = [
  {
    seedId: 'seed/omission-0',
    deletedText: 'The kitten also chases butterflies.',
  },
] as const;

/**
 Original carrying a row of five equals signs, the fence the builder once
 used, on a line of its own.
 */
const RULED_SOURCE = '第一行。\n=====\n第二行。';

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: '',
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        describe({
          name: buildRestorationJudgeMessages.name,
          children: [
            it({
              name: 'binds seed ids in reference-number order',
              fn: async () => {
                const plan = buildRestorationJudgeMessages({
                  sourceText: SOURCE_TEXT,
                  repairedText: REPAIRED_TEXT,
                  references: REFERENCES,
                },);
                expect(plan.seedIds,).toEqual(['seed/omission-0',],);
              },
            },),
            it({
              name: 'shows source, repaired translation, and numbered references',
              fn: async () => {
                const plan = buildRestorationJudgeMessages({
                  sourceText: SOURCE_TEXT,
                  repairedText: REPAIRED_TEXT,
                  references: REFERENCES,
                },);

                /**
                 User sheet carrying all three fenced sections.
                 */
                const sheet = plan.messages[1]?.content ?? '';
                expect(sheet,).toContain(SOURCE_TEXT,);
                expect(sheet,).toContain('REPAIRED TRANSLATION',);
                expect(sheet,).toContain(REPAIRED_TEXT,);
                expect(sheet,).toContain('REFERENCE 1: The kitten also chases butterflies.',);
              },
            },),
            it({
              name: 'CARRIES the house rules onto a grader anchored on the Chinese',
              fn: async () => {
                // The 2026-07-17 directive anchors this grade on the source, and an
                // anchor on the source is exactly what makes completeness bias
                // possible: a repair rendered vaguer than the Chinese because
                // reader protection asks for it reads as partial restoration. The
                // block does not move the anchor, it says what a shortfall is.
                const plan = buildRestorationJudgeMessages({
                  sourceText: SOURCE_TEXT,
                  repairedText: REPAIRED_TEXT,
                  references: REFERENCES,
                },);

                /**
                 Standing rules half of the exchange.
                 */
                const system = plan.messages[0]?.content ?? '';
                expect(system,).toContain('Reader protection outranks completeness',);
                expect(system,).toContain('Chinese marks no tense',);
                expect(system,).toContain('is restored rather than partial',);
                // LEDGER S20: the reference comes from an archive written before
                // the house rules, so the reason given must not say otherwise.
                expect(system,).toContain('the REFERENCE was cut from a translation written before those rules',);
                expect(system,).not.toContain('written under the same rules',);
              },
            },),
          ],
        },),
        describe({
          name: isRestorationVerdict.name,
          children: [
            ...RESTORATION_JUDGE_VERDICTS.map(function toCase(verdict,) {
              return it({
                name: `admits ${verdict}`,
                fn: async () => {
                  expect(isRestorationVerdict(verdict,),).toBe(true,);
                },
              },);
            },),
            it({
              name: 'rejects unlisted strings and non-strings',
              fn: async () => {
                expect(isRestorationVerdict('mostly-there',),).toBe(false,);
                expect(isRestorationVerdict(1,),).toBe(false,);
              },
            },),
          ],
        },),
      ],
    },),

    describe({
      name: 'fence choice',
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'FENCES the blocks with a delimiter the enclosed text cannot reproduce, so a passage holding a row '
            + 'of five equals signs cannot close its own block and turn what follows into instructions',
          fn: async () => {
            const content = userText({ messages: buildRestorationJudgeMessages({ sourceText: RULED_SOURCE, repairedText: 'Line one.', references: [], },).messages, },);

            expect(content.includes('====== ORIGINAL ======',),).toBe(true,);
            expect(content.includes('\n===== ',),).toBe(false,);
            expect(content.includes(RULED_SOURCE,),).toBe(true,);
          },
        },),
      ],
    },),

    describe({
      name: `${isRestorationJudgeWire.name} nulls`,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'REFUSES a reply that is no object, a judgment that is no object, and a reference that is no '
            + 'number, since nothing there binds a judgment to a seed',
          fn: async () => {
            // NULL AND A NUMERAL IN QUOTES ARE THE VALUES THE GUARDS ARE FOR: a
            // number or a word is refused by the checks behind them anyway, a
            // field read off null throws, and `'1'` passes every arithmetic
            // check a number would.
            expect(isRestorationJudgeWire(null,),).toBe(false,);
            expect(isRestorationJudgeWire({ judgments: [null,], },),).toBe(false,);
            expect(isRestorationJudgeWire({
              judgments: [{ reference: '1', verdict: 'restored', },],
            },),).toBe(false,);
            expect(isRestorationJudgeWire(5,),).toBe(false,);
            expect(isRestorationJudgeWire({ judgments: [5,], },),).toBe(false,);
            expect(isRestorationJudgeWire({
              judgments: [{ reference: 'one', verdict: 'restored', },],
            },),).toBe(false,);
          },
        },),
      ],
    },),

    describe({
      name: `${resolveRestorationJudgment.name} duplicates`,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'FINDS A DUPLICATE JUDGMENT where two judgments name one reference and KEEPS THE FIRST, since '
            + 'one seed gets one verdict',
          fn: async () => {
            expect(resolveRestorationJudgment({
              wire: {
                judgments: [{
                  reference: 1,
                  verdict: 'restored',
                }, {
                  reference: 1,
                  verdict: 'not-restored',
                },],
              },
              seedIds: ['seed/omission-0',],
            },),).toEqual({
              verdicts: { 'seed/omission-0': 'restored', },
              findings: ['duplicate-judgment (1)',],
            },);
          },
        },),
      ],
    },),
  ],
},);
