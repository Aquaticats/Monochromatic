# BUG-22 is_match/find_all/stream are O(n^2) on a repetitive prefix with a failing suffix

## Classification

- Type:
   performance,
   match-time algorithmic complexity (ReDoS class).
   The answer is
  correct;
   the time is quadratic where a derivative engine must be linear.
- Phase:
   match time,
   the forward-prefix path (`is_match_fwd_prefix`,
  `find_all_fwd_prefix`).
- Severity:
   bug.
   Under every limits-enabled config (default,
   ascii,
   full,
   js,
  hardened) `is_match` on `(a+)+b` over a 64 KiB run of `a` takes 4.4 s and scales as
  O(n^2);
   `stream` under ascii exceeds the 25 s watchdog.
   ieviev's invariant is that
  nothing should take >= 10 s without disabling limits;
   this crosses it on ~96 KiB
  and is already badly super-linear well below that.
   A membership test (`is_match`,
  a single boolean) should never be worse than linear.

## Minimal reproducer

```rust
//! Reproduction of the quadratic `is_match`:
//! on an input of one repeated letter with no match,
//! doubling the input quadruples the time.

/// The regex type whose `is_match` is quadratic.
use resharp::Regex;
let re = Regex::new(r"(a+)+b").unwrap();
let hay = vec![b'a'; 65536];   // no 'b' anywhere: the match fails
let _ = re.is_match(&hay);      // ~4.4 s; doubling the input quadruples the time
```

`(a|a)*b` behaves identically.
 Both return the correct answer (`false` on an all-`a`
haystack,
 `true` on `aaab`);
 only the time is wrong.

## Observed behaviour

`is_match`,
 config default,
 `hay = "a" * N` (no `b`),
 wall clock:

```text
N= 8192  0.068 s
N=16384  0.279 s   (x4.1 for 2x input)
N=32768  1.093 s   (x3.9)
N=65536  4.417 s   (x4.05)
```

`stream` scales the same way (0.017 / 0.067 / 0.275 / 1.097 s at N = 4096 / 8192 /
16384 / 32768),
 and `stream` under the ascii config exceeded the 25 s watchdog at
N = 65536.
 Doubling the input consistently quadruples the time:
 O(n^2).

Hardened does not help (identical times:
 0.067 / 0.279 / 1.099 / 4.363 s),
 which
distinguishes this from BUG-18:
 hardened only swaps the `find_all` driver,
 and this
defect is in the forward-prefix path that `is_match` and `find_all` both use.

The pure quantifier `(a+)+` (no trailing literal) is instant (0.0001 s at N = 32768),
as are `(a*)*`,
 `(aa)+`,
 `a+`,
 `(a+){2}`,
 `((a+)+)+`,
 `(a|a)*`,
 `(a+|a)*`.
 The trigger
is specifically a repetitive single-byte prefix plus a discriminating suffix literal
that is absent from the long run,
 so the full match keeps failing.

## Expected behaviour

`is_match` and `find_all` must be linear in the input length for a fixed pattern.
 A
derivative engine compiles `(a+)+b` to a small DFA (measured:
 5 forward states,
 4
reverse states,
 constant in N,
 so this is not state explosion);
 one linear pass
decides membership.

## Root cause

`(a+)+b` and `(a|a)*b` both select an `AnchoredFwd` prefix of the single byte `a`
(compile-time `debug` trace:
 `[fwd-prefix] anchor=pos0 (1 bytes)`,
`[prefix] selected=AnchoredFwd`).
 `is_match` then routes through
`is_match_fwd_prefix` (`resharp-engine/src/fwd.rs:61`):

```rust
//! The prefix search loop of `is_match`:
//! after a candidate fails it advances by one byte,
//! so the next candidate rescans bytes that the previous candidate already read.
// fwd.rs:78
let mut search_start = 0;
while let Some(candidate) = fwd_prefix.find_fwd(input, search_start) {
    let state = inner.fwd.walk_input(&mut inner.b, candidate, prefix_len, input)?;
    if state != 0 {
        let max_end = inner.fwd.scan_fwd_from(&mut inner.b, state, candidate + prefix_len, input)?;
        if max_end != engine::NO_MATCH && max_end > candidate {
            return Ok(true);
        }
    }
    search_start = candidate + 1;   // fwd.rs:92  <-- advance by ONE
}
```

The prefix is the byte `a`,
 so `find_fwd` returns a candidate at every position of
the run:
 0,
 1,
 2,
 ...,
 n-1.
 For each candidate,
 `scan_fwd_from` walks forward through
the remaining `a`s searching for `b`,
 traversing ~`n - candidate` bytes before
failing.
 After the failure,
 `search_start` advances by a single byte (`candidate +
1`),
 so the next candidate re-scans nearly the same suffix.
 The total work is
`sum_{c=0}^{n-1} (n - c) = O(n^2)`.

`find_all_fwd_prefix` (`fwd.rs:4`) has the same shape and the same defect:
 on a
successful match it skips ahead (`search_start = max_end`,
 `fwd.rs:47`),
 but on a
failed forward scan it falls through to `search_start = candidate + 1` (`fwd.rs:51`),
so a long non-matching run is re-scanned from every position.
 `stream` uses an
analogous forward driver and blows up the same way.

The fix is to advance `search_start` past the region the failed `scan_fwd_from`
already proved cannot start a match,
 rather than by one byte,
 or to decide
`is_match` with a single linear DFA pass (the bounded forward/reverse DFA already
exists) instead of the prefix-rescan loop.
 A membership query in particular never
needs the per-occurrence anchored re-scan.

## Affected configurations

Default,
 ascii,
 full,
 js,
 and hardened all reproduce (the prefix path is shared and
hardened does not change it).
 Only the limits-disabling `unbounded_size` config is
exempt by policy,
 and it would be slower,
 not faster.
 ascii is the worst for
`stream` (watchdog timeout at 64 KiB).

## Relationship to other findings

- Same O(n^2) "re-scan per position" anti-pattern family as BUG-18,
   but a distinct
  defect:
   BUG-18 is `find_all_nullable_slow` (`lib.rs:1794`) restarting per position
  on a nullable complement and is mitigated by the hardened driver;
   BUG-22 is
  `is_match_fwd_prefix`/`find_all_fwd_prefix` (`fwd.rs:61`/`:4`) re-scanning per
  prefix occurrence,
   hits `is_match` (not just `find_all`),
   is triggered by a
  repetitive prefix plus a failing discriminating literal,
   and is not mitigated by
  hardened.
   The two live in different functions and need separate fixes.
- Distinct from BUG-16 (O(n^3) lookbehind-of-lookahead) and BUG-17 (compile-time
  blowup):
   this is a match-time forward-prefix re-scan with a bounded DFA.

## Code quality

The failing-scan branch advancing `search_start` by one byte (`fwd.rs:92`,
`fwd.rs:51`) is the kind of "anchored search retried at every occurrence" that a
reasonable author flags on sight once the prefix is short and repetitive.
 The
information to skip is already in hand:
 `scan_fwd_from` knows how far it advanced,
 so
the loop can resume past that reach instead of one byte later.
 Worth converging the
`is_match` prefix path onto the linear DFA membership check it already has,
 since a
boolean query has no reason to run the per-occurrence anchored scan at all.
