# @monochromatic-dev/oxlint-plugin-stylistic

Oxlint JS plugin for source-layout rules:
configurable TSDoc asterisk-prefix formatting,
one-item-per-line formatting across multi-element constructs,
readable newline boundaries inside brace-delimited bodies,
statement-boundary semicolon enforcement,
trailing comma enforcement,
and explicit operator structure in nested expressions.

The per-line rules fire when 2 or more items share a source line
and auto-fix by placing every item on its own line with consistent indentation.
dprint's TypeScript formatter is disabled in this repository (replaced by this plugin),
so this plugin is the active layout authority for TypeScript expressions;
dprint still formats the other languages.

The expression-structure rules surface ambiguous operator precedence
by requiring explicit parentheses at operator boundaries.

## Rules

### Comment layout

- **require-asterisk-prefix**:
   requires one explicit `"always"` or `"never"` option and applies it to every body line of every TSDoc block,
  including blank lines.
  `always` requires canonical `*` prefixes;
  `never` forbids them while preserving literal-leading Markdown such as `**Note**` and `*through*`.
  Opening and closing delimiters are exempt.
  The rule reports every offending line and auto-fixes both directions.
  Configure star-less style as `"stylistic/require-asterisk-prefix": ["warn", "never"]`.

### Per-line rules

All per-line rules are auto-fixable via `oxlint --fix`.

- **param-per-line**:
   each function parameter on its own line.
  Covers `FunctionDeclaration`,
   `FunctionExpression`,
   `ArrowFunctionExpression`
  and the full TypeScript function-like set:
  `TSFunctionType`,
   `TSDeclareFunction`,
   `TSMethodSignature`,
  `TSCallSignatureDeclaration`,
   `TSConstructSignatureDeclaration`,
  `TSConstructorType`,
   `TSEmptyBodyFunctionExpression`.
- **argument-per-line**:
   each function call argument on its own line (`CallExpression`,
   `NewExpression`)
- **array-element-per-line**:
   each array literal element on its own line
- **object-property-per-line**:
   each object literal property on its own line
- **import-per-line**:
   each named import specifier on its own line
- **export-per-line**:
   each named export specifier on its own line
- **type-property-per-line**:
   each type literal or interface member on its own line
- **tuple-per-line**:
   each tuple type element on its own line
- **destructure-per-line**:
   each destructured property on its own line

### Statement boundaries

- **block-body-newline**:
   require every non-empty brace-delimited body to put its
  first token or comment on a line after `{` and its closing `}` on a line after
  the final token or comment.
  Covers `BlockStatement` bodies from functions,
   arrow functions,
   methods,
  control-flow statements,
   loops,
   `try`,
   `catch`,
   and `finally`;
   also covers
  `SwitchStatement`,
   `StaticBlock`,
   `ClassBody`,
   and `TSModuleBlock`.
  Auto-fixable;
   the fixer only replaces whitespace after `{` and before `}` with
  newlines plus existing indentation,
   so comments at the start or end of a block
  remain in place.
  Empty blocks with no tokens or comments between braces are allowed inline,
   so
  `function noop(): void {}` is valid.
  Comment-only blocks are non-empty and must be split,
   for example
  `function f(): void { /* note */ }` becomes a three-line block.
  The rule does not enforce Allman,
   Stroustrup,
   one-true-brace-style,
   or spacing
  between `}` and `else`,
   `catch`,
   or `finally`.
- **one-var-declaration-per-line**:
   each declarator in a `var`/`let`/`const`/`using` declaration on its own line.
  Operates in `'always'` mode:
   every multi-declarator declaration is flagged
  regardless of whether declarators have initializers.
  Auto-fixable;
   fix is suppressed if a comment lives between declarators (preserves the comment).
