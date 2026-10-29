import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';
import type {
  Root,
  RootContent,
} from 'mdast';


import {
  type DeepReadonlyData,
  parseMdxBody,
  type ProtectedAtom,
  requireMdxRefusal,
  scanTextAtoms,
  withMdxExhaustionRefused,
} from '@monochromatic-dev/translation-repair-document-structure/ts';

/**
 Recursively readonly mdast root, as this module BORROWS the parse result.

 @example
 ```ts
 const root: ReadonlyMdastRoot = attempt.root;
 ```
 */
type ReadonlyMdastRoot = DeepReadonlyData<Root>;

/**
 Recursively readonly mdast content node.

 @example
 ```ts
 const node: ReadonlyMdastContent = root.children[0];
 ```
 */
type ReadonlyMdastContent = DeepReadonlyData<RootContent>;

//region Paragraph inspection
// Reads one paragraph, base or candidate, into the ordered atom sequence the
// naturalness gate compares. Shares `parseMdxBody` with document parsing rather
// than scanning Markdown by hand, so the gate cannot disagree with the parser
// about what a link or a footnote is.
//
// A candidate is REJECTED rather than repaired when it is not exactly one
// paragraph. A rewrite that arrives as two paragraphs, or as a list, or with a
// heading bolted on, has changed the document's structure, and the lane's whole
// premise is that it changes only wording.

/**
 Why a paragraph could not be inspected.

 @example
 ```ts
 const reason: InspectionRejection = 'not-one-paragraph';
 ```
 */
export type InspectionRejection =
  | 'unparseable'
    | 'not-one-paragraph'
    | 'carries-markup';

/**
 Result of reading one paragraph.

 @example
 ```ts
 const inspection: ParagraphInspection = { kind: 'inspected', atoms, };
 ```
 */
export type ParagraphInspection =
  | {
    readonly kind: 'inspected';

    /**
     Protected atoms in the order they appear.
     */
    readonly atoms: readonly ProtectedAtom[];
  }
    | {
      readonly kind: 'rejected';

    /**
     Why inspection refused it.
     */
      readonly reason: InspectionRejection;
    };

/**
 mdast node kinds that carry structure a rewrite must not introduce; their
 presence means the candidate stopped being plain prose.
 */
const MARKUP_KINDS: ReadonlySet<string> = new Set([
  'html',
  'mdxFlowExpression',
  'mdxTextExpression',
  'mdxJsxFlowElement',
  'mdxJsxTextElement',
  'break',
]);

/**
 Reads one inline node into the atoms it contributes, if any.

 @param node - inline mdast node

 @returns Atoms this node contributes, in order

 @example
 ```ts
 const atoms = atomsOfNode({ node, },);
 ```
 */
function atomsOfNode(
  { node, }: { readonly node: ReadonlyMdastContent; },
): readonly ProtectedAtom[] {
  if (node.type === 'text')
    return scanTextAtoms({ text: node.value, },);
  if (node.type === 'inlineCode')
    return [{
      kind: 'inline-code',
      value: node.value,
    },];
  if (node.type === 'link')
    return [{
      kind: 'link-url',
      value: node.url,
    },];
  if (node.type === 'image')
    return [{
      kind: 'image-url',
      value: node.url,
    },];
  if ((node.type === 'linkReference') || (node.type === 'imageReference'))
    return [{
      kind: 'reference',
      value: node.identifier,
    },];
  if (node.type === 'footnoteReference')
    return [{
      kind: 'footnote',
      value: node.identifier,
    },];
  return [];
}

/**
 Outcome of the strict parse, kept as a discriminated union so a refusal
 travels as a real value rather than as an absent root.

 @example
 ```ts
 const attempt: ParseAttempt = { kind: 'refused', };
 ```
 */
type ParseAttempt =
  | {
    readonly kind: 'parsed';

    /**
     Tree of the paragraph.
     */
    readonly root: ReadonlyMdastRoot;
  }
    | { readonly kind: 'refused'; };

