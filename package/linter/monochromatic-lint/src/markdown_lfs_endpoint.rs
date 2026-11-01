//! What:
//!  The one place an LFS endpoint becomes a credential-free object base URL.
//! Why:
//!  The rule recognizes existing object URLs by an exact string prefix of this base and builds
//! new ones by literal concatenation,
//!  so a changed serialization changes both findings and fixes.
//! This is the restricted normalizer selected in
//! `doc/planning/native-lfs-url-normalization-evaluation.md` (section `Selected owner`):
//! it accepts `http` and `https` endpoints written in a plain ASCII form,
//!  returns exactly what the
//! incumbent `lfsObjectBase` returns for them,
//!  and rejects every other input with a named reason.
//! It is not a URL parser and never returns a best-effort string.
//! `fixtures/lfs-url-parity.json` holds the measured incumbent outputs it is tested against.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const url = new URL(endpoint); url.username = url.password = url.search = url.hash = '';
//! // return url.href.endsWith('/') ? url.href.slice(0, -1) : url.href;
//! ```

/// What:
///  Why an endpoint is outside the supported form;
///  variants are listed in evaluation order.
/// Why:
///  The first applicable reason is part of the contract and is what the parity fixture records.
/// The message never contains the endpoint,
///  which may hold credentials.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type LfsUrlRejection = 'scheme' | 'backslash' | 'ipv6_literal' | /* ... */ 'dot_segment';
/// ```
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
    /// The host holds a byte other than an ASCII letter,
    ///  digit,
    ///  `-`,
    ///  `.` or `_`.
    HostCharacter,
    /// The host is longer than 253 bytes.
    HostLength,
    /// The host has an empty label:
    ///  a leading,
    ///  trailing or doubled full stop.
    EmptyHostLabel,
    /// The last host label is a number and the host is not a canonical dotted-decimal IPv4 address.
    NumericHost,
    /// The port holds a byte other than an ASCII digit.
    PortCharacter,
    /// The port is greater than 65535.
    PortRange,
    /// The path holds a byte outside the supported path set.
    PathCharacter,
    /// A path segment is a single-dot or double-dot segment,
    ///  including `%2e` spellings.
    DotSegment,
}

/// What:
///  The fixture identifier and the user-facing explanation of each rejection.
/// Why:
///  Tests compare the identifier with the measured fixture;
///  a person reading the diagnostic
/// needs what was wrong and how to write the endpoint instead.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// rejection.fixtureName(); rejection.explanation();
/// ```
impl LfsUrlRejection {
    /// Identifier used by the `rejection` field of `fixtures/lfs-url-parity.json`.
    pub fn fixture_name(self) -> &'static str {
        match self {
            LfsUrlRejection::Scheme => return "scheme",
            LfsUrlRejection::Backslash => return "backslash",
            LfsUrlRejection::Ipv6Literal => return "ipv6_literal",
            LfsUrlRejection::EmptyHost => return "empty_host",
            LfsUrlRejection::HostCharacter => return "host_character",
            LfsUrlRejection::HostLength => return "host_length",
            LfsUrlRejection::EmptyHostLabel => return "empty_host_label",
            LfsUrlRejection::NumericHost => return "numeric_host",
            LfsUrlRejection::PortCharacter => return "port_character",
            LfsUrlRejection::PortRange => return "port_range",
            LfsUrlRejection::PathCharacter => return "path_character",
            LfsUrlRejection::DotSegment => return "dot_segment",
        }
    }

    /// What the endpoint did,
    ///  then how to write it so this linter accepts it.
    pub fn explanation(self) -> &'static str {
        match self {
            LfsUrlRejection::Scheme => {
                return "it does not begin with http:// or https://. Write the endpoint as https://host/path, with no leading whitespace.";
            }
            LfsUrlRejection::Backslash => {
                return "it holds a reverse solidus before the first ? or #. Write / instead.";
            }
            LfsUrlRejection::Ipv6Literal => {
                return "its host is a bracketed IPv6 literal, which is not supported. Use a host name or an IPv4 address.";
            }
            LfsUrlRejection::EmptyHost => {
                return "its host is empty. Write exactly two slashes after the scheme, then the host.";
            }
            LfsUrlRejection::HostCharacter => {
                return "its host holds a character other than an ASCII letter, a digit, -, . or _. Write the host in its ASCII form; an internationalized host works in its xn-- form.";
            }
            LfsUrlRejection::HostLength => {
                return "its host is longer than 253 bytes. Use a shorter host name.";
            }
            LfsUrlRejection::EmptyHostLabel => {
                return "its host has a leading, trailing or doubled full stop. Remove the extra full stop.";
            }
            LfsUrlRejection::NumericHost => {
                return "its host ends in a number but is not a canonical IPv4 address. Write the address as four decimal parts without leading zeros.";
            }
            LfsUrlRejection::PortCharacter => {
                return "its port holds a character other than an ASCII digit. Write the port as digits only.";
            }
            LfsUrlRejection::PortRange => {
                return "its port is greater than 65535. Write a port from 0 to 65535.";
            }
            LfsUrlRejection::PathCharacter => {
                return "its path holds a character outside the supported set. Percent-encode the character in .lfsconfig; existing percent escapes pass through unchanged.";
            }
            LfsUrlRejection::DotSegment => {
                return "its path has a single-dot or double-dot segment. Write the resolved path.";
            }
        }
    }
}

