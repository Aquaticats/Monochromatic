/**
 Guards against a test that writes a process global while module-test runs
 other cases beside it (ledger B79, M100). A suite runs its children sixteen
 at a time unless told otherwise, so a case that sets an environment
 variable, diverts `console.log` or sets `process.exitCode` across an await
 lets the cases beside it read its value, and restores that finish out of
 order leave one case's value behind for the rest of the file. A write is
 allowed only in a case every suite around which runs one case at a time
 (`concurrency: 1` on it or on a suite that holds it, all the way out to the
 suite awaited at the file's top).

 WHAT THE SCAN READS, in tests and test fixtures: an assignment (`=`, `??=`,
 `||=` and the rest) or a `delete` whose target is a member chain on
 `process`, `console`, `globalThis`, `Date`, `Math`, `performance`, `Intl`,
 `JSON`, `crypto` or `Symbol`; `Reflect.deleteProperty`, `Reflect.set`,
 `Reflect.defineProperty`, `Object.defineProperty` and `Object.assign` whose
 first argument is one of those or a chain on one; and `process.chdir` and
 `process.umask` called with an argument. A write inside a function declared
 by name is placed at every call of that name in its own file, and at every
 call in another test file that imports it by name from that file, under
 whatever name the import binds (ledger B120), followed out until a case or
 a file's top; a write inside a method or a callback is placed where that
 function was made. A case's stub through its own sandbox, `ctx.sinon`, is no
 write here, since module-test answers it to that case alone.

 A WRITE TO `Symbol` IS THE SLOT OF THE PARSE-END HOOK of the grammar
 (`parse-end-hook.ts` of document-structure), which tests of three packages
 set and remove with `Reflect.set` and `Reflect.deleteProperty`. It is the
 one global allowed beside other cases, and only where it is set and disposed
 synchronously: the helper that sets it is called as the initializer of a
 `using` declaration in a function that holds no `await`, so the slot is
 back before any other case runs. The same call in a function that awaits, a
 call that is not a `using` initializer and an assignment to a member of
 `Symbol` are reported as any other global's write is.

 OUT OF THE SCAN'S REACH: a write through an alias (`const env =
 process.env`), a global shadowed by a local of the same name, a case
 made by a function the suite's text only calls, and a writer reached
 through a namespace import or a re-export (the package's tests use
 neither; `dead-test-declarations.unit.test.ts` fails a namespace import). A writer that no call by
 name reaches, such as one kept in a table and called through it, is
 reported rather than passed.

 THE FIXTURE CASE COMES FIRST, so the package-wide case is read against a
 scan shown able to find each form and each placement (ledger M21).
 Fixtures are cat-themed; the package case reads this package's own tests.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  declaringModule,
  namesFamilyModule,
  readPackageSource,
} from './family-source.test-fixture.ts';
import {
  ancestorsOf,
  childNodes,
  identifierName,
  isTreeNode,
  literalText,
  parentsOf,
  parseSource,
  type SourceText,
  type TreeNode,
  unwrapped,
} from './source-scan.test-fixture.ts';
import { expectNoFindings, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scan-findings.test-fixture.ts';

//region Global writes
// The scan: which nodes write a process global, which case each write runs
// in, and whether every suite around that case runs one case at a time.

/**
 Globals a test can write that every case in the file shares.
 */
const GLOBAL_ROOTS: ReadonlySet<string> = new Set([
  'process',
  'console',
  'globalThis',
  'Date',
  'Math',
  'performance',
  'Intl',
  'JSON',
  'crypto',
  'Symbol',
],);

/**
 Calls that write through their first argument.
 */
const WRITING_CALLS: ReadonlySet<string> = new Set([
  'Reflect.deleteProperty',
  'Reflect.set',
  'Reflect.defineProperty',
  'Object.defineProperty',
  'Object.assign',
],);

/**
 `process` methods that set process-wide state when given an argument.
 */
const PROCESS_SETTERS: ReadonlySet<string> = new Set([
  'chdir',
  'umask',
],);

/**
 Test fixtures that are a process's own entry, each as the scan names its path: nothing imports one and no case runs
 beside it, so the exit code it leaves is the process's alone and no other case can read it. A write outside any case is
 allowed in these and nowhere else; adding one is a reviewed decision, and the package case fails for one that is gone.
 */
