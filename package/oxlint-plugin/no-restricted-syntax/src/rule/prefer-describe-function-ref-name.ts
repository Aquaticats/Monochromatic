import { readFileSync, } from 'node:fs';
import {
  dirname,
  resolve,
} from 'node:path';

import type {
  Context,
  CreateOnceRule,
  ESTree,
  Scope,
  Variable,
  VisitorWithHooks,
} from '@oxlint/plugins';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import {
  classifyNameInSourceText,
  type ExportedNameKind,
  isAllCapsSnakeName,
} from './prefer-describe-function-ref-name.export-scan.ts';

/**
 Imported source text cache keyed by resolved absolute path.
 */
const sourceTextByPath = new Map<string, string>();

/**
 Reads imported source text, returning an empty string when the file cannot
 be read.
 
 Oxlint rule visitors are synchronous, so relative-import classification
 cannot await filesystem I/O. Cache each path in {@link sourceTextByPath}
 so one lint pass reads every inspected import at most once.
 
 @param sourcePath - Resolved absolute path of imported source.
 
 @returns Source text, or empty string when reading fails.
 
 @example
 ```ts
 const text = readSourceTextOrEmpty('/repo/packages/example/src/index.ts');
 ```
 */
function readSourceTextOrEmpty(sourcePath: string,): string {
  /**
   Cached source text, or `undefined` when this path has not been read.
   */
  const cached = sourceTextByPath.get(sourcePath,);
  if (cached !== undefined)
    return cached;
  try {
    /* oxlint-disable no-restricted-syntax/no-sync -- sync oxlint visitor; classification must finish before report; see doc/troubleshooting/oxlint-js-plugin-sync-visitors.md. */
    /**
     Source text read from the imported file.
     */
    const content = readFileSync(
      sourcePath,
      'utf8',
    );
    /* oxlint-enable no-restricted-syntax/no-sync */
    sourceTextByPath.set(
      sourcePath,
      content,
    );
    return content;
  }
  catch (readError: unknown) {
    void readError;
    sourceTextByPath.set(
      sourcePath,
      '',
    );
    return '';
  }
}

/**
 Reads an imported source file, via {@link readSourceTextOrEmpty}, and
 reports whether `<name>` is declared as a function, class, or some other
 shape there.
 
 Resolves only relative paths; bails on workspace and external imports
 because the resolver would need to walk the package graph. Bails on
 read errors too. The caller falls back to the
 {@link isAllCapsSnakeName} heuristic in either case.
 
 Match patterns:
 
 - `export function <name>` -\> `'callable'`
 - `export async function <name>` -\> `'callable'`
 - `export function* <name>` -\> `'callable'`
 - `export class <name>` -\> `'callable'`
 - `export const <name>` (any init) -\> `'const'` (caller's responsibility
   to ignore; instance `.name` is rarely meaningful)
 - `export { ... <name> ... }` re-export from another file -\> `'reexport'`
   (caller falls back to the name-shape heuristic; following re-exports
   would require recursing through the package, beyond this rule's scope)
 - none of the above -\> `'unknown'` (treat as not callable)
 
 @param sourcePath - Resolved absolute path of the imported file.
 
 @param name - Name being inspected in the source's export surface.
 
 @returns Tag describing what kind of binding the source exposes.
 
 @example
 ```ts
 const kind = classifyExportedName({
   sourcePath: '/repo/packages/foo/src/bar.ts',
   name: 'bar',
 });
 // kind === 'callable' when bar.ts contains `export function bar() {}`
 ```
 */
function classifyExportedName(
  {
    sourcePath,
    name,
  }: {
    readonly sourcePath: string;
    readonly name: string;
  },
): ExportedNameKind {
  /**
   Source text of the imported file; empty when the read fails so callers can short-circuit.
   */
  const content = readSourceTextOrEmpty(sourcePath,);
  if (content === '')
    return 'unknown';
  return classifyNameInSourceText({
    content,
    name,
  },);
}

/**
 Determines whether the binding behind a {@link Variable} is callable in
 a way that makes `binding.name` a meaningful suite name.
 
 - Function declarations and class declarations: yes, `.name` returns the
   declared name.
 - `const`/`let`/`var` declarations: only when the initializer is itself a
   function or class expression. `const X = new Map()` and `const X = 5`
   produce `undefined` from `.name` and must be left as string literals.
 - Imports from a relative path: synchronously read the source file and,
   via {@link classifyExportedName}, look for `export function`/
   `export class`/`export const` for `name`. Treat `export const` (any
   init) as not callable. Re-exports from another file
   (`export { name } from '...'`) and read failures fall through to the
   {@link isAllCapsSnakeName} name-shape heuristic.
 - Imports from a workspace or external package: {@link isAllCapsSnakeName}
   -\> not callable; otherwise assumed callable.
 - Other definition kinds (parameter, catch clause, implicit global):
   not the rule's target.
 
 @param variable - Scope-manager {@link Variable} for the matched binding.
 
 @param currentFile - Absolute path of the file being linted, used to
   resolve relative import paths.
 
 @returns `true` when the binding is callable and the function-reference
   form makes sense.
 
 @example
 ```ts
 const callable = isCallableBinding({
   variable: scope.set.get('coerceArg'),
   currentFile: '/repo/packages/foo/src/index.ts',
 });
 // callable === true when coerceArg resolves to a function declaration
 ```
 */
