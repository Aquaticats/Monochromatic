//! What:
//!  Properties of the executable's per-source path:
//!  host rules,
//!  always-on processors,
//!  nested
//! doc tests,
//!  host-mapped findings,
//!  the bounded fix loop,
//!  and adversarial grouped edits.
//! Why:
//!  The rule and processor targets check their modules alone.
//!  This target drives the same
//! `process_source` call the executable makes for every file,
//!  so configuration matching of
//! virtual paths,
//!  projection to the host and fixing are exercised together.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const outcome = processSource(plan, source, fix); assertHostFindings(outcome, source);
//! ```

/// Import the production configuration,
///  planning and per-source pipeline.
use monochromatic_lint::{
    config_lookup::ConfigurationSource,
    config_match::{FileConfiguration, PreparedConfiguration, prepare_configuration},
    configuration::parse_configuration,
    diagnostic::{Diagnostic, Span},
    directive_options::DirectiveOptions,
    edits::{Edit, Fix, apply_fixes},
    markdown_rule_settings::markdown_rule_settings,
    processors::{VirtualSource, extract},
    run_file::{SourceOutcome, process_source},
    run_lfs::LfsRepos,
    run_paths::Language,
    run_plan::{FilePlan, RootRules},
    rust_rule_settings::rust_rule_settings,
};
/// Import native paths and shared ownership for the prepared configuration.
use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

/// What:
///  The configuration every case runs under.
/// Why:
///  `**/*.rs` reaches fenced Rust and doc tests,
///  and `**/*.md` reaches rustdoc,
///  so every
/// virtual file of a host is checked.
///  No rule here reads the filesystem or a Cargo workspace.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const CONFIGURATION = '[ { "files": ["**\/*.rs"], ... }, { "files": ["**\/*.md", "**\/*.mdx"], ... } ]';
/// ```
const CONFIGURATION: &str = r#"[
  { "files": ["**/*.rs"], "rules": {
    "rust/max-lines": { "severity": "warn", "max": 12 },
    "rust/require-rustdoc": { "severity": "error" }
  } },
  { "files": ["**/*.md", "**/*.mdx"], "rules": {
    "markdown/heading-increment": { "severity": "error" },
    "markdown/commands-show-output": { "severity": "error" },
    "markdown/no-duplicate-heading": { "severity": "error" },
    "markdown/single-h1": { "severity": "error" },
    "markdown/no-trailing-punctuation": { "severity": "error" },
    "markdown/no-bare-urls": { "severity": "error" },
    "markdown/no-emphasis-as-heading": { "severity": "error" },
    "markdown/fenced-code-language": { "severity": "error" },
    "markdown/link-image-reference-definitions": { "severity": "error" },
    "markdown/link-image-style": { "severity": "error" },
    "markdown/no-pipe-tables": { "severity": "error" },
    "markdown/semantic-line-breaks": { "severity": "error" }
  } }
]"#;

/// What:
///  One host with the rule codes it must produce,
///  counted by hand from the design,
///  sorted.
/// Why:
///  A raw-only generator could run forever without ever reaching a nested doc test;
///  these
/// cases prove each processor layer is reached and reported on every draw.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Case = { name: string; source: string; codes: string[] };
/// ```
pub struct Case {
    /// Host filename;
    ///  its extension chooses the language.
    pub name: &'static str,
    /// Host source with LF line endings.
    pub source: &'static str,
    /// Expected finding codes,
    ///  sorted.
    pub codes: &'static [&'static str],
}

