import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  flattenContainers,
  parseMdxBody,
  UnpositionedContainerError,
} from '@monochromatic-dev/translation-repair-document-structure';
import { parseDocument, } from '../dist/final/node/index.mjs';
import {
  nodeOwnsCloser,
  nodeOwnsOpener,
} from '@monochromatic-dev/translation-repair-document-structure/ts/container-tag-ownership.test-fixture.ts';

//region Container unwrapping tests
// Fixtures are cat-themed inventions that reproduce the STRUCTURE read off the
// pinned corpus: a page that collapses a trailing gallery into one disclosure
// element against a counterpart carrying the same blocks at top level. Real
// corpus text is never reproduced here.

/**
 Counterpart shape: blocks at top level, no disclosure wrapper.
 */
const FLAT_PAGE = `## Gallery

<PhotoScroll photos={['a.webp']} />

> Mittens napping in the sun

<PhotoScroll photos={['b.webp']} />

> Mittens demanding tuna
`;

/**
 Same blocks, wrapped in a disclosure the way the en side wraps them.
 */
const NESTED_PAGE = `## Gallery

<details>
<summary>Original</summary>

<PhotoScroll photos={['a.webp']} />

> Mittens napping in the sun

<PhotoScroll photos={['b.webp']} />

> Mittens demanding tuna

</details>
`;

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: '',
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        describe({
          name: 'flattenContainers via parseDocument',
          children: [
            it({
              name: 'promotes disclosure children so a nested page matches its flat counterpart',
              fn: async () => {
                /**
                 Blocks the flat page exposes.
                 */
                const { nodes: flat, } = parseDocument({ text: FLAT_PAGE, },);

                /**
                 Blocks the nested page exposes once containers flatten.
                 */
                const { nodes: nested, } = parseDocument({ text: NESTED_PAGE, },);

                // The nested page carries one extra block: its summary, which the
                // flat counterpart has no equivalent for.
                expect(nested.length,).toBe(flat.length + 1,);
                expect(
                  nested.filter(function isPhoto(node,) {
                    return node.text
                      .includes('PhotoScroll',);
                  },),
                ).toHaveLength(2,);
              },
            },),

            it({
              name: 'keeps a self-closing component whole, since it is content not packaging',
              fn: async () => {
                /**
                 Page whose only block is a childless component.
                 */
                const { nodes, } = parseDocument({
                  text: "<PhotoScroll photos={['a.webp']} />\n",
                },);

                expect(nodes,).toHaveLength(1,);
                expect(nodes.at(0,)
                  ?.kind,).toBe('mdxJsxFlowElement',);
              },
            },),

            it({
              name: 'preserves document order when promoting',
              fn: async () => {
                /**
                 Promoted blocks must take the container's exact place.
                 */
                const { nodes, } = parseDocument({ text: NESTED_PAGE, },);

                expect(nodes.at(0,)
                  ?.kind,).toBe('heading',);
                expect(nodes.at(-1,)
                  ?.kind,).toBe('blockquote',);
              },
            },),

            it({
              name: 'keeps promoted offsets sliceable from the original text',
              fn: async () => {
                /**
                 Every node's text must still equal its own offset slice, the
                 invariant offset-based splicing depends on.
                 */
                const { nodes, } = parseDocument({ text: NESTED_PAGE, },);

                for (const node of nodes) {
                  expect(
                    NESTED_PAGE.slice(
                      node.startOffset,
                      node.endOffset,
                    ),
                  ).toBe(node.text,);
                }
              },
            },),

            it({
              name: 'hands the disclosure tags to the promoted blocks beside them, which is the '
                + 'REVERSAL of what this case once asserted',
              fn: async () => {
                /**
                 Wrapper tags used to survive only as inter-block text, and that
                 is precisely what let a range boundary fall between an opening
                 tag and its closing one: every range here is minted from block
                 offsets, so a tag belonging to no block belonged to no range
                 either. Each tag now rides inside the block beside it, so a
                 range built from whole blocks cannot separate the two.
                 */
                const { nodes, containers, } = parseDocument({ text: NESTED_PAGE, },);
                if (containers.length === 0)
                  throw new Error('fixture reported no container, so this case would pass for the wrong reason',);
                for (const container of containers) {
                  expect(nodes
                    .some(function ownsOpener(node,): boolean {
                      return nodeOwnsOpener({ node, container, },);
                    },),).toBe(true,);
                  expect(nodes
                    .some(function ownsCloser(node,): boolean {
                      return nodeOwnsCloser({ node, container, },);
                    },),).toBe(true,);
                }
              },
            },),

            it({
              name: 'flattens a disclosure nested inside another disclosure',
              fn: async () => {
                /**
                 The work stack re-inspects promoted children, so depth is not
                 limited to one level.
                 */
                const { nodes, } = parseDocument({
                  text: `<details>
<summary>Outer</summary>

<details>
<summary>Inner</summary>

> Mittens all the way down

</details>

</details>
`,
                },);

                expect(
                  nodes.some(function isQuote(node,) {
                    return node.kind === 'blockquote';
                  },),
                ).toBe(true,);
              },
            },),
          ],
        },),
      ],
    },),

    describe({
      name: flattenContainers.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'REFUSES A CONTAINER WITH NO POSITION OF ITS OWN, whose tags nothing could then locate (ledger T8)',
          fn: async () => {
            /**
             A parsed disclosure, dissolved once as parsed so the refusal cannot
             pass for the wrong reason.
             */
            const { children: parsed, } = parseMdxBody({
              body: '<details>\n\n> Mittens on the windowsill\n\n</details>\n',
            },);
            expect(flattenContainers({ children: parsed, },).containers,).toHaveLength(1,);

            /**
             The same tree with each top-level block stripped of its own position
             while its children keep theirs, the one shape a constructed tree can
             reach the walk in.
             */
            const unpositioned = parsed.map(function dropPosition(child,) {
              return {
                ...child,
                position: undefined,
              };
            },);
            expect(() => flattenContainers({ children: unpositioned, },),).toThrow(UnpositionedContainerError,);
          },
        },),
        it({
          name: 'NAMES A FRAGMENT <> IN THE REFUSAL, the name the MDX extension writes as null read as empty (ledger T8)',
          fn: async () => {
            /**
             A parsed fragment, dissolved once as parsed so the refusal cannot pass
             for the wrong reason.
             */
            const { children: parsed, } = parseMdxBody({
              body: '<>\n\n> Mittens on the windowsill\n\n</>\n',
            },);
            expect(flattenContainers({ children: parsed, },).containers,).toHaveLength(1,);

            /**
             The fragment stripped of its own position.
             */
            const unpositioned = parsed.map(function dropPosition(child,) {
              return {
                ...child,
                position: undefined,
              };
            },);
            expect(() => flattenContainers({ children: unpositioned, },),).toThrow('container <> carries block children',);
          },
        },),
        it({
          name: 'KEEPS AS A BLOCK A CONTAINER WHOSE CHILDREN ARE NOT A LIST, which no parse writes and the walk cannot read (ledger T8)',
          fn: async () => {
            /**
             A parsed disclosure, its children replaced by a value that is not a
             list, cast past the type as foreign input can arrive.
             */
            const malformed = parseMdxBody({
              body: '<details>\n\n> Mittens on the windowsill\n\n</details>\n',
            },)
              .children
              .map(function unlisted(child,) {
                return {
                  ...child,
                  children: 'naps',
                };
              },) as unknown as Parameters<typeof flattenContainers>[0]['children'];
            const flattened = flattenContainers({ children: malformed, },);
            expect(flattened.containers,).toHaveLength(0,);
            expect(flattened.blocks,).toEqual(malformed,);
          },
        },),
      ],
    },),
  ],
},);

//endregion Container unwrapping tests