function isCallableBinding(
  {
    variable,
    currentFile,
  }: ForeignBorrowed<{
    readonly variable: Variable;
    readonly currentFile: string;
  }>,
): boolean {
  /**
   First definition site of the binding; absent for implicit globals the rule does not target.
   */
  const [def,] = variable.defs;
  if (def === undefined)
    return false;
  if ((def.type
    === 'FunctionName') || (def.type
      === 'ClassName'))
    return true;
  if (def.type
    === 'ImportBinding') {
    /**
     Walk up to the enclosing ImportDeclaration to inspect the source.
     */
    const { node, } = def;
    /**
     Resolved enclosing ImportDeclaration, or the non-import parent when scope-manager hands back something unexpected.
     */
    const decl = node.type
      === 'ImportDeclaration'
      ? node
      : node.parent;
    if ((decl === null) || (decl === undefined)
      || (decl.type
        !== 'ImportDeclaration'))
      return !isAllCapsSnakeName(variable.name,);
    /**
     Literal source string of the import, typed as `string | null` by ESTree to cover non-conforming nodes.
     */
    const sourceValue = decl.source
      .value;
    if ((typeof sourceValue) !== 'string')
      return !isAllCapsSnakeName(variable.name,);
    if (!sourceValue.startsWith('.',))
      return !isAllCapsSnakeName(variable.name,);
    /**
     Resolve relative to the file under lint.
     */
    const sourcePath = resolve(
      dirname(currentFile,),
      sourceValue,
    );
    /**
     Imports use the alias's local name; the source may export under a different identifier.
     */
    const sourceName = (node.type
      === 'ImportSpecifier') && ('imported' in node)
      ? (function getImportedName(): string {
        /**
         Imported-name slot on the specifier; an Identifier or string Literal per ESTree.
         */
        const { imported, } = node;
        if (imported.type
          === 'Identifier')
          return imported.name;
        return variable.name;
      })()
      : variable.name;
    /**
     Classification tag distinguishing callable, plain const, re-export, or unresolved bindings.
     */
    const kind = classifyExportedName({
      sourcePath,
      name: sourceName,
    },);
    if (kind === 'callable')
      return true;
    if (kind === 'const')
      return false;
    return !isAllCapsSnakeName(variable.name,);
  }
  if (def.type
    === 'Variable') {
    /**
     VariableDeclarator AST node carrying the initializer to inspect for callability.
     */
    const declarator = def.node;
    if (declarator.type
      !== 'VariableDeclarator')
      return false;
    /**
     Right-hand initializer; absent for `let x;`-style declarations the rule treats as non-callable.
     */
    const { init, } = declarator;
    if ((init === null) || (init === undefined))
      return false;
    return (
      (init.type
        === 'FunctionExpression')
        || (init.type
          === 'ArrowFunctionExpression')
        || (init.type
          === 'ClassExpression')
    );
  }
  return false;
}

