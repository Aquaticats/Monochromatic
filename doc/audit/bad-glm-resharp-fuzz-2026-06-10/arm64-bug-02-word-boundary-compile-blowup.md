# ARM64-BUG-2 Word boundary + word class compile blowup

## Classification

- Type:
   performance,
   compile-time blowup (denial-of-service relevant).
- Phase:
   compile time,
   inside `Regex::with_options`.
- Severity:
   `{8,}\b\w+\r` takes ~7s to compile on ARM64.

## Minimal reproducer

From fuzz artifact `pristine_compile_slow-unit-78703a7e91343a980d4b29129f575d24fa6805a5`:

```text
Raw bytes: 7b 38 2c 7d 5c 62 5c 77 2b 5c 72
Option byte: 0x7b (config index 5 = flag bundle)
Pattern (after option byte): {8,}\b\w+\r
```

On ARM64,
 replayed in 6986ms. On x86_64,
 this pattern is rejected as a parse
error (`{8,}` with no preceding element).

## Observed behaviour

The cargo-fuzz `compile` target on pristine ARM64 reported a slow-unit for
this input.
 Replay confirms 6986ms compile time on ARM64.

On x86_64,
 `Regex::new(r"{8,}\b\w+\r")` returns a parse error in ~10µs.
 This
discrepancy suggests either:
1.  The ARM64 parser accepts `{8,}` as a valid regex where x86_64 does not,
     or
2.  The option byte 0x7b selects a configuration where the pattern parses
    differently (the flag-bundle config with case-insensitive + ignore-whitespace
    + dot-matches-newline + no-multiline),
       or
3.  The compile path on ARM64 takes a different branch for this input.

This needs further investigation:
 the fuzz input's option byte 0x7b means the
compiler uses config index `0x7b % 6 = 5`,
 which is the flag bundle.
 The flag
bundle includes `ignore_whitespace(true)`,
 which could change how `{8,}` is
parsed (whitespace in the pattern body is ignored,
 potentially altering the
tokenisation).

## Expected behaviour

Either the pattern is rejected as a parse error (consistent with x86_64),
 or
if it is accepted,
 compilation completes in well under a second.

## Root cause

Under investigation.
 Two hypotheses:

1.  **Whitespace-flag parsing difference**:
     the `ignore_whitespace` flag may
    alter how the leading `{8,}` is tokenised on ARM64,
     causing a valid (but
    slow-to-compile) parse.
2.  **Common root with ARM64-BUG-1**:
     if the pattern does compile,
     the `\b\w+`
    portion triggers the same Unicode property class / word boundary minterm
    enumeration cost that causes ARM64-BUG-1.

## Relationship to 06-04 campaign

This is likely a new trigger for the same root cause as BUG-11/17/ARM64-BUG-1
(super-linear derivative construction for large character classes),
 but
specifically exposed by the ARM64 flag-bundle configuration that changes
parsing behaviour.