/// What:
///  Hosts covering rustdoc,
///  doc tests,
///  quoted fences,
///  MDX and two levels of nesting.
/// Why:
///  Each expectation follows from the design alone:
///  which virtual files exist,
///  which blocks
/// match their paths,
///  and which rule each violates.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const CASES: Case[] = [/* ... */];
/// ```
pub const CASES: &[Case] = &[
    // A doc test inside rustdoc: the unlabeled fence is a Markdown finding, the undocumented function a Rust one.
    Case {
        name: "doctest.rs",
        source: "//! File.\n\n/// Item prose.\n///\n/// ```\n/// //! Example.\n/// let value = 1; fn helper() {}\n/// ```\nfn item() {}\n",
        codes: &["markdown/fenced-code-language", "rust/require-rustdoc"],
    },
    // A fence inside a block quote: only the undocumented function in the virtual Rust file is reported.
    Case {
        name: "quoted.md",
        source: "# Title\n\n> ```rust\n> //! Docs.\n> let value = 1; fn helper() {}\n> ```\n",
        codes: &["rust/require-rustdoc"],
    },
    // A fence below JSX in MDX: the snippet lacks its opening `//!` line.
    Case {
        name: "jsx.mdx",
        source: "<section>\n\n```rs\nlet value = 1;\n```\n\n</section>\n",
        codes: &["rust/require-rustdoc"],
    },
    // Two Rust levels deep: the inner rustdoc's unlabeled fence and the innermost undocumented function.
    Case {
        name: "nested.rs",
        source: "//! Root.\n/// ````rust\n/// //! Level one.\n/// /// ```\n/// /// //! Level two.\n/// /// let item: u32 = 1; fn helper() {}\n/// /// ```\n/// fn example() {}\n/// ````\nfn outer() {}\n",
        codes: &["markdown/fenced-code-language", "rust/require-rustdoc"],
    },
    // Three fixable Markdown findings in one rustdoc comment, fixed together through the host.
    Case {
        name: "grouped.rs",
        source: "//! File.\n\n/// # Heading.\n///\n/// See https://example.com/a for details.\n///\n/// ```\n/// //! Example.\n/// let value: u32 = 1;\n/// ```\nfn item() {}\n",
        codes: &[
            "markdown/fenced-code-language",
            "markdown/no-bare-urls",
            "markdown/no-trailing-punctuation",
        ],
    },
    // A clean host: a documented authored main with a hidden doc-test line. Nothing is reported.
    Case {
        name: "clean.rs",
        source: "//! File.\n\n/// Item prose.\n///\n/// ```rust\n/// //! Example.\n/// /// Entry.\n/// fn main() {\n/// # let hidden: u32 = 1;\n/// }\n/// ```\nfn item() {}\n",
        codes: &[],
    },
    // A directive covers the first bare URL in a Markdown host; the second is reported and fixed.
    // The clean fence gives the case a virtual file, which the edit-group controls require.
    Case {
        name: "suppressed.md",
        source: "# Title\n\n<!-- monochromatic-lint-disable-next-line markdown/no-bare-urls -- reason -->\nSee https://example.com/a here\n\nSee https://example.com/b here\n\n```rust\n//! Example.\nlet value: u32 = 1;\n```\n",
        codes: &["markdown/no-bare-urls"],
    },
    // A Rust directive covers a bare URL in rustdoc, and one aimed at the line budget is refused.
    Case {
        name: "directive.rs",
        source: "//! File.\n\n// monochromatic-lint-disable-next-line markdown/no-bare-urls -- reason\n/// See https://example.com/a for details.\n// monochromatic-lint-disable-next-line rust/max-lines -- reason\nfn item() {}\n",
        codes: &["directive/non-suppressible"],
    },
    // An authored main is not wrapped, so it is an item of the virtual file and needs its own doc.
    Case {
        name: "authored.rs",
        source: "//! File.\n\n/// Item prose.\n///\n/// ```rust\n/// //! Example.\n/// fn main() {}\n/// ```\nfn item() {}\n",
        codes: &["rust/require-rustdoc"],
    },
];

/// What:
///  The exact host text `grouped.rs` must become under `--fix`.
/// Why:
///  Each fix lands inside the comment,
///  keeping its `/// ` prefix;
///  nothing else moves.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const GROUPED_FIXED = '...';
/// ```
pub const GROUPED_FIXED: &str = "//! File.\n\n/// # Heading\n///\n/// See <https://example.com/a> for details.\n///\n/// ```rust\n/// //! Example.\n/// let value: u32 = 1;\n/// ```\nfn item() {}\n";

/// What:
///  Replacement texts for adversarial edits.
/// Why:
///  Delimiters,
///  comment prefixes,
///  newlines,
///  wrapper syntax and astral text are what could
/// break a container or a mapping if projection accepted them carelessly.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const REPLACEMENTS = ['', 'x', '\n', '```\n', '*\/', '\r\n', '🚀', '# ', '/// ', 'fn main() {}'];
/// ```
const REPLACEMENTS: [&str; 10] = [
    "",
    "x",
    "\n",
    "```\n",
    "*/",
    "\r\n",
    "🚀",
    "# ",
    "/// ",
    "fn main() {}",
];

