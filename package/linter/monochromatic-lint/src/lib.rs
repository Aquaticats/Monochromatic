//! What:
//!  Internal implementation library for the native unified linter.
//! Why:
//!  The executable and its verification drivers must exercise the same implementation.
//! This is not a supported public linter API.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // Internal implementation entry imported by the executable and tests.
//! ```

/// What:
///  The configuration-value merge module,
///  visible to artifact and fuzz consumers.
/// Why:
///  Verification must exercise the exact implementation rather than a copied model.
/// Its visibility is internal integration plumbing,
///  not a stable user-facing library contract.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export * as configMerge from './config-merge';
/// ```
#[doc(hidden)]
pub mod config_merge;

/// Accepted command grammar with native path values.
#[doc(hidden)]
pub mod cli_options;
/// The `cargo` processes of the closure check.
mod closure_cargo;
/// Mapping of compiler reports to findings with the configured severity.
mod closure_findings;
/// The per-package prerequisite manifest of the closure check.
mod closure_manifest;
/// Grouping of the files that select the closure check by owning package.
mod closure_packages;
/// Machine prerequisite checks of the closure check.
mod closure_probe;
/// The prerequisites that prepare a build:
///  the IDE artifacts and the Android cross-compilation.
mod closure_probe_platform;
/// Cargo's JSON records read into closure-check reports.
mod closure_report;
/// The closure check as a step of a lint run.
mod closure_step;
/// Internal data-shape checks,
///  including duplicate decoded keys.
mod config_data;
/// Internal typed configuration errors shared by the executable and verification drivers.
#[doc(hidden)]
pub mod config_error;
/// Internal nearest-file and explicit configuration discovery.
#[doc(hidden)]
pub mod config_lookup;
/// Internal compiled file-pattern matching and effective rule settings.
#[doc(hidden)]
pub mod config_match;
/// Internal measurement of the repository-pinned tool version behind the stale-binary hint.
mod config_pin;
/// Internal ordered JSONC configuration parser.
#[doc(hidden)]
pub mod configuration;
/// Internal fixed-registry rule-option validation.
mod configuration_rules;
/// Internal typed JSONL findings shared by rule execution and processor mapping.
#[doc(hidden)]
pub mod diagnostic;
/// Inline disable directives:
///  the filter that suppresses covered findings and reports broken or unused directives.
mod directive_apply;
/// Findings about directives themselves,
///  with stable identifiers.
mod directive_findings;
/// Unused-directive reporting settings shared by the command line and file plans.
#[doc(hidden)]
pub mod directive_options;
/// The directive comment grammar.
mod directive_parse;
/// Active directives and the findings about the broken ones.
mod directive_resolve;
/// Candidate comments found by a real Rust or Markdown parse.
mod directive_scan;
/// Internal all-or-nothing grouped source edits.
#[doc(hidden)]
pub mod edits;
/// Native source-file discovery with explicit ignore and failure boundaries.
#[doc(hidden)]
pub mod file_discovery;
/// Bounded cycle-aware fixpoint execution over exact source snapshots.
#[doc(hidden)]
pub mod fix_loop;
/// Prose fix boundary that must not create new Markdown block syntax.
mod markdown_block_start;
/// Byte-addressed prose break points and abbreviation exclusions.
mod markdown_break_points;
/// Internal code-fence normalization.
#[doc(hidden)]
pub mod markdown_code;
/// Copyable prompt-only shell examples with byte-preserving grouped fixes.
#[doc(hidden)]
pub mod markdown_commands;
/// Used and unique reference definitions with localized removal fixes.
#[doc(hidden)]
pub mod markdown_definitions;
/// Selected Markdown rules over one shared parse,
///  in registry order.
#[doc(hidden)]
pub mod markdown_dispatch;
/// Duplicate headings scoped by their textual ancestor path.
#[doc(hidden)]
pub mod markdown_duplicate_headings;
/// Shared Markdown diagnostic construction.
mod markdown_finding;
/// Internal report-only heading checks.
#[doc(hidden)]
pub mod markdown_headings;
/// Repository `.lfsconfig` endpoint declarations.
#[doc(hidden)]
pub mod markdown_lfs_config;
/// Repository discovery and per-file LFS target resolution.
#[doc(hidden)]
pub mod markdown_lfs_context;
/// The single LFS endpoint normalizer.
#[doc(hidden)]
pub mod markdown_lfs_endpoint;
/// Object URLs for images whose target is LFS-tracked.
#[doc(hidden)]
pub mod markdown_lfs_image_url;
/// Pointer and content object ids.
mod markdown_lfs_oid;
/// Gitignore-syntax matching for tracked paths and rule exclusions.
#[doc(hidden)]
pub mod markdown_lfs_patterns;
/// SHA-256 of smudged file bytes.
mod markdown_lfs_sha256;
/// Lexical classification of image destinations.
mod markdown_lfs_target;
/// Internal link normalization checks.
#[doc(hidden)]
pub mod markdown_links;
/// Measured incumbent lexical-boundary conformance fixtures.
#[cfg(test)]
mod markdown_parity_tests;
/// Owned Markdown line and UTF-16 position indexes.
mod markdown_positions;
/// Paragraph/container ancestry for prose line-break fixes.
mod markdown_prose_context;
/// Heading punctuation checks with complete source escape/entity edits.
#[doc(hidden)]
pub mod markdown_punctuation;
/// Typed Markdown rule selection from validated merged JSONC.
#[doc(hidden)]
pub mod markdown_rule_settings;
/// Prose line breaks that replace their separating space,
/// with token,
///  delimiter and block guards.
#[doc(hidden)]
pub mod markdown_semantic_breaks;
/// Internal native Markdown/MDX parser interface.
#[doc(hidden)]
pub mod markdown_source;
/// HTML/MDX cell-text encoding at the generated table boundary.
mod markdown_table_text;
/// Pipe-table reporting and source-preserving HTML fallback.
#[doc(hidden)]
pub mod markdown_tables;
/// Validated arena traversal lists built by the source adapter's walk.
mod markdown_traversal;
/// Native literal and glob command-input expansion.
#[doc(hidden)]
pub mod path_inputs;
/// Input-expansion consumer controls.
#[cfg(test)]
mod path_inputs_tests;
/// Internal finalization of matched partial rule settings.
mod resolved_rules;
/// Host rules plus always-on processors for one source snapshot.
#[doc(hidden)]
pub mod run_check;
/// Mode selection and the complete lint invocation with injected environment.
#[doc(hidden)]
pub mod run_command;
/// Core processing-failure and refused-fix findings.
#[doc(hidden)]
pub mod run_failure;
/// Per-file read,
///  bounded fix loop and atomic rewrite.
#[doc(hidden)]
pub mod run_file;
/// Output controls,
///  debug notes and semantic-engine creation shared by every lint mode.
#[doc(hidden)]
pub mod run_finish;
/// Strict JSON rendering of an effective configuration.
#[doc(hidden)]
pub mod run_json;
/// Per-run sharing of LFS repository facts.
#[doc(hidden)]
pub mod run_lfs;
/// Rule listing,
///  starter configuration and effective-configuration printing.
#[doc(hidden)]
pub mod run_modes;
/// JSONL routing,
///  stdin-fix source output and exit-status accounting.
#[doc(hidden)]
pub mod run_output;
/// Absolute,
///  display and configuration-relative names,
///  and language by extension.
#[doc(hidden)]
pub mod run_paths;
/// Memoized configuration lookup and per-file plans.
#[doc(hidden)]
pub mod run_plan;
/// Real arguments,
///  streams and exit status.
#[doc(hidden)]
pub mod run_process;
/// Standard-input linting and fixing.
mod run_stdin;
/// Bounded worker threads with per-file panic containment.
#[doc(hidden)]
pub mod run_workers;
/// Atomic replacement that keeps permission bits.
#[doc(hidden)]
pub mod run_write;
/// Syntax-only Rust execution without initializing a Cargo workspace.
#[doc(hidden)]
pub mod rust_dispatch;
/// Declaration-site checks used by the full semantic explicit-types rule.
#[doc(hidden)]
pub mod rust_explicit_declarations;
/// Generic argument checking by actual declaration identity.
mod rust_explicit_generics;
/// Written inference-hole validation against resolved types.
mod rust_explicit_inference;
/// Full explicit-type policy over a registered semantic parse.
#[doc(hidden)]
pub mod rust_explicit_types;
/// Invocation-level Rust dispatch and lazy workspace caching.
#[doc(hidden)]
pub mod rust_file_engine;
/// Generic parameter ownership and argument-spelling helpers.
mod rust_generic_arguments;
/// Semantic disambiguation of type-shaped generic constant placeholders.
mod rust_inferred_constants;
/// Typed Rust rule selection from validated merged JSONC.
#[doc(hidden)]
pub mod rust_rule_settings;
/// Internal fixed Rust rule implementations.
#[doc(hidden)]
pub mod rust_rules;
/// Typed semantic workspace and query failures.
#[doc(hidden)]
pub mod rust_semantic_error;
/// Production session for selected-file analysis and exact source overlays.
#[doc(hidden)]
pub mod rust_semantic_session;
/// Internal shared Rust parse and code-line indexing.
#[doc(hidden)]
pub mod rust_source;
/// Internal syntax-only requirement that unit tests live in sidecar files.
#[doc(hidden)]
pub mod rust_tests_in_sidecars;
/// Read-only compiler and installed rust-src discovery.
#[doc(hidden)]
pub mod rust_toolchain;
/// Internal source-anchored explicit-type findings and semantic coverage errors.
mod rust_type_diagnostic;
/// Fixed Cargo workspace preparation and generated-input error handling.
#[doc(hidden)]
pub mod rust_workspace;

