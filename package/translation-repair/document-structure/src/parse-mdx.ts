import type { Root, } from 'mdast';
import remarkGfm from 'remark-gfm';
import remarkMath from 'remark-math';
import remarkMdx from 'remark-mdx';
import remarkParse from 'remark-parse';
import { unified, } from 'unified';

import {
  isJsonArray,
  isJsonRecord,
} from './json-guard.ts';
import {
  firstNestingExcess,
  isStackOverflow,
  nestingAccount,
  NestingBoundError,
  type NestingExcess,
  nestingRefusalAccount,
  STACK_OVERFLOW_ACCOUNT,
  STACK_OVERFLOW_MESSAGE,
} from '@monochromatic-dev/translation-repair-markup/ts';
import { noteHeadingFloors, } from './heading-floors.ts';
import { callParseEndHook, } from './parse-end-hook.ts';
import { NAMED_POSITION_UNSTATED, } from './refusal-text.ts';

//region MDX parsing
// Corpus pages compile as MDX upstream (@mdx-js/mdx with a Vue pragma), so this module
// parses with the same grammar family. GFM is enabled here even though upstream omits
// it: footnote reference/definition nodes carry the semantic graph this library
// validates, while emitted repairs preserve the literal textual convention.
//
// LEDGER X22 (2026-09-29): the site also compiles with remark-math
// (one-among-us/data scripts/mdx.ts), so text between dollar signs is a
// formula there, and the strict parse reads it the same way. Over the 92
// sources and archives at the pin and 273 settled pages, adding it changed no
// tree. The tolerant fallback stays without it: with no MDX grammar, the
// template strings inside JSX attributes (`'${...}'`) read as prose there, and
// math would pair their dollar signs in 171 of those 457 inputs.

/**
 Describes where an MDX refusal stopped, quoting nothing it read.

 MOSTLY SAFE ALREADY, and that is why this is narrow rather than absent. Four
 of five measured failure shapes report a position and an expectation. The
 fifth, an unclosed tag, embeds the tag NAME from the source, which is enough
 to carry a page's own markup into a stored finding.

 @param cause - caught value, of unknown type by construction

 @returns Phrase naming position and rule

 @example
 ```ts
 `refused to parse ${mdxRefusalSite({ cause, },)}`;
 ```
 */
function mdxRefusalSite({ cause, }: { readonly cause: unknown; },): string {
  if (cause instanceof NestingBoundError)
    return `because it is ${nestingAccount({ excess: cause.excess, },)}`;
  if (isStackOverflow(cause,))
    return `because it is ${STACK_OVERFLOW_ACCOUNT}`;
  if (!Error.isError(cause,))
    return `at ${NAMED_POSITION_UNSTATED}`;

  /**
   Package that raised it, prefixed so two rules sharing a name stay apart.
   */
  const from = (('source' in cause) && ((typeof cause.source) === 'string'))
    ? `${cause.source}/`
    : '';

  /**
   Rule the grammar names, falling back to the class that raised it.
   */
  const rule = (('ruleId' in cause) && ((typeof cause.ruleId) === 'string'))
    ? `${from}${cause.ruleId}`
    : cause.name;

  // `VFileMessage` sets `name` to "line:column". Any other Error's name is its
  // class, and neither spelling repeats the source.
  return `at ${cause.name} (${rule})`;
}

/**
 Where a parser message says the grammar stopped, each part present only
 when the message names it.
 */
type RefusalPlace = {
  /**
   One-based line.
   */
  readonly line?: number;

  /**
   One-based column.
   */
  readonly column?: number;
};

/**
 What a parser message says the grammar stopped at: the end of the span it
 names, or the point it names, or the message itself when it names neither.

 THE END OF A SPAN, NOT ITS START (ledger B86). `mdast-util-mdx-jsx` raises
 its refusals on leaving the span it names (`onErrorRightIsTag`,
 `exitMdxJsxTag` in 3.2.0), so an element left open in a paragraph stops the
 grammar where the paragraph ends. A `VFileMessage` copies its own `line` and
 `column` from the span's start, which read that refusal at the paragraph's
 first character. Micromark's own refusals name a point.

 @param cause - caught error, of unknown shape beyond being an object

 @returns Object whose `line` and `column`, when numeric, name the stop

 @example
 ```ts
 const stop = stopPointOf({ cause, },);
 ```
 */
