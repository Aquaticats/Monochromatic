/**
 Tests for document parsing over both corpus shapes:
 memorial-shaped MDX with GFM footnotes and JSX,
 and archive-shaped text with full-width bracket markers.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { hashContent, } from '@monochromatic-dev/translation-repair-document-structure';
import {
  isCommentFinding,
  parseDocument,
} from '../dist/final/node/index.mjs';

/**
 Memorial-shaped fixture mirroring corpus structure with cat-themed invention:
 front matter, section heading, paragraph with GFM footnote reference,
 inline-code marker look-alike, self-closing JSX element,
 blockquote, and link-wrapped footnote definition mirroring corpus style.
 */
const MEMORIAL_FIXTURE = `---
name: whiskers
---

## 简介

这只猫很喜欢晒太阳[^1]，行内代码 \`[^2]\` 不算引用。

<MailTo template="info" />

> 「猫说：喵。」

[^1]:[关于猫晒太阳习性的背景说明。](https://example.org/archived)
`;

/**
 Archive-shaped fixture with cat-themed invention:
 heading with marker reference, body reference, indented definitions,
 one resolving pair, one unresolved reference, one orphan, one duplicated pair.
 */
const ARCHIVE_FIXTURE = `## 猫须考〔1〕

\u3000\u3000猫的胡须很长〔3〕，猫的尾巴更长〔5〕。

\u3000\u3000〔3〕\u3000「胡须很长」\u3000语见《猫经》第一回。

\u3000\u3000〔5〕\u3000「尾巴更长」\u3000语见《尾巴谱》。

\u3000\u3000〔5〕\u3000重复的定义条目。

\u3000\u3000〔9〕\u3000从未被引用的孤儿定义。
`;

/**
 Comment-bearing fixture with cat-themed invention:
 front matter, a standalone comment hiding a footnote-marker look-alike,
 and a real resolving footnote pair.
 */
const COMMENTED_FIXTURE = `---
name: whiskers
---

## 简介

猫猫喜欢晒太阳[^1]。

<!-- 编辑备注：[^9] 这个被注释掉的引用不能进入脚注图 -->

猫猫也喜欢追蝴蝶。

[^1]:[关于猫晒太阳的注释。](https://example.org/a)
`;

/**
 Fixture whose body carries an unclosed MDX brace expression,
 failing the strict grammar without any HTML comment involved.
 */
