//! What:
//!  The form and file scope of a `git config` region,
//!  read as Git 2.56.0 reads it.
//! Why:
//!  The require-root policy exempts `git config` when it works on the per-user or
//!      system file,
//!  or lists.
//!  Whether `--global` is that option or a value being stored
//!      depends on the form:
//!  `git config user.name --global` writes the text `--global`
//!      into the repository's own configuration.
//!
//! Gotcha:
//!  Git also refuses two different actions of the form without a subcommand word
//!         (`--list --get`);
//!  that conflict is not modeled,
//!  so such a region still reports
//!         its listing.
//!  Git then runs nothing.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // parseConfigRegion(['--global', 'user.name']).global === true
//! // parseConfigRegion(['user.name', '--global']).global === false
//! ```

/// What:
///  Bring the option groups,
///  the tokenizer and its questions into this file.
/// Why:
///   This module only picks the form's groups and interprets what the tokenizer found.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { parseOptions } from './command_options.ts';
/// ```
use super::command_config_table::{
    CONFIG_STOP_MODE, DISPLAY_ROWS, GET_FILTER_ROWS, GET_OTHER_ROWS, GLOBAL, LEGACY_ACTION_ROWS,
    LEGACY_OTHER_ROWS, LIST, LIST_OTHER_ROWS, LOCATION_ROWS, SET_ROWS, SYSTEM, TYPE_ROWS,
    UNSET_ROWS,
};
use super::command_options::{
    DEFAULT_MODE, OptionError, OptionSpec, ParseMode, ParsedOptions, WrapperOccurrence,
    parse_options,
};
use super::command_options_query::{is_enabled, is_stated};
/// `OsString` is owned operating-system text of raw bytes (sibling `String` must be UTF-8).
use std::ffi::OsString;

/// What:
///  The form `cmd_config` dispatches to (builtin/config.c:1633-1660).
///  An `enum` is a
///       closed set of named alternatives.
/// Why:
///   `Legacy` is the form without a subcommand word,
///  where actions are options such as
///       `--list` and `--get`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ConfigForm = 'list' | 'get' | 'set' | 'unset' | 'rename-section' | 'remove-section' | 'edit' | 'legacy';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConfigForm {
    /// `git config list`.
    List,
    /// `git config get`.
    Get,
    /// `git config set`.
    Set,
    /// `git config unset`.
    Unset,
    /// `git config rename-section`.
    RenameSection,
    /// `git config remove-section`.
    RemoveSection,
    /// `git config edit`.
    Edit,
    /// No subcommand word:
    ///  the action is an option or implied by the argument count.
    Legacy,
}

/// What:
///  Facts of one `git config` region.
///  `Vec<WrapperOccurrence>` is an owned growable
///       list of wrapper-only flags.
/// Why:
///   The require-root decision reads the scope and the listing action;
///  the caller
///       removes its wrapper-only flags by the positions reported here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ConfigRegion = { form: ConfigForm; global: boolean; system: boolean; lists: boolean; wrapper: WrapperOccurrence[] };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConfigRegion {
    /// The form Git dispatches to.
    pub form: ConfigForm,
    /// `--global` is on after every use was applied.
    pub global: bool,
    /// `--system` is on after every use was applied.
    pub system: bool,
    /// The command lists variables:
    ///  `git config list`,
    ///  or `-l`/`--list` without a
    /// subcommand word.
    pub lists: bool,
    /// Wrapper-only flags in option position;
    ///  `token` indexes the region.
    pub wrapper: Vec<WrapperOccurrence>,
}

/// What:
///  The form a first region token selects,
///  or nothing.
///  `&[u8]` borrows the token.
/// Why:
///   Git matches the seven words exactly,
///  without abbreviation (parse-options.c
///       `parse_subcommand`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const form = FORMS.get(word);
/// ```
fn form_of_word(word: &[u8]) -> Option<ConfigForm> {
    // `(&[u8], ConfigForm)` is a pair of a spelling and the form it selects.
    let words: [(&[u8], ConfigForm); 7] = [
        (b"list", ConfigForm::List),
        (b"get", ConfigForm::Get),
        (b"set", ConfigForm::Set),
        (b"unset", ConfigForm::Unset),
        (b"rename-section", ConfigForm::RenameSection),
        (b"remove-section", ConfigForm::RemoveSection),
        (b"edit", ConfigForm::Edit),
    ];
    for (spelling, form) in words {
        if spelling == word {
            // `Some(x)` is the "present" case of `Option`.
            return Some(form);
        }
    }
    // `None` is the "absent" case of `Option`.
    return None;
}

