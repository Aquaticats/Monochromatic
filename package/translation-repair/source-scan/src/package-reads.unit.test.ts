/**
 Guards the reach of the family's tests and test fixtures: none reads, lists or runs git over the files of a
 package it does not declare (ledger B1265). The gate runs the packages a change can reach through the declared
 package graph (`workspace:*` in `package.json`), so a test that reads another package's files by path, with
 no declaration to say so, passes in a gate that never ran it and fails later in one that did. A test of a tool
 that reads files or history builds a throwaway folder or repository it reads; a check whose subject is the whole
 family's source lives in this package, whose input is the whole family and whose scans run in every gate; a test
 that needs another package's code imports that package's API through a dependency it declares.

 WHAT THE SCAN READS, in tests and test fixtures of every package but this one. A PATH CHAIN is an expression built
 from an anchor, the file's own place (`import.meta.dirname`, `import.meta.filename`, `import.meta.url`,
 `__dirname`, `__filename`) or the working directory (`process.cwd()`), through `join`, `resolve`, `normalize`,
 `dirname`, `fileURLToPath`, `pathToFileURL`, `new URL(relative, import.meta.url)`, `import.meta.resolve` of a
 relative specifier, a template literal or a `+` joining it to text, or a constant holding such a chain. A chain
 whose `..` segments climb past the package folder is a `climb` finding, keyed at the expression that climbs (a
 constant holding the chain, and not each place that reads the constant). The package folder is where the file's
 package sits, so a file at `src/corpus-run` may climb two levels (to the package) and no third. A static import,
 re-export, dynamic import or `require` of a relative specifier that climbs past the package folder is an `import`
 finding. A call of a read of the file system (`readFile`, `readdir`, `stat`, `access`, `opendir`, `existsSync`
 and their synchronous and stream forms, bare or through a member) whose path is written in literals from its
 root, without a variable at its start, is a `literal-read` finding when it begins by climbing or names
 `package/translation-repair`; a `glob` whose pattern begins by climbing past the package folder from its `cwd` is
 a `glob` finding. A git read (`log`, `show`, `diff`, `ls-files`, `ls-tree`, `rev-list`, `rev-parse`, `grep`,
 `cat-file`, `blame`, `status`, `describe`, `for-each-ref`, `merge-base`) is a `git-checkout` finding when its `-C`
 argument or its folder option (`cwd`, `cloneDir`, `repository`, `directory`) is a path chain (the live checkout),
 and a `git-cwd` finding when it names neither (git then runs where the test does, in the live checkout).

 LEFT ALONE, because it reads a folder the test made or a package it declares: a path rooted at a variable, member
 or call that is no anchor (a scratch folder, a corpus clone, a fixture repository's path), a package imported by
 name (an import of `@monochromatic-dev/translation-repair-<name>` or `import.meta.resolve` of one, which Node
 resolves only for a package the importing package declares), and the package's own folder, `dist` and `node_modules`
 among it. A literal naming `package/translation-repair` as data (a pathspec a fixture repository is asked for, a path
 a tool is handed to parse) is no read.

 OUT OF ITS REACH: a path built from a text another module returns, a climb written in a variable that
 holds the `..` segment, a git read whose arguments are built apart from its call and which has no `-C` argument,
 and a read of a file by a child process the test starts. Reading the package graph itself (which package
 imports which) is another scan's.

 THE FILES LISTED are reads the scan finds that belong to no package: each holds its reason, the file, the kind and the
 number of findings. A listed read the source no longer holds as often fails the scan, so the list cannot outlive
 its reasons.

 THE FIXTURE CASES COME FIRST, so the package-wide case is read against a scan shown able to find each form and to
 leave each control alone (ledger M21). Fixtures are cat-themed; the package case reads the family's own tests.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';

import {
  packageDirectoryOf,
  readPackageSource,
} from './family-source.test-fixture.ts';
import {
  identifierName,
  isTreeNode,
  literalText,
  memberName,
  nodesUnder,
  parentsOf,
  parseSource,
  type SourceText,
  type TreeNode,
  unwrapped,
} from './source-scan.test-fixture.ts';
import {
  expectFindingsAsListed,
  expectNoFindings,
  expectRecordAsListed,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/scan-findings.test-fixture.ts';

//region Package reads
// Where an expression built from a path anchor ends up relative to the package folder, and the calls that read
// through such a path.

/**
 The package this scan belongs to, whose scans read the whole family on purpose.
 */
const SCAN_PACKAGE = 'source-scan';

/**
 How an expression stands to the package folder: the level it ends at and the lowest it reached on the way, level 0
 being the package folder, 1 its `src`, and a negative level outside it.
 */
type Reach = {
  /**
   Whether the expression is a path chain at all.
   */
  readonly isChain: boolean;

  /**
   Level the expression names.
   */
  readonly end: number;

  /**
   Lowest level any step of the chain reached.
   */
  readonly lowest: number;

  /**
   Whether the chain starts from a constant the file already holds that leaves the package, which is reported at the
   constant and not again where it is read.
   */
  readonly inherited: boolean;
};

/**
 What an expression that is no path chain stands for.
 */
const NOT_A_CHAIN: Reach = {
  isChain: false,
  end: 0,
  lowest: 0,
  inherited: false,
};

/**
 The kinds of finding.
 */
type FindingKind = 'climb' | 'git-checkout' | 'git-cwd' | 'glob' | 'import' | 'literal-read';

/**
 One read the scan found.
 */
type PackageRead = {
  /**
   Location of the file, as `readPackageSource` names it.
   */
  readonly path: string;

  /**
   Line the expression starts on.
   */
  readonly line: number;

  /**
   Which kind of read.
   */
  readonly kind: FindingKind;

  /**
   The expression as written, on one line.
   */
  readonly shown: string;
};

/**
 A path written in literals from its start, or the mark that it is not.
 */
type WrittenPath = {
  /**
   Whether every part up to the first variable is a literal and the start is one.
   */
  readonly isWritten: boolean;

  /**
   Its text, a variable in it one ordinary segment.
   */
  readonly text: string;
};

/**
 What a path that does not start in literals stands for.
 */
const NOT_WRITTEN: WrittenPath = {
  isWritten: false,
  text: '',
};

/**
 Names of the calls that read the file system.
 */
const READ_CALLS: ReadonlySet<string> = new Set([
  'access',
  'accessSync',
  'createReadStream',
  'existsSync',
  'glob',
  'globSync',
  'lstat',
  'lstatSync',
  'open',
  'opendir',
  'opendirSync',
  'readdir',
  'readdirSync',
  'readFile',
  'readFileSync',
  'readlink',
  'readlinkSync',
  'realpath',
  'realpathSync',
  'stat',
  'statSync',
],);

/**
 Names of the calls that join path text, whose first argument carries the chain.
 */
const JOIN_CALLS: ReadonlySet<string> = new Set([
  'join',
  'normalize',
  'resolve',
],);

/**
 Names of the calls that hand a chain on unchanged.
 */
const PASS_CALLS: ReadonlySet<string> = new Set([
  'fileURLToPath',
  'pathToFileURL',
],);

/**
 Names of the git commands that read a repository's files or history.
 */
const GIT_READS: ReadonlySet<string> = new Set([
  'blame',
  'cat-file',
  'describe',
  'diff',
  'for-each-ref',
  'grep',
  'log',
  'ls-files',
  'ls-tree',
  'merge-base',
  'rev-list',
  'rev-parse',
  'show',
  'status',
],);

/**
 Names of the object keys that carry the folder a git command runs in.
 */
