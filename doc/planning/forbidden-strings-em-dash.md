# Em-dash enforcement via forbidden-strings

Date:
 2026-05-10

Investigation into using `forbidden-strings`
(the resharp-based deny-list scanner at `package/forbidden-strings/cli/`)
to enforce the `AGENTS.md` ban on em-dashes (`—`),
 en-dashes (`–`),
and their ASCII substitutes (`-`,
 `--`) when used as em-dashes in prose.

## Status

Feasibility confirmed for both unicode characters and the ASCII `--` shape.
Open work:
 exclusion-list expansion using the literal-space workaround
for `\b` inside complement bodies (see revised "Resharp HIR limits" below),
self-match handling for the unicode case,
 single-dash (`-`) handling.

## What works

### Unicode literals

Bare literal lines for `—` and `–` in the rule file work directly:

```text
—
–
```

The 7-byte boundary check does not fire on multibyte non-word characters,
so the literal substring-matches anywhere.
Verified by reading `package/forbidden-strings/cli/src/rule/atom.rs:21`
and the README's "non-ASCII characters ... gate correctly" note.

Empirical test against `/tmp/em-dash-fixture.md` (constructed during investigation):
two em-dashes and one en-dash matched at the expected byte offsets;
hyphens,
 `--watch`,
 `user-facing`,
 and ASCII `--` separators passed through cleanly.

### ASCII regex rules

Two regex rules verified to match em-dash shapes
while skipping high-confidence legitimate uses:

```text
/[a-z]--[a-z]/
/^.*[a-z] -- [a-z].*$&~(.*npm.*)&~(.*git.*)/
```

Rule 1 catches `word--word` (no surrounding spaces),
routed to the rust `regex` crate (no set-algebra operators present).
Consuming the boundary letters in the match span is fine
since the goal is detection,
 not precise span isolation.

Rule 2 catches `--` between alphabetic words
on lines that do not contain `npm` or `git`.
The `&` and `~()` operators route this rule through resharp
(`package/forbidden-strings/cli/src/rule/engine.rs:204` `requires_resharp`),
which supports the set-algebra needed for the line-level complements.
Since commit `67e844df`,
 the same routing predicate also detects
lookaround openers (`(?=`,
 `(?!`,
 `(?<=`,
 `(?<!`),
 so rules whose only
resharp-only feature is a lookaround compile cleanly without needing
`&_*` as a routing hint.

### Self-match safety for ASCII

The regex rule sources (`[a-z]--[a-z]` etc.)
do not contain `--` or `--` between literal letters;
the bracket characters break the pattern.
Confirmed:
 scanning these rules against themselves produces no hits.
This is an advantage over literal form for the ASCII case.

### Empirical scan results

Run on 2026-05-10 against the full tree (`forbidden-strings --all`):

- Rule 1:
   36 matches.
  Mix of true positives ("permission is not necessary for any reason--for" in LICENSE files)
  and false positives (markdown URL fragments such as `#monitoring--metrics`).
- Rule 2:
   2,117 matches.
  533 in `AUDIT.em-dash.md` (intentional examples cataloging violations),
  117 in `TODO.claude-code-words.md` (genuine em-dash style violations of shape "word -- description"),
  the rest scattered across markdown docs,
   comments,
   `mise.toml` task descriptions.

Top hit files:

- `AUDIT.em-dash.md` (533) — intentional self-violations
- `TODO.claude-code-words.md` (117) — true positives
- `.agents/skills/code-review/SKILL.md` (79)
- `package/audit/oph-common-look-and-feel/src/index.html` (33)
- `package/module/es/src/types/.../simplifiedSchema.behaviorTest.ts` (25)
- `package/webapp-productivity/done/PLAN.md` (19)
- `package/forbidden-strings/cli/src/rule.rs` (12)
- `mise.toml` (9)

## What does not work

### Resharp HIR limits inside `~(...)` complement bodies

The compile-time failure that earlier drafts of this document
attributed to "alternation count" or "seven chained complements"
is actually a feature restriction:
 resharp's HIR translator rejects
word-boundary assertions (`\b`,
 `\B`) and text/line anchors (`^`,
 `$`)
when they appear inside a complement body,
 returning
`Algebra(UnsupportedPattern)` at compile time.

Verified 2026-05-10 by sweeping rules through the release binary:

- single `~(.*(w0|w1|...|wN).*)` with simple bodies:
   500 alternatives compile cleanly
- chained `&~(.*w0.*)&~(.*w1.*)&...&~(.*wN.*)`:
   500 chains compile cleanly
- `~(.*\bnpm\b.*)` with any alternative count (including 1):
   fails
- `~(.*\B.*)`,
   `~(^foo$)`:
   fails
- `\bnpm\b&_*&~(.*foo.*)` (the `\b` is outside the complement):
   compiles cleanly
- `(?=foo\b)bar`,
   `(?<=[a-z])foo` (lookarounds with non-anchor bodies):
   compile cleanly
- `(?=^foo)bar`,
   `(?<=\b)foo`:
   fail

