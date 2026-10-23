/**
 Guards against a map entry written over whatever the key already held,
 without the key being read first (ledger B203 to B210 and the census that
 followed them). A `Map` filled by `set` keeps the later value for a key
 written twice, and nothing says so: a repeated claim id, slice index or
 model id answered for the later element while the earlier one vanished. A
 write the function reads the key before (`get` or `has` on the same map with
 the same key text, to merge, count, refuse or skip a repeat) says what a
 repeat means; a write with no such read has to say, in this file, what makes
 its key unique.

 WHAT THE SCAN READS, in the package's source; tests and test fixtures are not
 read. A call of a method named `set` with two arguments, whose enclosing
 functions, any of them, never hold `<receiver>.get(<key` or
 `<receiver>.has(<key` with the receiver and key written as the call writes
 them, whitespace aside. A write found by it is named by its file, its
 innermost named function, the receiver and the key text, so a line moving
 does not move it. Out of the scan's reach: a read of the key through another
 name or another function, which passes as a read though it may not be one,
 and a key repeated by a loop over data that holds the read in a callee;
 review has to catch those.

 A TYPED ARRAY'S SET IS NO MAP WRITE: `set(source, offset)` copies a source
 into the array from an offset and has no key, and the credential mask's
 copies of offsets were written as loops while the scan read them as map
 writes. The scan tells the two apart by what the receiver is: a name whose
 nearest declaration, in the scopes holding the write, makes it with a typed
 array's constructor, `from` or `of`, or annotates it as one (a destructured
 parameter by its member's type), is a typed array. A receiver the scan
 cannot follow (a member of another value, a name a pattern binds without its
 type, a name no scope of the file declares) stays a possible map write, the
 side a scan for unread keys can afford.

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

import { readPackageSource, } from './family-source.test-fixture.ts';
import {
  ancestorsOf,
  childNodes,
  identifierName,
  isTreeNode,
  memberName,
  nodesUnder,
  parentsOf,
  parseSource,
  type SourceText,
  type TreeNode,
  unwrapped,
} from './source-scan.test-fixture.ts';
import { expectNoFindings, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scan-findings.test-fixture.ts';

/**
 Node kinds that open a function.
 */
const FUNCTION_KINDS: ReadonlySet<string> = new Set(['ArrowFunctionExpression', 'FunctionDeclaration', 'FunctionExpression',],);

/**
 Constructors of the typed arrays, whose `set(source, offset)` copies a source
 into the array from an offset and writes no key.
 */
const TYPED_ARRAY_NAMES: ReadonlySet<string> = new Set([
  'BigInt64Array',
  'BigUint64Array',
  'Float16Array',
  'Float32Array',
  'Float64Array',
  'Int16Array',
  'Int32Array',
  'Int8Array',
  'Uint16Array',
  'Uint32Array',
  'Uint8Array',
  'Uint8ClampedArray',
],);

/**
 Static methods of a typed array's constructor that make a new array of it.
 */
const TYPED_ARRAY_MAKERS: ReadonlySet<string> = new Set([
  'from',
  'of',
],);

/**
 Node kinds whose `body` lists the statements of a block scope.
 */
const BLOCK_KINDS: ReadonlySet<string> = new Set([
  'BlockStatement',
  'Program',
  'StaticBlock',
],);

/**
 Node kinds that declare a name of their own beside the variables a block
 declares.
 */
const NAMED_DECLARATION_KINDS: ReadonlySet<string> = new Set([
  'ClassDeclaration',
  'FunctionDeclaration',
  'TSEnumDeclaration',
],);

/**
 The field of each loop kind that holds the declaration of its head.
 */
const LOOP_HEADS: ReadonlyMap<string, string> = new Map([
  [
    'ForInStatement',
    'left',
  ],
  [
    'ForOfStatement',
    'left',
  ],
  [
    'ForStatement',
    'init',
  ],
],);

/**
 What the declarations of one scope make of a name: a typed array, another
 value, or nothing, where an outer scope decides.
 */
type Binding = 'typed-array' | 'other value' | 'undeclared';

/**
 Whether a type annotation names a typed array.

 @param annotation - a node's `typeAnnotation` field, of any kind or none

 @returns Whether it annotates a typed array's constructor name, with or
 without type arguments

 @example
 ```ts
 const typed = annotatesTypedArray({ annotation: identifier.typeAnnotation, },);
 ```
 */