/// The shell dialects,
///  the constructs the workflow rules ban and the reading of a `shell:` value.
#[doc(hidden)]
pub mod shell_dialect;
/// Typed selection of the `shell/` rules from validated merged JSONC.
#[doc(hidden)]
pub mod shell_rule_settings;
/// The `shell/` rules over one extracted run body.
#[doc(hidden)]
pub mod shell_rules;
/// The scan that finds banned constructs in one shell run body.
#[doc(hidden)]
pub mod shell_scan;
/// The `cmd.exe` scanner.
mod shell_scan_cmd;
/// The POSIX-family and fish scanner.
mod shell_scan_posix;
/// The PowerShell scanner.
mod shell_scan_powershell;

/// Line reading for the workflow walk.
mod workflow_lines;
/// The records the workflow walk returns.
mod workflow_model;
/// Reading the values of `run:` and `shell:` keys.
mod workflow_run;
/// The structural walk that finds every step of a workflow or composite action.
mod workflow_walk;

/// Disposable native filesystem helpers used only by tests.
#[cfg(test)]
mod test_fs;

/// Compiles the stub `cargo` that the closure-check tests run instead of the real one.
#[cfg(test)]
mod closure_stub_support;

/// Shared orchestration fixtures:
///  disposable trees,
///  parsed command lines and JSONL decoding.
#[cfg(test)]
mod run_test_support;

