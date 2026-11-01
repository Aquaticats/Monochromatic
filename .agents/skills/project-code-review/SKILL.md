---
name: project-code-review
description: Review code according to project standards
---

# Code review

Structured code review that checks changes for correctness,
 type safety,
 security,
 style,
 and maintainability.
Produces actionable findings categorized by severity.

All underlying rules referenced below are defined in `AGENTS.md`.

## Process

Read the diff or files under review,
 then evaluate each category below.
Skip categories that do not apply to the language or change.

### Correctness

- Logic errors,
   off-by-one,
   unhandled edge cases
- Missing null/undefined checks
- Race conditions in async code
- Incorrect use of APIs or library methods
- Broken error propagation (swallowed exceptions,
   silent catch blocks)

#### Off-by-one and boundary errors

```ts
// Bad -- flag as WARNING
const lastItem = items[items.length];

// Good
const lastItem = items.at(-1,);
```

```ts
// Bad -- flag as WARNING: skips the last element
for (let index = 0; index < items.length - 1; index++) { ... }

// Good
// Looping is unavoidable here because each item has side effects
for (const item of items) { ... }
```

#### Missing null/undefined checks

```ts
// Bad -- flag as BLOCKER
function getUser(id: string,): User {
  const user = users.find(candidate => candidate.id === id);
  return user; // may be undefined
}

// Good
function getUser(id: string,): User {
  return notNullishOrThrow(users.find(function matchesId(candidate,) {
    return candidate.id === id;
  },),);
}
```

#### Race conditions

```ts
// Bad -- flag as WARNING: shared mutable state with concurrent access
const cache: Map<string, Data> = new Map();

async function getData(key: string,): Promise<Data> {
  if (!cache.has(key,)) {
    const data = await fetchData(key,);
    cache.set(key, data,); // another call may have set it while awaiting
  }
  return notNullishOrThrow(cache.get(key,),);
}
```

#### Broken error propagation

```ts
// Bad -- flag as WARNING: swallowed exception
try {
  await saveRecord(record,);
}
catch {
  // silently ignored
}

// Good
try {
  await saveRecord(record,);
}
catch (error) {
  l.error(`failed to save record: ${String(error,)}`,);
  throw new Error('Failed to save record', { cause: error, },);
}
```

### Type safety (TypeScript)

- Explicit return types on all functions
- `unknown` over `any`;
   no bare `Function` type
- No non-null assertions (`!`);
   use assertion functions or narrowing
- Branded types for domain primitives where appropriate
- `satisfies` over `as` when validating shape without widening
- `const` generic parameters;
   `readonly` array parameters
- Proper discriminated unions;
   narrow `typeof === 'symbol'` before identity checks

#### Explicit return types

```ts
// Bad -- flag as WARNING
function parseConfig(raw: string,) {
  return JSON.parse(raw,);
}

// Good
function parseConfig(raw: string,): Config {
  return JSON.parse(raw,) as Config;
}
```

#### `unknown` over `any`

```ts
// Bad -- flag as WARNING
function processInput(data: any): void { ... }

// Bad -- flag as WARNING: bare Function type
function runCallback(callback: Function): void { ... }

// Good
function processInput(data: unknown): void { ... }

// Good
function runCallback(callback: () => void): void { ... }
```

#### Domain-specific types

```ts
// Bad -- flag as NIT: primitive type loses domain meaning
function getUser({ id }: { id: string }): User { ... }
function setPermissions({ level }: { level: number }): void { ... }

// Good -- branded type for opaque identifiers
type UserId = string & { readonly __brand: 'UserId' };
function getUser({ id }: { id: UserId }): User { ... }

// Good -- union type for small finite sets
type PermissionLevel = 1 | 2 | 3;
function setPermissions({ level }: { level: PermissionLevel }): void { ... }

// Good -- template literal type for structured strings
type SemVer = `${number}.${number}.${number}`;
function parseVersion({ version }: { version: SemVer }): VersionInfo { ... }
```

#### Explicit type annotations and `satisfies`

