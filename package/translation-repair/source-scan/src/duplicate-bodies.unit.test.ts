/**
 Guards the copies family (audit area six, ledger B1 to B19; the prevention
 doc's "Copies of shared code"): no two functions in the package's source,
 its tests or its test fixtures may keep one body, in two files or in one,
 except the frozen copies an artifact version recomputes and refuses to
 disagree with (ledger B15), each listed here with the reason it stays.

 THE FIXTURES COME FIRST, so the package-wide case is read against a scan
 shown able to find each kind of copy (ledger M21: a check that could not
 fail). Fixtures are cat-themed; the package case reads this package's own
 source, tests included (ledger B116).

 A body is compared as its source text with every comment cut out and every
 whitespace character of its code dropped, so two copies differing only in
 layout or comments are one body; a string or template literal is compared
 as written, since its spaces are its value (ledger B114). Bodies shorter
 than `SHORTEST_COMPARED` such characters are left out, as a one-line
 accessor is no drift risk.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { compareCodePoints, } from '@monochromatic-dev/translation-repair-markup';
import { readPackageSource, } from './family-source.test-fixture.ts';
import {
  childNodes,
  isTreeNode,
  nodesUnder,
  parseSource,
  type SourceText,
  type TreeNode,
} from './source-scan.test-fixture.ts';
import { expectFindingsAsListed, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scan-findings.test-fixture.ts';

/**
 Shortest body, in normalized characters, the scan compares; the census the
 audit ran (`duplicate-bodies.mjs`) used the same floor.
 */
const SHORTEST_COMPARED = 80;

/**
 Frozen copies kept on purpose, each group as its sorted locations, with the
 reason it stays. A location is `path#name`.
 */
const FROZEN_COPIES: readonly {
  readonly locations: readonly string[];
  readonly reason: string;
}[] = [
  {
    locations: [
      'artifact-pool/corpus-run/artifact-two-lane-comparison.ts#judgeTwoLaneSlice',
      'lane-contest/lane-comparison.ts#judgeSlice',
    ],
    reason: 'the two-lane artifact reader recomputes each slice verdict with the rule its version was written '
      + 'under and refuses a file that disagrees (ledger B15)',
  },
  {
    locations: [
      'archive/absolute-naturalness-review-stage.ts#uniqueFindings',
      'artifact/corpus-run/artifact-two-lane-read-naturalness-seat.ts#uniqueNaturalnessFindings',
    ],
    reason: 'the naturalness seat reader recomputes its version\'s findings list and refuses disagreement (B15)',
  },
  {
    locations: [
      'archive/absolute-naturalness-review-stage.ts#firstOccurrence',
      'artifact/corpus-run/artifact-two-lane-read-naturalness-seat.ts#firstOccurrence',
    ],
    reason: 'nested inside the frozen copy of uniqueFindings',
  },
  {
    locations: [
      'archive/absolute-naturalness-review-stage.ts#same',
      'artifact/corpus-run/artifact-two-lane-read-naturalness-seat.ts#same',
    ],
    reason: 'nested inside the frozen copy of uniqueFindings',
  },
  {
    locations: [
      'run/corpus-run/run-seats-cooldown-repeat.unit.test.ts#refuseAndRead',
      'run/corpus-run/run-seats-futile-return.unit.test.ts#refuseAndRead',
    ],
    reason: 'the two rigs differ in the client they hand the seat reader: the cooldown cases must not hand it the '
      + 'futile returns, since they pin the give-up alone, and the futile-return cases must (ledger B1010)',
  },
];

/**
 Node kinds that carry a function body.
 */
const FUNCTION_KINDS: ReadonlySet<string> = new Set([
  'ArrowFunctionExpression',
  'FunctionDeclaration',
  'FunctionExpression',
],);

/**
 Node kinds whose own name a function held in them takes: a variable, a
 property, a method, a class field.
 */
const NAMING_KINDS: ReadonlySet<string> = new Set([
  'MethodDefinition',
  'Property',
  'PropertyDefinition',
  'VariableDeclarator',
],);

/**
 Name read where a node names nothing.
 */
const UNNAMED = '';

/**
 Name a location takes for a function nothing names.
 */
const ANONYMOUS = '(anonymous)';

/**
 Whether a name read from a node names something.

 @param name - name read, `UNNAMED` where there was none

 @returns Whether it is a name

 @example
 ```ts
 const named = ['', 'nap',].find(isNamed,); // 'nap'
 ```
 */
function isNamed(name: string,): boolean {
  return name !== UNNAMED;
}

