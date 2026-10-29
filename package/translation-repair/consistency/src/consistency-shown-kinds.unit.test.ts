/**
 Tests for the kind of each unit of an English at a span edge (n160 finding Y2): which markup is a boundary, which stays hidden, and
 that the list of block tags written in this package holds every name the Markdown parser keeps.

 Fixtures are cat-themed invention.

 @module
 */

import { createRequire, } from 'node:module';
import { pathToFileURL, } from 'node:url';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  BOUNDARY_TAG_NAMES,
  unitKindsOf,
} from '../dist/final/node/index.mjs';

/**
 The path of a package as another package resolves it, so a package this one does not depend on is reached through the one that does.

 @param from - file the resolution starts from
 @param name - package name

 @returns Resolved path of the package's entry

 @example
 ```ts
 const entry = resolvedFrom({ from: import.meta.url, name: 'remark-parse', });
 ```
 */
function resolvedFrom({ from, name, }: { readonly from: string; readonly name: string; },): string {
  return createRequire(from,).resolve(name,);
}

/**
 The kinds of the units of an English, as a string with one letter per unit: s shown, h hidden, b boundary.

 @param english - English

 @returns Letters

 @example
 ```ts
 pattern({ english: 'a<br/>b', },); // 'sbbbbbs'
 ```
 */
function pattern({ english, }: { readonly english: string; },): string {
  return unitKindsOf({ english, },)
    .map(function letter(kind,): string {
      return kind.charAt(0,);
    },)
    .join('',);
}