function stopPointOf({ cause, }: { readonly cause: object; },): object {
  if ((!('place' in cause))
    || ((typeof cause.place) !== 'object')
    || (cause.place === null))
    return cause;
  /**
   Point or span the message names.
   */
  const { place, } = cause;
  if (('end' in place)
    && ((typeof place.end) === 'object')
    && (place.end !== null))
    return place.end;
  return place;
}

/**
 Reads the line and column a parser message says the grammar stopped at,
 when it names them.

 A micromark message carries the place as numbers; anything else names no
 place. An element still open when the document ends is refused with no
 place at all, since the parser stopped at the end of the document.

 @param cause - caught value, of unknown type by construction

 @returns Line and column, each absent when unstated

 @example
 ```ts
 const { line, column, } = refusalPlace({ cause, },);
 ```
 */
function refusalPlace(
  { cause, }: { readonly cause: unknown; },
): RefusalPlace {
  if (!Error.isError(cause,))
    return {};
  /**
   Where the message says the grammar stopped.
   */
  const stop = stopPointOf({ cause, },);
  /**
   Line it names, when numeric.
   */
  const line: RefusalPlace = (('line' in stop) && ((typeof stop.line) === 'number'))
    ? { line: stop.line, }
    : {};
  /**
   Column it names, when numeric.
   */
  const column: RefusalPlace = (('column' in stop) && ((typeof stop.column) === 'number'))
    ? { column: stop.column, }
    : {};
  return {
    ...line,
    ...column,
  };
}

/**
 Signals MDX source that refuses to parse.

 Corpus documents compile upstream, so a refusal indicates corruption or a
 construct outside the mirrored grammar.

 @example
 ```ts
 throw new MdxParseError({ cause: error, droppedColumns: 0, },);
 ```
 */
export class MdxParseError extends Error {
  /**
   Declares this message safe to forward: it states where the grammar stopped
   and which rule it broke, and repeats no document text.
   */
  readonly messageNamesOnly: true = true;

  /**
   One-based line the grammar stopped on, absent when the parser named none.
   */
  readonly line?: number;

  /**
   One-based column the grammar stopped at, absent when the parser named none.
   */
  readonly column?: number;

  /**
   Builds failure stating where the grammar stopped, never what it read.

   DOES NOT CARRY THE PARSER ERROR AS `cause`, for the reason
   `FrontMatterParseError` records: a cause chain is rendered by Node's
   uncaught-exception reporter, and `parse-document.ts` used to stringify this
   one straight into a stored finding. THE ONE EXCEPTION is the engine's own
   stack exhaustion, whose message is fixed text that quotes nothing.

   @param cause - underlying micromark/remark error, read for position and rule

   @param droppedColumns - characters the parser dropped from the start of the
   body without counting them, added to the column of a first-line stop so it
   indexes the body as written

   @example
   ```ts
   new MdxParseError({ cause: error, droppedColumns: 0, },);
   ```
   */
  public constructor(
    {
      cause,
      droppedColumns,
    }: {
      readonly cause: unknown;
      readonly droppedColumns: number;
    },
  ) {
    super(
      `MDX body refused to parse ${mdxRefusalSite({ cause, },)}; corpus documents`
        + ' compile as MDX upstream, so failure signals corruption or an'
        + ' unsupported construct.',
      // The stack exhaustion alone rides along as the cause: its message is
      // fixed engine text, so the chain a reporter renders quotes nothing.
      isStackOverflow(cause,) ? { cause, } : undefined,
    );
    this.name = 'MdxParseError';
    /**
     Where the grammar stopped, as the parser's message carries it.
     */
    const place = refusalPlace({ cause, },);
    if (place.line !== undefined)
      this.line = place.line;
    if (place.column !== undefined)
      this.column = (place.line === 1)
        ? place.column + droppedColumns
        : place.column;
  }
}

//region Leading byte order mark
// THE PARSER DROPS A LEADING BYTE ORDER MARK WITHOUT COUNTING IT. `micromark`
// skips a U+FEFF opening its input and starts its offsets and first-line
// columns from the character after (`micromark@4.0.3` `lib/preprocess.js`, the
// `start` branch), so every position the parser reports for such a body sits
// one character short of the body as written. A paragraph preceded by the mark
// read as text beginning with the mark and ending one character before its
// end. Every reader of a position inherits that: block nodes, container spans,
// the footnote graph, and the offset a refusal names. The shift is undone
// here, once, where the positions are made, so no reader carries a mark-sized
// allowance of its own.
//
// The `estree` an expression node carries in its `data` keeps the parser's own
// coordinates: nothing in this package reads it.