const PROCESS_ENTRIES: readonly string[] = [
  'source-scan/rotation-sweep/rotation-sweep-cli.test-fixture.ts',
];

/**
 Node kinds that open a function.
 */
const FUNCTION_KINDS: ReadonlySet<string> = new Set([
  'ArrowFunctionExpression',
  'FunctionDeclaration',
  'FunctionExpression',
],);

/**
 Suites run this many cases at once when nothing says otherwise
 (`DEFAULT_CONCURRENCY` in module-test).
 */
const DEFAULT_CONCURRENCY = 16;

/**
 The global an expression is or hangs off, through members, parentheses and
 casts.

 @param node - expression read

 @param bare - whether the global itself counts, as a call's first argument
 does, or only a member of it, as an assignment's target does

 @returns Global's name, empty for anything else

 @example
 ```ts
 const root = globalOf({ node: assignment.left, bare: false, },); // 'process' for process.env.CAT
 ```
 */
function globalOf(
  {
    node,
    bare,
  }: {
    readonly node: unknown;
    readonly bare: boolean;
  },
): string {
  // WALKED OUT TO THE ROOT, counting the members passed on the way.
  for (let at: unknown = unwrapped({ node, },).inner, members = 0; ; members += 1) {
    if ((!isTreeNode(at,)) || (at.type !== 'MemberExpression')) {
      /**
       The root's name.
       */
      const name = identifierName({ node: at, },);
      return (GLOBAL_ROOTS.has(name,) && ((members > 0) || bare)) ? name : '';
    }
    at = unwrapped({ node: at.object, },).inner;
  }
}

/**
 Dotted name of a call's callee when it is a member of a name, as
 `Reflect.set`.

 @param callee - callee read

 @returns Dotted name, empty for any other callee

 @example
 ```ts
 const called = dottedCallee({ callee: call.callee, },);
 ```
 */
function dottedCallee({ callee, }: { readonly callee: unknown; },): string {
  /**
   The callee inside any wrappers.
   */
  const { inner, } = unwrapped({ node: callee, },);
  if ((!isTreeNode(inner,)) || (inner.type !== 'MemberExpression') || (inner.computed === true))
    return '';
  /**
   Object and member names.
   */
  const [object, member,] = [
    identifierName({ node: inner.object, },),
    identifierName({ node: inner.property, },),
  ];
  return ((object === '') || (member === '')) ? '' : `${object}.${member}`;
}

/**
 The write a node makes to a process global.

 @param node - node read

 @returns What kind of write it is, empty for none

 @example
 ```ts
 const form = writeOf({ node, },); // 'process assigned' for process.env.CAT = 'tabby'
 ```
 */
function writeOf({ node, }: { readonly node: TreeNode; },): string {
  if (node.type === 'AssignmentExpression') {
    /**
     Global the target hangs off.
     */
    const root = globalOf({ node: node.left, bare: false, },);
    return (root === '') ? '' : `${root} assigned`;
  }
  if ((node.type === 'UnaryExpression') && (node.operator === 'delete')) {
    /**
     Global the deleted member hangs off.
     */
    const root = globalOf({ node: node.argument, bare: false, },);
    return (root === '') ? '' : `${root} deleted`;
  }
  if (node.type !== 'CallExpression')
    return '';
  /**
   The call's callee as a dotted name, and its arguments.
   */
  const [called, args,] = [dottedCallee({ callee: node.callee, },), Array.isArray(node.arguments,) ? node.arguments : [],];
  if (WRITING_CALLS.has(called,) && (globalOf({ node: args[0], bare: true, },) !== ''))
    return called;
  return (called.startsWith('process.',) && PROCESS_SETTERS.has(called.slice('process.'.length,),) && (args.length > 0))
    ? called
    : '';
}

/**
 The global a write goes through.

 @param node - node read

 @returns Global's name, empty for a node that writes none

 @example
 ```ts
 const root = writtenGlobalOf({ node, },); // 'Symbol' for Reflect.set(Symbol, key, value)
 ```
 */
