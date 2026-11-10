# Implement always-only comma-dangle in oxlint-stylistic

## Purpose

Implement a new `stylistic/comma-dangle` rule in
`package/oxlint-plugin/stylistic`.
The rule should require trailing commas everywhere oxlint's JavaScript plugin
AST exposes a comma-delimited list where a trailing comma is valid.
It should support only the `always` behavior and no options.

This package is a TypeScript `@oxlint/plugins` JavaScript plugin,
 not a native
Rust oxlint rule.
Do not edit the upstream `oxc` Rust crates for this task.

## Suggested skills for the next session

- `testing-practices`:
   fixture and autofix tests use the monorepo test harness.
- `diagnose`:
   use only if oxlint plugin AST fields or autofix behavior diverge
  from the verified type definitions below.

## Current package context

Relevant files:

- `package/oxlint-plugin/stylistic/src/index.ts`:
   plugin entry point.
- `package/oxlint-plugin/stylistic/src/rule/semi.ts`:
   closest existing
  always-only,
   no-options,
   auto-fixable rule.
- `package/oxlint-plugin/stylistic/src/rule/argument-per-line.ts`:
   simple
  visitor shape and local structural node typing.
- `package/oxlint-plugin/stylistic/src/oxlint-stylistic.unit.test.ts`:
  fixture-based diagnostics and autofix tests.
- `package/test-fixture/oxlint-stylistic/.oxlintrc.fixture.json`:
   all plugin
  rules enabled for fixtures.
- `package/test-fixture/oxlint-stylistic/src/valid/`:
   valid fixtures.
- `package/test-fixture/oxlint-stylistic/src/invalid/`:
   invalid and autofix
  fixtures.

`@oxlint/plugins` resolved to version `1.67.0` from this package during
research.
 The type definitions at
`node_modules/.pnpm/@oxlint+plugins@1.67.0/node_modules/@oxlint/plugins/index.d.ts`
show these APIs and AST fields:

- `context.sourceCode.getLastToken(node, skipOptions?)`.
- `context.sourceCode.getTokenAfter(nodeOrToken, skipOptions?)`.
- `context.sourceCode.getTokenBefore(nodeOrToken, skipOptions?)`.
- `Fixer.insertTextAfter(nodeOrToken, text)`.
- `Fixer.replaceTextRange(range, text)`.
- `ArrayExpression.elements: Array<ArrayExpressionElement>` where holes are
  `null`.
- `ObjectExpression.properties: Array<ObjectPropertyKind>`.
- `ArrayPattern.elements: Array<BindingPattern | BindingRestElement | null>`.
- `ObjectPattern.properties: Array<BindingProperty | BindingRestElement>`.
- `ImportDeclaration.specifiers` and `ImportDeclaration.attributes`.
- `ExportNamedDeclaration.specifiers` and `ExportNamedDeclaration.attributes`.
- `ExportAllDeclaration.attributes`.
- `ImportExpression.source` and `ImportExpression.options`.
- Function-like nodes expose `params`.
- TypeScript tuple,
   enum,
   generic declaration,
   signature,
   and function-type
  nodes expose list fields matching their names.

## Research summary

Upstream `eslint.style` `comma-dangle` is large because it preserves every ESLint
mode and compatibility edge:

- Current rule implementation:
   478 LOC.
- Current JS tests:
   2265 LOC.
- Current TS tests:
   302 LOC.
- Current docs:
   377 LOC.
- Current generated option types:
   37 LOC.

A prior always-only ESLint-style sketch,
 still keeping all syntax categories,
measured 325 implementation LOC and 648 total rule-directory LOC.
A native oxlint Rust sketch measured 206 LOC including inline docs and tests.
Those numbers are estimates,
 not a limit.
They show that the target package should need a small rule file rather than a
port of the full ESLint rule.

The existing fixtures confirm this design fits established repo convention rather
than imposing a new one.
`package/test-fixture/oxlint-stylistic/src/valid/single-item.ts` already places a
trailing comma on every single-item construct (`[1,]`,
 `{ host: 'localhost', }`,
`[string,]`,
 `export { identity, }`,
 `identity(x: number,)`),
 and
