# JSONC block comments close at the first `*/`, even one inside an inner `//` comment or a quoted string

## Symptom

The comment-preserving JSONC parser in `package/jsonc-edit/core-ts` terminates a
block comment at the **first** `*/` it encounters,
 regardless of whether that
`*/` sits inside a `//` line comment or a quoted string written within the
enclosing `/* */` block:

```jsonc
/*
  some config
  // this line has */ in it
  more config
*/
{ "key": "value" }
```

The parser closes the block comment at the `*/` on the third line.
Everything from `in it` onward is then parsed as document content,
 and the
document fails to parse as JSONC.

## Root cause

Block comments do not nest in any C-family language.
The closing `*/` is always the first `*/` after the opening `/*`,
 with no
exception for quotes,
 line comments,
 or any other syntactic context inside the
comment body.
A comment is the shape `/* … */` where `…` is "any characters up to,
 but not
including,
 the first `*/`".

`package/jsonc-edit/core-ts/src/scan.ts` implements exactly that,
 with no
heuristic to second-guess it:

```ts
const close = source.indexOf('*/', index + 2,);
```

`scanBlockComment` returns the body up to that offset.
There is no attempt to skip a `*/` that happens to fall inside a `//` line
comment within the block,
 because the comment body has no inner structure to
inspect:
 a correct exception would need a second-order state machine
("inside a line comment,
 inside a block comment") that the grammar does not have.

## Verification

```ts
import { parseJsonc, } from '@monochromatic-dev/jsonc-edit-core-ts';

// Throws: the block closes at the inner `*/`, leaving `in it ... ` as content.
parseJsonc({
  source: '/*\n  // has */ in it\n*/\n{ "key": "value" }' as StringJsonc,
},);

// Parses cleanly: line comments terminate at end of line, so an embedded `*/`
// is harmless.
parseJsonc({
  source: '// has */ in it\n{ "key": "value" }' as StringJsonc,
},);
```

## Verified workaround

When a comment must contain `*/`,
 write it as `//` line comments instead of a
`/* */` block:

```jsonc
// some config
// this line has */ in it
// more config
{ "key": "value" }
```

Line comments terminate at the newline,
 so an embedded `*/` is just text.
JSONC configuration in this workspace favors `//` already,
 so the cost is small.

## What does not work

- Escaping `*/` inside a block comment (`*\/`,
   `* /`):
  the parser matches the literal `*/` token;
   backslash and whitespace inside a
  comment have no special meaning.
- Nesting block comments (`/* outer /* inner */ */`):
  C-family languages do not nest block comments;
   the first `*/` closes the outer
  block and the trailing `*/` becomes a syntax error.
- A richer scan that skips `*/` inside `//`:
  the required disambiguation is non-regular,
   and a full second-order lexer is
  not worth the cost for a pattern that is vanishingly rare in real JSONC.

## Why we do not file this upstream

The parser is in-tree (`package/jsonc-edit/core-ts`);
 there is no external
upstream.
The behavior matches every C-family language and is intentional:
 the first `*/`
wins,
 by design.
The limitation is accepted and documented here rather than worked around.

## Related

- `package/jsonc-edit/core-ts/src/scan.ts`:
  `scanBlockComment` (the `indexOf('*/')` close) and `scanLineComment`
  (line comments terminate at end of line,
   so they have no nesting hazard).
- `doc/troubleshooting/toml.md`:
  why JSONC was chosen over TOML for workspace configuration.
- `doc/decision/jsonc-edit-parser-foundation.md`:
  why the JSONC parser is hand-written rather than wrapping a library.
