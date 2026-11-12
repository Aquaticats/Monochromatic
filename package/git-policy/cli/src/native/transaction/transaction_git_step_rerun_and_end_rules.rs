//! What:
//!  Which Git steps may be run again after the system refused a process,
//! and which may be ended at a ceiling:
//!  the allow-lists,
//! the parse of a step's words into options and paths,
//! and the proven forms.
//! Why:
//!  The runner (`transaction_git.rs`) asks these questions of a `GitRequest` before it reruns or ends a step,
//!  and the step bounds (`transaction_git_step_bounds.rs`) ask the second;
//! they depend only on the request type.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // mayBeRerun(request): boolean; mayBeEnded(request): boolean; isSideRefWrite(request): boolean; subcommandOf(args)
//! ```

/// The request whose command line is judged.
use crate::transaction_git_request_and_reply_types::GitRequest;
/// `OsString` is owned operating-system text,
/// the type of a command-line argument.
use std::ffi::OsString;

/// Commands whose second run after a refusal is harmless:
/// they read,
/// or they write content-addressed objects,
///  or they set the same value again.
/// `symbolic-ref` and `merge-file` are not here:
/// only the forms the wrapper runs may be run again (`may_be_rerun`).
///  `init` is not run again in place at all:
/// its one form is ended at a ceiling and restarted in a fresh directory (`may_be_ended`).
const RERUNNABLE: [&str; 15] = [
    "rev-parse",
    "for-each-ref",
    "ls-files",
    "diff",
    "diff-index",
    "diff-tree",
    "cat-file",
    "show",
    "hash-object",
    "write-tree",
    "ls-remote",
    "merge-tree",
    "pack-objects",
    "index-pack",
    "version",
];

/// What:
///  The subcommand a request runs and where it sits among the arguments,
///  past the global options (`-c <pair>`,
/// `-C <directory>` and the `--option` forms).
/// Why:
///   The rerun allow-list is decided by the subcommand and,
/// for a few,
/// by its options.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const index = args.findIndex(...); // first token that is not a global option or its value
/// ```
pub(crate) fn subcommand_of(arguments: &[OsString]) -> Option<(usize, String)> {
    let mut index: usize = 0;
    while index < arguments.len() {
        let text: String = arguments[index].to_string_lossy().into_owned();
        if text == "-c" || text == "-C" {
            index += 2;
            continue;
        }
        if text.starts_with('-') {
            index += 1;
            continue;
        }
        return Some((index, text));
    }
    return None;
}

/// What:
///  Whether an `--object-format=<name>` option names a hash the wrapper creates shadows with:
/// `sha1` or `sha256`.
/// Why:
///  `is_shadow_creation` accepts nothing but the one `init` form the wrapper runs,
/// and the format is the one word in it that varies.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const objectFormatIsKnown = (option: string) => ['sha1', 'sha256'].includes(option.replace(/^--object-format=/, ''));
/// ```
fn object_format_is_known(option: &str) -> bool {
    return match option.strip_prefix("--object-format=") {
        Some(name) => matches!(name, "sha1" | "sha256"),
        None => false,
    };
}

/// Whether `words` are the one `init` the wrapper runs (`shadow_create.rs`):
/// `--bare --template= --object-format=<format>
///  --ref-format=reftable <path>`,
/// and nothing else,
/// so no `--separate-git-dir`,
/// template or relocation can stand in it.
fn is_shadow_creation(words: &[String]) -> bool {
    return match words {
        [bare, template, format, refs, path] => {
            bare == "--bare"
                && template == "--template="
                && object_format_is_known(format)
                && refs == "--ref-format=reftable"
                && !path.starts_with('-')
        }
        _ => false,
    };
}

/// What:
///  Whether a `symbolic-ref` is a form the wrapper runs that may be run again:
/// a read (`--quiet`,
/// `--short`,
/// `--no-recurse`,
/// one name),
///  or,
/// in a shadow repository the wrapper made for one transaction,
/// the write of one name to one target without any option.
/// Why:
///   The write forms the wrapper runs are in `shadow_refs.rs` and only on a shadow (`without_prefix`);
/// a write in the person's
///  repository,
/// a delete and a message (it appends a reflog entry) are branch writes or have effects the experiment did not run.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// reads(words) || (shadow && words.options.length === 0 && words.names.length === 2)
/// ```
fn symbolic_ref_may_be_rerun(words: &Words<'_>, shadow: bool) -> bool {
    if words.after.is_some() {
        return false;
    }
    let read: bool = words.only_options(&["--quiet", "-q", "--short", "--no-recurse"]) && words.before.len() == 1;
    let write_in_shadow: bool = shadow && words.options.is_empty() && words.before.len() == 2;
    return read || write_in_shadow;
}

