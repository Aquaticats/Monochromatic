//! What:
//!  The decision,
//! one tracked path at a time,
//! whether a clone of the source's file may stand for the bytes Git's own checkout would write,
//! and the words of the reason when it may not.
//! Why:
//!  A clone copies the source's working file and installs the source's index,
//! so the new worktree holds whatever bytes the source held.
//!  Git's checkout writes the conversion of the blob instead:
//! `ident` expands,
//! a line-ending rule can add carriage returns,
//! an encoding or a filter can rewrite the whole file,
//! and where a rule can only drop carriage returns on the way into the index,
//! a file whose recorded size equals its blob's holds the blob's bytes
//!  (the drop shrinks the file,
//! so equal sizes leave nothing dropped).
//! The rules here are Git's own,
//! from `convert.c` (`convert_attrs`,
//!  `git_path_check_crlf`,
//! `output_eol`,
//! `will_convert_lf_to_crlf`) at Git 2.56:
//! `text` decides first and the legacy `crlf` attribute is its fallback,
//! `eol` refines every action but the binary one (an action nothing named included),
//! and only an action still unnamed falls to `core.autocrlf`;
//! a rule that may convert answers "Git writes it",
//! so the error is always the expensive one and never a wrong worktree.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const rules = conversionRulesFromConfig(entries);
//! const verdict = decide(attributesOfPath, rules, { isLink, fileSize, blobSize }); // 'carry' | 'git writes'
//! ```

use crate::config_snapshot::ConfigEntry;
use crate::repo_read_config_text::parse_bool;

//region attribute values and the line-ending action Git makes of them

/// How Git reads the `text` attribute (`git_path_check_crlf`).
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum TextAttribute {
    /// No line-ending rule at all.
    #[default]
    Unspecified,
    /// `text` alone:
    /// text,
    /// with the ending `core.autocrlf` or `core.eol` names.
    Set,
    /// `-text`,
    /// or the literal value `unset`:
    /// never convert,
    /// or an action Git does not name.
    /// `git check-attr` prints both as `unset`,
    /// so the fence reads it as Git's undefined action,
    /// which is exact for `text=unset` and only more careful for `-text` (`git_path_check_crlf`).
    Unset,
    /// `text=input`:
    /// normalize on the way in,
    /// never add an ending on the way out.
    Input,
    /// `text=auto`:
    /// decide by the bytes.
    Auto,
    /// A value Git does not know,
    /// which it reads as no rule.
    Other,
}

/// How Git reads the `eol` attribute (`git_path_check_eol`).
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum EolAttribute {
    /// No `eol` attribute.
    #[default]
    Unspecified,
    /// `eol=lf`.
    Lf,
    /// `eol=crlf`.
    Crlf,
}

/// What `core.autocrlf` says,
/// in Git's own three readings.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AutoCrlf {
    /// Unset or false:
    /// a worktree keeps its own endings.
    False,
    /// `input`:
    /// normalize on the way in,
    /// never add an ending on the way out.
    Input,
    /// `true`:
    /// add the platform's ending on the way out.
    True,
}

/// What `core.eol` says.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EolSetting {
    /// Unset,
    /// which means the platform's ending.
    Unset,
    /// `lf`.
    Lf,
    /// `crlf`.
    Crlf,
}

/// Which ending Git would write for a blob under a line-ending action.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OutputEnding {
    /// No conversion at all (`output_eol` gives `EOL_UNSET`).
    Unset,
    /// The checkout never adds a carriage return.
    Lf,
    /// The checkout may add a carriage return.
    Crlf,
}

/// The action `convert_attrs` makes of a path's attributes and the configuration.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CrlfAction {
    /// Nothing has named an action yet (`CRLF_UNDEFINED`),
    /// which `core.autocrlf` decides.
    Undefined,
    /// Never convert,
    /// on either side.
    Binary,
    /// `text` with the platform's ending.
    Text,
    /// `text` with `eol=lf`,
    /// or `text=input`.
    TextInput,
    /// `text` with `eol=crlf`.
    TextCrlf,
    /// `text=auto` with the platform's ending.
    Auto,
    /// `text=auto eol=lf`.
    AutoInput,
    /// `text=auto eol=crlf`.
    AutoCrlf,
}

/// The conversion settings that bear on the line ending of every path.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ConversionRules {
    /// What `core.autocrlf` says.
    pub auto_crlf: AutoCrlf,
    /// What `core.eol` says.
    pub core_eol: EolSetting,
}

impl ConversionRules {
    /// Whether `text` alone would add carriage returns here (`text_eol_is_crlf`).
    /// Why a platform is not asked:
    ///  the ladder serves Unix only,
    /// where the platform's ending is the line feed.
    fn text_eol_is_crlf(&self) -> bool {
        return match self.auto_crlf {
            AutoCrlf::True => true,
            AutoCrlf::Input => false,
            AutoCrlf::False => self.core_eol == EolSetting::Crlf,
        };
    }

