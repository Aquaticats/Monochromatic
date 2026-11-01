/**
 Tests for text marker scanners and error paths of the parsing core.
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
  buildDocumentNodes,
  buildFootnoteGraph,
  MdxParseError,
  parseMarkdownBody,
  parseMdxBody,
  scanFullwidthMarkers,
  scanGfmReferenceLiterals,
  UnpositionedNodeError,
} from '@monochromatic-dev/translation-repair-document-structure';
import { parseDocument, } from '../dist/final/node/index.mjs';

/**
 Page whose footnote definition is nested, exactly as the corpus nests them.
 */
const NESTED_DEFINITION = [
  '## 猫',
  '',
  '<details>',
  '<summary>More</summary>',
  '',
  'Whiskers naps here[^1]',
  '',
  '[^1]: On the warm windowsill.',
  '',
  '</details>',
  '',
].join('\n',);

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: scanFullwidthMarkers.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'finds ASCII-digit markers with slice-local offsets',
          fn: async () => {
            expect(scanFullwidthMarkers({ slice: '猫须考〔1〕与〔23〕', },),).toEqual([
              { identifier: '1', localOffset: 3, },
              { identifier: '23', localOffset: 7, },
            ],);
          },
        },),

        it({
          name: 'normalizes full-width digits to ASCII identifiers',
          fn: async () => {
            expect(scanFullwidthMarkers({ slice: '尾巴〔１２〕', },),).toEqual([
              { identifier: '12', localOffset: 2, },
            ],);
          },
        },),

        it({
          name: 'ignores empty, unterminated, and non-digit brackets',
          fn: async () => {
            expect(scanFullwidthMarkers({ slice: '空〔〕未闭〔12猫〔喵〕', },),).toEqual([],);
          },
        },),
      ],
    },),

    describe({
      name: scanGfmReferenceLiterals.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'finds literal references and rejects stopper characters',
          fn: async () => {
            expect(scanGfmReferenceLiterals({ slice: '引用[^7]与[^note]，普通[括号]和[^带 空格]不算', },),)
              .toEqual([
                { identifier: '7', localOffset: 2, },
                { identifier: 'note', localOffset: 7, },
              ],);
          },
        },),

        it({
          name: 'ignores empty and unterminated literals',
          fn: async () => {
            expect(scanGfmReferenceLiterals({ slice: '空[^]和未闭[^12', },),).toEqual([],);
          },
        },),
      ],
    },),

    describe({
      name: buildFootnoteGraph.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'collects both conventions and reports only real integrity gaps',
          fn: async () => {
            /**
             Body pairing a mid-text fullwidth reference with its
             block-opening definition, a resolved GFM pair, and one
             never-referenced GFM definition.
             */
            const body =
              '猫的脚注〔1〕与蝴蝶。[^2]\n\n〔1〕关于猫的注释。\n\n[^2]: 关于蝴蝶的注释。\n\n[^3]: 没有人引用的注释。\n';
            const graph = buildFootnoteGraph({
              children: parseMarkdownBody({ body, },).children,
              bodyText: body,
              bodyOffset: 0,
            },);
            expect(
              graph.references.map(function toKey(reference,) {
                return `${reference.convention}/${reference.identifier}`;
              },),
            ).toEqual(['fullwidth-bracket/1', 'gfm/2',],);
            expect(graph.references[0]?.offset,).toBe(body.indexOf('〔1〕',),);
            expect(
              graph.definitions.map(function toKey(definition,) {
                return `${definition.convention}/${definition.identifier}`;
              },),
            ).toEqual(['fullwidth-bracket/1', 'gfm/2', 'gfm/3',],);
            expect(graph.findings,).toHaveLength(1,);
            const [finding,] = graph.findings;
            expect(finding?.kind,).toBe('orphan-definition',);
            expect(finding?.convention,).toBe('gfm',);
            expect(finding?.identifier,).toBe('3',);
            expect(finding?.nodeId,).toContain('block/',);
          },
        },),
        it({
          name: 'reports unresolved references and duplicate definitions',
          fn: async () => {
            /**
             Body with an undefined GFM reference and one fullwidth
             identifier defined twice, referenced never.
             */
            const body = '未定义的引用[^9]。\n\n〔2〕第一次定义。\n\n〔2〕第二次定义。\n';
            const graph = buildFootnoteGraph({
              children: parseMarkdownBody({ body, },).children,
              bodyText: body,
              bodyOffset: 0,
            },);

            /**
             Finding kinds grouped for branch-by-branch assertion.
             */
            const kinds = graph.findings.map(function toKind(finding,) {
              return finding.kind;
            },);
            expect(kinds.filter(function isUnresolved(kind,) {
              return kind === 'unresolved-reference';
            },),).toHaveLength(1,);
            expect(kinds.filter(function isDuplicate(kind,) {
              return kind === 'duplicate-definition';
            },),).toHaveLength(2,);
          },
        },),
      ],
    },),

    describe({
      name: parseMarkdownBody.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'parses JSX-hostile text as plain markdown without throwing',
          fn: async () => {
            /**
             Body that MDX parsing rejects but markdown accepts.
             */
            const root = parseMarkdownBody({ body: '<MaoBox 未闭合的组件\n\n喵。\n', },);
            expect(root.type,).toBe('root',);
            expect(root.children.length,).toBeGreaterThanOrEqual(1,);
          },
        },),
      ],
    },),

    describe({
      name: parseMdxBody.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'throws MdxParseError on invalid JSX',
          fn: async () => {
            /** Value caught from parse of unclosed JSX element. */
            let caught: unknown;
            try {
              parseMdxBody({ body: '<MaoBox 未闭合的组件\n\n喵。\n', },);
            }
            catch (error) {
              caught = error;
            }
            expect(caught instanceof MdxParseError,).toBe(true,);
          },
        },),
      ],
    },),

    describe({
      name: buildDocumentNodes.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'throws UnpositionedNodeError on constructed trees without positions',
          fn: async () => {
            /** Value caught from node construction over an unpositioned tree. */
            let caught: unknown;
            try {
              buildDocumentNodes({
                children: [{ type: 'paragraph', children: [], },],
                bodyText: '喵。',
                bodyOffset: 0,
              },);
            }
            catch (error) {
              caught = error;
            }
            expect(caught instanceof UnpositionedNodeError,).toBe(true,);
          },
        },),
      ],
    },),

    describe({
      name: 'parseDocument footnote graph beside autolink literals',
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'PARSES a page holding an autolink literal micromark did not tokenize, alone, beside a GFM '
            + 'footnote and beside a full-width one, reading the footnotes as the page holds them',
          fn: async () => {
            expect(parseDocument({ text: 'A cat ，www.cat.example naps.\n', },).footnoteGraph,).toEqual({
              references: [],
              definitions: [],
              findings: [],
            },);
            expect(parseDocument({ text: 'A cat[^1] ，www.cat.example naps.\n\n[^1]: The cat.\n', },)
              .footnoteGraph,).toEqual({
                references: [{ convention: 'gfm', identifier: '1', nodeId: 'block/0', offset: 5, },],
                definitions: [{ convention: 'gfm', identifier: '1', nodeId: 'block/1', },],
                findings: [],
              },);
            expect(parseDocument({ text: 'A cat〔1〕 ，www.cat.example naps.\n\n〔1〕The cat.\n', },)
              .footnoteGraph,).toEqual({
                references: [{ convention: 'fullwidth-bracket', identifier: '1', nodeId: 'block/0', offset: 5, },],
                definitions: [{ convention: 'fullwidth-bracket', identifier: '1', nodeId: 'block/1', },],
                findings: [],
              },);
          },
        },),
        it({
          name: 'READS an undefined GFM reference set off from an untokenized literal and no marker shape '
            + 'the parse puts in a link: one glued to the literal, or one whose label is a literal',
          fn: async () => {
            /**
             One run holding a reference in text, then a shape whose opening
             the transform takes into the link URL, then a label the
             transform links between its brackets.
             */
            const text = 'A cat ，www.cat.example [^9] naps ，www.cat.example[^8] and [^www.cat.example].\n';
            expect(parseDocument({ text, },).footnoteGraph,).toEqual({
              references: [{ convention: 'gfm', identifier: '9', nodeId: 'block/0', offset: text.indexOf('[^9]',), },],
              definitions: [],
              findings: [{ kind: 'unresolved-reference', convention: 'gfm', identifier: '9', nodeId: 'block/0', },],
            },);
          },
        },),
        it({
          name: 'READS a full-width marker inside the text of an autolink literal as a reference, tokenized or '
            + 'not, and one opening its block beside an untokenized literal as a definition',
          fn: async () => {
            /**
             Both literals take the glued `〔1〕` into their link, micromark's
             and the transform's alike; the second paragraph opens on its
             definition, the literal after it untokenized.
             */
            const texts = [
              'A cat ，www.cat.example〔1〕 naps.\n\n〔1〕，www.cat.example 的注释。\n',
              'A cat www.cat.example〔1〕 naps.\n\n〔1〕，www.cat.example 的注释。\n',
            ];
            for (const text of texts) {
              expect(parseDocument({ text, },).footnoteGraph,).toEqual({
                references: [
                  { convention: 'fullwidth-bracket', identifier: '1', nodeId: 'block/0', offset: text.indexOf('〔1〕',), },
                ],
                definitions: [{ convention: 'fullwidth-bracket', identifier: '1', nodeId: 'block/1', },],
                findings: [],
              },);
            }
          },
        },),
        it({
          name: 'READS NO marker shape in a JSX attribute where the element text is an untokenized literal, a '
            + 'GFM one or a full-width one',
          fn: async () => {
            const text = 'A cat <Cat title="[^9]" note="〔7〕">，www.cat.example [^8]</Cat> naps.\n';
            expect(parseDocument({ text, },).footnoteGraph,).toEqual({
              references: [{ convention: 'gfm', identifier: '8', nodeId: 'block/0', offset: text.indexOf('[^8]',), },],
              definitions: [],
              findings: [{ kind: 'unresolved-reference', convention: 'gfm', identifier: '8', nodeId: 'block/0', },],
            },);
          },
        },),
        it({
          name: 'READS NO GFM shape inside the URL of an autolink literal micromark tokenized, and still reads '
            + 'a full-width marker there',
          fn: async () => {
            const text = 'A cat https://cat.example/[^9]x naps www.cat.example/〔1〕x.\n';
            expect(parseDocument({ text, },).footnoteGraph,).toEqual({
              references: [
                { convention: 'fullwidth-bracket', identifier: '1', nodeId: 'block/0', offset: text.indexOf('〔1〕',), },
              ],
              definitions: [],
              findings: [
                { kind: 'unresolved-reference', convention: 'fullwidth-bracket', identifier: '1', nodeId: 'block/0', },
              ],
            },);
          },
        },),
        it({
          name: 'READS NO GFM shape inside the URL of an angle autolink, which the strict grammar refuses and the '
            + 'plain one tokenizes, and still reads a full-width marker there and a reference beside it',
          fn: async () => {
            const text = 'A cat <https://cat.example/[^9]x> naps <https://cat.example/〔1〕x> and [^8].\n';
            expect(parseDocument({ text, },).footnoteGraph,).toEqual({
              references: [
                { convention: 'fullwidth-bracket', identifier: '1', nodeId: 'block/0', offset: text.indexOf('〔1〕',), },
                { convention: 'gfm', identifier: '8', nodeId: 'block/0', offset: text.indexOf('[^8]',), },
              ],
              definitions: [],
              findings: [
                { kind: 'unresolved-reference', convention: 'fullwidth-bracket', identifier: '1', nodeId: 'block/0', },
                { kind: 'unresolved-reference', convention: 'gfm', identifier: '8', nodeId: 'block/0', },
              ],
            },);
          },
        },),
        it({
          name: 'READS a GFM reference in the label of an inline link and in text beside an angle autolink, where '
            + 'the link\'s first child starts one character after its own start as an angle autolink\'s does',
          fn: async () => {
            const text = 'A cat [paws [^7] nap](https://cat.example/u) naps <https://cat.example/v> [^8].\n';
            expect(parseDocument({ text, },).footnoteGraph,).toEqual({
              references: [
                { convention: 'gfm', identifier: '7', nodeId: 'block/0', offset: text.indexOf('[^7]',), },
                { convention: 'gfm', identifier: '8', nodeId: 'block/0', offset: text.indexOf('[^8]',), },
              ],
              definitions: [],
              findings: [
                { kind: 'unresolved-reference', convention: 'gfm', identifier: '7', nodeId: 'block/0', },
                { kind: 'unresolved-reference', convention: 'gfm', identifier: '8', nodeId: 'block/0', },
              ],
            },);
          },
        },),
        it({
          name: 'PARSES a page the strict grammar refuses and the plain one reads with an untokenized literal, '
            + 'reading its undefined reference',
          fn: async () => {
            /**
             An unclosed expression makes the MDX grammar refuse the page, so
             the plain grammar reads it, whose autolink transform also
             rebuilds the literal without positions.
             */
            const text = 'A cat {paw ，www.cat.example [^9] naps.\n';
            expect(parseDocument({ text, },).footnoteGraph,).toEqual({
              references: [{ convention: 'gfm', identifier: '9', nodeId: 'block/0', offset: text.indexOf('[^9]',), },],
              definitions: [],
              findings: [{ kind: 'unresolved-reference', convention: 'gfm', identifier: '9', nodeId: 'block/0', },],
            },);
          },
        },),
      ],
    },),

    describe({
      name: 'parseDocument footnote graph over containers',
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'resolves a reference whose DEFINITION sits inside a disclosure '
            + 'container. The node list flattens containers while the graph walked '
            + 'the RAW tree, so every definition inside one was invisible to it, '
            + 'and one corpus translation reported all ten of its references '
            + 'unresolved while carrying all ten definitions',
          fn: async () => {
            /**
             Graph built over the nested page.
             */
            const { footnoteGraph, } = parseDocument({ text: NESTED_DEFINITION, },);

            expect(footnoteGraph.references.length,).toBe(1,);
            expect(footnoteGraph.definitions.length,).toBe(1,);
            expect(footnoteGraph.findings,).toEqual([],);
          },
        },),

        it({
          name: 'still reports a genuinely dangling reference, so this widened what '
            + 'the graph can SEE without blunting what it reports. One corpus '
            + 'translation really did lose every one of its definitions',
          fn: async () => {
            /**
             Page referencing a definition that is nowhere at all.
             */
            const { footnoteGraph, } = parseDocument({
              text: '## 猫\n\nWhiskers naps here[^1]\n',
            },);

            expect(footnoteGraph.definitions.length,).toBe(0,);
            expect(footnoteGraph.findings.length,).toBe(1,);
            expect(footnoteGraph.findings[0]?.kind,).toBe('unresolved-reference',);
          },
        },),

        it({
          name: 'emits nodeIds naming blocks of the SAME list the document exposes. '
            + 'The graph counted containers the node list had already unwrapped, so '
            + 'every id after one named a different block than it meant',
          fn: async () => {
            /**
             Page with a container BEFORE the footnote, which is what shifted ids.
             */
            const { footnoteGraph, nodes, } = parseDocument({
              text: [
                '## 猫',
                '',
                '<details>',
                '<summary>More</summary>',
                '',
                'A nap.',
                '',
                '</details>',
                '',
                'Whiskers naps here[^1]',
                '',
                '[^1]: On the warm windowsill.',
                '',
              ].join('\n',),
            },);

            /**
             Block ids the document actually exposes.
             */
            const ids = new Set(nodes.map(function toId(
              _node,
              index,
            ) {
              return `block/${String(index,)}`;
            },),);
            for (
              const entry of [
                ...footnoteGraph.references,
                ...footnoteGraph.definitions,
              ]
            )
              expect(ids.has(entry.nodeId,),).toBe(true,);
          },
        },),
      ],
    },),
  ],
},);