`package/test-fixture/oxlint-stylistic/src/invalid/fixable-trailing-comma.ts`
exists solely to assert that `--fix` preserves trailing commas.
The repo already mandates trailing commas everywhere,
 so always-only with no
options enforces what the fixtures and dprint output already produce;
 it is not a
new policy.

## Required semantics

`stylistic/comma-dangle` should:

- Have no options,
   mirroring `stylistic/semi`.
- Use `schema: []`,
   so `['error', 'always']` and other option arrays are rejected.
- Report `Missing trailing comma.` when a required comma is absent.
- Be auto-fixable with `fixable: 'code'`.
- Require a trailing comma for single-item and multi-item lists.
- Ignore empty lists.
- Ignore list positions whose last element is a rest element because JavaScript
  grammar rejects a trailing comma there.
- Preserve comments by inserting after the last syntax token before the closing
  delimiter,
   not by rebuilding the whole list.
- Avoid multiline-specific behavior.
   `always-multiline`,
   `only-multiline`,
  `never`,
   and `ignore` are out of scope.
- Avoid ECMAScript version gates.
   Oxlint parses the current source;
   this plugin
  should inspect the AST it receives.

Out of scope for the first pass:

- A compatibility clone of ESLint's `TSTypeParameterInstantiation` special case.
  Current `@stylistic/comma-dangle` removes trailing commas from type argument
  instantiations even when the configured mode is `always`.
  This package is adding an always-only rule,
   so the first implementation should
  not add a `never` branch unless a repo fixture or oxlint parse behavior proves
  it is needed.
  Consequence to document,
   not to fix:
   use-site type arguments such as
  `new Set<string>()` and `foo<A, B>()` will not receive a trailing comma,
   because
  no visitor covers `TSTypeParameterInstantiation`;
   only
  `TSTypeParameterDeclaration` (the `<T>` at a declaration site,
   in the covered
  set below) is handled.
   Note this in the README so it is not later filed as
  missed coverage.

## Syntax categories to cover

Cover these visitor nodes:

- `ArrayExpression`.
- `ObjectExpression`.
- `ArrayPattern`.
- `ObjectPattern`.
- `ImportDeclaration`,
   named specifier list only,
   plus import attributes.
- `ExportNamedDeclaration`,
   specifier list only,
   plus import attributes.
- `ExportAllDeclaration`,
   import attributes only.
- `FunctionDeclaration`.
- `FunctionExpression`.
- `ArrowFunctionExpression`.
- `CallExpression`.
- `NewExpression`.
- `ImportExpression`.
- `TSEnumDeclaration`.
- `TSTypeParameterDeclaration`.
- `TSTupleType`.
- `TSDeclareFunction`.
- `TSEmptyBodyFunctionExpression`.
- `TSFunctionType`.
- `TSMethodSignature`.
- `TSCallSignatureDeclaration`.
- `TSConstructSignatureDeclaration`.
- `TSConstructorType`.

The current package already covers the same TypeScript function-like set in
`param-per-line`,
 and the README lists those node names.

## Implementation outline

Create `package/oxlint-plugin/stylistic/src/rule/comma-dangle.ts`.
Use `semi.ts` as the style reference for no-options metadata and reporting.
Use small local structural types,
 as the existing rules do,
 rather than trying
to model the whole ESTree graph.

Expected rule shape:

```typescript
// package/oxlint-plugin/stylistic/src/rule/comma-dangle.ts
import type {
  Context,
  CreateOnceRule,
  Fixer,
  Node,
  Span,
  Token,
  VisitorWithHooks,
} from '@oxlint/plugins';

/** Token value required at end of comma-delimited lists. */
const COMMA = ',';

/** Closing delimiters accepted after insertion point tokens. */
const CLOSE_DELIMITERS = new Set([
  ')',
  ']',
  '}',
  '>',
],);
```

Use helpers shaped like this:

```typescript
// package/oxlint-plugin/stylistic/src/rule/comma-dangle.ts
/** Parameters for trailing-comma checks. */
type CheckTrailingCommaParams = {
  /** Rule context used for token lookup and reporting. */
  readonly context: Context;
  /** Container node whose delimiter closes the list. */
  readonly container: Node;
  /** Last concrete item in the list, or null for empty lists. */
  readonly lastItem: Node | null;
  /** Whether to use the token before the container close delimiter. */
  readonly useContainerPenultimateToken?: boolean;
};

/** Returns last non-null item in a list. */
function lastNonNull<T,>(items: readonly (T | null)[] | undefined,): T | null {
  if (items === undefined)
    return null;

  for (let index = items.length - 1; index >= 0; index -= 1) {
    const item = items[index];
    if (item !== null && item !== undefined)
      return item;
  }

  return null;
}
```

The `for` loop above is intentional because it needs a reverse index scan.
Do not replace it with recursion.

Core checker behavior:

1.  Return for `lastItem === null`.
2.  Return for `lastItem.type === 'RestElement'`.
3.  Find the insertion token.
    - For arrays,
       objects,
       calls,
       `new`,
       and dynamic imports,
       prefer
      `sourceCode.getLastToken(container, 1)` because it targets the token before
      the closing delimiter.
    - For imports,
       exports,
       attributes,
       enums,
       type parameters,
       tuples,
       and
      function params,
       use `sourceCode.getTokenAfter(lastItem)` to detect an
      existing comma,
       otherwise use `sourceCode.getLastToken(lastItem)`.

    The two strategies are not interchangeable;
     do not collapse them to
    `getLastToken(container, 1)` everywhere.
    For imports,
     exports,
     and import attributes the comma-delimited list is not at
    the container tail:
     an `ImportDeclaration` node spans through `from 'source'`
    and the optional `;`,
     so `getLastToken(container, 1)` returns the source-string
    token,
     and inserting after it produces the broken
    `import { one } from 'pkg', ;`.
    The `getTokenAfter(lastItem)` form is correct for the array/object/call group
    too,
     but `getLastToken(container, 1)` is kept there because it reads the token
    before the close delimiter directly,
     including for trailing array holes such as
    `[one, ,]`.
4.  If the trailing token is already `,`,
     return.
5.  Check the next token is one of `)`,
     `]`,
     `}`,
     or `>` when a next token exists.
6.  Report on `lastItem` with `messageId: 'missingComma'`.
7.  Fix with `fixer.insertTextAfter(insertionToken, ',')`.

Use `context.report` with a typed fixer callback,
 matching `semi.ts`.

```typescript
// package/oxlint-plugin/stylistic/src/rule/comma-dangle.ts
context.report({
  node: lastItem,
  messageId: 'missingComma',
  fix(fixer: Fixer,): ReturnType<Fixer['insertTextAfter']> {
    return fixer.insertTextAfter(
      insertionToken,
      COMMA,
    );
  },
},);
```

Do not widen fix ranges the way ESLint does.
Oxlint's fixer already sorts fixes and skips overlapping fixes at application
boundaries,
 as verified in the upstream `crates/oxc_linter/src/fixer/mod.rs`
research.

## Node extraction notes

Use these field mappings:

- `ArrayExpression`:
   `lastNonNull(node.elements)`.
- `ObjectExpression`:
   `lastNonNull(node.properties)`.
- `ArrayPattern`:
   `lastNonNull(node.elements)`,
   with rest ignored by the generic
  `RestElement` guard.
- `ObjectPattern`:
   `lastNonNull(node.properties)`,
   with rest ignored by the
  generic `RestElement` guard.
- `ImportDeclaration`:
   find the last specifier only when it is an
  `ImportSpecifier`;
   default and namespace imports do not form a comma list.
  Also check `node.attributes`.
- `ExportNamedDeclaration`:
   check `node.specifiers` when non-empty.
  Also check `node.attributes`.
- `ExportAllDeclaration`:
   check `node.attributes` only.
- Function-like nodes:
   check `node.params`.
- `CallExpression` and `NewExpression`:
   check `node.arguments`.
- `ImportExpression`:
   check `node.options ?? node.source`.
- `TSEnumDeclaration`:
   check `node.body.members`.
- `TSTypeParameterDeclaration`:
   check `node.params`.
- `TSTupleType`:
   check `node.elementTypes`.

Structural type names can stay local to the file.
Existing rules assert visitor `Span` or `Node` values into local shapes when
`@oxlint/plugins` omits the exact field from the generic type.
Add scoped `oxlint-disable-next-line typescript/no-unsafe-type-assertion`
comments only when the linter requires them,
 and justify the oxlint AST field