function annotatesTypedArray({ annotation, }: { readonly annotation: unknown; },): boolean {
  if ((!isTreeNode(annotation,)) || (annotation.type !== 'TSTypeAnnotation'))
    return false;
  /**
   The annotated type.
   */
  const { typeAnnotation: annotated, } = annotation;
  return isTreeNode(annotated,)
    && (annotated.type === 'TSTypeReference')
    && TYPED_ARRAY_NAMES.has(identifierName({ node: annotated.typeName, },),);
}

/**
 Whether an expression makes a typed array: its constructor called with `new`,
 or its `from` or `of`.

 @param expression - an initializer or a default, of any kind or none

 @returns Whether the expression, its type assertions aside, makes one

 @example
 ```ts
 const typed = makesTypedArray({ expression: declarator.init, },);
 ```
 */
function makesTypedArray({ expression, }: { readonly expression: unknown; },): boolean {
  /**
   The expression inside any wrapper.
   */
  const { inner, } = unwrapped({ node: expression, },);
  if (!isTreeNode(inner,))
    return false;
  if (inner.type === 'NewExpression')
    return TYPED_ARRAY_NAMES.has(identifierName({ node: inner.callee, },),);
  if (inner.type !== 'CallExpression')
    return false;
  /**
   The called function.
   */
  const { callee, } = inner;
  return isTreeNode(callee,)
    && (callee.type === 'MemberExpression')
    && TYPED_ARRAY_NAMES.has(identifierName({ node: callee.object, },),)
    && TYPED_ARRAY_MAKERS.has(memberName({ node: callee, },),);
}

/**
 What a name is to a declaration that binds it: a typed array where its type
 or its value says so, another value otherwise.

 @param annotation - the binding's own type annotation, of any kind or none

 @param value - the value it starts with, of any kind or none

 @returns The binding

 @example
 ```ts
 const binding = boundAs({ annotation: id.typeAnnotation, value: declarator.init, },);
 ```
 */
function boundAs({ annotation, value, }: { readonly annotation: unknown; readonly value: unknown; },): Binding {
  return (annotatesTypedArray({ annotation, },) || makesTypedArray({ expression: value, },))
    ? 'typed-array'
    : 'other value';
}

/**
 Whether a pattern mentions a name anywhere in it, keys and defaults included,
 so a name it only mentions reads as another value and keeps its write found.

 @param pattern - a binding pattern

 @param name - name looked for

 @returns Whether an identifier under the pattern carries the name

 @example
 ```ts
 const mentioned = patternMentions({ pattern: declarator.id, name: 'beds', },);
 ```
 */
function patternMentions({ pattern, name, }: { readonly pattern: TreeNode; readonly name: string; },): boolean {
  return nodesUnder({ root: pattern, },).some(function carriesName(node,): boolean {
    return identifierName({ node, },) === name;
  },);
}

/**
 The type annotation a destructured parameter's type literal gives one of its
 members.

 @param pattern - an object pattern annotated with a type literal

 @param key - the member's name

 @returns The member's annotation, or nothing where the pattern's type is no
 literal or names no such member

 @example
 ```ts
 const annotation = memberAnnotation({ pattern, key: 'bytes', },);
 ```
 */
function memberAnnotation({ pattern, key, }: { readonly pattern: TreeNode; readonly key: string; },): unknown {
  /**
   The pattern's own annotation.
   */
  const { typeAnnotation: annotation, } = pattern;
  if ((!isTreeNode(annotation,)) || (!isTreeNode(annotation.typeAnnotation,)))
    return undefined;
  /**
   The members of the annotated type, none for a type that is no literal.
   */
  const { members, } = annotation.typeAnnotation;
  return (Array.isArray(members,) ? members : [])
    .filter(function isNode(member: unknown,): member is TreeNode {
      return isTreeNode(member,);
    },)
    .find(function namesKey(member,): boolean {
      return (member.type === 'TSPropertySignature') && (identifierName({ node: member.key, },) === key);
    },)
    ?.typeAnnotation;
}

/**
 What one parameter of a function makes of a name.

 @param parameter - the parameter, plain, defaulted or destructured

 @param name - name looked for

 @returns Its binding of the name, `undeclared` where it binds none

 @example
 ```ts
 const binding = parameterBinding({ parameter: fn.params[0], name: 'bytes', },);
 ```
 */