/// What:
///  Build the plan the executable would build for a host,
///  without touching the filesystem.
/// Why:
///  The fuzz process has no repository;
///  the plan's paths are only names for matching and display.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function plan(name: string): FilePlan;
/// ```
pub fn plan(name: &str) -> FilePlan {
    let base: PathBuf = PathBuf::from("/fuzz");
    let config: Arc<PreparedConfiguration> = Arc::new(
        prepare_configuration(ConfigurationSource {
            path: base.join("monochromatic-lint.config.jsonc"),
            base: base.clone(),
            blocks: parse_configuration(CONFIGURATION).expect("fixed configuration parses"),
        })
        .expect("fixed patterns compile"),
    );
    let language: Language = if name.ends_with(".rs") {
        Language::Rust
    } else if name.ends_with(".mdx") {
        Language::Mdx
    } else {
        Language::Markdown
    };
    let resolved: FileConfiguration = config
        .resolve(Path::new(name))
        .expect("fixed configuration resolves");
    let FileConfiguration::Configured { rules } = resolved else {
        panic!("every fuzz host is configured");
    };
    let root: RootRules = if language == Language::Rust {
        RootRules::Rust {
            settings: rust_rule_settings(&rules).expect("fixed Rust settings"),
        }
    } else {
        RootRules::Markdown {
            settings: markdown_rule_settings(&rules).expect("fixed Markdown settings"),
        }
    };
    return FilePlan {
        absolute: base.join(name),
        display: String::from(name),
        relative: PathBuf::from(name),
        language,
        config,
        root,
        directives: DirectiveOptions::default(),
    };
}

/// What:
///  Assert that a finding names the host and addresses host bytes.
/// Why:
///  Whatever layer produced it,
///  a finding must be usable against the host file alone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function assertHostFinding(finding: Diagnostic, name: string, source: string): void;
/// ```
fn assert_host_finding(finding: &Diagnostic, name: &str, source: &str) {
    assert_eq!(finding.filename, name, "{}", finding.code);
    assert!(!finding.labels.is_empty(), "{}", finding.code);
    for label in &finding.labels {
        let span: &Span = &label.span;
        assert!(span.offset <= source.len(), "{}", finding.code);
        assert!(
            span.length <= source.len() - span.offset,
            "{}",
            finding.code
        );
        assert!(source.is_char_boundary(span.offset), "{}", finding.code);
        assert!(
            source.is_char_boundary(span.offset + span.length),
            "{}",
            finding.code
        );
        assert!(span.line >= 1 && span.column >= 1, "{}", finding.code);
    }
    if let Some(fix) = &finding.fix {
        assert_applies(source, fix);
    }
}

/// What:
///  Assert that one fix group applies to the host by itself,
///  or is the deliberate empty-file refusal.
/// Why:
///  A group the applier rejects as out of range or self-overlapping would be a mapping defect.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function assertApplies(source: string, fix: Fix): string | undefined;
/// ```
fn assert_applies(source: &str, fix: &Fix) -> Option<String> {
    match apply_fixes(source, std::slice::from_ref(fix)) {
        Ok(applied) => return Some(applied.source),
        Err(error) => {
            assert_eq!(
                error.message,
                "Autofix would replace non-empty file with empty output; leaving file unchanged."
            );
            return None;
        }
    }
}

/// What:
///  Lint and fix one host through the production path and check every outcome.
/// Why:
///  Returns the sorted lint codes so structured cases can compare them with their expectation.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function checkHost(name: string, source: string): string[];
/// ```
pub fn check_host(name: &str, source: &str) -> Vec<String> {
    let planned: FilePlan = plan(name);
    let lfs: LfsRepos = LfsRepos::new();
    let lint: SourceOutcome = process_source(&planned, source, false, &lfs, None);
    assert!(lint.fixed.is_none());
    let mut codes: Vec<String> = Vec::<String>::new();
    let mut incomplete: bool = false;
    for finding in &lint.findings {
        assert_host_finding(finding, name, source);
        incomplete |= finding.processing_failure;
        codes.push(finding.code.clone());
    }
    codes.sort();
    let fixing: SourceOutcome = process_source(&planned, source, true, &lfs, None);
    match &fixing.fixed {
        Some(fixed) => {
            // A lint-time processing failure must have stopped the fixer.
            assert!(
                !incomplete,
                "{name} was fixed despite incomplete processing"
            );
            for finding in &fixing.findings {
                assert_host_finding(finding, name, fixed.as_str());
                assert!(!finding.processing_failure, "{}", finding.message);
            }
            // A non-empty host never becomes empty.
            assert!(source.is_empty() || !fixed.is_empty());
            // A settled result is a fixed point of the same path.
            if settled(fixing.notes.as_slice()) {
                let again: SourceOutcome =
                    process_source(&planned, fixed.as_str(), true, &lfs, None);
                assert_eq!(again.fixed.as_deref(), Some(fixed.as_str()), "{name}");
            }
        }
        None => {
            // A refused fix says so and reports against the unchanged source.
            let mut refused: bool = false;
            for finding in &fixing.findings {
                assert_host_finding(finding, name, source);
                refused |= finding.code == "core/fix-refused";
            }
            assert!(refused, "{name} was neither fixed nor refused");
        }
    }
    return codes;
}