/**
 The byte order mark the parser drops from the start of its input.
 */
const BYTE_ORDER_MARK = '\uFEFF';

/**
 How many characters the parser drops from the start of a body without
 counting them.

 @param body - body about to be parsed

 @returns One when the body opens with a byte order mark, otherwise zero

 @example
 ```ts
 const dropped = droppedWidth({ body: '\uFEFFThe cat naps.', },);
 ```
 */
function droppedWidth({ body, }: { readonly body: string; },): number {
  return body.startsWith(BYTE_ORDER_MARK,)
    ? BYTE_ORDER_MARK.length
    : 0;
}

/**
 A point of a parsed position, which the parser makes fresh for every node.
 */
type ParsedPoint = {
  /**
   One-based line.
   */
  line: number;

  /**
   One-based column.
   */
  column: number;

  /**
   Zero-based offset.
   */
  offset: number;
};

/**
 Whether a parsed value is a point.

 @param value - parsed value of unknown shape

 @returns True for an object carrying a numeric line, column and offset

 @example
 ```ts
 const isPoint = isParsedPoint(node.position?.start,);
 ```
 */
function isParsedPoint(value: unknown,): value is ParsedPoint {
  return isJsonRecord(value,)
    && ((typeof value.line) === 'number')
    && ((typeof value.column) === 'number')
    && ((typeof value.offset) === 'number');
}

/**
 Moves one parsed point on by the characters the parser dropped.

 @param point - start or end of a position, moved in place; a first-line
 point moves its column too

 @param width - characters the parser dropped from the start of the body

 @throws {@link Error} when the point lacks the numeric line, column and
 offset the parser sets on every point it makes

 @example
 ```ts
 movePastDropped({ point: node.position.start, width: 1, },);
 ```
 */
function movePastDropped(
  {
    point,
    width,
  }: {
    readonly point: unknown;
    readonly width: number;
  },
): void {
  if (!isParsedPoint(point,))
    throw new Error(
      'unreachable: a parsed point lacks a numeric line, column or offset, though the parser sets all three on every '
        + 'point it makes',
    );
  point.offset += width;
  if (point.line === 1)
    point.column += width;
}

/**
 Moves both ends of a parsed position on by the characters the parser
 dropped.

 @param position - `position` of a node, moved in place

 @param width - characters the parser dropped from the start of the body

 @throws {@link Error} when it is no position, which the parser sets on every
 node it builds

 @example
 ```ts
 movePositionPastDropped({ position: node.position, width: 1, },);
 ```
 */
function movePositionPastDropped(
  {
    position,
    width,
  }: {
    readonly position: unknown;
    readonly width: number;
  },
): void {
  if (!isJsonRecord(position,))
    throw new Error(
      'unreachable: a parsed node carries a position that is no object, though the parser sets one on every '
        + 'position it makes',
    );
  movePastDropped({
    point: position.start,
    width,
  },);
  movePastDropped({
    point: position.end,
    width,
  },);
}

/**
 What a tree walk follows from one node: its children and its attributes,
 never its `data`, which holds the expression syntax trees that carry their
 own coordinates.

 NOT AN ATTRIBUTE'S VALUE. The one value that is an object, an expression,
 is built with its type, its text and its `data` alone (`mdast-util-mdx-jsx`
 3.2.0, `exitMdxJsxTagAttributeValueExpression`), so it holds no position to
 move and nothing else to visit.

 @param node - parsed node or attribute

 @returns Members of the node still to visit

 @example
 ```ts
 const next = membersOf({ node: { children: [], }, },);
 ```
 */
function membersOf({ node, }: { readonly node: Readonly<Record<string, unknown>>; },): readonly unknown[] {
  /**
   Members found so far.
   */
  const members: unknown[] = [];
  /**
   Lists of nodes this node holds.
   */
  const lists = [
    node.children,
    node.attributes,
  ];
  for (const list of lists) {
    if (isJsonArray(list,))
      for (const member of list)
        members.push(member,);
  }
  return members;
}