const GIT_FOLDER_KEYS: ReadonlySet<string> = new Set([
  'cloneDir',
  'cwd',
  'directory',
  'repository',
],);

/**
 What the family folder is called in a path written whole.
 */
const FAMILY_FOLDER = 'package/translation-repair';

/**
 The text a path argument stands in for where it is not a literal: one ordinary segment.
 */
const UNKNOWN_SEGMENT = '\u0000';

/**
 Whether a name holds the word git, in any case.

 @param name - identifier, callee or text read

 @returns Whether it names git

 @example
 ```ts
 const named = namesGit({ name: 'fixtureGit', },); // true
 ```
 */
function namesGit({ name, }: { readonly name: string; }): boolean {
  return name.toLowerCase()
    .includes('git',);
}

/**
 The level a chain ends at after a path written as text, each `..` going up and each ordinary segment down.

 @param reach - where the chain stood before the text

 @param text - path text, its segments split on slashes

 @returns Where the chain ends and the lowest level it reached

 @example
 ```ts
 const next = afterText({ reach: { isChain: true, end: 1, lowest: 1, inherited: false, }, text: '../../dog', },); // end 0, lowest -1
 ```
 */
function afterText({ reach, text, }: { readonly reach: Reach; readonly text: string; },): Reach {
  return text
    .split('/',)
    .filter(function isStep(segment,): boolean {
      return (segment !== '') && (segment !== '.');
    },)
    .reduce(function stepped(passed: Reach, segment: string,): Reach {
      /**
       The level after this segment.
       */
      const end = passed.end + ((segment === '..') ? (-1) : 1);
      return {
        isChain: passed.isChain,
        end,
        lowest: Math.min(
          passed.lowest,
          end,
        ),
        inherited: passed.inherited,
      };
    }, reach,);
}

/**
 A chain that starts at a level.

 @param level - level the chain starts at

 @returns Chain, not yet inherited from a constant

 @example
 ```ts
 const start = startAt({ level: 2, },);
 ```
 */
function startAt({ level, }: { readonly level: number; }): Reach {
  return {
    isChain: true,
    end: level,
    lowest: level,
    inherited: false,
  };
}

/**
 The text a node writes as part of a path: a string, a template with each expression an ordinary segment, or one
 ordinary segment for anything else.

 @param node - argument read

 @returns Its text, split later on slashes

 @example
 ```ts
 const text = segmentText({ node, },); // '../' followed by a null character for a template that starts '../' and holds one expression
 ```
 */
function segmentText({ node, }: { readonly node: unknown; }): string {
  if (!isTreeNode(node,))
    return UNKNOWN_SEGMENT;
  if (node.type === 'Literal')
    return ((typeof node.value) === 'string') ? node.value as string : UNKNOWN_SEGMENT;
  if (node.type !== 'TemplateLiteral')
    return UNKNOWN_SEGMENT;
  /**
   The template's quasis.
   */
  const quasis = Array.isArray(node.quasis,) ? node.quasis as readonly unknown[] : [];
  return quasis
    .map(function cookedOf(quasi,): string {
      if ((!isTreeNode(quasi,)) || ((typeof quasi.value) !== 'object') || (quasi.value === null))
        return '';
      /**
       The quasi's cooked text.
       */
      const { cooked, } = quasi.value as Readonly<Record<string, unknown>>;
      return ((typeof cooked) === 'string') ? cooked as string : '';
    },)
    .join(UNKNOWN_SEGMENT,);
}

/**
 Whether a node is `import.meta` read through a member.

 @param node - node read

 @param member - member name

 @returns Whether the node is `import.meta.<member>`

 @example
 ```ts
 const isDirname = isImportMeta({ node, member: 'dirname', },);
 ```
 */
function isImportMeta({ node, member, }: { readonly node: unknown; readonly member: string; },): boolean {
  if ((!isTreeNode(node,)) || (node.type !== 'MemberExpression') || (!isTreeNode(node.object,)))
    return false;
  return (node.object.type === 'MetaProperty') && (memberName({ node, },) === member);
}

/**
 Name a call's callee is written with, bare or through a member.

 @param node - call or new expression

 @returns Identifier or member name, empty for any other callee

 @example
 ```ts
 const name = calleeName({ node: call, },); // 'join' for path.join(...)
 ```
 */
function calleeName({ node, }: { readonly node: TreeNode; }): string {
  /**
   The callee node.
   */
  const { callee, } = node;
  if (!isTreeNode(callee,))
    return '';
  return (callee.type === 'MemberExpression') ? memberName({ node: callee, },) : identifierName({ node: callee, },);
}

/**
 The arguments of a call.

 @param node - call or new expression

 @returns Its arguments, none where the node holds none

 @example
 ```ts
 const [first,] = argumentsOf({ node: call, },);
 ```
 */
function argumentsOf({ node, }: { readonly node: TreeNode; }): readonly unknown[] {
  return Array.isArray(node.arguments,) ? node.arguments as readonly unknown[] : [];
}

/**
 How a path chain stands to the package folder when its anchor is a name the file holds.

 @param name - identifier read

 @param depth - levels between the file's folder and the package folder: 1 for a file directly under `src`

 @param held - constants of the file that hold a chain, by name

 @returns Chain the name stands for, none for a name that is no anchor and holds no chain

 @example
 ```ts
 const reach = nameReach({ name: '__dirname', depth: 2, held: new Map(), },);
 ```
 */
function nameReach(
  {
    name,
    depth,
    held,
  }: {
    readonly name: string;
    readonly depth: number;
    readonly held: ReadonlyMap<string, Reach>;
  },
): Reach {
  if (name === '__dirname')
    return startAt({ level: depth, },);
  if (name === '__filename')
    return startAt({ level: depth + 1, },);
  /**
   A constant of the file that holds a chain.
   */
  const constant = held.get(name,);
  if (constant === undefined)
    return NOT_A_CHAIN;
  return {
    ...constant,
    inherited: constant.lowest < 0,
  };
}

/**
 How an expression stands to the package folder, where it is a path chain.

 @param node - expression read

 @param depth - levels between the file's folder and the package folder: 1 for a file directly under `src`

 @param held - constants of the file that hold a chain, by name

 @returns Where the chain ends, `NOT_A_CHAIN` for an expression that is none

 @example
 ```ts
 const reach = reachOf({ node: call, depth: 2, held: new Map(), },); // end 0, lowest 0 for join(import.meta.dirname, '..', '..')
 ```
 */