function parameterBinding({ parameter, name, }: { readonly parameter: TreeNode; readonly name: string; },): Binding {
  if (identifierName({ node: parameter, },) === name) {
    return boundAs({
      annotation: parameter.typeAnnotation,
      value: undefined,
    },);
  }
  if ((parameter.type === 'AssignmentPattern') && isTreeNode(parameter.left,)) {
    if (identifierName({ node: parameter.left, },) === name) {
      return boundAs({
        annotation: parameter.left.typeAnnotation,
        value: parameter.right,
      },);
    }
    return parameterBinding({
      parameter: parameter.left,
      name,
    },);
  }
  if (parameter.type === 'ObjectPattern') {
    for (const property of (parameter.properties as readonly TreeNode[])) {
      /**
       What the property binds, its default aside.
       */
      const target = (isTreeNode(property.value,) && (property.value.type === 'AssignmentPattern'))
        ? property.value.left
        : property.value;
      if ((property.type === 'Property') && (property.computed !== true) && (identifierName({ node: target, },) === name)) {
        return boundAs({
          annotation: memberAnnotation({
            pattern: parameter,
            key: identifierName({ node: property.key, },),
          },),
          value: (isTreeNode(property.value,) && (property.value.type === 'AssignmentPattern'))
            ? property.value.right
            : undefined,
        },);
      }
    }
  }
  return patternMentions({
    pattern: parameter,
    name,
  },)
    ? 'other value'
    : 'undeclared';
}

/**
 What one statement of a block declares of a name.

 @param statement - a statement of the block's body, an export around a
 declaration read as the declaration

 @param name - name looked for

 @returns Its binding of the name, `undeclared` where it declares none

 @example
 ```ts
 const binding = statementBinding({ statement: program.body[0], name: 'rugs', },);
 ```
 */
function statementBinding({ statement, name, }: { readonly statement: TreeNode; readonly name: string; },): Binding {
  /**
   The declaration the statement holds, an export unwrapped.
   */
  const declared = statement.type.startsWith('Export',) ? statement.declaration : statement;
  if (!isTreeNode(declared,))
    return 'undeclared';
  if (NAMED_DECLARATION_KINDS.has(declared.type,))
    return (identifierName({ node: declared.id, },) === name) ? 'other value' : 'undeclared';
  if (declared.type !== 'VariableDeclaration')
    return 'undeclared';
  for (const declarator of (declared.declarations as readonly TreeNode[])) {
    if (!isTreeNode(declarator.id,))
      continue;
    if (identifierName({ node: declarator.id, },) === name) {
      return boundAs({
        annotation: declarator.id.typeAnnotation,
        value: declarator.init,
      },);
    }
    if (patternMentions({
      pattern: declarator.id,
      name,
    },))
      return 'other value';
  }
  return 'undeclared';
}

/**
 What one scope's own declarations make of a name: a function's parameters, a
 block's statements, a loop's head or a catch's parameter.

 @param scope - a node holding the write, of any kind

 @param name - name looked for

 @returns Its binding of the name, `undeclared` for a node that declares none
 of it or declares nothing

 @example
 ```ts
 const binding = scopeBinding({ scope: ancestor, name: 'starts', },);
 ```
 */
function scopeBinding({ scope, name, }: { readonly scope: TreeNode; readonly name: string; },): Binding {
  if (FUNCTION_KINDS.has(scope.type,)) {
    for (const parameter of (scope.params as readonly TreeNode[])) {
      /**
       What this parameter binds of the name.
       */
      const binding = parameterBinding({
        parameter,
        name,
      },);
      if (binding !== 'undeclared')
        return binding;
    }
    return (identifierName({ node: scope.id, },) === name) ? 'other value' : 'undeclared';
  }
  if (BLOCK_KINDS.has(scope.type,)) {
    for (const statement of (scope.body as readonly TreeNode[])) {
      /**
       What this statement declares of the name.
       */
      const binding = statementBinding({
        statement,
        name,
      },);
      if (binding !== 'undeclared')
        return binding;
    }
    return 'undeclared';
  }
  if ((scope.type === 'CatchClause') && isTreeNode(scope.param,)) {
    return patternMentions({
      pattern: scope.param,
      name,
    },)
      ? 'other value'
      : 'undeclared';
  }
  /**
   A loop head's declaration, absent for any node that is no loop.
   */
  const head = LOOP_HEADS.get(scope.type,);
  if ((head === undefined) || (!isTreeNode(scope[head],)))
    return 'undeclared';
  return statementBinding({
    statement: scope[head],
    name,
  },);
}

