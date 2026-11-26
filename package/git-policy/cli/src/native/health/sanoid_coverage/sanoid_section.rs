//! What:
//!  One section of `sanoid.conf`,
//!  one setting inside it,
//!  the words sanoid reads a toggle as,
//!  and the text a person pastes into their configuration.
//! Why:
//!  The generated sections and the correction `doctor` prints must be one
//!  grammar,
//!  so a correction a person applies reads back as the section the check
//!  wanted.
//!  Sanoid reads a section's value as sanoid does,
//!  not as a casual reader would:
//!  it trims both ends,
//!  it takes the first of a repeated key,
//!  and it knows two word lists rather than Perl truth
//!  (`/usr/bin/sanoid:942` to `:943`,
//!  `:974` to `:979`,
//!  measured on this host on 2026-11-25).
//!  A value outside those lists,
//!  such as `autosnap = no # keep`,
//!  is a non-empty string and therefore snapshotted,
//!  which is why the reading here keeps values verbatim.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type SanoidSection = { dataset: string; settings: { key: string; value: string }[] };
//! renderSection(section); // "[tank/home]\n  autosnap = no\n"
//! ```

/// The key that makes a section cover every child dataset.
pub const RECURSIVE_SETTING: &str = "recursive";

/// The key that names the retention template a section uses.
pub const TEMPLATE_SETTING: &str = "use_template";

/// The key that says whether the section's datasets are snapshotted.
pub const AUTOSNAP_SETTING: &str = "autosnap";

/// The key that replaces the section's dataset with another path,
/// which is why a section that sets it cannot be reasoned about by name.
pub const PATH_SETTING: &str = "path";

/// The key that snapshots only the children of a section's dataset.
pub const PROCESS_CHILDREN_ONLY_SETTING: &str = "process_children_only";

/// The key that leaves a section's children out of its own pass.
pub const SKIP_CHILDREN_SETTING: &str = "skip_children";

/// The value the decided recursive section writes,
/// which honours each exclusion section below it.
pub const RECURSIVE_YES: &str = "yes";

/// The value that takes one `zfs snapshot -r` on the parent,
/// which no exclusion can leave a child out of.
pub const RECURSIVE_ZFS: &str = "zfs";

/// The value an exclusion section writes.
pub const AUTOSNAP_NO: &str = "no";

/// The words sanoid reads a toggle as true (`/usr/bin/sanoid:942`).
pub const TRUE_WORDS: &[&str] = &[
    "1", "true", "True", "TRUE", "yes", "Yes", "YES", "on", "On", "ON",
];

/// The words sanoid reads a toggle as false (`/usr/bin/sanoid:943`).
pub const FALSE_WORDS: &[&str] = &[
    "0", "false", "False", "FALSE", "no", "No", "NO", "off", "Off", "OFF",
];

/// What:
///  One `key = value` line inside a section.
/// Why:
///  A setting keeps the key's spelling and the value's text,
///  because sanoid is case-sensitive about keys and trims only whitespace,
///  so a check that normalized either would answer for a file sanoid reads
///  differently.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type SanoidSetting = { key: string; value: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SanoidSetting {
    /// The key as written,
    /// without surrounding spaces.
    pub key: String,
    /// The value as written,
    /// with its ends trimmed as sanoid trims them.
    pub value: String,
}

/// What:
///  One section of `sanoid.conf`:
///  the dataset it names and its settings in the order they were written.
/// Why:
///  A section is what sanoid is given,
///  and the order of the settings inside it does not decide anything,
///  while the order of the sections in the file does
///  (`/usr/bin/sanoid:1110`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type SanoidSection = { dataset: string; settings: SanoidSetting[] };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SanoidSection {
    /// The dataset the section's header names.
    pub dataset: String,
    /// The settings the section writes.
    pub settings: Vec<SanoidSetting>,
}

impl SanoidSection {
    /// A section of one dataset with the settings it writes.
    pub fn new(
        dataset: impl Into<String>,
        settings: Vec<SanoidSetting>,
    ) -> Self {
        return SanoidSection {
            dataset: dataset.into(),
            settings,
        };
    }

    /// The value the section writes for one key,
    /// or nothing when it writes none.
    pub fn value_of(
        &self,
        key: &str,
    ) -> Option<&str> {
        return self
            .settings
            .iter()
            .find(|setting| return setting.key == key)
            .map(|setting| return setting.value.as_str());
    }
}

/// What:
///  How sanoid reads one `recursive` value.
/// Why:
///  The three cases behave differently:
///  a true word honours each child's own section,
///  a value naming `zfs` takes one recursive snapshot no child can be left out
///  of,
///  and anything else leaves the section not covering its children at all,
///  so a check that answered only yes or no would report a silent
///  reconfiguration as the same answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Recursion = { kind: 'yes' } | { kind: 'zfs' } | { kind: 'off'; value?: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Recursion {
    /// A word sanoid reads as true.
    Yes,
    /// A value naming `zfs`,
    /// in any case and anywhere in the value.
    Zfs,
    /// No value,
    /// or a value sanoid does not read as true.
    Off {
        /// The value as written,
        /// or nothing when the section writes none.
        value: Option<String>,
    },
}

/// Whether a value is one sanoid reads as true.
pub fn is_true_word(value: &str) -> bool {
    return TRUE_WORDS.contains(&value);
}

/// Whether a value is one sanoid reads as false.
pub fn is_false_word(value: &str) -> bool {
    return FALSE_WORDS.contains(&value);
}

/// How sanoid reads one `recursive` value (`/usr/bin/sanoid:1075` to `:1076`).
pub fn recursion_of(value: Option<&str>) -> Recursion {
    let Some(written) = value else {
        return Recursion::Off {
            value: None,
        };
    };
    if written.to_lowercase().contains(RECURSIVE_ZFS) {
        return Recursion::Zfs;
    }
    if is_true_word(written) {
        return Recursion::Yes;
    }
    return Recursion::Off {
        value: Some(String::from(written)),
    };
}

/// What:
///  The two lines a section occupies,
///  as a person writes them.
/// Why:
///  A correction is pasted into a file sanoid reads,
///  so it must be exactly the text the check wanted to read back,
///  including the section header and the two-space indent the design's own
///  sample uses.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `[${dataset}]\n${settings.map(s => `  ${s.key} = ${s.value}\n`).join('')}`;
/// ```
pub fn render_section(section: &SanoidSection) -> String {
    let mut text: String = format!("[{}]\n", section.dataset);
    for setting in &section.settings {
        text.push_str(format!("  {} = {}\n", setting.key, setting.value).as_str());
    }
    return text;
}

/// What:
///  Several sections as one block of configuration text.
/// Why:
///  The generated file and a multi-section correction are one shape,
///  so a person applies the same text whether one section or all of them
///  drifted.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// sections.map(renderSection).join('\n');
/// ```
pub fn render_sections(sections: &[SanoidSection]) -> String {
    let mut text: String = String::new();
    for section in sections {
        if !text.is_empty() {
            text.push('\n');
        }
        text.push_str(render_section(section).as_str());
    }
    return text;
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "sanoid_section_tests.rs"]
mod tests;
