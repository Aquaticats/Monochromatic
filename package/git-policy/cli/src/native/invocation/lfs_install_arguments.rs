//! What:
//!  The `--skip-repo` the wrapper adds to `git lfs install` inside a repository,
//!  and the notice that says why.
//! Why:
//!  The wrapper starts every real Git with traditional hooks switched off
//! (`core.hooksPath` below `/dev/null` in `hooks_off.rs`),
//!  and `git lfs install` writes its four shims into that directory when it runs inside a repository:
//!  `mkdir /dev/null: not a directory`,
//!  status 2,
//!  and advice to run `git lfs update --manual` or `--force`,
//!  neither of which can work while the directory cannot exist.
//!  The global `filter.lfs.*` keys are written first and are what the wrapper's upload needs,
//!  and no hook is needed at all:
//!  the wrapper runs the upload itself before every push
//! (`package/git-policy/cli/SPEC.md`,
//! "Git LFS upload before a push").
//!  So the command the person typed runs with `--skip-repo`,
//!  which git-lfs's own help describes as "Skips installation of hooks into the local repository;
//!  use if you want to install the LFS filters but not make changes to the hooks",
//!  and the wrapper says in one notice what the person would otherwise have to work out:
//!  where this command asks git-lfs to put its filters, or why git-lfs refuses the command before it writes a key,
//!  and that no hook is needed here.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const [args, notice] = withSkipRepo(stripped.arguments, inRepository); // `git lfs install --skip-repo`
//! ```

/// The global-option layout,
/// to find the command word behind the caller's global options.
use crate::global_arguments::{GlobalOutcome, global_layout};
/// Owned operating-system text for the arguments.
use std::ffi::OsString;

/// The option git-lfs's `install` takes to write the filters and no hook.
pub const SKIP_REPO_OPTION: &str = "--skip-repo";

/// What:
///  The boolean one pflag value spells,
///  or nothing when it is not one.
/// Why:
///  git-lfs's `--local`, `--worktree`, `--system` and `--manual` are pflag booleans,
///  so `--local=true` and `--worktree=false` are read as its flag parser reads them.
///  The spellings are `strconv.ParseBool`'s, which pflag uses.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function booleanValue(text: string): boolean | undefined
/// ```
fn boolean_value(text: &str) -> Option<bool> {
    const TRUE_VALUES: [&str; 6] = ["1", "t", "T", "TRUE", "true", "True"];
    const FALSE_VALUES: [&str; 6] = ["0", "f", "F", "FALSE", "false", "False"];
    if TRUE_VALUES.contains(&text) {
        return Some(true);
    }
    if FALSE_VALUES.contains(&text) {
        return Some(false);
    }
    return None;
}

/// The long boolean options `git lfs install` registers (`commands/command_install.go:97-106`),
/// so a `=value` after one of them is a value `strconv.ParseBool` reads.
/// `--file` (`:99`) is the one string option and takes a path instead, and `--help` is cobra's own.
const BOOLEAN_OPTIONS: [&str; 7] = ["force", "local", "manual", "skip-repo", "skip-smudge", "system", "worktree"];

/// The short forms of the booleans `--force`, `--local`, `--worktree`, `--skip-smudge` and `--manual` have.
///
/// `--file` and `--system` have no short form, so no short option here takes anything but a boolean.
const BOOLEAN_LETTERS: [char; 5] = ['f', 'l', 'w', 's', 'm'];

/// What:
///  The help flag cobra adds to every command, by its long name and its short letter.
/// Why:
///  `InitDefaultHelpFlag` registers it as a boolean (`command.go:1219-1231` of cobra v1.10.2),
///  so it takes a `=value` and a group letter like the booleans above,
///  and a true value prints help and writes no key:
///  `-lh` sets `--local` and asks for help, and git-lfs prints help with status 0.
const HELP_OPTION: &str = "help";

/// The short form of `HELP_OPTION`.
const HELP_LETTER: char = 'h';

