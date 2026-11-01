# `arbitrary::Unstructured::choose` returns the first choice on empty input

Status:
 **diagnosed and worked around** in `package/jsonc-edit/core-rust.fuzz/src/generators.rs`.
 This is a documented behavior of the `arbitrary` crate,
 not a defect in it;
 the defect was an unbounded retry loop built on top of it.

## Symptom

A `cargo test --lib` run in the new JSONC fuzz sidecar never finished.
Two test binaries sat at 99% CPU for over half an hour,
one of them the survivor of an earlier `mise run test` whose wrapper had already been killed:

```text
2216311 Sl   99.2       33:27 .../jsonc_edit_fuzz-... --exact generators_tests::recorded_comment_bodies_appear_in_the_source --nocapture
```

The hanging test did not parse,
emit or edit anything.
It only drew documents from twelve-byte seeds,
so the loop was inside the generator.

## Root cause

The generator had to avoid duplicate object keys,
which the JSONC contract rejects,
so it retried until it drew an unused name:

```rust
let key = loop {
    let candidate = u.choose(&KEYS)?;
    if !used_keys.contains(candidate) {
        break *candidate;
    }
};
```

`arbitrary` 1.4.2 documents `choose` as returning the **first** choice rather than an error once
the byte budget is empty (`src/unstructured.rs`,
the doc comment above `pub fn choose`):

> Returns the first choice,
> not an error,
> if this `Unstructured` [is empty].

With a short seed the budget runs out,
`choose` then returns `KEYS[0]` forever,
and if `"a"` is already used the loop can never break.
`int_in_range` does **not** behave this way:
it returns `Err(NotEnoughData)` on an exhausted buffer,
which is why every other draw in the generator terminated normally and only the retry loop hung.

## Fix

Bound the retries and fall back deterministically.
`KEYS` is wider than the largest generated container,
so an unused name always exists:

```rust
fn fresh_key<'b>(u: &mut Unstructured<'_>, used: &[&'b str]) -> ArbitraryResult<&'b str> {
    for _ in 0..KEYS.len() {
        let candidate = u.choose(&KEYS)?;
        if !used.contains(candidate) {
            return Ok(*candidate);
        }
    }
    for candidate in KEYS.iter() {
        if !used.contains(candidate) {
            return Ok(*candidate);
        }
    }
    return Err(arbitrary::Error::NotEnoughData);
}
```

After the change all thirteen sidecar tests finished in well under a second.

## Rules this generalizes to

- Never retry `choose` in an unbounded loop.
   Its empty-input contract is "first choice",
   so exhaustion looks like an unlucky streak rather than an error.
- A test process that outlives its killed wrapper keeps burning CPU.
   `mise run test` timing out left the test binary spinning,
   so after any timeout kill the survivor by PID
   (`pkill -9 -f <binary name>`) before drawing conclusions from a later run.
- Piping `cargo test` through `tail` or `rg` hides a hang:
   the pipe buffers,
   so nothing appears until the process dies.
   Redirect to a file,
   or run each test with `--exact` under a per-test bound,
   to identify which one stalls.

## References

- `arbitrary` 1.4.2,
   `src/unstructured.rs`:
   `choose` (line 419) and `int_in_range` (line 300).
- `package/jsonc-edit/core-rust.fuzz/src/generators.rs`:
   `fresh_key` and its two call sites.