    /// What:
    ///  The action Git makes of one path's attributes (`convert_attrs`).
    /// Why:
    ///   `text` is read first and the legacy `crlf` attribute is its fallback,
    /// `eol` refines any action but the binary one,
    /// and an action still unnamed falls to `core.autocrlf`.
    /// A bare `eol=crlf` therefore makes Git write carriage returns,
    /// and a bare `eol=lf` makes the clean side strip them,
    /// with no `text` attribute anywhere
    ///  (`convert.c`,
    /// `git_path_check_crlf` and the `eol` branch of `convert_attrs`).
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function actionOf(text, crlf, eol, rules): CrlfAction;
    /// ```
    pub fn action_of(
        &self,
        text: TextAttribute,
        crlf: TextAttribute,
        eol: EolAttribute,
    ) -> CrlfAction {
        // Git reads `text`, and only an action that is still undefined falls back to the legacy `crlf` attribute.
        let primary: CrlfAction = crlf_action_of(text);
        let named: CrlfAction = if primary == CrlfAction::Undefined {
            crlf_action_of(crlf)
        } else {
            primary
        };
        // `eol` refines every action but the binary one, an action nothing named included.
        let with_eol: CrlfAction = match (named, eol) {
            (CrlfAction::Binary, _) => CrlfAction::Binary,
            (CrlfAction::Auto, EolAttribute::Lf) => CrlfAction::AutoInput,
            (CrlfAction::Auto, EolAttribute::Crlf) => CrlfAction::AutoCrlf,
            (_, EolAttribute::Lf) => CrlfAction::TextInput,
            (_, EolAttribute::Crlf) => CrlfAction::TextCrlf,
            (action, EolAttribute::Unspecified) => action,
        };
        return match with_eol {
            // `text` alone takes the platform's ending from the configuration (`text_eol_is_crlf`).
            CrlfAction::Text => {
                if self.text_eol_is_crlf() {
                    CrlfAction::TextCrlf
                } else {
                    CrlfAction::TextInput
                }
            }
            // An action nothing named is `core.autocrlf`'s to decide.
            CrlfAction::Undefined => match self.auto_crlf {
                AutoCrlf::False => CrlfAction::Binary,
                AutoCrlf::Input => CrlfAction::AutoInput,
                AutoCrlf::True => CrlfAction::AutoCrlf,
            },
            action => action,
        };
    }

    /// Which ending the checkout of a blob under this action would write (`output_eol`).
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function outputEnding(action, rules): OutputEnding;
    /// ```
    pub fn output_ending(
        &self,
        action: CrlfAction,
    ) -> OutputEnding {
        return match action {
            CrlfAction::Binary => OutputEnding::Unset,
            // Git's own `output_eol` answers the carriage return here, and `action_of` never leaves this action.
            CrlfAction::Undefined => OutputEnding::Crlf,
            CrlfAction::TextInput | CrlfAction::AutoInput => OutputEnding::Lf,
            CrlfAction::TextCrlf | CrlfAction::AutoCrlf => OutputEnding::Crlf,
            CrlfAction::Text | CrlfAction::Auto => {
                if self.text_eol_is_crlf() {
                    OutputEnding::Crlf
                } else {
                    OutputEnding::Lf
                }
            }
        };
    }
}

/// What:
///  The action one line-ending attribute value names on its own (`git_path_check_crlf`).
/// Why:
///   `text` and the legacy `crlf` attribute are read by the same function in Git,
/// so `crlf=input` means what `text=input` means,
/// and only a value Git does not name leaves the action undefined.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function crlfActionOf(value): CrlfAction;
/// ```
fn crlf_action_of(value: TextAttribute) -> CrlfAction {
    return match value {
        TextAttribute::Set => CrlfAction::Text,
        // The printed token is either `-text` (`CRLF_BINARY`) or the literal value `unset` (`CRLF_UNDEFINED`), and Git's
        // two readings differ: the undefined one is refined by `eol` and then decided by `core.autocrlf`. The fence
        // cannot tell them apart, so it takes the undefined reading: exact for `text=unset`, and only more careful where
        // Git was given `-text` (`convert_attrs` at 2.56.1).
        TextAttribute::Unset => CrlfAction::Undefined,
        TextAttribute::Input => CrlfAction::TextInput,
        TextAttribute::Auto => CrlfAction::Auto,
        TextAttribute::Unspecified | TextAttribute::Other => CrlfAction::Undefined,
    };
}

//endregion

//region the rules of one path

