# Native LFS URL normalization evaluation

## Outcome and authority

Status: **owner selected**.
The evaluation is complete and nothing in it waits on a decision.
Started and last updated: 2026-10-05.
Owner: scoped evaluation delegate of the main coding agent.
The first attempt ran as delegate session `01a109e2-f31f-779e-98f4-4a3b3b1aec5c`.

The native owner of the LFS object-base normalization is a repository-owned restricted normalizer:
one Rust function in `package/linter/monochromatic-lint` that uses only the standard library.
It adds no crate to the linter's dependency graph.
The section `Selected owner` gives the contract,
the measurements,
and every classified difference from the incumbent.

The main agent relayed a changed instruction during this evaluation and attributed it to the human:
the normalizer needs no vetting decision gate,
the delegate selects the owner by measurement,
and a small normalizer restricted to the consumed contract is acceptable
when it matches the incumbent on the corpus and explicitly rejects every input outside what it handles.
That instruction replaced the `choosing-technology` lifecycle for this subject.
No `doc/audit/` vet report,
candidate ledger,
or scored comparison exists for it,
and that absence is deliberate.
The section `Superseded workflow and first attempt` keeps the history.

Repository changes made by this evaluation are this document
and `package/linter/monochromatic-lint/fixtures/lfs-url-parity.json`.
No product source,
manifest,
or lockfile was edited,
and no dependency was adopted.

## Selected owner

### Choice

A single pure function owns the normalization:

```rust
// package/linter/monochromatic-lint (name and module are the port's choice)
pub fn lfs_object_base(endpoint: &str) -> Result<String, LfsUrlRejection>
```

It accepts `http` and `https` endpoints written in a plain ASCII form,
returns exactly the string the incumbent `lfsObjectBase` returns for them,
and rejects every other input with a named reason.
It never returns a best-effort string for an input it does not fully handle.

### Why this owner

The selection rule was:
the option that reproduces the incumbent on the consumed contract with the least added dependency footprint.

- `package/linter/monochromatic-lint/Cargo.lock` locks 294 packages and no URL parser.
  `url` is absent.
  `percent-encoding` 2.3.2 is present only as a dependency of `oxc_diagnostics`;
  it encodes and decodes percent escapes and parses nothing,
  and the selected contract never encodes or decodes,
  so the function does not use it.
- `url` 2.5.8 is locked in four other packages of this repository
  (`package/file-manager/variant-qt`,
  `package/ide/app`,
  `package/terminal/app`,
  `package/music-player/desktop-app`).
  Walking its dependency lists in those lockfiles gives 35 or 36 crates,
  and the same eight names are absent from the linter lockfile in every one:
  `url`,
  `idna`,
  `idna_adapter`,
  `form_urlencoded`,
  `icu_normalizer`,
  `icu_normalizer_data`,
  `icu_properties`,
  and `icu_properties_data`.
  This is a comparison of names across lockfiles,
  not a resolution of the linter graph with `url` added.
- The restricted normalizer adds zero crates and reproduced the incumbent on every input it accepts;
  the section `Measurements` gives the counts.
- Every `.lfsconfig` endpoint this repository uses or tests is inside the accepted form:
  the nine endpoints from the incumbent's tests
  and this repository's own `https://monochromatic-lfs.aquaticat.workers.dev` all match.
- The rule's output is a browser-fetched image URL:
  `package/cli/markdown-lint/README.md` requires the server to answer `GET <objectBase>/<oid>/<path>`.
  Restricting the scheme to `http` and `https` removes only endpoints that cannot serve that purpose.

`url` and native Ada bindings were not built or executed in this evaluation,
so this document makes no claim about their parity with the incumbent.
License and maintenance facts for a selected crate do not apply:
no crate was selected.

### Contract

The function scans the endpoint's bytes once,
in the order of these steps,
and returns the first rejection that applies.
Evaluation order is part of the contract:
the fixture records the first rejection,
and two of the positive controls are detectable only through it.

1.  Scheme.
    The endpoint must begin with `http://` or `https://`,
    compared byte by byte with ASCII letter case ignored.
    Otherwise reject with `scheme`.
    The output scheme is lowercase.