/**
 Parses one paragraph under the strict grammar only.

 The plain-markdown fallback is deliberately NOT used here. Document parsing
 downgrades so a whole corpus page still yields anchors; a single rewritten
 paragraph that needs the fallback is a candidate to refuse, not a document
 to rescue.

 @param text - exact paragraph source

 @returns Tree, or a refusal when the strict grammar rejected it

 @example
 ```ts
 const attempt = parseStrictly({ text: paragraph, },);
 ```
 */
function parseStrictly({ text, }: { readonly text: string; },): ParseAttempt {
  try {
    return {
      kind: 'parsed',
      root: withMdxExhaustionRefused({
        parse: parseMdxBody,
        input: { body: text, },
      },),
    };
  }
  catch (error) {
    // Only the strict grammar's own rejection becomes a refusal; anything
    // else is an unexpected state that must keep propagating.
    requireMdxRefusal({ error, },);
    return { kind: 'refused', };
  }
}

/**
 Reads one paragraph's protected atoms in document order.

 The paragraph is parsed TWICE, for two different questions.

 Alone, to answer whether it is exactly one paragraph: that is a fact about
 the candidate's own structure and must not be influenced by anything
 appended to it.

 Then with the document's link and footnote definitions appended, to answer
 what it references. GFM only produces a `footnoteReference` or a
 `linkReference` when a matching definition is in scope, so an isolated
 paragraph reports `[^1]` as literal text. That is not a cosmetic difference:
 the digit inside would be protected as a number while the marker syntax
 around it would not, and a rewrite turning `[^1]` into `1` would pass a gate
 that should have stopped it.

 @param text - exact paragraph source, base or candidate

 @param definitions - link and footnote definitions from the whole `T1`
 document, so references resolve; omitted means the paragraph references
 nothing defined elsewhere

 @returns Ordered atoms, or the reason the paragraph was refused

 @throws Error when the text read alone is one paragraph and the parse with
 the definitions leads with another block, which the blank line between them
 rules out

 @example
 ```ts
 const inspection = inspectParagraph({ text: paragraph, definitions, },);
 ```
 */
export function inspectParagraph(
  {
    text,
    definitions = '',
  }: {
    readonly text: string;
    readonly definitions?: string;
  },
): ParagraphInspection {
  /**
   Structure of the paragraph on its own terms.
   */
  const alone = parseStrictly({ text, },);
  if (alone.kind === 'refused')
    return {
      kind: 'rejected',
      reason: 'unparseable',
    };
  if (
    (alone.root
      .children
      .length
      !== 1)
      || (alone.root
        .children[0]
        ?.type
        !== 'paragraph')
  ) {
    return {
      kind: 'rejected',
      reason: 'not-one-paragraph',
    };
  }

  /**
   The same paragraph with definitions in scope, so its references resolve.
   */
  const parsed = definitions === ''
    ? alone
    : parseStrictly({ text: `${text}\n\n${definitions}`, },);
  if (parsed.kind === 'refused')
    return {
      kind: 'rejected',
      reason: 'unparseable',
    };

  /**
   Leading block, which the structure check already proved is the paragraph:
   the definitions follow it after a blank line, and a blank line ends a
   paragraph, so nothing appended can change what its first block is.
   */
  const [block,] = parsed.root
    .children;
  if ((block === undefined) || (block.type !== 'paragraph'))
    throw new Error(
      `unreachable: a text that parsed alone as one paragraph led with ${
        block?.type ?? 'no block'
      } once the definitions were appended after a blank line, which ends a paragraph before them`,
    );

  /**
   Inline nodes still to visit, held as a stack so the walk stays iterative
   over a tree of unknown depth; children push reversed to keep document
   order on pop.
   */
  const pending: ReadonlyMdastContent[] = [...block.children,].toReversed();

  /**
   Atoms in document order.
   */
  const atoms: ProtectedAtom[] = [];
  // Next node in document order, until the stack is empty.
  for (let node = pending.pop(); node !== undefined; node = pending.pop()) {
    if (MARKUP_KINDS.has(node.type,))
      return {
        kind: 'rejected',
        reason: 'carries-markup',
      };
    atoms.push(...atomsOfNode({ node, },),);

    // A link's own children are visited too: its text can carry numbers and
    // foreign runs that must survive exactly like any other prose.
    if ('children' in node)
      pending.push(...[...node.children,].toReversed(),);
  }
  return {
    kind: 'inspected',
    atoms,
  };
}

