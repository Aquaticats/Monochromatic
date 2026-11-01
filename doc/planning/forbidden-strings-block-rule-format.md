# Forbidden-strings block-form rule file format

Status:
SUPERSEDED 2026-07-20.
The vet (`doc/audit/tech-forbidden-strings-rule-file-format-vet-2026-07-20.md`)
ranked this design second;
the maintainer adopted the tail-format sectioned file instead
(`doc/decision/forbidden-strings-rule-file-format.md`,
spec `doc/planning/forbidden-strings-tail-rule-format.md`).
The verified engine facts and the sequencing constraint below remain
accurate and are cited by the successor documents.

Context:
the forbidden-regex engine is deliberately always-verbose.
Its cursor documents the intent directly
(`package/forbidden-strings/core/src/parse/cursor.rs`,
`skip_ignorable`):
"verbose mode is always on,
 so a rule may span many lines and carry
first-column comment lines."
The rule file format was the only thing preventing exactly that:
`parse_patterns` (`package/forbidden-strings/cli/src/rule/frx/format.rs`)
splits on newlines before the engine sees anything,
so a rule could never span lines
and per-alternative comments inside a large alternation were impossible.

## Verified engine facts the design rests on

- A newline inside a pattern is ordinary skippable whitespace
  (`cursor.rs`,
  `skip_ignorable`).
- A comment is a first-column `#` running to end of line,
   nothing else.
  A mid-line unescaped `#` is a literal byte via the `parse_atom` fall-through
  (`package/forbidden-strings/core/src/parse/grammar.rs`).
- An unescaped `/` is a literal byte (same fall-through);
  `\/` is the escaped spelling
  (`package/forbidden-strings/core/src/parse/escape.rs`).
- Flags policy is already engine-external:
  the file format accepts `m` and `x` as no-ops and fails closed on anything else.

## The format

Outside a block,
 classification is unchanged with one addition:

- Blank line,
   or line whose first non-whitespace byte is `#`:
   skipped.
- Line starting (after trim) with `/` that closes on the same line
  (a later `/` whose trailing run is all ASCII-lowercase):
  a single-line regex rule,
   exactly as today.
- Line starting (after trim) with `/` that does not close on the same line:
  opens a block rule.
  Text after the opening `/` on that line is pattern content.
- Any other non-blank line:
   a bare literal,
   exactly as today.

Inside a block:

- A line whose trimmed content is exactly `/` followed by an optional
  all-ASCII-lowercase flag run closes the block.
  The flag run obeys the existing policy
  (`m`/`x` dropped as no-ops,
   anything else a fail-closed load error).
- A line whose first non-whitespace byte is `#` is a comment and is dropped
  by the file format before the engine sees the pattern.
  This is deliberately laxer than the engine's first-column rule
  so comments may be indented;
  the format strips a superset of what the engine would strip,
  so the two layers never disagree about surviving content.
- Every other line (including blank lines) is pattern content.
  Content lines join with `\n` into one pattern body handed to the engine
  strict,
   where verbose mode treats the newlines as whitespace.
- End of file with an open block is a new fail-closed load error
  (`UnterminatedRegex`,
   redacted and index-bearing like the existing errors).

Consequences for authors:

- A content line that must begin with a literal `#` or consist of a lone `/`
  uses the engine escapes `\#` and `\/`.
- A bare literal starting with `/` that lacks a valid same-line close is no
  longer expressible as a bare literal;
   write it as a regex rule with `\/`.
  Audited 2026-07-20:
   zero such lines exist in `forbidden-strings.append.txt`,
  `forbidden-strings.append.local.txt`,
   or
  `package/forbidden-strings/cli/data/builtin-rules.txt`
  (263 slash-opening lines,
   every one closes on its own line),
  so nothing breaks at cutover.

## Rejected alternatives

- Trailing-position closer detection (a line merely ending with `/flags`):
  unsound,
   because a comment or content line may contain `/` mid-line
  (verified literal-byte semantics),
   silently closing the block early.
  The whole-line closer has no such collision.
- Trailing-backslash continuation markers:
  collide with the escaper's backslash-before-whitespace convention,
  where `\` followed by a newline is a literal newline byte.
- Indentation-based continuation:
  changes the meaning of indented lines,
   which today are trimmed literals.
- Engine-exact interior comments (first-column only,
   format strips nothing):
  an indented `# note` inside a block would silently become pattern content
  (whitespace skipped,
   then `#` parses as a literal byte);
   rejected as a trap.

## Sequencing constraint (critical)

An old binary reading a block-form file classifies the opener and closer as
bare literals.
A lone `/` literal matches every line containing a slash,
so the commit gate would flag nearly every file.
Order is therefore:

1.  Land the parser change with tests;
     release the scanner.
2.  Move the local gate and CI to the released binary.
3.  Only then rewrite any live rule file in block form.

Until step 3,
 block form must not appear in any live rule file.

## Payoff and relationships

- `forbidden-strings.append.txt` rewrites its two shortcode alternation rules
  in block form with each alternative's rationale comment directly attached,
  removing the comment-block-versus-rule drift risk without splitting back
  into per-code rules.
  Rule count and therefore rule indices stay unchanged.
- Orthogonal to the open rule-identity decision
  (`doc/planning/forbidden-strings-rule-identity-ux.md`);
  no fork there is prejudged.
- Ships in the same release as the short-literal word-boundary fix
  (commit `296c5169c`),
   one publish for all pending scanner changes.