function writtenGlobalOf({ node, }: { readonly node: TreeNode; },): string {
  if (node.type === 'AssignmentExpression')
    return globalOf({ node: node.left, bare: false, },);
  if ((node.type === 'UnaryExpression') && (node.operator === 'delete'))
    return globalOf({ node: node.argument, bare: false, },);
  if (node.type !== 'CallExpression')
    return '';
  /**
   The call's callee as a dotted name, and its arguments.
   */
  const [called, args,] = [dottedCallee({ callee: node.callee, },), Array.isArray(node.arguments,) ? node.arguments : [],];
  if (WRITING_CALLS.has(called,))
    return globalOf({ node: args[0], bare: true, },);
  return called.startsWith('process.',) ? 'process' : '';
}

/**
 Whether a node holds an `await`, in its own text or in a function inside it.

 @param node - node read

 @returns Whether it does

 @example
 ```ts
 const awaits = containsAwait({ node: fn, },);
 ```
 */
function containsAwait({ node, }: { readonly node: TreeNode; },): boolean {
  /**
   Nodes still to read.
   */
  const pending: TreeNode[] = [node,];
  for (let at = pending.pop(); at !== undefined; at = pending.pop()) {
    if (at.type === 'AwaitExpression')
      return true;
    pending.push(...childNodes({ node: at, },),);
  }
  return false;
}

/**
 Whether a call sets a slot that is disposed before the function holding it can yield: the call initializes a `using`
 declaration, and the function holds no `await`.

 @param node - call read

 @param fn - function that holds it

 @param parents - each node's parent

 @returns Whether the set and the disposal are synchronous

 @example
 ```ts
 const scoped = setsAndDisposesSynchronously({ node: call, fn, parents, },);
 ```
 */
function setsAndDisposesSynchronously(
  {
    node,
    fn,
    parents,
  }: {
    readonly node: TreeNode;
    readonly fn: TreeNode;
    readonly parents: ReadonlyMap<TreeNode, TreeNode>;
  },
): boolean {
  /**
   The declarator the call initializes, and the declaration around it.
   */
  const [declarator, declaration,] = ancestorsOf({ node, parents, },);
  return (declarator?.type === 'VariableDeclarator')
    && (declarator.init === node)
    && (declaration?.type === 'VariableDeclaration')
    && (declaration.kind === 'using')
    && (!containsAwait({ node: fn, },));
}

/**
 Whether a call is to a test helper of the given name.

 @param node - node read

 @param name - helper's name

 @returns Whether it calls that name directly

 @example
 ```ts
 const isSuite = callsNamed({ node, name: 'describe', },);
 ```
 */
function callsNamed(
  {
    node,
    name,
  }: {
    readonly node: TreeNode;
    readonly name: string;
  },
): boolean {
  return (node.type === 'CallExpression') && (identifierName({ node: node.callee, },) === name);
}

/**
 The concurrency a suite runs its cases at, given the one around it.

 @param suite - `describe` call read

 @param outer - concurrency of the suite holding it, the default at the top

 @returns Its own `concurrency` where written as a number, not a number where
 written as anything else, and the outer one where not written

 @example
 ```ts
 const effective = concurrencyOf({ suite, outer: DEFAULT_CONCURRENCY, },);
 ```
 */
function concurrencyOf(
  {
    suite,
    outer,
  }: {
    readonly suite: TreeNode;
    readonly outer: number;
  },
): number {
  /**
   The suite's options.
   */
  const options = Array.isArray(suite.arguments,) ? unwrapped({ node: suite.arguments[0], },).inner : undefined;
  if ((!isTreeNode(options,)) || (!Array.isArray(options.properties,)))
    return Number.NaN;
  /**
   Its `concurrency` option, where written.
   */
  const written = options.properties
    .filter(isTreeNode,)
    .find(function isConcurrency(property,): boolean {
      return (property.type === 'Property') && (identifierName({ node: property.key, },) === 'concurrency');
    },);
  if (written === undefined)
    return outer;
  /**
   The option's value.
   */
  const value = unwrapped({ node: written.value, },).inner;
  return (isTreeNode(value,) && (value.type === 'Literal') && ((typeof value.value) === 'number'))
    ? value.value as number
    : Number.NaN;
}

/**
 Why a case does not run alone, read off the suites around it.

 @param caseCall - `it` call read

 @param parents - each node's parent

 @returns Empty when every suite out to the one awaited at the file's top
 runs one case at a time, else why not

 @example
 ```ts
 const fault = suiteFault({ caseCall, parents, },);
 ```
 */