/**
 Counts the characters the parser dropped from the start of a body into
 every position of the tree it built.

 The root keeps its start, since it spans the body as written, dropped
 characters included; every other node, and every attribute, starts after
 them or later.

 @param root - tree the parser has just built, moved in place; nothing else
 holds it yet

 @param body - body the tree was parsed from

 @throws {@link Error} when the root carries no position, which the parser
 sets on every node it builds, when the walk reaches a member that is no
 object, and when a position is no object or a point of it lacks a numeric
 line, column or offset; a node the autolink-literal transform built carries
 no position and is passed over, since it holds none to move

 @example
 ```ts
 countDropped({ root: unified().use(remarkParse,).parse(body,), body, },);
 ```
 */
function countDropped(
  {
    root,
    body,
  }: {
    readonly root: Root;
    readonly body: string;
  },
): void {
  /**
   Characters the parser dropped.
   */
  const width = droppedWidth({ body, },);
  if (width === 0)
    return;
  if (root.position === undefined)
    throw new Error('unreachable: the parsed root carries no position, though the parser sets one on every node',);
  movePastDropped({
    point: root.position
      .end,
    width,
  },);
  /**
   Nodes still to visit, so a deep tree is walked without recursion.
   */
  const pending: unknown[] = [...root.children,];
  for (let node = pending.pop(); node !== undefined; node = pending.pop()) {
    if (!isJsonRecord(node,))
      throw new Error(
        'unreachable: the walk reached a member that is no object, though it visits only the nodes of a parsed tree '
          + 'and their attributes, and the parser builds each as an object',
      );
    /**
     The node's position, absent on a node the autolink-literal transform built.
     */
    const { position, } = node;
    if (position !== undefined)
      movePositionPastDropped({
        position,
        width,
      },);
    for (const member of membersOf({ node, },))
      pending.push(member,);
  }
}

//endregion Leading byte order mark

/**
 Parses MDX body text into an mdast tree with positions on every node.

 A BODY NESTED PAST THE BOUND IS REFUSED BEFORE THE PARSER READS IT
 (`nesting-bound.ts`), so the outcome of a deeply nested reply does not depend
 on how much stack the caller has left; a stack exhaustion that gets past the
 bound is the same refusal, with the exhaustion as its cause.

 @param body - MDX source with front matter already split away

 @returns mdast root whose node positions are body-relative character offsets in the body as written, a leading
 byte order mark counted

 @throws {@link MdxParseError} when source refuses to parse as MDX, when it
 nests past the bound, and for any other failure inside the grammar too, a
 stack overflow on deep nesting among them

 @example
 ```ts
 const root = parseMdxBody({ body: '# Title\n\nParagraph with[^1]\n\n[^1]: note\n', },);
 ```
 */
export function parseMdxBody({ body, }: { readonly body: string; },): Root {
  /**
   Where the body passes the nesting bound, when it does.
   */
  const nesting = firstNestingExcess({
    body,
    grammar: 'mdx',
  },);
  if (nesting.kind === 'beyond')
    throw new MdxParseError({
      cause: new NestingBoundError({ excess: nesting, },),
      droppedColumns: droppedWidth({ body, },),
    },);
  /**
   Tree the grammar built, positions still short of a leading byte order mark.
   */
  const root = (function parseStrict(): Root {
    try {
      return unified()
        .use(remarkParse,)
        .use(remarkMdx,)
        .use(remarkGfm,)
        .use(remarkMath,)
        .parse(body,);
    }
    catch (error) {
      throw new MdxParseError({
        cause: error,
        droppedColumns: droppedWidth({ body, },),
      },);
    }
  })();
  countDropped({
    root,
    body,
  },);
  noteHeadingFloors({ root, },);
  callParseEndHook({ root, },);
  return root;
}

/**
 The strict grammar's refusal a catch around {@link parseMdxBody} holds,
 for a catch that acts on the refusal alone.

 SHARED RATHER THAN REPEATED (ledger T8, sixth batch). Seven catches around
 the grammar each tested the class and rethrew anything else, and no test
 reached a rethrow: `parseMdxBody` raises every failure as an
 {@link MdxParseError}, and the rest of their bodies throws only where an
 invariant breaks (a parsed node without a position). The narrowing stands
 here once, where a case reaches the rethrow.

 THE ENGINE'S OWN STACK EXHAUSTION IS NOT READ HERE. A catch cannot tell an exhaustion
 raised by the parse from one raised by other code of its block, and only the first is a
 fact about the text. The call that makes the parse reads its exhaustion as the refusal
 ({@link withMdxExhaustionRefused}), so a refusal reaches this catch as the refusal and an
 exhaustion of the block's other code reaches it raw and propagates.

 @param error - what the catch caught

 @returns Refusal the grammar raised, or the one {@link withMdxExhaustionRefused} built for an exhaustion of the parse

 @throws The caught value unchanged when it is anything but the grammar's refusal,
 the engine's stack exhaustion among it, an unexpected state that must keep propagating

 @example
 ```ts
 const refusal = requireMdxRefusal({ error, },);
 ```
 */
