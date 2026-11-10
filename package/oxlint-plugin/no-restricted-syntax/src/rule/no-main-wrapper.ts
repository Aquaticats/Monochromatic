import type {
  Context,
  CreateOnceRule,
  ESTree,
  Scope,
  Variable,
  VisitorWithHooks,
} from '@oxlint/plugins';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

//region Recognising the wrapper

/**
 Name of the function a wrapper file declares only to call it.
 */
const WRAPPER_FUNCTION_NAME = 'main';

/**
 Strips a leading `await` so `await main()` and the synchronous `main()` reach
 the same check.
 
 @param expression - statement expression under inspection
 
 @returns the expression itself, or the operand of its `await`
 
 @example
 ```ts
 unawaited(awaitExpression).type; // 'CallExpression' for `await main()`
 ```
 */
function unawaited(
  expression: ForeignBorrowed<ESTree.Expression>,
): ForeignBorrowed<ESTree.Expression> {
  return expression.type === 'AwaitExpression' ? expression.argument : expression;
}

/**
 Returns the `main()` call a statement exists to make, or `undefined` when the
 statement is anything else.
 
 Only a zero-argument call to a bare `main` identifier qualifies. An argument
 means the call carries the script's input, and `main` read through a member
 expression belongs to another object.
 
 @param statement - statement under inspection
 
 @returns the offending call, or `undefined` when the statement is no wrapper call
 
 @example
 ```ts
 wrapperCallOf(expressionStatementOf('await main();'))?.callee.name; // 'main'
 ```
 */
function wrapperCallOf(
  statement: ForeignBorrowed<ESTree.Node>,
): ForeignBorrowed<ESTree.CallExpression> | undefined {
  if (statement.type
    !== 'ExpressionStatement')
    return undefined;
  /**
   The call inside the statement, with any `await` prefix stripped.
   */
  const expression = unawaited(statement.expression,);
  if (expression.type
    !== 'CallExpression')
    return undefined;
  if (expression.callee
    .type
    !== 'Identifier')
    return undefined;
  if (expression.callee
    .name
    !== WRAPPER_FUNCTION_NAME)
    return undefined;
  if (expression.arguments
    .length
    !== 0)
    return undefined;
  return expression;
}

/**
 Collects the statements a script runs as file-level code: the program body
 itself, plus the statements of a top-level `try` block, so a wrapper whose call
 sits in `try`/`catch` for logging still counts as file-level code.
 
 @param program - program node of the file under lint
 
 @returns every statement the file runs at top level, in source order
 
 @example
 ```ts
 fileLevelStatements(program).length; // 2 for `try { await main(); } catch {}` ... plus the try itself is expanded
 ```
 */
function fileLevelStatements(
  program: ForeignBorrowed<ESTree.Program>,
): readonly ForeignBorrowed<ESTree.Node>[] {
  /**
   File-level statements collected so far.
   */
  const statements: ForeignBorrowed<ESTree.Node>[] = [];
  for (const statement of program.body) {
    if (statement.type
      === 'TryStatement')
      statements.push(...statement.block.body,);
    else
      statements.push(statement,);
  }
  return statements;
}

//endregion Recognising the wrapper

//region Recognising the binding

/**
 Returns the scope a file's own top-level declarations live in.
 
 The scope manager nests that module scope below the global scope, which holds
 the JavaScript globals, so a lookup that stopped at `getScope(program)` would
 only ever find builtins. A file parsed as a script has no separate module scope
 and keeps its declarations in the global scope itself, which is the fallback.
 
 @param program - program node of the file under lint
 
 @param globalScope - outermost scope the scope manager reports for the program
 
 @returns the innermost scope whose block is the program
 
 @example
 ```ts
 fileScopeOf({ program, globalScope }).set.get('main'); // the file's own `main`
 ```
 */
function fileScopeOf({
  program,
  globalScope,
}: ForeignBorrowed<{
  readonly program: ESTree.Program;
  readonly globalScope: Scope;
}>,): ForeignBorrowed<Scope> {
  for (const scope of globalScope.childScopes) {
    if (scope.block
      === program)
      return scope;
  }
  return globalScope;
}

/**
 Checks that `main` is declared once, at file level, not exported, and is a
 function declaration: the shapes a wrapper takes.
 
 An exported declaration is importable, so callers in other files keep it
 alive, and a `const main = ...` initializer is a value other code can read.
 
 @param binding - module-scope binding the file's `main` calls resolve to
 
 @param program - program node of the file under lint
 
 @returns whether the binding is one file-level, non-exported function declaration
 
 @example
 ```ts
 declaresMain({ binding, program }); // true for `async function main(): Promise<void> {}`
 ```
 */
