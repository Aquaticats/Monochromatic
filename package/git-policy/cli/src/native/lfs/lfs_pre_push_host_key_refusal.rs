//! What:
//!  The refusal a transcript names when this machine does not know the remote's SSH host key,
//! and the lines of a transcript that are the started program's own.
//! Why:
//!  OpenSSH prints that same last line for every refusal it makes over a host key,
//!  and a changed key,
//!  a revoked key and an unknown one must not be answered with the same advice;
//!  the reading is therefore about the words an unknown key leaves,
//!  and the sentence the wrapper prints names the two ways to add the key it does not touch.
//! The push's failure record and the upload's explanation share the sentence,
//! so the two faces cannot say different things about one refusal,
//! and these readings live beside the rest of the module's texts so that each file passes the
//! length limit.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const own = ownLinesOf(text); const unknown = readAsUnknownHostKey(own);
//! ```
/// What:
///  The lines of a transcript that the program the wrapper started wrote itself,
///  without the lines Git relayed from the far end.
/// Why:
///   Git prefixes every line the other end writes with `remote: `,
///   and with a terminal and a real `TERM` it writes an erase-line escape before that prefix;
///   a server's hook can write a refusal's own words there,
///   and a push whose hook writes them and then lands must not be read as a refusal
///   (`doc/planning/cli-git-typed-network-command-next-step-at-a-new-host-key-design.md`,
///   "What a shipped K,
///   K2 or C must get right").
///   The rule belongs to the reading and not to a caller,
///   so every face that reads a transcript gets it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// lines.filter((line) => !line.replace(/^\u001b\[K/, '').startsWith('remote: ')).join('\n')
/// ```
pub(crate) fn own_lines_of(text: &str) -> String {
    // What: `Vec::new()` is an empty growable list; its siblings are `&[&str]`, a borrowed view, and `[&str; N]`, a fixed array.
    // Why:   The lines kept are joined again into one text the readings below compare words in.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const kept: string[] = [];
    // ```
    let mut kept: Vec<&str> = Vec::new();
    for line in text.lines() {
        // What: `strip_prefix` removes the erase-line escape when it is there, and answers nothing when it is not.
        // Why:   The prefix sits in front of the `remote: ` marker exactly on the arms that run at a terminal.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const withoutEscape = line.startsWith('\u001b[K') ? line.slice(3) : line;
        // ```
        let without_escape: &str = line.strip_prefix(ERASE_LINE_ESCAPE).unwrap_or(line);
        if without_escape.starts_with(RELAYED_LINE_PREFIX) {
            continue;
        }
        kept.push(line);
    }
    return kept.join("\n");
}

/// What:
///  Whether a transcript says the person's settings refuse to ask about a new host key.
/// Why:
///   OpenSSH's strict-checking line is also an unknown key's own line,
///   so a reading that only knew an unknown key from it would advise `ssh <host>`,
///   which reads the same configuration and refuses the same way;
///   the strict sentence's own command overrides that setting for one connection
///   (`doc/planning/cli-git-typed-network-command-next-step-at-a-new-host-key-design.md`,
///   the strict sentence).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// text.toLowerCase().includes('host key is known for') && text.toLowerCase().includes('and you have requested strict checking')
/// ```
pub fn reads_as_a_strict_host_key_refusal(text: &str) -> bool {
    // What: `to_ascii_lowercase` copies the text with its capital letters lowered; the sibling `to_lowercase` also rewrites
    //       letters outside ASCII, which OpenSSH's lines have none of.
    // Why:   The comparison is over words, and OpenSSH's sentence starts with a capital.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const folded = text.toLowerCase();
    // ```
    let folded: String = own_lines_of(text).to_ascii_lowercase();
    return folded.contains(NO_HOST_KEY_KNOWN_UNDER_STRICT_CHECKING_PHRASE)
        && folded.contains(STRICT_CHECKING_REQUESTED_PHRASE);
}