export function requireMdxRefusal({ error, }: { readonly error: unknown; },): MdxParseError {
  if (error instanceof MdxParseError)
    return error;
  throw error;
}

/**
 Makes one strict-grammar parse call, reading the engine's stack exhaustion it raises as the grammar's refusal.

 THE EXHAUSTION IS READ WHERE THE PARSE CALL ENDS and not in the catch of the block around it: a parse that
 runs out of stack where the refusal for it could not be built (the nesting check, or the handler that builds the
 refusal, standing within a few frames of the end of the stack) raises the engine's `RangeError` itself, and it is a
 fact about the text just as the refusal built in time is. The refusal is built here, after the stack has unwound to this
 call, so a boundary reads the page as text and goes on. An exhaustion raised by any other code of the boundary's block
 is not raised by this call, reaches {@link requireMdxRefusal} raw and propagates.

 @param parse - parse call, such as `parseMdxBody`, whose exhaustion is a fact about the text it was given

 @param input - what the parse is given

 @returns What the parse returns

 @throws {@link MdxParseError} built here, with the exhaustion as its cause, when the parse raises the engine's stack
 exhaustion; any other failure of the parse unchanged

 @example
 ```ts
 const root = withMdxExhaustionRefused({ parse: parseMdxBody, input: { body: 'A cat naps.', }, },);
 ```
 */
export function withMdxExhaustionRefused<Input, Output>(
  {
    parse,
    input,
  }: {
    readonly parse: (input: Input,) => Output;
    readonly input: Input;
  },
): Output {
  try {
    return parse(input,);
  }
  catch (error) {
    if (!isStackOverflow(error,))
      throw error;
    throw new MdxParseError({
      cause: error,
      droppedColumns: 0,
    },);
  }
}

/**
 Signals plain markdown text that nests too deeply to read.

 A `RangeError` by class, as the stack exhaustion it replaces was, so a reader
 of that class keeps working; it is this class and no other `RangeError` that
 {@link requireMarkdownRefusal} reads as the plain grammar's refusal.

 @example
 ```ts
 throw new MarkdownParseError({ reading: excess, },);
 ```
 */
export class MarkdownParseError extends RangeError {
  /**
   Declares this message safe to forward: it states which measure passed the
   bound and where, and repeats no document text.
   */
  readonly messageNamesOnly: true = true;

  /**
   Builds the refusal from where the body passed the bound, or from the stack
   exhaustion that was caught, quoting nothing of the body either way.

   THE PLACE IS IN THE MESSAGE AND NOWHERE ELSE. No reader of this refusal
   asks for a line or a column apart from its words, where a reader of the
   strict grammar's does (`strict-refusal-offset.ts`), so this class carries
   neither as a field.

   @param reading - where the body passed the bound, or `'stack-exhausted'`

   @param cause - stack exhaustion that was caught, whose message is fixed
   engine text, absent for a body refused by the bound

   @example
   ```ts
   new MarkdownParseError({ reading: excess, },);
   ```
   */
  public constructor(
    {
      reading,
      cause,
    }: {
      readonly reading: NestingExcess | 'stack-exhausted';
      readonly cause?: RangeError;
    },
  ) {
    super(
      `Plain markdown body refused to parse because it is ${nestingRefusalAccount({ reading, },)}.`,
      (cause === undefined) ? undefined : { cause, },
    );
    this.name = 'MarkdownParseError';
  }
}

/**
 Parses body text as plain markdown (GFM, no MDX extensions).
 Tolerant fallback grammar: markdown has no syntax error,
 so constructs the MDX grammar rejects (raw HTML, brace expressions)
 survive as literal `html` and text nodes instead of failing the document.

 NOT TOTAL: the parser descends once per nesting level, so thousands of nested
 quotation markers exhaust its stack (`translate-skeleton-page.ts` reports such
 a page as unread). A body nested past the bound (`nesting-bound.ts`) is
 refused before the parser reads it, so the outcome does not depend on how deep
 the caller stands; an exhaustion that gets past the bound is refused the same
 way. A catch acting on that refusal alone narrows with
 {@link requireMarkdownRefusal}.

 @param body - markdown source with front matter already split away

 @returns mdast root whose node positions are body-relative character offsets in the body as written, a leading
 byte order mark counted

 @throws {@link MarkdownParseError} when the body nests past the bound or
 nesting exhausts the parser's stack

 @example
 ```ts
 const root = parseMarkdownBody({ body: '<!-- note -->\n\nParagraph.\n', },);
 ```
 */
