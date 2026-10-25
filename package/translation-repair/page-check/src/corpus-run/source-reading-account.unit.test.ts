/**
 Tests for the account of what a reader sees of a page and of a source.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  PageSyntaxError,
  readerOfPage,
  readerOfSource,
  withoutLinkTargets,
} from '../../dist/final/node/index.mjs';

await describe({
  name: 'the account of what a reader sees',
  children: [
    describe({
      name: readerOfPage.name,
      children: [
        it({
          name: 'SHOWS A BRACE WRITTEN WITH A BACKSLASH AS THE BRACE, and a link with its target marked',
          fn: async () => {
            expect(readerOfPage({ carried: String.raw`她说 \{ 然后走了。`, },),).toBe('她说 { 然后走了。',);
            expect(readerOfPage({ carried: '见 [她的主页](https://example.net/a)', },),).toBe('见 ⟦https://example.net/a⟧她的主页',);
          },
        },),
        it({
          name: 'REFUSES A PAGE THAT HOLDS AN EXPRESSION OR A TAG, which the strict grammar reads as syntax',
          fn: async () => {
            expect(function readsExpression(): string {
              return readerOfPage({ carried: '她说 {x}', },);
            },).toThrow(PageSyntaxError,);
            expect(function readsTag(): string {
              return readerOfPage({ carried: '<Foo />', },);
            },).toThrow(PageSyntaxError,);
          },
        },),
        it({
          name: 'DECODES THE TARGET OF A LINK AS FAR AS IT READS, a percent escape that is no UTF-8 and a lone percent sign',
          fn: async () => {
            expect(readerOfPage({ carried: '[虎斑](%E0%A4)', },),).toBe('⟦\uFFFD⟧虎斑',);
            expect(readerOfPage({ carried: '[虎斑](a%zzb)', },),).toBe('⟦a%zzb⟧虎斑',);
            expect(readerOfPage({ carried: '[虎斑](100%)', },),).toBe('⟦100%⟧虎斑',);
            expect(readerOfPage({ carried: '[虎斑](a%20b%E4%BD%A0%2Fc)', },),).toBe('⟦a b你%2Fc⟧虎斑',);
          },
        },),
        it({
          name: 'MASKS A COMMENT THAT CLOSES AND READS THE TEXT AFTER IT',
          fn: async () => {
            expect(readerOfPage({ carried: '她说 <!-- 注释 --> 然后走了。', },),).toBe('她说 然后走了。',);
          },
        },),
      ],
    },),
    describe({
      name: readerOfSource.name,
      children: [
        it({
          name: 'SHOWS THE TEXT OF A SOURCE AS PLAIN MARKDOWN WITH RAW HTML READS IT',
          fn: async () => {
            expect(readerOfSource({ source: '她说 { 然后走了。', },),).toBe('她说 { 然后走了。',);
            expect(readerOfSource({ source: '<div>\n猫\n', },),).toBe('<div> 猫',);
          },
        },),
        it({
          name: 'DECODES THE TARGET OF A LINK AS FAR AS IT READS, a percent escape that is no UTF-8 and a lone percent sign',
          fn: async () => {
            expect(readerOfSource({ source: '[虎斑](%E0%A4)', },),).toBe('⟦\uFFFD⟧虎斑',);
            expect(readerOfSource({ source: '[虎斑](a%zzb)', },),).toBe('⟦a%zzb⟧虎斑',);
            expect(readerOfSource({ source: '[虎斑](100%)', },),).toBe('⟦100%⟧虎斑',);
            expect(readerOfSource({ source: '[虎斑](a%20b%E4%BD%A0%2Fc)', },),).toBe('⟦a b你%2Fc⟧虎斑',);
          },
        },),
        it({
          name: 'READS A COMMENT THAT NEVER CLOSES AS IT STANDS, and a text that opens with a byte order mark after the note',
          fn: async () => {
            expect(readerOfSource({ source: '<!-- a\n猫\n', },),).toBe('<!-- a 猫',);
            expect(readerOfSource({ source: '\uFEFF猫\n', },),).toBe('猫',);
          },
        },),
      ],
    },),
    describe({
      name: withoutLinkTargets.name,
      children: [
        it({
          name: 'TAKES OUT THE MARKS AND THE TARGET OF EACH LINK AND KEEPS THE REST',
          fn: async () => {
            expect(withoutLinkTargets({ account: '⟦u⟧见 ⟦v⟧她', },),).toBe('见 她',);
            expect(withoutLinkTargets({ account: '猫', },),).toBe('猫',);
          },
        },),
      ],
    },),
    describe({
      name: PageSyntaxError.name,
      children: [
        it({
          name: 'NAMES THE NODE TYPE AND NOTHING OF THE PAGE',
          fn: async () => {
            expect(new PageSyntaxError({ type: 'mdxTextExpression', },).message,).toBe(
              'the page holds a mdxTextExpression node, which the strict grammar reads as syntax',
            );
          },
        },),
      ],
    },),
  ],
},);
