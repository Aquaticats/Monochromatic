# bug-13: intersection with an optional lookahead leaks the consuming width

> Secondary class (see `dotnet-adjudication.md`):
>  the dotnet reference REJECTS
> this pattern at compile ("nested lookarounds").
>  rust's `ensure_supported_rec`
> guard accepts it,
>  then leaks the width.
>  Implementation divergence:
>  rust should
> reject (as the reference does) or compute the zero-width result.
>  The Lean value
> corroborates that rust's span is wrong;
>  it is not the sole authority.

Severity:
 soundness (silent,
 both debug and release;
 find_all AND find_anchored).
Found by the Lean position-level differential (seed-4004 run,
 case R48,
 trust0,
anchor-free,
 translator-faithful).
 Confirmed on the unmodified v0.6.12 stock
crate and against the Lean ground truth.
 This is a match-END over-extension (a
span returned TOO LONG),
 the opposite direction from bug-11/bug-12 (which drop
matches),
 and it lives in the forward match-extension path,
 not the null-start
collection.

Architecture:
 confirmed byte-identical on aarch64 (Apple M1) and x86-64.
`armprobe "a?&(?=a)?" "ab"` on the M1 returns `find_all=[(0,1),(1,1),(2,2)]`
(Lean ground truth is `0:0`;
 the leading `0:1` over-extends the zero-width
result),
 the same wrong result as x86.
 A later focused Lean round independently
re-found this family on ARM via form B:
 `armprobe
"((\W|((?!c)))&((_&[acd])&a))" "a"` returns `find_all=[(0,1)]` where Lean says no
match (a zero-width lookaround-in-union side `&`-intersected with a consuming
side leaks the consuming span).
 The ARM runs make "ARM-confirmed" demonstrated
rather than inferred.

## Minimal reproducer

```rust
//! Reproduction of the width leak of an optional lookahead under intersection:
//! the first span is one byte wide where it must have zero width.
// stock resharp v0.6.12, default config
/// The regex type and the option set that its construction takes.
use resharp::{Regex, RegexOptions};

let re = Regex::with_options(r"a?&(?=a)?", RegexOptions::default()).unwrap();
re.find_all(b"ab");        // -> [(0,1),(1,1),(2,2)]   WRONG: first span (0,1)
re.find_anchored(b"ab");   // -> Some((0,1))           WRONG
// correct: [(0,0),(1,1),(2,2)] / Some((0,0))
```

Identical in debug-assertions and release builds (no panic).
 `is_match` is
`true` (correct:
 a match exists),
 so only the span is wrong.

## Why (0,1) cannot be a match

`a?` is the language `{"", "a"}`.
 `(?=a)?` is `(?=a) | ε`;
 both alternatives are
zero-width,
 so its language is `{""}`.
 The intersection `a? & (?=a)?` requires
the SAME span on both sides,
 and the only common member is the empty string:

```txt
(a | ε) & ((?=a) | ε)
  = (a & (?=a)) | (a & ε) | (ε & (?=a)) | (ε & ε)
  =   EMPTY     |  EMPTY   | zero-width@next=a | zero-width
  = {""}   (zero-width only)
```

`a & (?=a)` is empty because a width-1 span and a zero-width assertion cannot be
the same span.
 So every match of `a? & (?=a)?` is zero-width;
 `(0,1)` (the span
`"a"`) is not in the language.
 The engine returns it anyway:
 it lets the
consuming `a` match while the satisfied lookahead `(?=a)` holds,
 conflating
"the lookahead assertion holds at this position" with "the lookahead is part of
this span",
 and the consuming width leaks into the result.

## Ground truth (Lean `llmatch`)

```txt
hay   Lean first span   rust find_all              rust find_anchored
"ab"  0:0               [(0,1),(1,1),(2,2)]        Some((0,1))
"a"   0:0               (analogous)                Some((0,1))
"b"   0:0               (analogous)                ...
```

Since the leftmost match is at offset 0,
 `llmatch`'s `0:0` is also the
longest-anchored-at-0 span that `find_anchored` must return;
 rust's `Some((0,1))`
is longer than the true match,
 confirming the leak in `find_anchored` too.

## Trigger shape (general): zero-width on one side, consuming on the other

The general fault:
 in `X & Y`,
 when one side can match ZERO-WIDTH at position `p`
(via a lookahead whose assertion holds) and the other side CONSUMES a char at `p`,
the engine pairs the zero-width match with the consuming match instead of
requiring both sides to span the same length,
 so the consumed width leaks.
 Two
distinct surface forms both reduce to this and were each Lean-confirmed:

```txt
# form A: nullable-consuming  &  satisfied-optional-lookahead
a?&(?=a)?   ab -> [(0,1),...]   leak  ((?=a) holds: next is 'a')
a?&(?!b)?   ab -> [(0,1),...]   leak  ((?!b) holds: next is not 'b')
a?&(?=c)?   ab -> [(0,0),...]   ok    ((?=c) false at 0 -> falls back to zero-width)
a&(?=a)?    ab -> []            ok    (in this form the consuming side must be nullable)

# form B: (consuming-class | satisfied-lookahead)  &  consuming
(\W|(?!c))&a   a -> [(0,1)]     leak  (\W cannot match 'a'; (?!c) holds zero-width;
                                        the right-hand consuming 'a' leaks through)
(\d|(?!c))&a   a -> [(0,1)]     leak
(\W|(?=a))&a   a -> [(0,1)]     leak
(?!c)&a        a -> []          ok    (no consuming-class branch to mask the leak)
```

Both forms share the essential pair:
 a satisfied zero-width lookahead on one side
of `&` and a consumed character on the other.
 Form A needs the consuming side
nullable so the zero-width branch exists there;
 form B puts the zero-width branch
in an alternation on the opposite side,
 so the consuming operand need NOT be
nullable.
 The earlier "both operands must be nullable" reading was too narrow:
`(\W|(?!c))&a` leaks with a non-nullable consuming `a`.
 When the lookahead is
unsatisfied the engine correctly produces no leak (`a?&(?=c)?`).
 Form B was found
by the focused Lean round (seed-5005 R292,
 minimized to `(\W|(?!c))&a`);
 Lean
gives `none` for all three form-B leakers.

## Distinctness

- vs bug-11 / bug-12:
   those are null-START faults that DROP matches (forward
  rejects a proposed start;
   reverse never proposes one).
   bug-13 is a match-END
  over-extension that ADDS a too-long span.
   Opposite direction,
   different path
  (forward match-extension / intersection nullability vs reverse-null
  collection).
- vs bug-10:
   bug-10 is `find_anchored` returning a span SHORTER than the true
  longest (complement-with-end-anchor).
   bug-13 returns one LONGER than the true
  match (intersection-with-optional-lookahead).
   Opposite direction,
   different
  trigger.
- vs the 2026-06-04 BUG-13 ("lookahead width leak",
   `(?=(?=c)c{1,3})`,
  recorded fixed on its trigger):
   same THEME (a lookahead's width leaking into a
  match) but a distinct,
   live construct (intersection of two nullables rather
  than a nested lookahead).
   The width-leak class is not fully closed in v0.6.12;
  it survives through `&`.

## Provenance

Lean lane,
 seed-4004 run:
 R48 (`((a)?&(((?=a)))?)` on `"a\nb"`,
 lean `0:0` vs
rust `0:1`),
 minimized to `a?&(?=a)?` on `"ab"`.
 The internal oracles miss it:
the result is internally self-consistent (find_all sorted,
 non-overlapping,
agreeing with is_match;
 find_anchored at offset 0),
 just positionally too long,
so only the external position reference exposes it.
