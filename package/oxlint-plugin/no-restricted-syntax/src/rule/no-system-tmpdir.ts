import type {
  Context,
  CreateOnceRule,
  ESTree,
  VisitorWithHooks,
} from '@oxlint/plugins';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

//region Rule definition

/**
 Module specifiers that expose the operating system's temporary directory.
 */
const OS_MODULE_SPECIFIERS: ReadonlySet<string> = new Set(['node:os', 'os',],);

/**
 Name of the Node function that returns the system temporary directory.
 */
const TMPDIR_NAME = 'tmpdir';

/**
 Bans the system temporary directory: which device backs `/tmp` is unknowable
 (a small tmpfs with a per-user quota filled once and failed every command in
 every session), so fixtures and working files go under a directory the code
 chooses, such as `node_modules/.monochromatic/<tool>` or the test harness's
 `scratchRoot()`.

 Reports a named `tmpdir` import from `node:os` or `os`, and any `x.tmpdir()`
 call, which covers `os.tmpdir()` and `await import('node:os')` results.

 @example
 ```ts
 // Bad
 import { tmpdir } from 'node:os';
 const directory = await mkdtemp(join(tmpdir(), 'work-'));

 // Good
 const directory = await mkdtemp(join(await scratchParent(), 'work-'));
 ```
 */
export const noSystemTmpdir: CreateOnceRule = {
  meta: {
    type: 'problem',
    docs: {
      description: 'Disallow the system temporary directory, whose backing device is unknowable.',
      recommended: true,
    },
    messages: {
      forbidden:
        'The system temporary directory is banned: its backing device is unknowable and a full tmpfs fails every command. Use `scratchRoot()` from `@monochromatic-dev/test-harness-core/ts/scratch-root.ts` in tests, or a directory under `node_modules/.monochromatic/<tool>` in tools.',
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
    return {
      ImportDeclaration(node: ForeignBorrowed<ESTree.ImportDeclaration>,): void {
        if (!OS_MODULE_SPECIFIERS.has(String(node.source.value,),))
          return;
        for (const specifier of node.specifiers) {
          if (specifier.type !== 'ImportSpecifier')
            continue;
          /**
           Name the module exports the binding under, an identifier or a string literal.
           */
          const importedName = specifier.imported.type === 'Identifier'
            ? specifier.imported.name
            : String(specifier.imported.value,);
          if (importedName !== TMPDIR_NAME)
            continue;
          context.report({
            node: specifier,
            messageId: 'forbidden',
          },);
        }
      },
      VariableDeclarator(node: ForeignBorrowed<ESTree.VariableDeclarator>,): void {
        if (node.id.type !== 'ObjectPattern' || node.init === null || node.init === undefined)
          return;
        /**
         The imported module expression, unwrapped from an `await`.
         */
        const source = node.init.type === 'AwaitExpression' ? node.init.argument : node.init;
        if (source.type !== 'ImportExpression' || source.source.type !== 'Literal')
          return;
        if (!OS_MODULE_SPECIFIERS.has(String(source.source.value,),))
          return;
        for (const property of node.id.properties) {
          if (property.type !== 'Property' || property.computed || property.key.type !== 'Identifier')
            continue;
          if (property.key.name !== TMPDIR_NAME)
            continue;
          context.report({
            node: property,
            messageId: 'forbidden',
          },);
        }
      },
      CallExpression(node: ForeignBorrowed<ESTree.CallExpression>,): void {
        /**
         Callee of the call under inspection.
         */
        const { callee, } = node;
        if (callee.type !== 'MemberExpression' || callee.computed)
          return;
        if (callee.property.type !== 'Identifier' || callee.property.name !== TMPDIR_NAME)
          return;
        context.report({
          node,
          messageId: 'forbidden',
        },);
      },
    };
  },
};

//endregion Rule definition
