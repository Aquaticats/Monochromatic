# ARM64-BUG-3 `\w+\b*` compile timeout on ARM64

## Classification

- Type:
   performance,
   compile-time blowup (denial-of-service relevant).
- Phase:
   compile time,
   inside `Regex::with_options`.
- Severity:
   the pattern `\w+\b*` causes a compile timeout on ARM64 but is
  rejected as "unsupported pattern" in ~1ms on x86_64.

## Minimal reproducer

From fuzz artifact `pristine_compile_timeout-05ed1261fa4261700ed203d26e07e7896d177052`:

```text
Raw bytes: 5d 5c 77 2b 5c 62 2a
Option byte: 0x5d (config index 0x5d % 6 = 5 = flag bundle)
Pattern (after option byte): \w+\b*
```

The ARM64 cargo-fuzz `compile` target on pristine reported a timeout for this
input.

On x86_64,
 `Regex::new(r"\w+\b*")` returns an error in ~1ms:
"unsupported pattern:
 eg.
 lookaround,
 `\b`/`^`/`$` inside a complement `~(...)`) or a star `*`".

## Observed behaviour

ARM64:
 compile timeout (> 10 seconds).
x86_64:
 rejected as unsupported pattern in ~1ms.

## Root cause

The `\b*` (word boundary as star operand) is rejected by the v0.6.12 parser
on x86_64 with the "unsupported pattern" check.
 On ARM64 with the
flag-bundle configuration (option byte 0x5d,
 config index 5),
 the pattern
may either:
1.  Bypass the parser's check and reach the compile phase,
     where the large
    minterm set from `\w+` combined with `\b` causes super-linear construction,
     or
2.  The rejection path itself is somehow bypassed under the flag-bundle
    configuration (e.g.,
     `ignore_whitespace` or `case_insensitive` changes
    how `\b*` is parsed).

This is another case where the v0.6.12 parser's "reject instead of fix"
approach fails to cover all configurations:
 the check that rejects the
pattern on x86_64 does not fire (or fires late) on ARM64 under certain
option configurations.

## Relationship to 06-04 campaign

BUG-4 from the 06-04 campaign was the NO_MATCH sentinel leak triggered by
`\Bb+`,
 `(?<=[^a])b+`,
 `~(_*$)`.
 The v0.6.12 fix rejects patterns with
`\b`/`\B` inside a star or complement,
 which avoids the sentinel leak but
does not fix the underlying engine defect.
 This ARM64 finding shows the
parser rejection is incomplete:
 certain option configurations allow the
defective pattern to reach the compile phase,
 where it causes a timeout
instead of a sentinel leak (the sentinel leak fix works,
 but the compile
cost is still super-linear).
