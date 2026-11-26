//! What:
//!  The host a remote URL names as the person typed it, and the words an `ssh` command needs to connect there.
//! Why:
//!  The sentence after a refusal names the host the person's remote names, because a `known_hosts` line is recorded under the
//!  name the connection asked about;
//!  and every command it offers is built from bytes a host name and a user may hold, so nothing it prints can be read by a
//!  shell as anything but a word
//!  (`doc/planning/cli-git-typed-network-command-next-step-at-a-new-host-key-design.md`, "Where the host comes from").
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const target = targetOf('ssh://user@host:2222/x'); // { host: 'host', connect: '-p 2222 user@host' }
//! ```

/// What:
///  The host as the person's remote names it, and the words an `ssh` command needs to connect there.
/// Why:
///   The sentence names the host in backticks and offers an `ssh` command;
///   both are built from the URL's own grammar, and every byte a host name cannot hold is dropped,
///   so nothing the sentence offers can be read by a shell as anything but a word
///   (`doc/planning/cli-git-typed-network-command-next-step-at-a-new-host-key-design.md`, "Where the host comes from").
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Target = { host: string; connect: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Target {
    /// The host as the remote names it, with every byte a host name cannot hold dropped.
    pub host: String,
    /// The words after `ssh`: an optional `-p <port>`, then an optional `user@`, then the host.
    pub connect: String,
}

/// What:
///  The target a URL names, or nothing when it names no host this sentence may print.
/// Why:
///  A sentence that named an empty host would offer `ssh ` and help nobody,
///  and one that put a shell word into a command would be a command the wrapper must never print.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function targetOf(url: string): Target | undefined
/// ```
pub fn target_of(url: &str) -> Option<Target> {
    // What: `(authority, scp)` is either the part after `ssh://` up to the path, or the part before the colon of a scp-like URL.
    // Why:   Both forms name the host in the same place; only where it ends differs.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const authority = url.includes('://') ? url.slice(url.indexOf('://') + 3).split('/')[0] : url.split(':')[0];
    // ```
    let (authority, port_in_url): (&str, Option<&str>) = match url.split_once("://") {
        Some((_scheme, rest)) => {
            let up_to_path: &str = rest.split('/').next().unwrap_or("");
            let (host_part, port) = split_the_port(up_to_path);
            (host_part, port)
        }
        None => {
            // A scp-like URL ends its host at the colon and its path begins after it, so the slash is not the only end.
            let up_to_the_path: &str = url.split('/').next().unwrap_or("");
            let (host_part, port) = split_the_port(up_to_the_path);
            (host_part, port)
        }
    };
    let (user_part, host_part): (Option<&str>, &str) = match authority.rfind('@') {
        Some(at) => (
            authority.get(..at),
            authority.get(at.saturating_add(1)..).unwrap_or(""),
        ),
        None => (None, authority),
    };
    // What: `?` returns nothing from this function when the host is not a plain word a sentence may print.
    // Why:   Dropping a byte instead of refusing the host could rewrite `ho$(st)` into `host` and tell the person to connect to
    //        a machine the remote never named.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const host = printableHost(hostPart); if (!host) return undefined;
    // ```
    let host: String = printable_host(host_part)?;
    // What: `filter` keeps the port only when it is a number, and `!= 22` is the default ssh port no option is needed for.
    // Why:   `-p` is written only where ssh would otherwise dial the wrong port; a port that is not a number is dropped.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const port = portInUrl && /^[0-9]{1,5}$/.test(portInUrl) && Number(portInUrl) !== 22 ? portInUrl : undefined;
    // ```
    let port: Option<&str> = port_in_url.filter(|text| {
        return text.len() <= 5
            && text.bytes().all(|byte| return byte.is_ascii_digit())
            && *text != "22";
    });
    let user: Option<String> = match user_part {
        Some(named) => Some(printable_user(named)?),
        None => None,
    };
    // What: `format!` builds one owned `String`; the siblings `&str` and `String::new()` cannot hold a computed value.
    // Why:   `ssh` reads its options before the destination, so `-p <port>` comes first and `user@` next.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const connect = [port ? `-p ${port}` : '', user ? `${user}@` : '', host].filter(Boolean).join(' ');
    // ```
    let connect: String = match (port, user.as_deref()) {
        (Some(number), Some(named)) => format!("-p {number} {named}@{host}"),
        (Some(number), None) => format!("-p {number} {host}"),
        (None, Some(named)) => format!("{named}@{host}"),
        (None, None) => host.clone(),
    };
    // What: `Some(..)` is the present variant of `Option`, carrying the target the sentence fills.
    // Why:   The two slots of the sentence are read from this one value.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return { host, connect };
    // ```
    return Some(Target {
        host,
        connect,
    });
}

/// What:
///  Split a URL authority into its host and a port written after a colon.
/// Why:
///  A port is written `host:port` in a URL, and an IPv6 host is written `[::1]`, whose colons are its own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function splitThePort(authority: string): [string, string | undefined] {
///   if (authority.startsWith('[')) { const end = authority.indexOf(']'); return [authority.slice(0, end + 1), authority.slice(end + 2) || undefined]; }
///   const colon = authority.indexOf(':'); return colon === -1 ? [authority, undefined] : [authority.slice(0, colon), authority.slice(colon + 1)];
/// }
/// ```
fn split_the_port(authority: &str) -> (&str, Option<&str>) {
    if let Some(rest) = authority.strip_prefix('[') {
        let Some(close) = rest.find(']') else {
            return (authority, None);
        };
        let host: &str = authority
            .get(..close.saturating_add(2))
            .unwrap_or(authority);
        let after: &str = authority.get(close.saturating_add(2)..).unwrap_or("");
        return (host, after.strip_prefix(':'));
    }
    match authority.split_once(':') {
        Some((host, port)) => return (host, Some(port)),
        None => return (authority, None),
    }
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
fn printable_host(host: &str) -> Option<String> {
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
fn printable_user(user: &str) -> Option<String> {
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