function reachOf(
  {
    node,
    depth,
    held,
  }: {
    readonly node: unknown;
    readonly depth: number;
    readonly held: ReadonlyMap<string, Reach>;
  },
): Reach {
  /**
   The node without the wrappers that change nothing.
   */
  const { inner, } = unwrapped({ node, },);
  if (!isTreeNode(inner,))
    return NOT_A_CHAIN;
  if (isImportMeta({ node: inner, member: 'dirname', },))
    return startAt({ level: depth, },);
  if (isImportMeta({ node: inner, member: 'filename', },) || isImportMeta({ node: inner, member: 'url', },))
    return startAt({ level: depth + 1, },);
  if (inner.type === 'Identifier') {
    return nameReach({
      name: identifierName({ node: inner, },),
      depth,
      held,
    },);
  }
  if (inner.type === 'TemplateLiteral') {
    /**
     The expressions of the template and the text around them.
     */
    const expressions = Array.isArray(inner.expressions,) ? inner.expressions as readonly unknown[] : [];
    const text = segmentText({ node: inner, },);
    if ((expressions.length === 0) || (!text.startsWith(UNKNOWN_SEGMENT,)))
      return NOT_A_CHAIN;
    /**
     The chain the first expression is.
     */
    const first = reachOf({
      node: expressions[0],
      depth,
      held,
    },);
    return first.isChain
      ? afterText({
        reach: first,
        text: text.slice(1,),
      },)
      : NOT_A_CHAIN;
  }
  if (inner.type === 'BinaryExpression') {
    if (inner.operator !== '+')
      return NOT_A_CHAIN;
    /**
     The chain the left side is.
     */
    const left = reachOf({
      node: inner.left,
      depth,
      held,
    },);
    return left.isChain
      ? afterText({
        reach: left,
        text: segmentText({ node: inner.right, },),
      },)
      : NOT_A_CHAIN;
  }
  if (inner.type === 'MemberExpression') {
    return ((memberName({ node: inner, },) === 'pathname') || (memberName({ node: inner, },) === 'href'))
      ? reachOf({
        node: inner.object,
        depth,
        held,
      },)
      : NOT_A_CHAIN;
  }
  if (inner.type === 'NewExpression') {
    if (identifierName({ node: inner.callee, },) !== 'URL')
      return NOT_A_CHAIN;
    /**
     The relative text and the base it is resolved against.
     */
    const [relative, base,] = argumentsOf({ node: inner, },);
    if (!isImportMeta({ node: base, member: 'url', },))
      return NOT_A_CHAIN;
    return afterText({
      reach: startAt({ level: depth, },),
      text: segmentText({ node: relative, },),
    },);
  }
  if (inner.type !== 'CallExpression')
    return NOT_A_CHAIN;
  /**
   What the call is named and what it is handed.
   */
  const name = calleeName({ node: inner, },);
  const args = argumentsOf({ node: inner, },);
  if ((name === 'cwd') && isTreeNode(inner.callee,) && (inner.callee.type === 'MemberExpression')
    && (identifierName({ node: inner.callee.object, },) === 'process'))
    return startAt({ level: 0, },);
  if ((name === 'resolve') && isImportMeta({ node: inner.callee, member: 'resolve', },)) {
    /**
     The specifier, which only a relative one names a file by path.
     */
    const specifier = segmentText({ node: args[0], },);
    return specifier.startsWith('.',)
      ? afterText({
        reach: startAt({ level: depth, },),
        text: specifier,
      },)
      : NOT_A_CHAIN;
  }
  if (PASS_CALLS.has(name,)) {
    return reachOf({
      node: args[0],
      depth,
      held,
    },);
  }
  if (name === 'dirname') {
    /**
     The chain whose folder is taken.
     */
    const whole = reachOf({
      node: args[0],
      depth,
      held,
    },);
    return whole.isChain
      ? afterText({
        reach: whole,
        text: '..',
      },)
      : NOT_A_CHAIN;
  }
  if (!JOIN_CALLS.has(name,))
    return NOT_A_CHAIN;
  /**
   The chain the first argument is.
   */
  const first = reachOf({
    node: args[0],
    depth,
    held,
  },);
  if (!first.isChain)
    return NOT_A_CHAIN;
  return args
    .slice(1,)
    .reduce(function joined(reach: Reach, argument: unknown,): Reach {
      return afterText({
        reach,
        text: segmentText({ node: argument, },),
      },);
    }, first,);
}

/**
 The constants of a file that hold a path chain, found by passing over its declarations until no more are found.

 @param nodes - every node of the file

 @param depth - levels between the file's folder and the package folder

 @returns Each constant's chain, by name

 @example
 ```ts
 const held = heldChains({ nodes, depth: 1, },); // FAMILY_ROOT to end -1, lowest -1
 ```
 */
function heldChains({ nodes, depth, }: { readonly nodes: readonly TreeNode[]; readonly depth: number; },): ReadonlyMap<string, Reach> {
  /**
   Chains found so far.
   */
  const held = new Map<string, Reach>();
  for (let again = true; again;) {
    again = false;
    for (const node of nodes) {
      if (node.type !== 'VariableDeclarator')
        continue;
      /**
       The constant's name and the chain its initializer is.
       */
      const name = identifierName({ node: node.id, },);
      if ((name === '') || held.has(name,))
        continue;
      const reach = reachOf({
        node: node.init,
        depth,
        held,
      },);
      if (!reach.isChain)
        continue;
      held.set(
        name,
        {
          ...reach,
          inherited: false,
        },
      );
      again = true;
    }
  }
  return held;
}

/**
 The line a node starts on.

 @param file - file read

 @param node - node read

 @returns Line number, counting from 1

 @example
 ```ts
 const line = lineNumberOf({ file, node, },);
 ```
 */
function lineNumberOf({ file, node, }: { readonly file: SourceText; readonly node: TreeNode; }): number {
  /**
   The text before the node.
   */
  const before = file.text.slice(
    0,
    node.start,
  );
  return before.split('\n',).length;
}

/**
 The expression as written, its lines joined by single spaces.

 @param file - file read

 @param node - node read

 @returns Its text on one line

 @example
 ```ts
 const shown = shownOf({ file, node, },);
 ```
 */
function shownOf({ file, node, }: { readonly file: SourceText; readonly node: TreeNode; }): string {
  /**
   The node's text.
   */
  const text = file.text.slice(
    node.start,
    node.end,
  );
  return text
    .split('\n',)
    .map(function trimmed(line,): string {
      return line.trim();
    },)
    .join(' ',);
}

/**
 The literal strings of an argument that is one string or a list of them.

 @param node - argument read

 @returns Strings, empty text for an element that is not a literal

 @example
 ```ts
 const patterns = patternsOf({ node: args[0], },); // the glob's patterns
 ```
 */
function patternsOf({ node, }: { readonly node: unknown; }): readonly string[] {
  if (!isTreeNode(node,))
    return [];
  if (node.type === 'ArrayExpression') {
    return (Array.isArray(node.elements,) ? node.elements as readonly unknown[] : []).map(function textOf(element,): string {
      return literalText({ node: element, },);
    },);
  }
  return [literalText({ node, },),];
}

/**
 The text a path written in literals from its start stands for: a string, a template that opens with text, or a join
 whose first argument is such.

 @param node - argument read

 @returns Path text, with the mark that it is not written where a variable starts it

 @example
 ```ts
 const written = writtenPath({ node: args[0], },); // text 'package/translation-repair/dog' for join('package', 'translation-repair', 'dog')
 ```
 */
function writtenPath({ node, }: { readonly node: unknown; }): WrittenPath {
  /**
   The node without the wrappers that change nothing.
   */
  const { inner, } = unwrapped({ node, },);
  if (!isTreeNode(inner,))
    return NOT_WRITTEN;
  if (inner.type === 'Literal') {
    return ((typeof inner.value) === 'string')
      ? {
        isWritten: true,
        text: inner.value as string,
      }
      : NOT_WRITTEN;
  }
  if (inner.type === 'TemplateLiteral') {
    /**
     The template's text, a variable in it an ordinary segment.
     */
    const text = segmentText({ node: inner, },);
    return text.startsWith(UNKNOWN_SEGMENT,)
      ? NOT_WRITTEN
      : {
        isWritten: true,
        text,
      };
  }
  if ((inner.type !== 'CallExpression') || (!JOIN_CALLS.has(calleeName({ node: inner, },),)))
    return NOT_WRITTEN;
  /**
   The root of the join, which must be written, and the arguments after it.
   */
  const [root, ...rest] = argumentsOf({ node: inner, },);
  /**
   The root's written text.
   */
  const rootText = writtenPath({ node: root, },);
  if (!rootText.isWritten)
    return NOT_WRITTEN;
  return {
    isWritten: true,
    text: [
      rootText.text,
      ...rest.map(function partOf(argument,): string {
        /**
         The argument's text where it is written.
         */
        const part = writtenPath({ node: argument, },);
        return part.isWritten ? part.text : UNKNOWN_SEGMENT;
      },),
    ].join('/',),
  };
}