function suiteFault(
  {
    caseCall,
    parents,
  }: {
    readonly caseCall: TreeNode;
    readonly parents: ReadonlyMap<TreeNode, TreeNode>;
  },
): string {
  /**
   The suites around the case, outermost first.
   */
  const suites = ancestorsOf({ node: caseCall, parents, },)
    .filter(function isSuite(node,): boolean {
      return callsNamed({ node, name: 'describe', },);
    },)
    .toReversed();
  /**
   Outermost suite, and what holds it out to the program.
   */
  const [outermost,] = suites;
  if (outermost === undefined)
    return 'in a case no suite in its text holds';
  /**
   Nodes between the outermost suite and the program.
   */
  const [awaited, ...statement] = ancestorsOf({ node: outermost, parents, },);
  if ((awaited?.type !== 'AwaitExpression') || (!statement.every(function isTopStatement(node,): boolean {
    return [
      'ExpressionStatement',
      'VariableDeclarator',
      'VariableDeclaration',
      'Program',
    ].includes(node.type,);
  },)))
    return 'in a suite not awaited at its file\'s top';
  /**
   Concurrency in force at the level read so far, from the top.
   */
  const level = { effective: DEFAULT_CONCURRENCY, };
  for (const suite of suites) {
    level.effective = concurrencyOf({ suite, outer: level.effective, },);
    if (level.effective !== 1)
      return 'in a suite that runs cases beside it';
  }
  return '';
}

/**
 Name a function is declared and called by, empty for a method, a callback
 or a function kept as a value.

 @param fn - function read

 @param parents - each node's parent

 @returns Its name

 @example
 ```ts
 const name = declaredName({ fn, parents, },);
 ```
 */
function declaredName(
  {
    fn,
    parents,
  }: {
    readonly fn: TreeNode;
    readonly parents: ReadonlyMap<TreeNode, TreeNode>;
  },
): string {
  if (fn.type === 'FunctionDeclaration')
    return identifierName({ node: fn.id, },);
  /**
   What holds the function.
   */
  const holder = parents.get(fn,);
  return (holder?.type === 'VariableDeclarator') ? identifierName({ node: holder.id, },) : '';
}

/**
 A name one file imports by name from a relative path or from a family package.
 */
type ImportedName = {
  /**
   Name the import binds in the importing file.
   */
  readonly local: string;

  /**
   Location of the file that declares it, as `readPackageSource` names files.
   */
  readonly from: string;

  /**
   Name that file exports it under.
   */
  readonly imported: string;
};

/**
 One test file as the scan follows writes through it.
 */
type FileSyntax = {
  /**
   File read.
   */
  readonly file: SourceText;

  /**
   Each node's parent.
   */
  readonly parents: ReadonlyMap<TreeNode, TreeNode>;

  /**
   Calls of a plain name, by the name called.
   */
  readonly calls: ReadonlyMap<string, readonly TreeNode[]>;

  /**
   Names the file imports by name from a relative path or from a family package.
   */
  readonly imports: readonly ImportedName[];
};

/**
 A place a write is followed out of.
 */
type Place = {
  /**
   File holding it.
   */
  readonly syntax: FileSyntax;

  /**
   Node there.
   */
  readonly node: TreeNode;
};

/**
 One test file's parents, calls by name and imports by name.

 @param file - file read

 @returns Its syntax as the scan follows writes through it

 @example
 ```ts
 const syntax = syntaxOf({ file, },);
 ```
 */
function syntaxOf({ file, }: { readonly file: SourceText; },): FileSyntax {
  /**
   The file's program and each node's parent there.
   */
  const { program, } = parseSource({ file, },);
  const parents = parentsOf({ program, },);
  /**
   Calls in this file, by the name called.
   */
  const calls = new Map<string, TreeNode[]>();
  for (const node of parents.keys()) {
    /**
     Name called, for a call of a plain name.
     */
    const name = (node.type === 'CallExpression') ? identifierName({ node: node.callee, },) : '';
    if (name === '')
      continue;
    if (!calls.has(name,))
      calls.set(name, [],);
    calls.get(name,)
      ?.push(node,);
  }
  return {
    file,
    parents,
    calls,
    imports: childNodes({ node: program, },)
      .filter(function importsFamilyFile(statement,): boolean {
        return (statement.type === 'ImportDeclaration')
          && namesFamilyModule({ specifier: literalText({ node: statement.source, },), },);
      },)
      .flatMap(function named(statement,): readonly ImportedName[] {
        return childNodes({ node: statement, },)
          .filter(function isNamed(specifier,): boolean {
            return specifier.type === 'ImportSpecifier';
          },)
          .map(function imported(specifier,): ImportedName {
            /**
             Name taken from the module the statement names.
             */
            const taken = identifierName({ node: specifier.imported, },);
            return {
              local: identifierName({ node: specifier.local, },),
              from: declaringModule({
                fromPath: file.path,
                specifier: literalText({ node: statement.source, },),
                imported: taken,
              },),
              imported: taken,
            };
          },);
      },),
  };
}