```ts
// Bad -- flag as WARNING: as widens the type
const config = { host: 'localhost', port: 8080, } as ServerConfig;

// Good -- explicit type annotation on the variable when possible
const config: ServerConfig = { host: 'localhost', port: 8080, };

// Good -- satisfies when direct annotation is not possible (e.g. exported configs)
export default { host: 'localhost', port: 8080, } satisfies ServerConfig;
```

#### Symbol union narrowing

Flag code that compares a value to a specific symbol and assumes the else branch is non-symbol:

```ts
// Anti-pattern -- flag as BLOCKER
if (out === NO_LITERAL) {
  // handle sentinel
}
else {
  // BUG: else may still be a symbol from the union
  use(out.parsed,);
}

// Correct pattern
if (typeof out === 'symbol') {
  if (out === NO_LITERAL) {
    // handle sentinel
  }
  else {
    throw new Error(`Unexpected symbol: ${String(out,)}`,);
  }
}
else {
  use(out.parsed,);
}
```

#### Generics

Flag missing `const` or `readonly` modifiers:

```ts
// Bad -- flag as WARNING
function processItems<T extends { id: string; },>(items: T[],): T[];

// Good
function processItems<const T extends { id: string; },>(
  items: readonly T[],
): readonly T[];

// Bad -- flag as WARNING
function myFn<const T,>(myArr: T[],): T[];

// Good
function myFn<const T,>(myArr: readonly T[],): T[];
```

Flag non-descriptive generic names:

```ts
// Bad -- flag as NIT
<T extends Record<string, unknown>>

// Good
<TUser extends Record<string, unknown>>
```

### Function signatures

- Functions with 2+ parameters must use a single destructured object parameter (named params)
- No rest parameters (`...args`) in functions we control;
   accept an array parameter instead
- No `const x = function() {}` -- use a function declaration instead
- No calling functions before their declaration in source order
- Exempt:
   callbacks whose signature is dictated by an external API or library (e.g. `.map()`,
   `.sort()`,
   event handlers)

#### No variable function expressions

Flag function expressions assigned to variables:

```ts
// Bad -- flag as WARNING
const greet = function greet(name: string,): void {};
const greet = function(name: string,): void {};

// Good -- use a function declaration
function greet(name: string,): void {}
```

#### Named parameters

Flag any function declaration or named arrow function with 2+ positional parameters:

```ts
// Bad -- flag as WARNING
function clamp(value: number, min: number, max: number,): number {
  return Math.max(min, Math.min(max, value,),);
}

// Good
function clamp(
  { value, min, max, }: { value: number; min: number; max: number; },
): number {
  return Math.max(min, Math.min(max, value,),);
}
```

```ts
// Bad -- flag as WARNING
function fetchUser(id: string, signal: AbortSignal): Promise<User> { ... }

// Good
function fetchUser({ id, signal }: { id: string; signal: AbortSignal }): Promise<User> { ... }
```

Callbacks passed to external APIs are exempt because the caller dictates the signature:

```ts
// OK -- signature dictated by Array.prototype.map
const doubled = items.map(function doubleByIndex(item, index,) {
  return multiply({ value: item, by: index, },);
},);

// OK -- signature dictated by Array.prototype.sort
const sorted = items.sort(function byPriority(left, right,) {
  return left.priority - right.priority;
},);
```

#### No use before declaration

Flag function calls that appear before the function's declaration in source order.
Hoisting makes this legal at runtime but breaks top-down readability:

```ts
// Bad -- flag as WARNING: b() called before its declaration
const a = b();

function b(): string {
  return 'hello';
}

// Good -- declaration before use
function b(): string {
  return 'hello';
}

const a = b();
```

#### Rest parameters

Flag rest parameters in functions we control:

```ts
// Bad -- flag as WARNING
function logMessages(...messages: readonly string[]): void { ... }

// Good
function logMessages({ messages }: { messages: readonly string[] }): void { ... }
```

### Immutability and style

- `const` over `let`;
   no unnecessary mutation
- **Justify or refactor**:
   any deviation from the preferred pattern must have a comment explaining why refactoring
to the preferred approach is not feasible;
   if no justification is given,
   flag it as WARNING
- Functional patterns (`map`/`filter`/`reduce`) over imperative loops
- `for...of` when iteration is unavoidable,
   never classic `for` loops