/// End-to-end controls for the shell rules over workflows and composite actions.
#[cfg(test)]
mod run_workflow_tests;

/// End-to-end directive controls through real rules,
///  processors,
///  the semantic engine,
///  the fix loop and the command.
#[cfg(test)]
mod run_directive_tests;

/// Full semantic annotation controls,
///  including source-overlay cache changes.
#[cfg(test)]
mod rust_explicit_types_tests;
/// Disposable Cargo-backed contexts for the semantic rule's conformance suite.
#[cfg(test)]
mod rust_semantic_test_support;

/// Production Cargo discovery,
///  generated-source and failure boundary controls.
#[cfg(test)]
mod rust_workspace_tests;

/// Shared consumer-level regressions for the initial Markdown rule ports.
#[cfg(test)]
mod markdown_basic_tests;

/// Deep embedded-source extraction and original-host projection seam.
#[doc(hidden)]
pub mod processors;
/// Syntax checking that excludes processor-generated scaffolding.
mod processors_check;
/// Authored native Rustdoc discovery and prefix preservation.
mod processors_docs;
/// Native Markdown/MDX Rust-fence discovery.
mod processors_fences;
/// Exact physical-line copying shared by processor adapters.
mod processors_lines;
/// Immutable processor source and mapping records.
mod processors_model;
/// Hidden-line and synthetic-main doctest preparation.
mod processors_prepare;
/// Atomic grouped fixes with native container re-extraction.
mod processors_projection;
/// Byte-safe processor group validation and source reconstruction.
mod processors_rewrite;
/// Composed authored positions and original-host column conventions.
mod processors_spans;
/// Virtual shell files for the steps of a workflow or composite action.
mod processors_workflow;
