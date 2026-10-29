/**
 Guards against a test case that passes without printing a line a count can
 see. module-test's `describe` logs the passing children of a suite named by
 a non-empty string as one `info` line, `[PASS] childA, childB`, and the
 passing children of a suite whose `name` is the empty string at `debug`,
 which the package's runs do not show. A case whose nearest enclosing suite
 has no name therefore ran and passed in no count of `[PASS]` lines the
 ledger and the commit messages quote, and a reader of a run cannot tell it
 ran. A failure of such a case still prints at `error` and still fails the
 file, so this is a gap in counting and visibility, not a case that cannot
 fail. The package's convention is an unnamed root suite whose children are
 named suites (`name: someFunction.name`).

 WHAT THE SCAN READS, in every `*.test.ts` file under `src`: each
 `it(...)` call, the nearest `describe(...)` call among its ancestors, and
 that call's `name` property. A name written as the empty string literal or
 as a template literal holding nothing is empty. The cases a spread of a
 mapped array builds count like any other, since the walk goes by ancestors
 and not by position in a list. A case under an unnamed suite nested in a
 named one is silent too: the nearest suite decides. A case with no
 enclosing `describe(...)` in its own file is a finding of its own, since
 no suite prints it.

 WHAT A FINDING NAMES: the file, each enclosing suite's name from the
 outermost, an empty name written `(unnamed)`, and the case's name as the
 source writes it.

 OUT OF THE SCAN'S REACH: a suite whose name is any other expression (a
 variable or a member such as `someFunction.name`), which is taken as
 non-empty, and a file that imports `describe` or `it` under another name.

 THE FIXTURES COME FIRST, so the package-wide case is read against a scan
 shown able to find each shape (ledger M21). Fixtures are cat-themed; the
 package case reads this package's own tests.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { readPackageSource, } from './family-source.test-fixture.ts';
import {
  ancestorsOf,
  identifierName,
  isTreeNode,
  literalText,
  nodesUnder,
  parentsOf,
  parseSource,
  type SourceText,
  type TreeNode,
} from './source-scan.test-fixture.ts';
import { expectNoFindings, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scan-findings.test-fixture.ts';

//region Silent case scan
// Each `it(...)` call read against the suites that hold it, nearest first.

/**
 What a suite's `name` property writes.
 */
type WrittenName =
  | {
    /**
     The name is a literal or an expressionless template.
     */
    readonly kind: 'plain';

    /**
     The string it carries.
     */
    readonly text: string;
  }
    | {
    /**
     The name is any other expression, which the scan takes as non-empty.
     */
      readonly kind: 'computed';
    };

/**
 Text a suite name property writes, when it writes a plain string.

 @param node - name property's value

 @returns String a literal or an expressionless template carries, or a
 computed name for any other expression

 @example
 ```ts
 const written = plainName({ node: property.value, },); // { kind: 'plain', text: '', } for name: ''
 ```
 */
function plainName({ node, }: { readonly node: unknown; },): WrittenName {
  if (!isTreeNode(node,))
    return { kind: 'computed', };
  if (node.type === 'Literal') {
    return ((typeof node.value) === 'string')
      ? {
        kind: 'plain',
        text: literalText({ node, },),
      }
      : { kind: 'computed', };
  }
  if (node.type !== 'TemplateLiteral')
    return { kind: 'computed', };
  /**
   The template's expressions and string parts.
   */
  const { expressions, quasis, } = node;
  if ((!Array.isArray(expressions,)) || (!Array.isArray(quasis,)) || (expressions.length > 0))
    return { kind: 'computed', };
  return {
    kind: 'plain',
    text: quasis.map(function cooked(quasi,): string {
      if ((!isTreeNode(quasi,)) || (!isTreeNode(quasi.value,)))
        return '';
      return ((typeof quasi.value.cooked) === 'string') ? quasi.value.cooked as string : '';
    },)
      .join('',),
  };
}

/**
 The `name` property's value in a call's first argument.

 @param call - `describe(...)` or `it(...)` call

 @returns Value node, `undefined` when the argument is no object or has
 no plain `name` property

 @example
 ```ts
 const value = nameValueOf({ call, },);
 ```
 */