- No single-letter variables (except math formulas)
- Magic numbers/strings extracted to named constants (0,
   1,
   2,
   -1,
   -2 exempt);
   when a named constant's value is composed from exempt-range arithmetic (e.g. `1 / (2 * 2)` for 1/4,
   `(16 - 2 - 1) / 16` for 13/16),
   do not flag the expression as a readability issue
- Extract and name complex conditions

#### `const` over `let`

```ts
// Bad -- flag as WARNING
let baseUrl = 'https://api.example.com';

// Good
const baseUrl = 'https://api.example.com';
```

```ts
// Bad -- flag as WARNING: accumulator mutated via let
let total = 0;
for (const item of items)
  total += item.price;

// Good
const total = items.reduce(function addPrice(sum, item,) {
  return sum + item.price;
}, 0,);
```

#### Magic numbers and strings

```ts
// Bad -- flag as WARNING
if (retries > 3) { ... }
await wait(5000);

// Good
const maxRetries = 3;
const retryDelayMs = 5000;
if (retries > maxRetries) { ... }
await wait(retryDelayMs);
```

```ts
// OK -- exempt-range composition in a named constant; do NOT flag
const BORDER_RADIUS = 1 / (2 * 2);
const FONT_SIZE = (16 - 2 - 1) / 16;
```

#### `for...of` over classic `for`

```ts
// Bad -- flag as WARNING
for (let index = 0; index < items.length; index++)
  process(items[index],);

// Good (when functional patterns do not apply)
// Iteration is unavoidable because process() has side effects
for (const item of items)
  process(item,);
```

#### Functional over imperative

Flag imperative patterns when functional alternatives exist:

```ts
// Bad -- flag as WARNING
let results = [];
for (let i = 0; i < items.length; i++) {
  if (items[i].isActive)
    results.push(items[i].value * 2,);
}

// Good
const results = items
  .filter(function isActive(item,) {
    return item.isActive;
  },)
  .map(function doubleValue(item,) {
    return item.value * 2;
  },);
```

#### Object iteration

Flag `for...in` loops on objects:

```ts
// Bad -- flag as WARNING
for (const key in obj) {
  if (Object.prototype.hasOwnProperty.call(obj, key,))
    result[key] = process(obj[key],);
}

// Good
Object.entries(obj,).forEach(function applyProcess([key, value,],) {
  result[key] = process(value,);
},);

// Good (for transformations)
const result = Object.fromEntries(
  Object.entries(obj,).map(function processEntry([key, value,],) {
    return [key, process(value,),];
  },),
);
```

#### No `switch` statements

Flag all `switch` statements.
 Use if/else chains for branching or `Record` lookups for mapping a discriminant to a value.

`switch` adds `break` boilerplate,
 risks accidental fallthrough,
 and encourages large blocks.
If/else works naturally with early returns and needs no special syntax.
When the logic is a pure mapping from key to value,
 a `Record` is more declarative and type-safe.

```ts
// Bad -- flag as WARNING: switch statement
switch (toolName) {
  case 'Read': {
    return `Reading ${shortPath(filePath,)}`;
  }
  case 'Edit': {
    return `Editing ${shortPath(filePath,)}`;
  }
  case 'Bash': {
    return `Running ${shortCommand(command,)}`;
  }
  default: {
    return toolName;
  }
}

// Good -- if/else for branching with logic
if (toolName === 'Read')
  return `Reading ${shortPath(filePath,)}`;
else if (toolName === 'Edit')
  return `Editing ${shortPath(filePath,)}`;
else if (toolName === 'Bash')
  return `Running ${shortCommand(command,)}`;
else
  return toolName;

// Good -- Record lookup for pure mappings
const TOOL_LABELS: Record<string, (input: ToolInput,) => string> = {
  Read: function readLabel({ filePath, },) {
    return `Reading ${shortPath(filePath,)}`;
  },
  Edit: function editLabel({ filePath, },) {
    return `Editing ${shortPath(filePath,)}`;
  },
  Bash: function bashLabel({ command, },) {
    return `Running ${shortCommand(command,)}`;
  },
};

const labelFn = TOOL_LABELS[toolName];
return labelFn !== undefined ? labelFn(input,) : toolName;
```