/**
 Every call of a function declared by name: in its own file under its name,
 and in each test file importing it by name from that file under the name
 the import binds.

 @param name - name the function is declared by

 @param syntax - file declaring it

 @param all - every test file read

 @returns Calls, each with its file

 @example
 ```ts
 const callers = callersOf({ name, syntax, all, },);
 ```
 */
function callersOf(
  {
    name,
    syntax,
    all,
  }: {
    readonly name: string;
    readonly syntax: FileSyntax;
    readonly all: readonly FileSyntax[];
  },
): readonly Place[] {
  return all.flatMap(function callsIn(other,): readonly Place[] {
    /**
     Names the function is called by in this file.
     */
    const locals = (other === syntax)
      ? [name,]
      : other.imports
        .filter(function fromDeclarer(entry,): boolean {
          return (entry.from === syntax.file.path) && (entry.imported === name);
        },)
        .map(function local(entry,): string {
          return entry.local;
        },);
    return locals.flatMap(function placesOf(local,): readonly Place[] {
      return (other.calls.get(local,) ?? []).map(function place(node,): Place {
        return {
          syntax: other,
          node,
        };
      },);
    },);
  },);
}

/**
 Why a write is not confined to cases that run alone, along every path out.

 @param write - write read

 @param syntax - file holding it

 @param all - every test file read, for calls of a writer other files import

 @returns One reason per path out that does not end in a case run alone

 @example
 ```ts
 const reasons = unconfined({ write, syntax, all, },);
 ```
 */
function unconfined(
  {
    write,
    syntax,
    all,
  }: {
    readonly write: TreeNode;
    readonly syntax: FileSyntax;
    readonly all: readonly FileSyntax[];
  },
): readonly string[] {
  /**
   The global the write goes through.
   */
  const root = writtenGlobalOf({ node: write, },);
  /**
   Reasons found so far.
   */
  const reasons: string[] = [];
  /**
   Functions already followed out of.
   */
  const followed = new Set<TreeNode>();
  /**
   Places still to follow out of: the write, then calls and made functions.
   */
  const pending: Place[] = [
    {
      syntax,
      node: write,
    },
  ];
  for (let at = pending.pop(); at !== undefined; at = pending.pop()) {
    /**
     Each node's parent in the file holding this place.
     */
    const { parents, } = at.syntax;
    /**
     The function holding this place, if any.
     */
    const fn = ancestorsOf({ node: at.node, parents, },).find(function opensFunction(node,): boolean {
      return FUNCTION_KINDS.has(node.type,);
    },);
    if (fn === undefined) {
      reasons.push((at.node === write) ? 'outside any case' : 'in a function no case reaches',);
      continue;
    }
    // The slot of the parse-end hook, set and disposed under `using` in a function that cannot yield, is back before any
    // other case runs.
    if ((root === 'Symbol') && setsAndDisposesSynchronously({
      node: at.node,
      fn,
      parents,
    },))
      continue;
    if (followed.has(fn,))
      continue;
    followed.add(fn,);
    /**
     What holds the function: a case's options when it is a case's body.
     */
    const [property, options, caseCall,] = ancestorsOf({ node: fn, parents, },);
    if ((property?.type === 'Property') && (identifierName({ node: property.key, },) === 'fn')
      && (caseCall !== undefined) && callsNamed({ node: caseCall, name: 'it', },)
      && Array.isArray(caseCall.arguments,) && (caseCall.arguments[0] === options)) {
      /**
       Why this case does not run alone.
       */
      const fault = suiteFault({ caseCall, parents, },);
      if (fault !== '')
        reasons.push(fault,);
      continue;
    }
    /**
     Name the function is called by.
     */
    const name = declaredName({ fn, parents, },);
    if (name === '') {
      pending.push({
        syntax: at.syntax,
        node: fn,
      },);
      continue;
    }
    /**
     Calls of that name, in its file and in the files importing it.
     */
    const callers = callersOf({
      name,
      syntax: at.syntax,
      all,
    },);
    if (callers.length === 0)
      reasons.push(`in ${name}, which no call by name reaches`,);
    pending.push(...callers,);
  }
  return reasons;
}