/**
 The folder a glob reads from: the `cwd` of its options, the working directory without one.

 @param options - second argument of the call

 @param depth - levels between the file's folder and the package folder

 @param held - constants of the file that hold a chain

 @returns Chain the folder is

 @example
 ```ts
 const base = globBase({ options: args[1], depth: 1, held, },);
 ```
 */
function globBase(
  {
    options,
    depth,
    held,
  }: {
    readonly options: unknown;
    readonly depth: number;
    readonly held: ReadonlyMap<string, Reach>;
  },
): Reach {
  if ((!isTreeNode(options,)) || (options.type !== 'ObjectExpression') || (!Array.isArray(options.properties,)))
    return startAt({ level: 0, },);
  /**
   The property that names the folder.
   */
  const cwdNode = (options.properties as readonly unknown[]).find(function isCwd(property,): boolean {
    return isTreeNode(property,) && (identifierName({ node: property.key, },) === 'cwd');
  },);
  if ((!isTreeNode(cwdNode,)) || (cwdNode.type !== 'Property'))
    return startAt({ level: 0, },);
  return reachOf({
    node: cwdNode.value,
    depth,
    held,
  },);
}

/**
 The reads a call of the file system makes through a path written whole in literals, and the climbs of a glob's pattern.

 @param node - call node

 @param depth - levels between the file's folder and the package folder

 @param held - constants of the file that hold a chain

 @returns Finding kinds the call earns

 @example
 ```ts
 const kinds = readKindsOf({ node, depth: 2, held, },); // ['literal-read'] for readFile('../dog/src/x.ts')
 ```
 */
function readKindsOf(
  {
    node,
    depth,
    held,
  }: {
    readonly node: TreeNode;
    readonly depth: number;
    readonly held: ReadonlyMap<string, Reach>;
  },
): readonly FindingKind[] {
  if (!READ_CALLS.has(calleeName({ node, },),))
    return [];
  /**
   The first argument and the options.
   */
  const [first, options,] = argumentsOf({ node, },);
  if (calleeName({ node, },).startsWith('glob',)) {
    /**
     The folder the pattern is read from.
     */
    const base = globBase({
      options,
      depth,
      held,
    },);
    // A folder that already left the package is reported at its own expression.
    if ((!base.isChain) || (base.lowest < 0))
      return [];
    return patternsOf({ node: first, },).some(function climbs(pattern,): boolean {
      return afterText({
        reach: base,
        text: pattern,
      },).lowest
        < 0;
    },)
      ? ['glob',]
      : [];
  }
  /**
   The path, where it is written in literals from its start.
   */
  const written = writtenPath({ node: first, },);
  if (!written.isWritten)
    return [];
  /**
   Where the working directory's chain ends after it.
   */
  const reach = afterText({
    reach: startAt({ level: 0, },),
    text: written.text,
  },);
  return ((reach.lowest < 0) || written.text.includes(FAMILY_FOLDER,)) ? ['literal-read',] : [];
}

/**
 What an argument list says of a git read: whether it is one, whether it carries a `-C` flag and the folder the flag
 names.

 @param node - array node read

 @returns Whether the command is a read, whether a `-C` flag is there, and the folder node

 @example
 ```ts
 const { folder, } = gitListOf({ node: array, },); // the node of import.meta.dirname for ['-C', import.meta.dirname, 'log']
 ```
 */
function gitListOf(
  { node, }: { readonly node: TreeNode; },
): { readonly isRead: boolean; readonly flagged: boolean; readonly folder: unknown; } {
  /**
   The array's elements.
   */
  const elements = Array.isArray(node.elements,) ? node.elements as readonly unknown[] : [];
  /**
   What the leading flags say: the position of the command, the folder and whether a `-C` flag is there.
   */
  const reading = elements.reduce(function flagged(
    passed: {
      readonly commandAt: number;
      readonly skip: number;
      readonly flagged: boolean;
      readonly folder: unknown;
    },
    element: unknown,
    index: number,
  ) {
    if (passed.commandAt >= 0)
      return passed;
    if (passed.skip > 0) {
      return {
        ...passed,
        skip: passed.skip - 1,
      };
    }
    /**
     The element's text.
     */
    const text = literalText({ node: element, },);
    if (text === '-C') {
      return {
        ...passed,
        skip: 1,
        flagged: true,
        folder: elements[index + 1],
      };
    }
    if (text === '-c') {
      return {
        ...passed,
        skip: 1,
      };
    }
    return text.startsWith('-',)
      ? passed
      : {
        ...passed,
        commandAt: index,
      };
  }, {
    commandAt: -1,
    skip: 0,
    flagged: false,
    folder: undefined,
  },);
  return {
    isRead: (reading.commandAt >= 0) && GIT_READS.has(literalText({ node: elements[reading.commandAt], },),),
    flagged: reading.flagged,
    folder: reading.folder,
  };
}

/**
 Whether an argument names git: the text `git`, a path ending in `/git`, or an identifier or call named for it.

 @param node - argument read

 @returns Whether git is named

 @example
 ```ts
 const names = mentionsGitIn({ node: argument, },); // true for REAL_GIT
 ```
 */
function mentionsGitIn({ node, }: { readonly node: unknown; }): boolean {
  if (!isTreeNode(node,))
    return false;
  if (node.type === 'Identifier')
    return namesGit({ name: identifierName({ node, },), },);
  if (node.type === 'Literal') {
    /**
     The literal's text.
     */
    const text = literalText({ node, },);
    return (text === 'git') || text.endsWith('/git',);
  }
  if (node.type === 'CallExpression')
    return namesGit({ name: calleeName({ node, },), },);
  if ((node.type !== 'ObjectExpression') || (!Array.isArray(node.properties,)))
    return false;
  return (node.properties as readonly unknown[]).some(function propertyMentions(property,): boolean {
    return isTreeNode(property,) && (identifierName({ node: property.key, },) === 'file')
      && mentionsGitIn({ node: property.value, },);
  },);
}

/**
 The folder option of a call's options object: `cwd`, `cloneDir`, `repository` or `directory`.

 @param node - call node

 @returns Option's value, none where the call has none

 @example
 ```ts
 const folder = optionFolderOf({ node: call, },);
 ```
 */
function optionFolderOf({ node, }: { readonly node: TreeNode; }): unknown {
  /**
   The properties of every options object the call is handed.
   */
  const properties = argumentsOf({ node, },).flatMap(function propertiesOf(argument,): readonly unknown[] {
    return (isTreeNode(argument,) && (argument.type === 'ObjectExpression') && Array.isArray(argument.properties,))
      ? argument.properties as readonly unknown[]
      : [];
  },);
  /**
   The property that names the folder.
   */
  const found = properties.find(function isFolder(property,): boolean {
    return isTreeNode(property,) && GIT_FOLDER_KEYS.has(identifierName({ node: property.key, },),);
  },);
  return isTreeNode(found,) ? found.value : undefined;
}

/**
 The git read a call makes through an array of arguments with no `-C` flag, when the call is git's.

 @param node - call node

 @returns Whether the call is a git read, and the folder option it names, none where it names none

 @example
 ```ts
 const { isGit, folder, } = gitCallOf({ node, },);
 ```
 */