/**
 Whether a write's receiver is a typed array: a name whose nearest
 declaration, in the scopes holding the write, makes or annotates one.

 @param receiver - the object the write's `set` is called on

 @param parents - each node's parent in the write's file

 @returns True only for a name so declared; a member, or a name the file
 declares nowhere or without its type, reads as no typed array

 @example
 ```ts
 const copies = receiverIsTypedArray({ receiver: callee.object, parents, },);
 ```
 */
function receiverIsTypedArray(
  {
    receiver,
    parents,
  }: {
    readonly receiver: TreeNode;
    readonly parents: ReadonlyMap<TreeNode, TreeNode>;
  },
): boolean {
  /**
   The receiver's name, empty for anything but a plain name.
   */
  const name = identifierName({ node: receiver, },);
  if (name === '')
    return false;
  for (const scope of ancestorsOf({
    node: receiver,
    parents,
  },)) {
    /**
     What this scope's own declarations make of the name.
     */
    const binding = scopeBinding({
      scope,
      name,
    },);
    if (binding !== 'undeclared')
      return binding === 'typed-array';
  }
  return false;
}

/**
 Writes the package makes over a key it never reads, each with what makes the
 key unique at that write. The id is the file, the innermost named function,
 the receiver and the key text; one entry covers every write with that id.
 */