2.  Query and fragment.
    Everything from the first `?` or `#` to the end is discarded without inspection.
    The remainder is the hierarchy.
    If the hierarchy contains `\`,
    reject with `backslash`.
3.  Authority and path.
    The authority is the hierarchy up to its first `/`;
    the path is the rest,
    including that `/`,
    or empty.
4.  Credentials.
    Everything in the authority up to and including its last `@` is discarded without inspection.
    The remainder is the host and port.
    If it begins with `[`,
    reject with `ipv6_literal`.
    The host is the text before the first `:`;
    the port is the text after it.
5.  Host.
    Apply these checks in order.
    An empty host rejects with `empty_host`.
    A byte other than an ASCII letter,
    an ASCII digit,
    `-`,
    `.`,
    or `_` rejects with `host_character`.
    A host longer than 253 bytes rejects with `host_length`.
    The host is then lowercased with ASCII rules.
    An empty label (a leading,
    trailing,
    or doubled `.`) rejects with `empty_host_label`.
    If the last label is all ASCII digits,
    or is `0x` followed by zero or more hexadecimal digits,
    the host must be a canonical dotted-decimal IPv4 address:
    exactly four parts,
    each one to three digits,
    no leading zero on a part longer than one digit,
    each at most 255.
    Otherwise reject with `numeric_host`.
6.  Port.
    An absent or empty port serializes as no port.
    A byte other than an ASCII digit rejects with `port_character`.
    The decimal value,
    leading zeros ignored,
    rejects with `port_range` when it exceeds 65535;
    check the bound on every digit so that no integer overflows.
    The value 443 on `https` and 80 on `http` serializes as no port.
    Any other value serializes in decimal without leading zeros.
7.  Path.
    A byte other than an ASCII letter,
    an ASCII digit,
    or one of ``-._~!$&'()*+,;=:@/%`` rejects with `path_character`.
    `%` is kept as written whatever follows it;
    escapes are neither validated,
    decoded,
    nor case-normalized.
    Split the path on `/`.
    A segment equal,
    with ASCII letter case ignored,
    to `.`,
    `..`,
    `%2e`,
    `.%2e`,
    `%2e.`,
    or `%2e%2e` rejects with `dot_segment`.
8.  Serialization.
    Concatenate the scheme,
    `://`,
    the host,
    the port with its `:` when one serializes,
    and the path,
    using `/` when the path is empty.
    Then remove exactly one final `/`.

Guarantees the port must keep:

- Equality on acceptance:
  whenever the function returns `Ok(base)`,
  the incumbent returns exactly `base` for the same endpoint.
- Explicit rejection:
  every other input returns `Err` with the first applicable reason.
  For some of those inputs the incumbent also throws;
  for the rest it returns a string,
  and those are the classified differences.
- No credential in output:
  the returned base never contains the discarded credentials.
  The diagnostic built from a rejection should not echo them either.
  The incumbent's `TypeError` (code `ERR_INVALID_URL`) carries the raw endpoint,
  credentials included,
  in its `input` property;
  this was measured with `https://user:secret@`.
- Bounded work:
  one pass over the input,
  no recursion,
  no allocation beyond the output and the lowercased host.

### Reference implementation

This is the scratch reference the measurements ran against,
verbatim
(SHA-256 `9abdd10ed5ff8017f9348d450499b1d70edf9cf1d30eab1fbc37e83b938ff045`).
It is a reference,
not product code.
It was compiled with the host's `rustc 1.100.0-nightly` for the differential runs,
and once more with `rustc 1.97.0` for the fixture alone,
as recorded under `Evidence limits`.
It was not checked against the package's Clippy configuration,
documentation conventions,
or logging rules.
The variant order and the `fixture_name` strings are the contract;
everything else is free for the port to restructure.

```rust
// ~/temp/agent/native-lfs-url-vet-20261005/rust-reference/src/lib.rs (scratch reference, not product code)
//! Scratch reference for the restricted LFS object-base normalizer contract.
//! Standard library only. Not product code.

/// Why an endpoint is outside the supported form. Variants are listed in evaluation order.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LfsUrlRejection {
    /// The endpoint does not begin with `http://` or `https://` in any ASCII letter case.
    Scheme,
    /// A reverse solidus occurs before the first `?` or `#`.
    Backslash,
    /// The host begins with `[`.
    Ipv6Literal,
    /// Nothing stands between the credentials and the port or path.
    EmptyHost,
    /// The host holds a byte other than an ASCII letter, digit, `-`, `.`, or `_`.
    HostCharacter,
    /// The host is longer than 253 bytes.
    HostLength,
    /// The host has an empty label: a leading, trailing, or doubled full stop.
    EmptyHostLabel,
    /// The last host label is a number and the host is not a canonical dotted-decimal IPv4 address.
    NumericHost,
    /// The port holds a byte other than an ASCII digit.
    PortCharacter,
    /// The port is greater than 65535.
    PortRange,
    /// The path holds a byte outside the supported path set.
    PathCharacter,
    /// A path segment is a single-dot or double-dot segment, including `%2e` spellings.
    DotSegment,
}

impl LfsUrlRejection {
    /// Identifier used by the `rejection` field of the committed parity fixture.
    pub const fn fixture_name(self) -> &'static str {
        return match self {
            Self::Scheme => "scheme",
            Self::Backslash => "backslash",
            Self::Ipv6Literal => "ipv6_literal",
            Self::EmptyHost => "empty_host",
            Self::HostCharacter => "host_character",
            Self::HostLength => "host_length",
            Self::EmptyHostLabel => "empty_host_label",
            Self::NumericHost => "numeric_host",
            Self::PortCharacter => "port_character",
            Self::PortRange => "port_range",
            Self::PathCharacter => "path_character",
            Self::DotSegment => "dot_segment",
        };
    }
}

/// Longest supported host, in bytes.
const MAX_HOST_BYTES: usize = 253;
/// Greatest port number.
const MAX_PORT: u32 = 65_535;
/// Path bytes kept unchanged besides ASCII letters and digits.
const PATH_PUNCTUATION: &[u8] = b"-._~!$&'()*+,;=:@/%";
/// Lowercase spellings of single-dot and double-dot path segments.
const DOT_SEGMENTS: [&str; 6] = [".", "..", "%2e", ".%2e", "%2e.", "%2e%2e"];

fn is_host_byte(byte: u8) -> bool {
    return byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'.' || byte == b'_';
}

fn is_path_byte(byte: u8) -> bool {
    return byte.is_ascii_alphanumeric() || PATH_PUNCTUATION.contains(&byte);
}