function gitCallOf({ node, }: { readonly node: TreeNode; }): { readonly isGit: boolean; readonly folder: unknown; } {
  /**
   Every array the call is handed, directly or as an object's property.
   */
  const lists = argumentsOf({ node, },).flatMap(function listsOf(argument,): readonly unknown[] {
    if (!isTreeNode(argument,))
      return [];
    if (argument.type === 'ArrayExpression')
      return [argument,];
    if ((argument.type !== 'ObjectExpression') || (!Array.isArray(argument.properties,)))
      return [];
    return (argument.properties as readonly unknown[]).flatMap(function valueOf(property,): readonly unknown[] {
      return isTreeNode(property,) ? [property.value,] : [];
    },);
  },);
  /**
   Whether anything says the call is git's: a name of the callee that holds "git", or an argument that names git.
   */
  const isGitCall = namesGit({ name: calleeName({ node, },), }) || argumentsOf({ node, },)
    .some(function mentionsGit(argument,): boolean {
      return mentionsGitIn({ node: argument, },);
    },);
  /**
   Whether one of the arrays is a git read with no `-C` flag.
   */
  const reads = isGitCall && lists.some(function isGitRead(list,): boolean {
    if ((!isTreeNode(list,)) || (list.type !== 'ArrayExpression'))
      return false;
    /**
     What the list writes.
     */
    const { isRead, flagged, } = gitListOf({ node: list, },);
    return isRead && (!flagged);
  },);
  return {
    isGit: reads,
    folder: reads ? optionFolderOf({ node, },) : undefined,
  };
}

/**
 Whether a node loads a module: an import, a re-export, a dynamic import or a `require`.

 @param node - node read

 @returns Specifier it loads, empty for any other node

 @example
 ```ts
 const specifier = loadedSpecifier({ node, },); // '../../dog/src/bark.ts' for import { bark } from '../../dog/src/bark.ts'
 ```
 */
function loadedSpecifier({ node, }: { readonly node: TreeNode; }): string {
  if ((node.type === 'ImportDeclaration') || (node.type === 'ExportAllDeclaration')
    || (node.type === 'ExportNamedDeclaration') || (node.type === 'ImportExpression'))
    return literalText({ node: node.source, },);
  if ((node.type === 'CallExpression') && (identifierName({ node: node.callee, },) === 'require'))
    return literalText({ node: argumentsOf({ node, },)[0], },);
  return '';
}

/**
 The reads a file makes outside its package, in the order of its nodes.

 @param file - file read

 @returns One read per expression, a constant's chain at the constant and not at each place the constant is read

 @throws {@link Error} when the file does not parse

 @example
 ```ts
 const reads = packageReadsOf({ file, },);
 ```
 */
function packageReadsOf({ file, }: { readonly file: SourceText; }): readonly PackageRead[] {
  if ((!file.isTest) || (packageDirectoryOf({ path: file.path, },) === SCAN_PACKAGE))
    return [];
  /**
   Levels between the file's folder and the package folder, which the key's slashes count.
   */
  const depth = file.path.split('/',).length - 1;
  /**
   The parsed program and every node of it.
   */
  const { program, } = parseSource({ file, },);
  const nodes = nodesUnder({ root: program, },);
  const parents = parentsOf({ program, },);
  const held = heldChains({
    nodes,
    depth,
  },);
  /**
   Reads found, in node order.
   */
  const reads: PackageRead[] = [];
  /**
   Records a finding.

   @param node - node the read starts at

   @param kind - kind of read
   */
  const note = (node: TreeNode, kind: FindingKind,): void => {
    reads.push({
      path: file.path,
      line: lineNumberOf({ file, node, },),
      kind,
      shown: shownOf({ file, node, },),
    },);
  };
  for (const node of nodes.toSorted(function byStart(left, right,): number {
    return left.start - right.start;
  },)) {
    /**
     The module the node loads, a relative one read from the file's folder.
     */
    const specifier = loadedSpecifier({ node, },);
    if (specifier.startsWith('.',)) {
      if (afterText({
        reach: startAt({ level: depth, },),
        text: specifier,
      },).lowest
        < 0)
        note(
          node,
          'import',
        );
      continue;
    }
    if (node.type === 'ArrayExpression') {
      /**
       Whether the list is a git read with a `-C` flag, and the folder the flag names.
       */
      const { isRead, flagged, folder, } = gitListOf({ node, },);
      if (isRead && flagged && reachOf({
        node: folder,
        depth,
        held,
      },).isChain)
        note(
          node,
          'git-checkout',
        );
      continue;
    }
    if ((node.type === 'CallExpression') || (node.type === 'NewExpression')
      || (node.type === 'MemberExpression') || (node.type === 'TemplateLiteral') || (node.type === 'BinaryExpression')) {
      /**
       The chain the node is.
       */
      const reach = reachOf({
        node,
        depth,
        held,
      },);
      /**
       The node that holds this one, which reports the chain when it is one too.
       */
      const parent = parents.get(node,);
      const insideChain = (parent !== undefined) && reachOf({
        node: parent,
        depth,
        held,
      },).isChain;
      if (reach.isChain && (!insideChain) && (reach.lowest < 0) && (!reach.inherited))
        note(
          node,
          'climb',
        );
    }
    if (node.type !== 'CallExpression')
      continue;
    for (const kind of readKindsOf({
      node,
      depth,
      held,
    },))
      note(
        node,
        kind,
      );
    /**
     Whether the call is a git read without a `-C` flag, and the folder it runs in.
     */
    const git = gitCallOf({ node, },);
    if (!git.isGit)
      continue;
    if (git.folder === undefined) {
      note(
        node,
        'git-cwd',
      );
      continue;
    }
    if (reachOf({
      node: git.folder,
      depth,
      held,
    },).isChain)
      note(
        node,
        'git-checkout',
      );
  }
  return reads;
}

/**
 A read as the scan reports it.

 @param read - read found

 @returns `path:line kind: expression`

 @example
 ```ts
 const text = readLine({ read, },); // 'dog/nap.unit.test.ts:3 climb: join(import.meta.dirname, \'..\', \'..\')'
 ```
 */
function readLine({ read, }: { readonly read: PackageRead; }): string {
  return `${read.path}:${String(read.line,)} ${read.kind}: ${read.shown}`;
}

/**
 Reads found in files, as `path:line kind: expression`.

 @param files - files read

 @returns One line per read, in file order

 @example
 ```ts
 const lines = readLines({ files, },);
 ```
 */
function readLines({ files, }: { readonly files: readonly SourceText[]; }): readonly string[] {
  return files.flatMap(function linesOf(file,): readonly string[] {
    return packageReadsOf({ file, },).map(function lineText(read,): string {
      return readLine({ read, },);
    },);
  },);
}

/**
 A read the family's source holds that belongs to no package, and why it may stay.
 */
type BoundaryRead = {
  /**
   Location of the file.
   */
  readonly path: string;

  /**
   Kind of the read.
   */
  readonly kind: FindingKind;

  /**
   How many such reads the file holds.
   */
  readonly count: number;

  /**
   Why the read belongs to no package.
   */
  readonly reason: string;
};

/**
 The reads of files that belong to no package.
 */
const BOUNDARY_READS: readonly BoundaryRead[] = [
  {
    path: 'typography/semantic-wrap-binary.unit.test.ts',
    kind: 'climb',
    count: 1,
    reason: 'the case holds the version the package pins to the one the repository root\'s toolchain.lock.json records, '
      + 'and the lock is root configuration that belongs to no package',
  },
];

/**
 The reads that stand once the listed ones are taken out.

 @param files - files read

 @param listed - reads of files that belong to no package

 @returns Every read not listed, then a line for each listed read the files hold more or fewer times than listed

 @example
 ```ts
 const unlisted = unlistedReads({ files, listed: BOUNDARY_READS, },);
 ```
 */