#### Naming extracted concepts

Flag inline complex conditions:

```ts
// Bad -- flag as NIT
if (status === 'pending' && retries < maxRetries && !isTimeout) {
  // retry logic
}

// Good
function canRetry(): boolean {
  return status === 'pending' && retries < maxRetries && !isTimeout;
}

if (canRetry()) {
  // retry logic
}
```

#### Single-letter variables

```ts
// Bad -- flag as WARNING
for (let i = 0; i < items.length; i++)

// Good
for (let itemIndex = 0; itemIndex < items.length; itemIndex++)
items.forEach(function processItem(item, itemIndex) { ... })
```

### Duplication and pattern ownership

Two checks that apply to every change,
not only to duplicate-code reports.
The three-letter codes are repository-wide handles,
unique across `AGENTS.md`,
the skills and the package docs,
so other documents can cite them.

`GRW` A repeated shape needs an owner.
When a diff adds,
keeps or approves a shape that already exists elsewhere,
name what owns the pattern:
a shared function,
a base class,
a macro,
a factory,
a lint rule,
or a written convention.
"It is only two places" and "it is too small to extract" are not answers,
because a pattern instantiated twice is a template for the third copy and the count then grows
without anyone deciding.
If nothing should own it,
say why the pattern is closed instead:
it cannot recur
(a protocol fixed elsewhere,
a pair of complements,
one instance per platform),
or each instance is data rather than code
(a test case,
a probe input,
a benchmark arm).

Severity:
WARNING when the shape exists in two places,
BLOCKER when the diff adds a third copy or the shape already exists at ten or more sites.

```ts
// Bad -- flag as WARNING: one policy, two spellings, no owner
function isNonEmptyLine(line: string,): boolean {
  return line !== '';
}

// elsewhere in the repository
function hasText(line: string,): boolean {
  return line.length > 0;
}

// Good: one owner, and every call site imports it
import { isNonEmptyLine, } from '@monochromatic-dev/agent-harness-shared-text-scan';
```

Measured example from this repository:
`this.name = '...'` sits at 210 sites across 146 files,
every one hand-written,
and a duplicate-code review dismissed twelve clusters of them as trivial boilerplate because no
rule asked who owns the pattern.
The idiom was two sites when it was first written.

`TWS` Two-sided code states its choice.
When a change adds or keeps two sides of one behaviour
(sync and async,
mirror event handlers,
complementary predicates,
per-arity overloads,
fork and upstream adapters,
per-locale renderers),
the diff or its commit message names which of three arrangements applies:
the sides share a core,
one side is generated from the other,
or both are hand-written on purpose.
Silence is how a repository reaches 22 hand-written sync and async pairs and 519 duplicated tokens
between `runPipe` and `runPipeAsync`.

Exception:
where identical code on both sides is what makes an observed difference meaningful,
as in differential-fuzz adapters,
hand-writing both is the method and the commit says so.

Severity:
WARNING for a new pair with no stated choice,
BLOCKER when the pair shares roughly 50 or more tokens of identical body and still has none.

```ts
// Bad -- flag as WARNING: a new twin, with no stated choice anywhere
export function readCssFile(path: string,): string { ... }
export async function readCssFileAsync(path: string,): Promise<string> { ... }

// Good: the choice is recorded where the pair is declared
/**
 Both sides are hand-written. The sync path serves build startup and cannot
 await; the async path serves plugins. A shared core was rejected because the
 sync side cannot call it.
 */
export function readCssFile(path: string,): string { ... }
```

### Security

- No hardcoded secrets,
   API keys,
   or credentials
- No unsanitized user input in SQL,
   shell commands,
   or HTML
- No overly permissive CORS,
   file permissions,
   or network exposure
- Secrets not logged,
   even at debug level

#### Hardcoded secrets

```ts
// Bad -- flag as BLOCKER
const apiKey = 'sk-live-abc123def456';

// Good
const apiKey = notNullishOrThrow(process.env['API_KEY'],);
```

#### Unsanitized user input

