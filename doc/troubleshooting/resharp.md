# Resharp: upstream bugs and forbidden-strings workarounds

Status note (forbidden-strings 0.2.0):
 the `resharp` dependency is gone.
 The engine swap replaced resharp and the `regex` crate with the in-house
`forbidden-regex` engine (`package/forbidden-strings/core`),
 so the consumer-side guards and pre-validators this document describes no longer
exist in the scanner (`src/rule/compile.rs`,
 `src/rule/engine.rs`,
 and `src/rule/nesting.rs` were deleted).
 This document stays as a durable record of the upstream bugs found and filed.

This document tracks the upstream resharp bugs that `forbidden-strings`
defends against,
 the consumer-side guards that block each,
 and the
verification path for each finding.

Current status (2026-06-08,
 resharp v0.6.11):
 one upstream restriction is
still live,
 Bug A (complement-of-lookaround),
 held behind a load-bearing
consumer-side guard.
 The 0.6.9 floor fixes are spent (see "Fixed upstream,
now spent"),
 and the 0.6.11 bump adds three more that this project filed
against the 2026-06-04 fuzz campaign:
 bug-07 (ASCII `\D`/`\S`/`\W`
negated-class nullability,
 PR #15),
 regression-01 (duplicate zero-width
`find_all` spans on negative lookahead,
 PR #14),
 and bug-04 (NO_MATCH sentinel
abort/leak in `find_all`,
 PR #16).
 The 0.6.10/0.6.11 delta does not touch the
reverse-pass `Kind::Compl` arm,
 so Bug A is presumed unchanged;
 it was last
behaviourally verified at v0.6.9 (`264e85b`,
 2026-06-04).

Update (2026-06-19,
 resharp v0.6.13,
 the version now pinned in
`package/forbidden-strings/cli/Cargo.lock`):
 every known crash and soundness
reproducer from the 2026-06-04 and 2026-06-11 campaigns was re-run against
published v0.6.13 in both debug and release.
 All crashes and checkable soundness
findings are resolved;
 one compile-cost finding (full-unicode `\w{N}` bounded
repeat,
 bug-06) is still live.
 Bug A remains a by-design restriction held by the
consumer guard.
 Full per-finding results and the robustness verdict:
`doc/audit/resharp-fuzz-2026-06-11/verification-0.6.13.md` and
`doc/audit/resharp-robustness-2026-06-19.md`.

A separate,
 broader fuzz campaign against v0.6.9 lives in
`doc/audit/resharp-fuzz-2026-06-04/` (twenty-three distinct root causes plus
one regression);
 bug-04,
 bug-07,
 and regression-01 from that campaign are
fixed in v0.6.11 (PRs #16,
 #15,
 #14).
 This file stays scoped to the
forbidden-strings consumer guards.

## Fixed upstream, now spent

- Bugs B,
   C,
   D,
   E,
   F:
   fixed in 0.6.4 (the maintainer shipped the prototype
  patches the same day issue
  [ieviev/resharp#5](https://github.com/ieviev/resharp/issues/5) was filed),
  re-verified fail-closed through v0.6.9.
   Their consumer-side pre-validators
  (`intersection_with_lookbehind`,
   `intersection_with_word_end_alternation`,
  `lookaround_in_alternation_with_sibling`,
  `complement_intersection_quantified_group`,
  `nested_lookahead_in_quantified_group`) in
  `package/forbidden-strings/cli/src/rule/engine.rs` stay as
  belt-and-suspenders;
   over-rejection is fail-closed-safe.
- Bug G (deep-nesting stack-overflow abort),
   the intersection-over-alternation
  algebra recursion,
   the hardened `find_all` zero-width drop,
   and the flaky
  `rev_bot_skip_terminates_fast` timing test:
   all four were bundled on branch
  `fix/zerowidth-findall-and-stack-overflow-guards`,
   filed as
  [ieviev/resharp#13](https://github.com/ieviev/resharp/pull/13),
   and merged
  upstream (squashed as `af6f2a5`,
   in main as of v0.6.9).
   Verified present in
  `264e85b`:
   `resharp-parser` has `DEFAULT_MAX_DEPTH = 1_000`,
  `resharp-algebra` has the `rw_active` re-entrancy guard on the distribution
  rewrites,
   `resharp-engine/src/fas.rs` emits the boundary zero-width matches
  (`max.extend(0..=data_end)`),
   and the timing test now asserts `\z` scaling
  rather than an absolute budget.
   The consumer-side `nesting_depth`
  pre-validator (cap 1,000) in
  `package/forbidden-strings/cli/src/rule/nesting.rs` stays
  belt-and-suspenders even though `max_depth` is now upstream.
- The 0.6.8 "compile-time timeout on small patterns" finding was a fork
  fuzz-harness measurement artifact (six compiles per libFuzzer unit under
  AddressSanitizer),
   not a resharp defect;
   resolved,
   not filed.
- Fixed in v0.6.11,
   filed by this project from the 2026-06-04 campaign and
  merged to ieviev/resharp main:
    - bug-07 ([ieviev/resharp#15](https://github.com/ieviev/resharp/pull/15)):
      the parser negates the byte class for ASCII `\D`/`\S`/`\W`,
       so a negated
      Perl class is no longer treated as nullable.
    - regression-01
      ([ieviev/resharp#14](https://github.com/ieviev/resharp/pull/14)):
      `find_all` registers each null position once,
       removing duplicate
      zero-width match spans on negative-lookahead shapes.
    - bug-04 ([ieviev/resharp#16](https://github.com/ieviev/resharp/pull/16)):
      `find_all` no longer aborts or leaks state on a NO_MATCH sentinel
      candidate.
  None of the three had a consumer-side pre-validator to retire (they were
  engine/parser match-correctness defects,
       not rule-shape rejects),
       so the
  only consumer change is the dependency floor bump to 0.6.11.

## Bug A: `\b`, `\B`, `^`, `$` inside complement bodies fail with `Algebra(UnsupportedPattern)`

### Symptom

A rule passed to [`resharp`][resharp] 0.5.
x through 0.6.
x (via the
consumer crate `forbidden-strings` 0.1.0 in this workspace,
 but the bug
is upstream) fails at compile time when its complement body contains a
word-boundary or text-anchor assertion.
 The compile-time error surfaces
with one of two variants depending on which rewrite path the offending
atom takes:

```text
forbidden-strings: rule on line N (resharp): Algebra(UnsupportedPattern)
forbidden-strings: rule on line N (resharp): Parse(ParseError { kind: UnsupportedResharpRegex, ... })
```

Resharp renders `Algebra(UnsupportedPattern)` as "unsupported lookaround
pattern" (`resharp-algebra/src/lib.rs:35`);
 `UnsupportedResharpRegex` is
emitted by the parser when an unrewritable assertion survives the boundary-
rewriting helper.
 The "Verification" section below lists which surface
patterns hit which variant.

User-facing patterns that trigger the failure:

```text
/em-dash&~(.*\bnpm\b.*)/
/em-dash&~(.*\B.*)/
/em-dash&~(^foo$)/
/(?<=[a-z]) -- (?=[a-z])&_*&~(.*\b(npm|git)\b.*)/
/^.*[a-z] -- [a-z].*$&~(.*[`].*)&~(.*npm.*)&~(.*\bgh\b.*)/
```

An earlier reading suggested an "algebra ceiling" tied to alternation
count or chained-complement count.
That reading is wrong:
500 alternatives inside a single complement and 500 chained complements
both compile cleanly,
provided the bodies use literal characters or character classes.
The trigger is the presence of `\b`,
 `\B`,
 `^`,
 or `$` inside `~(...)`,
regardless of size.

## Root cause

Three rewriting steps in resharp turn user-visible "anchors" into
internal lookarounds,
then the symbolic-derivative reverse pass refuses to reverse a complement
whose body contains a lookaround.

### Step 1: `^` and `$` become lookarounds in default multiline mode

`resharp-parser/src/lib.rs:64` defaults `multiline` to `true`:

```rust
multiline: true,
```

In multiline mode,
 `^` and `$` map to `StartLine` / `EndLine`,
and the AST-to-NodeId translator rewrites them to lookarounds
(`resharp-parser/src/lib.rs:1425-1441`):

```rust
ast::AssertionKind::StartLine => {
    if !self.multiline.get() {
        return Ok(NodeId::BEGIN);
    }
    let left = NodeId::BEGIN;
    let right = tb.mk_u8(b'\n');
    let union = tb.mk_union(left, right);
    Ok(tb.mk_lookbehind(union, NodeId::MISSING))
}
ast::AssertionKind::EndLine => {
    if !self.multiline.get() {
        return Ok(NodeId::END);
    }
    let left = NodeId::END;
    let right = tb.mk_u8(b'\n');
    let union = tb.mk_union(left, right);
    Ok(tb.mk_lookahead(union, NodeId::MISSING, 0))
}
```

So `^foo$` parsed under default flags becomes
`Lookbehind(begin|nl) · foo · Lookahead(end|nl)` in the NodeId tree.

`\A` and `\z` map to `StartText` / `EndText`
(`resharp-parser/src/lib.rs:1417-1418`),
which return `NodeId::BEGIN` / `NodeId::END` directly,
bypassing the lookaround rewrite:

```rust
ast::AssertionKind::StartText => Ok(NodeId::BEGIN),
ast::AssertionKind::EndText => Ok(NodeId::END),
```

### Step 2: `\b` and `\B` become lookarounds via context rewriting

`resharp-parser/src/lib.rs:1305-1346` pre-processes `\b`/`\B`
in concatenation contexts.
It inspects the atoms to the left and right of the boundary,
classifies each as `Word`,
 `NonWord`,
 or `Unknown`,
then rewrites the boundary as a lookaround whose body asserts the opposite class
(`resharp-parser/src/lib.rs:1329-1345`):

```rust
match (left, right) {
    (NonWord, Word) | (Word, NonWord) => Ok((NodeId::EPS, idx + 1)),
    (Word, _) => {
        let neg = tb.mk_neg_lookahead(word_id, 0);
        Ok((neg, idx + 1))
    }
    (NonWord, _) => {
        let tail = tb.mk_concat(word_id, NodeId::TS);
        self.merge_boundary_with_following_lookaheads(asts, idx, tail, translator, tb)
    }
    (_, Word) => Ok((tb.mk_neg_lookbehind(word_id), idx + 1)),
    (_, NonWord) => Ok((tb.mk_lookbehind(word_id, NodeId::MISSING), idx + 1)),
    _ => Err(self.error(self.span(), ast::ErrorKind::UnsupportedResharpRegex)),
}
```

A bare `\b` that survives this helper
(for example,
 `\b` between two unknowns)
falls through to the generic assertion handler at
`resharp-parser/src/lib.rs:1419-1424`,
which surfaces a different error at parse time:

```rust
ast::AssertionKind::WordBoundary => {
    Err(self.error(self.span(), ast::ErrorKind::UnsupportedResharpRegex))
}
ast::AssertionKind::NotWordBoundary => {
    Err(self.error(self.span(), ast::ErrorKind::UnsupportedResharpRegex))
}
```

In the em-dash rules,
the `\b` always has a known word neighbour
(`\bnpm\b` has Word on both sides via the literal letters),
so it passes parsing and reaches the algebra layer as a lookaround pair.

### Step 3: `reverse` refuses complement-of-lookaround

DFA construction walks the NodeId tree in both directions.
The reverse pass at `resharp-algebra/src/lib.rs:2203-2281` rewrites each node.
The complement case at lines 2233-2239 is the failure site:

```rust
Kind::Compl => {
    if self.contains_look(node_id.left(self)) {
        return Err(ResharpError::UnsupportedPattern);
    }
    let body = self.reverse(node_id.left(self))?;
    self.mk_compl(body)
}
```

`contains_look` (`resharp-algebra/src/lib.rs:978-981`) is a cheap meta-flag check:

```rust
pub fn contains_look(&mut self, node_id: NodeId) -> bool {
    self.get_meta_flags(node_id)
        .has(MetaFlags::CONTAINS_LOOKBEHIND.or(MetaFlags::CONTAINS_LOOKAHEAD))
}
```

The error variant is declared at `resharp-algebra/src/lib.rs:25`
and rendered at `:35`:

```rust
UnsupportedPattern,
// ...
ResharpError::UnsupportedPattern => write!(f, "unsupported lookaround pattern"),
```

So the call chain for `em&~(.*\bnpm\b.*)` is:
parser rewrites `\b` to lookarounds (step 2),
the complement body now `contains_look`,
DFA setup calls `reverse` on the complement,
reverse hits line 2234 and returns `Err(UnsupportedPattern)`.
The render is intentionally generic ("unsupported lookaround pattern"),
which masks the surface trigger.

### Why this restriction exists

Resharp implements Brzozowski-style symbolic derivatives over a structured
node tree,
and the matching engine derives both forward and reverse derivatives during
DFA construction
(the reverse pass is invoked from `ts_rev_start` at
`resharp-algebra/src/lib.rs:2196-2201`).
Reversing a complemented language with zero-width positional constraints
is not algebraically straightforward:
De Morgan-style pushdown of the complement past a lookaround does not
preserve the lookaround's position-sensitive semantics,
because the reverse operation flips which direction "before" and "after" point.
Rather than implement a per-case reverse for each lookaround kind nested
under complement,
the codebase bails.
The same node tree without the enclosing complement reverses cleanly:
the `Kind::Lookahead` arm at lines 2251-2273 returns `Ok` for `rel == 0`
lookaheads without anomalous tails.

## Verification

Verified 2026-05-10 against `resharp 0.5.2` (crates.
io checksum
`80f2ed5c008a621ce1ab18946bdca99584ed8a6c943f64dd73f7570a23ca1eb8`,
published 2026-05-09) via a synthetic Rust crate calling
`resharp::Regex::new` directly on each pattern,
 and against `resharp 0.5.1`
via the forbidden-strings 0.1.0 release binary
(`package/forbidden-strings/cli/target/release/forbidden-strings`).
The `0.5.1`-to-`0.5.2` upstream delta is streaming/seeking,
 aarch64+wasm
build targets,
 and a prefix-engine bugfix;
 none touch the `Kind::Compl`
arm of `reverse`,
 which lives at `resharp-algebra/src/lib.rs:2234-2235`
in 0.5.2 (previously quoted as `:2233-2239` against an earlier checkout;
slight line drift only).

Re-verified 2026-05-16 against `resharp 0.6.0` (published 2026-05-15)
via the same probe path.
 The `Kind::Compl` arm of `reverse` and the
parser rewrites at `resharp-parser/src/lib.rs:1305-1346`,
`:1419-1424`,
 and `:1425-1441` are all unchanged in 0.6.0.

Re-verified 2026-05-23 against published `resharp 0.6.3` and against git
HEAD `e0b8aba`.
 The complement-of-lookaround reject still fires for every
complement-body shape below;
 the behaviour is unchanged,
 only the source
lines drifted.
 At HEAD the reverse-pass `Kind::Compl` arm that returns
`UnsupportedPattern` is at `resharp-algebra/src/lib.rs:2242-2243`
(was `:2234-2235` in 0.6.0),
 `contains_look` is at `:979`,
 and the
`UnsupportedPattern` variant and its render are at `:25` and `:35`.
 The
parser rewrites drifted substantially:
 the boundary rewrite is around
`resharp-parser/src/lib.rs:1456-1491`,
 the generic word-boundary reject
at `:1562-1569`,
 and the multiline `^`/`$` rewrite at `:1577-1586`
(the `913c9fe accept more patterns`,
 `d1d560e javascript word boundary`,
and `ec54529 auto-rewrite more unsupported patterns` commits added new
`WordBoundary*` assertion kinds at `:1595-1610`).

Two patterns the earlier write-up listed wrongly were corrected during
this pass after testing them against both 0.6.0 and 0.6.3 (identical in
both,
 so neither is a 0.6.
x change):

- `/(?=^foo)bar/` compiles cleanly (it was listed under parser-layer
  rejects).
   The `^` inside a lookahead body does not break compilation.
- `/em.*\bword\b/` does NOT compile;
   it fails with
  `Algebra(UnsupportedPattern)` because the `\b` rewrite produces a
  negative lookbehind / lookahead that the reverse pass then refuses.
  The "move the boundary outside the complement" workaround below is
  therefore not reliable and is annotated accordingly.

Test harness (binary route):

```bash
cd /tmp
touch probe-input.txt
FS=/var/home/user/Monochromatic/package/forbidden-strings/cli/target/release/forbidden-strings
echo '<rule>' > probe-rule.txt
$FS --rules probe-rule.txt probe-input.txt
echo "EXIT=$?"   # 0: compile + scan OK; 2: rule error
```

Test harness (synthetic crate,
 exact error variant):

```toml
# /tmp/resharp052-repro/Cargo.toml
[package]
name = "resharp052_repro"
version = "0.0.0"
edition = "2021"
[dependencies]
resharp = "=0.5.2"
```

```rust
// /tmp/resharp052-repro/src/main.rs
use resharp::Regex;
fn main() {
    for src in [r"em&~(.*\bword\b.*)", r"em&~(.*\B.*)", r"em&~((?=foo).*)"] {
        match Regex::new(src) {
            Ok(_)  => println!("OK    {}", src),
            Err(e) => println!("ERR   {}\t{:?}", src, e),
        }
    }
}
```

### Rules that compile cleanly

- `/em&~(.*foo.*)/` (simple literal in complement body)
- `/em&~((?i)foo)/`,
   `/em&~([a-z]+)/`,
   `/em&~(.*[^a-z].*)/` (other features in complement body)
- `/em&~(\Afoo\z)/` (`\A`/`\z` text anchors inside complement;
   no lookaround rewrite).
  Compiles at v0.6.9.
   It rejected at 0.6.8 with `Algebra(UnsupportedPattern)`,
   but
  the v0.6.9 cleanup restored it,
   so the `\A`/`\z` workaround below works again.
- `/em\b/`,
   `/\bem\b&_*/`,
   `/\bem\b&_*&~(.*foo.*)/` (`\b` outside complement body)
- `/(?=\bem\b).*/` (`\b` inside a lookaround body,
   not inside a complement)
- 500 alternatives in a single `~(.*(w0|w1|...|w499).*)` with simple bodies
- 500 chained `&~(.*w0.*)&~(.*w1.*)&...&~(.*w499.*)`

### Rules that fail with `Algebra(UnsupportedPattern)` (algebra-layer reject)

Patterns whose offending atom is rewritten to a lookaround by the parser
but then refused by `reverse` at the `Kind::Compl` arm:

- `/em&~(.*\bnpm\b.*)/`,
   `/em&~(.*\bnpm.*)/`,
   `/em&~(.*npm\b.*)/`
  (`\b` in complement body,
   with a known word-class neighbour so the
  boundary rewrite succeeds and produces a lookaround pair)
- `/em&~(^foo$)/`,
   `/em&~(\Afoo$)/`,
   `/em&~(^foo\z)/`
  (default-multiline `^`/`$` rewritten to `Lookbehind`/`Lookahead`)
- `/em&~((?=foo).*)/`
  (user-explicit lookahead inside complement,
   no `\b`/`^`/`$` involved;
  proves the restriction is "lookaround in complement" generally,
   not
  word-boundary syntax specifically)

### Rules that fail with `Parse(UnsupportedResharpRegex)` (parser-layer reject)

Patterns where the parser's boundary-rewriter helper cannot classify the
atom's neighbours or the assertion sits in a lookaround body where the
rewrite chain is wired against the surrounding flag state:

- `/em&~(.*\B.*)/`
  (`\B` between two `.*` atoms;
   both neighbours classify as Unknown so
  the boundary-rewrite helper around `resharp-parser/src/lib.rs:1456-1491`
  falls through to the generic assertion handler at `:1562-1569`,
   which
  rejects bare `\B` outright)
- `/(?<=\b)foo/`
  (`\b` in a lookbehind body with no neighbouring word-class atom)

`/(?=^foo)bar/` was previously listed here but compiles cleanly in 0.6.0,
0.6.3,
 and HEAD;
 it is not a parser-layer reject.
 See the correction note
in the Verification section above.

The earlier "alternation count" / "seven chained complements" framing was a misdiagnosis:
every observed failing case contained a lookaround-introducing assertion in a complement or lookaround body,
and the count axis had no measured ceiling within practical bounds.

## Verified workarounds

### Replace `\b` with literal whitespace inside complement bodies

```text
# fails
/^.*[a-z] -- [a-z].*$&~(.*(\bnpm\b|\bgit\b).*)/

# compiles
/^.*[a-z] -- [a-z].*$&~(.* (npm|git) .*)/
```

Tradeoff:
 tokens at line start or line end are not bracketed by literal spaces
and slip through the exclusion.
For prose scans where the excluded tokens are toolchain names appearing
mid-line,
 this is acceptable.

### Replace `\b` with `\W` character class inside complement bodies

```text
# fails
/em&~(.*\bnpm\b.*)/

# compiles
/em&~(.*\Wnpm\W.*)/
```

Tradeoff:
 `\W` consumes a character on each side,
so the complement matches strings whose `npm` is bracketed by non-word characters
rather than just bordered by a word boundary.
Tokens at the absolute start or end of the scanned content
(no character before or after) still slip through.

### Use `\A`/`\z` instead of `^`/`$` inside complement bodies

This workaround compiled at 0.6.3,
 was rejected at 0.6.8,
 and compiles again at
v0.6.9 (the v0.6.9 cleanup restored it):

```text
# fails (all versions): ^/$ rewrite to lookarounds inside the complement
/em&~(^foo$)/

# compiles at 0.6.3 and v0.6.9; rejected at 0.6.8
/em&~(\Afoo\z)/
```

The semantics shift from "any line whose entirety is `foo`" to "the entire
scanned content is exactly `foo`",
 useful only when the rule already scans
whole-file content rather than per-line.
 Because this round-tripped across
versions (the 0.6.8 unsupported-pattern tightening rejected it,
 v0.6.9 readmits
it),
 prefer the literal-whitespace or `\W` substitutions above when you need a
version-stable rule;
 those compile on every version.

### Move the boundary check outside the complement (does not reliably work)

Lifting `\bword\b` out of `~(...)` into the main concatenation was once
recorded as a workaround,
 but it does not compile:

```text
# fails: Algebra(UnsupportedPattern)
/em&~(.*\bword\b.*)/

# also fails: Algebra(UnsupportedPattern) (verified in 0.6.0, 0.6.3, HEAD)
/em.*\bword\b/
```

A `\b` adjacent to a word-class atom still rewrites to a negative
lookbehind / lookahead,
 and `em.*` ahead of it forces the reverse pass
over a lookaround-bearing subtree,
 which hits the same
`UnsupportedPattern` reject.
 Prefer the literal-whitespace or `\W`
substitutions inside the complement body (above),
 which keep the rewrite
out of a reverse-over-lookaround position.

## What does not work

- **Splitting one complement across multiple rules.
  **
  Forbidden-strings combines rules via union,
  so any rule firing flags the line.
  Splitting makes detection more permissive,
   not less.
- **Inline `(?-m)` flag to disable multiline.
  **
  `/(?-m)em&~(^foo$)/` and variants still fail.
  The flag does not propagate into the complement body's parse context
  in the configurations tested,
   and the rewrite at
  `resharp-parser/src/lib.rs:1425-1441` runs against the surrounding
  flag state,
   not a locally-scoped override that reaches the assertion.
  Use `\A`/`\z` instead.
- **Wrapping the complement body in a non-capturing group with flag modifiers.
  **
  `/em&~((?-m:^foo$))/` fails identically;
  the `^`/`$` rewrite happens at AST translation,
   before the group's flag
  scope is applied to its children's positional semantics.

## Upstream filing artifact (do not file without an architectural prototype)

### Upstream filing decision

This repo's policy is to report an issue upstream only when all of the
following hold:
 we are absolutely sure it is the upstream's fault,
 they
can fix it,
 they are supporting the use case,
 the repo would welcome the
contribution,
 they are likely to fix it,
 and we have already prototyped a
minimal fix compatible with their architecture.
 Every reported issue that
does not satisfy all six is treated as a publicity incident.

Walking the six constraints against the resharp complement-of-lookaround
restriction:

1. **Is it really upstream's fault?
   ** Mostly no. The restriction is
   architectural.
    Brzozowski-style symbolic derivatives do not compose
   naturally with position-sensitive constraints under reversal;
    this
   doc's "Why this restriction exists" section spells out the algebraic
   reason.
    The default-multiline `^`/`$` rewrite and the `\b` to
   lookaround rewrite are defensible parser choices that interact badly
   with the architectural restriction;
    the badness lives in the
   interaction,
    not in any single decision.
    The only narrow surface-
   quality grievance is the generic "unsupported lookaround pattern"
   string not naming the trigger,
    but that is wording,
    not behaviour.

2. **Can upstream fix it?
   ** Partially.
    Positive-lookaround reverse cases
   are tractable via De Morgan body inversion;
    negative-lookaround
   reverse cases require preserving position-sensitive match-set
   semantics through the complement structure,
    which is non-trivial work
   touching the algebraic core.
    Not a 1-line change.

3. **Are they supporting this use case?
   ** No documented signal.
    The
   crate's stated value proposition is "high-performance regex engine
   with intersection and complement operations.
   " Lookarounds-in-
   complement sits at the intersection of two features that compose
   poorly;
    no upstream doc,
    example,
    or test shows the combination as
   expected to work.

4. **Would the repo welcome our contribution?
   ** Yes.
    The repo carries no
   `CONTRIBUTING.md`,
    issue template,
    pull-request template,
    or
   AI-assistance policy (checked the repository contents and the absent
   `.github/` directory),
    so no rule bars an external or AI-assisted
   filing;
    absence of a policy is not a fail.
    The positive signal is
   direct:
    issue
   [ieviev/resharp#5](https://github.com/ieviev/resharp/issues/5),
    a
   comparable detailed multi-bug report with proposed fixes from this
   project,
    was accepted and closed as completed,
    and the maintainer
   shipped the error-message fix in 0.6.4.

5. **Will they likely fix it?
   ** Upstream signal points the other way.
   Commit `e9676b4 2026-04-19 rejecting unsupported patterns, more
   tests` shows the project scoping down what is supported;
    commit
   `b256ea8 2026-04-24 rewrite negative lookaheads on construction`
   moved lookaround handling in a different direction (construction-
   time rewrites).
    The 0.5.1 to 0.5.2 delta was orthogonal (streaming/
   seeking,
    platform builds,
    prefix-engine bugfix).
    No movement on
   complement-of-lookaround in the visible history.

6. **Have we prototyped a minimal fix compatible with their
   architecture?
   ** No,
    and the auto-prototype step is not triggered:
    it
   fires only when constraints 1-5 hold or sorta-hold,
    and here
   constraints 1 and 5 fail outright (architectural fault,
    upstream
   leaning the other way).
    The "Suggested fix" section below is
   speculative design with no code,
    no correctness argument,
    no test
   against any nontrivial rule set.

We fail constraints 1,
 5,
 and 6 clearly;
 2 and 3 are equivocal at best,
while 4 is a clear yes.
 The decision is to not file the behavioural fix
upstream.

One part was separable and shipped:
 suggested fix 2 below (improve the
error message to name the surface trigger) is a wording change,
 not an
algebra change,
 so it cleared constraint 5 trivially and was folded into
the merged upstream issue
[ieviev/resharp#5](https://github.com/ieviev/resharp/issues/5);
 the
maintainer shipped the improved render string in 0.6.4.
 Only the
behavioural fix (actually supporting complement-of-lookaround),
 which
would touch the algebraic core,
 stays unaddressed;
 the behaviour is
unchanged through v0.6.9 (the shape still rejects,
 re-verified against
`264e85b`),
 and it is the single Bug A item still held back.

The consumer-side workaround is implemented in `forbidden-strings` as a
parse-time guard (`engine::lookaround_in_complement`) that rejects every
failing shape with a named-trigger error pointing to this doc.
 That
solves the user-facing problem at our boundary,
 where it actually
matters for us.
 The draft below is kept as a reference in case the
underlying situation changes (e.g.,
 upstream announces complement-of-
lookaround as supported,
 or someone in the project lands a prototype
fix and asks for community testing).
 Re-evaluating the six constraints
must precede any filing.

### Draft (do not file as-is)

Title:
 `Algebra(UnsupportedPattern)` for `\b`,
 `\B`,
 `^`,
 `$` inside
complement bodies;
 error string ("unsupported lookaround pattern") does
not mention the surface trigger

Labels:
 `bug`,
 `parser`,
 `documentation`

````md
## Description

Patterns of the form `A&~(B)` where `B` contains `\b`, `^`, `$`, or any
user-written lookaround fail at compile time with
`ResharpError::UnsupportedPattern`, rendered as "unsupported lookaround
pattern". Patterns where `B` contains `\B` (or where `\b`/`^` sit inside
a lookaround body that the parser cannot rewrite) instead fail with a
parse-layer `UnsupportedResharpRegex`. Neither error message names the
surface trigger.

The root cause chain:

1. The parser rewrites `^`/`$` to lookbehind/lookahead in default
   multiline mode (`resharp-parser/src/lib.rs:1425-1441`).
2. The parser rewrites `\b` to negative-lookahead / negative-lookbehind
   based on adjacent-atom classification
   (`resharp-parser/src/lib.rs:1305-1346`). When the helper cannot
   classify a `\b` or `\B` neighbour, the assertion falls through to
   the generic handler at `:1419-1424`, which rejects it with
   `UnsupportedResharpRegex`.
3. The reverse pass refuses to reverse a complement whose body contains
   any lookaround (`resharp-algebra/src/lib.rs:2234-2235`).

Legibility issue: a user writing `em&~(.*\bnpm\b.*)` has not written a
lookaround anywhere in the surface syntax, so the error "unsupported
lookaround pattern" is not actionable. A user writing
`em&~(.*\B.*)` gets a different error variant for the same conceptual
problem, fragmenting the symptom across two log surfaces.

## Reproduction

Against `resharp 0.5.2`:

```rust
use resharp::Regex;

// These four fail with Err(Algebra(UnsupportedPattern))
let _ = Regex::new(r"em&~(.*\bword\b.*)");
let _ = Regex::new(r"em&~(^foo$)");
let _ = Regex::new(r"em&~(\Afoo$)");
let _ = Regex::new(r"em&~((?=foo).*)");  // user-explicit lookaround in complement

// These two fail with Err(Parse(UnsupportedResharpRegex))
let _ = Regex::new(r"em&~(.*\B.*)");
let _ = Regex::new(r"(?<=\b)foo");
// NOTE: (?=^foo)bar compiles cleanly in 0.6.0/0.6.3/HEAD; do not include
// it as a failing case (the earlier draft listed it in error).
```
````

```rust
// These compile fine, demonstrating the trigger is positional, not size:
let _ = Regex::new(r"em&~(.*foo.*)").unwrap();
let _ = Regex::new(r"em&~(\Afoo\z)").unwrap();
let _ = Regex::new(r"em\b&_*&~(.*foo.*)").unwrap();
// 500 alternatives in a single complement compile cleanly
// 500 chained `&~(...)` complements compile cleanly
```

## Why this matters

Resharp's set algebra is the feature that makes complement-style
exclusion rules tractable;
 the rule shape `A&~(B)` is the primary use
case for choosing resharp over the standard `regex` crate.
 The natural
way to write "match A but not when bordered by token X" is
`A&~(.*\bX\b.*)`,
 and that fails opaquely.
 Users without algebra-layer
familiarity reach for alternation count or chain count as the suspected
trigger and report the wrong limit upstream.

## Suggested fix

Either of:

1. Lift the "no lookaround inside complement" restriction by handling
   the four lookaround reverse cases (`Kind::Lookahead` / `Kind::Lookbehind`,
   positive / negative) inline at the `Kind::Compl` arm of `reverse`
   (`resharp-algebra/src/lib.rs:2233`).
    Positive lookarounds can be
   pushed through De Morgan with body inversion;
    negative lookarounds
   require ensuring the complement structure of position-sensitive
   match-set semantics is preserved.

2. At minimum,
    improve the error message to name the surface trigger.
   `UnsupportedPattern` should distinguish "complement contains
   lookaround (introduced by `\b`/`\B`/`^`/`$` rewrite)" from
   "complement contains unhandled counted repetition" so users can map
   the error to a workaround without reading the algebra source.

## Workaround

Replace `\b` with literal whitespace or `\W` inside complement bodies;
use `\A`/`\z` in place of `^`/`$` when whole-content semantics are
acceptable.
 Move boundary assertions to the match site outside the
complement when the rule's intent permits.

---

## Other flags (low-confidence, un-reverified against v0.6.9)

These three were noted during the 0.6.4 source read and have not been
re-verified against v0.6.9;
 the line numbers below refer to 0.6.4 source.
 They
are latent or low-confidence,
 not confirmed live bugs:
 a non-saturating
min-length add not reachable under default limits,
 a `strip_lb`
acceptance-tightening,
 and a prefix-loop monotonicity question.

### Flag H: `get_bounded_length` min-length add is not saturating

`resharp-algebra/src/lib.rs:1061`,
 the `Kind::Concat` arm of
`get_bounded_length`:

```rust
(lmin + rmin, lmax.saturating_add(rmax))
```

The max-length add is saturating;
 the min-length add is a plain `+`.
 The
asymmetry suggests the max was hardened (consistent with the overflow
awareness behind the former Bug C / Bug F `saturating_add` fix) while the
min was missed.
 It is not reachable under
default limits:
 min-length accumulates through concatenation and is bounded
by the expanded node count,
 which `expanded_ast_limit` (50,000) caps far
below `u32::MAX`,
 so `lmin + rmin` cannot overflow for any pattern the
parser accepts.
 Latent defensive-consistency issue;
 the one-line fix is
`lmin.saturating_add(rmin)`.
 Flagged,
 not a live bug.

### Flag I: 0.6.4 `strip_lb` fail-closed rejects lookbehind shapes beyond the intersection case

`resharp-algebra/src/lib.rs:2005-2014`.
 The former Bug B fix (the
`strip_lb` fail-closed change shipped in 0.6.4) made `strip_lb`
return `Err(UnsupportedPattern)` (at `:2010-2011`,
 and `strip_lb_inner` at
`:2021`) when it cannot remove a lookbehind.
 `strip_lb` runs during
`find_all` for any lookbehind-bearing pattern,
 not only intersection
shapes,
 so 0.6.4 can now reject lookbehind patterns that 0.6.3 accepted,
including shapes with no intersection.
 forbidden-strings'
`intersection_with_lookbehind` pre-validator only guards `&` co-occurring
with `(?<=` / `(?<!`;
 a non-intersection lookbehind rule that `strip_lb`
cannot fully strip would surface `UnsupportedPattern` at scan time rather
than being caught by a pre-validator.
 This is a 0.6.3 to 0.6.4 acceptance
regression,
 not a soundness defect (the maintainer commented out their own
HTML-attribute,
 word-boundary,
 and user-agent tests with "TODO:
 reallow
once guaranteed 2 be correct").
 Whether it matters depends on whether any
production rule uses a lookbehind outside an intersection;
 the example
betterleaks config does not,
 so the practical risk is currently low.
Flagged as a scope item:
 a known-restriction class,
 not a new crash.

### Flag J: `prefix.rs` lookbehind fixpoint loop assumes monotonicity

`resharp-engine/src/prefix.rs:1006-1013` strips a lookbehind prefix in a
loop that breaks when `after == lb_stripped`:

```rust
loop {
    let stripped = b.strip_prefix_safe(lb_stripped);
    let after = b.nonbegins(stripped);
    if after == lb_stripped {
        break;
    }
    lb_stripped = after;
}
```

It terminates only if `strip_prefix_safe` then `nonbegins` is
monotone-shrinking toward a fixpoint.
 If that composition could oscillate
between two node ids for some input,
 the loop would not terminate.
 Low
confidence:
 these strip operations are normally monotone and no probe
triggered it.
 Flagged as a place to check if a future hang bisects into
`prefix.rs`,
 not a confirmed defect.

[resharp]: https://github.com/ieviev/resharp