/// What:
///  Whether a debug note records that the fix loop stopped because nothing changed.
/// Why:
///  Only a settled result must be a fixed point;
///  a cycle or an exhausted pass budget is a
/// documented stop that may still change on a later invocation.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function settled(notes: string[]): boolean { return notes.some(note => note.endsWith('stopped as Unchanged')); }
/// ```
fn settled(notes: &[String]) -> bool {
    for note in notes {
        if note.ends_with("stopped as Unchanged") {
            return true;
        }
    }
    return false;
}

/// What:
///  How far one adversarial edit group travelled.
/// Why:
///  The generator control counts these,
///  so a generator that only ever produced refused
/// groups could not pass as coverage of the apply path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type EditReach = 'no-virtual-file' | 'refused' | 'empty-refusal' | 'applied';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EditReach {
    /// The host could not be extracted or holds no virtual file to edit.
    NoVirtualFile,
    /// Projection refused the group;
    ///  the host is untouched.
    Refused,
    /// The projected group would have emptied a non-empty host and was refused by the applier.
    EmptyRefusal,
    /// The projected group was applied and the rewritten host was processed again.
    Applied,
}

/// What:
///  Project an adversarial edit group from one virtual file to the host and apply it.
/// Why:
///  Projection may refuse,
///  but what it accepts must apply cleanly,
///  leave a host the extractor
/// can read again without panicking,
///  and never empty a non-empty host silently.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function checkAdversarialEdits(name: string, source: string, data: Uint8Array): EditReach;
/// ```
pub fn check_adversarial_edits(name: &str, source: &str, data: &[u8]) -> EditReach {
    let planned: FilePlan = plan(name);
    let Ok(inputs): Result<Vec<VirtualSource>, _> = extract(
        String::from(name),
        String::from(source),
        planned.language.processor(),
    ) else {
        return EditReach::NoVirtualFile;
    };
    if inputs.is_empty() {
        return EditReach::NoVirtualFile;
    }
    let selector: usize = usize::from(data.first().copied().unwrap_or(0));
    let input: &VirtualSource = &inputs[selector % inputs.len()];
    let length: usize = input.source().len();
    let count: usize = 1 + usize::from(data.get(1).copied().unwrap_or(0)) % 3;
    let mut edits: Vec<Edit> = Vec::<Edit>::new();
    for index in 0..count {
        let at: usize = 2 + index * 3;
        let start: usize = usize::from(data.get(at).copied().unwrap_or(0)) * 3 % (length + 2);
        let width: usize = usize::from(data.get(at + 1).copied().unwrap_or(0)) % 9;
        let replacement: &str =
            REPLACEMENTS[usize::from(data.get(at + 2).copied().unwrap_or(0)) % REPLACEMENTS.len()];
        edits.push(Edit {
            start,
            end: start + width,
            replacement: String::from(replacement),
        });
    }
    let Ok(projected): Result<Fix, _> = input.project_fix(&Fix { edits }) else {
        return EditReach::Refused;
    };
    for edit in &projected.edits {
        assert!(edit.start <= edit.end && edit.end <= source.len());
        assert!(source.is_char_boundary(edit.start) && source.is_char_boundary(edit.end));
    }
    let Some(rewritten): Option<String> = assert_applies(source, &projected) else {
        return EditReach::EmptyRefusal;
    };
    // The rewritten host goes through the whole production path again without panicking.
    check_host(name, rewritten.as_str());
    return EditReach::Applied;
}

