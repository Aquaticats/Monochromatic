/**
 Tests for the page-level floor that keeps distinct source headings distinct.

 WHY THIS FLOOR EXISTS. On 2026-09-06 the yulianNyanner page rendered two
 different source headings as the same English word, because a translator
 note about "this title" was carried into every slice without its position.
 A slice floor cannot see two headings at once; the assembled page can. The
 pinned corpus has no source that repeats a heading and no archive that
 collapses two, so the floor refuses nothing the archives would ship.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  caught,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  assertHeadingsStayDistinct,
  CollapsedHeadingError,
} from '../../dist/final/node/index.mjs';

/**
 Original with two sections whose headings differ.
 */
const SOURCE_TEXT = '---\nname: 猫\n---\n\n## 小猫\n\n它睡了。\n\n## 大猫\n\n它醒了。\n';

await describe({
  name: assertHeadingsStayDistinct.name,
  children: [
    it({
      name: 'REFUSES a page that renders two different source headings as one heading, naming the '
        + 'entry and the count and quoting nothing (2026-09-06)',
      fn: async () => {
        /**
         What the floor threw on the collapsed page.
         */
        const refusal = caught(function publishCollapsed(): void {
          assertHeadingsStayDistinct({
            entryId: 'Cat',
            sourceText: SOURCE_TEXT,
            pageText: '---\nname: Cat\n---\n\n## Cat\n\nIt sleeps.\n\n## Cat\n\nIt wakes.\n',
          },);
        },);

        expect(refusal,).toBeInstanceOf(CollapsedHeadingError,);
        expect((refusal as Error).message,).toContain('Cat',);
        expect((refusal as Error).message,).toContain('2',);
        expect((refusal as Error).message,).not.toContain('小猫',);
        expect((refusal as Error).message,).not.toContain('It sleeps',);
      },
    },),

    it({
      name: 'PASSES a page whose headings stay as distinct as the original\'s',
      fn: async () => {
        expect(function publishDistinct(): void {
          assertHeadingsStayDistinct({
            entryId: 'Cat',
            sourceText: SOURCE_TEXT,
            pageText: '---\nname: Cat\n---\n\n## Kitten\n\nIt sleeps.\n\n## Tomcat\n\nIt wakes.\n',
          },);
        },).not.toThrow();
      },
    },),

    it({
      name: 'PASSES a page whose repeated heading repeats a heading the original repeats too, since '
        + 'identical originals may render identically',
      fn: async () => {
        expect(function publishRepeated(): void {
          assertHeadingsStayDistinct({
            entryId: 'Cat',
            sourceText: '## 猫\n\n一。\n\n## 猫\n\n二。\n',
            pageText: '## Cat\n\nOne.\n\n## Cat\n\nTwo.\n',
          },);
        },).not.toThrow();
      },
    },),

    it({
      name: 'SAYS NOTHING when the heading counts differ, since block structure is another floor\'s '
        + 'question and pairing headings by position would then compare the wrong ones',
      fn: async () => {
        expect(function publishFewer(): void {
          assertHeadingsStayDistinct({
            entryId: 'Cat',
            sourceText: SOURCE_TEXT,
            pageText: '## Cat\n\nIt sleeps and wakes.\n',
          },);
        },).not.toThrow();
      },
    },),

    it({
      name: 'STILL REFUSES a collapse where the counts differ, reading repeats rather than positions: a page '
        + 'heading repeated more often than the original repeats any heading renders two distinct ones as one '
        + '(ledger F-10: the floor went silent whenever a heading was also dropped or added)',
      fn: async () => {
        expect(function publishCollapsedAndShort(): void {
          assertHeadingsStayDistinct({
            entryId: 'Cat',
            sourceText: '## 小猫\n\n它睡了。\n\n## 大猫\n\n它醒了。\n\n## 老猫\n\n它吃了。\n',
            pageText: '## Cat\n\nIt sleeps.\n\n## Cat\n\nIt wakes and eats.\n',
          },);
        },).toThrow(CollapsedHeadingError,);
      },
    },),

    it({
      name: 'COMPARES headings by their words rather than their marks, so a level or trailing-space '
        + 'difference between two identical page headings does not hide a collapse',
      fn: async () => {
        expect(function publishMarks(): void {
          assertHeadingsStayDistinct({
            entryId: 'Cat',
            sourceText: SOURCE_TEXT,
            pageText: '## Cat\n\nIt sleeps.\n\n### Cat  \n\nIt wakes.\n',
          },);
        },).toThrow();
      },
    },),
  ],
},);