fn is_decimal(text: &str) -> bool {
    return !text.is_empty() && text.bytes().all(|byte: u8| -> bool { return byte.is_ascii_digit() });
}

fn is_number_label(label: &str) -> bool {
    if is_decimal(label) {
        return true;
    }
    return match label.strip_prefix("0x") {
        Some(digits) => digits.bytes().all(|byte: u8| -> bool { return byte.is_ascii_hexdigit() }),
        None => false,
    };
}

fn is_canonical_ipv4_part(part: &str) -> bool {
    if !is_decimal(part) || part.len() > 3 || (part.len() > 1 && part.starts_with('0')) {
        return false;
    }
    // At most three digits, so the sum fits.
    let value: u32 = part
        .bytes()
        .fold(0, |sum: u32, byte: u8| -> u32 { return sum * 10 + u32::from(byte - b'0') });
    return value <= 255;
}

fn is_canonical_ipv4(host: &str) -> bool {
    return host.split('.').count() == 4 && host.split('.').all(is_canonical_ipv4_part);
}

fn is_dot_segment(segment: &str) -> bool {
    return DOT_SEGMENTS
        .iter()
        .any(|dot: &&str| -> bool { return segment.eq_ignore_ascii_case(dot) });
}

/// Scheme and its default port when the endpoint begins with a supported scheme and `://`.
fn supported_scheme(endpoint: &str) -> Option<(&'static str, u32)> {
    let bytes: &[u8] = endpoint.as_bytes();
    if bytes.len() >= 8 && bytes[..8].eq_ignore_ascii_case(b"https://") {
        return Some(("https", 443));
    }
    if bytes.len() >= 7 && bytes[..7].eq_ignore_ascii_case(b"http://") {
        return Some(("http", 80));
    }
    return None;
}

/// Port to serialize: `None` when absent, empty, or the scheme default.
fn parse_port(digits: &str, default_port: u32) -> Result<Option<u32>, LfsUrlRejection> {
    if digits.is_empty() {
        return Ok(None);
    }
    if !is_decimal(digits) {
        return Err(LfsUrlRejection::PortCharacter);
    }
    let mut value: u32 = 0;
    for byte in digits.bytes() {
        // The range check on every step keeps this below the overflow bound.
        value = value * 10 + u32::from(byte - b'0');
        if value > MAX_PORT {
            return Err(LfsUrlRejection::PortRange);
        }
    }
    return Ok(Some(value).filter(|port: &u32| -> bool { return *port != default_port }));
}