function unlistedReads({ files, listed, }: { readonly files: readonly SourceText[]; readonly listed: readonly BoundaryRead[]; }): readonly string[] {
  /**
   The reads found.
   */
  const reads = files.flatMap(function readsOf(file,): readonly PackageRead[] {
    return packageReadsOf({ file, },);
  },);
  /**
   A line for each listed read held more or fewer times than listed.
   */
  const miscounted = listed.flatMap(function countOf({ path, kind, count, },): readonly string[] {
    /**
     How often the file holds that kind of read.
     */
    const held = reads.filter(function isListed(read,): boolean {
      return (read.path === path) && (read.kind === kind);
    },).length;
    return (held === count) ? [] : [`${path} ${kind}: listed ${String(count,)} and held ${String(held,)}`,];
  },);
  return [
    ...reads
      .filter(function stands(read,): boolean {
        return !listed.some(function covers({ path, kind, },): boolean {
          return (read.path === path) && (read.kind === kind);
        },);
      },)
      .map(function lineText(read,): string {
        return readLine({ read, },);
      },),
    ...miscounted,
  ];
}

//endregion Package reads

//region Fixture files
// Files of cats and dogs, each a read the scan must find or a control it must leave alone. A fixture text writes `@{`
// where the file it stands for writes the opening of a template placeholder, so no string of this file holds one.

/**
 A fixture file for the scan to read, with the kinds it must find.
 */
type ReadCase = {
  /**
   What the file does, which names the case.
   */
  readonly label: string;

  /**
   Location the file is read under; its slashes give the depth of its folder in the package.
   */
  readonly path: string;

  /**
   The file's text.
   */
  readonly text: string;

  /**
   The kinds of finding it must make, in node order.
   */
  readonly kinds: readonly FindingKind[];
};

/**
 Location of a file two folders under its package's `src` level, where `..` twice reaches the package folder.
 */
const DEEP = 'cat/corpus-run/nap.unit.test.ts';

/**
 Location of a file directly under its package's `src`, where `..` once reaches the package folder.
 */
const SHALLOW = 'cat/nap.unit.test.ts';

/**
 Location of a package source file, which is no test.
 */
const SOURCE_FILE = 'cat/nap.ts';

/**
 Label of the case whose constant holds the climb.
 */
const HELD_CLIMB = 'a constant that holds the climb, reported where it is held and not where it is read';

/**
 Label of the case whose glob pattern climbs.
 */
const GLOB_CLIMB = 'a glob whose pattern climbs past the package from the file\'s folder';

/**
 Reads that leave the package, one case for each form the census of ledger B1265 found and each form a rewrite of one
 could take.
 */
const OFFENDER_CASES: readonly ReadCase[] = [
  {
    label: 'a join of the file\'s folder with one more climb than the package folder allows',
    path: DEEP,
    text: 'export const dogs = join(import.meta.dirname, \'..\', \'..\', \'..\', \'dog\', \'src\',);\n',
    kinds: ['climb',],
  },
  {
    label: 'a resolve whose climbs are written in one string',
    path: SHALLOW,
    text: 'export const dogs = resolve(import.meta.dirname, \'../../dog\',);\n',
    kinds: ['climb',],
  },
  {
    label: 'a template literal that climbs after the file\'s folder',
    path: SHALLOW,
    text: 'export const dogs = `@{import.meta.dirname}/../../dog/src/bark.ts`;\n',
    kinds: ['climb',],
  },
  {
    label: 'a string joined to the file\'s folder with a plus',
    path: SHALLOW,
    text: 'export const dogs = import.meta.dirname + \'/../../dog\';\n',
    kinds: ['climb',],
  },
  {
    label: 'a URL resolved against the file\'s own URL that climbs past the package',
    path: DEEP,
    text: 'export const dogs = new URL(\'../../../dog/src/bark.ts\', import.meta.url,);\n',
    kinds: ['climb',],
  },
  {
    label: 'a path read off a URL that climbs past the package',
    path: SHALLOW,
    text: 'export const dogs = fileURLToPath(new URL(\'../../dog/src/bark.ts\', import.meta.url,),);\n',
    kinds: ['climb',],
  },
  {
    label: 'a URL whose relative text is a template that climbs past the package',
    path: SHALLOW,
    text: 'export const dogs = new URL(`../../@{name}/src/bark.ts`, import.meta.url,);\n',
    kinds: ['climb',],
  },
  {
    label: 'the folder of the file\'s own name, climbed',
    path: SHALLOW,
    text: 'export const dogs = join(dirname(fileURLToPath(import.meta.url,),), \'..\', \'..\', \'dog\',);\n',
    kinds: ['climb',],
  },
  {
    label: 'the file\'s own name climbed one more than its folder',
    path: SHALLOW,
    text: 'export const dogs = join(import.meta.filename, \'..\', \'..\', \'..\', \'dog\',);\n',
    kinds: ['climb',],
  },
  {
    label: 'the CommonJS folder name climbed',
    path: SHALLOW,
    text: 'export const dogs = join(__dirname, \'..\', \'..\', \'dog\',);\n',
    kinds: ['climb',],
  },
  {
    label: 'the working directory climbed',
    path: SHALLOW,
    text: 'export const dogs = join(process.cwd(), \'..\', \'dog\',);\n',
    kinds: ['climb',],
  },
  {
    label: 'import.meta.resolve of a relative specifier that climbs past the package',
    path: SHALLOW,
    text: 'export const dogs = import.meta.resolve(\'../../dog/src/bark.ts\',);\n',
    kinds: ['climb',],
  },
  {
    label: HELD_CLIMB,
    path: SHALLOW,
    text: [
      'const FAMILY_ROOT = join(import.meta.dirname, \'..\', \'..\',);',
      'export const dogs = async () => await readFile(join(FAMILY_ROOT, \'dog\', \'src\', \'bark.ts\',), \'utf8\',);',
      'export const cats = () => FAMILY_ROOT;',
    ].join('\n',),
    kinds: ['climb',],
  },
  {
    label: 'a constant that holds the file\'s folder and is climbed past the package where it is read',
    path: SHALLOW,
    text: [
      'const HERE = import.meta.dirname;',
      'export const dogs = join(HERE, \'..\', \'..\', \'dog\',);',
    ].join('\n',),
    kinds: ['climb',],
  },
  {
    label: 'a glob read from a folder that climbs past the package',
    path: SHALLOW,
    text: 'export const dogs = glob(\'*/src/**/*.ts\', { cwd: join(import.meta.dirname, \'..\', \'..\',), },);\n',
    kinds: ['climb',],
  },
  {
    label: GLOB_CLIMB,
    path: SHALLOW,
    text: 'export const dogs = glob(\'../../*/src/**/*.ts\', { cwd: import.meta.dirname, },);\n',
    kinds: ['glob',],
  },
  {
    label: 'a glob whose pattern climbs past the package from the working directory',
    path: SHALLOW,
    text: 'export const dogs = globSync([\'../*/src/**/*.ts\',],);\n',
    kinds: ['glob',],
  },
  {
    label: 'a static import of a file past the package folder',
    path: DEEP,
    text: 'import { bark, } from \'../../../dog/src/bark.ts\';\nexport const dogs = bark;\n',
    kinds: ['import',],
  },
  {
    label: 'a re-export and a dynamic import of a file past the package folder',
    path: SHALLOW,
    text: [
      'export { bark, } from \'../../dog/src/bark.ts\';',
      'export const loud = async () => await import(\'../../dog/src/howl.ts\');',
    ].join('\n',),
    kinds: ['import', 'import',],
  },
  {
    label: 'a require of a file past the package folder',
    path: SHALLOW,
    text: 'export const dogs = require(\'../../dog/src/bark.ts\');\n',
    kinds: ['import',],
  },
  {
    label: 'a read of a relative path written whole that climbs past the package',
    path: SHALLOW,
    text: 'export const dogs = async () => await readFile(\'../dog/src/bark.ts\', \'utf8\',);\n',
    kinds: ['literal-read',],
  },
  {
    label: 'a listing of a join of literals that climbs past the package',
    path: SHALLOW,
    text: 'export const dogs = async () => await fs.readdir(join(\'..\', \'dog\', \'src\',),);\n',
    kinds: ['literal-read',],
  },
  {
    label: 'a read of the family folder written whole',
    path: SHALLOW,
    text: 'export const dogs = () => readFileSync(\'package/translation-repair/dog/src/bark.ts\', \'utf8\',);\n',
    kinds: ['literal-read',],
  },
  {
    label: 'a listing of the family folder written in a template with no variable at its root',
    path: SHALLOW,
    text: 'export const dogs = async (name: string,) => await readdir(`package/translation-repair/@{name}/src`,);\n',
    kinds: ['literal-read',],
  },
  {
    label: 'a git read with no folder, which runs where the test runs',
    path: SHALLOW,
    text: 'export const dogs = async () => await spawn(\'git\', [\'ls-files\', \'--\', \'package/translation-repair/dog\',],);\n',
    kinds: ['git-cwd',],
  },
  {
    label: 'a git read through a helper named for git, with no folder',
    path: SHALLOW,
    text: 'export const dogs = async () => await fixtureGit({ args: [\'show\', \'HEAD:dog/src/bark.ts\',], },);\n',
    kinds: ['git-cwd',],
  },
  {
    label: 'a git read whose -C argument is the file\'s folder',
    path: SHALLOW,
    text: 'export const dogs = async () => await spawn(\'git\', [\'-C\', import.meta.dirname, \'log\', \'--\', \'.\',],);\n',
    kinds: ['git-checkout',],
  },
  {
    label: 'a git argument list whose -C argument is the working directory',
    path: SHALLOW,
    text: 'export const dogs = [\'-C\', process.cwd(), \'rev-list\', \'HEAD\',];\n',
    kinds: ['git-checkout',],
  },
  {
    label: 'a git read whose folder option climbs',
    path: SHALLOW,
    text: [
      'export const dogs = async () => await execFile(',
      '  \'git\', [\'diff\', \'--stat\',], { cwd: join(import.meta.dirname, \'..\', \'..\',), },',
      ');',
    ].join('\n',),
    kinds: ['git-checkout', 'climb',],
  },
  {
    label: 'a git read whose folder option is the file\'s folder, run through the real git binary',
    path: SHALLOW,
    text: 'export const dogs = async () => await run({ file: REAL_GIT, args: [\'rev-parse\', \'HEAD\',], cwd: import.meta.dirname, },);\n',
    kinds: ['git-checkout',],
  },
];

