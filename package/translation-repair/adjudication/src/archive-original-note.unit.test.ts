/**
 Tests for where the archive is the original, said by its translators' note.

 THE OWNER'S RULE OF 2026-09-08, after the twelfth hakureico pass rewrote
 Hanasaka's letter in five places under a note saying everything below it
 was written in English: a span such a note seals ships as the archive has
 it, and a page such a note calls the author's own English is declined. The
 two wordings the pinned corpus carries are the fixtures, and the quotes note
 hakureico also carries is the case that must NOT seal.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { parseDocument, } from '@monochromatic-dev/translation-repair-shared-contract';
import {
  archiveNoteReadingsOf,
  archiveOriginalReadingOf,
  readNote,
} from '@monochromatic-dev/translation-repair-corpus-source';
import { sealedNodeIds, } from '../dist/final/node/index.mjs';

//region Fixtures

/**
 cheonwoomaeng's note: the whole page is the author's English.
 */
const WHOLE_PAGE_NOTE = '这篇文章的原文即英文，作者的第一语言为英语，请翻译时不要动本篇。';

/**
 hakureico's note above the letter: everything below is the English original.
 */
const SPAN_NOTE = '这段话以下全部，包括结尾的两句祝愿，原文都是英文，中文是反向翻译的，请仅修可能造成误解或明显的非刻意语法错误，不大修';

/**
 hakureico's first note: most quotes were English; names no span.
 */
const QUOTES_NOTE = '本文的大部分引用原文都是英文，引用部分请仅修语法和可能造成误解的错误';

/**
 An archive shaped like hakureico's last section: an intro paragraph, the
 span note, a letter and a closing line, then a heading and a paragraph the
 seal must not reach.
 */
const SPAN_ARCHIVE = '## Her Letter\n\nShe left a letter for her friend:\n\n'
  + `<!-- ${SPAN_NOTE}-->\n\n> I am never gone,\n>\n> Just changed where I live,\n\nTime to sleep friends.\n\n`
  + '## Afterword\n\nThe editors thank everyone.\n';