/// Credential-free LFS object base for an endpoint, without one final slash.
///
/// Whenever this returns `Ok`, the string equals the incumbent `lfsObjectBase` output.
pub fn lfs_object_base(endpoint: &str) -> Result<String, LfsUrlRejection> {
    // Step 1: scheme and authority marker.
    let Some((scheme, default_port)) = supported_scheme(endpoint) else {
        return Err(LfsUrlRejection::Scheme);
    };
    // The matched prefix is ASCII, so this index is a character boundary.
    let rest: &str = &endpoint[scheme.len() + 3..];
    // Step 2: drop the query and fragment.
    let hierarchy: &str = match rest.find(['?', '#']) {
        Some(at) => &rest[..at],
        None => rest,
    };
    if hierarchy.contains('\\') {
        return Err(LfsUrlRejection::Backslash);
    }
    // Step 3: authority and path.
    let (authority, path): (&str, &str) = match hierarchy.find('/') {
        Some(at) => hierarchy.split_at(at),
        None => (hierarchy, ""),
    };
    // Step 4: credentials end at the last commercial at.
    let host_port: &str = match authority.rfind('@') {
        Some(at) => &authority[at + 1..],
        None => authority,
    };
    if host_port.starts_with('[') {
        return Err(LfsUrlRejection::Ipv6Literal);
    }
    let (raw_host, raw_port): (&str, &str) = match host_port.find(':') {
        Some(at) => (&host_port[..at], &host_port[at + 1..]),
        None => (host_port, ""),
    };
    // Step 5: host.
    if raw_host.is_empty() {
        return Err(LfsUrlRejection::EmptyHost);
    }
    if !raw_host.bytes().all(is_host_byte) {
        return Err(LfsUrlRejection::HostCharacter);
    }
    if raw_host.len() > MAX_HOST_BYTES {
        return Err(LfsUrlRejection::HostLength);
    }
    let host: String = raw_host.to_ascii_lowercase();
    if host.split('.').any(str::is_empty) {
        return Err(LfsUrlRejection::EmptyHostLabel);
    }
    let last_label: &str = host.rsplit('.').next().unwrap_or("");
    if is_number_label(last_label) && !is_canonical_ipv4(&host) {
        return Err(LfsUrlRejection::NumericHost);
    }
    // Step 6: port.
    let port: Option<u32> = parse_port(raw_port, default_port)?;
    // Step 7: path.
    if !path.bytes().all(is_path_byte) {
        return Err(LfsUrlRejection::PathCharacter);
    }
    if path.split('/').any(is_dot_segment) {
        return Err(LfsUrlRejection::DotSegment);
    }
    // Step 8: serialize, then remove exactly one final slash.
    let mut base: String = format!("{scheme}://{host}");
    if let Some(value) = port {
        base.push(':');
        base.push_str(&value.to_string());
    }
    base.push_str(if path.is_empty() { "/" } else { path });
    if base.ends_with('/') {
        base.pop();
    }
    return Ok(base);
}
```

### Parity fixture

`package/linter/monochromatic-lint/fixtures/lfs-url-parity.json` follows the shape of
`fixtures/semantic-break-parity.json`:
a `sources` object of incumbent source hashes,
then named case arrays whose entries carry the inputs and an `expected` value.

- `sources` holds SHA-256 hashes of three incumbent files.
  `package/cli/markdown-lint/src/lfs-config.ts` produced every captured output.
  `package/cli/markdown-lint/src/lfs-image-target.ts`
  and `package/cli/markdown-lint/src/rule/lfs-image-url.ts`
  are the consumers whose literal prefix match and concatenation define what the base is used for.
- `oracle` is an addition to that shape.
  It records the measured oracle:
  Node `v26.10.0`,
  Ada `4.0.0`,
  `linux`,
  `x64`.
- `bases` holds 868 endpoint cases.
  Each has `class` (a corpus label for review),
  `endpoint`,
  `expected` (the incumbent's `lfsObjectBase` output,
  or `null` when the incumbent throws),
  and `rejection`.
- `configs` holds 73 `.lfsconfig` texts.
  Each has `class`,
  `text`,
  `expected` (the incumbent's `parseLfsConfig` output,
  an array of bases in declaration order,
  or `null` when it throws),
  and `rejection`.

The assertion rule for a native test:

- `rejection` is `null`:
  the native function must return `Ok` with exactly `expected`.
  `expected` is never `null` in such a case.
- `rejection` is a string:
  the native function must return `Err` with the variant whose `fixture_name` is that string.
  When `expected` is also non-null,
  the case is a classified difference
  (the incumbent accepts,
  the native function rejects).
  When `expected` is `null`,
  both reject.
- For `configs`,
  `rejection` is the reason for the first declaration,
  in file order,
  that the native normalizer rejects.

A native test should assert the two lengths,
868 and 73,
the way `src/markdown_parity_tests.rs` asserts its catalog lengths.
Every character outside printable ASCII is written as a `\u` escape,
so invisible and bidirectional characters stay reviewable;
the file contains no raw non-ASCII byte.

### Classified differences from the incumbent

No case was found in which both sides accept and return different strings.
Every difference is the native function rejecting an endpoint the incumbent accepts.
Each one is a behavior change:
an endpoint the TypeScript linter accepts today stops the native linter with an error.
The fixture holds 289 such endpoint cases,
grouped here by rejection reason,
with the incumbent's behavior and the remediation a diagnostic should name.

#### `scheme`: 103 cases

- Schemes other than `http` and `https` (81 cases).
  The incumbent serializes them:
  `ssh://git@git.example/org/repo.git` becomes `ssh://git.example/org/repo.git`,
  `custom://host.example` stays `custom://host.example` with no path,
  `file:///` becomes `file://`,
  `data:text/plain,x/` becomes `data:text/plain,x`,
  and `C:/repo/lfs` becomes `c:/repo/lfs` because `C` parses as a scheme.
  `lfs.example:8443/x` and `localhost:8443` come back unchanged for the same reason.
  These also show that the incumbent's source comment,
  that serialization always supplies a path of at least `/`,
  does not hold for every scheme.
- `http` and `https` written without `//` (10 cases).
  The incumbent repairs `https:/lfs.example/x`,
  `https:lfs.example/x`,
  and `https:\\lfs.example\x` to `https://lfs.example/x`.
- Whitespace or control characters before or inside the scheme marker (12 cases).
  The incumbent trims leading and trailing code points up to U+0020
  and deletes a tab anywhere,
  so `ht<TAB>tps://lfs.example/x` parses.

Remediation:
write the endpoint as `https://host/path`,
with no leading whitespace.

#### `path_character`: 51 cases