const BROKEN_EXPRESSION_FIXTURE = `## 简介

猫猫的表达式 {'没有关闭

猫猫继续晒太阳。
`;

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: 'parseDocument (memorial shape)',
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'classifies node kinds and zones from MDX structure',
          fn: async () => {
            /** Parsed memorial-shaped fixture. */
            const doc = parseDocument({ text: MEMORIAL_FIXTURE, },);
            expect(doc.nodes.map(function toKind(node,): string {
              return node.kind;
            },),).toEqual(
              ['heading', 'paragraph', 'mdxJsxFlowElement', 'blockquote', 'footnoteDefinition',],
            );
            expect(doc.nodes.map(function toZone(node,): string {
              return node.zone;
            },),).toEqual(
              ['body', 'body', 'body', 'body', 'footnote-definition',],
            );
          },
        },),

        it({
          name: 'anchors every node so text equals offset-sliced source',
          fn: async () => {
            /** Parsed memorial-shaped fixture. */
            const doc = parseDocument({ text: MEMORIAL_FIXTURE, },);
            for (const node of doc.nodes) {
              expect(node.text,).toBe(doc.text.slice(node.startOffset, node.endOffset,),);
              expect(node.contentHash,).toBe(hashContent({ content: node.text, },),);
            }
          },
        },),

        it({
          name: 'parses deterministically: same source, same hashes and ids',
          fn: async () => {
            /** First parse of fixture. */
            const first = parseDocument({ text: MEMORIAL_FIXTURE, },);
            /** Second parse of identical fixture. */
            const second = parseDocument({ text: MEMORIAL_FIXTURE, },);
            expect(first.documentHash,).toBe(second.documentHash,);
            expect(first.nodes,).toStrictEqual(second.nodes,);
          },
        },),

        it({
          name: 'resolves GFM footnote graph and ignores marker look-alikes in inline code',
          fn: async () => {
            /** Parsed memorial-shaped fixture. */
            const doc = parseDocument({ text: MEMORIAL_FIXTURE, },);
            expect(doc.footnoteGraph.references,).toHaveLength(1,);
            expect(doc.footnoteGraph.references[0]?.identifier,).toBe('1',);
            expect(doc.footnoteGraph.references[0]?.convention,).toBe('gfm',);
            expect(doc.footnoteGraph.definitions,).toHaveLength(1,);
            expect(doc.footnoteGraph.findings,).toHaveLength(0,);
          },
        },),

        it({
          name: 'reports unresolved GFM references as findings',
          fn: async () => {
            /** Parsed source referencing an undefined footnote. */
            const doc = parseDocument({ text: '正文引用[^7]没有定义。\n', },);
            expect(doc.footnoteGraph.findings,).toEqual([
              {
                kind: 'unresolved-reference',
                convention: 'gfm',
                identifier: '7',
                nodeId: 'block/0',
              },
            ],);
          },
        },),
      ],
    },),

    describe({
      name: 'parseDocument (archive shape)',
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'classifies block-opening markers as definitions and mid-text markers as references',
          fn: async () => {
            /** Parsed archive-shaped fixture. */
            const doc = parseDocument({ text: ARCHIVE_FIXTURE, },);
            expect(doc.footnoteGraph.references.map(function toId(reference,): string {
              return reference.identifier;
            },),).toEqual(['1', '3', '5',],);
            expect(doc.footnoteGraph.definitions.map(function toId(definition,): string {
              return definition.identifier;
            },),).toEqual(['3', '5', '5', '9',],);
          },
        },),

        it({
          name: 'reports unresolved, orphan, and duplicate findings across the graph',
          fn: async () => {
            /** Parsed archive-shaped fixture. */
            const doc = parseDocument({ text: ARCHIVE_FIXTURE, },);
            /** Finding kinds keyed by identifier for order-free assertions. */
            const kinds = doc.footnoteGraph.findings.map(function toPair(finding,): string {
              return `${finding.kind} ${finding.identifier}`;
            },);
            expect(kinds,).toContain('unresolved-reference 1',);
            expect(kinds,).toContain('orphan-definition 9',);
            expect(kinds,).toContain('duplicate-definition 5',);
          },
        },),

        it({
          name: 'anchors reference offsets onto full source text',
          fn: async () => {
            /** Parsed archive-shaped fixture. */
            const doc = parseDocument({ text: ARCHIVE_FIXTURE, },);
            for (const reference of doc.footnoteGraph.references) {
              expect(
                doc.text.slice(reference.offset, reference.offset + 1,),
              ).toBe('〔',);
            }
          },
        },),
      ],
    },),

    describe({
      name: 'parseDocument (tolerant parsing)',
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'keeps parseFindings empty for strictly parsing documents',
          fn: async () => {
            expect(parseDocument({ text: MEMORIAL_FIXTURE, },).parseFindings,).toEqual([],);
          },
        },),

        it({
          name: 'parses comment-bearing documents, reporting each skipped comment',
          fn: async () => {
            /** Parsed comment-bearing fixture. */
            const doc = parseDocument({ text: COMMENTED_FIXTURE, },);

            expect(doc.parseFindings,).toEqual([{
              kind: 'html-comment-skipped',
              startOffset: COMMENTED_FIXTURE.indexOf('<!--',),
              endOffset: COMMENTED_FIXTURE.indexOf('-->',) + '-->'.length,
              detail: 'HTML comment masked to whitespace before parsing',
            },],);
            // Prose on both sides of the comment still parses into nodes.
            expect(doc.nodes.map(function toKind(node,): string {
              return node.kind;
            },),).toEqual(
              ['heading', 'paragraph', 'paragraph', 'footnoteDefinition',],
            );
            // Nodes still anchor byte-for-byte onto the ORIGINAL text.
            for (const node of doc.nodes) {
              expect(node.text,).toBe(doc.text.slice(node.startOffset, node.endOffset,),);
              expect(node.text,).not.toContain('编辑备注',);
            }
          },
        },),

        it({
          name: 'keeps commented-out marker look-alikes out of the footnote graph',
          fn: async () => {
            /** Parsed comment-bearing fixture. */
            const doc = parseDocument({ text: COMMENTED_FIXTURE, },);
            // The real reference resolves; the commented-out [^9] never appears.
            expect(doc.footnoteGraph.findings,).toEqual([],);
            expect(
              doc
                .footnoteGraph
                .references
                .map(function toIdentifier(reference,): string {
                  return reference.identifier;
                },),
            ).toEqual(['1',],);
          },
        },),

        it({
          name: 'reports an unterminated comment as its own finding kind',
          fn: async () => {
            /** Body whose comment swallows the tail of the document. */
            const doc = parseDocument({
              text: '## 简介\n\n猫猫晒太阳。\n\n<!-- 没有结束的备注\n尾巴',
            },);
            expect(doc.parseFindings.map(function toKind(finding,): string {
              return finding.kind;
            },),).toEqual(['unterminated-html-comment',],);
          },
        },),

        it({
          name: 'downgrades strict-grammar failures to markdown with a finding',
          fn: async () => {
            /** Parsed broken-expression fixture. */
            const doc = parseDocument({ text: BROKEN_EXPRESSION_FIXTURE, },);

            expect(doc.parseFindings,).toHaveLength(1,);
            expect(doc.parseFindings[0]?.kind,).toBe('mdx-downgraded',);
            expect(doc.parseFindings[0]?.detail,).toContain('fell back to plain markdown',);
            // The fallback grammar still yields anchored nodes over the whole body.
            expect(doc.nodes.length,).toBeGreaterThan(1,);
            for (const node of doc.nodes) {
              expect(node.text,).toBe(doc.text.slice(node.startOffset, node.endOffset,),);
            }
          },
        },),
        it({
          name: 'A COMMENT FINDING is a masked comment or one left open, and no other tolerance event',
          fn: async () => {
            expect(([
              'html-comment-skipped',
              'unterminated-html-comment',
              'invisible-line-masked',
              'mdx-downgraded',
            ] as const).map(function reads(kind,): boolean {
              return isCommentFinding({
                finding: {
                  kind,
                  startOffset: 0,
                  endOffset: 1,
                  detail: 'a cat walked across the keyboard',
                },
              },);
            },),).toEqual([true, true, false, false,],);
          },
        },),
      ],
    },),
    describe({
      name: 'A leading byte order mark',
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'ANCHORS every block at the characters it is made of, the mark left outside the first one',
          fn: async () => {
            /**
             Document opening with the mark, then two paragraphs.
             */
            const doc = parseDocument({ text: '\uFEFFThe cat naps.\n\nSecond cat.\n', },);

            expect(doc.nodes.map(function offsetsAndText(node,) {
              return [node.startOffset, node.endOffset, node.text,];
            },),).toEqual([
              [1, 14, 'The cat naps.',],
              [16, 27, 'Second cat.',],
            ],);
          },
        },),
        it({
          name: 'ANCHORS every block after front matter at the characters it is made of, a mark opening the body '
            + 'left outside the first one',
          fn: async () => {
            /**
             Document whose body, not the document, opens with the mark.
             */
            const doc = parseDocument({ text: '---\nname: cat\n---\n\uFEFFThe cat naps.\n', },);

            expect(doc.nodes.map(function offsetsAndText(node,) {
              return [node.startOffset, node.endOffset, node.text,];
            },),).toEqual([[19, 32, 'The cat naps.',],],);
          },
        },),
        it({
          name: 'PLACES both tags of a container and the block between them at the characters they are made of',
          fn: async () => {
            /**
             Document opening with the mark, then a container around one paragraph.
             */
            const doc = parseDocument({ text: '\uFEFF<Box>\n\nA cat.\n\n</Box>\n', },);

            expect(doc.containers,).toEqual([{
              name: 'Box',
              openerStartOffset: 1,
              openerEndOffset: 8,
              closerStartOffset: 14,
              closerEndOffset: 22,
            },],);
            expect(doc.nodes.map(function offsetsAndText(node,) {
              return [node.startOffset, node.endOffset, node.text,];
            },),).toEqual([[1, 22, '<Box>\n\nA cat.\n\n</Box>',],],);
          },
        },),
        it({
          name: 'PLACES a footnote reference at the character it is written at',
          fn: async () => {
            /**
             Document opening with the mark, then a paragraph with a reference and its definition.
             */
            const doc = parseDocument({ text: '\uFEFFThe cat[^1] naps.\n\n[^1]: note.\n', },);

            expect(doc.footnoteGraph,).toEqual({
              references: [{ convention: 'gfm', identifier: '1', nodeId: 'block/0', offset: 8, },],
              definitions: [{ convention: 'gfm', identifier: '1', nodeId: 'block/1', },],
              findings: [],
            },);
          },
        },),
      ],
    },),
  ],
},);
