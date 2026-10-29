/**
 Tests for the shared critic prompt sheet:
 closed vocabularies embedded, quote rules stated, both documents
 fenced verbatim, JSON-only reply demanded.
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
  ISSUE_CATEGORIES,
  ISSUE_SEVERITIES,
} from '@monochromatic-dev/translation-repair-shared-contract';
import { buildCriticMessages, } from '../dist/final/node/index.mjs';
import { userText, } from '@monochromatic-dev/translation-repair-provider/ts/chat-message-reading.test-fixture.ts';

/**
 Invented zh source shown to critics.
 */
const SOURCE_TEXT = '## 猫的日常\n\n小猫喜欢晒太阳。\n';

/**
 Invented translation under review.
 */
const TARGET_TEXT = '## A cat\'s day\n\nThe kitten loves sunbathing.\n';

/**
 Messages the prompt tests probe.
 */
const MESSAGES = buildCriticMessages({
  sourceText: SOURCE_TEXT,
  targetText: TARGET_TEXT,
},);

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
      name: buildCriticMessages.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'opens with system instructions, then one user sheet',
          fn: async () => {
            expect(MESSAGES.map(function toRole(message,) {
              return message.role;
            },),).toEqual(['system', 'user',],);
          },
        },),
        it({
          name: 'embeds every listed category and severity in the instructions',
          fn: async () => {
            /**
             System instructions under vocabulary probing.
             */
            const system = MESSAGES[0]?.content ?? '';
            for (const category of ISSUE_CATEGORIES)
              expect(system,).toContain(category,);
            for (const severity of ISSUE_SEVERITIES)
              expect(system,).toContain(severity,);
            expect(system,).toContain('ONLY a JSON object',);
          },
        },),
        it({
          name: 'states the emotional-completeness and foreign-phrase policy',
          fn: async () => {
            /**
             System instructions under policy probing.
             */
            const system = MESSAGES[0]?.content ?? '';
            // Naturalness outranks literalness, in both directions:
            // non-literal renderings are not defects, lost feeling is.
            expect(system,).toContain('not word-for-word correspondence',);
            expect(system,).toContain('Never report a defect merely because a rendering is not literal',);
            expect(system,).toContain('style/emotional-flattening',);
            // Foreign phrases keep their wording and gain a meaning beside it.
            expect(system,).toContain('policy/foreign-phrase-gloss',);
            expect(system,).toContain('keep that phrase in its original wording AND give its meaning alongside',);
            // The exception must not swallow the source language itself:
            // preserved CJK in an English page is the untranslated defect,
            // and the block detector reads it as non-translation evidence.
            expect(system,).toContain('The ORIGINAL\'s own language is never such a phrase',);
            expect(system,).toContain('including inside quotations and stylized multilingual lines',);
          },
        },),
        it({
          name: 'requires spans be judged in context and community usage be respected',
          fn: async () => {
            /**
             System instructions under policy probing.
             */
            const system = MESSAGES[0]?.content ?? '';
            // A word choice licensed by an adjacent clause read as a wrong term
            // when the critic judged the span alone.
            expect(system,).toContain('never in isolation',);
            expect(system,).toContain('licensed by a neighbouring clause',);
            // Community shorthand read literally became a fabricated identity.
            expect(system,).toContain('in-group vocabulary',);
            expect(system,).toContain('on the strength of a literal reading alone',);
          },
        },),
        it({
          name: 'states that the original is not golden and obligatory differences are not defects',
          fn: async () => {
            /**
             System instructions under policy probing.
             */
            const system = MESSAGES[0]?.content ?? '';
            // A translation repairing a deficiency in the original is doing its
            // job; the graded sample scored five false positives that were really
            // this, where the critic treated the source as the standard.
            expect(system,).toContain('The ORIGINAL is not golden',);
            // Each language forces choices the other leaves open, so meeting the
            // translation's own requirements is never an addition or omission.
            expect(system,).toContain('is REQUIRED, not added',);
            expect(system,).toContain('Punctuation and quotation conventions differ',);
            // A distinction English cannot mark cannot be dropped by the
            // translation, so it must not be reported as an omission.
            expect(system,).toContain('cannot be carried over',);
          },
        },),
        it({
          name: 'fences both documents verbatim on the user sheet',
          fn: async () => {
            /**
             User sheet carrying the fenced pair.
             */
            const sheet = MESSAGES[1]?.content ?? '';
            expect(sheet,).toContain(`===== ORIGINAL =====\n${SOURCE_TEXT}`,);
            expect(sheet,).toContain(`===== TRANSLATION =====\n${TARGET_TEXT}`,);
            expect(sheet,).toContain('===== END =====',);
          },
        },),

        it({
          name: 'CARRIES the cited references after the pair with their rule (class thirty-five), and no block at all when the original links nowhere',
          fn: async () => {
            /**
             Sheet built with one reference line.
             */
            const sheet = buildCriticMessages({
              sourceText: SOURCE_TEXT,
              targetText: TARGET_TEXT,
              referenceContext: '- reference 1 https://cats.example/post: the kitten has an older sister.',
            },)[1]?.content ?? '';
            expect(sheet.indexOf('===== CITED REFERENCES, EVIDENCE ONLY =====',),).toBeGreaterThan(sheet.indexOf('===== TRANSLATION =====',),);
            expect(sheet,).toContain('- reference 1 https://cats.example/post: the kitten has an older sister.',);
            expect(sheet,).toContain('never report or support it as accuracy/addition',);
            expect(MESSAGES[1]?.content ?? '',).not
              .toContain('CITED REFERENCES',);
          },
        },),

        it({
          name: 'writes an empty nearby-original line when only the nearby translation is supplied',
          fn: async () => {
            /**
             Sheet built with only the neighbouring archive translation.
             */
            const sheet = buildCriticMessages({
              sourceText: SOURCE_TEXT,
              targetText: TARGET_TEXT,
              neighbouringIncumbentText: 'Archive sentence about the cat.',
            },)[1]?.content ?? '';
            expect(sheet,).toContain(
              '===== NEARBY ORIGINAL, CONTEXT ONLY =====\n\n'
                + '===== NEARBY EXISTING TRANSLATION, CONTEXT ONLY =====\n'
                + 'Archive sentence about the cat.\n',
            );
          },
        },),

        it({
          name: 'writes an empty nearby-translation line when only the nearby original is supplied',
          fn: async () => {
            /**
             Sheet built with only the neighbouring original.
             */
            const sheet = buildCriticMessages({
              sourceText: SOURCE_TEXT,
              targetText: TARGET_TEXT,
              neighbouringSourceText: '隔壁的原文。',
            },)[1]?.content ?? '';
            expect(sheet,).toContain(
              '===== NEARBY ORIGINAL, CONTEXT ONLY =====\n'
                + '隔壁的原文。\n'
                + '===== NEARBY EXISTING TRANSLATION, CONTEXT ONLY =====\n\n',
            );
          },
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
            const content = userText({ messages: buildCriticMessages({ sourceText: RULED_SOURCE, targetText: 'Line one.', },), },);

            expect(content.includes('====== ORIGINAL ======',),).toBe(true,);
            expect(content.includes('\n===== ',),).toBe(false,);
            expect(content.includes(RULED_SOURCE,),).toBe(true,);
          },
        },),
      ],
    },),
  ],
},);
