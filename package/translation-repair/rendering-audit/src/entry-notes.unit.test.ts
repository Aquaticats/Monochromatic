/**
 Tests the notes an entry carries, rendered as identity-context lines.

 THE CASES MIRROR THE CORPUS'S SHAPES: one source footnote, an archive's
 editor comments (a translation hint and a glossary), a comment glossary under
 a heading, and a multi-line definition that must fold onto one line.
 Cat-themed invention throughout; no corpus content appears here.

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
  commentBody,
  commentNoteLines,
  entryNoteLines,
  foldedLine,
  footnoteNoteLines,
} from '../dist/final/node/index.mjs';

/**
 Original carrying one footnote and one comment.
 */
const SOURCE_TEXT = '---\nname: 橘子\n---\n\n她开了一家猫粮铺[^1]。\n\n<!-- 猫爬架：Cat Tree -->\n\n[^1]: 意为专卖「进口」猫粮的小店\n';

/**
 Archive carrying a multi-line comment and a two-line footnote.
 */
const TARGET_TEXT = '---\nname: Juzi\n---\n\n<!-- 翻译提示：\n\n这篇文章偶尔用猫的口吻。 -->\n\nShe ran a cat-food shop[^1].\n\n[^1]: A small shop selling\n    imported cat food.\n';

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: foldedLine.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'FOLDS every run of whitespace to one space and trims the ends, so a note stays one line',
          fn: async () => {
            expect(foldedLine({ text: '  [^1]: first\n    second\t third  ', },),).toBe('[^1]: first second third',);
            expect(foldedLine({ text: '\n \n', },),).toBe('',);
          },
        },),
      ],
    },),

    describe({
      name: commentBody.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'STRIPS the delimiters and keeps the tail of an unterminated comment',
          fn: async () => {
            expect(commentBody({ comment: '<!-- 猫爬架：Cat Tree -->', },),).toBe(' 猫爬架：Cat Tree ',);
            expect(commentBody({ comment: '<!-- never closed', },),).toBe(' never closed',);
          },
        },),
        it({
          name: 'KEEPS THE WHOLE TEXT when a finding\'s span did not start on the opening delimiter, and still strips '
            + 'the closing one',
          fn: async () => {
            expect(commentBody({ comment: 'a note the cat left -->', },),).toBe('a note the cat left ',);
            expect(commentBody({ comment: 'a note the cat left', },),).toBe('a note the cat left',);
          },
        },),
      ],
    },),

    describe({
      name: commentNoteLines.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'SAYS THE HEADING HAS NO WORDS for a comment sitting under a heading that is nothing but its opening '
            + 'marks, where the line read "under heading :" as if a word had been dropped',
          fn: async () => {
            expect(commentNoteLines({
              document: parseDocument({ text: '#\n\n<!-- The cat left a note. -->\n\nShe naps.\n', },),
              side: 'ARCHIVE',
            },),).toEqual(['- ARCHIVE editor comment under a heading that has no words: The cat left a note.',],);
          },
        },),
      ],
    },),

    describe({
      name: entryNoteLines.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'LABELS footnotes and comments by side and kind, footnotes first, the original before the '
            + 'archive, each folded onto one line',
          fn: async () => {
            /**
             Both sides parsed as preparation parses them.
             */
            const sourceDocument = parseDocument({ text: SOURCE_TEXT, },);
            /**
             Archive side.
             */
            const targetDocument = parseDocument({ text: TARGET_TEXT, },);
            expect(entryNoteLines({
              sourceDocument,
              targetDocument,
            },),).toEqual([
              '- ORIGINAL note: [^1]: 意为专卖「进口」猫粮的小店',
              '- ARCHIVE note: [^1]: A small shop selling imported cat food.',
              '- ORIGINAL editor comment before the first heading: 猫爬架：Cat Tree',
              '- ARCHIVE editor comment before the first heading: 翻译提示： 这篇文章偶尔用猫的口吻。',
            ],);
          },
        },),

        it({
          name: 'ANCHORS a comment to the heading it sits under, naming that heading on the line, since a '
            + 'note that says "this title" or "here" was carried into every slice of one entry without '
            + 'its position and seven of eight judges bound it to the wrong heading (2026-09-06)',
          fn: async () => {
            /**
             Two sections, a note under the second, and one note before any heading.
             */
            const document = parseDocument({
              text: '<!-- 全篇用第三人称讲猫的故事 -->\n\n## 小猫\n\n它睡了。\n\n### 大猫\n\n<!-- 此处小标题译作 Tomcat -->\n\n它醒了。\n',
            },);
            expect(commentNoteLines({
              document,
              side: 'ORIGINAL',
            },),).toEqual([
              '- ORIGINAL editor comment before the first heading: 全篇用第三人称讲猫的故事',
              '- ORIGINAL editor comment under heading 大猫: 此处小标题译作 Tomcat',
            ],);
          },
        },),

        it({
          name: 'EMITS NOTHING for documents carrying neither a footnote nor a comment, and no line for an '
            + 'empty comment',
          fn: async () => {
            /**
             Plain pair.
             */
            const plain = parseDocument({ text: '毛毛很可爱。\n', },);
            expect(entryNoteLines({
              sourceDocument: plain,
              targetDocument: parseDocument({ text: 'Mittens is adorable.\n', },),
            },),).toEqual([],);
            expect(commentNoteLines({
              document: parseDocument({ text: 'Body.\n\n<!-- -->\n', },),
              side: 'ARCHIVE',
            },),).toEqual([],);
            expect(footnoteNoteLines({
              document: plain,
              side: 'ORIGINAL',
            },),).toEqual([],);
          },
        },),
      ],
    },),
  ],
},);
