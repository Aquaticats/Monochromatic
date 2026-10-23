/**
 Guards how the package asks whether a text is blank (ledgers B40 and B128).
 Whether text a model wrote shows a reader anything is asked of
 `rendersAsNothing` (`renders-as-nothing.ts`), never of `trim()`, `length` or
 a comparison with the empty string, each of which keeps every invisible
 character that is not whitespace. The rule rested on habit and review, and a
 census on 2026-10-05 found it asked the old way in more places. Each test of
 the old shape the package still makes is listed here with why its question
 is another one, so a new one fails this test until it says which.

 WHAT THE SCAN READS, in the package's source (tests and their fixtures are
 left out). A comparison (`===`, `!==`, `==`, `!=`) with the empty string,
 written as a string or as a template, either way round. The `length` of a
 `trim`, `trimStart` or `trimEnd` call, or of a name its function declares
 with one, compared with anything. The `length` of a name its function
 annotates as `string`, alone or in a union (a parameter, one destructured
 from an inline type among them, or a variable), compared with zero or asked
 whether it is less than one. And any of those texts, or its `length`, tested
 for truth: the test of an `if`, a `?:` or a loop, the operand of `!`, or
 the left operand of `&&` or `||`. Tests are keyed
 `path#site`, where the site is the nearest enclosing named function or
 class, and counted per key, so a second test inside a listed function fails
 as a new function's would.

 OUT OF ITS REACH, since the source carries no types to read: the `length` or
 the truth of a string no annotation names (a field read off a reply, a
 destructured binding with no type), which reads as an array's would; a
 comparison with a constant that holds the empty string; a blank needle
 handed to `includes`, `startsWith` or `indexOf`; a length floor on a text
 that was never trimmed; and the result of a helper that trims, compared by
 its `length` in another function. A helper that answers the comparison for
 its callers is read once, where it compares.

 THE FIXTURE CASE COMES FIRST, so the package-wide case is read against a
 scan shown able to find each form (ledger M21). Fixtures are cat-themed; the
 package case reads this package's own source.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import { compareCodePoints, } from '@monochromatic-dev/translation-repair-markup';
import { readPackageSource, } from './family-source.test-fixture.ts';
import {
  childNodes,
  identifierName,
  isTreeNode,
  memberName,
  parseSource,
  type SourceText,
  type TreeNode,
  unwrapped,
} from './source-scan.test-fixture.ts';
import { expectRecordAsListed, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scan-findings.test-fixture.ts';

//region Blank-text tests
// The scan: which nodes ask whether a text is blank by its bytes, and the walk
// that keys each by its file and enclosing named function or class.

/**
 Node kinds that open a function, which names the site of the tests inside
 and declares the names they read.
 */
const FUNCTION_KINDS: ReadonlySet<string> = new Set([
  'ArrowFunctionExpression',
  'FunctionDeclaration',
  'FunctionExpression',
],);

/**
 Node kinds that open a class, which names the site of the tests inside.
 */
const CLASS_KINDS: ReadonlySet<string> = new Set([
  'ClassDeclaration',
  'ClassExpression',
],);

/**
 Operators that ask whether two values are the same.
 */
const EQUALITY_OPERATORS: ReadonlySet<string> = new Set([
  '!=',
  '!==',
  '==',
  '===',
],);

/**
 Operators that compare two values, equality among them.
 */
const COMPARISON_OPERATORS: ReadonlySet<string> = new Set([
  ...EQUALITY_OPERATORS,
  '<',
  '<=',
  '>',
  '>=',
],);

/**
 String members that return their text with whitespace cut from an end.
 */
const TRIM_MEMBERS: ReadonlySet<string> = new Set([
  'trim',
  'trimEnd',
  'trimStart',
],);

/**
 Node kinds whose `test` field is tested for truth.
 */
const TRUTH_TEST_KINDS: ReadonlySet<string> = new Set([
  'ConditionalExpression',
  'DoWhileStatement',
  'ForStatement',
  'IfStatement',
  'WhileStatement',
],);

/**
 Logical operators whose left operand is tested for truth.
 */
const TRUTH_OPERATORS: ReadonlySet<string> = new Set([
  '&&',
  '||',
],);

/**
 Comparisons that, with a length on the left and one on the right, ask
 whether the length is less than one or not.
 */
const LESS_THAN_ONE_OPERATORS: ReadonlySet<string> = new Set([
  '<',
  '>=',
],);

/**
 Each ordering comparison as it reads with its two sides swapped.
 */
const SWAPPED_OPERATORS: ReadonlyMap<string, string> = new Map([
  [
    '<',
    '>',
  ],
  [
    '<=',
    '>=',
  ],
  [
    '>',
    '<',
  ],
  [
    '>=',
    '<=',
  ],
],);

/**
 Length, in characters, of an empty template literal: its two backticks.
 */
const EMPTY_TEMPLATE_LENGTH = 2;

/**
 Names in reach of one node whose text the scan can tell is a string.
 */
type KnownTexts = {
  /**
   Names declared with a `trim`, `trimStart` or `trimEnd` call.
   */
  readonly trimmed: ReadonlySet<string>;

  /**
   Names annotated as `string`.
   */
  readonly strings: ReadonlySet<string>;
};

/**
 Whether a node is the empty string, written as a string or as a template.

 @param node - node read

 @returns Whether it is that literal, parentheses aside

 @example
 ```ts
 const empty = isEmptyText({ node: comparison.right, },);
 ```
 */
function isEmptyText({ node, }: { readonly node: unknown; },): boolean {
  /**
   The node inside any parentheses.
   */
  const { inner, } = unwrapped({ node, },);
  if (!isTreeNode(inner,))
    return false;
  if (inner.type === 'Literal')
    return inner.value === '';
  return (inner.type === 'TemplateLiteral') && ((inner.end - inner.start) === EMPTY_TEMPLATE_LENGTH);
}

/**
 Whether comparing a length with a number asks whether there is any length
 at all: any comparison with zero, or one that asks whether the length is
 less than one.

 @param operator - comparison as written with the length on its left

 @param count - what the length is compared with

 @returns Whether the comparison separates no characters from some

 @example
 ```ts
 const asksNone = countsNone({ operator: '<', count: comparison.right, },); // true for `length < 1`
 ```
 */
function countsNone(
  {
    operator,
    count,
  }: {
    readonly operator: string;
    readonly count: unknown;
  },
): boolean {
  /**
   The number inside any parentheses.
   */
  const { inner, } = unwrapped({ node: count, },);
  if ((!isTreeNode(inner,)) || (inner.type !== 'Literal'))
    return false;
  if (inner.value === 0)
    return true;
  return (inner.value === 1) && LESS_THAN_ONE_OPERATORS.has(operator,);
}

/**
 Whether a node calls `trim`, `trimStart` or `trimEnd` on something.

 @param node - node read

 @returns Whether it is such a call, parentheses aside

 @example
 ```ts
 const trims = isTrimCall({ node: declarator.init, },);
 ```
 */
function isTrimCall({ node, }: { readonly node: unknown; },): boolean {
  /**
   The node inside any parentheses.
   */
  const { inner, } = unwrapped({ node, },);
  if ((!isTreeNode(inner,)) || (inner.type !== 'CallExpression'))
    return false;
  /**
   What the call calls, inside any parentheses.
   */
  const { inner: callee, } = unwrapped({ node: inner.callee, },);
  return isTreeNode(callee,)
    && (callee.type === 'MemberExpression')
    && TRIM_MEMBERS.has(memberName({ node: callee, },),);
}

/**
 Whether a type annotation says `string`, alone or as one member of a union
 such as `string | undefined`.

 @param annotation - annotation a name or a type member carries, or none

 @returns Whether it annotates a string

 @example
 ```ts
 const text = annotatesString({ annotation: parameter.typeAnnotation, },);
 ```
 */
function annotatesString({ annotation, }: { readonly annotation: unknown; },): boolean {
  if ((!isTreeNode(annotation,)) || (annotation.type !== 'TSTypeAnnotation') || (!isTreeNode(annotation.typeAnnotation,)))
    return false;
  /**
   The annotated type.
   */
  const annotated = annotation.typeAnnotation;
  if (annotated.type === 'TSStringKeyword')
    return true;
  return (annotated.type === 'TSUnionType')
    && childNodes({ node: annotated, },).some(function isString(member,): boolean {
      return member.type === 'TSStringKeyword';
    },);
}

/**
 The nodes one field of a node lists.

 @param value - field read, a list or anything else

 @returns Its nodes in order, none when the field is no list

 @example
 ```ts
 const parameters = listedNodes({ value: fn.params, },);
 ```
 */
function listedNodes({ value, }: { readonly value: unknown; },): readonly TreeNode[] {
  /**
   The field as a list, empty when it is none.
   */
  const listed: readonly unknown[] = Array.isArray(value,) ? value : [];
  return listed.filter(function isNode(member,): member is TreeNode {
    return isTreeNode(member,);
  },);
}

/**
 Members an inline object type annotates as `string`, read through one
 wrapping type such as `Readonly<{ ... }>`.

 @param annotation - annotation a destructured parameter carries, or none

 @returns Names of its string members, none for any other annotation

 @example
 ```ts
 const members = stringMembers({ annotation: pattern.typeAnnotation, },);
 ```
 */