being asserted.

## Files to edit

Add:

- `package/oxlint-plugin/stylistic/src/rule/comma-dangle.ts`.
- `package/test-fixture/oxlint-stylistic/src/valid/comma-dangle.ts`.
- `package/test-fixture/oxlint-stylistic/src/invalid/comma-dangle.ts`.

Update:

- `package/oxlint-plugin/stylistic/src/index.ts`:
  import `commaDangle` and register `'comma-dangle': commaDangle`.
- `package/oxlint-plugin/stylistic/README.md`:
  add a rule entry under a new or existing layout section.
  Mention no options and always-only semantics.
- `package/test-fixture/oxlint-stylistic/.oxlintrc.fixture.json`:
  enable `stylistic/comma-dangle`.
- `package/oxlint-plugin/stylistic/src/oxlint-stylistic.unit.test.ts`:
  add diagnostic and autofix assertions.
- Existing valid and invalid fixtures as needed;
   see "Existing fixture impact"
  below for the measured per-file effect.

## Existing fixture impact

Measured surface at handoff:
 23 fixtures in
`package/test-fixture/oxlint-stylistic/src/invalid/`,
 7 in `src/valid/`.
Enabling `stylistic/comma-dangle` globally in `.oxlintrc.fixture.json` affects them
as follows.

Valid fixtures (each asserts no violations for the rules it covers):

- Already comma-safe,
   no change:
   `single-item.ts` (single-item lists already carry
  trailing commas),
   `empty-constructs.ts` (empty lists,
   which the rule ignores,
   so
  it doubles as the empty-list regression guard),
   `already-per-line.ts`,
  `no-mixed-operators.ts`,
   `chain-per-line.ts`,
   and `semi.ts`.
   Every multi-element
  list in these is multiline with a trailing comma,
   single-item with a trailing
  comma,
   or empty.
- `invocation-depth-per-line.ts` contains single-line comma-less constructs
  (`a(b(c()), other)`,
   `import(b(c()), opts)`,
   `a({ value: b(c()) })`) that the
  rule will flag.
   Its test filters to `stylistic(invocation-depth-per-line)` and
  does not assert an empty array,
   so the new diagnostics do not break it.
   Do not
  tighten that test to assert zero diagnostics.
   Either keep the filter or add
  trailing commas to those three constructs to keep the global fixture clean.

Invalid fixtures:
 most already trigger other rules,
 so assert the filtered
`stylistic(comma-dangle)` subset,
 never the full diagnostics array.
 Two need direct
attention:

- `invalid/fixable-trailing-comma.ts`:
   already fully comma-dangled,
   so `--fix`
  produces no comma changes.
   Use it to confirm the rule does not double-insert a
  comma after an existing one.
- `invalid/fixable.ts`:
   the autofix target that changes.
   The test file already
  holds two divergent assertions for it:
   `oxlint-stylistic.unit.test.ts:885`
  expects `  port: 3000,` (with comma) while `:1160` expects `  port: 3000\n`
  (without).
   Reconcile both:
   confirm which fix path each test exercises,
   and
  whether the comma rule makes `:885` redundant or `:1160` the only assertion that
  moves.
   Do not assume a single assertion moves.

## Fixture coverage checklist

Valid fixture should include:

- Single-item arrays,
   objects,
   patterns,
   function params,
   call arguments,
  imports,
   exports,
   tuples,
   generics,
   and enums with trailing commas.
- Empty arrays,
   objects,
   params,
   specifier lists,
   tuples,
   enums,
   and type
  parameter lists where syntax permits empty lists.
- Rest positions without required trailing comma:
  `function fn(...values: string[]) {}`,
   `const [...rest] = values`,
   and
  `const { ...rest } = value`.
- Default import and namespace import without named braces:
  `import value from 'pkg';` and `import * as value from 'pkg';`.

Invalid fixture should include missing commas for:

- `const values = [one]`.
- `const value = { one: 1 }`.
- `const [one] = values`.
- `const { one } = value`.
- `import { one } from 'pkg'`.
- `export { one }`.
- `function fn(one: string) {}`.
- `fn(one)`.
- `new Thing(one)`.
- `import(one)` and `import(one, options)`.
- `import data from 'data.json' with { type: 'json' }`.
- `export { value } from 'pkg' with { type: 'json' }`.
- `export * from 'pkg' with { type: 'json' }`.
- `enum Value { One }`.
- `function fn<T>() {}`.
- `type Pair = [string]`.
- `type Fn = (value: string) => void`.
- Type call,
   construct,
   method,
   constructor,
   and declare-function signatures.

Autofix fixture should verify that `--fix` inserts commas without rewriting
surrounding whitespace or comments.
Include at least one comment case:

```typescript
// package/test-fixture/oxlint-stylistic/src/invalid/comma-dangle.ts
const value = {
  one: 1 // keep comment
};
```

Expected fixed shape:

```typescript
// package/test-fixture/oxlint-stylistic/src/invalid/comma-dangle.ts
const value = {
  one: 1, // keep comment
};
```

## Test updates

Add a valid fixture test:

```typescript
// package/oxlint-plugin/stylistic/src/oxlint-stylistic.unit.test.ts
it({
  name: 'comma-dangle valid cases produce no violations',
  fn: async () => {
    const diagnostics = await lint('valid/comma-dangle.ts',);
    expect(diagnostics,).toEqual([],);
  },
},);
```

Add an invalid fixture test that filters only the new rule.
Do not assert the full diagnostics array if the fixture also triggers existing
rules.

```typescript
// package/oxlint-plugin/stylistic/src/oxlint-stylistic.unit.test.ts
it({
  name: 'reports missing trailing commas',
  fn: async () => {
    const diagnostics = await lint('invalid/comma-dangle.ts',);
    const commaDiagnostics = diagnostics.filter(function isCommaDangle(
      diagnostic,
    ): boolean {
      return diagnostic.code === 'stylistic(comma-dangle)';
    },);
    expect(commaDiagnostics.length,).toBeGreaterThan(0,);
    expect(commaDiagnostics[0]?.message,).toBe('Missing trailing comma.',);
  },
},);
```

Add an option rejection test mirroring the `semi` test if practical:

```jsonc
// package/test-fixture/oxlint-stylistic/.oxlintrc.comma-dangle-configured.fixture.json
{
  "$schema": "../../../node_modules/oxlint/configuration_schema.json",
  "jsPlugins": [
    "../../oxlint-plugin/stylistic/src/index.ts"
  ],
  "rules": {
    "stylistic/comma-dangle": [
      "error",
      "always"
    ]
  }
}
```

Expected stdout substring:

```text
Rule 'stylistic/comma-dangle' does not accept options
```

Add an autofix test with a temp fixture copy,
 following the existing `semi` and
`fixable.ts` tests.
The test should check representative inserted commas and then re-lint the fixed
copy.

Existing `invalid/fixable.ts` autofix assertions need updates after the rule is
enabled globally.
See "Existing fixture impact" for the `:885`/`:1160` divergence to reconcile;
 the
two autofix paths disagree today,
 so do not assume a single assertion moves.

## Verification commands

Run package-scoped tasks,
 not raw `oxlint` or `bun test`.
The available package tasks were measured with `mise tasks ls --all --name-only`:

```bash
mise run //package/oxlint-plugin/stylistic:lint:types; mise run //package/oxlint-plugin/stylistic:test:unit
```

Also run package oxlint after TypeScript passes:

```bash
mise run //package/oxlint-plugin/stylistic:lint:oxlint
```

If a targeted real-oxlint boundary check is needed,
 add a mise task for it before
running it.
Do not call raw `oxlint` directly from the agent shell.

## Definition of done

The next session is done when:

- `stylistic/comma-dangle` is registered and enabled in the fixture config.
- Rule options are rejected.
- Missing trailing commas report `Missing trailing comma.`.
- `--fix` inserts commas for every covered syntax category.
- Empty lists and rest positions do not report.
- Existing fixture tests account for the new rule's diagnostics and autofix
  output.
- `README.md` documents the rule.
- `mise run //package/oxlint-plugin/stylistic:lint:types` passes.
- `mise run //package/oxlint-plugin/stylistic:test:unit` passes.
- `mise run //package/oxlint-plugin/stylistic:lint:oxlint` passes.