/// What Git's attributes say about one path,
/// reduced to the ones that can change its bytes.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct PathAttributes {
    /// The `text` attribute.
    pub text: TextAttribute,
    /// The legacy `crlf` attribute,
    /// which Git reads when `text` names no action.
    pub crlf: TextAttribute,
    /// The `eol` attribute.
    pub eol: EolAttribute,
    /// Whether a `filter` attribute selects a program.
    pub filter: bool,
    /// Whether `ident` is set.
    pub ident: bool,
    /// Whether `working-tree-encoding` is set.
    pub encoding: bool,
}

/// What the fence decides for one path.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FenceVerdict {
    /// The clone's bytes are the ones Git's checkout writes.
    Carry,
    /// Git must write the path;
    /// the words say which rule can change the bytes.
    WriteThroughGit(String),
}

/// What:
///  Decide one tracked path.
/// Why:
///   A link is written as its target and never converted;
/// a filter,
/// `ident` and an encoding can rewrite the bytes;
/// an ending Git may add is a conversion the clone's bytes do not have;
/// and wherever the checkout writes the blob unchanged,
/// the file must have the blob's own size:
/// a rule that only strips carriage returns shrinks a file it touches,
/// and a path with no rule at all holds its blob's size only when it holds its bytes
///  (a file the source's index vouched for can still be a CRLF form an earlier rule wrote).
/// `blob_size` is `None` for a path the tree does not list,
/// which is a path Git writes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function decide(attributes, rules, facts): FenceVerdict;
/// ```
pub fn decide(
    attributes: &PathAttributes,
    rules: &ConversionRules,
    is_link: bool,
    file_size: u64,
    blob_size: Option<u64>,
) -> FenceVerdict {
    if is_link {
        return FenceVerdict::Carry;
    }
    if attributes.text == TextAttribute::Other {
        return FenceVerdict::WriteThroughGit(String::from(
            "its text attribute holds a value whose action Git does not name",
        ));
    }
    if attributes.filter {
        return FenceVerdict::WriteThroughGit(String::from(
            "a filter attribute selects a program that may rewrite its bytes",
        ));
    }
    if attributes.encoding {
        return FenceVerdict::WriteThroughGit(String::from(
            "a working-tree-encoding attribute changes its bytes in both directions",
        ));
    }
    if attributes.ident {
        return FenceVerdict::WriteThroughGit(String::from(
            "an ident attribute expands the keyword in the checked-out bytes",
        ));
    }
    let action: CrlfAction = rules.action_of(attributes.text, attributes.crlf, attributes.eol);
    if rules.output_ending(action) == OutputEnding::Crlf {
        return FenceVerdict::WriteThroughGit(String::from(
            "a line-ending rule may add carriage returns to the checked-out bytes",
        ));
    }
    return match blob_size {
        Some(size) if size == file_size => FenceVerdict::Carry,
        Some(_) => FenceVerdict::WriteThroughGit(String::from(
            "its recorded size differs from its blob's, so its bytes are not the blob's",
        )),
        None => FenceVerdict::WriteThroughGit(String::from("the tree lists no blob for it")),
    };
}

/// What:
///  Whether the path's checkout can write the blob unchanged,
/// so the blob's size decides whether the source's file is the blob.
/// Why:
///   A rule whose checkout never adds a carriage return and whose clean side only drops them
/// leaves equal sizes as the proof that nothing was dropped;
/// a path with no rule at all needs the same proof,
/// because a file can hold a CRLF form written under a rule that is gone today
///  (`core.autocrlf` unset after the add,
/// a tracked attributes file deleted by a later commit).
/// A filter,
/// an encoding or `ident` answers the fence before the ending is looked at.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const needsSize = (attributes, rules) => !converting(attributes) && outputEnding(actionOf(attributes)) !== 'crlf';
/// ```
pub fn needs_blob_size(
    attributes: &PathAttributes,
    rules: &ConversionRules,
) -> bool {
    if attributes.filter
        || attributes.encoding
        || attributes.ident
        || attributes.text == TextAttribute::Other
    {
        return false;
    }
    return rules.output_ending(rules.action_of(attributes.text, attributes.crlf, attributes.eol))
        != OutputEnding::Crlf;
}

/// What:
///  Whether a rule of the path's own converts its bytes on the way out,
/// with no size read at all.
/// Why:
///   These are the paths a fence must hand to Git whatever their sizes say,
/// so their count is the one a decline rule can price before reading a blob
///  (`fence_the_attributes`,
/// which decides the decline before anything is copied).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const converts = (attributes, rules) => filter || ident || encoding || otherText || outputEnding(actionOf(attributes)) === 'crlf';
/// ```
pub fn checkout_converts(
    attributes: &PathAttributes,
    rules: &ConversionRules,
) -> bool {
    if attributes.filter
        || attributes.encoding
        || attributes.ident
        || attributes.text == TextAttribute::Other
    {
        return true;
    }
    return rules.output_ending(rules.action_of(attributes.text, attributes.crlf, attributes.eol))
        == OutputEnding::Crlf;
}