const ALLOWED_BLIND_SETS: ReadonlyMap<string, string> = new Map<string, string>([
  [
    'shared-contract/active-footnote-markers.ts#activeFootnoteMarkers: runReferences.set(run.opener,...)',
    'the key is the tree node that opens a run, and each run of unpositioned text has its own first member',
  ],
  [
    'consistency/consistency-folded-links.ts#foldedLinksOf: emptyStarts.set(last,...)',
    'the key is the folded unit of the last character of a link the parse positions, and no two links end at the same character '
      + 'of the English, since a link ends where its own closing bracket stands',
  ],
  [
    'consistency/consistency-folded-links.ts#foldedLinksOf: emptyEnds.set(first,...)',
    'the key is the folded unit of the first character of a link the parse positions, and no two links start at the same character '
      + 'of the English, since a link starts where its own opening bracket stands',
  ],
  [
    'consistency/consistency-folded-links.ts#foldedLinksOf: closingSpans.set(first,...)',
    'the key is the folded unit of the first character of a link the parse positions, and no two links start at the same character '
      + 'of the English, since a link starts where its own opening bracket stands',
  ],
  [
    'consistency/consistency-link-twin.ts#settlePending: removedBefore.set(units.length,...)',
    'the key is the number of units the twin holds when the links taken out are settled, which grows with the next unit written, so '
      + 'no two settlements share a key',
  ],
  [
    'consistency/consistency-page-english.ts#pageOfSlices: offsets.set(slice.sliceIndex,...)',
    'one write per slice of the list the pass builds from the prepared slices, whose indexes `assertSliceIndexing` holds distinct',
  ],
  [
    'consistency/consistency-page-english.ts#laidOutPage: byIndex.set(entry.sliceIndex,...)',
    'one write per place the assembly reports, and it reports each replacement once, since a slice holds at most one '
      + 'replacement and `assertSliceIndexing` holds the indexes of the list distinct',
  ],
  [
    'consistency/consistency-page-english.ts#laidOutPage: offsets.set(sliceIndex,...)',
    'one write per slice of the list the pass builds from the prepared slices, whose indexes `assertSliceIndexing` holds distinct',
  ],
  [
    'consistency/consistency-page-english.ts#laidOutPage: shipped.set(sliceIndex,...)',
    'one write per replacement of the list the assembly was just given, and the assembly refuses two replacements naming one '
      + 'slice before this write runs',
  ],
  [
    'consistency/consistency-page-english.ts#pageOfSlices: shipped.set(slice.sliceIndex,...)',
    'one write per slice of the list the pass builds from the prepared slices, whose indexes `assertSliceIndexing` holds distinct',
  ],
  [
    'corpus-pass/corpus-run/pass-consistency-slices.ts#consistencySlicesOf: raw.set(sliceIndex,...)',
    'one write per slice of the list the pass reads from the prepared slices, whose indexes `assertSliceIndexing` holds distinct',
  ],
  [
    'house-rule/contributor-name-authority.ts#referenceLinkLabels: labels.set(markup,...)',
    'the label is cut from inside the markup the key is, so two links written alike carry one value',
  ],
  [
    'artifact-pool/corpus-run/artifact-eligible.ts#verdictsByTip: verdicts.set(tip,...)',
    'the tips come from a set of the census\'s commits, each asked once',
  ],
  [
    'artifact-pool/corpus-run/artifact-generation.ts#censusByGeneration: tipByEntry.set(entryId,...)',
    'an entry id is the stem of an artifact file name, and a directory lists a name once',
  ],
  [
    'page-pass/corpus-run/assembly-page-text.ts#pageTextBySlice: text.set(replacement.sliceIndex,...)',
    'a replacement supersedes the archive\'s text for its slice on purpose, and `spliceSlices` refuses two replacements naming one slice before any page pass reads the list',
  ],
  [
    'glossary/corpus-run/attempt-store.ts#countAttempt: attempts.set(id,...)',
    'the count is read through `attemptsOf` and written back plus one',
  ],
  [
    'run/corpus-run/command-line.ts#optionsOf: options.set(name,...)',
    'the names are the spec\'s own, and a measurement over every command found none declaring a flag in two groups (42 commands, 31 flags)',
  ],
  [
    'run/corpus-run/command-line.ts#readCommandLine: values.set(reading.name,...)',
    'a flag written twice is counted in `timesWritten` and refused by `countRefusals` unless it is declared repeatable',
  ],
  [
    'page-pass/corpus-run/contributor-name-restore.ts#restoreContributorNames: rewritten.set(sliceIndex,...)',
    'one write per slice of a loop over the prepared slices, whose indexes `assertSliceIndexing` holds distinct',
  ],
  [
    'coverage-census/corpus-run/coverage-tally.ts#cut: pieces.set(bundle,...)',
    'the loop reads a map keyed by bundle, and `cut` returns at once when it has run',
  ],
  [
    'coverage-census/corpus-run/coverage-tally.ts#paint: sites.set(key,...)',
    'the key names the bundle, the range and the function, so a repeat is the same site with the same value',
  ],
  [
    'slice-cache/corpus-run/entry-pictures.ts#gather: gathered.set(assetName,...)',
    'the names come from a set of the document\'s pictures, each read once',
  ],
  [
    'insertion/corpus-run/insertion-container-blocks.ts#countSide: byPosition.set(position,...)',
    'the position is the loop\'s own place among the prepared slices',
  ],
  [
    'insertion/corpus-run/insertion-container-deficit.ts#admitContainerDeficit: admitted.set(position,...)',
    'a row inside two nested carried containers is admitted by each, spends each one\'s deficit because it fills both, and keeps the later container\'s finding, one finding per passage as the TSDoc says',
  ],
  [
    'insertion/corpus-run/insertion-container-deficit.ts#spendDeficit: admitted.set(row.position,...)',
    'one row per position among the rows still unresolved, each visited once',
  ],
  [
    'page-check/corpus-run/jsx-attribute-restore.ts#restoreJsxAttributes: rewritten.set(sliceIndex,...)',
    'one write per slice of a loop over the prepared slices, whose indexes `assertSliceIndexing` holds distinct',
  ],
  [
    'page-check/corpus-run/list-spread-restore.ts#restoreListSpread: rewritten.set(sliceIndex,...)',
    'one write per slice of a loop over the prepared slices, whose indexes `assertSliceIndexing` holds distinct',
  ],
  [
    'corpus-pass/corpus-run/pass-consistency-apply-gains.ts#gainedBy: sides.set(edit.id,...)',
    'the id names a group and a place, a place belongs to exactly one replacement, and a slice lists each replacement '
      + 'once, so each id is written once',
  ],
  [
    'corpus-pass/corpus-run/pass-consistency-apply.ts#recordGaveWay: gaveWayTo.set(key,...)',
    'the key names a group and a place, a place belongs to exactly one replacement, a replacement lies in one slice and '
      + 'the loop visits each slice once, and a replacement is either left out for changing another group\'s words or '
      + 'overlapped by an earlier one and never both, so each key is written once',
  ],
  [
    'corpus-pass/corpus-run/pass-consistency-apply.ts#recordKept: keptBecause.set(key,...)',
    'the key names a group and a place, a place belongs to exactly one replacement, a replacement lies in one slice and '
      + 'the loop visits each slice once, and a replacement is left out once and given one reason, so each key is written once',
  ],
  [
    'corpus-pass/corpus-run/pass-consistency-apply.ts#recordOutcome: outcomes.set(key,...)',
    'the key names a group and a place, a place belongs to exactly one replacement, a replacement lies in one slice and '
      + 'the loop visits each slice once, and a replacement is either left out or applied and never both, so each key is '
      + 'written once',
  ],
  [
    'corpus-pass/corpus-run/pass-seated-pictures.ts#readPictures: priorReadings.set(name,...)',
    'a later reading of a picture supersedes the retained one on purpose, since it was read with the earlier as evidence',
  ],
  [
    'run/corpus-run/run-end-report.ts#registerRunEndReport: registered.set(key,...)',
    'a report registered again under its key replaces the earlier one on purpose, so a run that builds its client twice '
      + 'reports once, from the client that is current',
  ],
  [
    'glossary/corpus-run/prose-ranges.ts#inlineCodeSpans: spans.set(bodyOffset+nonNullishOrThrow(node.position?.star,...)',
    'two nodes of one tree never open at one offset',
  ],
  [
    'probe/corpus-run/rendering-audit-settled-repeat.ts#rowsBySubject: byKey.set(key,...)',
    'a row naming a subject an earlier row named is refused before this write, by the read of `firstAt` under the same key, a second map this scan does not follow',
  ],
  [
    'slice-cache/corpus-run/slice-cache-namespace.ts#loadNamespacedSlices: resumed.set(key,...)',
    'the key is the file name without its prefix and suffix, which no two names of one directory share, and the file\'s own `cacheKey` must equal it',
  ],
  [
    'translate-lane/document-readings.ts#readDocumentPictures: readings.set(assetName,...)',
    'the names come from a set of the document\'s pictures, each visited once',
  ],
  [
    'document-structure/footnote-graph.ts#collectBlockHits: runs.set(opened.opener,...)',
    'the key is the tree node that opens a run, and each run of unpositioned text has its own first member',
  ],
  [
    'document-structure/footnote-unpositioned-runs.ts#zoneByRawOffset: byOffset.set(at,...)',
    'the offset advances through the scan, and each occurrence is paired once',
  ],
  [
    'adjudication/introduced-defect-screen.ts#resolveProberChecks: checks.set(modelId,...)',
    'the model ids are the own keys of one record, which holds each once',
  ],
  [
    'slice-reading/pairing-pictures.ts#pairingPictureContext: reduced.set(assetName,...)',
    'the loop reads a map keyed by picture name',
  ],
  [
    'archive/prepare-with-pairing.ts#prepareDocumentPairWithRoster: blockPairings.set(pairIndex,...)',
    'the index is the loop\'s own place among the aligned section pairs',
  ],
  [
    'rendering-audit/rendering-audit-corroborate.ts#keepDistinct: groups.set(JSON.stringify(textsInCodePointOrder({texts:grou,...)',
    'one group per distinct membership on purpose, and two groups of one membership hold the same voices and defects',
  ],
  [
    'stage/stage-quorum.ts#collectRounds: unreadableSeats.set(outcome.modelId,...)',
    'a later round\'s unreadable answer of a seat replaces the earlier on purpose, and a seat that then answers readably is deleted',
  ],
  [
    'stage/stage-round.ts#askOnce: arrived.set(position,...)',
    'each position is asked once, and the abandoned outcome is written only where the answer did not arrive',
  ],
],);

