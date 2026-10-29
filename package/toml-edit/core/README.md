# @monochromatic-dev/toml-edit-core

Comment-preserving TOML read,
 edit,
 and write utility.

Wraps [`toml-eslint-parser`](https://github.com/ota-meshi/toml-eslint-parser) and adds the serializer it does not ship.

## Install

```sh
pnpm add @monochromatic-dev/toml-edit-core
```

## Mutation testing

Run the repository's container-isolated mutation tester from the repository root:

```sh
mise run //package/toml-edit/core:test:mutation
```

The task always passes `--full-suite` because tests are organized across API and cross-path files,
 not one test file per source file.
It runs the package's `*.unit.test.ts` tests against each mutant;
 the fuzz sidecar and conformance runner are separate campaigns.
The CLI writes `mutation-report.json` at the repository root by default and reports survivors without failing the task.
Use `--report` to choose another report path.
The default source scan includes the `src/conformance/` adapters,
 but the package unit suite does not run the upstream conformance runner;
 inspect those survivors separately rather than treating them as runtime-library coverage.

Preview mutant and test selection without starting containers,
 or limit the run to a package-relative source file:

```sh
mise run //package/toml-edit/core:test:mutation -- --dry-run src/toml-has.ts
mise run //package/toml-edit/core:test:mutation -- src/toml-has.ts
```

## API shape

Free-function API over an immutable `TomlEditState`.
Mutating functions return a fresh state.

## Examples

Round-trip a file (splice mode,
 byte-identical for LF input when no edits applied):

```ts
import {
  parseTomlEdit,
  tomlStringify,
} from '@monochromatic-dev/toml-edit-core';

const source = await Bun.file('mise.toml',).text();
const edit = parseTomlEdit({ source, },);
const text = tomlStringify({ edit, },);
// text === source (for LF input; CRLF input round-trips as LF, see Newlines)
```

## Newlines

`parseTomlEdit` normalizes `CRLF` line endings to `LF` before parsing and warns
that it did (suppressible with `MONOCHROMATIC_WARN=false`),
 so editing,
 splicing,
 and emission
only ever deal with single-byte `LF` newlines.
 A `CRLF` document therefore
round-trips as `LF`,
 not byte-for-byte.
 A bare carriage return (a lone `CR` not
part of `CRLF`) is invalid TOML and is rejected with `TomlEditError`.

Edit a key while preserving comments and original formatting:

```ts
import {
  parseTomlEdit,
  tomlSet,
  tomlStringify,
} from '@monochromatic-dev/toml-edit-core';

const e0 = parseTomlEdit({ source, },);
const e1 = tomlSet({ edit: e0, path: ['tools', 'bun',], value: 'latest', },);
await Bun.write('mise.toml', tomlStringify({ edit: e1, },),);
```

Build a TOML file from scratch (canonical mode):

```ts
import {
  emptyTomlEdit,
  tomlSet,
  tomlStringify,
} from '@monochromatic-dev/toml-edit-core';

const e0 = emptyTomlEdit();
const e1 = tomlSet({ edit: e0, path: ['title',], value: 'Demo', },);
const e2 = tomlSet({ edit: e1, path: ['tools', 'bun',], value: 'latest', },);
const out = tomlStringify({ edit: e2, },);
```

## Modes

`parseTomlEdit` accepts `mode: 'splice' | 'canonical'`.
Default is `splice`,
 which emits unmutated regions verbatim from the source.
`canonical` rebuilds text from the AST with consistent formatting on every `tomlStringify`.
`emptyTomlEdit` is always canonical (no source to splice).

## Mutation surface

`tomlSet` handles five resolution outcomes:

- **Existing key-value or array element**:
   replaces the value bytes canonically.
   Style and raw spelling are preserved for unchanged primitives.
- **Existing `[foo]` table or top-level**:
   replaces the body's key-values with the entries of the supplied JS object.
   Sub-tables (`[foo.sub]`) and top-level table headers are preserved.
   The JS value must be a plain object;
   arrays,
   scalars,
   and `Date` throw `TomlTypeError`.
   Pass `{}` to clear the body.
- **Missing path with the deepest parent present**:
   inserts a fresh entry.
   Multi-segment tails become dotted-key inserts (`a.b.c = 42` at top-level,
   `b.c = 42` inside `[a]`).
   Inline tables are extended in place (`foo = {}` -> `foo = { x = 1 }`).
   Path-create through a scalar or `TOMLArray` throws.
- **Existing array-of-tables collection (`[[foo]]`)**:
   requires an array value `[{...}, {...}, ...]`;
   one `[[foo]]` block is emitted per element.
   Pass `[]` to clear every instance.
   Multiple sibling standard tables under an implicit parent (`[a.b]` / `[a.c]` queried by `['a']`) are still rejected;
   set per sub-table.
- **Dotted-key collision** (sibling-table or inline-table key overlap that would not re-parse):
   rejected with `TomlImmutableNodeError`.

`tomlDelete` handles four resolution outcomes:

- **A key-value**:
   removes the entire line plus its trailing inline comment.
- **A `[foo]` table header**:
   removes the full block (header line + all body key-values + interleaved comments + trailing newline).
- **An array-of-tables collection**:
   removes every matched table block.
   Path `['foo']` against multiple `[[foo]]` instances clears them all;
   `['a']` against `[a.b]` + `[a.c]` removes both sub-tables.
- **An array element** at any depth (inside the direct value of a key-value,
   or nested inside one or more arrays under that value):
   walks the parent chain up to the enclosing key-value and rewrites the outermost array via canonical re-emission,
   omitting the targeted element at the deepest level.

Sub-path reads (`tomlGetValue` / `tomlHas`) project pending edits through the path:
 after `tomlDelete({ path: ['arr', 1] })` on `arr = [10, 20, 30]`,
 `tomlGetValue({ path: ['arr', 1] })` returns `30` (the new array's index 1).

### v1 limitation: canonical mode on a parsed source

In v1,
 `parseTomlEdit({ source, mode: 'canonical' })` falls back to splice emission when the source is non-empty.
Per-node canonical re-emission is implemented (used when individual nodes are mutated),
 but a full AST walk that re-formats every byte of the parse output is deferred to v2.
The intent:
 callers who only need canonical-from-scratch output go through `emptyTomlEdit()`;
 callers with a parsed source typically want to preserve the original layout for unchanged regions,
 which is what splice mode already provides.
If full canonical re-formatting of parsed source is required (e.g. enforce indent,
 normalise array layouts on every key),
 reparse the splice output through `emptyTomlEdit()` + setters,
 or open an issue.

### `tomlGetNode` and `tomlGetRaw` are clean-node views

Both return the parse-time AST node or source bytes of a node that is still
clean (unmutated) in the current document tree.
 After `tomlSet({ edit, path, value: 'new' })` on an existing path:

- `tomlGetValue({ edit, path })` returns `'new'` (it reads the current tree).
- `tomlGetNode({ edit, path })` throws `TomlPathNotFoundError`:
   the edited value is now synthetic and has no parse-time AST node.
   An unedited sibling path still returns its clean node.
- `tomlGetRaw({ edit, path })` likewise throws for the edited path (no clean
   source bytes back it),
   and returns the original slice for unedited paths.

For paths created by `tomlSet` that did not exist at parse time,
 neither function can return a node or bytes;
 `tomlStringify` and reparse first.

## Key quoting normalization

`normalizeTableHeaderKeyQuoting({ source, })` rewrites every literal-quoted
(single-quoted) key segment in a table header as a basic-quoted (double-quoted)
string and leaves every other byte as it was.
Bare keys and keys already written as basic strings are kept as written,
so the result is idempotent.
Key-value keys and literal string values keep their quote style;
only header key segments are spliced,
which makes the rewrite a semantic no-op that removes quote-style drift.
It reuses the parse path behind `parseTomlEdit` and throws `TomlEditError` on
invalid input.

## Unstable seam exports

The package re-exports a few internal encoders and emitters with an underscore prefix:
`_encodeKey`,
 `_formatPath`,
 `_isStrictPrefix`,
 `_isAttachedGap`,
 `_jsValueToTomlText`,
 `_emitContentNode`,
 `_emitStringValue`,
 and `_emitDocument`.

These carry no compatibility promise.
They exist for built-artifact test coverage,
 observability,
 and the property-based fuzz suite,
 which exercises them through the built artifact.
Their signatures may change without a major version bump,
 so application code must not depend on them.
See `doc/decision/toml-edit-fuzzing.md`.
