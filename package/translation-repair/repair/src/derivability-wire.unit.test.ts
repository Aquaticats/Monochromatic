/**
 Tests for the derivability probe sheet and its wire constants:
 the sheet shows the source and the deleted needles only (never any
 repaired text), seed ids bind by candidate number, and the response
 format names the judgment schema.
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
  buildDerivabilityMessages,
  DERIVABILITY_RESPONSE_FORMAT,
  DERIVABILITY_VERDICTS,
  isDerivabilityVerdict,
} from '../dist/final/node/index.mjs';
import { userText, } from '@monochromatic-dev/translation-repair-provider/ts/chat-message-reading.test-fixture.ts';

/**
 Invented zh source the probe judges against.
 */
const SOURCE_TEXT = '## 猫的日常\n\n小猫喜欢晒太阳。小猫也喜欢追蝴蝶。\n';

/**
 Deleted needles whose derivability the probe questions.
 */
const REFERENCES = [
  {
    seedId: 'seed/omission-0',
    deletedText: 'The kitten also chases butterflies.',
  },
  {
    seedId: 'seed/omission-1',
    deletedText: 'The kitten loves sunbathing.',
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
          name: buildDerivabilityMessages.name,
          children: [
            it({
              name: 'binds seed ids in candidate-number order',
              fn: async () => {
                const plan = buildDerivabilityMessages({
                  sourceText: SOURCE_TEXT,
                  references: REFERENCES,
                },);
                expect(plan.seedIds,).toEqual([
                  'seed/omission-0',
                  'seed/omission-1',
                ],);
              },
            },),
            it({
              name: 'numbers each needle as a candidate on a fenced sheet',
              fn: async () => {
                const plan = buildDerivabilityMessages({
                  sourceText: SOURCE_TEXT,
                  references: REFERENCES,
                },);

                /**
                 User sheet carrying source and candidates.
                 */
                const sheet = plan.messages[1]?.content ?? '';
                expect(sheet,).toContain(SOURCE_TEXT,);
                expect(sheet,).toContain('CANDIDATE 1: The kitten also chases butterflies.',);
                expect(sheet,).toContain('CANDIDATE 2: The kitten loves sunbathing.',);
              },
            },),
            it({
              name: 'asks about information derivable from the ORIGINAL alone',
              fn: async () => {
                const plan = buildDerivabilityMessages({
                  sourceText: SOURCE_TEXT,
                  references: REFERENCES,
                },);
                expect(plan.messages[0]?.content,).toContain('ONLY the ORIGINAL',);
              },
            },),
          ],
        },),
        describe({
          name: 'DERIVABILITY_RESPONSE_FORMAT',
          children: [
            it({
              name: 'names the judgment schema over reference and verdict',
              fn: async () => {
                expect(DERIVABILITY_RESPONSE_FORMAT.json_schema.name,)
                  .toBe('derivability_judgment',);
                expect(JSON.stringify(DERIVABILITY_RESPONSE_FORMAT.json_schema.schema,),)
                  .toContain('"judgments"',);
              },
            },),
          ],
        },),
        describe({
          name: isDerivabilityVerdict.name,
          children: [
            ...DERIVABILITY_VERDICTS.map(function toCase(verdict,) {
              return it({
                name: `admits ${verdict}`,
                fn: async () => {
                  expect(isDerivabilityVerdict(verdict,),).toBe(true,);
                },
              },);
            },),
            it({
              name: 'rejects unlisted strings and non-strings',
              fn: async () => {
                expect(isDerivabilityVerdict('purrable',),).toBe(false,);
                expect(isDerivabilityVerdict(1,),).toBe(false,);
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
            const content = userText({ messages: buildDerivabilityMessages({ sourceText: RULED_SOURCE, references: [], },).messages, },);

            expect(content.includes('====== ORIGINAL ======',),).toBe(true,);
            expect(content.includes('\n===== ',),).toBe(false,);
            expect(content.includes(RULED_SOURCE,),).toBe(true,);
          },
        },),
      ],
    },),
  ],
},);