//endregion

//region reading the configuration and Git's answers

/// What:
///  The last value the listing gives `key`,
/// as its own bytes,
/// or nothing when no entry has the key,
/// and as a missing value when the entry is written without one.
/// Why:
///   Git reads later entries over earlier ones,
/// so the search runs from the end,
/// and a value is the bytes the file holds,
///  which a decoding would shorten;
/// a key written without `=` is a bare name to Git's boolean parser,
/// not an absent key.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const lastValue = (entries, key) => entries.findLast((entry) => entry.key === key);
/// ```
fn last_value<'entries>(
    entries: &'entries [ConfigEntry],
    key: &str,
) -> Option<Option<&'entries [u8]>> {
    for entry in entries.iter().rev() {
        if entry.key == key {
            return Some(entry.value.as_deref());
        }
    }
    return None;
}

/// What:
///  The conversion settings of a configuration listing.
/// Why:
///   `core.autocrlf` alone converts every text file where it is true,
/// and `core.eol` decides what `text` alone adds.
/// A value Git does not know is read as the setting's default,
///  because a fence may only be more careful than Git.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function conversionRulesFromConfig(entries): ConversionRules;
/// ```
pub fn conversion_rules_from_config(entries: &[ConfigEntry]) -> ConversionRules {
    // `[core]` with a line holding only `autocrlf` is true to Git: `git_parse_maybe_bool_text(NULL)` is 1 and
    // `core.autocrlf` goes through `git_config_bool` (`parse.c`, `config.c` at 2.56.1). The fence read the missing value
    // as an absent key and carried the files before the gate re-check's `p01`.
    let auto_crlf: AutoCrlf = match last_value(entries, "core.autocrlf") {
        None => AutoCrlf::False,
        Some(value) if value.is_some_and(|held| return held.eq_ignore_ascii_case(b"input")) => {
            AutoCrlf::Input
        }
        Some(value) => match parse_bool(value) {
            Some(true) => AutoCrlf::True,
            Some(false) => AutoCrlf::False,
            None => AutoCrlf::True,
        },
    };
    let core_eol: EolSetting = match last_value(entries, "core.eol").flatten() {
        Some(value) if value.eq_ignore_ascii_case(b"lf") => EolSetting::Lf,
        Some(value) if value.eq_ignore_ascii_case(b"crlf") => EolSetting::Crlf,
        _ => EolSetting::Unset,
    };
    return ConversionRules {
        auto_crlf,
        core_eol,
    };
}

/// The value bytes `git check-attr` gives an unset attribute.
const UNSET_VALUE: &[u8] = b"unset";

/// The value bytes `git check-attr` gives a set attribute.
const SET_VALUE: &[u8] = b"set";

/// What:
///  The attributes of one path,
/// from the triples `git check-attr --all -z --stdin` prints.
/// Why:
///   Git resolves the precedence of the tracked,
/// `info/attributes`,
/// global,
/// system and worktree files itself,
/// and expands macros,
///  so the fence reads Git's answers instead of matching patterns again.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function applyTriple(attributes, name, value): void;
/// ```
pub fn apply_attribute(
    attributes: &mut PathAttributes,
    name: &[u8],
    value: &[u8],
) {
    match name {
        b"text" => {
            attributes.text = match value {
                SET_VALUE => TextAttribute::Set,
                UNSET_VALUE => TextAttribute::Unset,
                b"input" => TextAttribute::Input,
                b"auto" => TextAttribute::Auto,
                _ => TextAttribute::Other,
            };
        }
        b"eol" => {
            attributes.eol = match value {
                b"lf" => EolAttribute::Lf,
                b"crlf" => EolAttribute::Crlf,
                _ => EolAttribute::Unspecified,
            };
        }
        // The legacy `crlf` attribute, which Git reads with the same function as `text` when `text` names no action.
        b"crlf" => {
            attributes.crlf = match value {
                SET_VALUE => TextAttribute::Set,
                UNSET_VALUE => TextAttribute::Unset,
                b"input" => TextAttribute::Input,
                b"auto" => TextAttribute::Auto,
                _ => TextAttribute::Unspecified,
            };
        }
        b"filter" => {
            // A value, `set` or `unset`: `set` names no program Git runs, so only a name selects one.
            attributes.filter = value != UNSET_VALUE && value != SET_VALUE;
        }
        b"ident" => {
            attributes.ident = value == SET_VALUE;
        }
        b"working-tree-encoding" => {
            attributes.encoding = value != UNSET_VALUE && !value.is_empty();
        }
        _ => {}
    }
}

//endregion

#[cfg(test)]
#[path = "worktree_clone_conversion_rules_tests.rs"]
mod tests;