- **max-statements-per-line**:
   at most one statement per source line.
  Single-child container parents (`if`/`while`/`for`/`labeled`/`export-default`/`export-named`)
  exempt their inner statement,
   so `if (a) foo();` is allowed;
   the alternate branch of `if`/`else`
  is not exempt,
   so `if (a) foo(); else bar();` flags `bar()`.
  Auto-fixable;
   fix is suppressed if a comment lives between statements (preserves the comment).
- **semi**:
   require semicolons at the end of statement-like declarations and expressions.
  Mirrors `@stylistic/semi` in its default `"always"` mode only.
  The rule has no options:
   configure it as `"stylistic/semi": "error"`,
   not as
  `["error", "always"]`.
  Auto-fixable;
   the fixer inserts `;` after the node's last syntax token.

### List boundaries

- **comma-dangle**:
   require trailing commas in supported comma-delimited lists.
  Mirrors `@stylistic/comma-dangle` in plain `"always"` mode only.
  The rule has no options:
   configure it as `"stylistic/comma-dangle": "error"`,
   not as
  `["error", "always"]`.
  Auto-fixable;
   the fixer inserts `,` after the last syntax token before the list's closing delimiter.
  It covers arrays,
   objects,
   array and object patterns,
   named import and export specifiers,
  import attributes,
   function parameters,
   call and constructor arguments,
   dynamic imports,
  enum members,
   type parameter declarations,
   tuple types,
   and TypeScript function-like signatures.
  Empty lists are ignored.
  Final rest elements are ignored because JavaScript grammar rejects a trailing comma there.
  Use-site type arguments are not covered:
   `new Set<string>()` and `fn<A, B>()` do not receive
  trailing commas because oxlint exposes them as `TSTypeParameterInstantiation`,
   not as
  `TSTypeParameterDeclaration`.

### Expression structure

- **no-mixed-operators**:
   require parentheses around nested binary or logical expressions
  whose operator differs from the parent.
  Same-operator chains (`a + b + c`,
   `x && y && z`) are permitted because they are unambiguous under associativity,
  except chained exponentiation.
  Because `**` associates from the right,
   write `a ** (b ** c)` instead of `a ** b ** c`.
  Mixed operators (`a + b * c`,
   `x || y && z`) must also be disambiguated with explicit parens.
  This includes a unary negative literal in a comparison:
   write `index === (-1)` rather than `index === -1`.
  Parenthesizing the literal is the accepted existence-check idiom and is compatible with
   `unicorn/consistent-existence-index-check`.
  The autofix wraps each offending nested operand while preserving the AST's existing grouping.
