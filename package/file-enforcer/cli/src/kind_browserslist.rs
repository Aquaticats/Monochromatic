//! What:
//!  The `browserslist` rule kind:
//! resolve a `.browserslistrc` against the pinned data lock and write the resolved target list as JSON.
//! Why:
//!  Build tools that cannot resolve `.browserslistrc` themselves read one generated file.
//!  The rule owns its output outright,
//! reads the query file and the lock through the virtual view
//!  (so both are recorded inputs),
//! and delegates the resolution to the in-repo `monochromatic-browserslist` crate.
//!  Only pinned data is ever used:
//! the crate fetches a pinned tarball when the cache lacks it and never an unpinned one.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function browserslistRule(source: string, output: string, lock: string): Rule
//! ```

/// Imports `BrowserslistError`,
/// `DataLock`,
/// `DataSource`,
/// `render_browserslistrc` from `monochromatic_browserslist`.
use monochromatic_browserslist::{BrowserslistError, DataLock, DataSource, render_browserslistrc};
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
/// Imports `Arc` from `std::sync`.
use std::sync::Arc;

/// What:
///  Turns query-file text and lock text into the resolved JSON text.
///  A trait is an interface;
/// `dyn TargetRenderer` values are chosen at run time.
/// Why:
///  Production resolves against the cached pinned tarballs;
/// tests resolve against the committed fixture data,
/// so no test needs a network or a cache.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface TargetRenderer { render(configText: string, lockText: string): string }
/// ```
pub trait TargetRenderer: Send + Sync {
    /// Render the resolved targets as the exact text of the output file.
    fn render(&self, config_text: &str, lock_text: &str) -> Result<String, BrowserslistError>;
}

/// What:
///  The production renderer:
/// the pinned tarballs in the user cache directory,
/// fetched from the npm registry when absent.
/// Why:
///  `DataSource::user_default()` is the crate's documented entry for the engine.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class PinnedCacheRenderer implements TargetRenderer {}
/// ```
pub struct PinnedCacheRenderer;

/// The `impl TargetRenderer for PinnedCacheRenderer` block.
impl TargetRenderer for PinnedCacheRenderer {
    /// Parse the lock,
    /// open the user cache,
    /// and render.
    fn render(&self, config_text: &str, lock_text: &str) -> Result<String, BrowserslistError> {
        let lock = DataLock::from_json(lock_text)?;
        let source = DataSource::user_default()?;
        return render_browserslistrc(config_text, &lock, &source);
    }
}

/// What:
///  Builds a `BrowserslistRule` from a decoded `browserslist` config entry.
/// Why:
///  Registered under the kind `browserslist`,
/// replacing the placeholder that warns `kind-not-wired`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class BrowserslistBuilder implements RuleBuilder { constructor(readonly renderer: TargetRenderer) {} }
/// ```
pub struct BrowserslistBuilder {
    /// How the rules this builder makes resolve targets.
    renderer: Arc<dyn TargetRenderer>,
}

/// The `impl BrowserslistBuilder` block.
impl BrowserslistBuilder {
    /// The builder the command line registers:
    /// pinned data from the user cache.
    pub fn production() -> BrowserslistBuilder {
        return BrowserslistBuilder { renderer: Arc::new(PinnedCacheRenderer) };
    }

    /// A builder over another renderer,
    /// for tests and for tools that bring their own data.
    pub fn with_renderer(renderer: Arc<dyn TargetRenderer>) -> BrowserslistBuilder {
        return BrowserslistBuilder { renderer };
    }
}

/// The `impl RuleBuilder for BrowserslistBuilder` block.
impl RuleBuilder for BrowserslistBuilder {
    /// Build the rule,
    /// or explain that the entry is another kind.
    fn build(&self, config: &RuleConfig) -> Result<Box<dyn Rule>, Diagnostic> {
        match config {
            RuleConfig::Browserslist { id, source, output, lock, environment } => {
                return Ok(Box::new(BrowserslistRule {
                    id: id.clone(),
                    source: source.clone(),
                    output: output.clone(),
                    lock: lock.clone(),
                    environment: environment.clone(),
                    renderer: Arc::clone(&self.renderer),
                }));
            }
            other => {
                let message = format!("the browserslist builder was given a rule of kind {}", other.kind());
                return Err(Diagnostic::new("kind-unknown", &message).with_rule(other.id()));
            }
        }
    }
}