```ts
// Bad -- flag as BLOCKER: shell injection
const output = execSync(`grep ${userQuery} /var/log/app.log`,);

// Good
const output = execSync('grep', [userQuery, '/var/log/app.log',],);
```

```ts
// Bad -- flag as BLOCKER: SQL injection
const rows = db.query(`SELECT * FROM users WHERE name = '${name}'`,);

// Good
const rows = db.query('SELECT * FROM users WHERE name = ?', [name,],);
```

#### Secrets in logs

```ts
// Bad -- flag as BLOCKER
console.log('Authenticating with token:', token,);

// Good
const tokenPrefixLength = 4;
console.log('Authenticating with token:',
  token.slice(0, tokenPrefixLength,) + '...',);
```

### Naming and readability

- Semantic,
   descriptive names for variables,
   functions,
   types
- Comments explain WHY,
   not WHAT
- TSDoc on **all** declarations (functions,
   types,
   constants,
   classes -- including locals inside function bodies) with `@param`,
   `@returns`,
   `@example`;
   do not skip declarations that seem "obvious from context"
- `@throws` only on functions that actually throw;
   do not add `@throws` to functions that never throw
- Comments on their own line above code,
   never inline after code
- No narrative or promotional language in docs
- Region markers for substantial code blocks;
   flag missing markers as WARNING for substantial blocks and NIT for smaller ones

#### Comments inside template literals

Embed comments inside template literals using the `${''}` trick.
Do not use target-language comments (XML,
 HTML,
 etc.) or move the comment outside the template.

Reasons:

- Target-language comments require context switching between JS and the target language
- Editors cannot properly highlight target-language comments inside JS template literals
- Target-language comment syntax is easy to get wrong or forget across different languages

```ts
// Bad -- flag as NIT: target-language comment
const xml = `
  <os><type arch='x86_64'>hvm</type></os>
  <!-- Guest CPU requires AVX -->
  <cpu mode='host-passthrough'/>
`;

// Bad -- flag as NIT: comment disconnected from the line it explains
// Guest CPU requires AVX
const xml = `
  <os><type arch='x86_64'>hvm</type></os>
  <cpu mode='host-passthrough'/>
`;

// Good
const xml = `
  <os><type arch='x86_64'>hvm</type></os>
  ${
  // Guest CPU requires AVX
  ''}
  <cpu mode='host-passthrough'/>
`;
```

#### TSDoc quality

Flag WHY-less comments:

```ts
// Bad -- flag as NIT
/** Line counter starting at 1 */

// Good
/** Mutable counter needed to track newlines encountered while scanning */
```

Flag TSDoc on non-declarations:

```ts
// Bad -- flag as WARNING: TSDoc on a statement
/** Increment the counter */
counter++;

// Good
// Increment the counter to account for the trailing newline
counter++;
```

#### Escaping block comment terminators

Flag unescaped `*/` inside TSDoc blocks:

```ts
// Bad -- flag as BLOCKER: premature comment termination
/**
 * Returns a string like "/* comment */"
 */

// Good
/**
 * Returns a string like "/* comment *\\/"
 */
```

### Async patterns

- `async`/`await` over raw promises or callbacks
- No `.then()`,
   `.catch()`,
   `.finally()` -- use `async`/`await` with `try`/`catch` or let errors propagate naturally by throwing
- `Promise.all` for independent concurrent operations
- `Promise.allSettled` when partial failure is acceptable
- No `await` in loops without justification and eslint-disable comment
- `AbortController` for cancellable operations
- Streams and subprocesses properly consumed or cleaned up

#### No `.then()`/`.catch()`/`.finally()`

```ts
// Bad -- flag as WARNING
function loadConfig(): Promise<Config> {
  return readFile('config.json', 'utf8',)
    .then(raw => JSON.parse(raw,) as Config)
    .catch(error => {
      l.error(`failed to load config: ${String(error,)}`,);
      throw error;
    },);
}

// Good
async function loadConfig(): Promise<Config> {
  const raw = await readFile('config.json', 'utf8',);
  return JSON.parse(raw,) as Config;
}
```

#### No `await` in loops