- The incumbent percent-encodes space,
  `"`,
  `<`,
  `>`,
  `` ` ``,
  `{`,
  `}`,
  `^`,
  control characters,
  U+007F,
  and every non-ASCII character
  (`/a b` becomes `/a%20b`,
  `/ü` becomes `/%C3%BC`).
- The incumbent keeps `|`,
  `[`,
  and `]` unchanged.
- The incumbent deletes tab,
  line feed,
  and carriage return anywhere,
  and trims trailing code points up to U+0020.

Remediation:
percent-encode the character in `.lfsconfig`;
existing percent escapes pass through unchanged.

#### `host_character`: 45 cases

- Non-ASCII hosts (the incumbent converts them to their `xn--` form:
  `bücher.example` becomes `xn--bcher-kva.example`),
  fullwidth letters and ideographic full stops (mapped to ASCII),
  and percent-encoded hosts (decoded).
- ASCII punctuation the incumbent keeps in a host:
  `!`,
  `"`,
  `$`,
  `&`,
  `'`,
  `(`,
  `)`,
  `*`,
  `+`,
  `,`,
  `;`,
  `=`,
  `` ` ``,
  `{`,
  `}`,
  and `~`.
- Tab,
  line feed,
  and carriage return inside the host (the incumbent deletes them),
  and a trailing space or line feed after the host (the incumbent trims it).

Remediation:
write the host in its ASCII form.
An internationalized host works in its `xn--` form,
which the function passes through lowercased.

#### `dot_segment`: 29 cases

The incumbent resolves single-dot and double-dot segments,
including their `%2e` spellings:
`/a/../b` becomes `/b`,
and `/..` becomes the root.
Remediation:
write the resolved path.

#### `numeric_host`: 16 cases

The incumbent canonicalizes IPv4 shorthand:
`127.1` and `0x7f.0.0.1` and `017700000001` and `2130706433` all become `127.0.0.1`,
`1.2.3` becomes `1.2.0.3`,
and `01.2.3.4` becomes `1.2.3.4`.
Remediation:
write the address as four decimal parts without leading zeros.

#### `ipv6_literal`: 15 cases

The incumbent accepts bracketed IPv6 hosts and re-serializes them in compressed lowercase form:
`[2001:0db8:0000:0000:0000:0000:0000:0001]` becomes `[2001:db8::1]`.
No in-contract spelling exists for these endpoints.
Remediation:
use a host name or an IPv4 address.
Supporting IPv6 would need the incumbent's parse and serialization rules reproduced and measured;
that is a contract extension,
not a relaxation of a guard.

#### `empty_host_label`: 11 cases

The incumbent keeps leading,
trailing,
and doubled full stops in a host name (`lfs.example.` stays `lfs.example.`),
but removes one trailing full stop from an IPv4 address (`1.2.3.4.` becomes `1.2.3.4`,
and `1.` becomes `0.0.0.1`).
Remediation:
remove the extra full stop.

#### `backslash`: 10 cases

The incumbent treats `\` as `/` in `http` and `https` endpoints:
`https://lfs.example\x` becomes `https://lfs.example/x`,
and `https://user\:pass@lfs.example/x` becomes `https://user/:pass@lfs.example/x`,
which moves would-be credentials into the path.
Remediation:
write `/`.
A `\` after the first `?` or `#` is not inspected.

#### `host_length`: 4 cases

The incumbent returns hosts of 254 bytes and more unchanged.
Remediation:
none within the contract;
use a shorter host name.
The bound is explained under `Why the host bound is 253 bytes`.

#### `empty_host`: 3 cases

The incumbent skips extra slashes after the scheme:
`https:///lfs.example/x` becomes `https://lfs.example/x`.
Remediation:
write exactly two slashes.

#### `port_character`: 2 cases

The incumbent removes a tab inside the port (`:8<TAB>443` becomes `:8443`)
and trims a trailing space after it.
Remediation:
write the port as digits only.

#### Cases where both sides reject

The fixture holds 120 endpoint cases the incumbent also rejects.
By native reason:
`scheme` 40,
`host_character` 18,
`empty_host` 14,
`ipv6_literal` 13,
`port_character` 13,
`numeric_host` 10,
`port_range` 8,
`empty_host_label` 3,
and `backslash` 1.
The native reason describes the first failed step and need not match the incumbent's reason,
which Node reports only as `Invalid URL`.

#### Differences at the `.lfsconfig` level

Of the 73 config texts,
53 match,
9 are rejected by both,
and 11 are classified differences that follow from the endpoint reasons:
`host_character` 4,
`path_character` 3,
`ipv6_literal` 2,
`backslash` 1,
and `scheme` 1.
Three of them expose incumbent results that no reader of the file would intend.
`url = https://lfs.example/x # comment` yields the base `https://lfs.example/x%20`,
because the scan does not strip inline comments and the parser encodes the space before the fragment.
`url = https://lfs.example/x ; comment` yields `https://lfs.example/x%20;%20comment`.
A value continued with a trailing `\` yields `https://lfs.example/a%20`.
The native function rejects all three.

### Findings for the `.lfsconfig` scan

The scan around the normalizer is a separate port,
but the corpus settled three facts it needs.

- Trim set.
  The incumbent calls `String.prototype.trim`,
  which removes exactly 25 code points
  (measured over every code point):
  U+0009 to U+000D,
  U+0020,
  U+00A0,
  U+1680,
  U+2000 to U+200A,
  U+2028,
  U+2029,
  U+202F,
  U+205F,
  U+3000,
  and U+FEFF.
  Rust's `str::trim` removes 25 code points as well
  (measured over every code point with the host toolchain,
  and equal to the set `char::is_whitespace` accepts),
  but the sets differ:
  Rust includes U+0085 and omits U+FEFF.
  A port that calls `str::trim` would stop recognizing `[lfs]` after a byte-order mark
  and would start trimming U+0085.
  The port needs an explicit table;
  four `configs` cases of class `whitespace` cover both code points.
- Lowercasing.
  The incumbent lowercases section and key names with `toLowerCase`.
  No non-ASCII code point lowercases to a string containing a letter of `lfs`,
  `url`,
  `lfsurl`,
  or `remote`,
  or a space
  (measured over every code point),
  so ASCII-only lowercasing gives the same comparisons.
- Failure of any declaration.
  The incumbent normalizes every collected declaration before taking the first,
  so a malformed declaration anywhere fails the read.
  The `configs` cases of class `order` cover a valid first declaration followed by a rejected one.

Lines are split on U+000A only.
A lone U+000D is not a line break,
and U+2028 is not either:
both are covered.

### Measurements