/// What:
///  A planned `browserslist` rule.
/// Why:
///  `environment` names the config section to prefer.
///  The resolver implements no sections (a `[section]` line is refused as not implemented),
/// so a query file without sections resolves its default queries whatever the environment says,
/// which is what browserslist does for a missing section.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class BrowserslistRule implements Rule { source: string; output: string; lock: string; environment?: string }
/// ```
pub struct BrowserslistRule {
    /// Rule id.
    pub id: String,
    /// Repo-relative query file.
    pub source: String,
    /// Repo-relative output path.
    pub output: String,
    /// Repo-relative data lock.
    pub lock: String,
    /// Config section to prefer;
    /// recorded, and reported when resolution fails.
    pub environment: Option<String>,
    /// How targets are resolved.
    renderer: Arc<dyn TargetRenderer>,
}

/// What:
///  Turn a resolver error into the diagnostic the run reports.
/// Why:
///  The code says which stage failed;
/// the hint says what the user can do about it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function diagnose(error: BrowserslistError, rule: BrowserslistRule): Diagnostic
/// ```
fn diagnose(error: &BrowserslistError, rule: &BrowserslistRule) -> Diagnostic {
    let message = format!("rule {}: {error}", rule.id);
    let environment = rule.environment.as_deref().unwrap_or("none");
    match error {
        BrowserslistError::NotImplemented { .. } => {
            let hint = format!(
                "only the query lines listed in package/browserslist/core-rust/README.md are supported; edit {} (environment: {environment})",
                rule.source
            );
            return Diagnostic::new("browserslist-not-implemented", &message).with_hint(&hint).with_path(&rule.source).with_rule(&rule.id);
        }
        BrowserslistError::Browserslist { .. } => {
            return Diagnostic::new("browserslist-query-failed", &message).with_path(&rule.source).with_rule(&rule.id);
        }
        BrowserslistError::Network { .. } => {
            let hint = "the pinned tarballs are fetched once into the user cache; connect to the network and rerun";
            return Diagnostic::new("browserslist-fetch-failed", &message).with_hint(hint).with_path(&rule.lock).with_rule(&rule.id);
        }
        BrowserslistError::Integrity { .. } => {
            let hint = "the downloaded bytes do not match the lock; rerun, or regenerate the lock with the browserslist update task";
            return Diagnostic::new("browserslist-integrity-mismatch", &message).with_hint(hint).with_path(&rule.lock).with_rule(&rule.id);
        }
        BrowserslistError::Data { .. } | BrowserslistError::Io { .. } => {
            return Diagnostic::new("browserslist-data-failed", &message).with_path(&rule.lock).with_rule(&rule.id);
        }
    }
}

/// The `impl BrowserslistRule` block.
impl BrowserslistRule {
    /// What:
    ///  Read one input through the view,
    /// failing with a path-bearing diagnostic when it is absent.
    /// Why:
    ///  Both inputs are required;
    /// an absent query file or lock is a configuration mistake,
    /// not an empty input.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// requireText(view: View, path: string, what: string): string
    /// ```
    fn require_text(&self, view: &mut View, path: &str, what: &str) -> Result<String, Diagnostic> {
        match view.read_text(path)? {
            Some(text) => return Ok(text),
            None => {
                let message = format!("rule {}: the {what} {path} does not exist", self.id);
                return Err(Diagnostic::new("browserslist-input-missing", &message).with_path(path).with_rule(&self.id));
            }
        }
    }
}

/// The `impl Rule for BrowserslistRule` block.
impl Rule for BrowserslistRule {
    /// The rule id.
    fn id(&self) -> &str {
        return &self.id;
    }

    /// The kind tag.
    fn kind(&self) -> &str {
        return "browserslist";
    }

    /// What:
    ///  Resolve the query file and plan the output file.
    /// Why:
    ///  The output is a pure function of the query file,
    /// the lock and the pinned data,
    /// so a conforming file plans a write of identical bytes and is not drift.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// plan(view: View): RulePlan
    /// ```
    fn plan(&self, view: &mut View) -> Result<RulePlan, Diagnostic> {
        let config_text = self.require_text(view, &self.source, "query file")?;
        let lock_text = self.require_text(view, &self.lock, "data lock")?;
        let rendered = self.renderer.render(&config_text, &lock_text).map_err(|error| return diagnose(&error, self))?;
        let change = Change::Write { path: self.output.clone(), content: rendered.into_bytes(), mode: None };
        return Ok(RulePlan { changes: vec![change], warnings: Vec::new() });
    }
}

/// Tests for the rule.
#[cfg(test)]
#[path = "kind_browserslist_tests.rs"]
mod tests;
