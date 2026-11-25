//! What:
//!  The values a setup run fills into the unit texts,
//!  and one file of the install plan:
//!  where it is written and the whole text written there.
//! Why:
//!  Every text is built from the same few facts,
//!  and a value the kernel or systemd would read as something else must be
//!  refused before a unit file exists:
//!  a checkout path carrying a space is two words on a line whose first word is
//!  a program,
//!  and a drop-in folder under a name no unit has is read by nothing.
//!  The refusal names the input and the way forward, so a setup run fails where
//!  a person can fix the input rather than at the next boot
//!  (`doc/decision/cli-git-zfs-step-19-boot-mount-provider-and-ordering-choices.md`,
//!  V3 and V7).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type Facts = { checkout: '/home/user/Monochromatic', rootFile: '/usr/libexec/…', keyUnit: 'zfs-load-key@…' };
//! type File = { path: '/etc/systemd/system/…', text: '[Unit]\n…' };
//! ```

use std::path::{Path, PathBuf};

/// What:
///  Why one unit text was not built.
/// Why:
///  each cause names the value a person gave,
///  because the repair is to change that value and run the setup again.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type UnitTextRefusal = { kind: 'path-not-text' | 'not-one-word' | 'slug'; what, value };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UnitTextRefusal {
    /// A path the text must name is not UTF-8 or carries a byte systemd would
    /// split on.
    PathNotOneWord {
        /// Which value it is,
        /// as the sentence names it.
        what: &'static str,
        /// The path as the caller gave it.
        path: PathBuf,
    },
    /// A word the text must name is empty or carries whitespace,
    /// a quote or a backslash.
    WordNotOneWord {
        /// Which value it is.
        what: &'static str,
        /// The word as the caller gave it.
        word: String,
    },
    /// A nested dataset's unit slug is empty or carries a character a unit name
    /// does not take.
    Slug {
        /// The slug as the caller gave it.
        slug: String,
    },
    /// A path is not absolute,
    /// so the unit would name a path relative to whatever directory a reader
    /// stood in.
    PathNotAbsolute {
        /// Which value it is.
        what: &'static str,
        /// The path as the caller gave it.
        path: PathBuf,
    },
    /// A line the plan needs is absent from the text the plan is built from,
    /// which would install a file whose companion cannot work with it.
    MissingLine {
        /// The line,
        /// as the sentence names it.
        line: &'static str,
        /// The unit whose text lacks it.
        unit: &'static str,
    },
}

impl UnitTextRefusal {
    /// The refusal in words,
    /// naming the value and what to give instead.
    pub fn message(&self) -> String {
        return match self {
            UnitTextRefusal::PathNotOneWord { what, path } => format!(
                "{what} is `{}`, which a systemd line cannot carry: a path with a space, a tab, a newline, a quote, a backslash or bytes that are not text becomes more than one word or another word. Nothing was written. Ways forward: give a path with none of those characters; the route's checkout and the root-owned file are both ordinary paths on this machine.",
                path.display()
            ),
            UnitTextRefusal::WordNotOneWord { what, word } => format!(
                "{what} is `{word}`, which a systemd line cannot carry: a word with a space, a tab, a newline, a quote or a backslash is split or regrouped by systemd. Nothing was written. Ways forward: give the unit's name as `systemctl` prints it, alone."
            ),
            UnitTextRefusal::Slug { slug } => format!(
                "the nested dataset's unit slug `{slug}` is not one a unit name takes: give its short name in lowercase letters, digits and single hyphens, as the dataset's own last name component reads."
            ),
            UnitTextRefusal::PathNotAbsolute { what, path } => format!(
                "{what} is `{}`, which is not an absolute path, so a unit line would resolve it against the directory a reader stood in. Nothing was written. Ways forward: give the full path, beginning with `/`.",
                path.display()
            ),
            UnitTextRefusal::MissingLine { line, unit } => format!(
                "{unit} carries no `{line}` line, and the propagation pair asks the gate for a reload: without that line the request is a restart, which stops the gate, runs its stop pass and unmounts every attached clone. Nothing was written. Ways forward: install the gate text this build writes, which carries the line, and add the propagation pair only beside it."
            ),
        };
    }
}

/// What:
///  The facts the gate, the provider drop-in and the propagation pair are
///  built from.
/// Why:
///  the checkout is the path the gate's `RequiresMountsFor=` names and the mount
///  unit the propagation pair binds to;
///  the root file is the program every `Exec` line starts;
///  and the key unit is the generator's own where a machine has one,
///  whose absence removes the two lines that would otherwise wait forever.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const facts = { checkout: '/home/user/Monochromatic', rootFile: '/usr/libexec/…', keyUnit: null };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CheckoutBootFacts {
    /// The checkout path,
    /// absolute.
    pub checkout: PathBuf,
    /// The root-owned regular file the root modes are installed as,
    /// absolute.
    pub root_file: PathBuf,
    /// The generator's key-loading unit for the checkout's encryption root,
    /// present only on a machine that has one.
    pub key_unit: Option<String>,
}