await describe({
  name: unitKindsOf.name,
  concurrency: 1,
  children: [
    describe({
      name: 'the block tag list',
      children: [
        it({
          name: 'HOLDS every name of the parser\'s own list of HTML block names, read from the installed micromark',
          fn: async () => {
            /**
             The chain of packages from the package this one depends on for its parse to the one that holds the list.
             This package declares no parser of its own, so the chain starts at the document structure package, which declares
             `remark-parse`; resolving `remark-parse` from here passed only where a parent directory's `node_modules` held it
             (ledger B802).
             */
            let at = import.meta.url;
            for (
              const name of [
                '@monochromatic-dev/translation-repair-document-structure',
                'remark-parse',
                'mdast-util-from-markdown',
                'micromark',
                'micromark-core-commonmark',
              ] as const
            )
              at = pathToFileURL(resolvedFrom({ from: at, name, },),).href;
            /**
             The path of the parser's list.
             */
            const listPath = resolvedFrom({ from: at, name: 'micromark-util-html-tag-name', },);
            /**
             The parser's two lists: the block names, and the raw names (`pre`, `script`, `style`, `textarea`) whose elements the
             parser reads to their closing tag.
             */
            const { htmlBlockNames, htmlRawNames, } = await import(pathToFileURL(listPath,).href) as {
              readonly htmlBlockNames: readonly string[];
              readonly htmlRawNames: readonly string[];
            };
            expect(htmlBlockNames.length > 40,).toBe(true,);
            expect(
              htmlBlockNames.filter(function missing(name,): boolean {
                return !BOUNDARY_TAG_NAMES.has(name,);
              },),
            ).toEqual([],);
            expect(htmlRawNames.length > 0,).toBe(true,);
            expect(
              htmlRawNames.filter(function missing(name,): boolean {
                return !BOUNDARY_TAG_NAMES.has(name,);
              },),
            ).toEqual([],);
          },
        },),

        it({
          name: 'HOLDS the break, the rule, the image and the two ruby annotation tags beside the parser\'s names',
          fn: async () => {
            for (const name of ['br', 'hr', 'img', 'rt', 'rp',])
              expect(BOUNDARY_TAG_NAMES.has(name,),).toBe(true,);
          },
        },),

        it({
          name: 'LEAVES OUT the tags that only wrap text',
          fn: async () => {
            for (const name of ['b', 'i', 'em', 'strong', 'span', 'a', 'u', 's', 'sub', 'sup', 'ruby', 'code', 'mark', 'small',])
              expect(BOUNDARY_TAG_NAMES.has(name,),).toBe(false,);
          },
        },),
      ],
    },),

    describe({
      name: 'the kinds',
      children: [
        it({
          name: 'READS a break tag as a boundary with or without a slash, and the text around it as shown',
          fn: async () => {
            expect(pattern({ english: 'a<br/>b', },),).toBe('sbbbbbs',);
            expect(pattern({ english: 'a<br />b', },),).toBe('sbbbbbbs',);
          },
        },),

        it({
          name: 'READS the tags of a wrapping element as hidden and of a ruby annotation as boundaries',
          fn: async () => {
            expect(pattern({ english: 'a<b>c</b>d', },),).toBe('shhhshhhhs',);
            expect(pattern({ english: '<ruby>k<rt>j</rt></ruby>', },),).toBe('hhhhhhsbbbbsbbbbbhhhhhhh',);
          },
        },),

        it({
          name: 'READS the pipes and padding of a table row as boundaries and the cell text as shown',
          fn: async () => {
            expect(pattern({ english: '| a | b |\n| - | - |\n| c | d |', },),).toBe('bbsbbbsbbshhhhhhhhhsbbsbbbsbb',);
          },
        },),

        it({
          name: 'READS an image as boundaries and the content of inline code as shown beside hidden backticks',
          fn: async () => {
            expect(pattern({ english: 'a![p](u)b', },),).toBe('sbbbbbbbs',);
            expect(pattern({ english: 'a`kit`b', },),).toBe('shssshs',);
          },
        },),

        it({
          name: 'READS the tags of the four raw elements as boundaries, since the parser reads them to their closing tag (n166, Z7)',
          fn: async () => {
            expect(pattern({ english: 'a<pre>b</pre>c', },),).toBe(`s${'b'.repeat(5,)}s${'b'.repeat(6,)}s`,);
            expect(pattern({ english: 'a<textarea></textarea>b', },),).toBe(`s${'b'.repeat(10 + 11,)}s`,);
            expect(pattern({ english: 'a<script></script>b', },),).toBe(`s${'b'.repeat(8 + 9,)}s`,);
            expect(pattern({ english: 'a<style></style>b', },),).toBe(`s${'b'.repeat(7 + 8,)}s`,);
          },
        },),

        it({
          name: 'READS the one padding space of inline code as hidden beside the backticks, and the code word as shown (n166, Z6)',
          fn: async () => {
            expect(pattern({ english: 'a` kit `b', },),).toBe('shhssshhs',);
            expect(pattern({ english: 'a`` kit ``b', },),).toBe('shhhssshhhs',);
          },
        },),

        it({
          name: 'READS a space of inline code as shown unless it is the one padding space on each side of a content that is not all spaces '
            + '(n166, Z6)',
          fn: async () => {
            expect(pattern({ english: 'a` kit`b', },),).toBe('shsssshs',);
            expect(pattern({ english: 'a`kit `b', },),).toBe('shsssshs',);
            expect(pattern({ english: 'a`  `b', },),).toBe('shsshs',);
            expect(pattern({ english: 'a`  kit  `b', },),).toBe('shhssssshhs',);
          },
        },),

        it({
          name: 'READS an image with no alt text as hidden, and an image with an alt of spaces or a word as boundaries (n169)',
          fn: async () => {
            expect(pattern({ english: 'a![](u)b', },),).toBe('shhhhhhs',);
            expect(pattern({ english: 'a![](u "t")b', },),).toBe(`s${'h'.repeat(10,)}s`,);
            expect(pattern({ english: 'a![](u)![](v)b', },),).toBe(`s${'h'.repeat(12,)}s`,);
            expect(pattern({ english: 'a![ ](u)b', },),).toBe('sbbbbbbbs',);
            expect(pattern({ english: 'a![p](u)b', },),).toBe('sbbbbbbbs',);
          },
        },),

        it({
          name: 'READS a line break inside the resource of a link or an image as hidden with the rest of it, and a line break between '
            + 'words as shown (n169)',
          fn: async () => {
            expect(pattern({ english: 'a[](u\n)b', },),).toBe('shhhhhhs',);
            expect(pattern({ english: 'a[x](u\n)b', },),).toBe('shshhhhhs',);
            expect(pattern({ english: 'a[x](u\n "t\nt")b', },),).toBe(`shs${'h'.repeat(11,)}s`,);
            expect(pattern({ english: 'a![x](u\n)b', },),).toBe(`s${'b'.repeat(8,)}s`,);
            expect(pattern({ english: 'a\nb', },),).toBe('sss',);
            expect(pattern({ english: 'a\n\nb', },),).toBe('ssss',);
          },
        },),
      ],
    },),
  ],
},);