/**
 The identifier a node names, where its name field holds one.

 @param node - node read

 @param field - field holding the name: `id` for a function or a variable, `key` for a property

 @returns Name, or `UNNAMED`

 @example
 ```ts
 const name = identifierIn({ node, field: 'id', },);
 ```
 */
function identifierIn(
  {
    node,
    field,
  }: {
    readonly node: TreeNode;
    readonly field: 'id' | 'key';
  },
): string {
  /**
   Node in that field.
   */
  const named = node[field];
  return (isTreeNode(named,) && (named.type === 'Identifier') && ((typeof named.name) === 'string'))
    ? named.name
    : UNNAMED;
}

/**
 Node kinds whose source is one literal token, compared as written: a
 string's or a template's spaces are part of its value, so dropping them
 would group two bodies that build different text (ledger B114).
 */
const LITERAL_KINDS: ReadonlySet<string> = new Set([
  'Literal',
  'TemplateElement',
],);

/**
 A stretch of a body the comparison treats apart from code: a comment, cut,
 or a literal token, kept as written.
 */
type BodySpan = {
  /**
   Offset where it starts.
   */
  readonly start: number;

  /**
   Offset where it ends.
   */
  readonly end: number;

  /**
   Whether its text is kept as written rather than cut.
   */
  readonly kept: boolean;
};

/**
 Whether one character of code shows, as opposed to spacing.

 @param character - one code point

 @returns Whether `trim` leaves it

 @example
 ```ts
 const shown = Array.from('a b',).filter(isVisible,); // ['a', 'b']
 ```
 */
function isVisible(character: string,): boolean {
  return character.trim() !== '';
}

/**
 Orders two spans by where they start.

 @param left - one span

 @param right - another

 @returns Negative, zero or positive

 @example
 ```ts
 const ordered = spans.toSorted(byStart,);
 ```
 */
function byStart(left: BodySpan, right: BodySpan,): number {
  return left.start - right.start;
}

/**
 A body's text with every comment inside it removed and every whitespace
 character of its code dropped, its literal tokens kept as written.

 @param text - file text

 @param body - body node

 @param comments - file's comments, in order

 @returns Normalized body

 @example
 ```ts
 const normalized = normalizedBody({ text, body, comments, },);
 ```
 */
function normalizedBody(
  {
    text,
    body,
    comments,
  }: {
    readonly text: string;
    readonly body: TreeNode;
    readonly comments: readonly { readonly start: number; readonly end: number; }[];
  },
): string {
  /**
   Comments and literal tokens inside the body, in source order; neither
   holds the other, since a comment is no node and a literal holds no
   comment.
   */
  const spans = [
    ...comments
      .filter(function within(comment,): boolean {
        return (comment.start >= body.start) && (comment.end <= body.end);
      },)
      .map(function cut(comment,): BodySpan {
        return {
          start: comment.start,
          end: comment.end,
          kept: false,
        };
      },),
    ...nodesUnder({ root: body, },)
      .filter(function isLiteral(node,): boolean {
        return LITERAL_KINDS.has(node.type,);
      },)
      .map(function keep(node,): BodySpan {
        return {
          start: node.start,
          end: node.end,
          kept: true,
        };
      },),
  ].toSorted(byStart,);
  /**
   Where each run of code between spans starts.
   */
  const starts = [
    body.start,
    ...spans.map(function after(span,): number {
      return span.end;
    },),
  ];
  return starts.map(function run(start, at,): string {
    /**
     Span closing this run, none after the last.
     */
    const closing = spans[at];
    /**
     The run's code with its spacing dropped.
     */
    const code = Array.from(text.slice(
      start,
      closing?.start ?? body.end,
    ),)
      .filter(isVisible,)
      .join('',);
    return (closing?.kept === true)
      ? `${code}${text.slice(
        closing.start,
        closing.end,
      )}`
      : code;
  },)
    .join('',);
}

/**
 Every compared body in one file, as normalized text beside its location.

 @param file - source file read

 @returns Pairs of normalized body and location

 @throws {@link Error} when the file does not parse, since a file the scan cannot read hides its copies

 @example
 ```ts
 const bodies = bodiesOf({ file: { path: 'cat.ts', text: 'function nap() { return 1; }', isTest: false, }, },);
 ```
 */