/**
 Files that read only what a test may: its own package, a folder it made, a corpus clone, a package it declares.
 */
const CONTROL_CASES: readonly ReadCase[] = [
  {
    label: 'a file beside the test',
    path: DEEP,
    text: 'export const own = join(import.meta.dirname, \'nap-cases.jsonc\',);\n',
    kinds: [],
  },
  {
    label: 'the package\'s built output, reached by climbing to the package folder and no further',
    path: DEEP,
    text: 'export const built = join(import.meta.dirname, \'..\', \'..\', \'dist\', \'final\', \'node\',);\n',
    kinds: [],
  },
  {
    label: 'the built output through a URL, from the file directly under src',
    path: SHALLOW,
    text: 'export const built = fileURLToPath(new URL(\'../dist/final/node/index.mjs\', import.meta.url,),);\n',
    kinds: [],
  },
  {
    label: 'a sibling file through a URL',
    path: SHALLOW,
    text: 'export const sibling = new URL(\'sibling.ts\', import.meta.url,);\n',
    kinds: [],
  },
  {
    label: 'the package\'s installed folder through a URL',
    path: SHALLOW,
    text: 'export const logs = fileURLToPath(new URL(\'../node_modules/.monochromatic\', import.meta.url,),);\n',
    kinds: [],
  },
  {
    label: 'the folder of the file\'s own name',
    path: DEEP,
    text: 'export const here = dirname(fileURLToPath(import.meta.url,),);\n',
    kinds: [],
  },
  {
    label: 'the working directory, joined below it',
    path: SHALLOW,
    text: 'export const here = join(process.cwd(), \'node_modules\',);\n',
    kinds: [],
  },
  {
    label: 'a path from the working directory to a folder the test made',
    path: SHALLOW,
    text: 'export const there = relative(process.cwd(), join(scratch.path, \'file\', \'below\',),);\n',
    kinds: [],
  },
  {
    label: 'a folder the test made, climbed and read',
    path: SHALLOW,
    text: 'export const up = async () => await readFile(join(scratch.path, \'..\', \'..\', \'nap.txt\',), \'utf8\',);\n',
    kinds: [],
  },
  {
    label: 'a fixture folder holding the family\'s folder names as its own files',
    path: SHALLOW,
    text: [
      'export const inside = async () => await readFile(',
      '  join(scratch.path, \'package/translation-repair/dog/src/bark.ts\',), \'utf8\',',
      ');',
      'export const listed = async () => await readdir(`@{scratch.path}/package/translation-repair`,);',
    ].join('\n',),
    kinds: [],
  },
  {
    label: 'the family folder as data a tool is handed',
    path: SHALLOW,
    text: [
      'const FAMILY = \'package/translation-repair\';',
      'export const sources = auditedSourcesOf({ sources: `@{FAMILY}/*/src`, },);',
      '// readFile(\'package/translation-repair/dog\') is only a comment',
    ].join('\n',),
    kinds: [],
  },
  {
    label: 'a git read in a fixture repository, through a helper that takes the clone',
    path: SHALLOW,
    text: 'export const log = async () => await fixtureGit({ cloneDir: repo.path, args: [\'log\', \'--\', \'package/translation-repair/dog\',], },);\n',
    kinds: [],
  },
  {
    label: 'a git read of a corpus clone with its path at the -C argument',
    path: SHALLOW,
    text: 'export const show = async () => await spawn(REAL_GIT, [\'-C\', archive.pin.cloneDir, \'show\', `@{sha}:page.md`,],);\n',
    kinds: [],
  },
  {
    label: 'a git read handed the clone beside its arguments, as the cache account tests do',
    path: SHALLOW,
    text: 'export const out = async () => await cacheAccountGitOutput({ args: [\'-C\', repo.path, \'log\', \'--format=%H\',], },);\n',
    kinds: [],
  },
  {
    label: 'a git command that writes, run where the test runs',
    path: SHALLOW,
    text: 'export const init = async () => await spawn(\'git\', [\'init\', \'--quiet\',],);\n',
    kinds: [],
  },
  {
    label: 'names of git commands in a list that is no git call',
    path: SHALLOW,
    text: 'export const verbs = [\'log\', \'show\', \'diff\',];\n',
    kinds: [],
  },
  {
    label: 'a package imported by name, through its API and by the built entry it declares',
    path: DEEP,
    text: [
      'import { bark, } from \'@monochromatic-dev/translation-repair-dog/ts/bark.ts\';',
      'import { howl, } from \'../../dist/final/node/index.mjs\';',
      'import { sit, } from \'./sit.test-fixture.ts\';',
      'export const entry = import.meta.resolve(\'@monochromatic-dev/translation-repair-dog\',);',
      'export const built = builtDirectoryOf({ packageEntry: import.meta.resolve(\'@monochromatic-dev/translation-repair-dog\',), },);',
      'export const dogs = [bark, howl, sit,];',
    ].join('\n',),
    kinds: [],
  },
  {
    label: 'a climb in a package source file, which is no test',
    path: SOURCE_FILE,
    text: 'export const dogs = join(import.meta.dirname, \'..\', \'..\', \'dog\',);\n',
    kinds: [],
  },
  {
    label: 'a climb and a git read in a test of the source-scan package, whose input is the whole family',
    path: 'source-scan/nap.unit.test.ts',
    text: 'export const dogs = join(import.meta.dirname, \'..\', \'..\', \'dog\',);\nexport const git = spawn(\'git\', [\'ls-files\',],);\n',
    kinds: [],
  },
];