/**
 Every write to a process global in the tests given that is not confined to
 cases run one at a time, keyed `path: form, reason`.

 @param files - files read; package source is skipped

 @returns Keys sorted, one per write and path out

 @example
 ```ts
 const writes = unconfinedWrites({ files, },);
 ```
 */
function unconfinedWrites({ files, }: { readonly files: readonly SourceText[]; },): readonly string[] {
  /**
   Keys found so far.
   */
  const found: string[] = [];
  /**
   Every test file's syntax, read once.
   */
  const all = files
    .filter(function testFile({ isTest, },): boolean {
      return isTest;
    },)
    .map(function read(file,): FileSyntax {
      return syntaxOf({ file, },);
    },);
  for (const syntax of all) {
    for (const write of syntax.parents.keys()) {
      /**
       The write this node makes, if any.
       */
      const form = writeOf({ node: write, },);
      if (form !== '') {
        found.push(...unconfined({ write, syntax, all, },)
          .filter(function notAnEntrysOwn(reason,): boolean {
            return (!PROCESS_ENTRIES.includes(syntax.file.path,)) || (reason !== 'outside any case');
          },)
          .map(function keyed(reason,): string {
            return `${syntax.file.path}: ${form}, ${reason}`;
          },),);
      }
    }
  }
  return found.toSorted();
}

//endregion Global writes