/**
 Characters of a key's text an id keeps, so a key written as a long expression
 names its write without carrying the expression.
 */
const KEY_TEXT_LENGTH = 48;

/**
 Characters the scan reads as whitespace, which the source's formatter writes
 between tokens.
 */
const WHITESPACE: ReadonlySet<string> = new Set([' ', '\n', '\t', '\r',],);

/**
 Text of a node with its whitespace removed.

 @param file - file the node stands in

 @param node - node read

 @returns The node's text, joined

 @example
 ```ts
 const key = compactText({ file, node: call.arguments[0], },); // 'cat.id'
 ```
 */
function compactText({ file, node, }: { readonly file: SourceText; readonly node: TreeNode; },): string {
  /**
   Characters kept, in order.
   */
  const kept: string[] = [];
  for (const character of file.text.slice(
    node.start,
    node.end,
  )) {
    if (!WHITESPACE.has(character,))
      kept.push(character,);
  }
  return kept.join('',);
}

/**
 Writes a file makes over a key its function never reads.

 @param file - source file read

 @returns One id per such write, without repeats

 @example
 ```ts
 const ids = blindWrites({ file, },);
 ```
 */
function blindWrites({ file, }: { readonly file: SourceText; },): readonly string[] {
  /**
   Ids found so far.
   */
  const found = new Set<string>();
  /**
   The file's program.
   */
  const { program, } = parseSource({ file, },);
  /**
   Each node's parent, so a receiver is read through the scopes holding it.
   */
  const parents = parentsOf({ program, },);
  /**
   Nodes still to visit, each with the functions that hold it, outermost
   first.
   */
  const pending: { readonly node: TreeNode; readonly holders: readonly TreeNode[]; }[] = [{
    node: program,
    holders: [],
  },];
  while (pending.length > 0) {
    /**
     Node visited now.
     */
    const current = pending.pop() as (typeof pending)[number];
    /**
     Functions holding the node's children.
     */
    const holders = FUNCTION_KINDS.has(current.node.type,)
      ? [...current.holders, current.node,]
      : current.holders;
    pending.push(...childNodes({ node: current.node, },).map(function withHolders(child,) {
      return {
        node: child,
        holders,
      };
    },),);
    /**
     The call's callee, when the node is a call.
     */
    const { callee, } = current.node;
    if ((current.node.type !== 'CallExpression') || (!isTreeNode(callee,)) || (callee.type !== 'MemberExpression'))
      continue;
    /**
     The call's arguments.
     */
    const args = current.node.arguments as readonly TreeNode[];
    if ((memberName({ node: callee, },) !== 'set') || (args.length !== 2) || (!isTreeNode(callee.object,)))
      continue;
    if (receiverIsTypedArray({
      receiver: callee.object,
      parents,
    },))
      continue;
    /**
     The map written and the key written, as text.
     */
    const receiver = compactText({
      file,
      node: callee.object,
    },);
    const key = compactText({
      file,
      node: nonNullishArgument({ args, },),
    },);
    /**
     Whether any function holding the write reads the key of the same map.
     */
    const read = holders.some(function readsKey(holder,): boolean {
      /**
       The holder's text, joined.
       */
      const text = compactText({
        file,
        node: holder,
      },);
      return text.includes(`${receiver}.get(${key}`,) || text.includes(`${receiver}.has(${key}`,);
    },);
    if (read)
      continue;
    /**
     Innermost holder that names itself.
     */
    const named = holders.findLast(function hasName(holder,): boolean {
      return identifierName({ node: holder.id, },) !== '';
    },);
    found.add(`${file.path}#${(named === undefined) ? '<module>' : identifierName({ node: named.id, },)}: ${receiver}.set(${key.slice(
      0,
      KEY_TEXT_LENGTH,
    )},...)`,);
  }
  return [...found,].toSorted();
}