/// Render the explanation through the ordinary error interface.
impl std::fmt::Display for LfsUrlRejection {
    /// Borrow the formatter only while writing the fixed explanation.
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return formatter.write_str(self.explanation());
    }
}

/// Mark the rejection as a standard error for application error handling.
impl std::error::Error for LfsUrlRejection {}

/// What:
///  The longest supported host,
///  in bytes.
/// Why:
///  253 bytes is the longest host name that fits DNS;
///  the incumbent returns longer hosts unchanged.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const MAX_HOST_BYTES = 253;
/// ```
const MAX_HOST_BYTES: usize = 253;

/// What:
///  The greatest port number.
/// Why:
///  A port is 16 bits.
///  `u32` (not `u16`) holds the value while one more digit is added,
///  so
/// the range check runs before anything can overflow.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const MAX_PORT = 65_535;
/// ```
const MAX_PORT: u32 = 65_535;

/// What:
///  Path bytes kept unchanged besides ASCII letters and digits.
/// Why:
///  These are the bytes the incumbent serializes without encoding or rewriting.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const PATH_PUNCTUATION = "-._~!$&'()*+,;=:@/%";
/// ```
const PATH_PUNCTUATION: &[u8] = b"-._~!$&'()*+,;=:@/%";

/// What:
///  Lowercase spellings of single-dot and double-dot path segments.
/// Why:
///  The incumbent resolves these,
///  including their percent-encoded forms;
///  this function rejects them instead.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const DOT_SEGMENTS = ['.', '..', '%2e', '.%2e', '%2e.', '%2e%2e'];
/// ```
const DOT_SEGMENTS: [&str; 6] = [".", "..", "%2e", ".%2e", "%2e.", "%2e%2e"];

/// Whether a byte may appear in a supported host.
fn is_host_byte(byte: u8) -> bool {
    return byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'.' || byte == b'_';
}

/// Whether a byte may appear in a supported path.
fn is_path_byte(byte: u8) -> bool {
    return byte.is_ascii_alphanumeric() || PATH_PUNCTUATION.contains(&byte);
}

/// Whether a byte is an ASCII decimal digit.
fn is_digit_byte(byte: u8) -> bool {
    return byte.is_ascii_digit();
}

/// Whether a byte is an ASCII hexadecimal digit.
fn is_hex_byte(byte: u8) -> bool {
    return byte.is_ascii_hexdigit();
}

/// What:
///  Whether every byte of the text satisfies a named predicate.
/// Why:
///  One explicit loop serves the host,
///  port,
///  path and number checks.
/// `fn(u8) -> bool` is a plain function pointer,
///  so no closure is needed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function allBytes(text: string, accept: (byte: number) => boolean): boolean;
/// ```
fn all_bytes(text: &str, accept: fn(u8) -> bool) -> bool {
    for byte in text.bytes() {
        if !accept(byte) {
            return false;
        }
    }
    return true;
}

/// Whether the text is one or more ASCII decimal digits.
fn is_decimal(text: &str) -> bool {
    return !text.is_empty() && all_bytes(text, is_digit_byte);
}

/// What:
///  Whether a host label is a number as the incumbent's host parser sees it.
/// Why:
///  A last label of decimal digits,
///  or `0x` followed by zero or more hexadecimal digits,
///  makes
/// the incumbent parse the whole host as an IPv4 address.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function isNumberLabel(label: string): boolean;
/// ```
fn is_number_label(label: &str) -> bool {
    if is_decimal(label) {
        return true;
    }
    if let Some(digits) = label.strip_prefix("0x") {
        return all_bytes(digits, is_hex_byte);
    }
    return false;
}

/// What:
///  Whether one dotted part is canonical:
///  one to three digits,
///  no leading zero,
///  at most 255.
/// Why:
///  Only canonical parts serialize unchanged in the incumbent.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function isCanonicalIpv4Part(part: string): boolean;
/// ```
fn is_canonical_ipv4_part(part: &str) -> bool {
    if !is_decimal(part) || part.len() > 3 || (part.len() > 1 && part.starts_with('0')) {
        return false;
    }
    // At most three digits, so the sum stays far below the u32 limit.
    let mut value: u32 = 0;
    for byte in part.bytes() {
        value = value * 10 + u32::from(byte - b'0');
    }
    return value <= 255;
}

/// Whether the host is exactly four canonical dotted-decimal parts.
fn is_canonical_ipv4(host: &str) -> bool {
    let mut parts: usize = 0;
    for part in host.split('.') {
        if !is_canonical_ipv4_part(part) {
            return false;
        }
        parts += 1;
    }
    return parts == 4;
}

/// Whether a path segment is a dot segment in any letter case of its `%2e` spellings.
fn is_dot_segment(segment: &str) -> bool {
    for dot in DOT_SEGMENTS {
        if segment.eq_ignore_ascii_case(dot) {
            return true;
        }
    }
    return false;
}