- **chain-per-line**:
   require one chain segment per source line for binary,
   logical,
  member,
   and call chains,
   laying out the operator and member axes independently.
  The message is `Put each operator, member, or method step in this chain on its own line.`
  Firing once on the outermost chain root,
   the rule computes break offsets on decoupled
  axes:
   a member or call chain breaks on its own member-step count,
   and an operator chain
  breaks on its own operator count,
   so neither axis inflates the other.
  A break point is a member-name step (`.name`,
   `?.name`) or a binary or logical
  operator's right operand (the operator renders leading,
   for example `+ c`);
  call steps (`(args)`) and computed steps (`[expr]`,
   `?.[expr]`) are attached and ride on
  the line before them.
  A member or call chain keeps the leaf and the first member step on the head line and
  breaks every later step.
  An operator chain keeps the source-first operator on the head line and breaks the rest;
  but when any operand's own member chain breaks,
   every operator also moves to its own
  line.
  The two axes do not interact,
   so a single operator whose operand is a one-step member
  access stays on one line:
   `a.b === c` and `task.dueDate ?? '?'` do not break.
  A multi-step member operand does break,
   carrying its operator onto a line of its own:
  `a.b.c > 0` becomes `a.b` then `.c` then `> 0`.
  Short chains stay on one line:
   `obj.method()`,
   `arr[0][1]`,
   `a + b`,
   `a.b === c`.
  Multi-step chains break:
   `obj.foo.bar` becomes `obj.foo` then `.bar`;
  `a + b + c + d` keeps `a + b` on the head line then `+ c` then `+ d`;
  `arr.map(f).filter(g)`,
   `obj.a.b.c`,
   and `foo().bar()` all split.
  Covers `BinaryExpression`,
   `LogicalExpression`,
   `MemberExpression`,
   `CallExpression`,
  including optional chaining (`?.`),
   computed access (`a[b]`),
   call chains (`f().g()`),
  TypeScript type arguments (`a.b<T>().c`),
   right-associative `**`,
   and chains threaded
  through `!`,
   `as`,
   and `satisfies` (`a.b!.c.d` splits).
  Grouping parentheses isolate inner chains (`(a + b) + c` keeps `(a + b)` opaque);
  the rule trusts `no-mixed-operators` to have parenthesized mixed precedence,
   so it
  flattens an unparenthesized operator run as one chain regardless of operator text.
  The autofix inserts newlines only at break offsets and slices everything else verbatim,
  so operand text survives,
   it leaves no trailing whitespace,
   and it converges to a fixed
  point.
  It never collapses whitespace at non-break points:
   a legacy split that the decoupled
  rule no longer breaks (no break offsets) is left verbatim and goes unreported rather
  than being rejoined onto one line.
  The fix is suppressed (the rule reports without a fix) only when a comment sits in the
  collapsible head,
   before the first break offset;
  a comment at or after the first break rides verbatim on its continuation slice,
   so the
  fix still applies (for example a comment buried in a trailing call's arguments).
  Tagged templates (`` tag`x` ``) and `new` expressions are opaque leaves,
   not chain
  participants.
  When `no-mixed-operators` and `chain-per-line` both apply to one region,
   several
  `oxlint --fix` passes are needed (an upstream single-pass limitation):
   the chain
  breaks,
   the wrap adds parentheses,
   then the now-nested inner chain re-indents,
   so a
  caller should run `--fix` until the file stops changing;
  `no-mixed-operators` also parenthesizes same-precedence mixed-operator runs such as
  `a + b - c`,
   so in the combined config that case becomes `(a + b) - c` rather than a
  flattened chain.
- **invocation-depth-per-line**:
   allow at most two nested invocation heads on one source line.
  The message is `No more than two nested invocations may start on one line; split the operand onto its own line.`
  A counted invocation is a `CallExpression` (including optional calls),
   `NewExpression`,
   or
  `ImportExpression`.
  The rule walks each operand spine,
   the chain of single-argument invocations threaded through
  transparent wrappers,
   and counts how many invocation heads begin on each source line;
  a line carrying three or more heads fails.
  Transparent wrappers the spine passes through:
   `await`,
   unary operators (`void`,
   `!`,
   `typeof`),
  `yield`/`yield*`,
   a single spread argument,
   optional-chaining (`?.`),
   and the TypeScript
  `as`,
   `satisfies`,
   `!`,
   type-assertion,
   and instantiation expressions.
  Grouping parentheses are not a wrapper node (oxlint strips them),
   so `a((b(c())))` counts as
  depth three.
  The rule is threshold-only:
   an already-split layout passes when every line stays within the
  limit,
   so `a(b(` then `c(),` then `))` is allowed.
  Multi-argument calls belong to `argument-per-line` and callee chains to `chain-per-line`,
   so this
  rule descends only the single operand and leaves those axes alone;
  a dynamic import with options (`import(x, opts)`) stops the source spine,
   and a tagged template
  or object literal breaks the parent spine while child spines inside it are still checked.
  The autofix splits the highest invocation on each violating line:
   the operand moves to its own
  line indented two spaces,
   a trailing comma is added (before any trailing line or block comment),
  and the closing bracket returns to the head-line indent.
  Grouping parentheses around the operand and template-literal quasis are preserved verbatim.
  Each report splits one level,
   so a deep spine and any overlap with `argument-per-line`,
  `object-property-per-line`,
   or `array-element-per-line` converge over several `oxlint --fix`
  passes (the same upstream single-pass limitation `chain-per-line` notes above).

## Usage

Reference the package in `oxlint.config.ts`:

```typescript
// oxlint.config.ts
import { defineConfig, } from 'oxlint';
export default defineConfig({
  jsPlugins: ['@monochromatic-dev/oxlint-plugin-stylistic',],
},);
```

The package default export resolves to the prebuilt,
 self-contained
`dist/final/node/index.mjs` (run `mise run //package/oxlint-plugin/stylistic:build` first).
TypeScript source is available at the `/ts` subpath (`/ts/*` for individual files) for
development.

## Design decisions

**Union/intersection types excluded**:
 a `union-per-line` rule was prototyped but dropped.
Inline type assertions like `Span & Record<string, unknown>` and `T | null | undefined`
generated too much noise to be worth enforcing.

**Blank-line rules excluded**:
 `lines-between-*`,
 `lines-around-*`,
 and `padding-line-between-*`
rules are intentionally out of scope.
Vertical whitespace stays with dprint plus local judgment;
this plugin enforces item boundaries and expression structure only.

**Minimum 2 items for per-line rules**:
 single-item constructs are never flagged by per-line rules.
An array with one element or a function with one parameter stays on one line.
`comma-dangle` is separate and still requires a trailing comma for a supported single-item list.

**Shared per-line implementation**:
 per-line rules delegate to `checkItemsPerLine` in the utility layer.
Each per-line visitor only extracts the relevant container and items from the AST,
keeping those rule files minimal.

## Source files

- `index.ts`:
   plugin entry point;
   assembles all rules into the oxlint plugin object
- `rule/`:
   one file per rule,
   each exporting a `CreateOnceRule`;
  `require-asterisk-prefix.ts` reads configured mode during `Program`,
  when Oxlint has installed per-file options
- `utility/item-per-line.ts`:
   shared detection and reporting logic for per-line rules
- `utility/item-per-line-fix.ts`:
   shared autofix logic (indentation detection,
   content rebuild)
- `utility/per-line-boundary.ts`:
   explicit container boundary offsets for item-per-line rules
- `utility/needs-fix.ts`:
   line-sharing detection between items and container delimiters
- `utility/source-filler.ts`:
   shared whitespace,
   semicolon,
   and comma filler checks between syntax nodes
- `utility/comma-dangle.ts`:
   shared trailing comma token lookup and reporting helpers
- `utility/block-body-newline.ts`:
   shared brace token lookup,
   content detection,
   line lookup,
  and nested dense-body indentation helpers
- `utility/range.ts`:
   `rangeOf` and `at` helpers for untyped AST node access
- `utility/line-at.ts` and `utility/indent.ts`:
   source-line lookup and indentation helpers
- `utility/has-parens.ts`:
   source-level paren detection for `no-mixed-operators`
- `utility/chain.ts`,
  `utility/chain-flatten.ts`,
  and `utility/chain-render.ts`:
   chain-root detection,
   iterative break-offset calculation,
   and canonical rendering for `chain-per-line`
- `utility/invocation-spine.ts` and `utility/invocation-depth-fix.ts`:
   invocation-spine traversal and autofix rendering for `invocation-depth-per-line`

## Tests

Fixture-based tests cover all rules plus autofix verification,
and focused utility tests cover chain and indentation helpers:

```bash
mise run buildAndTest -- package/oxlint-plugin/stylistic/src/oxlint-stylistic.unit.test.ts
mise run //package/oxlint-plugin/stylistic:test:unit
```

Test fixtures live in `package/test-fixture/oxlint-stylistic/src/` with `valid/` and `invalid/` directories.
Dedicated configs verify `always`,
`never`,
missing-mode,
and invalid-mode behavior.