function stringMembers({ annotation, }: { readonly annotation: unknown; },): ReadonlySet<string> {
  if ((!isTreeNode(annotation,)) || (!isTreeNode(annotation.typeAnnotation,)))
    return new Set();
  /**
   The annotated type.
   */
  const annotated = annotation.typeAnnotation;
  /**
   Type arguments of a wrapping type, none for a type written bare.
   */
  const wrapped = isTreeNode(annotated.typeArguments,) ? listedNodes({ value: annotated.typeArguments.params, },) : [];
  /**
   The inline object type: the annotation itself, or a wrapper's one argument.
   */
  const literal = (wrapped.length === 1) ? wrapped[0] : annotated;
  if ((literal === undefined) || (literal.type !== 'TSTypeLiteral'))
    return new Set();
  return new Set(listedNodes({ value: literal.members, },)
    .filter(function isStringMember(member,): boolean {
      return (member.type === 'TSPropertySignature') && annotatesString({ annotation: member.typeAnnotation, },);
    },)
    .map(function nameOf(member,): string {
      return identifierName({ node: member.key, },);
    },),);
}

/**
 Names one parameter binds to a string its annotation names.

 @param parameter - parameter of a function, with or without a default

 @returns The names, none where the annotation names no string

 @example
 ```ts
 const names = stringNamesOf({ parameter, },);
 ```
 */
function stringNamesOf({ parameter, }: { readonly parameter: TreeNode; },): readonly string[] {
  /**
   The parameter without its default.
   */
  const pattern = ((parameter.type === 'AssignmentPattern') && isTreeNode(parameter.left,)) ? parameter.left : parameter;
  if (pattern.type === 'Identifier')
    return annotatesString({ annotation: pattern.typeAnnotation, },) ? [identifierName({ node: pattern, },),] : [];
  if (pattern.type !== 'ObjectPattern')
    return [];
  /**
   Members the pattern's inline type annotates as strings.
   */
  const members = stringMembers({ annotation: pattern.typeAnnotation, },);
  return listedNodes({ value: pattern.properties, },)
    .filter(function bindsString(property,): boolean {
      return members.has(identifierName({ node: property.key, },),);
    },)
    .map(function boundName(property,): string {
      /**
       What the member is bound to, without its default.
       */
      const bound = (isTreeNode(property.value,) && (property.value.type === 'AssignmentPattern'))
        ? property.value.left
        : property.value;
      return identifierName({ node: bound, },);
    },);
}

/**
 Names a function, or a file outside every function, declares that the scan
 can tell are text: its string parameters, and the variables it declares with
 a trim or annotates as `string`. Functions nested inside are read when the
 walk reaches them.

 @param root - function or program read

 @param outer - names already in reach of it

 @returns Names in reach of the nodes inside

 @example
 ```ts
 const known = knownTextsIn({ root: fn, outer, },);
 ```
 */
function knownTextsIn(
  {
    root,
    outer,
  }: {
    readonly root: TreeNode;
    readonly outer: KnownTexts;
  },
): KnownTexts {
  /**
   Names declared with a trim, the outer ones among them.
   */
  const trimmed = new Set(outer.trimmed,);
  /**
   Names annotated as strings, the outer ones among them.
   */
  const strings = new Set(outer.strings,);
  /**
   Parameters of a function, none for a program.
   */
  const parameters = listedNodes({ value: root.params, },);
  for (const parameter of parameters) {
    for (const name of stringNamesOf({ parameter, },))
      strings.add(name,);
  }
  /**
   Nodes still to visit, none of them inside a nested function.
   */
  const pending: TreeNode[] = [...childNodes({ node: root, },),];
  for (let node = pending.pop(); node !== undefined; node = pending.pop()) {
    if (FUNCTION_KINDS.has(node.type,))
      continue;
    if ((node.type === 'VariableDeclarator') && isTreeNode(node.id,) && (node.id.type === 'Identifier')) {
      if (isTrimCall({ node: node.init, },))
        trimmed.add(identifierName({ node: node.id, },),);
      if (annotatesString({ annotation: node.id.typeAnnotation, },))
        strings.add(identifierName({ node: node.id, },),);
    }
    pending.push(...childNodes({ node, },),);
  }
  return {
    trimmed,
    strings,
  };
}

/**
 Which kind of text a node is, for the scan: a trim's result, a name
 annotated as a string, or neither.

 @param node - node read

 @param known - names in reach of the node

 @returns `trimmed`, `string`, or empty for anything else

 @example
 ```ts
 const kind = textKindOf({ node: test, known, },);
 ```
 */
function textKindOf(
  {
    node,
    known,
  }: {
    readonly node: unknown;
    readonly known: KnownTexts;
  },
): string {
  if (isTrimCall({ node, },))
    return 'trimmed';
  /**
   The node inside any parentheses.
   */
  const { inner, } = unwrapped({ node, },);
  /**
   The name the node is, empty for any other node.
   */
  const name = identifierName({ node: inner, },);
  if (name === '')
    return '';
  if (known.trimmed.has(name,))
    return 'trimmed';
  return known.strings.has(name,) ? 'string' : '';
}

/**
 Which kind of text a node reads the `length` of.

 @param node - node read

 @param known - names in reach of the node

 @returns `trimmed`, `string`, or empty where the node reads no such length

 @example
 ```ts
 const kind = lengthKindOf({ node: comparison.left, known, },);
 ```
 */
function lengthKindOf(
  {
    node,
    known,
  }: {
    readonly node: unknown;
    readonly known: KnownTexts;
  },
): string {
  /**
   The node inside any parentheses.
   */
  const { inner, } = unwrapped({ node, },);
  if ((!isTreeNode(inner,)) || (inner.type !== 'MemberExpression') || (memberName({ node: inner, },) !== 'length'))
    return '';
  return textKindOf({
    node: inner.object,
    known,
  },);
}

/**
 Whether a node is tested for truth as a text the scan knows, or as its
 `length`.

 @param node - operand or test read

 @param known - names in reach of the node

 @returns Whether its truth is a blankness test

 @example
 ```ts
 const tested = isTruthOfText({ node: statement.test, known, },);
 ```
 */
function isTruthOfText(
  {
    node,
    known,
  }: {
    readonly node: unknown;
    readonly known: KnownTexts;
  },
): boolean {
  return (textKindOf({
    node,
    known,
  },) !== '') || (lengthKindOf({
    node,
    known,
  },) !== '');
}

/**
 Whether a comparison asks whether a text is blank by its bytes.

 @param node - binary expression read

 @param known - names in reach of the node

 @returns Whether it compares with the empty string, compares a trim's
 length with anything, or asks whether an annotated string has any length

 @example
 ```ts
 const asks = comparesBlank({ node, known, },);
 ```
 */
function comparesBlank(
  {
    node,
    known,
  }: {
    readonly node: TreeNode;
    readonly known: KnownTexts;
  },
): boolean {
  /**
   The comparison's operator.
   */
  const operator = String(node.operator,);
  if (EQUALITY_OPERATORS.has(operator,) && (isEmptyText({ node: node.left, },) || isEmptyText({ node: node.right, },)))
    return true;
  if (!COMPARISON_OPERATORS.has(operator,))
    return false;
  /**
   Kind of text whose length the left side reads.
   */
  const left = lengthKindOf({
    node: node.left,
    known,
  },);
  /**
   Kind of text whose length the right side reads.
   */
  const right = lengthKindOf({
    node: node.right,
    known,
  },);
  if ((left === 'trimmed') || (right === 'trimmed'))
    return true;
  if (left === 'string') {
    return countsNone({
      operator,
      count: node.right,
    },);
  }
  return (right === 'string') && countsNone({
    operator: SWAPPED_OPERATORS.get(operator,) ?? operator,
    count: node.left,
  },);
}

/**
 Whether a node asks whether a text is blank by its bytes.

 @param node - node read

 @param known - names in reach of the node

 @returns Whether it is one of the tests the scan reads

 @example
 ```ts
 const asks = asksBlank({ node, known, },);
 ```
 */
function asksBlank(
  {
    node,
    known,
  }: {
    readonly node: TreeNode;
    readonly known: KnownTexts;
  },
): boolean {
  if (node.type === 'BinaryExpression') {
    return comparesBlank({
      node,
      known,
    },);
  }
  if (TRUTH_TEST_KINDS.has(node.type,)) {
    return isTruthOfText({
      node: node.test,
      known,
    },);
  }
  if ((node.type === 'UnaryExpression') && (node.operator === '!')) {
    return isTruthOfText({
      node: node.argument,
      known,
    },);
  }
  return (node.type === 'LogicalExpression')
    && TRUTH_OPERATORS.has(String(node.operator,),)
    && isTruthOfText({
      node: node.left,
      known,
    },);
}

/**
 Blankness tests in the package's source, counted per `path#site`.

 @param files - files read; tests and fixtures are skipped

 @returns Count per key, keys sorted

 @example
 ```ts
 const tests = blankTextTests({ files, },);
 ```
 */