/// What:
///  Why git-lfs refuses a `git lfs install` before it writes a `filter.lfs.*` key,
///  as the words the caller typed show it.
/// Why:
///  The notice must not name a destination for a command that reaches none,
///  and each of these refusals is readable from the words themselves,
///  so the notice asks Git and git-lfs nothing before every install
///  (a query on that path could fail on its own, which the notice would then have to answer for too).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type InstallRefusal = { kind: 'word', word: string } | { kind: 'file-without-path' } | { kind: 'several-destinations' }
/// ```
#[derive(Clone)]
enum InstallRefusal {
    /// One word pflag rejects: `-l=yes`, `-l=`, `--local=yes`.
    Word(String),
    /// `--file` with no word after it, which pflag rejects as needing an argument.
    FileWithoutPath,
    /// More than one of `--local`, `--system`, `--worktree` and `--file`.
    SeveralDestinations,
}

impl InstallRefusal {
    /// What:
    ///  Why git-lfs stops, in words a person can act on.
    /// Why:
    ///   The person typed the word and can change it,
    ///  so the reason names that word or the count as git-lfs itself states it,
    ///  and the notice adds that no `filter.lfs.*` key is written, which is what the person cannot see.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function reason(refusal: InstallRefusal): string
    /// ```
    fn reason(&self) -> String {
        return match self {
            InstallRefusal::Word(word) => format!("its flag parser rejects `{word}`"),
            InstallRefusal::FileWithoutPath => String::from("`--file` needs a path and none followed it"),
            InstallRefusal::SeveralDestinations => String::from(
                "only one of `--local`, `--system`, `--worktree` and `--file` can be specified, and this command asks \
                 for more than one",
            ),
        };
    }
}

/// What:
///  Where the `filter.lfs.*` keys of one `git lfs install` are written,
///  as git-lfs's `install` flags choose it.
/// Why:
///  git-lfs 3.8.1 writes the keys to one of five places (`commands/command_install.go:44-59` refuses more than one):
///  the global file by default,
///  `--local` this repository's `.git/config`,
///  `--worktree` this worktree's configuration while Git's `extensions.worktreeConfig` is set,
///  this repository's `.git/config` when it is not and the repository holds one worktree,
///  and nothing at all when it is not and the repository holds several (Git refuses that write and git-lfs ends with
///  status 2),
///  `--system` the system file,
///  and `--file <path>` that file.
///  The scope is read from the words so the notice can name where the command asks for the keys,
///  or, for `--worktree`, every place Git may put them,
///  rather than calling every scope global.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type InstallScope = { kind: 'global' } | { kind: 'local' } | { kind: 'worktree' } | { kind: 'system' } | { kind: 'file', path: string }
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
enum InstallScope {
    /// The global file git-lfs writes when no scope flag is given.
    Global,
    /// This repository's `.git/config`.
    Local,
    /// This worktree's configuration.
    Worktree,
    /// The system-wide configuration.
    System,
    /// The file a `--file` names.
    File(String),
}

impl InstallScope {
    /// What:
    ///  Where this scope writes the keys,
    ///  as a person reads it.
    /// Why:
    ///   The notice names the file the command asks git-lfs for,
    ///  so a person who reads their global file and finds nothing knows where to look.
    ///  `--worktree` is the one scope the wrapper cannot answer without asking Git,
    ///  and it asks nothing on every `git lfs install`:
    ///  so it names each outcome Git chooses instead,
    ///  including the one where Git refuses and writes no file.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function where(scope: InstallScope): string
    /// ```
    fn where_it_writes(&self) -> String {
        return match self {
            InstallScope::Global => String::from("your global configuration"),
            InstallScope::Local => String::from("this repository's `.git/config`"),
            InstallScope::Worktree => String::from(
                "this worktree's configuration (`config.worktree` in this worktree's Git directory) while \
                 `extensions.worktreeConfig` is set, this repository's `.git/config` when it is not and the repository \
                 holds one worktree, or no file at all when it is not and the repository holds several, where Git \
                 refuses the write and git-lfs ends with status 2",
            ),
            InstallScope::System => String::from("the system configuration"),
            InstallScope::File(path) => format!("`{path}`"),
        };
    }
}

