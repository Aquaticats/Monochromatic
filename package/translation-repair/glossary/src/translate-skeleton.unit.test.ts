/**
 Tests for reading one slice into the shape a translation of it has to match.

 WHAT THIS MODULE IS FOR, and why its branches are worth asking about
 directly. Structure and references survive a translation and wording does
 not: 三只猫 becomes "three cats" and no digit is left on either side, so a
 gate anchored to prose would fire on every correct rendering. What a
 candidate must still carry is the sequence of blocks and the machine-readable
 identities inside them, and that is exactly what this reads.

 THE DETAIL FIELD IS THE PART A KIND CHECK MISSES. Two headings are both
 `heading` and a level-two turned into a level-three has changed the document;
 two lists are both `list` and a bulleted turned into a numbered has too. Four
 cases pin what distinguishes them, and one pins that everything else says
 nothing rather than inventing a difference.

 ATOMS ARE FILTERED, NOT COLLECTED. `atomsOfNode` reads seven node types and
 the walk keeps five kinds, so a case per kind is a case per arm.

 ONE ARM IS NOT REACHABLE FROM HERE: the rethrow of an error that is not an
 `MdxParseError`. Every input this function accepts is a string, and the
 grammar answers a string with either a tree or its own refusal, so reaching
 the rethrow needs a seam this module does not have. It is left uncovered
 deliberately rather than by oversight.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  blockDetail,
  readSliceSkeleton,
} from '../dist/final/node/index.mjs';

/**
 Reads a slice and returns its blocks, refusing anything unparseable.

 @param text - slice source

 @returns Block shapes in document order

 @throws {@link Error} when the slice did not parse, so a case that meant to
 read blocks cannot silently assert about a refusal instead

 @example
 ```ts
 expect(blocksOf({ text: '## Cats', },),).toEqual([{ kind: 'heading', detail: 'level 2', },],);
 ```
 */
function blocksOf(
  { text, }: { readonly text: string; },
): readonly { readonly kind: string; readonly detail: string; }[] {
  /**
   Outcome of reading this slice.
   */
  const read = readSliceSkeleton({ text, },);
  if (read.kind !== 'read')
    throw new Error(`expected a readable slice, got ${read.kind}`,);
  return read
    .skeleton
    .blocks;
}

/**
 Reads a slice and returns its atoms, refusing anything unparseable.

 @param text - slice source

 @returns Protected atoms in document order

 @throws {@link Error} when the slice did not parse

 @example
 ```ts
 expect(atomsOf({ text: 'Run `catnip` now.', },),).toEqual([{ kind: 'inline-code', value: 'catnip', },],);
 ```
 */