/// What:
///  Whether text a failed push printed says this machine does not know the SSH host key.
/// Why:
///   The no-prompt rule sends OpenSSH's yes/no question for an unknown host key to a program that answers nothing,
///  and an empty answer is not "yes",
///  so the connection is refused before any login and OpenSSH prints `Host key verification failed.`
///  (`confirm`,
///  `sshconnect.c` at `V_10_6_P2` lines 603 to 627).
///  That refusal is neither a credential the push lacked nor a fault it can retry away,
///  so it gets its own sentence.
///  OpenSSH prints that same last line for every refusal it makes over a host key,
///  and the other refusals are different causes that must keep OpenSSH's own words alone,
///  as `main` prints them:
/// a changed key says someone may be attacking the connection,
///  and a revoked key says a stolen key may be in use,
///  so the sentence that says to add the key would answer either warning with advice to trust the very key it doubts.
///  The reading is therefore a positive one about the words an unknown key leaves:
///  every refusal prints a line that names a host key or a certificate,
///  so a transcript is this refusal only when its host-key words are an unknown key's own:
///  the last line by itself (the default `ask` path,
///  where OpenSSH prints nothing of its own and the question is the askpass program's,
///  `sshconnect.c` lines 1211 to 1246),
///  the sentence for a key this machine has none of under strict checking (line 1204),
///  or the wrapper's own sentence,
/// which a pre-push hook may echo into the push's words.
///  Anything else that names a host key or a certificate is another refusal:
///  a changed key (`warn_changed_key`,
///  lines 1716 to 1726,
///  and `Host key for ... has changed and you have requested strict checking.`,
///  lines 1358 to 1360),
///  a key a `@revoked` line marks (the `HOST_REVOKED` case,
///  lines 1287 to 1303)
///  or a `RevokedHostKeys` file lists (lines 1552 to 1556),
///  and a certificate that does not fit the host (`Certificate invalid: ...`,
///  `sshkey.c`).
///  So a refusal this release has never seen keeps OpenSSH's words rather than gaining a sentence that may mislead.
///  Two kinds of line are passed over,
///  because neither can hold a refusal's own words:
///  the lines OpenSSH prints only at a debug log level (`debug1: `,
///  `debug2: `,
///  `debug3: `,
///  which `do_log` writes in front of those three levels alone,
///  `log.c` lines 371 to 382),
///  and the words inside single quotes,
///  which OpenSSH writes only around a host name,
///  an address or a path,
///  and Git only around the remote's URL in the line that ends a push its hook refused.
///  A certificate that does not fit is no refusal of its own:
///  `fail:` drops the certificate and retries with the plain key (`sshconnect.c` lines 1473 to 1486),
///  so the words that end the connection are that plain key's.
///  Two certificate cases still answer as other refusals:
///  a key a `@revoked` line marks,
///  whose words OpenSSH prints before `fail:` returns (lines 1287 to 1303),
///  and `Certificate invalid: ...` (`sshkey.c`),
///  which a certificate that does not fit the host prints before that last line.
///  Each arm's transcript was measured against a throwaway `sshd` with OpenSSH 10.6p2
///  and the wrapper's own two variables (`SSH_ASKPASS=false`,
///  `SSH_ASKPASS_REQUIRE=force`),
///  certificates included.
///  One refusal is still read as an unknown key though it is not:
///  with `CheckHostIP=yes` and a key this machine knows for the host beside a different one for its address,
///  OpenSSH writes that warning into `confirm`'s question (`sshconnect.c` lines 1429 to 1447),
///  the askpass program gets the question and standard error gets nothing of it,
///  so the transcript is the last line alone,
///  exactly as an unknown key's,
///  and nothing in a transcript can tell the two apart.
///  The sentence's way to add the key is then wrong for it and its way to connect once is right,
///  because the question plain `ssh` asks is OpenSSH's own about the address.
///  A second case is read as an unknown key for a different reason:
///  under `LogLevel=FATAL` OpenSSH prints no `error` line at all
///  (`do_log` writes `fatal` and `error` without a level word,
///  `log.c` lines 354 to 393),
///  so a changed key and a key a `@revoked` line or a `RevokedHostKeys` file revokes leave only
///  `Host key verification failed.` on standard error,
///  exactly as an unknown key does,
///  and `main` reads those two the same way.
///  The sentence's way to add the key is wrong for them and its way to connect once is right,
///  and no transcript of the words can tell them from an unknown key at that level.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const lines = text.split('\n').filter((line) => !isADebugLine(line))
/// text.includes('Host key verification failed') && lines.every((line) => !namesAHostKeyOutsideQuotes(line) || isAnUnknownKeysOwnLine(line))
/// ```
pub fn reads_as_unknown_host_key(text: &str) -> bool {
    // What: `own` holds the lines this program wrote, without what Git relayed from the far end.
    // Why:   The phrase is looked for in the same lines the loop below reads, so a transcript whose only refusal's words came
    //        from the far end is not read as one.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const own = ownLinesOf(text); if (!own.includes('Host key verification failed')) return false;
    // ```
    let own: String = own_lines_of(text);
    if !own.contains(HOST_KEY_NOT_KNOWN_PHRASE) {
        return false;
    }
    // One fold of the constant, and one of each line: the comparison is the whole reason to read the lines.
    let last_line: String = HOST_KEY_NOT_KNOWN_PHRASE.to_ascii_lowercase();
    for line in own.lines() {
        let folded: String = line.to_ascii_lowercase();
        if is_a_debug_line(folded.as_str()) || !names_a_host_key_outside_quotes(folded.as_str()) {
            continue;
        }
        let an_unknown_keys_own: bool = folded.contains(last_line.as_str())
            || (folded.contains(NO_HOST_KEY_KNOWN_UNDER_STRICT_CHECKING_PHRASE)
                && folded.contains(STRICT_CHECKING_REQUESTED_PHRASE))
            || folded.contains(HOST_KEY_NOT_KNOWN_SENTENCE_OPENING);
        if an_unknown_keys_own {
            continue;
        }
        return false;
    }
    return true;
}

