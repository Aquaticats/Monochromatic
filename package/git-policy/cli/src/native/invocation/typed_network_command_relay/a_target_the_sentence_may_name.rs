//! What:
//!  The host a remote URL names as the person typed it, and the words an `ssh` command needs to connect there.
//! Why:
//!  The sentence after a refusal names the host the person's remote names, because a `known_hosts` line is recorded under the
//!  name the connection asked about;
//!  and every command it offers is built from bytes a host name and a user may hold, so nothing it prints can be read by a
//!  shell as anything but a word
//!  (`doc/planning/cli-git-typed-network-command-next-step-at-a-new-host-key-design.md`, "Where the host comes from").
//!  Where a URL's host ends, whether a number after it is a port, and which bytes a host or a user may hold are read in
//!  `a_remote_url_s_words_a_sentence_may_print.rs`, by Git's own `connect.c` at v2.56.0, and this file assembles its answers into the
//!  one command a person can type.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const target = targetOf('ssh://user@host:2222/x'); // { host: 'host', connect: '-p 2222 user@host' }
//! ```

// What: The URL's own word readings: where a host ends, what a port is, and which bytes a host or a user may hold.
// Why:  Git reads those, and this file only assembles the command from them.
//
// In TS you'd write (pseudocode):
// ```ts
// import { beforeTheColonThatBeginsThePath, printableHost, printableUser, theHostAndAPort } from './a-remote-url-s-words-a-sentence-may-print';
// ```
use super::a_remote_url_s_words_a_sentence_may_print::{
    before_the_colon_that_begins_the_path,
    printable_host,
    printable_user,
    the_host_and_a_port,
};

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
    // What: `(user_and_host, a_port_may_follow)` is the text that names the user and the host, and whether a port may be
    //       written after that host.
    // Why:   Git reads the two URL forms differently: a URL with a scheme has an authority that ends at its first `/`,
    //        and a scp-like URL's host ends at its first `:`, where that URL's path begins.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const [userAndHost, aPortMayFollow] = url.includes('://')
    //   ? [url.slice(url.indexOf('://') + 3).split('/')[0], true]
    //   : [beforeTheColonThatBeginsThePath(url), false];
    // ```
    let (user_and_host, a_port_may_follow): (&str, bool) = match url.split_once("://") {
        Some((_scheme, after_the_scheme)) => {
            (after_the_scheme.split('/').next().unwrap_or(""), true)
        }
        None => (before_the_colon_that_begins_the_path(url), false),
    };
    // What: `(user_text, host_and_a_port)` splits at the last `@`, which is where a URL's user ends and its host begins.
    // Why:   ssh reads the last `@` as that boundary too, so a password holding one still leaves the host after it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const at = userAndHost.lastIndexOf('@');
    // const [userText, hostAndAPort] = at === -1 ? [undefined, userAndHost] : [userAndHost.slice(0, at), userAndHost.slice(at + 1)];
    // ```
    let (user_text, host_and_a_port): (Option<&str>, &str) = match user_and_host.rfind('@') {
        Some(at) => (
            user_and_host.get(..at),
            user_and_host.get(at.saturating_add(1)..).unwrap_or(""),
        ),
        None => (None, user_and_host),
    };
    // What: `(host_text, port_in_url)` is the host with a bracketed address unwrapped, and the number written after it.
    // Why:   Git hands ssh the address without its brackets, and one URL form writes no port at all.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const [hostText, portInUrl] = theHostAndAPort(hostAndAPort, aPortMayFollow);
    // ```
    let (host_text, port_in_url): (&str, Option<&str>) =
        the_host_and_a_port(host_and_a_port, a_port_may_follow);
    // What: `?` returns nothing from this function when the host is not a plain word a sentence may print.
    // Why:   Dropping a byte instead of refusing the host could rewrite `ho$(st)` into `host` and tell the person to connect to
    //        a machine the remote never named.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const host = printableHost(hostText); if (!host) return undefined;
    // ```
    let host: String = printable_host(host_text)?;
    // What: `filter` keeps the port only when it is a number, and `!= 22` is the default ssh port no option is needed for.
    // Why:   `-p` is written only where ssh would otherwise dial the wrong port; an empty tail or a word is dropped, so the
    //        sentence never offers `-p ` with nothing after it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const port = portInUrl && /^[0-9]{1,5}$/.test(portInUrl) && portInUrl !== '22' ? portInUrl : undefined;
    // ```
    let port: Option<&str> = port_in_url.filter(|text| {
        return !text.is_empty()
            && text.len() <= 5
            && text.bytes().all(|byte| return byte.is_ascii_digit())
            && *text != "22";
    });
    // What: `split_once(':')` cuts the user at the first colon, which is where a URL writes a password after the user name.
    // Why:   A password is never printed into a command: a command lands in a shell's history and in a terminal's scrollback,
    //        and the user name before the colon is all the sentence needs.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const userName = userText === undefined ? undefined : userText.split(':')[0];
    // ```
    let user_name: Option<&str> = match user_text {
        Some(named) => match named.split_once(':') {
            Some((name, _password)) => Some(name),
            None => Some(named),
        },
        None => None,
    };
    // What: `?` returns nothing when the user is not one word an `ssh` command may hold.
    // Why:   The user stands in the command's own text, so a byte a shell reads as syntax must refuse the whole target.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const user = userName === undefined ? undefined : printableUser(userName); if (userName !== undefined && !user) return undefined;
    // ```
    let user: Option<String> = match user_name {
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