await describe({
  name: 'process globals written only in cases run one at a time (ledger B79)',
  children: [
    it({
      name: 'FINDS each form written in a case run beside others, through helpers called by name and the '
        + 'functions a write is made in, in a suite run alone inside one that is not, outside any case, in a '
        + 'writer no call reaches and in a suite not awaited, and leaves sequenced cases, sandbox stubs, text '
        + 'a child process runs and package source',
      fn: async () => {
        expect(unconfinedWrites({
          files: [
            {
              path: 'litter.unit.test.ts',
              isTest: true,
              text: [
                'await describe({ name: \'litter\', concurrency: 1, children: [',
                '  it({ name: \'sets\', fn: async () => { process.env.CAT = \'tabby\'; await nap(); }, },),',
                '  it({ name: \'holds\', fn: async () => { using held = heldDial(); await nap(); }, },),',
                '], },);',
                'function heldDial() {',
                '  const before = process.env.CAT;',
                '  Reflect.deleteProperty(',
                '    process.env,',
                '    \'CAT\',',
                '  );',
                '  return { [Symbol.dispose]() { process.env.CAT = before; }, };',
                '}',
              ].join('\n',),
            },
            {
              path: 'catnip.unit.test.ts',
              isTest: true,
              text: [
                'await describe({ name: \'catnip\', children: [',
                '  it({ name: \'diverts\', fn: async () => { console.log = () => {}; }, },),',
                '  it({ name: \'chains\', fn: async () => { outerHelper(); }, },),',
                '  it({ name: \'sandboxed\', fn: async (ctx) => { ctx.sinon.stub(console, \'log\'); }, },),',
                '  it({ name: \'spawns\', fn: async () => { void `process.env.PATH = \'/cats\';`; }, },),',
                '  it({ name: \'moves\', fn: async () => {',
                '    process.chdir(\'/cats\'); process.umask(0o22); globalThis.purr ??= 1; delete process.env.NAP;',
                '    Object.assign(process.env, { A: \'1\', },); Object.defineProperty(globalThis, \'x\', { value: 1, },);',
                '    Reflect.set(Date, \'now\', () => 0,); process.cwd(); process.umask();',
                '  }, },),',
                '], },);',
                'function outerHelper() { innerHelper(); }',
                'function innerHelper() { Math.random = () => 0.5; }',
              ].join('\n',),
            },
            {
              path: 'whiskers.unit.test.ts',
              isTest: true,
              text: [
                'process.exitCode = 0;',
                'const CLEARERS = { cat: function clearCat() { delete process.env.CAT; }, };',
                'await describe({ name: \'\', children: [',
                '  describe({ name: \'inner\', concurrency: 1, children: [',
                '    it({ name: \'inner sets\', fn: async () => { process.env.CAT = \'calico\'; }, },),',
                '  ], },),',
                '  describe({ name: \'other\', children: [',
                '    it({ name: \'reads\', fn: async () => { void process.env.CAT; }, },),',
                '  ], },),',
                '], },);',
                'describe({ name: \'loose\', concurrency: 1, children: [',
                '  it({ name: \'unawaited\', fn: async () => { console.error = () => {}; }, },),',
                '], },);',
              ].join('\n',),
            },
            {
              path: 'kibble.ts',
              isTest: false,
              text: 'process.env.FOOD = \'fish\';',
            },
          ],
        },),).toEqual([
          'catnip.unit.test.ts: Math assigned, in a suite that runs cases beside it',
          'catnip.unit.test.ts: Object.assign, in a suite that runs cases beside it',
          'catnip.unit.test.ts: Object.defineProperty, in a suite that runs cases beside it',
          'catnip.unit.test.ts: Reflect.set, in a suite that runs cases beside it',
          'catnip.unit.test.ts: console assigned, in a suite that runs cases beside it',
          'catnip.unit.test.ts: globalThis assigned, in a suite that runs cases beside it',
          'catnip.unit.test.ts: process deleted, in a suite that runs cases beside it',
          'catnip.unit.test.ts: process.chdir, in a suite that runs cases beside it',
          'catnip.unit.test.ts: process.umask, in a suite that runs cases beside it',
          'whiskers.unit.test.ts: console assigned, in a suite not awaited at its file\'s top',
          'whiskers.unit.test.ts: process assigned, in a suite that runs cases beside it',
          'whiskers.unit.test.ts: process assigned, outside any case',
          'whiskers.unit.test.ts: process deleted, in a function no case reaches',
        ],);
      },
    },),
    it({
      name: 'FOLLOWS a writer a fixture exports to every file importing it by name, under an alias or not, '
        + 'leaves a case run alone that calls it, finds one run beside others, and reports a writer no file '
        + 'imports',
      fn: async () => {
        expect(unconfinedWrites({
          files: [
            {
              path: 'den.test-fixture.ts',
              isTest: true,
              text: [
                'export function pointDen(path: string): Disposable {',
                '  process.env.DEN = path;',
                '  return { [Symbol.dispose]: function restoreDen(): void { delete process.env.DEN; }, };',
                '}',
                'export function strayDen(): void { process.env.STRAY = \'yarn\'; }',
              ].join('\n',),
            },
            {
              path: 'corpus-run/den-alone.unit.test.ts',
              isTest: true,
              text: [
                'import { pointDen as point, } from \'../den.test-fixture.ts\';',
                'await describe({ name: \'alone\', concurrency: 1, children: [',
                '  it({ name: \'points\', fn: async () => { using pointed = point(\'/den\'); await nap(); }, },),',
                '], },);',
              ].join('\n',),
            },
            {
              path: 'den-beside.unit.test.ts',
              isTest: true,
              text: [
                'import { pointDen, } from \'./den.test-fixture.ts\';',
                'await describe({ name: \'beside\', children: [',
                '  it({ name: \'points\', fn: async () => { using pointed = pointDen(\'/den\'); await nap(); }, },),',
                '], },);',
              ].join('\n',),
            },
          ],
        },),).toEqual([
          'den.test-fixture.ts: process assigned, in a suite that runs cases beside it',
          'den.test-fixture.ts: process assigned, in strayDen, which no call by name reaches',
          'den.test-fixture.ts: process deleted, in a suite that runs cases beside it',
        ],);
      },
    },),
    it({
      name: 'ALLOWS A WRITE OUTSIDE ANY CASE IN A NAMED PROCESS ENTRY ONLY, and still finds one of its writes in a '
        + 'suite that runs cases beside it, and the same write in any other fixture',
      fn: async () => {
        expect(unconfinedWrites({
          files: [
            {
              path: PROCESS_ENTRIES[0] ?? '',
              isTest: true,
              text: [
                'process.exitCode = 6;',
                'await describe({ name: \'entry\', children: [',
                '  it({ name: \'sets\', fn: async () => { process.env.CAT = \'tabby\'; await nap(); }, },),',
                '], },);',
              ].join('\n',),
            },
            {
              path: 'den-entry.test-fixture.ts',
              isTest: true,
              text: 'process.exitCode = 6;',
            },
          ],
        },),).toEqual([
          'den-entry.test-fixture.ts: process assigned, outside any case',
          `${PROCESS_ENTRIES[0] ?? ''}: process assigned, in a suite that runs cases beside it`,
        ],);
      },
    },),
    it({
      name: 'FINDS A WRITE TO `Symbol`, where the parse-end hook of the grammar stands, unless it is set and disposed '
        + 'synchronously under `using`: a set across an await, a set with no disposal in scope and an assignment are found '
        + 'in a suite that runs cases beside it, and a fixture that sets and disposes in a function with no await is not',
      fn: async () => {
        expect(unconfinedWrites({
          files: [
            {
              path: 'hook.unit.test.ts',
              isTest: true,
              text: [
                'await describe({ name: \'hook\', children: [',
                '  it({ name: \'scoped\', fn: async () => { using held = standing(); void held; }, },),',
                '  it({ name: \'awaits\', fn: async () => { using held = standing(); await nap(); }, },),',
                '  it({ name: \'loose\', fn: async () => { setHook(); }, },),',
                '  it({ name: \'assigned\', fn: async () => { Symbol.cat = 1; }, },),',
                '], },);',
                'function standing() {',
                '  Reflect.set(Symbol, PARSE_END_HOOK_KEY, () => {});',
                '  return { [Symbol.dispose]() { Reflect.deleteProperty(Symbol, PARSE_END_HOOK_KEY); }, };',
                '}',
                'function setHook() { Reflect.set(Symbol, PARSE_END_HOOK_KEY, () => {}); }',
              ].join('\n',),
            },
            {
              path: 'seam.test-fixture.ts',
              isTest: true,
              text: [
                'export function withHook(run) {',
                '  using held = hookSet();',
                '  run();',
                '}',
                'export async function withHookLater(run) {',
                '  using held = hookSet();',
                '  await run();',
                '}',
                'function hookSet() {',
                '  Reflect.set(Symbol, PARSE_END_HOOK_KEY, 1);',
                '  return { [Symbol.dispose]: function putBack() { Reflect.deleteProperty(Symbol, PARSE_END_HOOK_KEY); }, };',
                '}',
              ].join('\n',),
            },
            {
              path: 'beside.unit.test.ts',
              isTest: true,
              text: [
                'import { withHook, withHookLater, } from \'./seam.test-fixture.ts\';',
                'await describe({ name: \'beside\', children: [',
                '  it({ name: \'sync\', fn: async () => { withHook(() => {}); }, },),',
                '  it({ name: \'later\', fn: async () => { await withHookLater(async () => {}); }, },),',
                '], },);',
              ].join('\n',),
            },
          ],
        },),).toEqual([
          'hook.unit.test.ts: Reflect.deleteProperty, in a suite that runs cases beside it',
          'hook.unit.test.ts: Reflect.set, in a suite that runs cases beside it',
          'hook.unit.test.ts: Reflect.set, in a suite that runs cases beside it',
          'hook.unit.test.ts: Symbol assigned, in a suite that runs cases beside it',
          'seam.test-fixture.ts: Reflect.deleteProperty, in a suite that runs cases beside it',
          'seam.test-fixture.ts: Reflect.set, in a suite that runs cases beside it',
        ],);
      },
    },),
    it({
      name: 'WRITES NO PROCESS GLOBAL in this package\'s tests outside a case run one at a time',
      fn: async () => {
        /**
         Every package file, tests among them.
         */
        const files = await readPackageSource();
        expect(files.some(function testFile({ isTest, },): boolean {
          return isTest;
        },),).toBe(true,);
        expectNoFindings({
          findings: PROCESS_ENTRIES.filter(function gone(entry,): boolean {
            return !files.some(function holdsEntry({ path, },): boolean {
              return path === entry;
            },);
          },),
        },);
        expectNoFindings({ findings: unconfinedWrites({ files, },), },);
      },
    },),
  ],
},);