/// Whether one line,
/// in lower case,
/// names a host key or a certificate outside the single quotes in which OpenSSH and Git write data,
/// which every refusal OpenSSH makes over a host key does in at least one line.
/// OpenSSH writes a host name,
/// an address or a file path inside them,
/// and Git writes the remote's URL,
/// so the word in a remote's name is no refusal's own.
fn names_a_host_key_outside_quotes(folded: &str) -> bool {
    for words in folded.split('\'').step_by(2) {
        if words.contains(HOST_KEY_WORDS) || words.contains(CERTIFICATE_WORD) {
            return true;
        }
    }
    return false;
}

/// Whether one line,
/// in lower case,
/// is one OpenSSH prints only while the account asks it for debug output,
/// which no refusal it makes over a host key is:
/// `error`,
/// `logit` and `fatal` print their message alone at any log level,
/// and a debug line that names a host key or a certificate is either news about the key
/// (`Server host key: ...`,
/// `sshconnect.c` at `V_10_6_P2` lines 1523 to 1539)
/// or the certificate retry's own note (lines 1308 to 1317),
/// which `fail:` follows by dropping the certificate for the plain key (lines 1473 to 1486).
fn is_a_debug_line(folded: &str) -> bool {
    for prefix in OPENSSH_DEBUG_LINE_PREFIXES {
        if folded.starts_with(prefix) {
            return true;
        }
    }
    return false;
}

/// The last line OpenSSH prints when it will not connect because of the host key,
/// whichever refusal it found (`sshconnect2.c` at `V_10_6_P2` lines 102 to 104,
/// after `verify_host_key` answered non-zero).
const HOST_KEY_NOT_KNOWN_PHRASE: &str = "Host key verification failed";