function declaresMain({
  binding,
  program,
}: ForeignBorrowed<{
  readonly binding: Variable;
  readonly program: ESTree.Program;
}>,): boolean {
  if (binding.defs
    .length
    !== 1)
    return false;
  /**
   The single definition of the binding.
   */
  const [definition,] = binding.defs;
  if (definition === undefined)
    return false;
  if (definition.type
    !== 'FunctionName')
    return false;
  /**
   The declaration the definition names.
   */
  const declaration = definition.node;
  if (declaration.type
    !== 'FunctionDeclaration')
    return false;
  return program.body
    .some(function isThisStatement(statement,): boolean {
      return statement === declaration;
    },);
}

/**
 Checks that every reference to `main` is one of the calls the rule found.
 
 A single extra reference means the function is called for its value, passed to
 another function, or exported, so it is not declared only to be called at the
 end of the file, and inlining its body would duplicate it.
 
 @param references - every reference the scope manager resolved to the binding
 
 @param calls - file-level `main()` calls of the script
 
 @returns whether the calls are the binding's only references
 
 @example
 ```ts
 callsAreOnlyReferences({ references, calls }); // true for one call and one reference
 ```
 */
function callsAreOnlyReferences({
  references,
  calls,
}: ForeignBorrowed<{
  readonly references: readonly { readonly identifier: ESTree.Identifier; }[];
  readonly calls: readonly ESTree.CallExpression[];
}>,): boolean {
  if (references.length
    !== calls.length)
    return false;
  return calls.every(function isReferenced(call,): boolean {
    return references.some(function isThisReference(reference,): boolean {
      return reference.identifier
        === call.callee;
    },);
  },);
}

//endregion Recognising the binding

//region Rule definition

/**
 Bans the `main()` wrapper a script declares only to call it at the end, in
 favor of running the statements at file level with top-level `await`.
 
 A wrapper adds a name, a call hop, and an indentation level, and it moves the
 `@throws` contract one frame away from the code that raises it. The repository
 convention is top-level code, as the script conventions in
 `.agents/skills/project-code-review/SKILL.md` state.
 
 Only this shape is reported: one file-level function declaration named `main`,
 not exported, whose every reference is a file-level `main()` or
 `await main()` call. A `main` that is exported, passed to a runner such as a
 test harness, called with arguments, or guarded by `import.meta.main` stays
 unreported because inlining it would change what the file does.
 
 @example
 ```ts
 // Bad
 async function main(): Promise<void> {
   await run();
 }
 await main();
 
 // Good
 await run();
 ```
 */
export const noMainWrapper: CreateOnceRule = {
  meta: {
    type: 'suggestion',
    docs: {
      description: 'Disallow the main() wrapper a script declares only to call at the end.',
      recommended: true,
    },
    messages: {
      forbidden:
        'This file declares `main` only to call it here. Run the statements at file level, using top-level `await` for async work, so the body and its throw contract sit where the script runs.',
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
     Reports the file-level `main()` calls of a script whose `main` binding
     exists only for them.
     
     Runs at `Program:exit` so the scope manager has resolved every reference to
     the binding before the rule decides whether the wrapper is the only caller.

     @param program - program node of the file under lint
     */
    function reportWrapperCalls(program: ForeignBorrowed<ESTree.Program>,): void {
      /**
       Outermost scope the scope manager reports for this program.
       */
      const globalScope = context.sourceCode
        .getScope(program,);
      /**
       Module-scope binding the file's `main` mentions resolve to, or `undefined` when the file never names one.
       */
      const binding = fileScopeOf({
        program,
        globalScope,
      },)
        .set
        .get(WRAPPER_FUNCTION_NAME,);
      if (binding === undefined)
        return;
      if (!declaresMain({
        binding,
        program,
      },))
        return;
      /**
       File-level `main()` calls of this script.
       */
      const calls: ForeignBorrowed<ESTree.CallExpression>[] = [];
      for (const statement of fileLevelStatements(program,)) {
        /**
         The wrapper call this statement makes, when it makes one.
         */
        const call = wrapperCallOf(statement,);
        if (call !== undefined)
          calls.push(call,);
      }
      if (calls.length
        === 0)
        return;
      if (!callsAreOnlyReferences({
        references: binding.references,
        calls,
      },))
        return;
      for (const call of calls)
        context.report({
          node: call,
          messageId: 'forbidden',
        },);
    }

    return { 'Program:exit': reportWrapperCalls, };
  },
};

//endregion Rule definition