/// What:
///  The complete option table of one form,
///  assembled from Git's groups in Git's
///       order.
///  `Vec<OptionSpec>` is an owned list,
///  built once per parse.
/// Why:
///   The completion control compares each assembled table with the binary's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function configTable(form: ConfigForm): OptionSpec[] { return groups[form].flat(); }
/// ```
pub fn config_table(form: ConfigForm) -> Vec<OptionSpec> {
    // `&[&[OptionSpec]]` borrows a list of borrowed groups; `match` picks one per form.
    let groups: &[&[OptionSpec]] = match form {
        ConfigForm::List => &[LOCATION_ROWS, DISPLAY_ROWS, TYPE_ROWS, LIST_OTHER_ROWS],
        ConfigForm::Get => &[
            LOCATION_ROWS,
            GET_FILTER_ROWS,
            DISPLAY_ROWS,
            TYPE_ROWS,
            GET_OTHER_ROWS,
        ],
        ConfigForm::Set => &[LOCATION_ROWS, TYPE_ROWS, SET_ROWS],
        ConfigForm::Unset => &[LOCATION_ROWS, UNSET_ROWS],
        ConfigForm::RenameSection | ConfigForm::RemoveSection | ConfigForm::Edit => {
            &[LOCATION_ROWS]
        }
        ConfigForm::Legacy => &[
            LOCATION_ROWS,
            LEGACY_ACTION_ROWS,
            DISPLAY_ROWS,
            TYPE_ROWS,
            LEGACY_OTHER_ROWS,
        ],
    };
    let mut table: Vec<OptionSpec> = Vec::<OptionSpec>::new();
    for group in groups {
        // `extend_from_slice` copies the group's rows onto the end of the table.
        table.extend_from_slice(group);
    }
    return table;
}

/// What:
///  Parse the region after `git config`.
///  `Result<A, B>` is "either success `A` or
///       failure `B`";
///  `wrapper_flags` are the caller's wrapper-only spellings.
/// Why:
///   Git looks at the first token only to pick the form.
///  A subcommand word selects its
///       table;
///  a leading `--` is dropped by that first look,
///  so the form without a
///       subcommand then reads the rest as options again (config.c:1643-1660).
///  Leading
///       wrapper flags are skipped first,
///  because the wrapper removes them before Git
///       runs:
///  `git config --cli-git-keep-going list` reaches Git as `git config list`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseConfigRegion(region: string[], wrapperFlags: string[]): ConfigRegion; // throws OptionError
/// ```
pub fn parse_config_region(
    region: &[OsString],
    wrapper_flags: &[&[u8]],
) -> Result<ConfigRegion, OptionError> {
    let mut wrapper: Vec<WrapperOccurrence> = Vec::<WrapperOccurrence>::new();
    // Region index of the first token that is not a wrapper flag.
    let mut first: usize = 0;
    while first < region.len() {
        let token: &[u8] = region[first].as_encoded_bytes();
        // Index of the matching wrapper flag; the list length means "none".
        let mut flag: usize = 0;
        while flag < wrapper_flags.len() && wrapper_flags[flag] != token {
            flag += 1;
        }
        if flag == wrapper_flags.len() {
            break;
        }
        wrapper.push(WrapperOccurrence { flag, token: first });
        first += 1;
    }
    let mut form: ConfigForm = ConfigForm::Legacy;
    // Region index of the first token the form's own parser reads.
    let mut offset: usize = first;
    // `if let Some(found) = ...` runs only when a token follows the wrapper flags.
    if let Some(found) = region.get(first) {
        let word: &[u8] = found.as_encoded_bytes();
        if let Some(named) = form_of_word(word) {
            form = named;
            offset = first + 1;
        } else if word == b"--" {
            offset = first + 1;
        }
    }
    let mode: ParseMode = if form == ConfigForm::List || form == ConfigForm::Edit {
        DEFAULT_MODE
    } else {
        CONFIG_STOP_MODE
    };
    let table: Vec<OptionSpec> = config_table(form);
    // `&region[offset..]` borrows the tokens from `offset` to the end.
    let outcome: Result<ParsedOptions, OptionError> =
        parse_options(&region[offset..], table.as_slice(), mode, wrapper_flags);
    // `match` on a `Result` handles both cases; token indexes are moved back to the region.
    let parsed: ParsedOptions = match outcome {
        Ok(found) => found,
        Err(refusal) => {
            // `Err(x)` is the failure case of `Result`.
            return Err(OptionError {
                kind: refusal.kind,
                token: refusal.token + offset,
            });
        }
    };
    for occurrence in &parsed.wrapper {
        wrapper.push(WrapperOccurrence {
            flag: occurrence.flag,
            token: occurrence.token + offset,
        });
    }
    // `Ok(x)` is the success case of `Result`.
    return Ok(ConfigRegion {
        form,
        global: is_enabled(&parsed, GLOBAL),
        system: is_enabled(&parsed, SYSTEM),
        lists: form == ConfigForm::List || (form == ConfigForm::Legacy && is_stated(&parsed, LIST)),
        wrapper,
    });
}

/// Forms,
///  scopes and option positions,
///  with real Git 2.56.0 controls.
#[cfg(test)]
#[path = "command_config_tests.rs"]
mod tests;