function bodiesOf({ file, }: { readonly file: SourceText; },): readonly (readonly [string, string])[] {
  /**
   Parsed file.
   */
  const { program, comments, } = parseSource({ file, },);
  /**
   Bodies found so far.
   */
  const found: (readonly [string, string])[] = [];
  /**
   Nodes still to visit, each with the name its holder gives a function.
   */
  const pending: { readonly node: TreeNode; readonly held: string; }[] = [{
    node: program,
    held: UNNAMED,
  },];
  while (pending.length > 0) {
    /**
     Node visited now.
     */
    const { node, held, } = pending.pop() as { readonly node: TreeNode; readonly held: string; };
    if (FUNCTION_KINDS.has(node.type,) && isTreeNode(node.body,)) {
      /**
       Body normalized for comparison.
       */
      const normalized = normalizedBody({
        text: file.text,
        body: node.body,
        comments,
      },);
      if (normalized.length >= SHORTEST_COMPARED) {
        found.push([
          normalized,
          `${file.path}#${[identifierIn({ node, field: 'id', },), held,].find(isNamed,) ?? ANONYMOUS}`,
        ],);
      }
    }
    /**
     Name this node gives a function it holds.
     */
    const gives = NAMING_KINDS.has(node.type,)
      ? ([identifierIn({ node, field: 'id', },), identifierIn({ node, field: 'key', },),].find(isNamed,) ?? UNNAMED)
      : UNNAMED;
    childNodes({ node, },).forEach(function queue(child,): void {
      pending.push({
        node: child,
        held: gives,
      },);
    },);
  }
  return found;
}

/**
 Orders two groups by their first location.

 @param left - one group of sorted locations

 @param right - another

 @returns Negative, zero or positive, by code point, the same on every
 machine (ledger B95)

 @example
 ```ts
 const ordered = groups.toSorted(byFirstLocation,);
 ```
 */
function byFirstLocation(left: readonly string[], right: readonly string[],): number {
  return compareCodePoints({
    left: left[0] ?? '',
    right: right[0] ?? '',
  },);
}

/**
 Groups of two or more functions keeping one body, each as its sorted
 locations, the groups in order of their first location.

 @param files - source files read

 @returns Duplicate groups

 @example
 ```ts
 const groups = duplicateGroups({ files, },);
 ```
 */
function duplicateGroups({ files, }: { readonly files: readonly SourceText[]; },): readonly (readonly string[])[] {
  /**
   Locations per normalized body.
   */
  const byBody = new Map<string, string[]>();
  files.flatMap(function scan(file,) {
    return bodiesOf({ file, },);
  },)
    .forEach(function place([body, location,],): void {
      byBody.set(body, [...(byBody.get(body,) ?? []), location,],);
    },);
  return [...byBody.values(),]
    .filter(function repeated(locations,): boolean {
      return locations.length >= 2;
    },)
    .map(function sorted(locations,): readonly string[] {
      return locations.toSorted();
    },)
    .toSorted(byFirstLocation,);
}

/**
 A cat-themed function whose body clears the compared floor.

 @param name - function name

 @param bowl - identifier the body reads, to make two bodies differ

 @param note - comment written inside the body

 @returns Source text

 @example
 ```ts
 const text = catFunction({ name: 'feedCat', bowl: 'bowl', note: 'kibble first', },);
 ```
 */
function catFunction(
  {
    name,
    bowl,
    note,
  }: {
    readonly name: string;
    readonly bowl: string;
    readonly note: string;
  },
): string {
  return [
    `export function ${name}(${bowl}: readonly number[]): number {`,
    `  // ${note}`,
    `  return ${bowl}.filter((kibble) => kibble > 0).map((kibble) => kibble * 2).reduce((sum, kibble) => sum + kibble, 0);`,
    '}',
  ].join('\n',);
}

/**
 A cat-themed function whose body clears the compared floor and carries one
 literal twice.

 @param name - function name

 @param purr - literal's text, to make two bodies differ inside a literal only

 @param quote - delimiter: a string's quote or a template's backtick

 @returns Source text

 @example
 ```ts
 const text = catPurr({ name: 'purrOnce', purr: 'pr r', quote: '\'', },);
 ```
 */
function catPurr(
  {
    name,
    purr,
    quote,
  }: {
    readonly name: string;
    readonly purr: string;
    readonly quote: '\'' | '`';
  },
): string {
  return [
    `export function ${name}(bowl: readonly number[]): string {`,
    `  return bowl.filter((kibble) => kibble > 0).map((kibble) => String(kibble) + ${quote}${purr}${quote})`
      + `.join(${quote}${purr}${quote});`,
    '}',
  ].join('\n',);
}

/**
 A fixture file, as the scan reads one.

 @param path - file name

 @param text - file text

 @returns Source file

 @example
 ```ts
 const file = fixture({ path: 'cat.ts', text: 'function nap() {}', },);
 ```
 */