function blankTextTests({ files, }: { readonly files: readonly SourceText[]; },): Readonly<Record<string, number>> {
  /**
   Count per key so far.
   */
  const counts = new Map<string, number>();
  for (const file of files) {
    if (file.isTest)
      continue;
    /**
     The file's program.
     */
    const { program, } = parseSource({ file, },);
    /**
     Nodes still to visit, each with its enclosing site and the names in its
     reach.
     */
    const pending: {
      readonly node: TreeNode;
      readonly site: string;
      readonly known: KnownTexts;
    }[] = [{
      node: program,
      site: '<module>',
      known: knownTextsIn({
        root: program,
        outer: {
          trimmed: new Set(),
          strings: new Set(),
        },
      },),
    },];
    for (let next = pending.pop(); next !== undefined; next = pending.pop()) {
      /**
       Node visited now, its site and the names in its reach.
       */
      const {
        node,
        site,
        known,
      } = next;
      /**
       Whether the node opens a function.
       */
      const opensFunction = FUNCTION_KINDS.has(node.type,);
      /**
       Site for the node's children: a function or class the node names.
       */
      const here = ((opensFunction || CLASS_KINDS.has(node.type,)) && isTreeNode(node.id,))
        ? identifierName({ node: node.id, },)
        : site;
      /**
       Names in reach of the node's children.
       */
      const inside = opensFunction
        ? knownTextsIn({
          root: node,
          outer: known,
        },)
        : known;
      if (asksBlank({
        node,
        known,
      },)) {
        /**
         The test's key.
         */
        const key = `${file.path}#${site}`;
        counts.set(
          key,
          (counts.get(key,) ?? 0) + 1,
        );
      }
      pending.push(...childNodes({ node, },).map(function withSite(child,) {
        return {
          node: child,
          site: here,
          known: inside,
        };
      },),);
    }
  }
  return Object.fromEntries([...counts,].toSorted(function byKey(
    [left,],
    [right,],
  ): number {
    return compareCodePoints({
      left,
      right,
    },);
  },),);
}

//endregion Blank-text tests

//region Listed tests
// Every blankness test the package makes, with why its question is not
// whether text a model wrote shows a reader anything. A reason many tests
// share is named once; a test of text a model wrote carries its own.

/**
 Why a test of a setting stands.
 */
const SETTING = 'a setting, key, path or model id read from the environment, the command line or the run\'s '
  + 'configuration, which no model writes';

/**
 Why a test of something the package put together stands.
 */
const ASSEMBLED = 'a block, note, line or key part the package assembled (declared names, cited references, picture '
  + 'readings, neighbouring passages, a dispute note, a log line), which it writes empty exactly when it has none';

/**
 Why a test of a provider's own field stands.
 */
const PROTOCOL = 'a frame, payload line or field of a provider\'s protocol, whose empty form the protocol defines';

/**
 Why a test of one stream delta stands.
 */
const DELTA = 'one stream delta of a reply, where an empty delta adds nothing and any other, one of spaces among '
  + 'them, is part of the reply being put together';

/**
 Why a test inside a scanner stands.
 */
const SCANNER = 'one character, token or line of a text a scanner walks, where blank is what the grammar being read '
  + 'calls blank';

/**
 Why a guard against an empty needle stands.
 */
const EMPTY_NEEDLE = 'an empty needle or key, which a search would find at every offset, refused before the search';

/**
 Why a test of text nobody generated stands.
 */
const PARSED = 'text of the original, the archive, front matter, corpus metadata or a reviewed reference, which no '
  + 'model wrote';

/**
 Why a test of the package's own empty string stands.
 */
const SENTINEL = 'an empty string the package itself writes or returns to say none: a helper that found nothing, a '
  + 'state not yet set, a constant for no prefix';

/**
 Why a test of something read back stands.
 */
const RECORD = 'a field of a record, file name, ledger, log line or sheet the package wrote and reads back';

/**
 Why a test of a program's output stands.
 */
const TOOL = 'output of a program or a lookup service, not a model\'s wording';

/**
 Why a test of a slice's text as it stands is right.
 */
const ARCHIVE_ABSENT = 'the archive or page text of a slice as it stands, which the package passes empty where the '
  + 'slice holds none';

/**
 Why a test of the standing text stands.
 */
const STANDING = 'the standing text of a slice, empty where neither the archive nor a lane holds wording there, '
  + 'which is when no slate is bought; whether a standing text shows a reader anything is the floors\' question '
  + '(`readStandingVerdict` validates it as a candidate)';

/**
 Why a test of an archive block revision stands where the review stage has
 already read it.
 */
const DECIDED_REMOVAL = 'an archive block revision after `runArchiveBlockReviewStage` has read one that shows a '
  + 'reader nothing as the empty removal, so the empty string is the decided removal';

/**
 Why a test of an audit finding's focus stands.
 */
const AUDIT_FOCUS = 'the focus one side of an audit finding rests on, returned empty for a side that anchored none; '
  + 'the anchor (`rendering-audit-anchor.ts`) anchors no focus that shows a reader nothing';

/**
 Why a test of an editor width arm's text stands.
 */
const WIDTH_ARM = 'the text an editor width arm shipped, which the trial writes empty when the arm shipped none';

/**
 Why a test of what crosses a program's standard streams stands.
 */
const STREAM = 'a passage handed to a program on standard input, and what the program wrote back: an empty one is no '
  + 'byte crossing the stream, which the wrap returns as it came or refuses, and whether the text shows a reader '
  + 'anything is not asked there';

/**
 Each blankness test the package makes, as `path#site`, with how many there
 are and why `rendersAsNothing` is not the question there.
 */