/// What:
///  The fuzz entry:
///  a counted case in two newline spellings,
///  adversarial edits against it,
/// then the raw input as a host in each language.
/// Why:
///  Every draw reaches nested processors,
///  and arbitrary text still exercises recovery paths.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function checkOrchestration(data: Uint8Array): void;
/// ```
pub fn check_orchestration(data: &[u8]) {
    let selector: usize = usize::from(data.first().copied().unwrap_or(0));
    let crlf: bool = data.get(1).copied().unwrap_or(0) % 2 == 1;
    let case: &Case = &CASES[selector % CASES.len()];
    let source: String = if crlf {
        case.source.replace('\n', "\r\n")
    } else {
        String::from(case.source)
    };
    assert_eq!(
        check_host(case.name, source.as_str()),
        case.codes,
        "{}",
        case.name
    );
    let tail: &[u8] = if data.len() > 2 { &data[2..] } else { &[] };
    check_adversarial_edits(case.name, source.as_str(), tail);
    // Invalid UTF-8 cannot be a source String; its bytes already selected a structured case.
    if let Ok(raw) = std::str::from_utf8(data) {
        for name in ["raw.rs", "raw.md", "raw.mdx"] {
            check_host(name, raw);
            check_adversarial_edits(name, raw, tail);
        }
    }
}

/// Every counted case produces its hand-counted codes in both newline spellings.
#[test]
fn counted_cases_reach_every_processor_layer() {
    for (index, case) in CASES.iter().enumerate() {
        for newline in 0..2_u8 {
            check_orchestration(&[u8::try_from(index).expect("bounded case index"), newline]);
        }
        assert_eq!(
            check_host(case.name, case.source),
            case.codes,
            "{}",
            case.name
        );
    }
    check_orchestration(&[]);
    check_orchestration(&[255, 254, 253]);
}

/// The grouped case is fixed through the host comment to exactly the expected text.
#[test]
fn grouped_rustdoc_fixes_land_in_the_host_comment() {
    let planned: FilePlan = plan("grouped.rs");
    let lfs: LfsRepos = LfsRepos::new();
    let source: &str = CASES[4].source;
    let outcome: SourceOutcome = process_source(&planned, source, true, &lfs, None);
    assert_eq!(outcome.fixed.as_deref(), Some(GROUPED_FIXED));
    assert!(outcome.findings.is_empty(), "{:?}", outcome.findings);
    let crlf: SourceOutcome = process_source(
        &planned,
        source.replace('\n', "\r\n").as_str(),
        true,
        &lfs,
        None,
    );
    assert_eq!(
        crlf.fixed.as_deref(),
        Some(GROUPED_FIXED.replace('\n', "\r\n").as_str())
    );
}

/// Adversarial edit groups are refused or applied without panicking for every selector byte,
/// and the generator reaches both the refusal and the apply path.
#[test]
fn adversarial_edit_groups_are_refused_or_applied_safely() {
    let mut applied: usize = 0;
    let mut refused: usize = 0;
    let mut other: usize = 0;
    for case in CASES {
        for first in 0..=255_u8 {
            let data: [u8; 11] = [
                first,
                first.wrapping_mul(7),
                first.wrapping_mul(13),
                first.wrapping_add(3),
                first.wrapping_mul(3),
                first.wrapping_mul(29),
                first.wrapping_add(1),
                first.wrapping_mul(5),
                first.wrapping_mul(17),
                first.wrapping_add(2),
                first.wrapping_mul(11),
            ];
            match check_adversarial_edits(case.name, case.source, &data) {
                EditReach::Applied => applied += 1,
                EditReach::Refused => refused += 1,
                EditReach::EmptyRefusal | EditReach::NoVirtualFile => other += 1,
            }
        }
    }
    // Every counted case holds a virtual file and none can be emptied by nine replaced bytes.
    assert_eq!(other, 0);
    assert_eq!(applied + refused, CASES.len() * 256);
    // Measured on 2026-10-05: 563 applied and 1229 refused. The bounds leave room for processor
    // changes while still failing if either path stops being reached.
    assert!(applied >= 256, "only {applied} groups were applied");
    assert!(refused >= 256, "only {refused} groups were refused");
}

/// Raw Unicode,
///  byte-order marks and mixed newlines go through every language without panicking.
#[test]
fn raw_hosts_are_processed_in_every_language() {
    for raw in [
        "",
        "\u{feff}",
        "\u{feff}# 🚀 Title.\r\n\r\n```\r\n🚀\r\n```\r",
        "/// ```\n/// ```\n",
        "/** ```rust\n * let 🚀 = 1;\n * ``` */\nfn f() {}\n",
        "<A>\n</B>\n",
        "> - ```rust\n>   # hidden\n>   ```\n",
        "//! ```\n//! /// ```\n//! /// /// ```\n//! /// /// x\n//! /// /// ```\n//! /// ```\n//! ```\n",
    ] {
        check_orchestration(raw.as_bytes());
    }
}
