//! What:
//!  The `jetbrains-harper` rule kind:
//! apply this repository's Harper writing-style policy to the newest installed JetBrains IDE's LSP4IJ settings.
//! Why:
//!  The policy lives in two XML files of the IDE's options directory,
//! each embedding JSON in attributes.
//!  The rule belongs to the per-user config,
//! whose view root is the home directory,
//! so every path it reads and writes is relative to the home directory
//!  (`.config/JetBrains/<product>/options/...`).
//!  Like the retired TypeScript policy it does nothing when the IDE,
//! its settings files or the base Harper server
//! are absent,
//! and says so in a warning.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function jetbrainsHarperRule(settings: HarperPolicy): Rule
//! ```

/// Imports `CONFIG_DIRECTORY`,
/// `LANGUAGE_FILE`,
/// `USER_FILE`,
/// `latest_options_directory` from `crate::jetbrains_discovery`.
use crate::jetbrains_discovery::{CONFIG_DIRECTORY, LANGUAGE_FILE, USER_FILE, latest_options_directory};
/// Imports the JSON helpers from `crate::harper_json`.
use crate::harper_json::{disable_linters, format_object, parse_object, scoped_config, union_excludes, with_schema_defaults};
/// Imports the XML helpers from `crate::harper_xml`.
use crate::harper_xml::{find_base_entry, find_entry, option_value, parse};
/// Imports the entry edits from `crate::harper_xml_edit`.
use crate::harper_xml_edit::{ScopedUserEntry, upsert_language_entry, upsert_user_entry};
/// Imports `HarperScopedServer` from `monochromatic_file_enforcer::config_parts`.
use monochromatic_file_enforcer::config_parts::HarperScopedServer;
/// Imports `RuleConfig` from `monochromatic_file_enforcer::config_types`.
use monochromatic_file_enforcer::config_types::RuleConfig;
/// Imports `Diagnostic` from `monochromatic_file_enforcer::diagnostic`.
use monochromatic_file_enforcer::diagnostic::Diagnostic;
/// Imports `Change`,
/// `Rule`,
/// `RuleBuilder`,
/// `RulePlan` from `monochromatic_file_enforcer::rule`.
use monochromatic_file_enforcer::rule::{Change, Rule, RuleBuilder, RulePlan};
/// Imports `View` from `monochromatic_file_enforcer::view`.
use monochromatic_file_enforcer::view::View;
/// Imports `Map`,
/// `Value` from `serde_json`.
use serde_json::{Map, Value};

/// The options the scoped servers copy from the base server,
/// in output order.
const COPIED_OPTIONS: [&str; 7] = [
    "commandLine",
    "installAlreadyDone",
    "installerConfigurationContent",
    "serverUrl",
    "templateId",
    "workingDir",
    "workspaceFolderStrategyConfiguration",
];

/// What:
///  Builds a `JetbrainsHarperRule` from a decoded `jetbrains-harper` config entry.
/// Why:
///  Registered under the kind `jetbrains-harper`,
/// replacing the placeholder that warns `kind-not-wired`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class JetbrainsHarperBuilder implements RuleBuilder {}
/// ```
pub struct JetbrainsHarperBuilder;

/// The `impl RuleBuilder for JetbrainsHarperBuilder` block.
impl RuleBuilder for JetbrainsHarperBuilder {
    /// Build the rule,
    /// or explain that the entry is another kind.
    fn build(&self, config: &RuleConfig) -> Result<Box<dyn Rule>, Diagnostic> {
        match config {
            RuleConfig::JetbrainsHarper { id, product_prefixes, disabled_linters, exclude_patterns, scoped_servers } => {
                return Ok(Box::new(JetbrainsHarperRule {
                    id: id.clone(),
                    product_prefixes: product_prefixes.clone(),
                    disabled_linters: disabled_linters.clone(),
                    exclude_patterns: exclude_patterns.clone(),
                    scoped_servers: scoped_servers.clone(),
                }));
            }
            other => {
                let message = format!("the jetbrains-harper builder was given a rule of kind {}", other.kind());
                return Err(Diagnostic::new("kind-unknown", &message).with_rule(other.id()));
            }
        }
    }
}

/// What:
///  A planned `jetbrains-harper` rule.
/// Why:
///  Holds the policy:
/// which IDE products,
/// which linters are disabled on the base server,
/// which files it ignores,
/// and which extra servers are scoped to named files.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class JetbrainsHarperRule implements Rule { productPrefixes: string[]; ... }
/// ```
pub struct JetbrainsHarperRule {
    /// Rule id.
    pub id: String,
    /// IDE product prefixes;
    /// the newest installed version wins.
    pub product_prefixes: Vec<String>,
    /// Linters disabled on the base server.
    pub disabled_linters: Vec<String>,
    /// Patterns added to the base server's exclude list.
    pub exclude_patterns: Vec<String>,
    /// Extra servers scoped to named files.
    pub scoped_servers: Vec<HarperScopedServer>,
}

/// The settings files of one IDE product,
/// read through the view.
struct SettingsFiles {
    /// Home-relative path of the language settings file.
    language_path: String,
    /// Its text.
    language_text: String,
    /// Home-relative path of the user-defined settings file.
    user_path: String,
    /// Its text.
    user_text: String,
}