function atomsOf(
  { text, }: { readonly text: string; },
): readonly { readonly kind: string; readonly value: string; }[] {
  /**
   Outcome of reading this slice.
   */
  const read = readSliceSkeleton({ text, },);
  if (read.kind !== 'read')
    throw new Error(`expected a readable slice, got ${read.kind}`,);
  return read
    .skeleton
    .atoms;
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: blockDetail.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'names a heading by its level, which is the difference a kind '
            + 'check misses: two headings are both `heading` and a level-two '
            + 'turned into a level-three has changed the document',
          fn: async () => {
            expect(blockDetail({
              node: {
                type: 'heading',
                depth: 3,
                children: [],
              },
            },),)
              .toBe('level 3',);
          },
        },),

        it({
          name: 'names an ordered list ordered',
          fn: async () => {
            expect(blockDetail({
              node: {
                type: 'list',
                ordered: true,
                children: [],
              },
            },),)
              .toBe('ordered',);
          },
        },),

        it({
          name: 'names a list bulleted where the parser said it is not ordered',
          fn: async () => {
            expect(blockDetail({
              node: {
                type: 'list',
                ordered: false,
                children: [],
              },
            },),)
              .toBe('bulleted',);
          },
        },),

        it({
          name: 'names a list bulleted where the parser said NOTHING about '
            + 'ordering, since mdast leaves the field off and reading an absent '
            + 'field as ordered would report a change no author made',
          fn: async () => {
            expect(blockDetail({
              node: {
                type: 'list',
                children: [],
              },
            },),)
              .toBe('bulleted',);
          },
        },),

        it({
          name: 'says nothing about a block whose kind already says everything, '
            + 'rather than inventing a difference two of them could be compared on',
          fn: async () => {
            expect(blockDetail({
              node: {
                type: 'paragraph',
                children: [],
              },
            },),)
              .toBe('',);
          },
        },),
      ],
    },),

    describe({
      name: readSliceSkeleton.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'reads a plain paragraph as one block carrying no atoms, which is '
            + 'the control every other case departs from',
          fn: async () => {
            expect(blocksOf({ text: 'The cat naps on the sill.', },),)
              .toEqual([
                {
                  kind: 'paragraph',
                  detail: '',
                },
              ],);
            expect(atomsOf({ text: 'The cat naps on the sill.', },),).toEqual([],);
          },
        },),

        it({
          name: 'READS a slice that carries an HTML comment under the grammar the document was read in, '
            + 'masking the comment to whitespace so it is no block and refuses nothing, since 17 of 92 '
            + 'sources carry one and on 2026-09-06 every such slice of yulianNyanner was an original '
            + 'that could not be read',
          fn: async () => {
            expect(blocksOf({ text: '## Cats\n\n<!-- (this section is a translator note) -->\n\nThe cat naps.', },),)
              .toEqual([
                {
                  kind: 'heading',
                  detail: 'level 2',
                },
                {
                  kind: 'paragraph',
                  detail: '',
                },
              ],);
            // A comment carrying markup of its own is still a comment.
            expect(blocksOf({ text: 'The cat naps.\n\n<!-- <p style="text-align: center;"></p> -->', },),)
              .toEqual([
                {
                  kind: 'paragraph',
                  detail: '',
                },
              ],);
            // Atoms after a comment keep their offsets, since the mask is the same length as the comment.
            expect(atomsOf({ text: '<!-- note -->\n\nRun `catnip` now.', },),)
              .toEqual([
                {
                  kind: 'inline-code',
                  value: 'catnip',
                },
              ],);
          },
        },),

        it({
          name: 'carries the heading level and the list ordering through to the '
            + 'block shapes, so the detail rule is reachable from real Markdown '
            + 'and not only from a hand-built node',
          fn: async () => {
            expect(blocksOf({ text: '## Cats\n\n1.  one\n2.  two', },),)
              .toEqual([
                {
                  kind: 'heading',
                  detail: 'level 2',
                },
                {
                  kind: 'list',
                  detail: 'ordered',
                },
              ],);
          },
        },),

        it({
          name: 'reads a bulleted list as bulleted, which is the other arm of the '
            + 'same rule and the one a translation turning bullets into numbers '
            + 'would break',
          fn: async () => {
            expect(blocksOf({ text: '-   one\n-   two', },),)
              .toEqual([
                {
                  kind: 'list',
                  detail: 'bulleted',
                },
              ],);
          },
        },),

        it({
          name: 'KEEPS inline code as an atom, since an author fenced it '
            + 'precisely so it would not be rewritten',
          fn: async () => {
            expect(atomsOf({ text: 'Run `catnip --now` today.', },),)
              .toEqual([
                {
                  kind: 'inline-code',
                  value: 'catnip --now',
                },
              ],);
          },
        },),

        it({
          name: 'KEEPS a link target and an image target as separate kinds, since '
            + 'both are machine-readable identities and a reader comparing them '
            + 'has to know which one moved',
          fn: async () => {
            expect(atomsOf({
              text: 'See [the sill](https://example.invalid/sill) and '
                + '![a cat](https://example.invalid/cat.png).',
            },),)
              .toEqual([
                {
                  kind: 'link-url',
                  value: 'https://example.invalid/sill',
                },
                {
                  kind: 'image-url',
                  value: 'https://example.invalid/cat.png',
                },
              ],);
          },
        },),

        it({
          name: 'KEEPS a link reference by its label rather than by the target it '
            + 'resolves to, which is what survives when the definition moves',
          fn: async () => {
            expect(atomsOf({
              text: 'See [the sill][sill] now.\n\n[sill]: https://example.invalid/sill',
            },),)
              .toEqual([
                {
                  kind: 'reference',
                  value: 'sill',
                },
              ],);
          },
        },),

        it({
          name: 'reads an image reference as the same `reference` kind as a link '
            + 'reference, since a label is a label and the node type it came from '
            + 'is not what a translation can change',
          fn: async () => {
            expect(atomsOf({
              text: 'Look ![a cat][cat] here.\n\n[cat]: https://example.invalid/cat.png',
            },),)
              .toEqual([
                {
                  kind: 'reference',
                  value: 'cat',
                },
              ],);
          },
        },),

        it({
          name: 'KEEPS BOTH ENDS of a footnote, the marker and the definition, '
            + 'because they are separate nodes and a translation that dropped '
            + 'either would leave the other pointing at nothing',
          fn: async () => {
            expect(atomsOf({
              text: 'The cat naps.[^one]\n\n[^one]: On the sill.',
            },),)
              .toEqual([
                {
                  kind: 'footnote',
                  value: 'one',
                },
                {
                  kind: 'footnote',
                  value: 'one',
                },
              ],);
          },
        },),

        it({
          name: 'finds atoms nested inside blocks and reports them in DOCUMENT '
            + 'ORDER across the whole slice, which is the property the walk '
            + 'reverses its stack pushes for',
          fn: async () => {
            expect(atomsOf({
              text: '# Cats\n\nSee [one](https://example.invalid/one).\n\n'
                + '-   And `two`.\n\n> Then ![three](https://example.invalid/three.png).',
            },),)
              .toEqual([
                {
                  kind: 'link-url',
                  value: 'https://example.invalid/one',
                },
                {
                  kind: 'inline-code',
                  value: 'two',
                },
                {
                  kind: 'image-url',
                  value: 'https://example.invalid/three.png',
                },
              ],);
          },
        },),

        it({
          name: 'DROPS emphasis and the prose inside it, since wording is what a '
            + 'translation is allowed to change and the judges are the instrument '
            + 'for it',
          fn: async () => {
            expect(atomsOf({ text: 'The *cat* naps on the **sill**.', },),)
              .toEqual([],);
          },
        },),

        it({
          name: 'reads empty text as a slice carrying nothing rather than '
            + 'refusing it, since a slice with no blocks is a legal thing for a '
            + 'preparation to hand over',
          fn: async () => {
            expect(blocksOf({ text: '', },),).toEqual([],);
            expect(atomsOf({ text: '', },),).toEqual([],);
          },
        },),

        it({
          name: 'REPORTS a slice the grammar refused as unparseable, carrying the '
            + 'parser`s own account so a finding names something a model can act '
            + 'on rather than saying only that something went wrong',
          fn: async () => {
            /**
             Outcome of reading a slice holding an unclosed component.
             */
            const read = readSliceSkeleton({
              text: '<MaoBox 未闭合的组件\n\n喵。\n',
            },);

            expect(read.kind,).toBe('unparseable',);
            expect((read as { readonly detail: string; }).detail,)
              .toContain('MdxParseError',);
          },
        },),
        it({
          name: 'READS a slice that owns a container opener without its closer, carrying the tag as an '
            + 'atom, since the first block inside a container owns its opening tag (Huasheng, 2026-09-07)',
          fn: async () => {
            /**
             Head of a disclosure block as a slice carries it.
             */
            const head = '<details>\n<summary>**A cat**</summary>\n\nThe cat naps.';

            // A `<summary>` holding phrasing on one line is a paragraph of inline
            // JSX to this grammar, inside a container or out of one.
            expect(blocksOf({ text: head, },),).toEqual([
              {
                kind: 'paragraph',
                detail: '',
              },
              {
                kind: 'paragraph',
                detail: '',
              },
            ],);
            expect(atomsOf({ text: head, },),).toEqual([
              {
                kind: 'container-tag',
                value: '<details>',
              },
            ],);
          },
        },),

        it({
          name: 'READS a slice that owns the closer alone, and puts it after the content atoms',
          fn: async () => {
            expect(atomsOf({ text: 'The cat wakes[^1].\n\n</details>\n\n[^1]: At dawn.', },),).toEqual([
              {
                kind: 'footnote',
                value: '1',
              },
              {
                kind: 'footnote',
                value: '1',
              },
              {
                kind: 'container-tag',
                value: '</details>',
              },
            ],);
          },
        },),

        it({
          name: 'LEAVES a container whose both tags sit in one slice to the grammar, which reads it as '
            + 'one element and carries no tag atom',
          fn: async () => {
            expect(atomsOf({ text: '<details>\n\nThe cat.\n\n</details>', },),).toEqual([],);
          },
        },),

        it({
          name: 'READS AN ELEMENT WHOSE OPENER SHARES ITS LINE WITH CONTENT and whose closer stands alone as one '
            + 'element (ledger X10, NIGHT81473140 slice 22): the lone-tag masker paired tag lines only, so it '
            + 'blanked the closer as unpartnered and left the opener with no end, and the slice read as '
            + 'unparseable while the document read it whole',
          fn: async () => {
            // THE SHAPE THE ENTRY HAS: the opener and a self-closing component on
            // one line, which the grammar reads as flow, and the closer alone.
            expect(atomsOf({
              text: '<blockquote><CatQuote lines={["The cat naps.", "She wakes at four."]} />\n</blockquote>',
            },),).toEqual([],);
            // The closer with no opener anywhere is still masked and carried.
            expect(atomsOf({ text: 'The cat naps.\n\n</blockquote>', },),).toEqual([
              {
                kind: 'container-tag',
                value: '</blockquote>',
              },
            ],);
          },
        },),
        it({
          name: 'READS the inline half of a container through a quoted or braced value holding the character '
            + 'that would end it, so a self-closing tag is refused on its own `>` and its closing half stays '
            + 'lone',
          fn: async () => {
            // THE QUOTE SWALLOWS the `>` that would end the tag and the closing quote releases it.
            expect(atomsOf({
              text: '猫猫在窗台上睡觉。\n<details title=">"/>猫猫追蝴蝶。\n</details>',
            },),).toEqual([
              {
                kind: 'container-tag',
                value: '</details>',
              },
            ],);
            expect(atomsOf({
              text: "猫猫在窗台上睡觉。\n<details title='>'/>猫猫追蝴蝶。\n</details>",
            },),).toEqual([
              {
                kind: 'container-tag',
                value: '</details>',
              },
            ],);
            // THE BRACE HOLDS the `>` that would end the tag.
            expect(atomsOf({
              text: '猫猫在窗台上睡觉。\n<details data-x={1 > 0}/>猫猫追蝴蝶。\n</details>',
            },),).toEqual([
              {
                kind: 'container-tag',
                value: '</details>',
              },
            ],);
          },
        },),

        it({
          name: 'READS A LINE HOLDING ONLY A BYTE ORDER MARK AS BLANK between two paragraphs, as the document '
            + 'parse reads it, so the slice has the two blocks the page has and not one welded block',
          fn: async () => {
            expect(blocksOf({ text: 'The cat naps.\n\uFEFF\nThe cat eats fish.', },),).toEqual([
              {
                kind: 'paragraph',
                detail: '',
              },
              {
                kind: 'paragraph',
                detail: '',
              },
            ],);
            // A zero-width space alone on a line is read the same way, and a mark inside a line of text is content.
            expect(blocksOf({ text: 'The cat naps.\n\u200B\nThe cat eats fish.', },),).toHaveLength(2,);
            expect(blocksOf({ text: 'The cat naps.\n\uFEFFThe cat eats fish.', },),).toHaveLength(1,);
          },
        },),
      ],
    },),
  ],
},);