impl CheckoutBootFacts {
    /// The facts with every value checked,
    /// so each text can be built without a second refusal path.
    ///
    /// # Errors
    /// A checkout or root file that is not absolute or not one word,
    /// or a key unit that is not one word.
    pub fn checked(&self) -> Result<(), UnitTextRefusal> {
        check_path("the checkout path", self.checkout.as_path())?;
        check_path("the root-owned file's path", self.root_file.as_path())?;
        if let Some(key_unit) = &self.key_unit {
            check_unit_name("the key-loading unit's name", key_unit)?;
        }
        return Ok(());
    }
}

/// What:
///  One dataset whose mountpoint lies inside the checkout,
///  which gets a unit of its own ordered after the gate.
/// Why:
///  `zfs mount -a` mounts such a dataset before the checkout or under a shared
///  one, so V4 sets it `canmount=noauto` and V9 gives it a unit that starts only
///  after the gate proved the checkout
///  (`doc/decision/cli-git-zfs-step-19-boot-mount-provider-and-ordering-choices.md`,
///  V4 and V9).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const nested = { dataset: 'zpcachyos/nosnap/vm-builder-output', mountpoint: '/home/user/Monochromatic/package/vm-builder/cli/output', slug: 'output' };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NestedDatasetMount {
    /// The dataset's full name,
    /// as `zfs list -H -o name` prints it.
    pub dataset: String,
    /// The dataset's own mountpoint,
    /// which the unit's condition reads.
    pub mountpoint: PathBuf,
    /// The short word the unit's name carries,
    /// taken from the dataset's own last name component.
    pub slug: String,
}

impl NestedDatasetMount {
    /// The nested mount with every value checked.
    ///
    /// # Errors
    /// A dataset name that is not one word,
    /// a mountpoint that is not absolute or not one word,
    /// or a slug a unit name does not take.
    pub fn checked(&self) -> Result<(), UnitTextRefusal> {
        check_word("the nested dataset's name", self.dataset.as_str())?;
        check_path("the nested dataset's mountpoint", self.mountpoint.as_path())?;
        let mut letters = self.slug.chars();
        let first_ok: bool = matches!(letters.next(), Some(letter) if letter.is_ascii_lowercase() || letter.is_ascii_digit());
        let rest_ok: bool = self.slug.chars().all(|letter| return letter.is_ascii_lowercase() || letter.is_ascii_digit() || letter == '-');
        if !first_ok || !rest_ok {
            return Err(UnitTextRefusal::Slug { slug: self.slug.clone() });
        }
        return Ok(());
    }
}

/// What:
///  One file the setup writes:
///  its absolute path and the whole text that belongs there.
/// Why:
///  the setup and the controls write the same files,
///  and a control that rebuilt a text of its own would assert nothing about the
///  text a machine gets.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const file = { path: '/etc/systemd/system/reconcile-….service', text: '…' };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InstalledUnitFile {
    /// Where the file is written.
    pub path: PathBuf,
    /// The bytes written there,
    /// ending with one newline.
    pub text: String,
}

/// Check one path a unit line carries,
/// which must be absolute and one word.
///
/// # Errors
/// Not absolute,
/// not text,
/// or carrying a character systemd splits on.
pub fn check_path(what: &'static str, path: &Path) -> Result<(), UnitTextRefusal> {
    let _text: &str = path_text(what, path)?;
    return Ok(());
}

/// One path as the text a unit line carries,
/// after proving it is absolute and one word.
///
/// Why:
///  a caller that must write the path on a line reads it once here, so no
///  second check can disagree with the first.
///
/// # Errors
/// Not absolute,
/// not text,
/// or carrying a character systemd splits on.
pub fn path_text<'a>(what: &'static str, path: &'a Path) -> Result<&'a str, UnitTextRefusal> {
    if !path.is_absolute() {
        return Err(UnitTextRefusal::PathNotAbsolute { what, path: path.to_path_buf() });
    }
    return match crate::checkout_boot_units::boot_unit_names::path_as_one_word(path) {
        Some(text) => Ok(text),
        None => Err(UnitTextRefusal::PathNotOneWord { what, path: path.to_path_buf() }),
    };
}

/// Check one unit name a dependency line carries.
///
/// # Errors
/// Empty,
/// or carrying whitespace.
pub fn check_unit_name(what: &'static str, word: &str) -> Result<(), UnitTextRefusal> {
    if !crate::checkout_boot_units::boot_unit_names::is_one_unit_name(word) {
        return Err(UnitTextRefusal::WordNotOneWord { what, word: String::from(word) });
    }
    return Ok(());
}

/// Check one word a unit line carries.
///
/// # Errors
/// Empty,
/// or carrying whitespace,
/// a quote or a backslash.
pub fn check_word(what: &'static str, word: &str) -> Result<(), UnitTextRefusal> {
    if !crate::checkout_boot_units::boot_unit_names::is_one_word(word) {
        return Err(UnitTextRefusal::WordNotOneWord { what, word: String::from(word) });
    }
    return Ok(());
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "boot_unit_facts_tests.rs"]
mod tests;