/// What:
///  The lowercase scheme and its default port when the endpoint begins with a supported scheme and `://`.
/// Why:
///  Scheme letters compare without case;
///  the output scheme is always lowercase.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function supportedScheme(endpoint: string): [scheme: string, defaultPort: number] | undefined;
/// ```
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

/// What:
///  The port to serialize:
///  `None` when absent,
///  empty,
///  or the scheme's default.
/// Why:
///  The incumbent drops a default port and leading zeros;
///  the bound is checked on every digit
/// so no integer overflows on a long digit string.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parsePort(digits: string, defaultPort: number): number | undefined;
/// ```
fn parse_port(digits: &str, default_port: u32) -> Result<Option<u32>, LfsUrlRejection> {
    if digits.is_empty() {
        return Ok(None);
    }
    if !is_decimal(digits) {
        return Err(LfsUrlRejection::PortCharacter);
    }
    let mut value: u32 = 0;
    for byte in digits.bytes() {
        value = value * 10 + u32::from(byte - b'0');
        if value > MAX_PORT {
            return Err(LfsUrlRejection::PortRange);
        }
    }
    if value == default_port {
        return Ok(None);
    }
    return Ok(Some(value));
}

/// What:
///  Validate and lowercase the host.
/// Why:
///  The checks run in the contract's order:
///  empty,
///  character set,
///  length,
///  empty label,
///  numeric form.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function supportedHost(rawHost: string): string; // throws the first applicable rejection
/// ```
fn supported_host(raw_host: &str) -> Result<String, LfsUrlRejection> {
    if raw_host.is_empty() {
        return Err(LfsUrlRejection::EmptyHost);
    }
    if !all_bytes(raw_host, is_host_byte) {
        return Err(LfsUrlRejection::HostCharacter);
    }
    if raw_host.len() > MAX_HOST_BYTES {
        return Err(LfsUrlRejection::HostLength);
    }
    let host: String = raw_host.to_ascii_lowercase();
    let mut last_label: &str = "";
    for label in host.split('.') {
        if label.is_empty() {
            return Err(LfsUrlRejection::EmptyHostLabel);
        }
        last_label = label;
    }
    if is_number_label(last_label) && !is_canonical_ipv4(host.as_str()) {
        return Err(LfsUrlRejection::NumericHost);
    }
    return Ok(host);
}

/// What:
///  Validate the path's bytes and segments.
/// Why:
///  `%` is kept as written whatever follows it;
///  escapes are neither validated,
///  decoded nor case-normalized.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function checkPath(path: string): void; // throws the first applicable rejection
/// ```
fn check_path(path: &str) -> Result<(), LfsUrlRejection> {
    if !all_bytes(path, is_path_byte) {
        return Err(LfsUrlRejection::PathCharacter);
    }
    for segment in path.split('/') {
        if is_dot_segment(segment) {
            return Err(LfsUrlRejection::DotSegment);
        }
    }
    return Ok(());
}

/// What:
///  Turn an `lfs.url` or `remote.<name>.lfsurl` value into the base that object URLs start with.
/// Why:
///  Whenever this returns `Ok`,
///  the string equals the incumbent `lfsObjectBase` output for the
/// same endpoint;
///  every other input returns the first applicable rejection.
///  Credentials,
///  query and
/// fragment are discarded without inspection,
///  and exactly one final slash is removed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function lfsObjectBase(endpoint: string): string; // throws LfsUrlRejection
/// ```
pub fn lfs_object_base(endpoint: &str) -> Result<String, LfsUrlRejection> {
    // Step 1: scheme and authority marker.
    let Some((scheme, default_port)): Option<(&'static str, u32)> = supported_scheme(endpoint)
    else {
        return Err(LfsUrlRejection::Scheme);
    };
    // The matched prefix is ASCII, so this index is a character boundary.
    let rest: &str = &endpoint[scheme.len() + 3..];
    // Step 2: drop the query and fragment, then refuse a reverse solidus in what remains.
    let hierarchy: &str = match rest.find(['?', '#']) {
        Some(at) => &rest[..at],
        None => rest,
    };
    if hierarchy.contains('\\') {
        return Err(LfsUrlRejection::Backslash);
    }
    // Step 3: the authority runs to the first slash; the path is the rest, including that slash.
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
    // Steps 5 to 7: host, port and path, in that order.
    let host: String = supported_host(raw_host)?;
    let port: Option<u32> = parse_port(raw_port, default_port)?;
    check_path(path)?;
    // Step 8: serialize, then remove exactly one final slash.
    let mut base: String = format!("{scheme}://{host}");
    if let Some(value) = port {
        base.push(':');
        base.push_str(value.to_string().as_str());
    }
    if path.is_empty() {
        base.push('/');
    } else {
        base.push_str(path);
    }
    if base.ends_with('/') {
        base.pop();
    }
    return Ok(base);
}

/// Measured-fixture and contract controls stay outside release artifacts.
#[cfg(test)]
#[path = "markdown_lfs_endpoint_tests.rs"]
mod tests;
