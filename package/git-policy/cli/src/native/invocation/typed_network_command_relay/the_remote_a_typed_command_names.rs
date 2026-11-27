//! What:
//!  Which typed command the person wrote,
//!  the remote URL Git would use for it,
//!  and whether ssh carries that URL.
//! Why:
//!  The decision reads the remote first and relays only a command ssh carries;
//!  every other command keeps the process replacement it has today
//!  (`doc/decision/cli-git-typed-network-command-relays-at-a-new-host-key.md`).
//!  The URL is read from Git itself,
//! so `url.<base>.insteadOf` rewrites are applied exactly as the command will apply them.
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
// What: The four command words, whose home is the option table each one selects.
// Why:  The index of this folder names the four through this module, so they are re-exported here.
//
// In TS you'd write (pseudocode):
// ```ts
// export type { TypedNetworkCommand } from './the-options-before-the-remote';
// ```
pub use super::the_options_before_the_remote::TypedNetworkCommand;

/// What:
///  The typed network command this argument list names,
/// its words after the command word,
/// and Git's global options before it.
/// Why:
///  One reading decides all three,
/// so a caller cannot pair one command's word with another's arguments;
///  the global options matter because `-C` moves the directory Git reads the repository in,
/// and the remote must be read in the
///  repository the command itself will read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// typedCommand(argv): { command: TypedNetworkCommand; region: string[]; prefix: string[] } | undefined
/// ```
pub fn typed_command(
    arguments: &[OsString]
) -> Option<(TypedNetworkCommand, &[OsString], &[OsString])> {
    // What: `(word, region)` is a pair: the subcommand and the words after it.
    // Why:   The global options and the command word are not the command's own arguments.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const found = commandTokens(argv); if (!found) return undefined;
    // ```
    let (word, region): (&OsString, &[OsString]) = command_tokens(arguments)?;
    // What: `prefix_len` is how many tokens of the caller's list are Git's global options, so the list before it is that prefix.
    // Why:   A read of the remote must run in the directory the command runs in, and `-C` is one of those options.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const prefix = argv.slice(0, globalLayout(argv).prefix_len);
    // ```
    let prefix_len: usize = crate::global_arguments::global_layout(arguments).prefix_len;
    let prefix: &[OsString] = arguments.get(..prefix_len).unwrap_or(&[]);
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
    return Some((command, region, prefix));
}

/// What:
///  Where the remote URL of a typed command is read from.
/// Why:
///  The reading names the caller's choices so one function can answer for all four:
///  a clone carries its URL in its own words and needs no repository,
///  and a fetch,
/// pull or push names a remote the repository knows,
/// or a URL of its own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RemoteRead = { operand: string } | { named: boolean };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RemoteRead<'a> {
    /// The command's own words carry the URL (a clone).
    Operand(&'a std::ffi::OsStr),
    /// The command names a remote,
    /// or names none and Git selects the branch's.
    NamedRemote(Option<&'a std::ffi::OsStr>),
}

/// What:
///  How the remote URL is read for a command:
/// a clone's own word,
/// or a remote name.
/// Why:
///  The caller states the command,
/// and this says which of Git's two questions to ask.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// command === 'clone' ? { operand: region[0] } : { named: region[0] }
/// ```
pub fn remote_read_for<'a>(
    command: TypedNetworkCommand,
    region: &'a [OsString],
) -> Option<RemoteRead<'a>> {
    let operand: Option<&'a OsString> = remote_operand(command, region);
    // What: `?` answers nothing when a clone names no URL, which is Git's own error to print.
    // Why:   A clone without a URL is not a command the wrapper can read a remote for.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (command === 'clone') return { operand: operand! };
    // ```
    if command == TypedNetworkCommand::Clone {
        let url: &'a OsString = operand?;
        return Some(RemoteRead::Operand(url.as_os_str()));
    }
    // What: `map` wraps the name when there is one and answers nothing when the command names no remote.
    // Why:   A command without an operand asks Git for the branch's own remote.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return { named: operand };
    // ```
    return Some(RemoteRead::NamedRemote(operand.map(OsString::as_os_str)));
}

