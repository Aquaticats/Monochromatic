//! What:
//!  How the `jetbrains-harper` policy rewrites the two settings files of one IDE product:
//! it reads them through the view,
//! patches the base server's configuration and schema,
//! writes one entry per scoped server,
//! and copies the base entry's options into the user-defined file.
//! Why:
//!  The rule itself decides when the policy applies and what the plan says;
//! this role only turns the settings text into the text the policy wants.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // methods of class JetbrainsHarperRule that rewrite the settings files
//! ```

/// Imports the rule,
/// the settings-file record,
/// and the copied-option names from the parent module.
use super::{
    COPIED_OPTIONS,
    JetbrainsHarperRule,
    SettingsFiles,
};
/// Imports the JSON helpers from `crate::harper_json`.
use crate::harper_json::{
    disable_linters,
    format_object,
    parse_object,
    scoped_config,
    union_excludes,
    with_schema_defaults,
};
/// Imports the XML helpers from `crate::harper_xml`.
use crate::harper_xml::{
    find_entry,
    option_value,
    parse,
};
/// Imports the entry edits from `crate::harper_xml_edit`.
use crate::harper_xml_edit::{
    ScopedUserEntry,
    upsert_language_entry,
    upsert_user_entry,
};
/// Imports `LANGUAGE_FILE`,
/// `USER_FILE` from `crate::jetbrains_discovery`.
use crate::jetbrains_discovery::{
    LANGUAGE_FILE,
    USER_FILE,
};
/// Imports `Diagnostic` from `monochromatic_file_enforcer::diagnostic`.
use monochromatic_file_enforcer::diagnostic::Diagnostic;
/// Imports `View` from `monochromatic_file_enforcer::view`.
use monochromatic_file_enforcer::view::View;
/// Imports `Map`,
/// `Value` from `serde_json`.
use serde_json::{
    Map,
    Value,
};

/// What:
///  The settings half of the `jetbrains-harper` rule,
/// split from `kind_jetbrains_harper.rs` so each file holds one role.
/// Why:
///  A reader looking for what the policy writes reads this file,
/// and a reader looking for when it runs reads `kind_jetbrains_harper.rs`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // class JetbrainsHarperRule { readSettings(); updatedLanguage(); updatedUser(); }
/// ```
impl JetbrainsHarperRule {
    /// What:
    ///  Read both settings files of a product through the view.
    /// Why:
    ///  A product without both files is not a managed setup,
    /// like a machine without the IDE.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// readSettings(view: View, optionsDirectory: string): SettingsFiles | undefined
    /// ```
    pub(super) fn read_settings(
        &self,
        view: &mut View,
        options_directory: &str,
    ) -> Result<Option<SettingsFiles>, Diagnostic> {
        let language_path = format!("{options_directory}/{LANGUAGE_FILE}");
        let user_path = format!("{options_directory}/{USER_FILE}");
        let language_text = view.read_text(&language_path)?;
        let user_text = view.read_text(&user_path)?;
        match (language_text, user_text) {
            (Some(language), Some(user)) => {
                return Ok(Some(SettingsFiles {
                    language_path,
                    language_text: language,
                    user_path,
                    user_text: user,
                }));
            }
            _ => return Ok(None),
        }
    }

    /// The base server's patched configuration,
    /// from the configuration the language settings file holds for it.
    fn base_config(
        &self,
        settings: &SettingsFiles,
        key: &str,
    ) -> Result<Map<String, Value>, Diagnostic> {
        let document = parse(&settings.language_path, &settings.language_text)?;
        let Some(entry) = find_entry(&document, key) else {
            let message = format!(
                "{}: no language server settings entry for the base server {key}",
                settings.language_path
            );
            return Err(Diagnostic::new("harper-xml-invalid", &message)
                .with_path(&settings.language_path)
                .with_rule(&self.id));
        };
        let existing = match option_value(&document, entry, "configurationContent") {
            Some(text) => text,
            None => "{}".to_string(),
        };
        let mut config = parse_object(&existing, "configurationContent", &settings.language_path)?;
        disable_linters(&mut config, &self.disabled_linters);
        union_excludes(&mut config, &self.exclude_patterns);
        return Ok(config);
    }

    /// The schema text shared by the base and scoped entries.
    fn schema_text(
        &self,
        settings: &SettingsFiles,
        key: &str,
    ) -> Result<String, Diagnostic> {
        let document = parse(&settings.language_path, &settings.language_text)?;
        let schema_text = match find_entry(&document, key) {
            Some(entry) => option_value(&document, entry, "configurationSchemaContent"),
            None => None,
        };
        let existing = match schema_text {
            Some(text) => text,
            None => "{\"properties\":{}}".to_string(),
        };
        let schema = parse_object(
            &existing,
            "configurationSchemaContent",
            &settings.language_path,
        )?;
        return Ok(format_object(with_schema_defaults(schema)));
    }

    /// The language settings file after the policy:
    /// the base entry patched,
    /// each scoped entry written.
    pub(super) fn updated_language(
        &self,
        settings: &SettingsFiles,
        key: &str,
    ) -> Result<String, Diagnostic> {
        let base = self.base_config(settings, key)?;
        let schema = self.schema_text(settings, key)?;
        let path = settings.language_path.as_str();
        let mut text = upsert_language_entry(
            path,
            &settings.language_text,
            key,
            &format_object(base.clone()),
            &schema,
        )?;
        for server in &self.scoped_servers {
            let config = format_object(scoped_config(&base, &server.disabled_linters));
            text = upsert_language_entry(path, &text, &server.id, &config, &schema)?;
        }
        return Ok(text);
    }

    /// The user-defined settings file after the policy:
    /// each scoped entry written,
    /// with options copied from the base server's entry.
    pub(super) fn updated_user(
        &self,
        settings: &SettingsFiles,
        key: &str,
    ) -> Result<String, Diagnostic> {
        let document = parse(&settings.user_path, &settings.user_text)?;
        let mut copied: Vec<(String, String)> = Vec::new();
        if let Some(base) = find_entry(&document, key) {
            for name in COPIED_OPTIONS {
                if let Some(value) = option_value(&document, base, name) {
                    copied.push((name.to_string(), value));
                }
            }
        }
        let mut text = settings.user_text.clone();
        for server in &self.scoped_servers {
            let entry = ScopedUserEntry {
                key: &server.id,
                name: &server.name,
                language_id: &server.language_id,
                file_names: &server.file_names,
                copied: &copied,
            };
            text = upsert_user_entry(&settings.user_path, &text, &entry)?;
        }
        return Ok(text);
    }
}