The two patterns that earlier failed:

```text
/(?<=[a-z]) -- (?=[a-z])&_*&~(.*\b(npm|bun|...|forbidden-strings)\b.*)/
/^.*[a-z] -- [a-z].*$&~(.*[`].*)&~(.*npm.*)&~(.*bun.*)&~(.*git.*)&~(.*mise.*)&~(.*cargo.*)&~(.*gh\b.*)/
```

both compile cleanly once `\b` is removed from the complement bodies.
The 35-alternative count and the 7-chain count were unrelated to the failures.

Workaround for token-boundary matching:
 replace `\bnpm\b` with `npm`
(literal whitespace) inside complement bodies.
Verified:
 `~(.*(\bnpm\b|\bgit\b).*)` fails;
`~(.* (npm|git) .*)` succeeds.
Tradeoff:
 tokens at start or end of line are not bordered by literal space
and would slip through.
Acceptable for prose scans where excluded toolchain names appear mid-line.

### Sub-span exclusion vs anchored matching

`A&~(B)` applies the complement to the match span only,
 not surrounding context.
Without anchors,
 the engine finds a sub-span that satisfies both conditions
even when the whole line contains an excluded token.
Verified empirically:
 an unanchored rule
matched at column 7 within line `Git: git log -- file.txt is legitimate.`
because the sub-span starting at column 7 ("it log -- file.
txt is leg")
does not contain "git".
Anchors (`^...$`) force the match to span an entire line.

### Code-block awareness

Regex cannot track `` ``` `` fence state across lines without backreferences
(resharp does not support backreferences either).
Em-dashes inside fenced code blocks are flagged
unless the line happens to contain a command name in the exclusion list.
No clean regex workaround.

## Self-match issues for unicode rules

A bare `—` rule fires inside the forbidden-strings package itself:
roughly 40 occurrences in `src/rule/atom_tests.rs` (test inputs),
`atom.rs` and `extract.rs` (doc comments explaining em-dash handling),
and `README.md` line 169.
Plus `AGENTS.md` (states the rule),
 `AUDIT.em-dash.md` (lists violations),
`TROUBLESHOOTING.*.md` files quoting external output.

The scanner's only path-level exclusion is
`is_skipped_file` at `package/forbidden-strings/cli/src/main.rs:171-193`,
a hardcoded `matches!` over five exact basenames.
No glob,
 no path prefix,
 no rule-scoped exclusion exists.

Three options for handling these self-matches:

1. Extend `is_skipped_file` with package paths or test-fixture basenames.
   Trade-off:
    those files become exempt from every rule,
   including the betterleaks-ported credential rules.
   Whether that gap is acceptable on those specific files is a policy call.
2. Add per-rule path-prefix exclusion to the rule grammar.
   Real scanner change touching `rule.rs` and the per-file scan loop.
3. Relocate test-fixture em-dashes into a `data/` file outside scan scope
   and rewrite docs to reference `U+2014` instead of literal characters.
   Costly across roughly 40 sites;
    partially defeats the purpose
   of testing the literal character.

## Improvement potentials

Notes for the next iteration session.
Not implemented.

### Exclusion list expansion

Rule 2's exclusion list currently has only `npm` and `git`.
The earlier framing blamed an "algebra ceiling" on alternation count;
the actual blocker (see "Resharp HIR limits inside `~(...)` complement bodies"
above) was `\b` inside the complement,
 not size.
Splitting exclusions across multiple rules does not help:
multiple rules combine via union (any rule firing flags the line),
which makes detection more permissive,
 not more restrictive.

Three viable paths:

1. Use the literal-space workaround for the toolchain exclusion list.
   `~(.* (npm|bun|pnpm|yarn|deno|node|mise|hk|gh|cargo|git|jq|...) .*)`
   compiles cleanly at sizes well beyond the 35-element original list
   (no measured ceiling up to 500 alternatives).
   Tradeoff:
    misses tokens at start or end of line;
    revisit on a
   sampled corpus to confirm the lost coverage is small.
2. Pre-process the corpus before scanning:
   strip fenced code blocks,
    strip markdown URL anchors,
   strip inline backtick spans.
   Requires scanner code changes
   (a pre-pass between file read and rule application).
3. Hand-pick a smaller,
    high-impact exclusion set per rule.
   Iterate against the corpus,
    classify each false positive,
   keep only the exclusions that retire the most false positives.
   Still useful even with path 1 available,
    because some false-positive
   classes (markdown anchors,
    inline backticks) are not toolchain names
   and need their own exclusion shape.

Categories of false positive observed in the empirical scan:

- markdown anchor links (lines containing `](` or `#`)
- inline code (lines with backticks)
- toolchain commands (`mise`,
   `bun`,
   `pnpm`,
   `yarn`,
   `cargo`,
   `deno`,
   `node`,
   `hk`,
   `gh`,
   `jq`)
- shell builtins (`cp`,
   `mv`,
   `rm`,
   `cat`,
   `echo`,
   `exec`,
   `find`,
   `ls`,
   `cd`)