/// What:
///  What one `git lfs install` asks for:
///  the destination it asks git-lfs for, or the refusal that stops it before one.
/// Why:
///   The notice names one or the other and never both,
///  because a destination git-lfs never reaches would describe a request to write that never happened.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type InstallAsk = { kind: 'writes', scope: InstallScope } | { kind: 'refused', refusal: InstallRefusal }
/// ```
enum InstallAsk {
    /// git-lfs runs the command and writes the keys here.
    Writes(InstallScope),
    /// git-lfs refuses the words before it writes a key.
    Refused(InstallRefusal),
}

/// What:
///  One `git lfs install` read from the words after its `install` word:
///  the destination flags it sets, whether `--manual` is on,
///  whether the words ask for help, whether the caller turned `--skip-repo` off,
///  and the first word pflag refuses.
/// Why:
///   git-lfs's own flags are the only source of these answers,
///  and `--` ends them,
///  so nothing after it is read as a flag.
///  One reading answers every question the wrapper's rewrite and notice have,
///  so the destination, the refusal, and where the wrapper's own option belongs can never disagree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type InstallRequest = { local: boolean, worktree: boolean, system: boolean, file?: string, manual: boolean, help: boolean, skipRepo?: [number, boolean], unread: boolean, refused?: InstallRefusal }
/// ```
#[derive(Default)]
struct InstallRequest {
    /// `--local` or `-l`, as the last word that named it left it.
    local: bool,
    /// `--worktree` or `-w`.
    worktree: bool,
    /// `--system`, which has no short form.
    system: bool,
    /// The path `--file` names, when it names a non-empty one.
    file: Option<String>,
    /// Whether `--manual` or `-m` was asked for with a true value.
    manual: bool,
    /// Whether `--help` or `-h` was asked for with a true value, which prints help and writes no key.
    help: bool,
    /// The index of the last word before `--` that set `--skip-repo`, when one did.
    skip_repo_at: Option<usize>,
    /// Whether the value that word left was on.
    skip_repo_on: bool,
    /// Whether a word this reading does not name, a long option or a short letter, was read.
    unread_option: bool,
    /// Whether such a word was read before the first refusal this reading names.
    unread_option_before_refusal: bool,
    /// The first word pflag refuses, when one was read.
    refused: Option<InstallRefusal>,
}

impl InstallRequest {
    /// What:
    ///  Where the keys go when git-lfs writes them:
    ///  the file, worktree, local or system scope the words select,
    ///  and the global file when they select none.
    /// Why:
    ///   git-lfs picks the scope the same way and writes the global file when it is given no destination
    ///  (`lfs/attribute.go:154-190`), and `ask` never asks this when the words select more than one destination,
    ///  so at most one of these is set and the order of the checks decides nothing.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const scope = file ? { kind: 'file', path: file }
    ///   : worktree ? { kind: 'worktree' } : local ? { kind: 'local' } : system ? { kind: 'system' } : { kind: 'global' }
    /// ```
    fn scope(&self) -> InstallScope {
        if let Some(path) = &self.file {
            return InstallScope::File(path.clone());
        }
        if self.worktree {
            return InstallScope::Worktree;
        }
        if self.local {
            return InstallScope::Local;
        }
        if self.system {
            return InstallScope::System;
        }
        return InstallScope::Global;
    }

    /// What:
    ///  How many of the four destinations the words ask for.
    /// Why:
    ///   git-lfs counts the true flags and a `--file` path, and refuses the command when the count is more than one
    ///  (`commands/command_install.go:44-59`),
    ///  so the notice needs that same count and not merely the scope it picks first.
    fn destinations(&self) -> usize {
        let flags: usize = usize::from(self.local) + usize::from(self.worktree) + usize::from(self.system);
        return flags + usize::from(self.file.is_some());
    }