function nameValueOf({ call, }: { readonly call: TreeNode; },): unknown {
  /**
   The call's arguments.
   */
  const args: readonly unknown[] = Array.isArray(call.arguments,) ? call.arguments : [];
  const [first,] = args;
  if ((!isTreeNode(first,)) || (first.type !== 'ObjectExpression'))
    return undefined;
  /**
   The object's members.
   */
  const properties: readonly unknown[] = Array.isArray(first.properties,) ? first.properties : [];
  /**
   The member named `name`, written plainly.
   */
  const member = properties.find(function isName(property,): boolean {
    return isTreeNode(property,) && (property.type === 'Property') && (property.computed !== true)
      && (identifierName({ node: property.key, },) === 'name');
  },);
  return isTreeNode(member,) ? member.value : undefined;
}

/**
 Whether a node is a call of a function by that plain name.

 @param node - node read

 @param callee - function name

 @returns Whether it is `callee(...)`

 @example
 ```ts
 const isIt = isCallOf({ node, callee: 'it', },);
 ```
 */
function isCallOf({ node, callee, }: { readonly node: TreeNode; readonly callee: string; },): boolean {
  return (node.type === 'CallExpression') && (identifierName({ node: node.callee, },) === callee);
}

/**
 Every case under a suite with no name, or under none, in the given files.

 @param files - source files read, whose tests the scan parses

 @returns Each as `path > suite > case`, sorted without repeats

 @example
 ```ts
 const silent = silentCases({ files, },);
 ```
 */
function silentCases({ files, }: { readonly files: readonly SourceText[]; },): readonly string[] {
  return [
    ...new Set(files
      .filter(function isTestFile(file,): boolean {
        return file.path.endsWith('.test.ts',);
      },)
      .flatMap(function inFile(file,): readonly string[] {
        /**
         The file's program and each node's parent.
         */
        const { program, } = parseSource({ file, },);
        const parents = parentsOf({ program, },);
        return nodesUnder({ root: program, },)
          .filter(function isCase(node,): boolean {
            return isCallOf({
              node,
              callee: 'it',
            },);
          },)
          .flatMap(function located(node,): readonly string[] {
            /**
             The suites holding the case, nearest first.
             */
            const suites = ancestorsOf({
              node,
              parents,
            },)
              .filter(function isSuite(ancestor,): boolean {
                return isCallOf({
                  node: ancestor,
                  callee: 'describe',
                },);
              },)
              .map(function suiteName(suite,): WrittenName {
                return plainName({ node: nameValueOf({ call: suite, },), },);
              },);
            /**
             The case's name as the source writes it.
             */
            const written = nameValueOf({ call: node, },);
            const caseName = isTreeNode(written,) ? file.text.slice(
              written.start,
              written.end,
            )
              .split('\n',)
              .map(function trimmed(line,): string {
                return line.trim();
              },)
              .join(' ',) : '(no name)';
            if (suites.length === 0)
              return [`${file.path} > (outside any suite) > ${caseName}`,];
            if ((suites[0]?.kind !== 'plain') || (suites[0].text !== ''))
              return [];
            return [
              `${file.path} > ${
                suites.toReversed()
                  .map(function shown(name,): string {
                    if (name.kind === 'computed')
                      return '(named)';
                    return name.text === '' ? '(unnamed)' : name.text;
                  },)
                  .join(' > ',)
              } > ${caseName}`,
            ];
          },);
      },),),
  ].toSorted();
}

//endregion Silent case scan

/**
 A test file for the fixture case.

 @param path - file name, which must end in `.test.ts` for the scan to read it

 @param lines - file lines

 @returns Source file

 @example
 ```ts
 const file = fixture({ path: 'cat.unit.test.ts', lines: ['await describe({ name: \'\', children: [], },);',], },);
 ```
 */
function fixture({ path, lines, }: { readonly path: string; readonly lines: readonly string[]; },): SourceText {
  return {
    path,
    text: lines.join('\n',),
    isTest: true,
  };
}

/**
 The cat fixtures: a file whose cases sit straight under the unnamed root;
 a file whose cases sit under named suites beside a name taken from a
 member, which the scan leaves alone; a file with an unnamed suite nested in
 a named one; a file whose cases a spread of a mapped array builds under the
 unnamed root and under a named suite; a file whose suite name is an empty
 template literal and one whose name is a template with an expression; a
 file with a case outside any suite; and a file that is no test.
 */
