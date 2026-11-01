# json-number 0.4.10: checked number creation lacks a documented unsized layout guarantee

Status:
 **source-level portability gate**,
 not a reproduced runtime memory fault.
 The published crate was not executed as a number foundation after this finding.
 A separate safe repository-owned number prototype and its consumer passed their existing tests.
 No upstream issue was filed.

## Symptom

A consumer calling the apparently checked `json_number::Number::new("1")` or
 `json_number::NumberBuf::new(b"1".to_vec())` passes through an unsafe conversion
 of `&[u8]` to `&Number`.
 `Number` is a single-field dynamically sized struct with the default Rust representation,
 not `#[repr(transparent)]`.
 The [Rust Reference][layout-rust] does not guarantee that its byte field starts at the slice's address.
 Consequently,
 the published source does not establish the portable layout proof
required before accepting this crate as a memory-safe dependency.
 This is **not** a claim that a particular rustc build crashes or demonstrably produces undefined behavior;
 there is no runtime error message to quote.

## Root cause

The reviewed upstream checkout is `timothee-haudebourg/json-number` at
 `283af83056759712e0f9888b8d8b27278cb00792`.
 The published 0.4.10 `src/lib.rs` has the same SHA-256 as the checkout:
 `467186254f8695eba06e5065759056c48b6b832c89dbe39d43d52b73981deff0`.
 The archived package checksum is
 `479dfd2ad8e4b4ae076b031f72ef2f3791f65e2a0f51e5f3408dbf716c4c2f82`.
 This finding applies to that exact release,
 not merely a branch sharing its version string.

1. `src/lib.rs:110-120` defines the unsized wrapper without any representation attribute:

   ```rust
   // src/lib.rs:110-120
   #[derive(PartialEq, Eq, PartialOrd, Ord, Hash)]
   pub struct Number {
       data: [u8],
   }
   ```

   Its derived equality compares the raw bytes,
   so `1` and `1.0` remain unequal without a separate mathematical-identity layer.

2. `Number::new` scans JSON number bytes,
   then calls the unchecked constructor even on the checked public path
   (`src/lib.rs:121-203`):

   ```rust
   // src/lib.rs:188-197
   if matches!(
       state,
       State::Zero | State::NonZero | State::FractionalRest | State::ExponentRest
   ) {
       Ok(unsafe { Self::new_unchecked(s) })
   } else {
       Err(InvalidNumber(data))
   }
   ```

3. `new_unchecked` transmutes a byte-slice reference into a `Number` reference
   (`src/lib.rs:199-207`).
   The owned `NumberBuf::new` entry calls `Number::new`,
   and its `as_number` also calls `new_unchecked`
   (`src/lib.rs:524-537,579-584`).
   An external wrapper using the checked owned constructor cannot avoid this path:

   ```rust
   // src/lib.rs:199-207
   pub unsafe fn new_unchecked<B: AsRef<[u8]> + ?Sized>(data: &B) -> &Number {
       std::mem::transmute(data.as_ref())
   }
   ```

4. The [Rust Reference][layout-rust] says default `repr(Rust)` makes only field-alignment,
   type-alignment and non-overlap guarantees,
   with no other data-layout guarantees.
   The [transparent representation][layout-transparent] explicitly guarantees
   the same layout and ABI as its only nonzero field.
   [DST metadata rules][dst] say a struct with an unsized tail shares the tail's metadata;
   matching metadata does **not** independently prove that its field starts at offset zero.
   [`transmute` documentation][transmute] additionally requires the resulting reference
   to be valid for its destination type,
   not merely the same size as its source.
   Grammar validation ensures ASCII bytes for `as_str`;
   it cannot establish the missing struct-layout condition.

## Verification

- Release evidence:
   published `json-number` 0.4.10 source and commit `283af83056759712e0f9888b8d8b27278cb00792`,
   with matching source hash and archived checksum recorded here.
- Source audit:
   reading the published `src/lib.rs:110-120,188-214,524-537,579-584`
   shows an unannotated unsized wrapper and two checked-entry paths to the transmute.
   The Rust Reference and `std::mem::transmute` documentation establish the missing guarantee.
   This is a language-contract check,
   not an empirical offset or Miri result.
- Independent safe comparison:
   `cd -- ~/temp/agent/jsonc-exact-number-probe && mise run test` passed six unit tests,
   covering equal and unequal decimal spellings,
   malformed tokens,
   huge exponent arithmetic,
   exact rational oracle comparisons and hash/equality consistency.
   `cd -- ~/temp/agent/jsonc-exact-number-consumer && mise run test` imported the prototype
   and printed `consumer exact-number equality passed`.
   These commands test the owned prototype,
   **not** the published `json-number` unsafe conversion.

- Source-admitted grammar examples:
   `1`,
   `1.0`,
   `1e0` and `-0` reach the checked constructor's accepted states.
   Their values would require a separate exact comparator because the crate derives lexical equality.