//endregion Fixtures

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: readNote.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'READS A NOTE THAT SPEAKS OF AN ENGLISH ORIGINAL in a wording no mark knows as an unmarked claim, '
            + 'which seals nothing but is logged, and a note that does not as advisory (ledger E12)',
          fn: async () => {
            expect([
              readNote({ note: 'This entry was originally written in English by the cat.', },),
              readNote({ note: '猫的原文是英文，请保留', },),
              readNote({ note: QUOTES_NOTE, },),
              readNote({ note: '起床战争：Bed Wars', },),
              readNote({ note: '请不要改动猫的名字', },),
            ],).toEqual([
              'unmarked-original-claim',
              'unmarked-original-claim',
              'unmarked-original-claim',
              'advisory',
              'advisory',
            ],);
            /**
             An archive whose only note speaks of an English original in an
             unknown wording.
             */
            const document = parseDocument({
              text: '## Nap\n\n<!-- This entry was originally written in English by the cat. -->\n\nThe cat slept.\n',
            },);
            expect({
              reading: archiveOriginalReadingOf({ document, },),
              notes: archiveNoteReadingsOf({ document, },),
            },).toEqual({
              reading: { kind: 'none', },
              notes: [{
                note: 'This entry was originally written in English by the cat.',
                reading: 'unmarked-original-claim',
              },],
            },);
          },
        },),
        it({
          name: 'reads the whole-page marks as whole-page, the below-original-English marks as a span, and '
            + 'the quotes note as no seal',
          fn: async () => {
            expect(readNote({ note: WHOLE_PAGE_NOTE, },),).toBe('whole-page',);
            expect(readNote({ note: '请翻译时不要动本篇', },),).toBe('whole-page',);
            // The English wording, with the pinned corpus's spelling on gqt and the
            // spelling it meant (the fourth gqt pass of 2026-09-09 repaired an
            // English original before this read as whole-page).
            expect(readNote({ note: '(Original Language: Engish)', },),).toBe('whole-page',);
            expect(readNote({ note: '(original language: English)', },),).toBe('whole-page',);
            expect(readNote({ note: SPAN_NOTE, },),).toBe('span',);
            expect(readNote({ note: QUOTES_NOTE, },),).not.toBe('span',);
            expect(readNote({ note: '起床战争：Bed Wars', },),).toBe('advisory',);
          },
        },),
      ],
    },),

    describe({
      name: archiveOriginalReadingOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'reads a whole-page note as the whole page, outranking any span note beside it',
          fn: async () => {
            /**
             Both notes on one page.
             */
            const document = parseDocument({
              text: `<!-- ${WHOLE_PAGE_NOTE} -->\n\nBody.\n\n<!-- ${SPAN_NOTE}-->\n\nMore.\n`,
            },);
            expect(archiveOriginalReadingOf({ document, },),).toStrictEqual({
              kind: 'whole-page',
              note: WHOLE_PAGE_NOTE,
            },);
          },
        },),

        it({
          name: 'seals from the end of a span note to the next heading, and no further',
          fn: async () => {
            /**
             Parsed fixture.
             */
            const document = parseDocument({ text: SPAN_ARCHIVE, },);
            /**
             The reading.
             */
            const reading = archiveOriginalReadingOf({ document, },);
            if (reading.kind !== 'spans')
              throw new Error(`expected spans, read ${reading.kind}`,);
            expect(reading.spans.length,).toBe(1,);
            /**
             The one span.
             */
            const [span,] = reading.spans;
            if (span === undefined)
              throw new Error('unreachable: one span',);
            expect(span.note,).toBe(SPAN_NOTE,);
            expect(span.startOffset,).toBe(SPAN_ARCHIVE.indexOf('-->',) + '-->'.length,);
            expect(span.endOffset,).toBe(SPAN_ARCHIVE.indexOf('## Afterword',),);
            expect(SPAN_ARCHIVE.slice(
              span.startOffset,
              span.endOffset,
            ),).toContain('I am never gone',);
            expect(SPAN_ARCHIVE.slice(
              span.startOffset,
              span.endOffset,
            ),).not
              .toContain('Afterword',);
          },
        },),

        it({
          name: 'seals to the end of the archive when no heading follows the note',
          fn: async () => {
            /**
             The fixture without its afterword.
             */
            const text = SPAN_ARCHIVE.slice(
              0,
              SPAN_ARCHIVE.indexOf('## Afterword',),
            );
            /**
             The reading.
             */
            const reading = archiveOriginalReadingOf({ document: parseDocument({ text, },), },);
            if (reading.kind !== 'spans')
              throw new Error(`expected spans, read ${reading.kind}`,);
            expect(reading.spans[0]?.endOffset,).toBe(text.length,);
          },
        },),

        it({
          name: 'reads nothing on a page with only the quotes note, a glossary note, or no note at all',
          fn: async () => {
            expect(archiveOriginalReadingOf({
              document: parseDocument({ text: `<!-- ${QUOTES_NOTE}-->\n\n> Quoted.\n`, },),
            },),).toStrictEqual({ kind: 'none', },);
            expect(archiveOriginalReadingOf({
              document: parseDocument({ text: '<!-- 起床战争：Bed Wars -->\n\nBody.\n', },),
            },),).toStrictEqual({ kind: 'none', },);
            expect(archiveOriginalReadingOf({
              document: parseDocument({ text: '## Heading\n\nBody.\n', },),
            },),).toStrictEqual({ kind: 'none', },);
          },
        },),

        it({
          name: 'keeps one span where a second span note sits inside the first reach',
          fn: async () => {
            /**
             Two span notes under one heading.
             */
            const document = parseDocument({
              text: `Intro.\n\n<!-- ${SPAN_NOTE}-->\n\n> One.\n\n<!-- ${SPAN_NOTE}-->\n\n> Two.\n`,
            },);
            /**
             The reading.
             */
            const reading = archiveOriginalReadingOf({ document, },);
            if (reading.kind !== 'spans')
              throw new Error(`expected spans, read ${reading.kind}`,);
            expect(reading.spans.length,).toBe(1,);
          },
        },),
      ],
    },),

    describe({
      name: sealedNodeIds.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'names every block lying wholly inside a span and none outside it',
          fn: async () => {
            /**
             Parsed fixture.
             */
            const document = parseDocument({ text: SPAN_ARCHIVE, },);
            /**
             The reading.
             */
            const reading = archiveOriginalReadingOf({ document, },);
            if (reading.kind !== 'spans')
              throw new Error(`expected spans, read ${reading.kind}`,);
            /**
             Sealed ids.
             */
            const sealed = sealedNodeIds({
              nodes: document.nodes,
              spans: reading.spans,
            },);
            /**
             Texts of the sealed blocks.
             */
            const sealedTexts = document.nodes
              .filter(function isSealed(node,): boolean {
                return sealed.has(node.id,);
              },)
              .map(function toText(node,): string {
                return node.text;
              },);
            expect(sealedTexts.length,).toBe(2,);
            expect(sealedTexts[0],).toContain('I am never gone',);
            expect(sealedTexts[1],).toBe('Time to sleep friends.',);
          },
        },),
      ],
    },),
  ],
},);