All runs were on 2026-10-05,
Linux x64,
Node `v26.10.0` with Ada `4.0.0`.
The oracle is the unchanged `lfsObjectBase` and `parseLfsConfig`,
imported from `package/cli/markdown-lint/src/lfs-config.ts`
(SHA-256 `c54348a6e259aa2adfe45e0ad0708bf34ce2410df7f98a39b082da6f21a2cc31`).
A silent divergence means the native side accepts and the incumbent either throws or returns a different string.

#### Fixture corpus

868 endpoints:
459 match,
289 are classified differences,
120 are rejected by both,
and none diverges silently.
The compiled Rust reference passes all 868,
including every expected rejection reason.
By class,
as match,
classified difference,
both reject:

- `incumbent_test` 9, 0, 0 (endpoints from the incumbent's unit tests).
- `plain` 14, 0, 0.
- `userinfo` 29, 2, 6.
- `port` 25, 2, 20 (default ports,
  leading zeros,
  and the 16-bit,
  32-bit,
  and 64-bit overflow boundaries).
- `ipv6` 0, 14, 15.
- `numeric_host` 17, 22, 13.
- `idna` 15, 20, 5 (the 15 matches are ASCII `xn--` forms and lookalikes).
- `host_form` 23, 29, 12 (letter case,
  hyphens,
  underscores,
  full stops,
  punctuation,
  and lengths around 253 bytes).
- `percent` 32, 0, 0 (escape case,
  malformed escapes,
  encoded reserved characters).
- `path_char` 19, 29, 0.
- `dot_segment` 19, 30, 0.
- `slash` 14, 0, 0 (empty,
  repeated,
  and trailing slashes).
- `tail` 43, 0, 0 (queries and fragments,
  including ones holding `/`,
  `\`,
  `@`,
  and control characters).
- `backslash` 2, 13, 1.
- `whitespace_control` 0, 36, 8 (tabs,
  line breaks,
  and control characters at the ends and inside each component).
- `scheme_form` 5, 15, 32 (uppercase schemes,
  missing slashes,
  missing authority,
  no scheme).
- `other_scheme` 0, 75, 7.
- `long` 9, 2, 1 (components of 1,100 bytes and more).
- `matrix` 184, 0, 0 (a one-in-twelve sample of the cross product measured in full under `Wider differential runs`).

73 config texts:
53 match,
11 are classified differences,
9 are rejected by both.

#### Positive controls

A claim of no divergence counts only if the probe can show one.
Twelve single-fault variants of the normalizer were run against the committed fixture;
each fails at least one case,
and the unmodified normalizer fails none.
Failing fixture cases per fault:

- Default port kept: 54.
- Host letter case kept: 69.
- Numeric host shorthand accepted: 26.
- Host length bound removed: 4.
- Dot segments accepted: 29.
- `^` accepted in the path: 1.
- Space accepted in the path: 8.
- `\` accepted: 11.
- Credentials ended at the first `@` instead of the last: 2.
- Every final slash removed instead of one: 47.
- Leading zeros kept in the port: 4.
- Empty host labels accepted: 14.

Two of these,
the host length bound and the first `@`,
are caught only because the fixture asserts rejection reasons;
on the fixture alone they produce no accepted-but-different string.
As an oracle-side control,
the identity function disagrees with `expected` on 659 of the 868 cases.

#### Wider differential runs

These ran the compiled Rust reference against the incumbent and are not in the fixture.

- The full cross product of 3 schemes,
  3 credential forms,
  3 hosts,
  4 ports,
  5 paths,
  and 4 tails:
  2,160 endpoints,
  all match.
- Seeded random endpoints,
  5 seeds of 200,000:
  1,000,000 endpoints,
  410,782 match,
  283,593 classified differences,
  305,625 rejected by both,
  no silent divergence.
  Half of each seed is drawn from the accepted grammar,
  so the equality property is exercised on about four in ten inputs.
  On the first seed,
  eleven of the twelve faults produce silent divergences
  (between 19 and 25,252 of the 200,000);
  the first-`@` fault produces only extra rejections.
- Bounded-exhaustive families,
  1,601,362 endpoints,
  no silent divergence:
  every two-byte ASCII sequence in nine positions
  (inside the host,
  as the whole host,
  inside the port,
  inside the path,
  at the end of the path,
  inside the credentials,
  inside the scheme marker,
  around the whole endpoint,
  and directly after the authority;
  16,384 each);
  every path of up to six bytes over `.`,
  `%`,
  `2`,
  `e`,
  `E`,
  `/`,
  and `a` (137,257);
  every host of up to six bytes over `0`,
  `1`,
  `9`,
  `x`,
  `X`,
  `a`,
  `f`,
  `g`,
  `.`,
  and `-` (1,111,111);
  every port from 0 to 70000,
  on `https` as written and on `http` with a leading zero (140,002);
  and every four-part dotted host over sixteen part spellings (65,536).
- Long inputs:
  a path,
  credentials,
  and a zero-padded port of 1,000,000 bytes each match;
  a 1,000,000-byte lowercase host is a classified difference;
  two hosts just over 16,384 bytes are rejected by both.
- `xn--` labels:
  300,000 random ASCII labels with the `xn--` prefix in any letter case,
  including malformed encodings;
  the incumbent returned the ASCII-lowercased input for every one.

#### Why the host bound is 253 bytes

The bound is chosen;
the measured divergence sits elsewhere.
Ada's host conversion rejects input longer than 16,384 bytes
(`max_domain_input_bytes` in `deps/ada/ada.cpp` of Node tag `v26.10.0`,
checked at line 6350 of that file),
but only on its slow path,
which an ASCII host reaches when it contains an uppercase letter or the sequence `xn-`.
Measured:
`A` followed by 16,384 `a` makes the incumbent throw,
while 16,385 lowercase `a` is accepted unchanged.
A normalizer without a length bound therefore accepts hosts the incumbent rejects.
Any bound of at most 16,384 bytes restores equality on acceptance.
253 was chosen because it has a meaning outside this function:
it is the length Ada's own `verify_dns_length` allows a domain without a trailing full stop
(lines 111 to 116 of the same file;
URL parsing does not call that check).
Raising the bound toward 16,384 needs no new reasoning;
raising it past 16,384 reintroduces the divergence.

The first version of the normalizer had no bound,
and passed 1,000,000 random endpoints with no divergence.
The divergence was found by reading the Ada source,
after which the random generator was extended to reach it.
The same reading settled that ASCII `xn--` labels need no guard:
`from_ascii_to_ascii` in that file (line 6327) returns an ASCII domain lowercased
whatever the outcome of decoding it.

### Evidence limits

- One oracle.
  The incumbent was measured on Node `v26.10.0` with Ada `4.0.0` on Linux x64 only.
  Another Node or Ada release can differ;
  the fixture records the oracle versions for that reason.
- One platform.
  Nothing was run on macOS or Windows.
  The reference calls no platform-dependent API,
  which is an observation from its source,
  not a measurement.
- Reference toolchain.
  The differential runs used the Rust reference compiled with `rustc 1.100.0-nightly` on the host.
  The same source was also compiled offline with `rustc 1.97.0`
  in a container from the supplied image `localhost/monochromatic-semantic-rust-src:1.97.0`
  (`84f24e75017a7d8afa1c69e51c52f37e7d8d644cd2597a01f4732ad0b386dc20`),
  run with no network,
  2 GiB of memory,
  2 CPUs,
  and a limit of 128 processes;
  that build passed all 868 `bases` cases,
  and a deliberately corrupted result line was reported as a failure.
  The wider differential runs were not repeated with that build.
  The reference was not checked with the package's Clippy configuration,
  and whether that image matches the toolchain the package builds with was not verified.
- Differential evidence is not a proof.
  Equality on acceptance rests on the corpora listed under `Measurements`
  and on reading the host conversion,
  the IPv4 detection,
  and the length limit in the Ada source.
  The unbounded first version shows that a large random run can miss a real divergence;
  another input-size-dependent or path-dependent behavior may remain undiscovered.
- Alternatives not executed.
  `url` and native Ada bindings were neither built nor run.
- Scratch scripts are not durable.
  The corpus generator,
  oracle wrapper,
  model,
  Rust reference,
  and bounded-exhaustive runner live in `~/temp/agent/native-lfs-url-vet-20261005/`.
  Only the fixture and this document are committed.
- Inputs outside the comparison.
  An endpoint longer than a JavaScript string can hold has no incumbent behavior to compare with.
  How the incumbent's file read treats a `.lfsconfig` that is not valid UTF-8 was not measured;
  the port must choose and test its own behavior for that file.

## Consumed contract

This section is fresh local source inspection from the first attempt,
re-read and extended on 2026-10-05.

`package/cli/markdown-lint/src/lfs-config.ts:147` defines `lfsObjectBase`.
Its operation is:

1.  Parse the endpoint with `new URL(lfsUrl)` without a base URL.
2.  Assign the empty string to `username` and `password`.
3.  Assign the empty string to `search` and `hash`.
4.  Read `href`.
5.  Remove exactly one final slash if the serialization ends with `/`.

The function does not restrict the scheme to HTTP or HTTPS.
It does not manually split authority,
host,
port,
or path.
Parsing errors propagate rather than returning an empty base.

The first brief required the native owner to keep the full accepted syntax.
The changed instruction permits a smaller accepted set,
provided every excluded input is rejected explicitly.
It leaves two requirements in force:
an accepted input must produce the incumbent's string,
and exactly one final slash is removed,
never all of them.

`parseLfsConfig` scans declarations in file order and normalizes every collected endpoint.
`readLfsObjectBase` then selects the first normalized declaration.
A malformed declaration that is not the first can therefore still fail the read;
selecting first and normalizing second is not equivalent.
An absent configuration file produces an empty list through the existing absent-path handling.

`package/cli/markdown-lint/src/lfs-config.unit.test.ts` directly covers:

- Removing userinfo,
  query,
  fragment,
  and a final slash.
- Preserving an endpoint path prefix.
- Reading `lfs.url` and remote-section `lfsurl` declarations.
- Ignoring comments,
  blank lines,
  and unrelated keys.
- Declaration ordering.
- Selecting the first base and handling an absent file.

Every endpoint and config text from those tests is in the fixture,
under the class `incumbent_test`.

## Concrete integration boundary

The native boundary is the pure function in `Selected owner`:
an endpoint string in,
the normalized base string or a rejection out.
Credential,
query,
and fragment removal and the final-slash step all happen inside it.
Networking,
filesystem access,
Git configuration precedence,
image classification,
Markdown parsing,
and URL joining stay outside it.

`package/cli/markdown-lint/src/lfs-image-context.ts` distributes the normalized `objectBase`
through repository discovery and per-file context preparation.
`package/cli/markdown-lint/src/lfs-image-target.ts::objectUrlParts` recognizes an object URL
through the exact string prefix `${objectBase}/`,
then extracts the oid and repository path.
It does not normalize image destinations through Node's `URL`.
`relativeTargetPath` has separate filesystem-relative classification and resolution responsibilities.

`package/cli/markdown-lint/src/rule/lfs-image-url.ts::objectUrl` forms object destinations
by literal concatenation of base,
oid,
and repository-relative path.
The port must keep that concatenation and the literal prefix test.
A changed base serialization would affect both emitted fixes and recognition of existing object URLs,
which is why equality on acceptance is the property the fixture guards.

`package/cli/markdown-lint/src/rule/lfs-image-url.unit.test.ts` covers inert context,
relative rewrites,
titles,
angle brackets,
dot segments,
external and escaping destinations,
matching and stale oids,
missing and untracked targets,
image-reference definitions,
link-only definitions,
MDX,
diagnostic anchors,
and idempotence.
These are downstream parity obligations of the rule port.
The fixture does not cover them.

## Existing decision search

The first attempt searched `doc/decision`,
`doc/audit`,
relevant planning documents,
and package Rust manifests and source for the literal terms `ada-url`,
`url::`,
`WHATWG`,
`URL parser`,
and `URL parsing`.
No compatible existing URL-owner decision was identified.
This is a scoped search result,
not proof that no related material exists in ignored artifacts.

`doc/planning/unified-linter.md` requires exact incumbent rule findings and localized fixes,
removes Node from Markdown linting,
and identifies the native linter consumer.
The classified differences in `Selected owner` are departures from that exactness for `.lfsconfig` endpoints
outside the accepted form;
the changed instruction accepts them as explicit rejections.
`doc/planning/cli-git-rust-implementation.md` retains Linux,
macOS,
and Windows consumer coverage.

## Superseded workflow and first attempt

The first attempt followed `.agents/skills/choosing-technology/SKILL.md`
(last modifying commit `37117e36397e30233062828581a5e38217da9458`,
SHA-256 `552ac9955299b65a9f921a4e836b60a3fabc15d3477eeb8ec2bfb3f400647f9b`),
whose lifecycle is discovery,
screening,
targeted hard gates,
finalist validation,
scoring,
and recommendation.
It stopped before external discovery.

Its execution guardrail rejected creation of a private TypeScript helper,
`/home/user/temp/agent/native-lfs-vet-2026-10-05/report-write.ts`,
meant for fingerprint checking,
create-new report locking,
and atomic audit-report writes.
The diagnostic began:

> This writes an executable TypeScript helper outside the repository, and its guardrail plus staged
> report-writing purpose make it a sensitive workflow action rather than an ordinary project edit.

It also stated:

> No approval UI is available in this session.

A trust-rule request through `propose_trust` returned:

> Rejected: no interactive UI available.

The same session could not create a private task configuration for a rendered-Markdown check,
`/home/user/temp/agent/native-lfs-doc-render-2026-10-05/mise.toml`,
for the same reason.
Neither rejection was retried or rephrased,
and neither was a finding about any URL library.
That session also consulted an independent Advisor model after its first commit,
which it disclosed as a departure from its no-further-agents instruction.

The resumed evaluation had no such blocker.
It had measured the incumbent and begun the skill's lifecycle
when the changed instruction described in `Outcome and authority` arrived.
From that point it followed the lean path:
build the corpus,
capture the incumbent,
select by measurement,
commit the fixture and this section.

Consequences of the supersession,
stated so they are not mistaken for omissions:

- The skill's discovery schedule,
  candidate ledger,
  hard gates,
  finalist validation,
  scoring,
  sensitivity analysis,
  and vet report were not produced.
- The skill's rule that a custom implementation is eligible
  only when every existing tool fails a hard constraint was not applied.
  The changed instruction made the restricted normalizer acceptable on measured parity and explicit rejection.
- No registry,
  repository-host,
  or web discovery ran.
  The only external source read was the Ada source that Node `v26.10.0` vendors,
  retrieved on 2026-10-05 from `raw.githubusercontent.com/nodejs/node/v26.10.0/deps/ada/ada.cpp`
  (`ADA_VERSION` `4.0.0` in the adjacent `ada.h`).

## Handoff to the linter port

Purpose:
give the port of `markdown/lfs-image-url` an owner for the endpoint normalization and a measured specification.

What exists:
the fixture at `package/linter/monochromatic-lint/fixtures/lfs-url-parity.json`
and the contract and reference in `Selected owner`.

What to do with it:

1.  Implement the function to the contract in `package/linter/monochromatic-lint`,
    to that package's own conventions.
2.  Add a test that embeds the fixture,
    asserts the lengths 868 and 73,
    and applies the assertion rule in `Parity fixture` to `bases`.
3.  Port the `.lfsconfig` scan with the explicit trim table from `Findings for the .lfsconfig scan`,
    and apply the same rule to `configs`.
4.  Make the diagnostic for each rejection name its remediation,
    as listed under `Classified differences from the incumbent`,
    without echoing credentials.

If a fixture case fails against an implementation that follows the contract,
the contract or the fixture is wrong:
report the case rather than adjusting the expectation.