/**
 Verdict of the structural gate over one rewrite.

 @example
 ```ts
 const verdict: AtomGateVerdict = { kind: 'preserved', };
 ```
 */
export type AtomGateVerdict =
  | { readonly kind: 'preserved'; }
    | {
      readonly kind: 'refused';

    /**
     Scorecard-stable account of the first divergence.
     */
      readonly detail: string;
    };

/**
 Renders one atom for a gate refusal.

 @param atom - atom to describe

 @returns Kind and value in one token

 @example
 ```ts
 const label = describeAtom({ kind: 'number', value: '17', },);
 ```
 */
function describeAtom(atom: ProtectedAtom,): string {
  return `${atom.kind}:${atom.value}`;
}

/**
 Checks that a rewrite carries every protected atom through unchanged and in
 the same order.

 Order is the point. Comparing multisets would pass a candidate that turned
 "3 cats and 5 dogs" into "5 cats and 3 dogs", or that swapped two links'
 destinations, or two names' positions: every atom still present, every claim
 different.

 @param base - `T1` paragraph the rewrite replaces

 @param candidate - proposed replacement

 @param definitions - link and footnote definitions from the whole `T1`
 document, so both sides resolve their references identically

 @returns Whether the rewrite may proceed, and what diverged when not

 @example
 ```ts
 const verdict = gateParagraphRewrite({ base, candidate, },);
 ```
 */
export function gateParagraphRewrite(
  {
    base,
    candidate,
    definitions = '',
  }: {
    readonly base: string;
    readonly candidate: string;
    readonly definitions?: string;
  },
): AtomGateVerdict {
  /**
   Atoms of the text being replaced.
   */
  const baseInspection = inspectParagraph({
    text: base,
    definitions,
  },);
  if (baseInspection.kind === 'rejected')
    return {
      kind: 'refused',
      detail: `base paragraph not inspectable (${baseInspection.reason})`,
    };

  /**
   Atoms of the proposed replacement.
   */
  const candidateInspection = inspectParagraph({
    text: candidate,
    definitions,
  },);
  if (candidateInspection.kind === 'rejected')
    return {
      kind: 'refused',
      detail: `candidate rejected (${candidateInspection.reason})`,
    };

  /**
   Atoms the base carries.
   */
  const expected = baseInspection.atoms;

  /**
   Atoms the candidate carries.
   */
  const actual = candidateInspection.atoms;
  if (expected.length !== actual.length)
    return {
      kind: 'refused',
      detail: `protected atom count changed (${String(expected.length,)} to ${
        String(actual.length,)
      })`,
    };

  /**
   First position where the two sequences disagree, absent when they match.
   */
  const divergence = expected.findIndex(function differs(
    atom,
    index,
  ) {
    /**
     Candidate's atom at the same position.
     */
    const other = actual[index];
    return (other === undefined)
      || (other.kind !== atom.kind)
      || (other.value !== atom.value);
  },);
  if (divergence === (-1))
    return { kind: 'preserved', };
  return {
    kind: 'refused',
    detail: `protected atom ${String(divergence + 1,)} changed (${
      describeAtom(nonNullishOrThrow(expected[divergence],),)
    } became ${describeAtom(nonNullishOrThrow(actual[divergence],),)})`,
  };
}

//endregion Paragraph inspection