    /// What:
    ///  The destination the words ask git-lfs for, or why it refuses the command before it writes one.
    /// Why:
    ///   pflag stops at the first word it cannot read, before git-lfs counts the destinations,
    ///  so a refused word answers first and the count decides after it.
    fn ask(&self) -> InstallAsk {
        if let Some(refusal) = &self.refused {
            return InstallAsk::Refused(refusal.clone());
        }
        if self.destinations() > 1 {
            return InstallAsk::Refused(InstallRefusal::SeveralDestinations);
        }
        return InstallAsk::Writes(self.scope());
    }

    /// What:
    ///  Whether these words make git-lfs print help and write nothing.
    /// Why:
    ///   cobra reads every flag before it looks at the help flag (`command.go:919-936` of cobra v1.10.2),
    ///  so a word the parser refuses is a refusal and only a good parse with help on is a help query:
    ///  `--help=true` and `-lh` print help with status 0 and write no key,
    ///  and `-lh --local=yes` is refused.
    fn asks_for_help(&self) -> bool {
        return self.refused.is_none() && self.help;
    }

    /// What:
    ///  Remember that a word this reading does not name was read,
    ///  and that it came before the first refusal when no refusal was read yet.
    /// Why:
    ///   pflag stops at the first word it cannot read (`flag.go:1153-1155` of pflag v1.0.9),
    ///  so a word git-lfs may refuse stops it before any later refusal this reading names,
    ///  and the notice must not state a reason git-lfs never reached.
    fn note_unread_option(&mut self) {
        self.unread_option = true;
        self.unread_option_before_refusal |= self.refused.is_none();
    }

    /// What:
    ///  The index the wrapper's `--skip-repo` goes at,
    ///  or nothing when a copy the caller wrote already stands.
    /// Why:
    ///   pflag sets a flag again for each word that names it and keeps the last value (`flag.go:484-517`),
    ///  so a copy before the caller's `--skip-repo=false` would be undone and the hook write would fail after all:
    ///  the wrapper's copy follows the word that withdrew it,
    ///  and goes right after `install` when the caller wrote none.
    ///  A word after `--` is no option, so a word there that looks like one is not the option.
    fn skip_repo_insertion(&self, install_index: usize) -> Option<usize> {
        return match self.skip_repo_at {
            Some(at) => (!self.skip_repo_on).then_some(at + 1),
            None => Some(install_index + 1),
        };
    }

    /// What:
    ///  Whether the wrapper's copy answers a word of the caller's that turned `--skip-repo` off.
    fn answers_a_word_that_turned_it_off(&self) -> bool {
        return self.skip_repo_at.is_some() && !self.skip_repo_on;
    }

    /// What:
    ///  Why git-lfs stops, when the words show it,
    ///  or nothing when a word this reading does not name may be the one it stops at.
    /// Why:
    ///   pflag stops at the first word it cannot read, which may be an option this reading passes through,
    ///  so a later word this reading refuses would be named as the reason for a refusal git-lfs never made there.
    ///  A count of destinations is decided only after every word is read,
    ///  so there any word this reading does not name may come first.
    fn named_reason(&self, refusal: &InstallRefusal) -> Option<String> {
        let unread_first: bool = match refusal {
            InstallRefusal::SeveralDestinations => self.unread_option,
            InstallRefusal::Word(_) | InstallRefusal::FileWithoutPath => self.unread_option_before_refusal,
        };
        return (!unread_first).then(|| { return refusal.reason(); });
    }