export function parseMarkdownBody({ body, }: { readonly body: string; },): Root {
  /**
   Where the body passes the nesting bound, when it does.
   */
  const nesting = firstNestingExcess({
    body,
    grammar: 'markdown',
  },);
  if (nesting.kind === 'beyond')
    throw new MarkdownParseError({ reading: nesting, },);
  /**
   Tree the grammar built, positions still short of a leading byte order mark.
   */
  const root = (function parseLoosely(): Root {
    try {
      return unified()
        .use(remarkParse,)
        .use(remarkGfm,)
        .parse(body,);
    }
    catch (error) {
      // Only the engine's stack exhaustion is this grammar's refusal; any
      // other failure is an unexpected state that must keep propagating.
      if (!isStackOverflow(error,))
        throw error;
      throw new MarkdownParseError({
        reading: 'stack-exhausted',
        cause: error,
      },);
    }
  })();
  countDropped({
    root,
    body,
  },);
  noteHeadingFloors({ root, },);
  callParseEndHook({ root, },);
  return root;
}

/**
 The plain grammar's refusal a catch around {@link parseMarkdownBody} holds,
 for a catch that acts on the refusal alone.

 THE PLAIN GRAMMAR'S OWN REFUSAL IS THE ONE READ: plain markdown has no syntax
 error, so the one way it refuses a text is a nesting too deep to read, which
 {@link parseMarkdownBody} raises as a {@link MarkdownParseError}. Where the
 strict grammar turns every failure into an {@link MdxParseError}, this one
 lets any other failure propagate, any other `RangeError` among them (ledger
 B100). THE ENGINE'S OWN STACK EXHAUSTION IS NOT READ HERE, for the reason
 {@link requireMdxRefusal} gives: the call that makes the parse reads its exhaustion as the
 refusal ({@link withMarkdownExhaustionRefused}), and an exhaustion of the block's other
 code reaches this catch raw and propagates.

 @param error - what the catch caught

 @returns Refusal the plain grammar raised, or the one {@link withMarkdownExhaustionRefused} built for an exhaustion of the parse

 @throws The caught value unchanged when it is anything but the plain
 grammar's refusal, the engine's stack exhaustion among it, an unexpected state that must keep propagating

 @example
 ```ts
 const refusal = requireMarkdownRefusal({ error, },);
 ```
 */
export function requireMarkdownRefusal({ error, }: { readonly error: unknown; },): MarkdownParseError {
  if (error instanceof MarkdownParseError)
    return error;
  throw error;
}

/**
 Makes one plain-grammar parse call, reading the engine's stack exhaustion it raises as the grammar's refusal.

 THE EXHAUSTION IS READ WHERE THE PARSE CALL ENDS, for the reason {@link withMdxExhaustionRefused} gives: a parse that
 ran out of stack where the refusal for it could not be built raises the engine's `RangeError`, and that is the nesting
 too deep to read that the refusal names. The refusal is built here, after the stack has unwound to this call. An
 exhaustion raised by any other code of the boundary's block is not raised by this call and propagates.

 @param parse - parse call, such as `parseMarkdownBody`, whose exhaustion is a fact about the text it was given

 @param input - what the parse is given

 @returns What the parse returns

 @throws {@link MarkdownParseError} built here, with the exhaustion as its cause, when the parse raises the engine's stack
 exhaustion; any other failure of the parse unchanged

 @example
 ```ts
 const root = withMarkdownExhaustionRefused({ parse: parseMarkdownBody, input: { body: 'A cat naps.', }, },);
 ```
 */
export function withMarkdownExhaustionRefused<Input, Output>(
  {
    parse,
    input,
  }: {
    readonly parse: (input: Input,) => Output;
    readonly input: Input;
  },
): Output {
  try {
    return parse(input,);
  }
  catch (error) {
    if (!isStackOverflow(error,))
      throw error;
    throw new MarkdownParseError({
      reading: 'stack-exhausted',
      cause: error,
    },);
  }
}