/**
 The character a template placeholder opens with.
 */
const DOLLAR = String.fromCodePoint(0x24,);

/**
 The file a case describes, a test or a fixture by its location.

 @param readCase - case read

 @returns File as the scan reads it

 @example
 ```ts
 const file = fileOf({ readCase: OFFENDER_CASES[0], },);
 ```
 */
function fileOf({ readCase, }: { readonly readCase: ReadCase; }): SourceText {
  return {
    path: readCase.path,
    text: readCase.text.replaceAll(
      '@{',
      `${DOLLAR}{`,
    ),
    isTest: readCase.path.endsWith('.test.ts',) || readCase.path.endsWith('.test-fixture.ts',),
  };
}

/**
 The kinds a case's file earns, as the scan finds them.

 @param readCase - case read

 @returns Kinds in node order

 @example
 ```ts
 const kinds = kindsFound({ readCase: OFFENDER_CASES[0], },); // ['climb']
 ```
 */
function kindsFound({ readCase, }: { readonly readCase: ReadCase; }): readonly string[] {
  return packageReadsOf({ file: fileOf({ readCase, },), },).map(function kindOf({ kind, },): string {
    return kind;
  },);
}

/**
 The case with a label.

 @param label - label of the case

 @returns Case

 @throws {@link Error} when no offender case carries the label

 @example
 ```ts
 const readCase = offenderLabelled({ label: GLOB_CLIMB, },);
 ```
 */
function offenderLabelled({ label, }: { readonly label: string; }): ReadCase {
  return nonNullishOrThrow(OFFENDER_CASES.find(function carries(readCase,): boolean {
    return readCase.label === label;
  },),);
}

/**
 The kinds each case in a list earns, by the case's label.

 @param cases - cases read

 @returns Kinds the scan finds for each label

 @example
 ```ts
 const found = foundByLabel({ cases: CONTROL_CASES, },);
 ```
 */
function foundByLabel({ cases, }: { readonly cases: readonly ReadCase[]; }): Readonly<Record<string, readonly string[]>> {
  return Object.fromEntries(cases.map(function foundOf(readCase,): readonly [string, readonly string[]] {
    return [readCase.label, kindsFound({ readCase, },),];
  },),);
}

/**
 The kinds each case in a list must earn, by the case's label.

 @param cases - cases read

 @returns Kinds each case states

 @example
 ```ts
 const listed = listedByLabel({ cases: CONTROL_CASES, },);
 ```
 */
function listedByLabel({ cases, }: { readonly cases: readonly ReadCase[]; }): Readonly<Record<string, readonly string[]>> {
  return Object.fromEntries(cases.map(function listedOf(readCase,): readonly [string, readonly string[]] {
    return [readCase.label, readCase.kinds,];
  },),);
}

//endregion Fixture files

await describe({
  name: 'tests read no package they do not declare',
  children: [
    it({
      name: 'FINDS EACH WAY A TEST OR FIXTURE LEAVES ITS PACKAGE FOLDER, a climb written as a join, a resolve, a template, '
        + 'a plus, a URL, a folder name, a held constant, a glob pattern, an import, a require, a path written whole and '
        + 'a git read of the live checkout',
      fn: async () => {
        expectRecordAsListed({
          found: foundByLabel({ cases: OFFENDER_CASES, },),
          listed: listedByLabel({ cases: OFFENDER_CASES, },),
        },);
      },
    },),
    it({
      name: 'LEAVES ALONE THE PACKAGE\'S OWN FOLDER, A FOLDER THE TEST MADE, A CORPUS CLONE, A PACKAGE IT DECLARES, and a '
        + 'family folder written as data, in tests, fixtures and a test of this package',
      fn: async () => {
        expectRecordAsListed({
          found: foundByLabel({ cases: CONTROL_CASES, },),
          listed: listedByLabel({ cases: CONTROL_CASES, },),
        },);
      },
    },),
    it({
      name: 'NAMES THE FILE, THE LINE, THE KIND AND THE EXPRESSION of each read it finds, once for a constant that holds a '
        + 'climb',
      fn: async () => {
        expectFindingsAsListed({
          findings: readLines({
            files: [
              fileOf({ readCase: offenderLabelled({ label: HELD_CLIMB, },), },),
              fileOf({ readCase: offenderLabelled({ label: GLOB_CLIMB, },), },),
            ],
          },),
          listed: [
            'cat/nap.unit.test.ts:1 climb: join(import.meta.dirname, \'..\', \'..\',)',
            'cat/nap.unit.test.ts:1 glob: glob(\'../../*/src/**/*.ts\', { cwd: import.meta.dirname, },)',
          ],
        },);
      },
    },),
    it({
      name: 'LISTS A READ OF A FILE THAT BELONGS TO NO PACKAGE and fails for a listed read the source holds more or fewer '
        + 'times than listed, so the list cannot outlive its reason',
      fn: async () => {
        /**
         A file with the one climb a listed entry covers.
         */
        const holding: SourceText = {
          path: SHALLOW,
          text: 'export const lock = join(import.meta.dirname, \'..\', \'..\', \'toolchain.lock.json\',);\n',
          isTest: true,
        };
        /**
         The entry that covers it.
         */
        const entry: BoundaryRead = {
          path: SHALLOW,
          kind: 'climb',
          count: 1,
          reason: 'the lock belongs to no package',
        };

        expectFindingsAsListed({
          findings: [
            ...unlistedReads({ files: [holding,], listed: [entry,], },),
            ...unlistedReads({ files: [], listed: [entry,], },),
            ...unlistedReads({ files: [holding,], listed: [], },),
            ...unlistedReads({ files: [holding,], listed: [{ ...entry, count: 2, },], },),
          ],
          listed: [
            'cat/nap.unit.test.ts climb: listed 1 and held 0',
            'cat/nap.unit.test.ts:1 climb: join(import.meta.dirname, \'..\', \'..\', \'toolchain.lock.json\',)',
            'cat/nap.unit.test.ts climb: listed 2 and held 1',
          ],
        },);
      },
    },),
    it({
      name: 'FINDS NO TEST OR FIXTURE OF THE FAMILY THAT READS, LISTS OR RUNS GIT OVER THE FILES OF A PACKAGE IT DOES NOT '
        + 'DECLARE, outside the listed reads of files that belong to no package',
      fn: async () => {
        /**
         Every package file, source among them to be skipped.
         */
        const files = await readPackageSource();
        expectNoFindings({ findings: unlistedReads({ files, listed: BOUNDARY_READS, },), },);
        expect(BOUNDARY_READS.every(function holdsReason({ reason, },): boolean {
          return reason !== '';
        },),).toBe(true,);
      },
    },),
  ],
},);