/// What:
///  The URL Git will use,
/// asked of Git,
/// with every `insteadOf` rewrite applied.
/// Why:
///   A remote written `gh:served.git` becomes `ssh://target/work/served.git` through `url.<base>.insteadOf`,
///   and the transport that decides whether ssh carries the command is the rewritten one
///   (`doc/planning/cli-git-typed-network-command-next-step-at-a-new-host-key-design.md`,
/// "Where the host comes from").
///   A push asks for the push URL,
/// because a `pushurl` or a `pushInsteadOf` can send it elsewhere.
///   Nothing is read when Git refuses the question:
/// an unknown remote,
/// or no remote at all,
/// keeps today's behaviour.
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
pub fn remote_url(
    command: TypedNetworkCommand,
    read: &RemoteRead<'_>,
    prefix: &[OsString],
    real_git: &Path,
    overlay: &[(OsString, OsString)],
) -> Option<String> {
    // What: `Vec<(Vec<OsString>, bool)>` is a list of alternative questions, asked in order.
    // Why:   A named remote is read by `remote get-url`, which also answers a push URL,
    //        and a URL the person typed is read only by `ls-remote --get-url`;
    //        the second question is asked when Git refuses the first.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const questions: string[][] = [];
    // ```
    // What: `Option<&OsStr>` is the remote the command names, or nothing when Git selects the branch's own.
    // Why:   The same operand ends every question that names a remote.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const named = read.named ?? undefined;
    // ```
    let operand: Option<&std::ffi::OsStr> = match read {
        RemoteRead::Operand(url) => Some(*url),
        RemoteRead::NamedRemote(name) => *name,
    };
    let mut questions: Vec<Vec<OsString>> = Vec::new();
    if command == TypedNetworkCommand::Clone {
        questions.push(vec![
            OsString::from("ls-remote"),
            OsString::from("--get-url"),
        ]);
    } else {
        let mut by_name: Vec<OsString> = vec![OsString::from("remote"), OsString::from("get-url")];
        if command == TypedNetworkCommand::Push {
            // A push may go to another URL than a fetch, and the push URL is the one its transport turns on.
            by_name.push(OsString::from("--push"));
        }
        questions.push(by_name);
        questions.push(vec![
            OsString::from("ls-remote"),
            OsString::from("--get-url"),
        ]);
    }
    for mut words in questions {
        // What: `Some(word)` is the present variant of `Option`, pushed onto the question when the command names a remote.
        // Why:   A question without the operand asks for the default remote, which is what a command without one uses.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (operand) words.push(operand);
        // ```
        if let Some(word) = operand {
            words.push(word.to_os_string());
        }
        // What: `to_vec` copies Git's global options in front of the question.
        // Why:   `-C` and `--git-dir` name the repository the command reads, so the question must run there.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const asking = [...prefix, ...words];
        // ```
        let mut asking: Vec<OsString> = prefix.to_vec();
        asking.extend(words);
        if let Some(url) = ask_for_a_url(asking, real_git, overlay) {
            return Some(url);
        }
    }
    return None;
}

/// What:
///  Ask Git one question about the remote and answer the URL it printed,
/// or nothing when it refused.
/// Why:
///  The question reads configuration alone and connects to nothing,
/// and a refused question must keep today's behaviour instead
///  of a relay over a transport nobody read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function askForAUrl(asking: string[], git, overlay): string | undefined {
///   const out = runCaptured(git, asking, overlay, PLUMBING_STEP_CEILING);
///   return out.status === 0 && out.stdout.trim() ? out.stdout.trim() : undefined;
/// }
/// ```
fn ask_for_a_url(
    asking: Vec<OsString>,
    real_git: &Path,
    overlay: &[(OsString, OsString)],
) -> Option<String> {
    // What: The captured run starts Git with the wrapper's own options and keeps at most one step's output.
    // Why:   The read is the wrapper's own work and must never wait for a person.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const output = runCaptured(gitCommand(git, asking, overlay, 'wrapper'), PLUMBING_STEP_CEILING);
    // ```
    let output: std::process::Output =
        crate::captured_child_run_to_its_end::output_of_within_bound(
            &mut crate::forwarding::git_command(
                real_git,
                asking.as_slice(),
                overlay,
                crate::hooks_off::StartOwner::Wrapper,
            ),
            crate::wait_ceilings::PLUMBING_STEP_CEILING,
        )
        .ok()?;
    if !output.status.success() {
        return None;
    }
    // What: `from_utf8_lossy` reads the printed URL as text; a URL is bytes and the comparison is with ASCII schemes.
    // Why:   The answer is Git's, and only its own printing is read.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const printed = decoder.decode(output.stdout).trim();
    // ```
    let printed: std::borrow::Cow<'_, str> = String::from_utf8_lossy(output.stdout.as_slice());
    let trimmed: &str = printed.trim();
    if trimmed.is_empty() {
        return None;
    }
    return Some(trimmed.to_owned());
}

/// What:
///  Whether ssh carries a URL:
/// the schemes ssh speaks,
/// or Git's scp-like `[user@]host:path` form,
/// which is ssh too.
/// Why:
///   Only a command ssh carries is relayed,
/// and everything else keeps its process replacement.
///   An `https`,
/// `git`,
/// `file` or local-path remote must not get a pseudo-terminal or a pipe
///   (`doc/decision/cli-git-typed-network-command-relays-at-a-new-host-key.md`,
/// "Reading first leaves every command ssh does not
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
        return matches!(
            scheme.to_ascii_lowercase().as_str(),
            "ssh" | "git+ssh" | "ssh+git"
        );
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
    return where_the_colon_is > 0
        && slash.is_none_or(|where_the_slash_is| return where_the_colon_is < where_the_slash_is);
}

/// Pure readings are controlled in the test module.
#[cfg(test)]
#[path = "the_remote_a_typed_command_names_tests.rs"]
mod tests;