/**
 Prefers `describe({ name: myFn.name, ... })` over
 `describe({ name: 'myFn', ... })` whenever `myFn` is a callable binding
 in scope.
 
 The function-reference form keeps suite names synchronised with renames:
 `Function.prototype.name` updates automatically when the underlying
 declaration is renamed, while a string literal silently drifts.
 
 Fires only when the matched binding is callable, tested via
 {@link isCallableBinding} (function declaration, class declaration,
 import not in {@link isAllCapsSnakeName} shape, or `const` initialized with a
 function/class expression). Bindings that have no meaningful `.name` are
 left alone:
 
 - `const X = new Map()` -- instance `.name` is `undefined`.
 - `const X = { ... }` -- instance `.name` is `undefined`.
 - `const X = 5` -- numbers have no `.name`.
 - `import { THIRTY } from '...'` -- when `THIRTY` is {@link isAllCapsSnakeName}
   the import is assumed to be a constant value, not a function.
   (Camel-case imports of non-callable values remain a residual false
   positive -- the rule cannot resolve imports at lint time without I/O.)
 
 Empty-string names (`name: ''`) are exempted: the harness uses them
 for invisible top-level suites where the filename already identifies
 the test target.
 
 Global identifiers (browser `find`, `name`, `event`, etc. enabled via
 `env.browser`) live in the global scope and are skipped during the
 scope-chain walk. The rule only inspects the module scope and any
 intermediate function/block scopes.
 
 Harness self-tests in `package/test-harness/core/src/{describe,it}.unit.test.ts`
 are circular by design (the function under test IS the local binding);
 they opt out via inline
 `oxlint-disable-next-line no-restricted-syntax/prefer-describe-function-ref-name`.
 
 @example
 ```ts
 // Bad; string literal mirrors an in-scope import of a function
 import { coerceArg } from './coerce-arg.ts';
 await describe({ name: 'coerceArg', children: [\/* ... *\/] });
 
 // Good; function reference auto-syncs on rename
 import { coerceArg } from './coerce-arg.ts';
 await describe({ name: coerceArg.name, children: [\/* ... *\/] });
 
 // Good; MANAGERS is a `const = new Map()`, .name is undefined
 const MANAGERS = new Map();
 await describe({ name: 'MANAGERS', children: [\/* ... *\/] });
 
 // Good; no binding named 'fixtures' in scope
 await describe({ name: 'fixtures', children: [\/* ... *\/] });
 
 // Good; empty name (harness convention for invisible top-level suite)
 await describe({ name: '', children: [\/* ... *\/] });
 ```
 */
export const preferDescribeFunctionRefName: CreateOnceRule = {
  meta: {
    type: 'suggestion',
    docs: {
      description:
        "Prefer `describe({ name: fn.name })` over `describe({ name: 'fn' })` when `fn` is an in-scope binding.",
      recommended: true,
    },
    messages: {
      forbidden: "`describe` name `'{{name}}'` matches the in-scope binding `{{name}}`. "
        + 'Replace with `{{name}}.name` so renames stay in sync. '
        + 'See .agents/skills/testing-practices/SKILL.md for the convention. '
        + 'If the function under test IS the local binding (harness self-test), '
        + 'add `oxlint-disable-next-line no-restricted-syntax/prefer-describe-function-ref-name` '
        + 'with a justification.',
    },
  },
  /**
   Handles foreign Oxlint callback.
   
   @param context - Foreign rule context receiving diagnostics.
   
   @mutates context - Emits Oxlint diagnostics through foreign rule context.
   
   @example
   ```ts
   createOnce(context);
   ```
   */
  createOnce(context: ForeignBorrowed<Context>,): VisitorWithHooks {
    /**
     Inspects a `describe(...)` call expression and reports when its
     `name` property is a string literal that matches an in-scope
     binding classified callable by {@link isCallableBinding}.
     
     @param node - The `CallExpression` AST node.
     */
    function checkCall(node: ForeignBorrowed<ESTree.CallExpression>,): void {
      if ((node.callee
        .type
        !== 'Identifier') || (node.callee
          .name
          !== 'describe'))
        return;
      /**
       First argument of the call, or `undefined` when none was passed.
       */
      const [firstArg,] = node.arguments;
      if ((firstArg === undefined) || (firstArg.type
        !== 'ObjectExpression'))
        return;
      for (const prop of firstArg.properties) {
        if (prop.type
          !== 'Property')
          continue;
        if (prop.computed)
          continue;
        if (prop.shorthand)
          continue;
        if ((prop.key
          .type
          !== 'Identifier') || (prop.key
            .name
            !== 'name'))
          continue;
        if ((prop.value
          .type
          !== 'Literal') || ((typeof prop.value
            .value) !== 'string'))
          continue;
        /**
         String value of the `name` property.
         */
        const stringValue = prop.value
          .value;
        if (stringValue === '')
          return;
        /* oxlint-disable no-restricted-syntax/no-nullish-union -- mirrors eslint's Scope API: `context.sourceCode.getScope` returns `Scope` and `scope.upper` is typed `Scope | null` (null above the global scope), so this scope-walk cursor must admit `null`; external-boundary type, not a nullish escape we control */
        for (
          let scope: Scope | null = context.sourceCode
            .getScope(node,);
          (scope !== null) && (scope.type
            !== 'global');
          scope = scope.upper
        ) {
          /**
           Binding registered in this scope under `stringValue`, or `undefined` when the scope has none.
           */
          const variable = scope.set
            .get(stringValue,);
          if (variable === undefined)
            continue;
          if (isCallableBinding({
            variable,
            currentFile: context.filename,
          },)) {
            context.report({
              node: prop.value,
              messageId: 'forbidden',
              data: { name: stringValue, },
            },);
          }
          return;
        }
        /* oxlint-enable no-restricted-syntax/no-nullish-union */
        return;
      }
    }

    return {
      CallExpression: checkCall,
    };
  },
};
