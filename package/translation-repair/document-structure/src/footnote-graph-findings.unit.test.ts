/**
 Tests for the integrity findings a footnote graph reports, read through the
 graph the package builds, since the findings are computed from its
 references and definitions alone. Fixtures are cat-themed invention.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  buildFootnoteGraph,
  parseMarkdownBody,
} from '../dist/final/node/index.mjs';

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: 'footnote graph findings',
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'REPORTS unresolved references in reference order, then orphan definitions, then every copy of a '
            + 'duplicate definition',
          fn: async () => {
            /**
             A paragraph referring to two notes no block defines, a GFM
             definition nobody refers to, and a full-width one written twice.
             */
            const body = '猫[^9]与〔1〕。\n\n[^2]: 没有人引用。\n\n〔3〕第一次。\n\n〔3〕第二次。\n';
            expect(buildFootnoteGraph({
              children: parseMarkdownBody({ body, },).children,
              bodyText: body,
              bodyOffset: 0,
            },).findings,).toEqual([
              { kind: 'unresolved-reference', convention: 'fullwidth-bracket', identifier: '1', nodeId: 'block/0', },
              { kind: 'unresolved-reference', convention: 'gfm', identifier: '9', nodeId: 'block/0', },
              { kind: 'orphan-definition', convention: 'gfm', identifier: '2', nodeId: 'block/1', },
              { kind: 'orphan-definition', convention: 'fullwidth-bracket', identifier: '3', nodeId: 'block/2', },
              { kind: 'orphan-definition', convention: 'fullwidth-bracket', identifier: '3', nodeId: 'block/3', },
              { kind: 'duplicate-definition', convention: 'fullwidth-bracket', identifier: '3', nodeId: 'block/2', },
              { kind: 'duplicate-definition', convention: 'fullwidth-bracket', identifier: '3', nodeId: 'block/3', },
            ],);
          },
        },),
        it({
          name: 'KEYS a footnote by convention and identifier together, so a GFM reference leaves a full-width '
            + 'definition of the same number unresolved and orphaned',
          fn: async () => {
            const body = '猫[^1]。\n\n〔1〕注释。\n';
            expect(buildFootnoteGraph({
              children: parseMarkdownBody({ body, },).children,
              bodyText: body,
              bodyOffset: 0,
            },).findings,).toEqual([
              { kind: 'unresolved-reference', convention: 'gfm', identifier: '1', nodeId: 'block/0', },
              { kind: 'orphan-definition', convention: 'fullwidth-bracket', identifier: '1', nodeId: 'block/1', },
            ],);
          },
        },),
        it({
          name: 'REPORTS NOTHING where every reference resolves and every definition is referred to once',
          fn: async () => {
            const body = '猫[^1]与〔2〕。\n\n[^1]: 一。\n\n〔2〕二。\n';
            expect(buildFootnoteGraph({
              children: parseMarkdownBody({ body, },).children,
              bodyText: body,
              bodyOffset: 0,
            },).findings,).toEqual([],);
          },
        },),
      ],
    },),
  ],
},);
