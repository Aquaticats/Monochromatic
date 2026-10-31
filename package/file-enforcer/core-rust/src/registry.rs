//! What:
//!  The registry that turns decoded config entries into rules,
//! and the placeholder for kinds whose library is not
//!  linked yet.
//! Why:
//!  The dependency-catalog library,
//! the browserslist crate and the JetBrains XML crate are separate crates;
//! each
//!  registers its own builder under its kind (`registry.register("browserslist", Box::new(builder))`) and replaces
//!  the placeholder,
//! with no change to the engine.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! class RuleRegistry { register(kind: string, builder: RuleBuilder): void; build(config: Config): Rule[] }
//! ```

/// Imports `Config`,
/// `RuleConfig` from `crate::config_types`.
use crate::config_types::{Config, RuleConfig};
/// Imports `Diagnostic` from `crate::diagnostic`.
use crate::diagnostic::Diagnostic;
/// Imports `ConcatRule` from `crate::kind_concat`.
use crate::kind_concat::ConcatRule;
/// Imports `MirrorRule` from `crate::kind_mirror`.
use crate::kind_mirror::MirrorRule;
/// Imports `MisePathListRule` from `crate::kind_mise_path_list`.
use crate::kind_mise_path_list::MisePathListRule;
/// Imports `ModeOnlyRule` from `crate::kind_mode_only`.
use crate::kind_mode_only::ModeOnlyRule;
/// Imports `PackageLicensesRule` from `crate::kind_package_licenses`.
use crate::kind_package_licenses::PackageLicensesRule;
/// Imports `TomlKeysRule` from `crate::kind_toml_keys`.
use crate::kind_toml_keys::TomlKeysRule;
/// Imports `Rule`,
/// `RuleBuilder`,
/// `RulePlan` from `crate::rule`.
use crate::rule::{Rule, RuleBuilder, RulePlan};
/// Imports `View` from `crate::view`.
use crate::view::View;
/// Imports `BTreeMap` from `std::collections`.
use std::collections::BTreeMap;

/// What:
///  Builds the rule kinds this crate implements itself.
/// Why:
///  One builder covers the six built-in kinds by matching the decoded variant.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class BuiltinBuilder implements RuleBuilder {}
/// ```
pub struct BuiltinBuilder;

/// What:
///   How the built-in kinds are built:
/// one match over the config variants.
/// Why:
///   The six kinds this crate implements need no external crate,
/// so one builder serves them all.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class BuiltinBuilder implements RuleBuilder { build(config): Rule }
/// ```
impl RuleBuilder for BuiltinBuilder {
    /// What:
    ///  Build one built-in rule from its config entry.
    /// Why:
    ///  Kinds owned by other crates are not matched here;
    /// they get their own builders.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// build(config: RuleConfig): Rule
    /// ```
    fn build(&self, config: &RuleConfig) -> Result<Box<dyn Rule>, Diagnostic> {
        match config {
            RuleConfig::Concat { id, output, parts, mode } => {
                return Ok(Box::new(ConcatRule::new(id, output, parts, mode)));
            }
            RuleConfig::ModeOnly { id, path, mode } => return Ok(Box::new(ModeOnlyRule::new(id, path, mode))),
            RuleConfig::PackageLicenses { id, manifests, directory, texts, requires } => {
                return Ok(Box::new(PackageLicensesRule {
                    id: id.clone(),
                    manifests: manifests.clone(),
                    directory: directory.clone(),
                    texts: texts.clone(),
                    requires: requires.clone(),
                }));
            }
            RuleConfig::Mirror { id, from, to } => {
                if from == to || from.starts_with(&format!("{to}/")) || to.starts_with(&format!("{from}/")) {
                    let message = format!("rule {id}: the mirror source {from} and target {to} must not contain each other");
                    return Err(Diagnostic::new("config-invalid", &message).with_rule(id));
                }
                return Ok(Box::new(MirrorRule { id: id.clone(), from: from.clone(), to: to.clone() }));
            }
            RuleConfig::TomlKeys { id, manifests, only_dirs, edits, blocks } => {
                return Ok(Box::new(TomlKeysRule {
                    id: id.clone(),
                    manifests: manifests.clone(),
                    only_dirs: only_dirs.clone(),
                    edits: edits.clone(),
                    blocks: blocks.clone(),
                }));
            }
            RuleConfig::MisePathList { id, output, directories, header } => {
                return Ok(Box::new(MisePathListRule {
                    id: id.clone(),
                    output: output.clone(),
                    directories: directories.clone(),
                    header: header.clone(),
                }));
            }
            other => {
                let message = format!("the kind {} has no built-in implementation", other.kind());
                return Err(Diagnostic::new("kind-unknown", &message).with_rule(other.id()));
            }
        }
    }
}

/// What:
///  A rule whose kind is declared but whose library is not linked into this build.
/// Why:
///  Planning it yields a `kind-not-wired` warning and no changes,
/// which is not drift:
/// the run stays usable while the
///  library that owns the kind is still being built.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class NotWiredRule implements Rule { id: string; kind: string; owner: string }
/// ```
pub struct NotWiredRule {
    /// Id of the rule entry that named the unlinked kind.
    id: String,
    /// The kind that nothing has registered a builder for.
    kind: String,
    /// The library that implements the kind,
    /// named in the warning.
    owner: String,
}