//region Refusals built before the stack is short
// A REFUSAL IS BUILT WITH THE STACK ALMOST SPENT, and the first refusal a process builds cannot be. V8 compiles
// a function the first time it is called, and a compile that finds too little stack to run in fails with the
// engine's own `RangeError` for a stack exhaustion (`src/codegen/compiler.cc`, `FailWithPreparedException`, in V8's
// source). So a body that exhausts the stack reached `new MdxParseError(...)` in the refusal handler of `parseMdxBody` with that code
// still uncompiled, and what escaped `parseMdxBody` was a raw `RangeError` and not the refusal this module
// promises. MEASURED on the built package with the stack-edge search of `stack-edge.test-fixture.ts`: in a process
// that had built no refusal, 299 of 300 searches found the raw `RangeError`, with the handler's `new MdxParseError`
// at the top of its stack, and 100 of 100 in a process with the optimizing compilers off; after one refusal taken at
// a safe depth, 99 of 100 found the refusal (the hundredth was the search's own recursion overflowing).
//
// THE CODE THAT BUILDS EACH REFUSAL IS RUN ONCE WHEN THE MODULE LOADS, so it is compiled before any stack is short
// of room for compiling it. The sample cause is a `RangeError` with the engine's message, the one a runaway recursion
// raises, so the constructors take the branch a real exhaustion takes. WHAT THIS DOES NOT PROMISE: V8 discards the
// compiled form of a function that goes unused through enough garbage collections and compiles it again on its next
// call, so a long-lived process that has built no refusal for a long time is back where a new one starts; and a caller
// that stands so close to the end of the stack that not even a small function can run still gets a raw `RangeError`,
// from the nesting check that runs first or from the handler. THAT RESIDUAL IS READ WHERE THE PARSE CALL ENDS:
// `withMdxExhaustionRefused` and `withMarkdownExhaustionRefused` wrap the parse call of every boundary that narrows
// with `requireMdxRefusal` or `requireMarkdownRefusal`, and build the same refusal from the engine's `RangeError`
// after the stack has unwound to them. The catch of the boundary does not read it: an exhaustion raised by other code
// of the same block is no fact about the text.
//
// WHAT HOLDS AT THE STACK EDGE, and what does not. Where the exhaustion falls inside such a wrapper the refusal is
// built and the caller of the parse reads the page as text. That is not the end of the work: the work that follows the
// parse (for the consistency pass, the rest of `proseViewOf` in `consistency-prose-text.ts`) needs more stack than the
// handler has left there, so a raw `RangeError` can still escape that work and reach `corpus-run/pass-consistency.ts`,
// which stops the pass for that page and leaves the page its wording; sweeps of about 1,500 caller depths over six
// attempts around the depth where `proseViewOf` exhausts the stack found it return the normal view or a raw
// `RangeError`, and never a converted view read as text (the edge moved between attempts as the engine compiled
// frames, so this is no proof that no window exists). The pass does not stand near that edge: measured on the built
// package, the engine allows 10,936 nested calls of a trivial body, a body at the nesting bound needs 16 or 17 calls
// more than a trivial one, and the pass is 42 to 51 calls deep when it reads the prose of a small slice, the parse's
// own calls included. So the edge is out of the pass's reach in practice.

/**
 The refusals of each grammar for a stack exhaustion.
 */
type ReadyRefusals = {
  /**
   The strict grammar's refusal.
   */
  readonly strict: MdxParseError;

  /**
   The plain grammar's refusal.
   */
  readonly plain: MarkdownParseError;
};

/**
 Builds each refusal for a stack exhaustion once and drops it.

 @returns Refusals built, which the module-level call that runs this at load drops

 @example
 ```ts
 const { strict, plain, } = readyRefusals();
 ```
 */
function readyRefusals(): ReadyRefusals {
  /**
   What an engine raises for a runaway recursion, as far as the refusals read it.
   */
  const exhaustion = new RangeError(STACK_OVERFLOW_MESSAGE,);
  return {
    strict: new MdxParseError({
      cause: exhaustion,
      droppedColumns: droppedWidth({ body: '', },),
    },),
    plain: new MarkdownParseError({
      reading: 'stack-exhausted',
      cause: exhaustion,
    },),
  };
}

// The refusals are built when the module loads and dropped: building them is the work, so their code is compiled.
readyRefusals();

//endregion Refusals built before the stack is short

//endregion MDX parsing
