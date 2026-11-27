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
//!  The rewriting of the settings text lives in `kind_jetbrains_harper_settings.rs`,
//! and the rule here decides when it runs.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function jetbrainsHarperRule(settings: HarperPolicy): Rule
//! ```

/// What:
///  How the policy rewrites the two settings files of one IDE product.
/// Why:
///  Reading them through the view,
/// patching the base server's entries and writing the scoped ones is one role,
/// and it lives in its own file so this one holds the rule itself.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { readSettings, updatedLanguage, updatedUser } from './kind_jetbrains_harper_settings'
/// ```
#[path = "kind_jetbrains_harper_settings.rs"]
mod settings;

/// Imports the XML helpers from `crate::harper_xml`.
use crate::harper_xml::{
    find_base_entry,
    parse,
};
/// Imports `LANGUAGE_FILE`,
/// `USER_FILE`,
/// `jetbrains_config_directory`,
/// `latest_options_directory` from `crate::jetbrains_discovery`.
use crate::jetbrains_discovery::{
    LANGUAGE_FILE,
    USER_FILE,
    jetbrains_config_directory,
    latest_options_directory,
};
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
use monochromatic_file_enforcer::rule::{
    Change,
    Rule,
    RuleBuilder,
    RulePlan,
};
/// Imports `View` from `monochromatic_file_enforcer::view`.
use monochromatic_file_enforcer::view::View;
/// Imports `OsString` from `std::ffi`.
use std::ffi::OsString;

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
///  It carries the `XDG_CONFIG_HOME` value the rules it builds use,
/// so tests choose the value instead of reading the process environment.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class JetbrainsHarperBuilder implements RuleBuilder { constructor(readonly xdgConfigHome: string | undefined) {} }
/// ```
pub struct JetbrainsHarperBuilder {
    /// The `XDG_CONFIG_HOME` value to honour;
    /// nothing means the variable is unset.
    pub xdg_config_home: Option<OsString>,
}

/// The `impl JetbrainsHarperBuilder` block.
impl JetbrainsHarperBuilder {
    /// A builder that honours the `XDG_CONFIG_HOME` the process was started with,
    /// as the retired TypeScript tool did.
    pub fn from_environment() -> JetbrainsHarperBuilder {
        return JetbrainsHarperBuilder {
            xdg_config_home: std::env::var_os("XDG_CONFIG_HOME"),
        };
    }
}

/// The `impl RuleBuilder for JetbrainsHarperBuilder` block.
impl RuleBuilder for JetbrainsHarperBuilder {
    /// Build the rule,
    /// or explain that the entry is another kind.
    fn build(
        &self,
        config: &RuleConfig,
    ) -> Result<Box<dyn Rule>, Diagnostic> {
        match config {
            RuleConfig::JetbrainsHarper {
                id,
                product_prefixes,
                disabled_linters,
                exclude_patterns,
                scoped_servers,
            } => {
                return Ok(Box::new(JetbrainsHarperRule {
                    id: id.clone(),
                    xdg_config_home: self.xdg_config_home.clone(),
                    product_prefixes: product_prefixes.clone(),
                    disabled_linters: disabled_linters.clone(),
                    exclude_patterns: exclude_patterns.clone(),
                    scoped_servers: scoped_servers.clone(),
                }));
            }
            other => {
                let message = format!(
                    "the jetbrains-harper builder was given a rule of kind {}",
                    other.kind()
                );
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
    /// The `XDG_CONFIG_HOME` value the IDE's configuration directory is derived from;
    /// unset,
    /// empty and relative values mean `.config` under the home directory.
    pub xdg_config_home: Option<OsString>,
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
    fn skip(
        &self,
        reason: &str,
    ) -> RulePlan {
        let message = format!(
            "rule {}: {reason}; the Harper policy was not applied",
            self.id
        );
        let warning = Diagnostic::new("jetbrains-harper-skipped", &message).with_rule(&self.id);
        return RulePlan {
            changes: Vec::new(),
            warnings: vec![warning],
            errors: Vec::new(),
        };
    }

    /// The ids of the scoped servers,
    /// which are never taken for the base server.
    fn scoped_ids(&self) -> Vec<String> {
        let mut ids: Vec<String> = Vec::new();
        for server in &self.scoped_servers {
            ids.push(server.id.clone());
        }
        return ids;
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
    fn plan(
        &self,
        view: &mut View,
    ) -> Result<RulePlan, Diagnostic> {
        let config_directory =
            match jetbrains_config_directory(view.root(), self.xdg_config_home.as_deref()) {
                Ok(directory) => directory,
                Err(reason) => return Ok(self.skip(&reason)),
            };
        let Some(options_directory) =
            latest_options_directory(view, &config_directory, &self.product_prefixes)?
        else {
            let reason = format!(
                "no installed product under {config_directory} starts with {:?}",
                self.product_prefixes
            );
            return Ok(self.skip(&reason));
        };
        let Some(settings) = self.read_settings(view, &options_directory)? else {
            return Ok(self.skip(&format!(
                "{options_directory} lacks {LANGUAGE_FILE} or {USER_FILE}"
            )));
        };
        let user_document = parse(&settings.user_path, &settings.user_text)?;
        let Some((_, key)) = find_base_entry(&user_document, &self.scoped_ids()) else {
            return Ok(self.skip(&format!(
                "{} holds no Harper base server",
                settings.user_path
            )));
        };
        let language = self.updated_language(&settings, &key)?;
        let user = self.updated_user(&settings, &key)?;
        let mut plan = RulePlan::default();
        if language != settings.language_text {
            plan.changes.push(Change::Write {
                path: settings.language_path.clone(),
                content: language.into_bytes(),
                mode: None,
            });
        }
        if user != settings.user_text {
            plan.changes.push(Change::Write {
                path: settings.user_path.clone(),
                content: user.into_bytes(),
                mode: None,
            });
        }
        return Ok(plan);
    }
}

/// Tests for the rule.
#[cfg(test)]
#[path = "kind_jetbrains_harper_tests.rs"]
mod tests;