- Source-rejected grammar examples:
   `+1`,
   `00`,
   `1e+` and an empty input do not reach the accepting states.
   These are conclusions from the state machine,
   not runtime claims about an executed candidate.
- Portability failure condition:
   default Rust representation gives no field-offset guarantee for the transmuted target.
   No executable input can demonstrate the absence of a language guarantee;
   a test that happens to pass on one compiler would not resolve it.

For a repeatable source check,
 inspect the pinned published archive and the public Rust guarantees:

```bash
# doc/troubleshooting/json-number-unsized-layout.md
rg --line-number 'pub struct Number|pub fn new_unchecked|std::mem::transmute|fn as_number' "$HOME"/.cargo/registry/src/*/json-number-0.4.10/src/lib.rs
```

The registry cache path in this command is an example from the audited machine;
 use the local Cargo registry path on another machine.

## Verified workarounds

- Use the repository-owned safe JSON-number validator and decimal-identity prototype
   in `~/temp/agent/jsonc-exact-number-probe/`,
   with raw token spelling retained separately by the caller.
   Its unit and disposable consumer commands passed in this audit.
   The tradeoff is maintaining the scanner and normalization code locally;
   the scratch crate is **not** adopted product code and has not completed the full technology vet.

## What does not work

- Wrapping `NumberBuf::new` and delegating equality to a safe normalizer:
   `NumberBuf::new` already calls `Number::new`,
   so a later wrapper does not remove the unsafe layout assumption.
- A passing sample program,
   Miri run or measured field offset on one build:
   this could probe that compiler's implementation,
   but cannot supply a missing cross-platform Rust language guarantee.
- Calling `new_unchecked` only after our own syntax validation:
   the source still transmutes to a type without a guaranteed transparent layout.

## Upstream filing decision

No matching entry exists in this repository's `.out-of-scope/` list.
 Upstream searches for `repr transparent` and `transmute` across issues and PRs found no matching thread.
 Upstream [issue 4][issue-four] addresses a separate historical `lexical` dependency concern;
 its maintainer states that issue was fixed in version 0.4.9.
 No report or patch was sent.

1. **Upstream fault**:
   unresolved.
   The public docs do not establish the needed layout guarantee,
   but this audit has not proved a concrete invalid result
   or checked a formal compiler guarantee outside the cited Reference.
   Do not equate an unproved safety case with demonstrated undefined behavior.
2. **Fixability**:
   yes in principle.
   A separately reviewed transparent representation
   and justified pointer conversion could supply a documented layout boundary.
3. **Supported use case**:
   yes for checked raw JSON-number token storage (`src/lib.rs:110-128`).
   Mathematical equality is explicitly **not** claimed by upstream.
4. **Contribution welcome**:
   `README.md:43-48` specifies dual licensing for submitted contributions.
   The checkout has no `CONTRIBUTING.md` or `.github/` policy;
   no ban on outside or AI-assisted reports was found in the inspected files.
5. **Maintainer disposition**:
   neither searched tracker results nor the README state a refusal.
   This is not a prediction of acceptance.
6. **Prototype**:
   not attempted.
   Constraint 1 remains unresolved,
   so the auto-prototype rule for constraints 1 through 5 holding does not apply.
   A separate fork would need representation,
   metadata,
   UTF-8 invariant and consumer tests before reconsideration.

The draft is **do not file as-is**:
 it asks for clarification rather than asserting a proven incident,
 and no upstream communication is authorized by this port request.

~~~md
Title: Does `Number` need transparent layout for its checked slice conversion?

In json-number 0.4.10,
`Number` wraps `[u8]` without `#[repr(transparent)]`
(`src/lib.rs:110-120`).
`Number::new` checks grammar and then calls `new_unchecked`,
which transmutes `&[u8]` to `&Number` (`src/lib.rs:188-207`).
`NumberBuf::new` reaches the same operation.

The Rust Reference's default representation does not document the target
field offset,
whereas `#[repr(transparent)]` would guarantee the single field's layout.
Can the checked constructor's conversion be justified by a documented
guarantee that also covers this unsized wrapper and its reference metadata?
This is a request to clarify the safety argument,
not a claim of reproduced undefined behavior.
No candidate code was executed on a suspect layout in this audit.
~~~

[layout-rust]: https://doc.rust-lang.org/reference/type-layout.html#the-rust-representation
[layout-transparent]: https://doc.rust-lang.org/reference/type-layout.html#the-transparent-representation
[dst]: https://doc.rust-lang.org/reference/dynamically-sized-types.html#r-dynamic-sized.pointer-types
[transmute]: https://doc.rust-lang.org/std/mem/fn.transmute.html
[issue-four]: https://github.com/timothee-haudebourg/json-number/issues/4