/// The erase-line escape Git writes in front of every line it relays from the far end,
/// when its standard error is a terminal and `TERM` is not dumb (`sideband.c` at `v2.56.1`,
/// lines 316 to 318).
const ERASE_LINE_ESCAPE: &str = "\u{1b}[K";

/// The prefix Git writes in front of every line the far end printed (`sideband.c` at `v2.56.1`).
const RELAYED_LINE_PREFIX: &str = "remote: ";

/// The words,
/// in lower case,
/// with which a line names the thing a host-key refusal is about.
const HOST_KEY_WORDS: &str = "host key";

/// The word,
/// in lower case,
/// with which a line names the other thing a host-key refusal is about.
const CERTIFICATE_WORD: &str = "certificate";

/// The words `do_log` puts in front of every line of a debug log level,
/// and of those three levels alone (`log.c` at `V_10_6_P2` lines 371 to 382);
/// a line an error,
/// a log or a fatal refusal printed on standard error carries no prefix at all.
const OPENSSH_DEBUG_LINE_PREFIXES: &[&str] = &["debug1: ", "debug2: ", "debug3: "];

/// The words of OpenSSH's sentence for a host key this machine has none of while the account asks for strict checking
/// (`check_host_key`'s `HOST_NEW` case,
/// `sshconnect.c` lines 1204 and 1205).
const NO_HOST_KEY_KNOWN_UNDER_STRICT_CHECKING_PHRASE: &str = "host key is known for";

/// The words that end that sentence,
/// which no changed or revoked key's line carries beside `host key is known for`.
const STRICT_CHECKING_REQUESTED_PHRASE: &str = "and you have requested strict checking";

/// The opening words,
/// in lower case,
/// of the sentence this module prints for an unknown host key,
/// which a pre-push hook may echo into the words of the push it refused,
/// and which the service's record reads so that one refusal is told once.
pub(crate) const HOST_KEY_NOT_KNOWN_SENTENCE_OPENING: &str =
    "this machine does not know the ssh host key";

/// What:
///  The one sentence a refusal prints when the cause is a host key this machine does not know:
///  it says the connection was refused before any login,
///  and names the two ways to add the key that the wrapper does not touch,
///  then to retry.
/// Why:
///   The project's answer of 2026-11-20 keeps the refusal the no-prompt rule causes at a terminal
///  and puts the way out where a person meets it:
///  connect once with plain `ssh <host>`,
///  which still asks its own question,
///  or add the host key to `known_hosts`,
///  then push again.
///  No retry can add a host key,
///  so saying only that the push failed would leave the person with no move.
///  The sentence is shared by the push's failure record and the upload's explanation,
///  so the two faces cannot say different things about one refusal.
///  `host` is the host the wrapper can name for this refusal,
///  and the remote is written as `ssh <host>` only when it names none:
///  an upload's destination may be another host than the Git remote,
///  and a person told to run `ssh` must be told which machine to name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `monochromatic-git-policy-cli: this machine does not know the SSH host key ... Connect once with plain ${host ? `ssh ${host}` : 'ssh <host>'} ...`
/// ```
pub fn host_key_not_known_sentence(host: Option<&str>) -> String {
    let connect: String = match host {
        Some(name) => format!("Connect once with plain `ssh {name}`"),
        None => String::from("Connect once with plain `ssh <host>`"),
    };
    return format!(
        "monochromatic-git-policy-cli: this machine does not know the SSH host key of the remote, and no command the wrapper \
         runs asks whether to trust a new one (`SSH_ASKPASS_REQUIRE=force` sends that yes/no question to a program that \
         answers nothing), so the connection was refused before any login. {connect}, which the wrapper does not touch and \
         which still asks, or add the host key to `known_hosts`, then push again."
    );
}