### Single-dash case (`-` em-dash)

The audit reports 625 occurrences of `-` used as em-dash.
Higher false-positive rate due to subtraction (`a - b`),
negative numbers (`-5`),
date ranges (`2024 - 2026`).
Tractable with the same regex plus complement approach
but the exclusion list is broader and less reliable.

### Wider character classes

Current `[a-z]` misses uppercase em-dash usage such as "Word -- Another".
Widen to `[a-zA-Z]` or to a unicode word class.
Need to verify the resharp engine handles wider classes
without hitting the algebra ceiling earlier.

### Code-fence pre-stripping

A two-pass scanner that strips fenced regions before applying rules
would solve the code-block-awareness gap.
Requires scanner code changes (not pure rule-level fix).

### Inline backtick parity check

Inline backtick parity on a single line is expressible in regex
by counting pairs before the position.
Could be added as a complement:
 exclude positions
preceded on the same line by an odd number of backticks.
Pattern shape (untested):
 `~(.*`[^`\n]*` -- ` -- ...).
Worth prototyping;
 might trip the algebra ceiling.

### File-level exclusion via skip list

If the scanner gains path-glob skip support
(extension to `is_skipped_file`),
the rule grammar would not need code-block awareness
for files that are uniformly code (`.ts`,
 `.rs`,
 `.css`).
Em-dash bans in those files are noise anyway:
the meaningful target is markdown and prose.

A cheap proxy:
 skip the rule entirely on lines
that look like they are inside a code-only file
by reading the file extension at scan time.
The scanner currently does not branch on extension.

### Compose with file-enforcer for fix automation

Once the rule reliably flags violations,
file-enforcer could include an em-dash-fix transform
that rewrites flagged spans to the recommended punctuation
(paired commas,
 parentheses,
 colon,
 semicolon,
 period)
before re-running the lint.
Out of scope for the current investigation.

## Post-migration note: forbidden-regex dialect

Date:
 2026-07-17.
 The engine migration (issues #375 through #389) replaced resharp with the
in-house `forbidden-regex` engine (`package/forbidden-strings/core`) after this
investigation was written.
 Two of the blockers documented above have different answers under
the new dialect.

### Bounded complements no longer need the literal-space workaround

The "Resharp HIR limits inside `~(...)` complement bodies" section found that resharp's HIR
translator rejected `\b`,
 `\B`,
 `^`,
 and `$` whenever they appeared inside a `~(...)`
complement body,
 which forced the literal-space workaround (`~(.* npm .*)` instead of
`~(.*\bnpm\b.*)`) and left tokens at the start or end of a line uncovered.
 `forbidden-regex`
carries no such restriction:
 its supported-constructs list treats `^`,
 `$`,
 and `\b` as
ordinary anchors usable anywhere the dialect allows one,
 and its rejected-at-compile-time list
(`package/forbidden-strings/core/README.md`) does not single out complements.
 A rule
ported to the new dialect can use `~(.*\bnpm\b.*)` directly,
 once its quantifiers are bounded
(see the next point),
 retiring both the literal-space workaround and the line-edge coverage
gap it left.

### Unbounded quantifiers must become bounded ones

The rules recorded above (`/^.*[a-z] -- [a-z].*$&~(.*npm.*)&~(.*git.*)/` and similar) rely on
unbounded `.*`.
 `forbidden-regex` rejects `*`,
 `+`,
 and unbounded `{n,}` outright at compile
time;
 only bounded repetition (`a?`,
 `a{3}`,
 `a{3,6}`) is supported,
 and a pattern that can
match the empty string is also rejected,
 so `~(Y)` alone still needs a concrete positive
operand alongside it.
 Porting any exclusion rule from this document to the new dialect means
rewriting each `.*` as a bounded form such as `.{0,200}`,
 sized to the longest line worth
scanning,
 not a mechanical find-and-replace of the operator syntax alone.

Together,
 these two points mean the "Exclusion list expansion" work above is worth
revisiting under the new dialect:
 the anchor restriction that motivated the literal-space
workaround is gone,
 but the unbounded-quantifier restriction is new (resharp allowed
unbounded `.*`),
 so any concrete rule text drafted from this document needs bounded
repetition rather than the resharp-era shapes quoted verbatim.

## Reference

- Resharp engine:
   `https://github.com/ieviev/resharp`
  (cloned to `/tmp/resharp` during investigation)
- Resharp syntax docs:
   `/tmp/resharp/docs/syntax.md`
- Forbidden-strings source:
   `package/forbidden-strings/cli/`
- Existing audit data:
   `AUDIT.em-dash.md` (untracked at investigation time)
- `AGENTS.md` em-dash rule:
   under "Documentation standards"
- Test fixtures used for prototyping during investigation:
  `/tmp/em-dash-fixture.md`,
  `/tmp/em-dash-fixture-real.md`,
  `/tmp/em-dash-fixture-no-space.md`
- Rule iterations during investigation:
   `/tmp/em-dash-rules-v{1..8}.txt`,
  `/tmp/em-dash-rules-real.txt`
