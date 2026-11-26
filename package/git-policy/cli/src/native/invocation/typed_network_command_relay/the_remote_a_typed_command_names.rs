//! What:
//!  Which typed command the person wrote, the remote URL Git would use for it, and whether ssh carries that URL.
//! Why:
//!  The decision reads the remote first and relays only a command ssh carries;
//!  every other command keeps the process replacement it has today
//!  (`doc/decision/cli-git-typed-network-command-relays-at-a-new-host-key.md`).
//!  The URL is read from Git itself, so `url.<base>.insteadOf` rewrites are applied exactly as the command will apply them.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const command = typedCommand(argv); const url = remoteUrl(command, git, argv); if (sshCarries(url)) relay();
//! ```

// What: `OsString` is owned operating-system text of raw bytes (its sibling `String` must be UTF-8).
// Why:  The person's words are never decoded or re-encoded on their way to Git.
//
// In TS you'd write (pseudocode):
// ```ts
// // process.argv entries, byte-preserving.
// ```
use std::ffi::OsString;
// What: `Path` is a borrowed filesystem path of raw bytes.
use std::path::Path;
// What: The first word after Git's global options, and the words after it.
use crate::global_arguments::command_tokens;
// What: The reading that finds the remote among those words, with Git's own option table.
// Why:  A value of an option is not the remote, and only that table says which token is which.
//
// In TS you'd write (pseudocode):
// ```ts
// import { remoteOperand } from './the-options-before-the-remote';
// ```
use super::the_options_before_the_remote::remote_operand;

/// What:
///  A typed network command whose remote may be carried by ssh.
/// Why:
///  These are the four the decision covers; a command outside this list keeps the process replacement it has today.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type TypedNetworkCommand = 'clone' | 'fetch' | 'pull' | 'push';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TypedNetworkCommand {
    /// `git clone`.
    Clone,
    /// `git fetch`.
    Fetch,
    /// `git pull`.
    Pull,
    /// `git push`.
    Push,
}

/// What:
///  The typed network command this argument list names, and its words after the command word.
/// Why:
///  One reading decides both, so a caller cannot pair one command's word with another's arguments.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// typedCommand(argv): { command: TypedNetworkCommand; region: string[] } | undefined
/// ```
pub fn typed_command(arguments: &[OsString]) -> Option<(TypedNetworkCommand, &[OsString])> {
    // What: `(word, region)` is a pair: the subcommand and the words after it.
    // Why:   The global options and the command word are not the command's own arguments.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const found = commandTokens(argv); if (!found) return undefined;
    // ```
    let Some((word, region)) = command_tokens(arguments) else {
        return None;
    };
    // What: `as_encoded_bytes` reads the word as its raw bytes without decoding them.
    // Why:   The comparison is with Git's own command words, which are ASCII.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // switch (Buffer.from(word).toString()) { ... }
    // ```
    let command: TypedNetworkCommand = match word.as_encoded_bytes() {
        b"clone" => TypedNetworkCommand::Clone,
        b"fetch" => TypedNetworkCommand::Fetch,
        b"pull" => TypedNetworkCommand::Pull,
        b"push" => TypedNetworkCommand::Push,
        _other => return None,
    };
    return Some((command, region));
}

/// What:
///  Where the remote URL of a typed command is read from.
/// Why:
///  The reading names the caller's choices so one function can answer for all four:
///  a clone carries its URL in its own words and needs no repository,
///  and a fetch, pull or push names a remote the repository knows, or a URL of its own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RemoteRead = { operand: string } | { named: boolean };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RemoteRead<'a> {
    /// The command's own words carry the URL (a clone).
    Operand(&'a std::ffi::OsStr),
    /// The command names a remote, or names none and Git selects the branch's.
    NamedRemote(Option<&'a std::ffi::OsStr>),
}

/// What:
///  How the remote URL is read for a command: a clone's own word, or a remote name.
/// Why:
///  The caller states the command, and this says which of Git's two questions to ask.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// command === 'clone' ? { operand: region[0] } : { named: region[0] }
/// ```
pub fn remote_read_for<'a>(command: TypedNetworkCommand, region: &'a [OsString]) -> Option<RemoteRead<'a>> {
    let operand: Option<&'a OsString> = remote_operand(command, region);
    return match command {
        // A clone with no URL is Git's own error to print, and nothing is relayed.
        TypedNetworkCommand::Clone => operand.map(|url| RemoteRead::Operand(url.as_os_str())),
        TypedNetworkCommand::Fetch | TypedNetworkCommand::Pull | TypedNetworkCommand::Push => {
            Some(RemoteRead::NamedRemote(operand.map(OsString::as_os_str)))
        }
    };
}