/// What:
///  The options a `merge-file` may carry and still write nothing:
/// the answer on standard output,
/// the object names instead of file names,
///  the conflict style,
/// a marker width,
/// the three labels and the quiet flag.
/// Why:
///  `merge_file_reads_only` accepts a `merge-file` only when every option is one of these;
///  `--marker-size` is here because it changes the markers and no file,
///  and `-L` because its value is a label.
const MERGE_FILE_READ_OPTIONS: [&str; 12] = ["--stdout", "-p", "--object-id", "--zdiff3", "--diff3", "--ours", "--theirs", "--union", "-q", "--quiet", "-L", "--marker-size"];

/// What:
///  Whether a `merge-file` is the read form the word-level second pass runs:
/// `--stdout` with `--object-id` over three object names,
/// no pathspec after a `--`,
///  and no other option than `MERGE_FILE_READ_OPTIONS` names.
/// Why:
///  `git merge-file` writes the merged bytes into its first file unless it is told `--stdout`,
/// and with `--object-id` the three inputs are object names,
///  so that form writes nothing at all and may be run twice.
/// The second pass starts exactly it over the three stages of one conflicted path
/// (`merge_second_pass_git_lines.rs`),
///  and it must be a step the wrapper ends:
/// the pass parses that answer,
/// and the captured files of a step that is never ended keep
///  only the last 8 MiB of it (`output_capture.rs`,
/// `STEP_OUTPUT_LIMIT`),
/// which is a file's beginning lost.
///  Measured on Git 2.56.0 in a throwaway repository on 2026-11-11,
/// under `strace --trace=openat,creat,truncate,unlink,rename,mkdir,link,symlink`:
///  the form opens nothing for writing (only `/dev/null`),
/// removes and renames nothing,
/// and leaves the repository byte-identical
///  after a second run,
/// with the reflog,
/// the refs,
/// the index,
/// the worktree,
/// the objects and the Git directory compared;
/// the experiment that decides a ceiling for a write in place
///  has nothing to cut here (`doc/planning/cli-git-no-unbounded-waits.md`,
/// "The cut experiment").
fn merge_file_reads_only(words: &Words<'_>) -> bool {
    if !words.only_options(&MERGE_FILE_READ_OPTIONS) {
        return false;
    }
    let prints: bool = words.has("--stdout") || words.has("-p");
    return words.after.is_none() && prints && words.has("--object-id");
}

/// What:
///  The arguments after the subcommand,
/// as text.
/// Why:
///  The rerun and ending rules judge the words of a step,
/// and arguments that are not valid text are read lossily so a
///  odd byte never makes a rule panic.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// request.args.slice(position + 1).map(String)
/// ```
fn words_after(request: &GitRequest, position: usize) -> Vec<String> {
    let mut words: Vec<String> = Vec::new();
    for argument in &request.arguments[position + 1..] {
        words.push(argument.to_string_lossy().into_owned());
    }
    return words;
}

/// What:
///  Whether a `config` option is one that only reads:
/// `--get`,
/// `--get-all`,
/// `--get-regexp`,
/// `--list` or `-l`.
/// Why:
///  `may_be_rerun` allows a `config` that reads and no other,
/// because a second read changes nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isReadOption = (option: string) => ['--get', '--get-all', '--get-regexp', '--list', '-l'].includes(option);
/// ```
fn is_read_option(option: &str) -> bool {
    return matches!(option, "--get" | "--get-all" | "--get-regexp" | "--list" | "-l");
}

/// What:
///  Whether running the request a second time after the system refused it is harmless.
/// Why:
///   Git reports a refused process or thread as a failure after it started,
///  so only commands that read,
/// write content-addressed objects,
/// or change a private index the same way twice
///  may be run again;
///  reference updates,
/// commits and every other writer are not,
///  and recovery already completes or rolls those back.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// RERUNNABLE.includes(subcommand) || (subcommand === 'config' && reads(args)) || ...
/// ```
pub(crate) fn may_be_rerun(request: &GitRequest) -> bool {
    let Some((position, name)) = subcommand_of(request.arguments.as_slice()) else {
        return false;
    };
    let rest: Vec<String> = words_after(request, position);
    match name.as_str() {
        "config" => return rest.iter().map(String::as_str).any(is_read_option),
        "remote" => return rest.first().map(String::as_str) == Some("get-url"),
        "add" | "reset" | "read-tree" | "update-index" => return request.index.is_some(),
        "symbolic-ref" => return symbolic_ref_may_be_rerun(&Words::of(rest.as_slice()), request.without_prefix),
        "merge-file" => return merge_file_reads_only(&Words::of(rest.as_slice())),
        other => return RERUNNABLE.contains(&other),
    }
}