/**
 First argument of a two-argument call, which the caller has counted.

 @param args - the call's arguments

 @returns The key argument

 @throws {@link Error} when the call has no argument, which the caller's count excludes

 @example
 ```ts
 const key = nonNullishArgument({ args: call.arguments, },);
 ```
 */
function nonNullishArgument({ args, }: { readonly args: readonly TreeNode[]; },): TreeNode {
  /**
   The first argument.
   */
  const [first,] = args;
  if (first === undefined)
    throw new Error('unreachable: a call counted to hold two arguments holds none',);
  return first;
}

/**
 Writes the package's source makes over a key it never reads.

 @param files - files read, tests among them to be skipped

 @returns Ids of the writes no allowance names, then allowances no write uses

 @example
 ```ts
 const loose = unnamedBlindWrites({ files, },);
 ```
 */
function unnamedBlindWrites({ files, }: { readonly files: readonly SourceText[]; },): readonly string[] {
  /**
   Every blind write in the source, by id.
   */
  const written = files
    .filter(function isSource(file,): boolean {
      return !file.isTest;
    },)
    .flatMap(function blindIn(file,): readonly string[] {
      return blindWrites({ file, },);
    },);
  return [
    ...written
      .filter(function unnamed(id,): boolean {
        return !ALLOWED_BLIND_SETS.has(id,);
      },)
      .map(function unreadWrite(id,): string {
        return `${id}: no allowance says what makes the key unique`;
      },),
    ...[...ALLOWED_BLIND_SETS.keys(),]
      .filter(function stale(id,): boolean {
        return !written.includes(id,);
      },)
      .map(function staleAllowance(id,): string {
        return `${id}: allowed, and no such write stands`;
      },),
  ].toSorted();
}