const HELD_TESTS: Readonly<Record<string, {
  readonly tests: number;
  readonly why: string;
}>> = {
  'archive/absolute-naturalness-review-stage.ts#summarizeSeat': {
    tests: 2,
    why: ASSEMBLED,
  },
  'adjudication/adjudicate-prompt.ts#buildAdjudicationMessages': {
    tests: 3,
    why: ASSEMBLED,
  },
  'provider/anthropic-completion.ts#extractAnthropicCompletion': {
    tests: 3,
    why: 'a payload line and a stop reason of the provider\'s protocol, and the prose beside a tool answer, '
      + 'compared only to log how much of it was set aside; whether the tool answered is asked of '
      + '`rendersAsNothing`',
  },
  'provider/anthropic-completion.ts#foldMessageDelta': {
    tests: 1,
    why: PROTOCOL,
  },
  'provider-stream/anthropic-delta-scan.ts#readDelta': {
    tests: 1,
    why: DELTA,
  },
  'provider-stream/anthropic-delta-scan.ts#readLine': {
    tests: 1,
    why: PROTOCOL,
  },
  'provider/anthropic-tool.ts#renderToolSystemPrompt': {
    tests: 1,
    why: ASSEMBLED,
  },
  'provider/anthropic-whole-message.ts#framesOf': {
    tests: 1,
    why: PROTOCOL,
  },
  'provider/anthropic-whole-message.ts#isProtocolWord': {
    tests: 1,
    why: PROTOCOL,
  },
  'archive/archive-block-review-wire.ts#buildArchiveBlockReviewMessages': {
    tests: 1,
    why: ASSEMBLED,
  },
  'archive/archive-block-review-wire.ts#isWritten': {
    tests: 1,
    why: ASSEMBLED,
  },
  'archive/archive-composite-revision.ts#compositeOfDisjointRevisions': {
    tests: 1,
    why: DECIDED_REMOVAL,
  },
  'archive/archive-replacement-candidates.ts#replacementCandidates': {
    tests: 1,
    why: DECIDED_REMOVAL,
  },
  'archive/archive-revision-shape.ts#revisionShapeFindings': {
    tests: 1,
    why: DECIDED_REMOVAL,
  },
  'markup/ascii-letters.ts#isAsciiDigits': {
    tests: 1,
    why: SCANNER,
  },
  'assembly/assembly-container-halves.ts#carriesText': {
    tests: 1,
    why: 'a replacement at a slice the archive never translated, where the empty string is how a half '
      + 'ships nothing; the translate lane decides no wording there that shows a reader nothing '
      + '(`assertAbsentSliceFilled`), and `spliceSlices` asks `rendersAsNothing` again',
  },
  'assembly/assembly-orphan-trim.ts#isDefinition': {
    tests: 1,
    why: SENTINEL,
  },
  'assembly/assembly-orphan-trim.ts#isGone': {
    tests: 1,
    why: SENTINEL,
  },
  'assembly/assembly-orphan-trim.ts#isProse': {
    tests: 1,
    why: SENTINEL,
  },
  'assembly/assembly-orphan-trim.ts#keep': {
    tests: 4,
    why: 'a footnote label a helper returns empty for a block that is no definition, and the gap and text '
      + 'the function itself is putting together',
  },
  'assembly/assembly-orphan-trim.ts#walkLine': {
    tests: 2,
    why: 'a blank line by the markup\'s own grammar, and a footnote label a helper returns empty for a line '
      + 'that is no definition',
  },
  'assembly/assembly-repetition.ts#whitespaceTokensOf': {
    tests: 1,
    why: SCANNER,
  },
  'provider/bedrock-ledger.ts#bedrockCreditUsdFrom': {
    tests: 1,
    why: SETTING,
  },
  'provider/bedrock-ledger.ts#bedrockLedgerPathFrom': {
    tests: 1,
    why: SETTING,
  },
  'provider/bedrock-ledger.ts#toEntry': {
    tests: 1,
    why: RECORD,
  },
  'provider/bedrock-stream-end.ts#isUsageChunk': {
    tests: 1,
    why: PROTOCOL,
  },
  'translate-floor/bilingual-pair-bound.ts#nonEmpty': {
    tests: 1,
    why: SCANNER,
  },
  'translate-floor/bilingual-pair-bound.ts#pairBoundFindings': {
    tests: 1,
    why: ARCHIVE_ABSENT,
  },
  'house-rule/candidate-ledger.ts#readContestLedger': {
    tests: 1,
    why: SETTING,
  },
  'adjudication/candidate-select-wire.ts#isCanonicalIndexText': {
    tests: 1,
    why: 'the text of a ballot\'s quoted index, asked whether it spells a whole number; the empty text is '
      + 'none, and any invisible character fails the digit test that follows',
  },
  'provider/chat-json-outcome.ts#readJsonOutcome': {
    tests: 2,
    why: SENTINEL,
  },
  'slice-reading/cited-reference-fetch.ts#failureOf': {
    tests: 1,
    why: TOOL,
  },
  'slice-reading/cited-reference-fetch.ts#fetchedOf': {
    tests: 3,
    why: TOOL,
  },
  'reference/cited-reference-lookup.ts#citedReferenceBlock': {
    tests: 1,
    why: SETTING,
  },
  'reference/cited-reference-lookup.ts#referenceLineOf': {
    tests: 2,
    why: TOOL,
  },
  'slice-reading/cited-reference-rule.ts#citedReferenceBlockText': {
    tests: 1,
    why: ASSEMBLED,
  },
  'slice-reading/cited-reference-rule.ts#citedReferenceCandidateLines': {
    tests: 1,
    why: ASSEMBLED,
  },
  'slice-reading/cited-reference-rule.ts#citedReferenceEvidence': {
    tests: 1,
    why: ASSEMBLED,
  },
  'reference/cited-reference-scan.ts#hasContent': {
    tests: 1,
    why: SCANNER,
  },
  'glossary/community-glossary.ts#departuresOf': {
    tests: 1,
    why: ARCHIVE_ABSENT,
  },
  'provider-stream/completion-shape.ts#readFinishReason': {
    tests: 1,
    why: PROTOCOL,
  },
  'corpus-pass/consolidate-driver-subject.ts#consolidationSubjectOf': {
    tests: 3,
    why: ASSEMBLED,
  },
  'consolidate/consolidate-gate-wire.ts#buildConsolidateGateMessages': {
    tests: 2,
    why: ASSEMBLED,
  },
  'consolidate/consolidate-gate-wire.ts#hasPolicy': {
    tests: 1,
    why: ASSEMBLED,
  },
  'artifact/consolidate-key.ts#consolidateRunShape': {
    tests: 1,
    why: ASSEMBLED,
  },
  'artifact/consolidate-key.ts#consolidateSliceKey': {
    tests: 5,
    why: ASSEMBLED,
  },
  'consolidate/consolidate-settle-context.ts#settlementContextOf': {
    tests: 3,
    why: ASSEMBLED,
  },
  'consolidate/consolidate-settle.ts#settleConsolidation': {
    tests: 1,
    why: STANDING,
  },
  'consolidate/consolidate-slice-buy.ts#buyConsolidationAttempt': {
    tests: 1,
    why: STANDING,
  },
  'consolidate/consolidate-standing-verdict.ts#readStandingVerdict': {
    tests: 1,
    why: ARCHIVE_ABSENT,
  },
  'consolidate/consolidate-wire.ts#buildConsolidateMessages': {
    tests: 2,
    why: ASSEMBLED,
  },
  'consolidate/consolidate-wire.ts#isDisplayed': {
    tests: 1,
    why: 'a lane\'s candidate on the producer sheet, which the package passes empty for a lane that wrote '
      + 'none',
  },
  'consolidate/consolidate-wire.ts#isPresent': {
    tests: 1,
    why: ASSEMBLED,
  },
  'consolidate/consolidate-wire.ts#renderBlock': {
    tests: 1,
    why: ASSEMBLED,
  },
  'consolidate/consolidation-polish-gate-wire.ts#buildConsolidationPolishGateMessages': {
    tests: 2,
    why: ASSEMBLED,
  },
  'consolidate/consolidation-polish-gate-wire.ts#hasRule': {
    tests: 1,
    why: ASSEMBLED,
  },
  'consolidate/consolidation-polish-round.ts#present': {
    tests: 1,
    why: ASSEMBLED,
  },
  'slice-reading/container-integrity.ts#nameOf': {
    tests: 1,
    why: PARSED,
  },
  'house-rule/contributor-name-authority.ts#carriesNames': {
    tests: 1,
    why: PARSED,
  },
  'house-rule/contributor-name-authority.ts#contributorDeclarationLines': {
    tests: 2,
    why: PARSED,
  },
  'house-rule/contributor-name-authority.ts#nonempty': {
    tests: 1,
    why: PARSED,
  },
  'corpus-source/corpus-name-index.ts#nonEmpty': {
    tests: 1,
    why: PARSED,
  },
  'page-check/corpus-run/archive-block-repair.ts#repairArchiveBlocks': {
    tests: 1,
    why: DECIDED_REMOVAL,
  },
  'page-check/corpus-run/archive-italic-spans.ts#titleLike': {
    tests: 1,
    why: PARSED,
  },
  'page-check/corpus-run/archive-italic-title-restore.ts#isTitleInProse': {
    tests: 1,
    why: SENTINEL,
  },
  'page-check/corpus-run/archive-stub.ts#stripStubMarkersWithOrigins': {
    tests: 3,
    why: SCANNER,
  },
  'artifact-pool/corpus-run/artifact-generation.ts#censusByGeneration': {
    tests: 1,
    why: RECORD,
  },
  'artifact-pool/corpus-run/artifact-placement.ts#readPlacement': {
    tests: 1,
    why: RECORD,
  },
  'artifact-pool/corpus-run/artifact-pool.ts#resolvePool': {
    tests: 4,
    why: 'a required commit as the invoker wrote it, which the entry reads from its environment and hands in, '
      + 'and the full id the function resolved it to, which it leaves empty when none was required',
  },
  'artifact-pool/corpus-run/artifact-provenance.ts#assertArtifactProvenance': {
    tests: 2,
    why: RECORD,
  },
  'artifact/corpus-run/artifact-two-lane-read-naturalness-seat.ts#parseFinding': {
    tests: 1,
    why: 'a finding stored in an artifact, held to the empty-string rule the record was written under; the '
      + 'wire guard asks `rendersAsNothing` before a finding is ever stored',
  },
  'artifact/corpus-run/artifact-two-lane-read-naturalness-seat.ts#parseNaturalnessReviewSeat': {
    tests: 1,
    why: RECORD,
  },
  'artifact/corpus-run/artifact-two-lane-read-consistency-rendering.ts#requireDefinitions': {
    tests: 1,
    why: RECORD,
  },
  'artifact-pool/corpus-run/artifact-two-lane-rebuild-rows.ts#carveDivergence': {
    tests: 1,
    why: SENTINEL,
  },
  'artifact/corpus-run/artifact-two-lane-rebuild.ts#rebuildPreparation': {
    tests: 1,
    why: SENTINEL,
  },
  'rendering-audit/corpus-run/attribution-line.ts#readSignature': {
    tests: 1,
    why: 'whether any name span stands between a signature\'s dashes and its end mark; a span that shows a '
      + 'reader nothing is still where the signer belongs, the name restore writes the signer\'s rendering '
      + 'into it, and `nameAuthorities` asks `rendersAsNothing` before a page name becomes a rendering',
  },
  'probe/corpus-run/attribution-read.ts#attributionEntryOf': {
    tests: 1,
    why: RECORD,
  },
  'probe/corpus-run/audit-sensitivity-oracle.ts#meetsOracle': {
    tests: 1,
    why: AUDIT_FOCUS,
  },
  'probe/corpus-run/audit-sensitivity-oracle.ts#shownText': {
    tests: 1,
    why: AUDIT_FOCUS,
  },
  'probe/corpus-run/audit-sensitivity-print.ts#voiceLines': {
    tests: 1,
    why: ASSEMBLED,
  },
  'page-check/corpus-run/blockquote-paragraphs.ts#blockquoteParagraphs': {
    tests: 2,
    why: SCANNER,
  },
  'page-check/corpus-run/blockquote-quote-unify.ts#wraps': {
    tests: 1,
    why: SCANNER,
  },
  'run/corpus-run/budget-sample-keys.ts#readProviderKeys': {
    tests: 10,
    why: SETTING,
  },
  'slice-cache/corpus-run/cache-account-commits.ts#sourceCommitOf': {
    tests: 1,
    why: TOOL,
  },
  'slice-cache/corpus-run/cache-account-git.ts#written': {
    tests: 1,
    why: TOOL,
  },
  'slice-cache/corpus-run/cache-account-sources.ts#listed': {
    tests: 1,
    why: TOOL,
  },
  'page-check/corpus-run/canadian-date-parts.ts#monthStartsName': {
    tests: 1,
    why: SCANNER,
  },
  'page-check/corpus-run/canadian-date-parts.ts#readDay': {
    tests: 1,
    why: SCANNER,
  },
  'page-check/corpus-run/canadian-spelling-capital.ts#hasLetter': {
    tests: 1,
    why: SCANNER,
  },
  'page-check/corpus-run/canadian-spelling-context.ts#isTokenCharacter': {
    tests: 2,
    why: SCANNER,
  },
  'run/corpus-run/cap-census-read.ts#endsField': {
    tests: 1,
    why: RECORD,
  },
  'run/corpus-run/cap-override.ts#resolveHardCapMinutes': {
    tests: 1,
    why: SETTING,
  },
  'run/corpus-run/command-flags.ts#isNamed': {
    tests: 1,
    why: SETTING,
  },
  'run/corpus-run/command-line.ts#optionReading': {
    tests: 1,
    why: SETTING,
  },
  'corpus-pass/corpus-run/corpus-pass-lines.ts#isSaid': {
    tests: 1,
    why: SENTINEL,
  },
  'run/corpus-run/corpus-pin-override.ts#readCorpusPinSetting': {
    tests: 6,
    why: SETTING,
  },
  'coverage-census/corpus-run/coverage-census-commit.ts#named': {
    tests: 1,
    why: TOOL,
  },
  'coverage-census/corpus-run/coverage-census-commit.ts#packageCommit': {
    tests: 1,
    why: TOOL,
  },
  'coverage-census/corpus-run/coverage-census-family.ts#isFamilyBundleName': {
    tests: 2,
    why: TOOL,
  },
  'coverage-census/corpus-run/coverage-census-input.ts#readBaselineCensus': {
    tests: 1,
    why: RECORD,
  },
  'coverage-census/corpus-run/coverage-census-print.ts#outermostInPackage': {
    tests: 1,
    why: TOOL,
  },
  'coverage-census/corpus-run/coverage-control-decoy.ts#isLocatable': {
    tests: 1,
    why: 'a span the decoy cut keeps clear of, where an empty one has no place in the text; `tryCase` '
      + 'throws on a span that shows a reader nothing before any decoy is cut',
  },
  'coverage-census/corpus-run/coverage-control.ts#tryCase': {
    tests: 1,
    why: SENTINEL,
  },
  'glossary/corpus-run/directory-id-name.ts#fits': {
    tests: 1,
    why: PARSED,
  },
  'glossary/corpus-run/directory-id-name.ts#isId': {
    tests: 1,
    why: PARSED,
  },
  'glossary/corpus-run/directory-id-name.ts#isOtherRendering': {
    tests: 1,
    why: PARSED,
  },
  'glossary/corpus-run/directory-id-name.ts#nonEmpty': {
    tests: 1,
    why: PARSED,
  },
  'glossary/corpus-run/directory-id-name.ts#readsAs': {
    tests: 1,
    why: PARSED,
  },
  'page-check/corpus-run/dropped-destinations.ts#markdownDestinations': {
    tests: 1,
    why: 'a link\'s destination, an address and no wording, where an empty one leads nowhere a page could '
      + 'owe',
  },
  'bench/corpus-run/editor-calibrate-run.ts#runEditorCalibrate': {
    tests: 1,
    why: ASSEMBLED,
  },
  'bench/corpus-run/editor-width-control.ts#damageable': {
    tests: 1,
    why: SENTINEL,
  },
  'bench/corpus-run/editor-width-model.ts#classifyWidths': {
    tests: 2,
    why: WIDTH_ARM,
  },
  'bench/corpus-run/editor-width-slice.ts#runWidthSlice': {
    tests: 2,
    why: WIDTH_ARM,
  },
  'glossary/corpus-run/env-text-setting.ts#textSettingOf': {
    tests: 1,
    why: SETTING,
  },
  'page-pass/corpus-run/handle-gloss-place.ts#carriesGloss': {
    tests: 1,
    why: 'an appearance\'s gloss after `appearancesIn` has read a parenthesis that shows a reader nothing '
      + 'as none, so the empty string is the decided absence',
  },
  'page-pass/corpus-run/heading-title-lines.ts#titleOf': {
    tests: 1,
    why: 'the title a helper returns empty for a line that is no HTML heading; whether a heading\'s '
      + 'rendering shows a reader anything is asked of `rendersAsNothing` where a rendering is taken from '
      + 'it (`title-reference-unify.ts`)',
  },
  'insertion/corpus-run/insertion-carried-anchor.ts#abutting': {
    tests: 1,
    why: PARSED,
  },
  'insertion/corpus-run/markup-slice.ts#hasContent': {
    tests: 1,
    why: SCANNER,
  },
  'run/corpus-run/meter-report-text.ts#spanText': {
    tests: 1,
    why: ASSEMBLED,
  },
  'bench/corpus-run/model-catalog-compare.ts#classify': {
    tests: 2,
    why: TOOL,
  },
  'bench/corpus-run/model-catalog-print.ts#printModelCatalog': {
    tests: 1,
    why: SETTING,
  },
  'page-check/corpus-run/name-gloss-restore.ts#readGlossLine': {
    tests: 1,
    why: PARSED,
  },
  'page-pass/corpus-run/ordinal-style.ts#filled': {
    tests: 1,
    why: SCANNER,
  },
  'consistency/consistency-sheet.ts#buildConsistencyMessages': {
    tests: 1,
    why: ASSEMBLED,
  },
  'consistency/consistency-run-off.ts#buildConsistencyRunOffMessages': {
    tests: 1,
    why: ASSEMBLED,
  },
  'consistency/consistency-carry-writer-sheet.ts#buildCarryWriterMessages': {
    tests: 1,
    why: ASSEMBLED,
  },
  'consistency/consistency-carry-writer-sheet.ts#isPresent': {
    tests: 1,
    why: ASSEMBLED,
  },
  'consistency/consistency-carry-choice-sheet.ts#buildCarryChoiceMessages': {
    tests: 1,
    why: ASSEMBLED,
  },
  'consistency/consistency-fold.ts#locateWords': {
    tests: 1,
    why: EMPTY_NEEDLE,
  },
  'consistency/consistency-slot-definitions.ts#definitionsMovingSlotOf': {
    tests: 1,
    why: SENTINEL,
  },
  'consistency/consistency-count.ts#locatedRendering': {
    tests: 1,
    why: SENTINEL,
  },
  'consistency/consistency-contains-words.ts#containsWords': {
    tests: 2,
    why: EMPTY_NEEDLE,
  },
  'consistency/consistency-label-form.ts#trailingNumberOf': {
    tests: 1,
    why: SENTINEL,
  },
  'markup/latin-letters.ts#isSpacedScriptWordCharacter': {
    tests: 1,
    why: SCANNER,
  },
  'shared-contract/page-definitions.ts#withDefinitions': {
    tests: 1,
    why: SENTINEL,
  },
  'typography/typography-link-text.ts#isSpace': {
    tests: 1,
    why: SCANNER,
  },
  'page-pass/corpus-run/ordinal-style.ts#renderOrdinal': {
    tests: 1,
    why: SENTINEL,
  },
  'corpus-pass/corpus-run/pass-attest-references.ts#attestPassReferences': {
    tests: 1,
    why: ASSEMBLED,
  },
  'corpus-pass/corpus-run/pass-entry-persist.ts#printSettledLines': {
    tests: 1,
    why: ASSEMBLED,
  },
  'corpus-pass/corpus-run/pass-page-titles.ts#passPageTitles': {
    tests: 1,
    why: ASSEMBLED,
  },
  'corpus-pass/corpus-run/pass-prepare.ts#isLine': {
    tests: 1,
    why: ASSEMBLED,
  },
  'corpus-pass/corpus-run/pass-prepare.ts#prepareOver': {
    tests: 1,
    why: ASSEMBLED,
  },
  'corpus-pass/corpus-run/pass-prepare.ts#preparePassEntry': {
    tests: 1,
    why: ASSEMBLED,
  },
  'insertion/corpus-run/pool-generation.ts#isKnown': {
    tests: 1,
    why: RECORD,
  },
  'run/corpus-run/process-identity.ts#startTicksOfStat': {
    tests: 1,
    why: TOOL,
  },
  'glossary/corpus-run/prose-ranges.ts#pastConstruct': {
    tests: 2,
    why: SCANNER,
  },
  'page-check/corpus-run/published-page-check.ts#filledAnAnchor': {
    tests: 1,
    why: ARCHIVE_ABSENT,
  },
  'page-check/corpus-run/quote-style-unify.ts#unifyQuoteStyle': {
    tests: 1,
    why: SENTINEL,
  },
  'probe/corpus-run/rendering-audit-settled-buy.ts#readPage': {
    tests: 1,
    why: ASSEMBLED,
  },
  'probe/corpus-run/rendering-audit-settled-digest.ts#digestAuditedText': {
    tests: 1,
    why: ASSEMBLED,
  },
  'probe/corpus-run/rendering-audit-settled-report-run.ts#runSettledReport': {
    tests: 1,
    why: SETTING,
  },
  'run/corpus-run/report-log-read.ts#linesOfLog': {
    tests: 1,
    why: SCANNER,
  },
  'run/corpus-run/required-providers.ts#assertRequiredProvidersReady': {
    tests: 1,
    why: SETTING,
  },
  'bench/corpus-run/roster-card-ask.ts#readAsk': {
    tests: 1,
    why: SETTING,
  },
  'bench/corpus-run/roster-card-listing.ts#fetchListing': {
    tests: 1,
    why: SETTING,
  },
  'run/corpus-run/run-config.ts#resolveRunsDir': {
    tests: 1,
    why: SETTING,
  },
  'run/corpus-run/run-providers.ts#configureProviders': {
    tests: 20,
    why: SETTING,
  },
  'run/corpus-run/budget-sample-run.ts#sampleBudgets': {
    tests: 3,
    why: SETTING,
  },
  'provider/direct-usage.ts#fingerprintOf': {
    tests: 1,
    why: PROTOCOL,
  },
  'provider/claude-body.ts#buildClaudeBody': {
    tests: 1,
    why: SENTINEL,
  },
  'provider/claude-stream-facts.ts#claudeStreamFactsOf': {
    tests: 4,
    why: PROTOCOL,
  },
  'run/corpus-run/run-timing-parse.ts#countIn': {
    tests: 1,
    why: RECORD,
  },
  'run/corpus-run/run-timing-parse.ts#readCallTiming': {
    tests: 2,
    why: RECORD,
  },
  'insertion/corpus-run/runner-closure.ts#readRunnerClosure': {
    tests: 1,
    why: SETTING,
  },
  'insertion/corpus-run/runner-closure.ts#scanFor': {
    tests: 1,
    why: SCANNER,
  },
  'probe/corpus-run/score-crosscheck-authors.ts#isSet': {
    tests: 1,
    why: ASSEMBLED,
  },
  'glossary/corpus-run/sheet-path.ts#isSeedSafe': {
    tests: 1,
    why: SETTING,
  },
  'slice-cache/corpus-run/slice-cache-namespace.ts#belongsToNamespace': {
    tests: 1,
    why: SENTINEL,
  },
  'slice-cache/corpus-run/slice-cache-namespace.ts#discardNamespace': {
    tests: 1,
    why: RECORD,
  },
  'slice-cache/corpus-run/slice-cache-namespace.ts#isNamed': {
    tests: 1,
    why: SENTINEL,
  },
  'slice-cache/corpus-run/slice-census-entry.ts#censusEntry': {
    tests: 1,
    why: SENTINEL,
  },
  'slice-cache/corpus-run/slice-cost-report-run.ts#reportSliceCost': {
    tests: 1,
    why: SETTING,
  },
  'slice-cache/corpus-run/slice-overlap.ts#readOverlapSetting': {
    tests: 1,
    why: SETTING,
  },
  'run/corpus-run/spend-ceiling.ts#resolveSpendCeilingUsd': {
    tests: 1,
    why: SETTING,
  },
  'run/corpus-run/spend-read.ts#readSpendLine': {
    tests: 1,
    why: RECORD,
  },
  'run/corpus-run/spend-read.ts#usdOf': {
    tests: 1,
    why: RECORD,
  },
  'page-pass/corpus-run/title-reference-rewrite.ts#nonEmpty': {
    tests: 1,
    why: SCANNER,
  },
  'page-pass/corpus-run/title-reference-rewrite.ts#rewriteLocated': {
    tests: 1,
    why: 'whether the small-letter words leading into a glossed run used the whole run up; what is left is '
      + 'replaced by the heading\'s rendering whatever it shows, so a remainder that shows a reader '
      + 'nothing is written over with the title',
  },
  'page-pass/corpus-run/title-reference-scope.ts#framedPageLine': {
    tests: 2,
    why: SENTINEL,
  },
  'bench/corpus-run/window-trial-ledger.ts#endAtLineBoundary': {
    tests: 2,
    why: RECORD,
  },
  'bench/corpus-run/window-trial-ledger.ts#toPlaced': {
    tests: 1,
    why: RECORD,
  },
  'bench/corpus-run/window-trial-slice.ts#runSliceArms': {
    tests: 1,
    why: PARSED,
  },
  'page-check/corpus-run/would-ship-text.ts#archiveStandsOr': {
    tests: 1,
    why: RECORD,
  },
  'page-check/corpus-run/would-ship-text.ts#lanesAgreedOn': {
    tests: 1,
    why: RECORD,
  },
  'corpus-source/corpus-source.ts#nonEmpty': {
    tests: 1,
    why: TOOL,
  },
  'archive/coverage-wire.ts#buildCoverageMessages': {
    tests: 1,
    why: ASSEMBLED,
  },
  'adjudication/critic-prompt.ts#buildCriticMessages': {
    tests: 2,
    why: ASSEMBLED,
  },
  'adjudication/declared-identity-rule.ts#declaredNamesBlock': {
    tests: 1,
    why: ASSEMBLED,
  },
  'adjudication/declared-names-evidence.ts#declaredNamesEvidence': {
    tests: 1,
    why: ASSEMBLED,
  },
  'lane-contest/delivery-coherence.ts#assertDeliveryCoherent': {
    tests: 2,
    why: RECORD,
  },
  'repair/derive-seeds.ts#splitSentences': {
    tests: 1,
    why: SCANNER,
  },
  'translate-floor/disputed-wording.ts#disputedWordingFindings': {
    tests: 1,
    why: 'a candidate with its whitespace taken out, asked whether any character is left to search for a '
      + 'disputed wording; an invisible one is searched and holds none',
  },
  'adjudication/document-preparation.ts#prepareDocumentPair': {
    tests: 1,
    why: ASSEMBLED,
  },
  'repair/edit-prompt.ts#buildEditorMessages': {
    tests: 3,
    why: ASSEMBLED,
  },
  'repair/edit-prompt.ts#isPresent': {
    tests: 1,
    why: ASSEMBLED,
  },
  'repair/edit-prompt.ts#regionBlock': {
    tests: 1,
    why: ARCHIVE_ABSENT,
  },
  'rendering-audit/entry-notes.ts#isWhitespace': {
    tests: 1,
    why: SCANNER,
  },
  'repair/fidelity-damage.ts#alterSharedNumber': {
    tests: 2,
    why: SENTINEL,
  },
  'repair/fidelity-damage.ts#deleteOneSentence': {
    tests: 2,
    why: 'a sentence a helper returns empty when the slice offers none, and the archive text left once the '
      + 'control has cut that sentence',
  },
  'repair/fidelity-damage.ts#insertBorrowedSentence': {
    tests: 2,
    why: SENTINEL,
  },
  'repair/fidelity-damage.ts#isUsable': {
    tests: 1,
    why: SENTINEL,
  },
  'probe/fidelity-reference-build.ts#buildReviewedFidelityReference': {
    tests: 2,
    why: PARSED,
  },
  'probe/fidelity-reference-read.ts#readReference': {
    tests: 1,
    why: PARSED,
  },
  'probe/fidelity-reference-request.ts#blankIdentity': {
    tests: 1,
    why: SETTING,
  },
  'probe/fidelity-reference-text.ts#applyReviewedEdits': {
    tests: 2,
    why: PARSED,
  },
  'repair/fidelity-window-positions.ts#fidelityWindowPositions': {
    tests: 1,
    why: PARSED,
  },
  'repair/fidelity-window.ts#present': {
    tests: 2,
    why: PARSED,
  },
  'document-structure/footnote-graph.ts#collectRegionHits': {
    tests: 1,
    why: SCANNER,
  },
  'document-structure/footnote-graph.ts#scanFullwidthMarkers': {
    tests: 1,
    why: SCANNER,
  },
  'document-structure/footnote-identifier.ts#carriesText': {
    tests: 1,
    why: SCANNER,
  },
  'document-structure/footnote-mentions.ts#opensDefinition': {
    tests: 1,
    why: SCANNER,
  },
  'page-pass/footnote-rewrite-map.ts#footnoteRewriteMap': {
    tests: 1,
    why: EMPTY_NEEDLE,
  },
  'slice-reading/front-matter-comment-authority.ts#contributorAttribution': {
    tests: 1,
    why: PARSED,
  },
  'slice-reading/front-matter-comment-authority.ts#locationComment': {
    tests: 1,
    why: PARSED,
  },
  'translate-floor/front-matter-translation.ts#validateFrontMatterTranslation': {
    tests: 1,
    why: 'what a front matter candidate writes outside its metadata block, where anything but whitespace, '
      + 'an invisible character included, is text shipped outside the block and is refused',
  },
  'glossary/glossary-match.ts#isSpaceLike': {
    tests: 1,
    why: SCANNER,
  },
  'glossary/glossary-match.ts#occurrenceStarts': {
    tests: 1,
    why: EMPTY_NEEDLE,
  },
  'glossary/grace-override.ts#adoptCalibrationGrace': {
    tests: 1,
    why: SETTING,
  },
  'glossary/grace-override.ts#readWindowDial': {
    tests: 1,
    why: SETTING,
  },
  'slice-reading/grade-sheet-read.ts#headedWords': {
    tests: 1,
    why: RECORD,
  },
  'glossary/handle-token.ts#carriesHandleToken': {
    tests: 1,
    why: EMPTY_NEEDLE,
  },
  'house-rule/identity-context.ts#isDeclared': {
    tests: 1,
    why: PARSED,
  },
  'house-rule/identity-han-items.ts#nonEmpty': {
    tests: 1,
    why: PARSED,
  },
  'slice-reading/image-reading-pair.ts#stated': {
    tests: 1,
    why: ASSEMBLED,
  },
  'slice-reading/image-reading-sense.ts#solidCharacters': {
    tests: 1,
    why: SCANNER,
  },
  'slice-reading/image-reading-stage.ts#readImageAsset': {
    tests: 1,
    why: 'a reply of no characters, which the log names an empty reply; one that shows a reader nothing '
      + 'and is not empty is refused as too short by `readingMakesSense`, and either way the picture '
      + 'stays unread',
  },
  'markup/inline-container-tags.ts#scan': {
    tests: 1,
    why: SCANNER,
  },
  'corpus-source/insertion-separator.ts#composedInsertion': {
    tests: 4,
    why: 'the join of the fragments that say something, empty exactly when none did, and the archive text '
      + 'either side of the boundary; each fragment is asked of `rendersAsNothing` before the join',
  },
  'repair/inspect-paragraph.ts#inspectParagraph': {
    tests: 1,
    why: ASSEMBLED,
  },
  'adjudication/introduced-defect-probe.ts#runIntroducedDefectProbe': {
    tests: 1,
    why: ASSEMBLED,
  },
  'adjudication/introduced-defect-screen.ts#restatesPriorIssue': {
    tests: 1,
    why: 'a claim\'s quote as the screen folded it, thrown on when empty since an empty quote sits inside '
      + 'every prior quote; the screen asks `rendersAsNothing` of each side before this reader runs',
  },
  'adjudication/introduced-defect-wire.ts#buildIntroducedDefectMessages': {
    tests: 3,
    why: ASSEMBLED,
  },
  'run/judge-fidelity.ts#runFidelityTrial': {
    tests: 1,
    why: ASSEMBLED,
  },
  'lane-contest/lane-contest-key.ts#laneContestRunShape': {
    tests: 1,
    why: ASSEMBLED,
  },
  'lane-contest/lane-contest-stage.ts#contestLaneSlice': {
    tests: 1,
    why: ARCHIVE_ABSENT,
  },
  'lane-contest/lane-contest-wire.ts#buildLaneContestMessages': {
    tests: 2,
    why: ASSEMBLED,
  },
  'lane-contest/lane-contest-wire.ts#hasPolicy': {
    tests: 1,
    why: ASSEMBLED,
  },
  'glossary/licensed-quotes.ts#isUsable': {
    tests: 1,
    why: 'the quoted text of a claim\'s anchored span, the document\'s own slice, empty at an insertion '
      + 'point; `locateQuote` anchors no quote that shows a reader nothing',
  },
  'slice-reading/line-structure-addendum.ts#isPresent': {
    tests: 1,
    why: ASSEMBLED,
  },
  'house-rule/linked-title-declared-name.ts#differs': {
    tests: 1,
    why: PARSED,
  },
  'house-rule/linked-title-declared-name.ts#nonEmpty': {
    tests: 1,
    why: PARSED,
  },
  'typography/log-context.ts#isNamed': {
    tests: 2,
    why: ASSEMBLED,
  },
  'typography/log-context.ts#laneSliceText': {
    tests: 1,
    why: ASSEMBLED,
  },
  'reference/lookup-cache.ts#lookupCacheDir': {
    tests: 1,
    why: SETTING,
  },
  'reference/lookup-cache.ts#packageCacheDir': {
    tests: 1,
    why: SETTING,
  },
  'markup/markup-atom-scan.ts#isFootnoteLabel': {
    tests: 2,
    why: SCANNER,
  },
  'markup/markup-atom-scan.ts#tagEnd': {
    tests: 1,
    why: SCANNER,
  },
  'document-structure/mask-container-tags.ts#multiLineTagEnd': {
    tests: 1,
    why: SCANNER,
  },
  'markup/mdx-tag-start.ts#opensMdxTag': {
    tests: 1,
    why: SCANNER,
  },
  'house-rule/name-projection.ts#nameOccurrences': {
    tests: 1,
    why: EMPTY_NEEDLE,
  },
  'typography/nested-single-quotes.ts#isDigitAt': {
    tests: 1,
    why: SCANNER,
  },
  'typography/nested-single-quotes.ts#opensAfter': {
    tests: 1,
    why: SCANNER,
  },
  'typography/nested-single-quotes.ts#opensBeforeMarkup': {
    tests: 1,
    why: SCANNER,
  },
  'typography/nested-single-quotes.ts#singleShape': {
    tests: 1,
    why: SCANNER,
  },
  'provider/neuralwatt-comments.ts#commentsOfLine': {
    tests: 1,
    why: SENTINEL,
  },
  'provider/neuralwatt-day.ts#neuralWattDayStartFrom': {
    tests: 1,
    why: SETTING,
  },
  'provider/neuralwatt-ledger-config.ts#neuralWattDailyWhFrom': {
    tests: 1,
    why: SETTING,
  },
  'provider/neuralwatt-ledger-config.ts#neuralWattLedgerPathFrom': {
    tests: 1,
    why: SETTING,
  },
  'provider/neuralwatt-ledger.ts#toEntry': {
    tests: 1,
    why: RECORD,
  },
  'provider/openrouter-endpoint.ts#nameOf': {
    tests: 1,
    why: PROTOCOL,
  },
  'provider/openrouter-error-finish.ts#openRouterErrorFinishOf': {
    tests: 1,
    why: PROTOCOL,
  },
  'rendering-audit/page-name-glossary.ts#toPair': {
    tests: 1,
    why: PARSED,
  },
  'artifact/page-title-lexicon-wire.ts#buildPageTitleLexiconMessages': {
    tests: 1,
    why: ASSEMBLED,
  },
  'stage/pair-blocks-wire.ts#buildBlockPairingMessages': {
    tests: 2,
    why: ASSEMBLED,
  },
  'shared-contract/photo-reference.ts#assetNameOf': {
    tests: 1,
    why: PARSED,
  },
  'archive/prepare-block-pairing.ts#prepareBlockPairing': {
    tests: 1,
    why: ASSEMBLED,
  },
  'translate-stage/present-tense-words.ts#isWhitespace': {
    tests: 2,
    why: SCANNER,
  },
  'translate-stage/present-tense-words.ts#isWordCharacter': {
    tests: 1,
    why: SCANNER,
  },
  'translate-stage/present-tense-words.ts#namesAt': {
    tests: 1,
    why: EMPTY_NEEDLE,
  },
  'page-pass/preservation-check.ts#blank': {
    tests: 1,
    why: EMPTY_NEEDLE,
  },
  'adjudication/producer-silence.ts#coverageGapLines': {
    tests: 3,
    why: ASSEMBLED,
  },
  'provider-stream/quote-depth-clamp.ts#clampLine': {
    tests: 1,
    why: SCANNER,
  },
  'provider-stream/quote-line.ts#carriesContent': {
    tests: 1,
    why: SCANNER,
  },
  'provider-stream/quote-line.ts#pastQuoteMarkers': {
    tests: 1,
    why: SCANNER,
  },
  'markup/reading-corroboration.ts#collapsedWhitespace': {
    tests: 1,
    why: SCANNER,
  },
  'markup/reading-refusal.ts#readsAsRefusal': {
    tests: 1,
    why: 'a length bound on a picture reading, past which it is no refusal sentence; no test of whether it '
      + 'is blank',
  },
  'reference/reference-line-head.ts#numberedReferenceLines': {
    tests: 1,
    why: ASSEMBLED,
  },
  'reference/reference-line-head.ts#pageText': {
    tests: 1,
    why: SENTINEL,
  },
  'repair/refine-prompt.ts#buildRefineMessages': {
    tests: 2,
    why: ASSEMBLED,
  },
  'repair/refine-selection-context.ts#noteEvidence': {
    tests: 1,
    why: ASSEMBLED,
  },
  'repair/refine-slice-key.ts#refineRunShape': {
    tests: 1,
    why: ASSEMBLED,
  },
  'repair/refine-slice-key.ts#refineSliceKey': {
    tests: 2,
    why: ASSEMBLED,
  },
  'corpus-source/rendered-break-prompt.ts#sourceBreakCounts': {
    tests: 1,
    why: ARCHIVE_ABSENT,
  },
  'rendering-audit/rendering-audit-prompt.ts#buildRenderingAuditMessages': {
    tests: 2,
    why: ASSEMBLED,
  },
  'repair/repair-chunk.ts#repairChunk': {
    tests: 1,
    why: ASSEMBLED,
  },
  'adjudication/repair-selection-evidence.ts#repairSelectionSourceEvidence': {
    tests: 2,
    why: ASSEMBLED,
  },
  'assembly/repair-sheet.ts#renderRegion': {
    tests: 1,
    why: ARCHIVE_ABSENT,
  },
  'repair/repair-slice-buy.ts#repairUnderSignal': {
    tests: 1,
    why: ASSEMBLED,
  },
  'repair/repair-slice-key.ts#repairRunShape': {
    tests: 1,
    why: ASSEMBLED,
  },
  'repair/repair-slice-key.ts#repairSliceKey': {
    tests: 3,
    why: ASSEMBLED,
  },
  'provider/request-pace.ts#hyperRequestsPerHour': {
    tests: 1,
    why: SETTING,
  },
  'adjudication/resolution-wire.ts#buildResolutionMessages': {
    tests: 2,
    why: ASSEMBLED,
  },
  'lane-contest/sheet-binding.ts#assertSheetMatchesManifest': {
    tests: 1,
    why: RECORD,
  },
  'lane-contest/sheet-binding.ts#requireSheetSeed': {
    tests: 1,
    why: RECORD,
  },
  'typography/semantic-wrap-binary.ts#wrapWithLintBinary': {
    tests: 2,
    why: STREAM,
  },
  'house-rule/sheet-line-text.ts#flattenSpace': {
    tests: 1,
    why: SCANNER,
  },
  'translate-floor/slice-cost-read.ts#spelledAs': {
    tests: 1,
    why: RECORD,
  },
  'translate-floor/source-break-display.ts#sourceBreakDisplay': {
    tests: 1,
    why: ARCHIVE_ABSENT,
  },
  'glossary/source-only-breaks.ts#sourceOnlyBreakFindings': {
    tests: 1,
    why: ARCHIVE_ABSENT,
  },
  'provider/stream-completion.ts#extractStreamedCompletion': {
    tests: 1,
    why: PROTOCOL,
  },
  'provider/stream-completion.ts#foldChunk': {
    tests: 1,
    why: DELTA,
  },
  'provider-stream/stream-cut.ts#reportStreamProgress': {
    tests: 1,
    why: PROTOCOL,
  },
  'provider-stream/stream-degeneration.ts#watchForDegeneration': {
    tests: 1,
    why: DELTA,
  },
  'provider-stream/stream-delta-scan.ts#readLine': {
    tests: 4,
    why: 'a payload line and a served-by name of the provider\'s protocol, and one frame\'s content and '
      + 'reasoning deltas, where an empty delta adds nothing and any other is part of the reply',
  },
  'provider-stream/stream-delta-scan.ts#reasoningField': {
    tests: 1,
    why: DELTA,
  },
  'provider-stream/stream-recurrence-watch.ts#watchForRecurrence': {
    tests: 1,
    why: DELTA,
  },
  'translate-floor/translate-community-term.ts#findingsFor': {
    tests: 1,
    why: SENTINEL,
  },
  'translate-floor/translate-floor-ground.ts#readFrontMatterGround': {
    tests: 1,
    why: ARCHIVE_ABSENT,
  },
  'glossary/translate-quote-balance.ts#isBlank': {
    tests: 1,
    why: SCANNER,
  },
  'translate-floor/translate-sheet-leak.ts#fencedLabel': {
    tests: 1,
    why: 'whether a line\'s two fence runs leave any character between them; whether what lies between is a '
      + 'label is asked of `rendersAsNothing`',
  },
  'translate-floor/translate-signer-handle.ts#findingsAt': {
    tests: 1,
    why: SENTINEL,
  },
  'glossary/translate-skeleton-page.ts#readPageSkeleton': {
    tests: 1,
    why: ARCHIVE_ABSENT,
  },
  'translate-stage/translate-slate-evidence.ts#translateSlateEvidence': {
    tests: 5,
    why: ASSEMBLED,
  },
  'translate-lane/translate-slice-key.ts#translateRunShape': {
    tests: 1,
    why: ASSEMBLED,
  },
  'translate-lane/translate-slice-key.ts#translateSliceKey': {
    tests: 4,
    why: ASSEMBLED,
  },
  'translate-lane/translate-slice.ts#settleTranslateSlice': {
    tests: 1,
    why: ARCHIVE_ABSENT,
  },
  'translate-floor/translate-untranslated.ts#withoutWhitespace': {
    tests: 1,
    why: SCANNER,
  },
  'glossary/translate-validate-blocks.ts#describeBlock': {
    tests: 1,
    why: SENTINEL,
  },
  'translate-floor/translate-wire.ts#buildTranslateMessages': {
    tests: 6,
    why: 'blocks the package assembled for the sheet, written empty when it has none, and the archive text '
      + 'of the slice, empty where the archive holds nothing',
  },
  'translate-floor/translate-wire.ts#isPresent': {
    tests: 1,
    why: ASSEMBLED,
  },
  'translate-stage/translated-slate-criteria.ts#translatedSlateBreakEvidence': {
    tests: 1,
    why: ASSEMBLED,
  },
  'translate-stage/translated-slate-criteria.ts#translatedSlateCriteria': {
    tests: 1,
    why: ASSEMBLED,
  },
  'document-structure/translators-note.ts#cutAround': {
    tests: 3,
    why: SCANNER,
  },
  'document-structure/translators-note.ts#isWhitespace': {
    tests: 1,
    why: SCANNER,
  },
  'document-structure/unwrap-container.ts#UnpositionedContainerError': {
    tests: 1,
    why: PARSED,
  },
  'markup/word-bounds.ts#tokenStarts': {
    tests: 1,
    why: EMPTY_NEEDLE,
  },
  'markup/word-bounds.ts#wordStarts': {
    tests: 1,
    why: EMPTY_NEEDLE,
  },
  'translate-floor/wording-coherence.ts#assertWordingCoherent': {
    tests: 1,
    why: ARCHIVE_ABSENT,
  },
  'reference/work-title-lookup.ts#toLine': {
    tests: 1,
    why: TOOL,
  },
  'reference/work-title-lookup.ts#workTitleLookupLines': {
    tests: 1,
    why: SETTING,
  },
  'glossary/writer-grace-override.ts#readWriterGrace': {
    tests: 1,
    why: SETTING,
  },
  'run/corpus-run/repeat-census-lines.ts#lineRole': {
    tests: 1,
    why: PARSED,
  },
  'run/corpus-run/repeat-census-refrain.ts#censusClausesOf': {
    tests: 1,
    why: SCANNER,
  },
  'corpus-source/repeat-groups-wording.ts#cutEndAt': {
    tests: 1,
    why: SCANNER,
  },
  'corpus-source/repeat-groups-wording.ts#wordingStreamOf': {
    tests: 1,
    why: SCANNER,
  },
};