/// The `impl JetbrainsHarperRule` block.
impl JetbrainsHarperRule {
    /// A plan that changes nothing and says why.
    fn skip(&self, reason: &str) -> RulePlan {
        let message = format!("rule {}: {reason}; the Harper policy was not applied", self.id);
        let warning = Diagnostic::new("jetbrains-harper-skipped", &message).with_rule(&self.id);
        return RulePlan { changes: Vec::new(), warnings: vec![warning] };
    }

    /// The ids of the scoped servers,
    /// which are never taken for the base server.
    fn scoped_ids(&self) -> Vec<String> {
        return self.scoped_servers.iter().map(|server| return server.id.clone()).collect();
    }

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
    fn read_settings(&self, view: &mut View, options_directory: &str) -> Result<Option<SettingsFiles>, Diagnostic> {
        let language_path = format!("{options_directory}/{LANGUAGE_FILE}");
        let user_path = format!("{options_directory}/{USER_FILE}");
        let language_text = view.read_text(&language_path)?;
        let user_text = view.read_text(&user_path)?;
        match (language_text, user_text) {
            (Some(language), Some(user)) => {
                return Ok(Some(SettingsFiles { language_path, language_text: language, user_path, user_text: user }));
            }
            _ => return Ok(None),
        }
    }

    /// The base server's patched configuration,
    /// from the configuration the language settings file holds for it.
    fn base_config(&self, settings: &SettingsFiles, key: &str) -> Result<Map<String, Value>, Diagnostic> {
        let document = parse(&settings.language_path, &settings.language_text)?;
        let Some(entry) = find_entry(&document, key) else {
            let message = format!("{}: no language server settings entry for the base server {key}", settings.language_path);
            return Err(Diagnostic::new("harper-xml-invalid", &message).with_path(&settings.language_path).with_rule(&self.id));
        };
        let existing = option_value(&document, entry, "configurationContent").unwrap_or_else(|| return "{}".to_string());
        let mut config = parse_object(&existing, "configurationContent", &settings.language_path)?;
        disable_linters(&mut config, &self.disabled_linters);
        union_excludes(&mut config, &self.exclude_patterns);
        return Ok(config);
    }

    /// The schema text shared by the base and scoped entries.
    fn schema_text(&self, settings: &SettingsFiles, key: &str) -> Result<String, Diagnostic> {
        let document = parse(&settings.language_path, &settings.language_text)?;
        let existing = find_entry(&document, key)
            .and_then(|entry| return option_value(&document, entry, "configurationSchemaContent"))
            .unwrap_or_else(|| return "{\"properties\":{}}".to_string());
        let schema = parse_object(&existing, "configurationSchemaContent", &settings.language_path)?;
        return Ok(format_object(with_schema_defaults(schema)));
    }

    /// The language settings file after the policy:
    /// the base entry patched,
    /// each scoped entry written.
    fn updated_language(&self, settings: &SettingsFiles, key: &str) -> Result<String, Diagnostic> {
        let base = self.base_config(settings, key)?;
        let schema = self.schema_text(settings, key)?;
        let path = settings.language_path.as_str();
        let mut text = upsert_language_entry(path, &settings.language_text, key, &format_object(base.clone()), &schema)?;
        for server in &self.scoped_servers {
            let config = format_object(scoped_config(&base, &server.disabled_linters));
            text = upsert_language_entry(path, &text, &server.id, &config, &schema)?;
        }
        return Ok(text);
    }

    /// The user-defined settings file after the policy:
    /// each scoped entry written,
    /// with options copied from the base server's entry.
    fn updated_user(&self, settings: &SettingsFiles, key: &str) -> Result<String, Diagnostic> {
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

/// The `impl Rule for JetbrainsHarperRule` block.
impl Rule for JetbrainsHarperRule {
    /// The rule id.
    fn id(&self) -> &str {
        return &self.id;
    }

    /// The kind tag.
    fn kind(&self) -> &str {
        return "jetbrains-harper";
    }

    /// What:
    ///  Find the newest product,
    /// derive both settings files and plan a write for each file whose text differs.
    /// Why:
    ///  A machine that does not have the IDE or the Harper server yet is not an error.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// plan(view: View): RulePlan
    /// ```
    fn plan(&self, view: &mut View) -> Result<RulePlan, Diagnostic> {
        let Some(options_directory) = latest_options_directory(view, &self.product_prefixes)? else {
            let reason = format!("no installed product under {CONFIG_DIRECTORY} starts with {:?}", self.product_prefixes);
            return Ok(self.skip(&reason));
        };
        let Some(settings) = self.read_settings(view, &options_directory)? else {
            return Ok(self.skip(&format!("{options_directory} lacks {LANGUAGE_FILE} or {USER_FILE}")));
        };
        let user_document = parse(&settings.user_path, &settings.user_text)?;
        let Some((_, key)) = find_base_entry(&user_document, &self.scoped_ids()) else {
            return Ok(self.skip(&format!("{} holds no Harper base server", settings.user_path)));
        };
        let language = self.updated_language(&settings, &key)?;
        let user = self.updated_user(&settings, &key)?;
        let mut plan = RulePlan::default();
        if language != settings.language_text {
            plan.changes.push(Change::Write { path: settings.language_path.clone(), content: language.into_bytes(), mode: None });
        }
        if user != settings.user_text {
            plan.changes.push(Change::Write { path: settings.user_path.clone(), content: user.into_bytes(), mode: None });
        }
        return Ok(plan);
    }
}

/// Tests for the rule.
#[cfg(test)]
#[path = "kind_jetbrains_harper_tests.rs"]
mod tests;
