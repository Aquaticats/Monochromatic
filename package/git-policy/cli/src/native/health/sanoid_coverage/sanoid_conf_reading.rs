//! What:
//!  A reading of a `sanoid.conf` text:
//!  its sections in file order,
//!  each setting with the line it stands on,
//!  and the reasons a text cannot be proved to be what sanoid reads.
//! Why:
//!  The coverage check compares a person's configuration with the decided
//!  coverage,
//!  and the order of the sections is part of that comparison,
//!  because sanoid copies a recursive section's settings onto every child
//!  without checking for a child section of its own
//!  (`/usr/bin/sanoid:1110`).
//!  A text this reading refuses is answered unknown and never as a pass,
//!  because a guess here would claim a coverage nobody proved.
//!  The readings follow what the installed `Config::IniFiles` 3.000003 and
//!  sanoid 2.3.0 do on this host,
//!  measured on 2026-11-25:
//!  keys are case-sensitive,
//!  a repeated key keeps its first value,
//!  values are trimmed at both ends,
//!  and only a line whose first non-space character is `#` or `;` is a
//!  comment.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const config = readSanoidConf(text); // { sections: [{ dataset, line, settings, duplicates }] }
//! ```

use super::sanoid_section::{AUTOSNAP_SETTING, SanoidSetting, is_false_word};

/// What:
///  One key a section writes more than once.
/// Why:
///  Sanoid takes the first value and warns (`/usr/bin/sanoid:974` to `:977`),
///  so the reading keeps the first and names the lines,
///  letting the report say a person's second line does nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type DuplicateKey = { key: string; firstLine: number; line: number };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DuplicateKey {
    /// The key the section writes twice.
    pub key: String,
    /// The line sanoid reads.
    pub first_line: usize,
    /// The line sanoid ignores.
    pub line: usize,
}

/// What:
///  One setting together with the line it stands on.
/// Why:
///  A finding names the line a person edits,
///  and the rendering type carries no line because it is also the text a
///  correction adds.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ReadSetting = { setting: SanoidSetting; line: number };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReadSetting {
    /// The key and the value as written.
    pub setting: SanoidSetting,
    /// The line the setting stands on.
    pub line: usize,
}

/// What:
///  One section of a person's configuration.
/// Why:
///  The line keeps a finding able to name where the section it moved stands,
///  and the duplicates are kept so the report can name what sanoid ignores.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ReadSection = { dataset: string; line: number; settings: ReadSetting[]; duplicates: DuplicateKey[] };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReadSection {
    /// The dataset the header names.
    pub dataset: String,
    /// The line the header stands on.
    pub line: usize,
    /// The settings,
    /// each key once,
    /// in the order they were written.
    pub settings: Vec<ReadSetting>,
    /// Every key the section writes again.
    pub duplicates: Vec<DuplicateKey>,
}

/// What:
///  The sections of one configuration text in the order sanoid reads them.
/// Why:
///  File order is the behaviour the exclusions depend on,
///  so it is the reading's first fact and never a set.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ReadConfig = { sections: ReadSection[] };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReadConfig {
    /// The sections in file order.
    pub sections: Vec<ReadSection>,
}

/// What:
///  Why a text cannot be proved to be the configuration sanoid reads.
/// Why:
///  Each refusal names the line a person edits,
///  and the report answers unknown rather than finding,
///  because this build proves nothing about a text it did not read
///  completely.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ConfigRefusal = { kind: 'line'; line: number; problem: string } | { kind: 'duplicate-section'; dataset: string; ... };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ConfigRefusal {
    /// One line is not a section header,
    /// a setting or a comment.
    Line {
        /// The line the reading stopped at.
        line: usize,
        /// What is wrong with it.
        problem: String,
    },
    /// One dataset has two sections,
    /// and what sanoid does with the later one is not proved here.
    DuplicateSection {
        /// The dataset both sections name.
        dataset: String,
        /// The first section's header line.
        first_line: usize,
        /// The later section's header line.
        again_line: usize,
    },
}

impl ConfigRefusal {
    /// The sentence a report prints for this refusal.
    pub fn reason(&self) -> String {
        return match self {
            ConfigRefusal::Line { line, problem } => format!("line {line} is not a section header, a setting or a comment: {problem}"),
            ConfigRefusal::DuplicateSection { dataset, first_line, again_line } => {
                format!("the dataset {dataset} has a section on line {first_line} and another on line {again_line}, and this build does not prove what sanoid does with the later one")
            }
        };
    }
}

