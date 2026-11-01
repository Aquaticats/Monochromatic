# TODO

## Probe ideas

### From W3C/CSSWG drafts and issues

**Sources to re-mine periodically:**

- `w3c/csswg-drafts` issues; active spec debates reveal where training data is thin
- `https://www.w3.org/TR/`:
  look for First Public Working Drafts;
  the gap between "discussed" and "implemented" is where models fail
- `drafts.csswg.org`: editor's drafts move faster than published WDs

**Candidates:**

`css-values-5 if()` transpiler
: The `if()` function uses `;`-separated branches with `media()`/`supports()`/`style()` conditions and `else`.
A transpiler to `@media`/`@supports`-wrapped rules would expose whether models understand
the `else` chaining semantics (negate all previous conditions) and the `style()` query type.
Status: editor's draft only, still has open edge-case issues (#13189 cycle detection).
Hold until spec stabilizes.

`css-mixins-1 @function` transpiler
: The official parameterized CSS custom function spec (FPWD May 2025, distinct from the `@apply`
probe already in the suite). Key edge cases: typed parameter defaults, local variable scoping
(locals shadow params, params shadow call-site custom props), nested function calls, `@media`
inside function bodies.
Status: FPWD but active issues (#13522 multiple `@result`, #13524 auto-wrap).
Good candidate once those are resolved.

`css-values-5 random()` with fixed seed
: The `random()` function has a `fixed <number>` mode that makes output deterministic, making it
testable. Edge cases: `<random-value-sharing>` semantics (per-element vs per-property caching),
`random-item()` with `{}` wrapped comma-containing values, interactions with inheritance.
Status: spec still actively debated (issues #13132, #13337 rename discussion open as of 2026-02).
Add probe once `<random-value-sharing>` naming and semantics are frozen.

`css-values-5 @when/@else` transpiler
: Rewrites `@when media(...) and supports(...)` / `@else` chains to equivalent nested
`@media`/`@supports` blocks. The key edge case: `@else` requires generating the negation
of all preceding conditions (complexity grows with chain length).
Status: Working Draft, October 2025.
Good candidate; simpler than `if()` but tests the same chaining logic.

---

### From IETF RFCs

**Sources to re-mine:**

- `https://www.rfc-editor.org/rfc/`: filter by date for recent publications
- `https://datatracker.ietf.org/`: search by area (Web) for upcoming RFCs

**Candidates:**

`RFC 9651 HTTP Structured Fields` parser + canonical serializer ⭐
: Published September 2024. Defines a typed header value format with Lists, Dictionaries, Items,
and Parameters. Parse from text, output canonical re-serialized form (or "ERROR" for must_fail).

Trip wires models consistently get wrong:

- Token (`sugar`) vs String (`"sugar"`): different allowed character sets, different semantics
- Boolean true omits the value in serialization: `a` not `a=?1`; false is `a=?0`
- `-0` parses to `0`, must re-serialize as `0`
- Leading zeros in integers/decimals: strip them (`042` → `42`)
- Decimal precision: max 3 decimal places, always at least 1 (`1.5` not `1.50`)
- Display String `%"..."`: percent-encoding uses only lowercase hex (`%c3%bc` not `%C3%BC`);
  unescaped non-ASCII and control chars are rejected
- Date type: `@<integer>`: must fail if a decimal is given (`@1659578233.12` is invalid)
- Byte sequence: `:base64:` with standard base64 padding

Canonical test suite: `github.com/httpwg/structured-field-tests` (JSON test vectors,
updated January 2026). Can drive the container check directly.

`RFC 8785 JCS` (JSON Canonicalization Scheme) ⭐
: Published 2020, but niche enough that implementations are sparse in training data.
Input: JSON object. Output: canonical UTF-8 bytes.

Trip wires:

- Key sort uses UTF-16 code unit order, not Unicode code points or locale; matters for keys
  containing supplementary characters (surrogate pairs sort differently)
- Number serialization must match ES6 `JSON.stringify` exactly: integers without decimal point
  up to 2^53, floats use minimum-precision representation (Grisu/Ryu)
- Recursive sort on nested objects
- `-0` must serialize as `0`

Reference test vectors: `github.com/cyberphone/json-canonicalization`

---

### From TC39

**Sources to re-mine:**

- `github.com/tc39/proposals`: Stage 2-3 proposals only; Stage 1 is too unstable
- Individual proposal repos for open issues revealing edge cases

**Candidates:**

`Temporal PlainDate arithmetic` (Stage 3) ⭐
: Input: lines of `YYYY-MM-DD + PnYnMnWnDTnHnMnS [constrain|reject]`.
Output: resulting date string or "ERROR".

Trip wires:

- Months are added before days (not equivalent to adding total days)
- `overflow: constrain` clamps to last day of month, not rolls over:
  `2024-01-31 + P1M` → `2024-02-29`, not `2024-03-01`
- `overflow: reject` throws if result would be out of range
- `2024-02-29 + P1Y` with `constrain` → `2025-02-28` (2025 not a leap year)
- Leap year detection (divisible by 4, except centuries, except 400-year centuries)

Models reach for `Date` or a polyfill, neither of which is available in the container.
The correct algorithm must be implemented from scratch.

Spec: `tc39/proposal-temporal`, well-defined at Stage 3. Polyfill exists
(`@js-temporal/polyfill`) but is not installable in the container.

---

### From WHATWG

**Sources to re-mine:**

- `github.com/whatwg/url`: open issues surface edge cases
- `github.com/web-platform-tests/wpt`: test vectors for URL, Encoding, Streams

**Candidates:**

`WHATWG URL IPv4 normalizer`
: Input: one URL per line. Output: normalized URL with IPv4 address resolved, or "INVALID".
Scope to IPv4 host normalization + path `.`/`..` removal only (full URL parser is too large).

Trip wires:

- Octal notation: `http://0177.0.0.1/` → `http://127.0.0.1/`
- Hex notation: `http://0x7f000001/` → `http://127.0.0.1/`
- Condensed notation: `http://127.1/` → `http://127.0.0.1/`
- Mixed bases per octet: `http://0xC0.0250.0.1/` → `http://192.168.0.1/`
- Path normalization: `http://example.com/a/b/../c/./d` → `http://example.com/a/c/d`

Test vectors: `wpt/url/resources/urltestdata.json`
Status: stable spec, but the IPv4 parser sub-spec is genuinely obscure; models know about
it but consistently get the mixed-base and condensed forms wrong.