    /// What:
    ///  The flags one long option sets,
    ///  or the refusal pflag reads from its `=value`.
    /// Why:
    ///   pflag splits a long option at its first `=`;
    ///  a boolean option takes the twelve spellings `strconv.ParseBool` reads,
    ///  and a value outside them stops the command with status 127.
    ///  A long option this file does not name is left as typed and remembered,
    ///  because the list of every option git-lfs registers would go stale as git-lfs adds one,
    ///  and its own message answers a word that is not one.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function readLongOption(request: InstallRequest, at: number, name: string, typed: string): InstallRefusal | undefined
    /// ```
    fn read_long_option(&mut self, at: usize, name: &str, typed: &str) -> Option<InstallRefusal> {
        let (option, given): (&str, Option<&str>) = match name.split_once('=') {
            Some((option, given)) => (option, Some(given)),
            None => (name, None),
        };
        let modelled: bool = BOOLEAN_OPTIONS.contains(&option) || option == HELP_OPTION;
        if !modelled {
            self.note_unread_option();
        }
        if let Some(value) = given
            && modelled
            && boolean_value(value).is_none()
        {
            return Some(InstallRefusal::Word(typed.to_owned()));
        }
        let on: bool = given.is_none_or(|spelled| { return boolean_value(spelled) == Some(true); });
        match option {
            "local" => self.local = on,
            "worktree" => self.worktree = on,
            "system" => self.system = on,
            "manual" => self.manual = on,
            HELP_OPTION => self.help = on,
            "skip-repo" => {
                self.skip_repo_at = Some(at);
                self.skip_repo_on = on;
            }
            _unread => {}
        }
        return None;
    }

    /// What:
    ///  The flags one group of short options sets,
    ///  or the refusal pflag reads from it.
    /// Why:
    ///   pflag reads a group letter by letter,
    ///  and a `=` after a letter gives that letter its value and ends the group:
    ///  `-l=false` withdraws `--local`,
    ///  and `-lw=false` sets `--local` while withdrawing `--worktree`.
    ///  A value that is no spelling of a boolean, and a `=` with nothing after it,
    ///  stop the command with status 127;
    ///  the second is why `-l=` fails on the `=` it then reads as a letter of its own.
    ///  A letter this file does not name is left as typed and remembered,
    ///  and a group that carries `h` asks for help, which writes no key.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function readShortGroup(request: InstallRequest, letters: string, typed: string): InstallRefusal | undefined
    /// ```
    fn read_short_group(&mut self, letters: &str, typed: &str) -> Option<InstallRefusal> {
        let mut remaining: &str = letters;
        while let Some(letter) = remaining.chars().next() {
            let after: &str = remaining.strip_prefix(letter).unwrap_or_default();
            let modelled: bool = BOOLEAN_LETTERS.contains(&letter) || letter == HELP_LETTER;
            if !modelled {
                self.note_unread_option();
            }
            if let Some(value) = after.strip_prefix('=') {
                if modelled && (value.is_empty() || boolean_value(value).is_none()) {
                    return Some(InstallRefusal::Word(typed.to_owned()));
                }
                let on: bool = boolean_value(value) == Some(true);
                match letter {
                    'l' => self.local = on,
                    'w' => self.worktree = on,
                    'm' => self.manual = on,
                    HELP_LETTER => self.help = on,
                    _unread => {}
                }
                return None;
            }
            match letter {
                'l' => self.local = true,
                'w' => self.worktree = true,
                'm' => self.manual = true,
                HELP_LETTER => self.help = true,
                _unread => {}
            }
            remaining = after;
        }
        return None;
    }

    /// What:
    ///  One `git lfs install` read from the words after its `install` word.
    /// Why:
    ///   This is the only place those words are read,
    ///  so the destination and the refusal come from one reading of them,
    ///  and a word after `--` is no option, as pflag reads it.
    ///  A word `--file` takes as its value is no option either,
    ///  even when it starts with a dash: pflag gives the option the next word whatever it looks like.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const request = new InstallRequest()
    /// for (let index = installIndex + 1; index < args.length; index += 1) { if (args[index] === '--') break; /* long option, short group, --file */ }
    /// return request
    /// ```
    fn read(arguments: &[OsString], install_index: usize) -> InstallRequest {
        let mut request: InstallRequest = InstallRequest::default();
        let mut index: usize = install_index + 1;
        while let Some(word) = arguments.get(index) {
            let at: usize = index;
            index += 1;
            let text: String = word.to_string_lossy().into_owned();
            if text == "--" {
                break;
            }
            let refusal: Option<InstallRefusal> = if let Some(path) = text.strip_prefix("--file=") {
                request.file = (!path.is_empty()).then(|| { return path.to_owned(); });
                None
            } else if text == "--file" {
                match arguments.get(index) {
                    Some(path) => {
                        index += 1;
                        let named: String = path.to_string_lossy().into_owned();
                        request.file = (!named.is_empty()).then_some(named);
                        None
                    }
                    None => Some(InstallRefusal::FileWithoutPath),
                }
            } else if let Some(name) = text.strip_prefix("--") {
                request.read_long_option(at, name, text.as_str())
            } else if let Some(letters) = text.strip_prefix('-').filter(|letters| { return !letters.starts_with('-'); }) {
                request.read_short_group(letters, text.as_str())
            } else {
                None
            };
            request.refused = request.refused.or(refusal);
        }
        return request;
    }
}