/// Steps that change nothing,
/// or that write only objects nothing refers to,
/// and that `RERUNNABLE` does not name.
/// A second run of an
///  object writer writes the same bytes and a partial result is a loose object or a temporary pack that no reference names,
/// so
///  ending one costs nothing.
/// Only the name is judged:
/// none of these steps writes a reference,
/// the index,
/// the worktree or the
///  configuration whatever its options.
/// `fast-import` is not among them (sol-1 round 2):
/// its input can update branches and
///  checkpoint them,
/// and the cut experiment ran the wrapper's own stream and found a marks lock file a cut left behind that the
///  second run could not pass,
/// so it is never ended.
const READS_AND_OBJECT_WRITERS: [&str; 15] = [
    "rev-list",
    "merge-base",
    "log",
    "ls-tree",
    "status",
    "var",
    "diff-files",
    "check-attr",
    "check-ignore",
    "stripspace",
    "fmt-merge-msg",
    "show-ref",
    "name-rev",
    "commit-tree",
    "mktree",
];

/// The words of a request after its subcommand,
/// sorted into the options before a `--`,
/// the other words before it,
/// and the words
///  after it.
/// Only options that a proven form lists are accepted,
/// so no option here takes a value the next word could be mistaken
///  for.
struct Words<'a> {
    /// The words that start with a hyphen,
    /// before any `--`.
    options: Vec<&'a str>,
    /// The other words before any `--`.
    before: Vec<&'a str>,
    /// The words after a `--`,
    /// when there was one.
    after: Option<Vec<&'a str>>,
}

impl<'a> Words<'a> {
    /// Sort `words`.
    fn of(words: &'a [String]) -> Self {
        let mut sorted: Words<'a> = Words { options: Vec::new(), before: Vec::new(), after: None };
        for word in words.iter().map(String::as_str) {
            match sorted.after.as_mut() {
                Some(after) => after.push(word),
                None if word == "--" => sorted.after = Some(Vec::new()),
                None if word.starts_with('-') && word.len() > 1 => sorted.options.push(word),
                None => sorted.before.push(word),
            }
        }
        return sorted;
    }

    /// Whether every option is one of `allowed`.
    fn only_options(&self, allowed: &[&str]) -> bool {
        for option in &self.options {
            if !allowed.contains(option) {
                return false;
            }
        }
        return true;
    }

    /// Whether any word came after a `--` and there is at least one of them.
    fn names_paths(&self) -> bool {
        return match &self.after {
            Some(paths) => !paths.is_empty(),
            None => false,
        };
    }

    /// How many of `wanted` are among the options.
    fn count_present(&self, wanted: &[&str]) -> usize {
        let mut present: usize = 0;
        for option in wanted {
            if self.has(option) {
                present += 1;
            }
        }
        return present;
    }

    /// Whether the option `wanted` is there.
    fn has(&self, wanted: &str) -> bool {
        return self.options.contains(&wanted);
    }
}

/// The setting the wrapper puts on the command line of every write of a side ref,
/// with `--no-deref`:
/// a side ref is not created with a
///  reflog and is not followed through a symbolic ref.
/// It is hygiene and not a proof:
/// the write is never ended at a ceiling,
/// whatever
///  its argv says,
/// because a reflog that another Git creates between any check and the write is appended to twice by a cut write and
///  its rerun,
/// and nothing the wrapper holds excludes an ordinary Git (cross-vendor review of round 4,
/// P1 B).
pub(crate) const NO_REFLOG_SETTING: &str = "core.logAllRefUpdates=false";

/// What:
///  Whether a `restore` option is one the cut experiment ran:
/// `--staged`,
/// `--worktree`,
/// `--quiet`,
/// `-q` or `--source=<tree>`.
/// Why:
///  `is_proven_to_run_twice` accepts a `restore` only when every option is one of these.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isRestoreOption = (option: string) => ['--staged', '--worktree', '--quiet', '-q'].includes(option) || option.startsWith('--source=');
/// ```
fn is_restore_option(option: &&str) -> bool {
    return matches!(*option, "--staged" | "--worktree" | "--quiet" | "-q") || option.starts_with("--source=");
}