const CAT_FILES: readonly SourceText[] = [
  fixture({
    path: 'purring.unit.test.ts',
    lines: [
      'await describe({',
      '  name: \'\',',
      '  children: [',
      '    it({ name: \'purrs when stroked\', fn: async () => {}, },),',
      '    it({ name: \'kneads the blanket\', fn: async () => {}, },),',
      '  ],',
      '},);',
    ],
  },),
  fixture({
    path: 'napping.unit.test.ts',
    lines: [
      'await describe({',
      '  name: \'\',',
      '  children: [',
      '    describe({',
      '      name: curl.name,',
      '      children: [it({ name: \'curls up in the sun\', fn: async () => {}, },),],',
      '    },),',
      '    describe({',
      '      name: \'stretch\',',
      '      children: [it({ name: \'stretches after a nap\', fn: async () => {}, },),],',
      '    },),',
      '  ],',
      '},);',
    ],
  },),
  fixture({
    path: 'grooming.unit.test.ts',
    lines: [
      'await describe({',
      '  name: \'\',',
      '  children: [',
      '    describe({',
      '      name: \'lick\',',
      '      children: [',
      '        it({ name: \'licks a paw\', fn: async () => {}, },),',
      '        describe({',
      '          name: \'\',',
      '          children: [it({ name: \'licks the other paw\', fn: async () => {}, },),],',
      '        },),',
      '      ],',
      '    },),',
      '  ],',
      '},);',
    ],
  },),
  fixture({
    path: 'hunting.unit.test.ts',
    lines: [
      'await describe({',
      '  name: \'\',',
      '  children: [',
      '    ...[\'mouse\', \'sparrow\',].map(prey => it({',
      `      name: \`stalks a \${prey}\`,`,
      '      fn: async () => {},',
      '    },),),',
      '    describe({',
      '      name: pounce.name,',
      '      children: [',
      `        ...['string', 'ball',].map(toy => it({ name: \`pounces on a \${toy}\`, fn: async () => {}, },),),`,
      '      ],',
      '    },),',
      '  ],',
      '},);',
    ],
  },),
  fixture({
    path: 'templates.unit.test.ts',
    lines: [
      'await describe({',
      `  name: \`\${suiteName}\`,`,
      '  children: [it({ name: \'purrs under a computed suite name\', fn: async () => {}, },),],',
      '},);',
      'await describe({',
      '  name: ``,',
      '  children: [it({ name: \'purrs under an empty template name\', fn: async () => {}, },),],',
      '},);',
    ],
  },),
  fixture({
    path: 'stray.unit.test.ts',
    lines: [
      'const strayCase = it({ name: \'sits outside every suite\', fn: async () => {}, },);',
      'export { strayCase, };',
    ],
  },),
  fixture({
    path: 'purring-helper.ts',
    lines: ['export const silent = it({ name: \'is no test file\', fn: async () => {}, },);',],
  },),
];

await describe({
  name: '',
  children: [
    describe({
      name: 'cases that print no pass line',
      children: [
        it({
          name: 'FINDS cases under the unnamed root, under an unnamed suite nested in a named one, under a suite '
            + 'named by an empty template, built by a spread of a mapped array, and outside any suite; KEEPS '
            + 'cases under a suite named by a member, by a string, by a template with an expression, and in a '
            + 'file that is no test',
          fn: async () => {
            expect(silentCases({ files: CAT_FILES, },),).toEqual([
              'grooming.unit.test.ts > (unnamed) > lick > (unnamed) > \'licks the other paw\'',
              `hunting.unit.test.ts > (unnamed) > \`stalks a \${prey}\``,
              'purring.unit.test.ts > (unnamed) > \'kneads the blanket\'',
              'purring.unit.test.ts > (unnamed) > \'purrs when stroked\'',
              'stray.unit.test.ts > (outside any suite) > \'sits outside every suite\'',
              'templates.unit.test.ts > (unnamed) > \'purrs under an empty template name\'',
            ],);
          },
        },),
        it({
          name: 'FINDS NO CASE THAT PRINTS NO PASS LINE across the package\'s tests',
          fn: async () => {
            expectNoFindings({ findings: silentCases({ files: await readPackageSource(), },), },);
          },
        },),
      ],
    },),
  ],
},);
