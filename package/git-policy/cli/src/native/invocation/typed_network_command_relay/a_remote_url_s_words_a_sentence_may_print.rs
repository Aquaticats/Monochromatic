//! What:
//!  The words of a remote URL a sentence may print: where a URL's host ends, whether a number after it is a port, and the
//!  one byte rule a host name and a user name each have to keep.
//! Why:
//!  The sentence after a refusal names the host the person's remote names and offers an `ssh` command built from it
//!  (`doc/planning/cli-git-typed-network-command-next-step-at-a-new-host-key-design.md`, "Where the host comes from"),
//!  and every one of these readings follows Git's own `connect.c` at v2.56.0, the release `MINIMUM_GIT_VERSION` pins,
//!  so the command offered reaches the host and the port Git's own child reaches.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // beforeTheColonThatBeginsThePath('git@host:/srv/repo.git') -> 'git@host'
//! // theHostAndAPort('[2001:db8::1]:2200', true) -> ['2001:db8::1', '2200']
//! // printableHost('host'), printableUser('user') -> the text, or undefined
//! ```

/// What:
///  The part of a scp-like URL that names its user and host, which ends at the colon that begins its path.
/// Why:
///   Git ends a scp-like URL's host at its first colon, so a number after that colon is the path's first segment and never a
///   port, and an absolute path after it names no port at all.
///   A bracketed address holds colons of its own, so the colon that ends the host is the first one after its `]`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function beforeTheColonThatBeginsThePath(url: string): string {
///   const open = url.indexOf('['); const close = open === -1 ? -1 : url.indexOf(']', open);
///   const from = close === -1 ? 0 : close + 1; const colon = url.indexOf(':', from);
///   return colon === -1 ? url : url.slice(0, colon);
/// }
/// ```
pub(super) fn before_the_colon_that_begins_the_path(url: &str) -> &str {
    // What: `after_the_address` is the byte after a bracketed address's `]`, and zero when the URL holds no bracketed
    //       address.
    // Why:   Only a colon after that byte can begin the path; the address's own colons are before it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let afterTheAddress = 0;
    // ```
    let mut after_the_address: usize = 0;
    if let Some(open) = url.find('[') {
        // What: `and_then` runs the search for `]` only when the URL holds an opening bracket, and `Some(close)` is the
        //       present variant of its answer.
        // Why:   An address's own colons are only skipped when its bracket is closed.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const close = open === -1 ? undefined : url.indexOf(']', open);
        // ```
        if let Some(close) = url.get(open..).and_then(|rest| return rest.find(']')) {
            after_the_address = open.saturating_add(close).saturating_add(1);
        }
    }
    // What: `find(':')` answers the first colon at or after the address, and `get(..)` keeps everything before it.
    // Why:   That colon is the one Git reads as the end of the host.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const offset = url.indexOf(':', afterTheAddress);
    // return offset === -1 ? url : url.slice(0, afterTheAddress + offset);
    // ```
    match url
        .get(after_the_address..)
        .and_then(|tail| return tail.find(':'))
    {
        Some(offset) => {
            return url
                .get(..after_the_address.saturating_add(offset))
                .unwrap_or(url);
        }
        None => return url,
    }
}

/// What:
///  Split the text after a URL's user into the host and a port written after it, unwrapping a bracketed address.
/// Why:
///   A port is written `host:port`, a bracketed address is written `[::1]`, and Git hands ssh the address without its
///   brackets, so the sentence names and offers the address Git's child gets.
///   A scp-like URL writes no port after its host at all.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function theHostAndAPort(text: string, aPortMayFollow: boolean): [string, string | undefined] {
///   if (text.startsWith('[')) { const close = text.indexOf(']'); if (close === -1) return [text, undefined];
///     const after = text.slice(close + 1); return [text.slice(1, close), aPortMayFollow && after.startsWith(':') ? after.slice(1) : undefined]; }
///   const colon = text.indexOf(':'); if (colon === -1) return [text, undefined];
///   return [text.slice(0, colon), aPortMayFollow ? text.slice(colon + 1) : undefined];
/// }
/// ```
pub(super) fn the_host_and_a_port(
    text: &str,
    a_port_may_follow: bool,
) -> (&str, Option<&str>) {
    // What: `strip_prefix('[')` answers what follows an opening bracket, or nothing when the text has no bracket.
    // Why:   A bracketed address's colons are its own, so a port is read only after its closing bracket.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const inside = text.startsWith('[') ? text.slice(1) : undefined;
    // ```
    if let Some(inside) = text.strip_prefix('[') {
        let Some(close) = inside.find(']') else {
            // An address whose bracket never closes names no host this sentence may print, and the host check refuses the
            //  bracket the text still holds.
            return (text, None);
        };
        let address: &str = inside.get(..close).unwrap_or("");
        // What: `after_the_address` is what follows the `]`, which is empty or a colon and a port.
        // Why:   Only a URL with a scheme writes a port here; a scp-like URL's colon begins the path.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const afterTheAddress = inside.slice(close + 1);
        // ```
        let after_the_address: &str = inside.get(close.saturating_add(1)..).unwrap_or("");
        // What: `port` is the number after that colon, and nothing unless a port may follow this host.
        // Why:   A scp-like URL's colon begins its path, so a number there is the path's own first segment.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // let port = undefined;
        // ```
        let mut port: Option<&str> = None;
        if a_port_may_follow {
            port = after_the_address.strip_prefix(':');
        }
        return (address, port);
    }
    // What: `split_once(':')` answers both halves around the first colon, or nothing when the text has none.
    // Why:   That colon ends the host; what follows it is a port only in the form that writes one.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const parts = text.split(':'); const host = parts[0]; const afterTheColon = parts[1];
    // ```
    let Some((host, after_the_colon)) = text.split_once(':') else {
        return (text, None);
    };
    // What: `port` is the text after the host's colon, and nothing for the form that writes no port.
    // Why:   The scp-like form's colon begins its path, so what follows the host there is not a port.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let port = undefined;
    // ```
    let mut port: Option<&str> = None;
    if a_port_may_follow {
        port = Some(after_the_colon);
    }
    return (host, port);
}

/// What:
///  The host name a person may be shown and a shell reads as one word, or nothing when it holds any other byte.
/// Why:
///  A host name holds letters, digits, dots, hyphens and, for an address, colons and percent signs;
///  any other byte means this is not a host a sentence may name,
///  and dropping it instead would print a different host than the remote named.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// /^[A-Za-z0-9._:%-]+$/.test(host) ? host : undefined
/// ```
pub(super) fn printable_host(host: &str) -> Option<String> {
    if host.is_empty() {
        return None;
    }
    if !host.chars().all(|character| {
        return character.is_ascii_alphanumeric()
            || matches!(character, '.' | '_' | ':' | '%' | '-');
    }) {
        return None;
    }
    // What: `to_owned` copies the text into an owned `String`, which outlives the URL it was read from; the sibling `&str` would
    //       borrow it.
    // Why:   The target outlives the read of the URL.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return host;
    // ```
    return Some(host.to_owned());
}

/// What:
///  The user a URL may name, or nothing when it holds any byte a word of an `ssh` command cannot.
/// Why:
///  A user is one word of that command, so the same rule keeps the sentence free of shell syntax.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// /^[A-Za-z0-9._-]+$/.test(user) ? user : undefined
/// ```
pub(super) fn printable_user(user: &str) -> Option<String> {
    if user.is_empty() {
        return None;
    }
    if !user.chars().all(|character| {
        return character.is_ascii_alphanumeric() || matches!(character, '.' | '_' | '-');
    }) {
        return None;
    }
    return Some(user.to_owned());
}