//endregion Listed tests

await describe({
  name: 'blank-text tests (ledgers B40 and B128)',
  children: [
    it({
      name: 'FINDS a comparison with the empty string written as a string or a template, either way round; the '
        + 'length of a trim call or of a name declared with one compared with anything; the length of a name '
        + 'annotated as a string, alone or in a union, compared with zero or asked whether it is less than one, either '
        + 'way round; and each tested for truth; LEAVES an array\'s length, an unannotated name\'s length, a trimmed '
        + 'length in arithmetic, two trims compared with each other, a string\'s length compared with another '
        + 'number, and a test file',
      fn: async () => {
        expect(blankTextTests({
          files: [
            {
              path: 'naps.ts',
              isTest: false,
              text: [
                'export function isQuiet(purr: string,): boolean {',
                '  return purr === \'\';',
                '}',
                'export function isLoud(meow: string,): boolean {',
                '  return `` !== meow;',
                '}',
                'export function naps(note: string,): boolean {',
                '  const trimmed = note.trim();',
                '  return (trimmed.length > 3) || (note.trimEnd().length === 0);',
                '}',
                'export function stirs({ note, tail, }: Readonly<{ readonly note: string; readonly tail: number; }>,): boolean {',
                '  if (note.trimStart())',
                '    return tail > 0;',
                '  return (!note) || (note.length > 0) || (0 === note.length);',
                '}',
                'export function wakes(notes: readonly string[],): string {',
                '  const first: string = notes[0] ?? \'asleep\';',
                '  return first ? first : (first.length === 0 ? \'quiet\' : \'awake\');',
                '}',
                'export class Basket {',
                '  holds(toy: string,): boolean {',
                '    return toy.trim() !== \'\';',
                '  }',
                '}',
                'export const stray = \'\' === [\'mouse\',].join(\'\',);',
                'export function counts(tails: readonly string[], { name, }: { readonly name: string[]; }, toy: string,): boolean {',
                '  const loose = toy.slice(1,);',
                '  return (tails.length === 0) || (name.length === 0) || (loose.length === 0) || (toy.length > 3);',
                '}',
                'export function pads(note: string,): number {',
                '  const trimmed = note.trim();',
                '  return note.length - trimmed.length;',
                '}',
                'export function same(left: string, right: string,): boolean {',
                '  return left.trim() === right.trim();',
                '}',
                'export function sleeps(purr: string, yawn: string | undefined,): boolean {',
                '  return (purr.length < 1) || (1 <= purr.length) || (purr.length > 1) || (!yawn);',
                '}',
              ].join('\n',),
            },
            {
              path: 'naps.unit.test.ts',
              isTest: true,
              text: 'export const quiet = (purr: string,): boolean => purr === \'\';',
            },
          ],
        },),).toEqual({
          'naps.ts#<module>': 1,
          'naps.ts#Basket': 1,
          'naps.ts#isLoud': 1,
          'naps.ts#isQuiet': 1,
          'naps.ts#naps': 2,
          'naps.ts#sleeps': 3,
          'naps.ts#stirs': 4,
          'naps.ts#wakes': 2,
        },);
      },
    },),
    it({
      name: 'FINDS NO BLANKNESS TEST across the package but those named with why each asks another question',
      fn: async () => {
        /**
         Count per key the named tests allow.
         */
        const allowed = Object.fromEntries(Object.entries(HELD_TESTS,)
          .map(function toCount([
            key,
            held,
          ],) {
            return [
              key,
              held.tests,
            ];
          },),);
        expectRecordAsListed({
          found: blankTextTests({ files: await readPackageSource(), },),
          listed: allowed,
        },);
      },
    },),
  ],
},);