/// What:
///  The command with `--skip-repo` after the word the caller left it off at,
///  and the notice that says what that means,
///  or the command as it stands and no notice when it is not `git lfs install`.
/// Why:
///   A command outside a repository writes no hook,
///  so it needs no rewrite and no notice about hooks;
///  a help query answers what was asked and changes nothing;
///  and a caller who wrote the option already, with a true value, is told the same thing anyway,
///  because the reason is the wrapper's and not the caller's.
///  The notice is built from the words the caller typed,
///  so it names the destination this command asks git-lfs for, and for `--worktree` every place Git may put the keys,
///  rather than calling every scope global;
///  and it states that request and not a write,
///  because git-lfs can stop after it reads the words well, and the sentence would then be false.
///  A command whose words git-lfs refuses before it writes a key is told that refusal instead,
///  because the notice names no destination that nothing reaches.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (words.slice(installIndex, installIndex + 2).join(' ') !== 'lfs install') return [args, '']; const request = read(args); if (request.asksForHelp()) return [args, '']; const at = request.skipRepoInsertion(installIndex); return [at === undefined ? args : inserted(args, at), notice(request)];
/// ```
pub fn with_skip_repo(arguments: Vec<OsString>, in_a_repository: bool) -> (Vec<OsString>, String) {
    if !in_a_repository {
        return (arguments, String::new());
    }
    let Some(install_index) = install_word(&arguments) else {
        return (arguments, String::new());
    };
    let request: InstallRequest = InstallRequest::read(arguments.as_slice(), install_index);
    if request.asks_for_help() {
        return (arguments, String::new());
    }
    let note: String = notice(&request);
    let Some(at) = request.skip_repo_insertion(install_index) else {
        return (arguments, note);
    };
    let mut rewritten: Vec<OsString> = arguments;
    rewritten.insert(at, OsString::from(SKIP_REPO_OPTION));
    return (rewritten, note);
}