/// What:
///  Whether the request's step is one the cut experiment ran twice to the same state,
/// in the exact form it ran.
/// Why:
///   The owner's criterion (2026-11-04,
/// "Never cut a branch write (Recommended)"):
/// "Steps that can safely run twice keep their
///  ceilings".
/// Decided by experiment and not by reading (`doc/planning/cli-git-no-unbounded-waits.md`,
/// "The cut experiment"):
/// each
///  form was ended with SIGKILL and with SIGTERM at every mutating system call of its run on a throwaway repository,
/// the lock
///  files a killed Git leaves were removed as the wrapper's sweep removes them,
/// the step was run again,
/// and the refs,
/// reflogs,
///  index,
/// worktree,
/// stash list,
/// configuration and every file of the Git directory were compared with an uncut run's.
///  This is a whitelist of those forms,
/// parsed by their options and words (sol-1:
/// a rule by the name of the step ended
///  `config --add`,
/// `stash -m list` and `update-ref --create-reflog`,
/// which are not the forms that were run):
/// `stash list --format=%H`,
///  `clean --force -d`,
/// `read-tree --reset -u <tree>`,
/// `update-index`
///  with `--refresh`,
/// with `--index-info` (and `-z`) or with `--add --cacheinfo <triple>`,
/// `add --update` or `add --all` with
///  optional pathspecs,
/// `checkout-index --all --force`,
/// `restore` with `--staged`,
/// `--worktree`,
/// `--source=` and paths after `--`,
///  `reset [--quiet] <commit> -- <paths>`,
/// and `config [--local] <key> <value>`.
/// The experiment ran these on Git 2.56.0 with
///  `core.fsync=none`,
/// the files ref backend,
/// no hooks and no concurrent writer.
///  Any other form of any of them,
/// and any other step,
/// is not proven and has no ceiling.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// PROVEN_FORMS.some((form) => form.matches(parse(subcommand, words)))
/// ```
fn is_proven_to_run_twice(name: &str, rest: &[String]) -> bool {
    let words: Words<'_> = Words::of(rest);
    let no_paths = words.after.is_none();
    return match name {
        "stash" => rest.first().map(String::as_str) == Some("list") && rest.len() == 2 && rest[1] == "--format=%H",
        "clean" => no_paths && words.before.is_empty() && words.only_options(&["--force", "-d"]) && words.has("--force") && words.has("-d"),
        "read-tree" => no_paths && words.before.len() == 1 && words.only_options(&["--reset", "-u"]) && words.has("--reset") && words.has("-u"),
        "update-index" => {
            let refresh: bool = no_paths && words.before.is_empty() && words.options == ["--refresh"];
            let index_info: bool = no_paths && words.before.is_empty() && words.has("--index-info") && words.only_options(&["--index-info", "-z"]);
            let cache_info: bool = no_paths && words.before.len() == 1 && words.only_options(&["--add", "--cacheinfo"]) && words.has("--add") && words.has("--cacheinfo");
            refresh || index_info || cache_info
        }
        "add" => {
            let one_of: usize = words.count_present(&["--update", "-u", "--all", "-A"]);
            words.only_options(&["--update", "-u", "--all", "-A"]) && one_of == 1
        }
        "checkout-index" => no_paths && words.before.is_empty() && words.only_options(&["--all", "-a", "--force", "-f"]) && (words.has("--all") || words.has("-a")) && (words.has("--force") || words.has("-f")),
        "restore" => {
            let options_known: bool = words.options.iter().all(is_restore_option);
            options_known && words.before.is_empty() && words.names_paths()
        }
        "reset" => words.only_options(&["--quiet", "-q"]) && words.before.len() == 1 && words.names_paths(),
        "config" => no_paths && words.only_options(&["--local"]) && words.before.len() == 2,
        _ => false,
    };
}

/// What:
///  Whether a step that shows no sign of work may be ended.
/// Why:
///   The quiet window ends a step only when the wrapper can answer for what it did:
///  a step it runs again,
///  or one whose caller puts back every byte it changed
///  (`ended_at_its_bound` in `transaction_git_request_and_reply_types.rs`).
/// A step that is neither is only announced and waited for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// mayBeRerun(request) || request.endedAtItsBound
/// ```
pub(crate) fn may_be_ended_when_quiet(request: &GitRequest) -> bool {
    return may_be_rerun(request) || request.ended_at_its_bound;
}