```ts
// Bad -- flag as WARNING: sequential when concurrency is possible
for (const url of urls) {
  const response = await fetch(url,);
  results.push(await response.json(),);
}

// Good
async function fetchJson(url: string,): Promise<unknown> {
  const response = await fetch(url,);
  return response.json();
}

const results = await Promise.all(urls.map(fetchJson,),);
```

#### Manual promise creation

Flag explicit `new Promise` when utilities exist:

```ts
// Bad -- flag as WARNING
function delay(ms,) {
  return new Promise(function resolveAfterDelay(resolve,) {
    setTimeout(resolve, ms,);
  },);
}

// Good
import { wait, } from '@monochromatic-dev/module-es';
// Use wait(ms) directly
```

### Imports and modules

- Grouped:
   builtins,
   external deps,
   workspace packages,
   relative,
   type-only
- Named imports over default imports
- `import type` for type-only imports
- File extensions in relative imports when required by config
- No circular dependencies

#### Import grouping and style

```ts
// Bad -- flag as WARNING: ungrouped, missing extension, default import, missing import type
import { readFile, } from 'node:fs/promises';
import { z, } from 'zod';
import config from './config';
import { Config, } from './types';

// Good
import { readFile, } from 'node:fs/promises';

import { z, } from 'zod';

import { parseConfig, } from '@monochromatic-dev/module-es';

import { loadSettings, } from './config.ts';

import type { Config, } from './types.ts';
```

### Error handling

- Prefer letting errors propagate by throwing
- No `try...finally` -- use `using`/`await using` with `Symbol.dispose`/`Symbol.asyncDispose` for cleanup
- Custom error classes extending `Error` for domain errors
- No `process.exit()`;
   throw instead
- Multi-line error messages use `outdent` from `@cspotcode/outdent`
- `@throws` in TSDoc only for functions that actually throw;
   flag `@throws` on non-throwing functions as NIT
- `notNullishOrThrow` instead of non-null assertion (`!`)
- Every `catch (error)` uses its binding:
   log the caught value (even expected) through a tagged logger or rethrow

#### `using` over `try...finally`

```ts
// Bad -- flag as WARNING
const handle = openResource();
try {
  await process(handle,);
}
finally {
  handle.close();
}

// Good
await using handle = openResource();
await process(handle,);
```

#### No `process.exit()`

```ts
// Bad -- flag as WARNING
if (!isValid) {
  console.error('Invalid input',);
  process.exit(1,);
}

// Good
if (!isValid)
  throw new Error('Invalid input',);
```

#### `outdent` for multi-line error messages

```ts
// Bad -- flag as NIT
throw new Error(
  'Failed to process record.\n'
    + `Expected type: ${expectedType}\n`
    + `Received type: ${receivedType}`,
);

// Good
import { outdent, } from '@cspotcode/outdent';

throw new Error(outdent`
  Failed to process record.
  Expected type: ${expectedType}
  Received type: ${receivedType}
`,);
```

#### Custom error classes

```ts
class ValidationError extends Error {
  constructor(
    message: string,
    public readonly field: string,
  ) {
    super(message,);
    this.name = 'ValidationError';
  }
}
```

#### Type guards

```ts
function isString(value: unknown,): value is string {
  return typeof value === 'string';
}
```

#### Assertion functions

```ts
function assertIsString(value: unknown,): asserts value is string {
  if (typeof value !== 'string')
    throw new TypeError('Expected string',);
}
```

#### notNullishOrThrow over non-null assertion

```ts
// Bad -- flag as BLOCKER
const value = possiblyUndefined!;

// Good
import { notNullishOrThrow, } from '@monochromatic-dev/module-es';
const value = notNullishOrThrow(possiblyUndefined,);
```

#### Silent error handling

Flag catch blocks that neither log the caught value nor rethrow:

```ts
// Bad -- flag as WARNING
catch (error) { }

// Good -- fatal error
catch (error) {
  throw new Error('Failed to get index stats', { cause: error, },);
}

// Good - expected error that shouldn't interrupt program operation.
catch (error) {
  l.error(`failed to get index stats: ${String(error,)}`,);
}
```

Flag silently discarded unexpected states:

```ts
// Bad -- flag as BLOCKER
if (!(event instanceof CustomEvent))
  return;

// Good
if (!(event instanceof CustomEvent))
  throw new TypeError('Expected CustomEvent',);
```

### Markdown quality

- No tables in markdown files -- use nested headings or lists instead
- Flag any existing tables in changed markdown files as WARNING with a suggestion to convert

#### Tables to lists

```md
<!-- Bad -- flag as WARNING -->

| Name | Type   | Default   |
| ---- | ------ | --------- |
| host | string | localhost |
| port | number | 8080      |

<!-- Good -->

- host -- `string`, default `localhost`
- port -- `number`, default `8080`
```

### Testing gaps

- New logic paths missing corresponding test cases
- Edge cases not covered (empty input,
   boundary values,
   error paths)
- Mocking that hides real integration issues
- Flaky patterns (timing dependencies,
   shared mutable state)

### Commit messages

When reviewing PRs or multi-commit bundles,
 also review commit messages.

#### Format

Conventional Commits:
 `<type>(<scope>): <subject>` with optional body and footer.
Types:
 `feat`,
 `fix`,
 `docs`,
 `style`,
 `refactor`,
 `perf`,
 `test`,
 `build`,
 `ci`,
 `chore`,
 `revert`.
Scope:
 package name or `*` for multi-package changes.

#### Single type and scope

```txt
feat(module-es): add a

Description of a
```

#### Multiple types and/or scopes

```txt
feat(module-es): enhance error handling utilities

error.assert.throw: add assertion-based error throwing
- Implement conditional error throwing based on assertions
- Include TypeScript type narrowing support

error.throw: add unified error throwing utility
- Implement consistent error creation
- Provide better stack traces
- Include custom error types

test(module-es): achieve 100% coverage for error utilities
- Add comprehensive test cases
- Use parameterized tests for edge cases
- Ensure proper type inference testing
```

#### Rules

- Group related changes by type (feat,
   fix,
   test,
   etc.)
- Do not mix different types in the same scope section
- Be specific about what changed,
   not just which files
- Focus on "what" and "why"
- Flag partial commit messages (describing only some changes) as WARNING

### CSS quality

When changes include CSS,
 check:

- Native platform features over JS reimplementations (`<dialog>`,
   Popover API,
   CSS nesting)
- `rem` for all sizing;
   `calc()` for derivation;
   no `px` except device-pixel contexts
- Logical properties everywhere (no physical `left`/`right`/`top`/`bottom`)
- No shorthand properties that combine unrelated axes or sub-properties;
   single-axis/single-concept shorthands are fine (`padding-inline`,
   `margin-block`,
   `border-radius`,
   `inset`,
   `gap`)
- Colors via CSS custom properties;
   no `var()` fallbacks (exception:
   user-configurable)
- No `!important`
- `:focus-visible` on interactive elements;
   `48px` minimum touch targets
- Shallow native nesting (3 levels max)
- Data attributes for state/variant styling,
   not BEM modifiers

#### Logical properties

```css
/* Bad -- flag as WARNING: physical properties */
margin-left: 1rem;
padding-top: 0.5rem;
text-align: left;
top: 0;
right: 0;

/* Good */
margin-inline-start: 1rem;
padding-block-start: 0.5rem;
text-align: start;
inset-block-start: 0;
inset-inline-end: 0;
```

#### `rem` sizing

```css
/* Bad -- flag as WARNING */
font-size: 14px;
padding-block: 8px;
border-radius: 4px;

/* Good */
font-size: calc(14 / 16 * 1rem);
padding-block: calc(8 / 16 * 1rem);
border-radius: calc(4 / 16 * 1rem);
```

#### `:focus-visible` and touch targets

```css
/* Bad -- flag as WARNING: :focus instead of :focus-visible */
button:focus {
  outline-color: var(--focus-ring);
}

/* Bad -- flag as WARNING: touch target too small */
button {
  min-inline-size: 2rem;
  min-block-size: 2rem;
}

/* Good */
button {
  min-inline-size: 3rem; /* 48px equivalent */
  min-block-size: 3rem;

  &:focus-visible {
    outline-color: var(--focus-ring);
  }
}
```

#### Shallow nesting