await describe({
  name: 'map writes over an unread key (ledger B203 to B210)',
  children: [
    it({
      name: 'FINDS a write over a key its function never reads, by file, function, map and key, and leaves a write '
        + 'after a get or a has of the same key, a write read by an outer function, a one-argument set and a '
        + 'bare function named set',
      fn: async () => {
        expect(blindWrites({
          file: {
            path: 'cat.ts',
            text: [
              'export function nap(beds: Map<string, number>, name: string): void { beds.set(name, 1); }',
              'export function purr(beds: Map<string, number>, name: string): void {',
              '  beds.set(name, (beds.get(name) ?? 0) + 1);',
              '}',
              'export function knead(beds: Map<string, number>, name: string): void {',
              '  if (beds.has(name)) return;',
              '  beds.set(name, 1);',
              '}',
              'export function groom(beds: Map<string, number>, names: string[]): void {',
              '  if (beds.has(names[0])) return;',
              '  names.forEach(function each(name: string): void { beds.set(names[0], 1); });',
              '}',
              'export function stretch(beds: Map<string, number>, names: string[]): void {',
              '  names.forEach(function each(name: string): void { beds.set(name, 2); });',
              '}',
              'export function yawn(rugs: Headers, name: string): void { rugs.set(name); }',
              'export function pounce(): void { sets(\'a\', 1); }',
            ].join('\n',),
            isTest: false,
          },
        },),).toEqual([
          'cat.ts#each: beds.set(name,...)',
          'cat.ts#nap: beds.set(name,...)',
        ],);
      },
    },),
    it({
      name: 'LEAVES A TYPED ARRAY\'S SET, which copies a source into the array at an offset and writes no key, where '
        + 'the receiver is declared as one: made by a constructor or by `from`, annotated, or a parameter so typed, '
        + 'plain or destructured',
      fn: async () => {
        expect(blindWrites({
          file: {
            path: 'cat.ts',
            text: [
              'export function copy(source: Int32Array, offset: number): Int32Array {',
              '  const starts = new Int32Array(8);',
              '  starts.set(source, offset);',
              '  return starts;',
              '}',
              'export function fill(source: number[]): void {',
              '  const ends = Int32Array.from(source);',
              '  ends.set(source, 1);',
              '  const marks: Uint8Array = make();',
              '  marks.set(source, 0);',
              '}',
              'export function paste(bytes: Float64Array, source: Float64Array): void { bytes.set(source, 2); }',
              'export function splice({ bytes, source, }: { readonly bytes: Uint16Array; readonly source: Uint16Array; }): void {',
              '  bytes.set(source, 3);',
              '}',
            ].join('\n',),
            isTest: false,
          },
        },),).toEqual([],);
      },
    },),
    it({
      name: 'FINDS A MAP\'S SET WHERE A TYPED ARRAY OF THE SAME NAME stands in another function or an outer scope, '
        + 'since the receiver is read through the binding the write names, and FINDS a set on a member it cannot follow',
      fn: async () => {
        expect(blindWrites({
          file: {
            path: 'cat.ts',
            text: [
              'const rugs = new Uint8Array(4);',
              'export function copy(): Int32Array { const beds = new Int32Array(8); return beds; }',
              'export function nap(name: string): void { const beds = new Map<string, number>(); beds.set(name, 1); }',
              'export function purr(rugs: Map<string, number>, name: string): void { rugs.set(name, 1); }',
              'export function knead(name: string): void {',
              '  const mats = new Uint8Array(2);',
              '  { const mats = new Map<string, number>(); mats.set(name, 1); }',
              '}',
              'export function groom({ beds, name, }: { readonly beds: Map<string, number>; readonly name: string; }): void {',
              '  beds.set(name, 1);',
              '}',
              'export function stretch(view: { readonly starts: Int32Array; }, source: Int32Array): void {',
              '  view.starts.set(source, 0);',
              '}',
            ].join('\n',),
            isTest: false,
          },
        },),).toEqual([
          'cat.ts#groom: beds.set(name,...)',
          'cat.ts#knead: mats.set(name,...)',
          'cat.ts#nap: beds.set(name,...)',
          'cat.ts#purr: rugs.set(name,...)',
          'cat.ts#stretch: view.starts.set(source,...)',
        ],);
      },
    },),
    it({
      name: 'FINDS NO MAP WRITE OVER AN UNREAD KEY across the package that no allowance explains, and no allowance '
        + 'for a write that is gone',
      fn: async () => {
        /**
         Every package file, tests among them to be skipped.
         */
        const files = await readPackageSource();
        expect(files.some(function isSource(file,): boolean {
          return !file.isTest;
        },),).toBe(true,);
        expectNoFindings({ findings: unnamedBlindWrites({ files, },), },);
      },
    },),
  ],
},);