/// What:
///   How the placeholder rule behaves:
/// it plans nothing and warns.
/// Why:
///   A kind whose crate is not linked yet must not break a run,
/// but its outputs must not look checked.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class NotWiredRule implements Rule { plan(view): RulePlan { return { changes: [], warnings: [kindNotWired] }; } }
/// ```
impl Rule for NotWiredRule {
    /// The rule id.
    fn id(&self) -> &str {
        return &self.id;
    }

    /// The kind tag.
    fn kind(&self) -> &str {
        return &self.kind;
    }

    /// What:
    ///  Report that the kind is not linked and plan nothing.
    /// Why:
    ///  The warning names the kind,
    /// the rule and the library that will provide it.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// plan(view: View): RulePlan
    /// ```
    fn plan(&self, _view: &mut View) -> Result<RulePlan, Diagnostic> {
        let message = format!(
            "the rule {} of kind {} was skipped: {} is not linked into this build, so its outputs are neither written nor checked",
            self.id, self.kind, self.owner
        );
        let warning = Diagnostic::new("kind-not-wired", &message).with_rule(&self.id);
        return Ok(RulePlan { changes: Vec::new(), warnings: vec![warning] });
    }
}

/// What:
///  Builds a `NotWiredRule` for one kind.
/// Why:
///  The default registry installs one for each kind another crate owns.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class NotWiredBuilder implements RuleBuilder { constructor(readonly owner: string) {} }
/// ```
pub struct NotWiredBuilder {
    /// Name of the library that will implement the kind.
    pub owner: String,
}

/// What:
///   How a placeholder is built for a kind that nothing has registered.
/// Why:
///   The registry installs one for each seam kind so a config that names it still loads.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class NotWiredBuilder implements RuleBuilder { build(config): Rule }
/// ```
impl RuleBuilder for NotWiredBuilder {
    /// Build the placeholder rule for `config`.
    fn build(&self, config: &RuleConfig) -> Result<Box<dyn Rule>, Diagnostic> {
        return Ok(Box::new(NotWiredRule {
            id: config.id().to_string(),
            kind: config.kind().to_string(),
            owner: self.owner.clone(),
        }));
    }
}

/// What:
///  Maps each kind tag to its builder.
/// Why:
///  Building a rule list from a config is a lookup per entry.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class RuleRegistry { builders: Map<string, RuleBuilder> }
/// ```
pub struct RuleRegistry {
    /// The builder for each kind name;
    /// `register` replaces an entry.
    builders: BTreeMap<String, Box<dyn RuleBuilder>>,
}

/// What:
///   Creating a registry,
/// registering a kind's builder,
/// and building every rule of a config.
/// Why:
///   A kind registered later replaces the placeholder,
/// and a config with several bad entries reports them all.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class RuleRegistry { static withDefaults(); register(kind, builder); build(config): Rule[] }
/// ```
impl RuleRegistry {
    /// What:
    ///  The registry with the six built-in kinds and placeholders for `rust-catalog`,
    /// `browserslist` and
    ///  `jetbrains-harper`.
    /// Why:
    ///  This is what the command line uses until the other libraries are linked.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// RuleRegistry.withDefaults()
    /// ```
    pub fn with_defaults() -> RuleRegistry {
        let mut registry = RuleRegistry { builders: BTreeMap::new() };
        for kind in ["concat", "mode-only", "package-licenses", "mirror", "toml-keys", "mise-path-list"] {
            registry.register(kind, Box::new(BuiltinBuilder));
        }
        let owners = [
            ("rust-catalog", "the dependency-catalog Rust library"),
            ("browserslist", "the in-repo browserslist crate"),
            ("jetbrains-harper", "the in-repo XML edit crate"),
        ];
        for (kind, owner) in owners {
            registry.register(kind, Box::new(NotWiredBuilder { owner: owner.to_string() }));
        }
        return registry;
    }

    /// What:
    ///  Install or replace the builder for a kind.
    /// Why:
    ///  This is the seam:
    /// another crate calls it with its own builder.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// register(kind: string, builder: RuleBuilder): void
    /// ```
    pub fn register(&mut self, kind: &str, builder: Box<dyn RuleBuilder>) {
        self.builders.insert(kind.to_string(), builder);
    }

    /// What:
    ///  Build every rule of a config,
    /// collecting every failure.
    /// Why:
    ///  Whole-run validation:
    /// all problems are reported before anything is planned.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// build(config: Config): Rule[]
    /// ```
    pub fn build(&self, config: &Config) -> Result<Vec<Box<dyn Rule>>, Vec<Diagnostic>> {
        let mut rules: Vec<Box<dyn Rule>> = Vec::new();
        let mut errors: Vec<Diagnostic> = Vec::new();
        for entry in &config.rules {
            match self.builders.get(entry.kind()) {
                Some(builder) => match builder.build(entry) {
                    Ok(rule) => rules.push(rule),
                    Err(error) => errors.push(error),
                },
                None => {
                    let message = format!("no builder is registered for the kind {}", entry.kind());
                    errors.push(Diagnostic::new("kind-unknown", &message).with_rule(entry.id()));
                }
            }
        }
        if errors.is_empty() {
            return Ok(rules);
        }
        return Err(errors);
    }
}