/// What:
///  What one section says about one key.
/// Why:
///  A key spelled with other capitals is not the key sanoid reads,
///  so the answer keeps it apart from an absent key:
///  the first is a text this build cannot prove,
///  the second is a value the template decides.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type SettingLookup = { kind: 'found'; value: string; line: number } | { kind: 'absent' } | { kind: 'case-variant'; spelling: string; line: number };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SettingLookup {
    /// The section writes the key.
    Found {
        /// The value as written,
        /// trimmed of its ends.
        value: String,
        /// The line the key stands on.
        line: usize,
    },
    /// The section does not write the key.
    Absent,
    /// The section writes the key with other capitals,
    /// which sanoid does not read as the key.
    CaseVariant {
        /// The spelling the section uses.
        spelling: String,
        /// The line the key stands on.
        line: usize,
    },
}

impl ReadSection {
    /// The value the section writes for one key,
    /// or nothing when it writes none.
    pub fn written_value(&self, key: &str) -> Option<&str> {
        return self.settings.iter().find(|entry| return entry.setting.key == key).map(|entry| return entry.setting.value.as_str());
    }

    /// What the section says about one key.
    pub fn setting(&self, key: &str) -> SettingLookup {
        for entry in &self.settings {
            if entry.setting.key == key {
                return SettingLookup::Found { value: entry.setting.value.clone(), line: entry.line };
            }
        }
        for entry in &self.settings {
            if entry.setting.key.eq_ignore_ascii_case(key) {
                return SettingLookup::CaseVariant { spelling: entry.setting.key.clone(), line: entry.line };
            }
        }
        return SettingLookup::Absent;
    }

    /// Whether the section turns its datasets' snapshots off with a word sanoid
    /// reads as false.
    pub fn snapshots_off(&self) -> bool {
        return match self.setting(AUTOSNAP_SETTING) {
            SettingLookup::Found { value, .. } => is_false_word(value.as_str()),
            SettingLookup::Absent | SettingLookup::CaseVariant { .. } => false,
        };
    }
}

impl ReadConfig {
    /// The position of one dataset's section in file order.
    pub fn position(&self, dataset: &str) -> Option<usize> {
        return self.sections.iter().position(|section| return section.dataset == dataset);
    }

    /// One dataset's section.
    pub fn section(&self, dataset: &str) -> Option<&ReadSection> {
        return self.sections.iter().find(|section| return section.dataset == dataset);
    }
}

/// What:
///  The sections of a configuration text in file order.
/// Why:
///  The coverage check needs one reading of one text,
///  and this function is the only place that grammar is decided.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// readSanoidConf(text); // throws ConfigRefusal
/// ```
pub fn read_sanoid_conf(text: &str) -> Result<ReadConfig, ConfigRefusal> {
    let mut sections: Vec<ReadSection> = Vec::new();
    for (index, raw) in text.split('\n').enumerate() {
        let line: usize = index + 1;
        let trimmed: &str = raw.trim_end_matches('\r').trim();
        if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with(';') {
            continue;
        }
        if let Some(inner) = trimmed.strip_prefix('[') {
            let Some(name) = inner.strip_suffix(']') else {
                return Err(ConfigRefusal::Line { line, problem: String::from("a section header must close its bracket") });
            };
            let dataset: &str = name.trim();
            if dataset.is_empty() {
                return Err(ConfigRefusal::Line { line, problem: String::from("a section header must name a dataset") });
            }
            if let Some(first) = sections.iter().find(|section| return section.dataset == dataset) {
                return Err(ConfigRefusal::DuplicateSection { dataset: String::from(dataset), first_line: first.line, again_line: line });
            }
            sections.push(ReadSection { dataset: String::from(dataset), line, settings: Vec::new(), duplicates: Vec::new() });
            continue;
        }
        let Some((written_key, written_value)) = trimmed.split_once('=') else {
            return Err(ConfigRefusal::Line { line, problem: String::from("a setting must be written key = value") });
        };
        let key: &str = written_key.trim();
        if key.is_empty() {
            return Err(ConfigRefusal::Line { line, problem: String::from("a setting must name a key") });
        }
        let Some(section) = sections.last_mut() else {
            return Err(ConfigRefusal::Line { line, problem: String::from("the setting stands before the first section header") });
        };
        let value: &str = written_value.trim();
        if let Some(first) = section.settings.iter().find(|entry| return entry.setting.key == key) {
            section.duplicates.push(DuplicateKey { key: String::from(key), first_line: first.line, line });
            continue;
        }
        section.settings.push(ReadSetting { setting: SanoidSetting { key: String::from(key), value: String::from(value) }, line });
    }
    return Ok(ReadConfig { sections });
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "sanoid_conf_reading_tests.rs"]
mod tests;
