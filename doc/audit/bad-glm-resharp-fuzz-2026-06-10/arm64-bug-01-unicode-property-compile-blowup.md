# ARM64-BUG-1 Unicode property class compile blowup (ARM NEON path)

## Classification

- Type:
   performance,
   compile-time blowup (denial-of-service relevant).
- Phase:
   compile time,
   inside `Regex::new` / `with_options`.
- Severity:
   a 6-character pattern takes 7.25 seconds to compile on ARM64,
  versus 162ms on x86_64 (45x slower).
   Multiple distinct Unicode property
  class patterns cause timeouts under all option configurations.

## Minimal reproducers

```rust
//! Reproduction of the ARM64 compile-time blowup:
//! three Unicode property patterns whose `Regex::new` is far slower on ARM64 than on x86_64,
//! each commented with the timing or outcome that was observed.

/// The regex type whose construction time is measured.
use resharp::Regex;
/// The clock that times each construction.
use std::time::Instant;

// ARM64: ~7.25s, x86_64: ~162ms
let start = Instant::now();
let _ = Regex::new(r"\P{L}2");
eprintln!("{:?}", start.elapsed());

// ARM64: slow-unit (compile > 1s)
let _ = Regex::new(r"\p{L}");

// ARM64: timeout
let _ = Regex::new(r"\p{L}*&_*[A-Za-z]");
```

The fuzz campaign's `compile` target found these patterns across all three
build variants (pristine,
 nosimd,
 fork),
 confirming the defect is not
SIMD-specific but is amplified by the ARM64 compilation path.

## Observed behaviour

Compile time measured from the ARM64 cargo-fuzz `compile` target (single
`Regex::with_options` call,
 `-timeout=10`):

<table>
<thead>
<tr>
<th>Pattern</th>
<th>Variant</th>
<th>Option config</th>
<th>Compile time</th>
</tr>
</thead>
<tbody>
<tr>
<td>`\P{L}2`</td>
<td>pristine</td>
<td>default (0x2a % 6 = 0)</td>
<td>7252 ms (timeout)</td>
</tr>
<tr>
<td>`\P{L}2`</td>
<td>pristine</td>
<td>flag bundle (0x2a)</td>
<td>7252 ms (timeout)</td>
</tr>
<tr>
<td>`{8,}\b\w+\r`</td>
<td>pristine</td>
<td>flag bundle (0x7b % 6 = 5)</td>
<td>~6986 ms (slow-unit)</td>
</tr>
<tr>
<td>`\w+\b*`</td>
<td>pristine</td>
<td>flag bundle (0x5d % 6 = 5)</td>
<td>timeout</td>
</tr>
<tr>
<td>`\p{L}*&amp;_*[A-Za-z]`</td>
<td>nosimd</td>
<td>ascii (0x5b % 6 = 5)</td>
<td>timeout</td>
</tr>
<tr>
<td>`\P{L}.[[^a-z]0l:]]`</td>
<td>nosimd</td>
<td>ascii (0x37 % 6 = 1)</td>
<td>slow-unit</td>
</tr>
<tr>
<td>`\p{L}`</td>
<td>fork</td>
<td>ascii (0x3d % 6 = 3)</td>
<td>slow-unit</td>
</tr>
<tr>
<td>`Sherl\P{L}ock</td>
<td>Holmes`</td>
<td>fork</td>
<td>ascii (0x53 % 6 = 5)</td>
<td>slow-unit</td>
</tr>
<tr>
<td>`Script(?m)\P{L}_:._**`</td>
<td>fork</td>
<td>default (0x28 % 6 = 4)</td>
<td>timeout</td>
</tr>
</tbody>
</table>

## Expected behaviour

Compilation of a small pattern completes in well under a second.
 The dotnet
reference compiles all these patterns instantly.
 On x86_64,
 `\P{L}2` compiles
in 162ms (still slow for a 6-character pattern,
 but not a timeout).

## Root cause

The derivative/minterm construction for Unicode property classes (`\p{L}`,
`\P{L}`) under the ARM64 compilation path has a much steeper cost curve than
the x86_64 AVX2 path.
 The v0.6.12 fix for BUG-11/17 (bracketed perl class
repeat compile blowup) addressed the `[\w]`-bracketed class case but did not
cover Unicode property classes,
 which take a different compilation path.

On ARM64,
 the property-class minterm enumeration appears to enumerate a much
larger state space than on x86_64.
 The NEON code path (`neon.rs`) is used at
match time,
 not compile time,
 so the nosimd variant exhibiting the same
defect confirms the issue is in the algebraic derivative construction
(`resharp-algebra`),
 not the NEON match path.

## Relationship to 06-04 campaign

This is the ARM64-specific face of **BUG-11/BUG-17** from the 2026-06-04
campaign.
 The root cause is the same (super-linear derivative/state
construction for large character classes),
 but the ARM64 path triggers it on
Unicode property class patterns that are fast (or at least much less slow)
on x86_64.
 The developer's claim that BUG-11/17 were fixed is partially
correct for x86_64 bracketed-perl-class patterns,
 but the underlying
super-linear cost remains for Unicode property classes and is amplified on
ARM64.

## Source location

The derivative construction is in `resharp-algebra/src/lib.rs`.
 The Unicode
property class expansion happens in the parser
(`resharp-parser/src/lib.rs`),
 which converts `\p{L}` to a large character
class that the algebra then processes.
 The minterm enumeration cost scales
with the size of this class.