/// What:
///  The URL Git will use, asked of Git, with every `insteadOf` rewrite applied.
/// Why:
///   A remote written `gh:served.git` becomes `ssh://target/work/served.git` through `url.<base>.insteadOf`,
///   and the transport that decides whether ssh carries the command is the rewritten one
///   (`doc/planning/cli-git-typed-network-command-next-step-at-a-new-host-key-design.md`, "Where the host comes from").
///   A push asks for the push URL, because a `pushurl` or a `pushInsteadOf` can send it elsewhere.
///   Nothing is read when Git refuses the question: an unknown remote, or no remote at all, keeps today's behaviour.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function remoteUrl(command, read, git, overlay): string | undefined {
///   const args = command === 'clone' ? ['ls-remote', '--get-url', operand]
///     : command === 'push' ? ['remote', 'get-url', '--push', ...(operand ? [operand] : [])]
///     : ['remote', 'get-url', ...(operand ? [operand] : [])];
///   const out = runGitCaptured(git, args, overlay);
///   return out.status === 0 ? out.stdout.trim() : undefined;
/// }
/// ```
pub fn remote_url(command: TypedNetworkCommand, read: &RemoteRead<'_>, real_git: &Path, overlay: &[(OsString, OsString)]) -> Option<String> {
    // What: `Vec::new()` is an empty growable list; its siblings are `&[OsString]` and `[OsString; N]`.
    // Why:   The question's words are built per command and end with the operand when there is one.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const asking: string[] = [];
    // ```
    let mut asking: Vec<OsString> = Vec::new();
    let operand: Option<&std::ffi::OsStr> = match read {
        RemoteRead::Operand(url) => {
            asking.push(OsString::from("ls-remote"));
            asking.push(OsString::from("--get-url"));
            Some(*url)
        }
        RemoteRead::NamedRemote(name) => {
            asking.push(OsString::from("remote"));
            asking.push(OsString::from("get-url"));
            if command == TypedNetworkCommand::Push {
                // A push may go to another URL than a fetch, and the push URL is the one its transport turns on.
                asking.push(OsString::from("--push"));
            }
            // What: `&OsStr` is a borrow of an operating-system string; `copied` copies the reference out of the `Option`.
            // Why:   The name is pushed into the question below without being taken from the caller.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // const name: string | undefined = read.named;
            // ```
            *name
        }
    };
    if let Some(word) = operand {
        asking.push(word.to_os_string());
    }
    // What: The captured run answers the question without connecting: `remote get-url` and `ls-remote --get-url` read
    //       configuration alone.
    // Why:   The answer decides whether this process relays at all, so it must be Git's own answer.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const output = runCaptured(git, asking, overlay, PLUMBING_STEP_CEILING);
    // ```
    let output: std::process::Output = crate::captured_child_run_to_its_end::output_of_within_bound(
        &mut crate::forwarding::git_command(real_git, asking.as_slice(), overlay, crate::hooks_off::StartOwner::Wrapper),
        crate::wait_ceilings::PLUMBING_STEP_CEILING,
    )
    .ok()?;
    if !output.status.success() {
        return None;
    }
    // What: `from_utf8` reads the printed URL as text and answers nothing when the bytes are not UTF-8.
    // Why:   A URL with a byte no text holds cannot be compared with the schemes ssh carries.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let url: string; try { url = decoder.decode(output.stdout).trim(); } catch { return undefined; }
    // ```
    let printed: std::borrow::Cow<'_, str> = String::from_utf8_lossy(output.stdout.as_slice());
    let trimmed: &str = printed.trim();
    if trimmed.is_empty() {
        return None;
    }
    return Some(trimmed.to_owned());
}

/// What:
///  Whether ssh carries a URL: the schemes ssh speaks, or Git's scp-like `[user@]host:path` form, which is ssh too.
/// Why:
///   Only a command ssh carries is relayed, and everything else keeps its process replacement.
///   An `https`, `git`, `file` or local-path remote must not get a pseudo-terminal or a pipe
///   (`doc/decision/cli-git-typed-network-command-relays-at-a-new-host-key.md`, "Reading first leaves every command ssh does not
///   carry exactly as it is today").
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function sshCarries(url: string): boolean {
///   const scheme = /^([a-z][a-z0-9+.-]*):\/\//i.exec(url)?.[1];
///   if (scheme) return ['ssh', 'git+ssh', 'ssh+git'].includes(scheme.toLowerCase());
///   return url.indexOf(':') > 0 && (url.indexOf('/') === -1 || url.indexOf(':') < url.indexOf('/'));
/// }
/// ```
pub fn ssh_carries(url: &str) -> bool {
    // What: `split_once` splits at the first `://` and answers both halves, or nothing when it is not there.
    // Why:   A URL with a scheme names its transport there, and no scp-like form has one.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const parts = url.split('://');
    // ```
    if let Some((scheme, _rest)) = url.split_once("://") {
        return matches!(scheme.to_ascii_lowercase().as_str(), "ssh" | "git+ssh" | "ssh+git");
    }
    // What: `rfind('@')` finds the last user separator, so a user with an `@` is still one user.
    // Why:   Git reads the part after the last `@` as the host.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const authority = url.slice(url.lastIndexOf('@') + 1);
    // ```
    let authority: &str = match url.rfind('@') {
        Some(at) => url.get(at.saturating_add(1)..).unwrap_or(""),
        None => url,
    };
    // What: `find` answers the byte position of one character, or nothing.
    // Why:   Git's own reading of a scp-like URL is a colon that comes before any slash.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const colon = authority.indexOf(':'); const slash = authority.indexOf('/');
    // return colon > 0 && (slash === -1 || colon < slash);
    // ```
    let colon: Option<usize> = authority.find(':');
    let slash: Option<usize> = authority.find('/');
    let Some(where_the_colon_is) = colon else {
        return false;
    };
    return where_the_colon_is > 0 && slash.is_none_or(|where_the_slash_is| return where_the_colon_is < where_the_slash_is);
}

/// Pure readings are controlled in the test module.
#[cfg(test)]
#[path = "the_remote_a_typed_command_names_tests.rs"]
mod tests;