```css
/* Bad -- flag as WARNING: 4+ levels of nesting */
.card {
  & .header {
    & .title {
      & span {
        color: var(--accent-fg);
      }
    }
  }
}

/* Good -- max 3 levels */
.card {
  .title {
    & span {
      color: var(--accent-fg);
    }
  }
}
```

#### Shorthand properties

```css
/* Bad -- flag as WARNING: multi-axis/multi-concept shorthands */
border: 1px solid #111;
padding: 0.5rem 1rem;
margin: 0 auto;
background: #fff url(...) no-repeat center;

/* Good -- longhand for multi-concept properties */
border-width: calc(1 / 16 * 1rem);
border-style: solid;
border-color: var(--gray-fg);

/* Good -- single-axis/single-concept shorthands are fine */
padding-block: 0.5rem;
padding-inline: 1rem;
margin-block: 0;
margin-inline: auto;
border-radius: 0.25rem;
inset: 0;
gap: 1rem;
```

#### Color tokens

```css
/* Bad -- flag as WARNING */
color: #111;
border-color: #a00;

/* Good */
color: var(--gray-fg);
border-color: var(--error-fg);
```

#### State styling

```html
<!-- Bad -- flag as NIT -->
<span class="pill pill--loading">

<!-- Good -->
<span class="pill" data-loading>
```

```css
/* Bad */
.pill--loading {
  opacity: 0.5;
}

/* Good */
.pill {
  &[data-loading] {
    opacity: 0.5;
  }
}
```

### Logging

- All loggers must use the `tagged` wrapper;
   no raw `console.*` or untagged logger instances
- Tags at every function/module boundary using `myFn.name` or subsystem name
- Composed tags for nested calls -- pass `tagged({ tag, l })` down the call chain
- No manual tag prefixes in message strings

#### Untagged logger

```ts
// Bad -- flag as WARNING: untagged logger
export const l: Logger = $;

// Good
export const l: Logger = tagged({ tag: 'rss', },);
```

#### Manual tag in message string

```ts
// Bad -- flag as WARNING: manual tag prefix
l.info('[cycle] capture complete',);

// Good
const l = tagged({ tag: 'cycle', l: parentLogger, },);
l.info('capture complete',);
```

#### Shallow tagging

```ts
// Bad -- flag as NIT: logger not re-tagged for sub-function
function processItem({ item, l, }: { item: Item; l: Logger; },): void {
  l.info('processing',);
  transformItem({ item, l, },);
}

// Good -- deep tagging
function processItem(
  { item, l: parentLogger, }: { item: Item; l: Logger; },
): void {
  const l = tagged({ tag: processItem.name, l: parentLogger, },);
  l.info('processing',);
  transformItem({ item, l, },);
}
```

### Script conventions

- No bash/shell scripts -- TypeScript only,
   executed with Node
- Top-level code,
   no `main()` wrapper;
   top-level await for async
- No `process.exit()` -- throw errors instead

#### No `main()` wrapper

```ts
// Bad -- flag as WARNING
async function main(): Promise<void> {
  const data = await loadData();
  console.log(data,);
}

main();

// Good
const data = await loadData();
console.log(data,);
```

## Region markers

Flag missing `//region`/`//endregion` markers on substantial code blocks:

```ts
//region User Authentication Logic -- Handles user login, registration, and session management

function loginUser(credentials: UserCredentials,): UserSession {
  return {} as UserSession;
}

function registerUser(details: UserDetails,): UserProfile {
  return {} as UserProfile;
}

//endregion User Authentication Logic
```

## Output format

```text
Summary: <one-line description of the change and overall assessment>

Findings:

BLOCKER:
- <file:line> <description and suggested action>

WARNING:
- <file:line> <description and suggested action>

NIT:
- <file:line> <description and suggested action>

NON-ACTIONABLE:
- <file:line> <description and best-effort ideas>
```

Omit empty severity sections.
If no issues found,
 write "No issues found" under Findings.

Every item in BLOCKER,
 WARNING,
 and NIT must include a concrete suggested action.
If you spot a real problem but cannot determine a specific fix,
 place it under NON-ACTIONABLE with best-effort ideas -- still report it,
 just be honest about uncertainty.