/// What:
///  Whether the request's step may be ended at a ceiling:
/// it can be run twice,
/// by the allow-list `may_be_rerun`,
/// or it only reads
///  or writes unreferenced objects,
/// or it is a proven form,
///  or its caller puts back every byte it changed.
/// Why:
///   Anything else has no ceiling,
/// because data is lost by guessing wrong and a wait is not (the coordinator's reading of the owner's
///  criterion):
/// the commit service's landing thread waits for it,
/// and any other command stops waiting and leaves it running.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// mayBeRerun(request) || request.endedAtItsBound || READS_AND_OBJECT_WRITERS.includes(name) || isProvenToRunTwice(name, words)
/// ```
pub(crate) fn may_be_ended(request: &GitRequest) -> bool {
    if may_be_rerun(request) {
        return true;
    }
    // A caller that puts back every byte its step changed proves the ending itself, and no name of a command is enough
    //  for that on its own (`transaction_git_request_and_reply_types.rs`, `ended_at_its_bound`).
    if request.ended_at_its_bound {
        return true;
    }
    let Some((position, name)) = subcommand_of(request.arguments.as_slice()) else {
        return false;
    };
    if READS_AND_OBJECT_WRITERS.contains(&name.as_str()) {
        return true;
    }
    let rest: Vec<String> = words_after(request, position);
    // The shadow's creation cannot be run twice in place (a cut `init` leaves a directory the second run fails in), and it is ended at
    //  its ceiling because its creator discards that directory and starts a fresh one (`shadow_create.rs`, `init_with_git`), which the
    //  cut experiment found equal to an uncut run.
    if name == "init" {
        return request.without_prefix && is_shadow_creation(&rest);
    }
    return is_proven_to_run_twice(name.as_str(), rest.as_slice());
}

/// What:
///  Whether the request is a branch write:
/// an `update-ref` that is not one of the wrapper's own side-ref writes.
/// Why:
///   The owner's pick of 2026-11-08 ("B:
/// let a branch write finish (Recommended)") lets a stop of the service wait for the write the landing makes to move the branch,
/// and no other step:
/// the import of objects and every read are still ended by the stop.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// isBranchWrite(request) // subcommand is update-ref and not isSideRefWrite(request)
/// ```
pub(crate) fn is_branch_write(request: &GitRequest) -> bool {
    let Some((_position, name)) = subcommand_of(request.arguments.as_slice()) else { return false };
    return name == "update-ref" && !is_side_ref_write(request);
}

/// Whether the request is an `update-ref` of one of the wrapper's own side refs,
/// `refs/cli-git/kept/<number>` or
///  `refs/worktree/cli-git-service/<number>`,
/// with at most `-d` and `--no-deref` and no input:
/// what `keep_unlanded` and the scratch
///  deletions of `service_land.rs` run.
pub(crate) fn is_side_ref_write(request: &GitRequest) -> bool {
    let Some((position, name)) = subcommand_of(request.arguments.as_slice()) else { return false };
    if name != "update-ref" || request.input.is_some() {
        return false;
    }
    let rest: Vec<String> = words_after(request, position);
    let words: Words<'_> = Words::of(rest.as_slice());
    let Some(first_word) = words.before.first() else {
        return false;
    };
    return words.after.is_none() && words.only_options(&["-d", "--no-deref"]) && is_side_ref_name(first_word);
}

/// What:
///  Whether a word is one of the wrapper's own side refs:
/// `refs/cli-git/kept/<number>`,
/// the name beside it `refs/cli-git/kept/<number>-rescue`
///  that a worktree removal keeps a held commit on when that name cannot be written,
/// or `refs/worktree/cli-git-service/<number>`.
/// Why:
///  `is_side_ref_write` accepts an `update-ref` only for these names,
/// and each ends in a number,
/// with `-rescue` allowed after the kept name's number.
///  The words are the ones
///  `service_scratch_ref_of_a_dropped_batch.rs` writes,
/// spelled again here because that folder uses this one and may not be used by it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// /^refs\/(cli-git\/kept\/[0-9]+(-rescue)?|worktree\/cli-git-service\/[0-9]+)$/.test(word)
/// ```
fn is_side_ref_name(word: &str) -> bool {
    let number: Option<&str> = match word.strip_prefix("refs/cli-git/kept/") {
        Some(rest) => Some(rest.strip_suffix("-rescue").unwrap_or(rest)),
        None => word.strip_prefix("refs/worktree/cli-git-service/"),
    };
    let Some(digits) = number else { return false };
    return !digits.is_empty() && digits.as_bytes().iter().all(u8::is_ascii_digit);
}