/// What:
///  What a person is told when the wrapper added `--skip-repo` to their `git lfs install`.
/// Why:
///   The command then says only "Git LFS initialized.",
///  which does not say that no hook was installed or that none is needed,
///  and the person who ran it in a repository is the one who would wonder.
///  It states the destination this command asks git-lfs for,
///  the hooks that were not,
///  and the step that replaces them;
///  and it states that request rather than a completed write,
///  because git-lfs can stop after it reads the words well and then write nothing.
///  A command git-lfs refuses before it writes a key is told the refusal instead,
///  because naming a destination there would name a write that does not happen,
///  and the reason is left out when a word this reading does not name may be the one git-lfs stops at.
///  A run with `--manual` that git-lfs runs is told why the instructions that option asks for are not printed:
///  they are instructions for merging a hook this wrapper installs nowhere and never runs,
///  and a refused command prints none of them because it is refused, so this is not added to it.
///  A caller's own `--skip-repo=false` is answered by the wrapper's copy after it,
///  so the notice says that word did not win.
///  `request` is the one reading of the caller's words,
///  which is where the destination, the refusal, `--manual` and that word come from.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `monochromatic-git-policy-cli: \`git lfs install\` ran with \`--skip-repo\` ...
/// ${ask.kind === 'writes'
///   ? `This command asks git-lfs to put the \`filter.lfs.*\` keys in ${where}`
///   : `git-lfs refuses this command, because ${reason}`} ...`
/// ```
fn notice(request: &InstallRequest) -> String {
    let ask: InstallAsk = request.ask();
    let asked: String = match &ask {
        InstallAsk::Writes(scope) => {
            format!("This command asks git-lfs to put the `filter.lfs.*` keys in {}. ", scope.where_it_writes())
        }
        InstallAsk::Refused(refusal) => match request.named_reason(refusal) {
            Some(reason) => format!(
                "git-lfs refuses this command, because {reason}, so no `filter.lfs.*` key is written to any configuration \
                 file. "
            ),
            None => String::from(
                "git-lfs refuses this command, so no `filter.lfs.*` key is written to any configuration file. ",
            ),
        },
    };
    let mut text: String = format!(
        "monochromatic-git-policy-cli: `git lfs install` ran with `--skip-repo`, which the wrapper adds. {asked}The wrapper \
         does not run traditional hooks, and it uploads the Git LFS objects itself before each push, so no `pre-push` hook is \
         installed in this repository and none is needed; the `post-checkout`, `post-commit` and `post-merge` hooks it would \
         install are not needed here either. Without the option git-lfs tries to write its hooks into the directory the \
         wrapper disables, which fails and ends the command with status 2 and advice that cannot work."
    );
    if request.answers_a_word_that_turned_it_off() {
        text.push_str(
            " The last word that set `--skip-repo` turned it off, and the hooks it asks for cannot be installed here: the \
             directory git-lfs writes them into is disabled, so the wrapper's copy follows that word and they are skipped \
             all the same.",
        );
    }
    text.push('\n');
    if request.manual && matches!(ask, InstallAsk::Writes(_)) {
        text.push_str(manual_notice().as_str());
    }
    return text;
}

/// What:
///  What a run with `--manual` is told about the hook instructions it asked for.
/// Why:
///   `--manual` makes git-lfs print instructions for merging its hooks into this repository by hand,
///  and that printing happens in the step `--skip-repo` skips (`commands/command_install.go:30-32`),
///  so the person who asked for them gets none and is owed the reason.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// 'monochromatic-git-policy-cli: `--manual` asks git-lfs for the instructions that merge its hooks by hand; none are printed ...'
/// ```
fn manual_notice() -> String {
    return String::from(
        "monochromatic-git-policy-cli: `--manual` asks git-lfs for the instructions that merge its hooks into this \
         repository by hand, and none are printed here: they are instructions for a hook the wrapper installs nowhere and \
         never runs.\n",
    );
}

/// What:
///  The index of the `install` word of a `git lfs install` command,
///  or nothing for every other command.
/// Why:
///   The command word and its subcommand are read through the same layout the rest of the wrapper reads,
///  so a global option before `lfs` cannot hide it;
///  a help query is left to the one reading of the words, which alone knows whether they ask for help,
///  and `git lfs install hooks` is left alone,
///  because the option changes nothing for that form.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const layout = globalLayout(args); if (layout.outcome !== 'command') return undefined; if (args[layout.prefixLen] !== 'lfs') return undefined; ...
/// ```
fn install_word(arguments: &[OsString]) -> Option<usize> {
    let layout = global_layout(arguments);
    if layout.outcome != GlobalOutcome::Command {
        return None;
    }
    let command: usize = layout.prefix_len;
    if arguments.get(command)? != "lfs" {
        return None;
    }
    let install: usize = command + 1;
    if arguments.get(install)? != "install" {
        return None;
    }
    if arguments.get(install + 1).is_some_and(|word| { return word == "hooks"; }) {
        return None;
    }
    return Some(install);
}

/// The rewrite is controlled in the test module.
#[cfg(test)]
#[path = "lfs_install_arguments_tests.rs"]
mod tests;