function fixture(
  {
    path,
    text,
  }: {
    readonly path: string;
    readonly text: string;
  },
): SourceText {
  return {
    path,
    text,
    isTest: false,
  };
}

await describe({
  name: 'duplicate function bodies',
  children: [
    it({
      name: 'FINDS a body kept in two files whatever their comments and layout, and one kept twice in one file',
      fn: async () => {
        expect(duplicateGroups({
          files: [
            fixture({
              path: 'cat-a.ts',
              text: catFunction({ name: 'feedCat', bowl: 'bowl', note: 'kibble first', },),
            },),
            fixture({
              path: 'cat-b.ts',
              text: catFunction({ name: 'serveCat', bowl: 'bowl', note: 'water after', },)
                .replaceAll('(kibble) => kibble > 0', '(kibble)=>kibble>0',),
            },),
          ],
        },),).toEqual([['cat-a.ts#feedCat', 'cat-b.ts#serveCat',],],);
        expect(duplicateGroups({
          files: [fixture({
            path: 'cat-c.ts',
            text: [
              catFunction({ name: 'feedCat', bowl: 'bowl', note: 'one', },),
              catFunction({ name: 'feedKitten', bowl: 'bowl', note: 'two', },),
            ].join('\n\n',),
          },),],
        },),).toEqual([['cat-c.ts#feedCat', 'cat-c.ts#feedKitten',],],);
      },
    },),
    it({
      name: 'LEAVES bodies that differ in one name, and short bodies kept twice',
      fn: async () => {
        expect(duplicateGroups({
          files: [
            fixture({
              path: 'cat-a.ts',
              text: catFunction({ name: 'feedCat', bowl: 'bowl', note: 'one', },),
            },),
            fixture({
              path: 'cat-b.ts',
              text: catFunction({ name: 'feedCat', bowl: 'dish', note: 'one', },),
            },),
            fixture({
              path: 'cat-c.ts',
              text: 'export function nap(): number { return 1; }\nexport function doze(): number { return 1; }',
            },),
          ],
        },),).toEqual([],);
      },
    },),
    it({
      name: 'COMPARES LITERALS AS WRITTEN: leaves bodies whose strings or templates differ only in their spaces, '
        + 'or in a U+FEFF that `trim` reads as space, and still groups bodies whose literals match '
        + 'and whose code is laid out differently',
      fn: async () => {
        expect(duplicateGroups({
          files: [
            fixture({
              path: 'cat-a.ts',
              text: catPurr({ name: 'purrOnce', purr: 'pr r', quote: '\'', },),
            },),
            fixture({
              path: 'cat-b.ts',
              text: catPurr({ name: 'purrTwice', purr: 'pr  r', quote: '\'', },),
            },),
            fixture({
              path: 'cat-c.ts',
              text: catPurr({ name: 'purrQuietly', purr: '\uFEFF', quote: '`', },),
            },),
            fixture({
              path: 'cat-d.ts',
              text: catPurr({ name: 'purrSoftly', purr: '\uFEFF ', quote: '`', },),
            },),
          ],
        },),).toEqual([],);
        expect(duplicateGroups({
          files: [
            fixture({
              path: 'cat-a.ts',
              text: catPurr({ name: 'purrOnce', purr: 'pr r', quote: '`', },),
            },),
            fixture({
              path: 'cat-b.ts',
              text: catPurr({ name: 'purrAgain', purr: 'pr r', quote: '`', },)
                .replaceAll(' => ', '=>',),
            },),
          ],
        },),).toEqual([['cat-a.ts#purrOnce', 'cat-b.ts#purrAgain',],],);
      },
    },),
    it({
      name: 'KEEPS NO BODY IN TWO PLACES across the package\'s source, tests and fixtures, but the frozen copies '
        + 'listed with their reasons, and every listed copy still stands',
      fn: async () => {
        /**
         Every source file, tests and test fixtures included: a helper copied
         between tests drifts as one copied between modules does (ledger
         B103), and a test that copies a module's body tests its copy.
         */
        const files = await readPackageSource();
        expect(files.some(function isTestFile(file,): boolean {
          return file.isTest;
        },),).toBe(true,);
        expectFindingsAsListed({
          findings: duplicateGroups({ files, },),
          listed: FROZEN_COPIES
            .map(function locationsOf({ locations, },): readonly string[] {
              return locations.toSorted();
            },)
            .toSorted(byFirstLocation,),
        },);
      },
    },),
  ],
},);
