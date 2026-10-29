//! Companion grammars paint content that bundled languages embed:
//!  comments,
//!  expressions,
//!  and scripts.
//! Each fragment below is painted differently,
//!  or not at all,
//!  when its companion grammar is absent.

/// What:
///  `mod syntax_support;` compiles the sibling file `syntax_support/mod.rs` into this
///       test program as a module named `syntax_support`.
/// Why:
///   Cargo builds each file in `tests/` as its own program,
///  so shared helpers are
///       included as a module rather than imported from a package.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import * as syntaxSupport from './syntax_support/mod';
/// ```
mod syntax_support;

/// The one shared assertion:
///  recognized language plus one painted fragment.
use syntax_support::assert_reads_as;

/// What:
///  `#[test]` marks the function below as a test the harness runs;
///  the function takes
///       nothing and passes unless it panics.
/// Why:
///   The comment grammar classifies issue references;
///  the host language alone paints the
///       whole comment with one role.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// test('comment grammar paints issue references inside comments', () => { /* ... */ });
/// ```
#[test]
fn comment_grammar_paints_issue_references_inside_comments() {
    assert_reads_as(
        "/project/src/index.ts",
        "// Tracked in #123 upstream.\nconst cat = 1;\n",
        "typescript",
        "constant",
        "#123",
    );
}

/// The regex grammar separates quantifiers from literal pattern characters.
#[test]
fn regex_grammar_paints_operators_inside_ecmascript_literals() {
    assert_reads_as(
        "/project/src/index.ts",
        "const pattern = /ca+t/;\n",
        "typescript",
        "operator",
        "+",
    );
}

/// The JSDoc grammar classifies braced types inside documentation comments.
/// The tag name itself is painted by the comment grammar's mention rule,
///  so it proves nothing here.
#[test]
fn jsdoc_grammar_paints_types_inside_documentation_comments() {
    assert_reads_as(
        "/project/src/index.ts",
        "/** @param {string} name */\nfunction cat(name: unknown) {}\n",
        "typescript",
        "type",
        "string",
    );
}

/// The format-arguments grammar classifies the format type inside Rust formatting macros.
/// The placeholder name would prove nothing:
///  Rust itself paints the same word at its binding.
#[test]
fn format_args_grammar_paints_format_types_inside_rust_macros() {
    assert_reads_as(
        "/project/src/main.rs",
        "fn main() { let cat = 1; println!(\"{cat:?}\"); }\n",
        "rust",
        "special",
        "?",
    );
}

/// The inline Markdown grammar classifies escapes inside paragraphs.
#[test]
fn markdown_inline_grammar_paints_escapes_inside_paragraphs() {
    assert_reads_as(
        "/project/README.md",
        "A literal \\* star.\n",
        "markdown",
        "constant",
        "\\*",
    );
}

/// The AWK grammar classifies program text passed to `awk` in shell.
#[test]
fn awk_grammar_paints_programs_inside_shell_commands() {
    assert_reads_as(
        "/project/run.sh",
        "awk '{ print $1 }' cats.txt\n",
        "bash",
        "keyword",
        "print",
    );
}

/// HTML embeds stylesheets and scripts,
///  both bundled languages.
#[test]
fn html_paints_embedded_stylesheets_and_scripts() {
    assert_reads_as(
        "/project/index.html",
        "<style>a { color: red; }</style>\n",
        "html",
        "variable",
        "color",
    );
    assert_reads_as(
        "/project/index.html",
        "<script>const cat = 1;</script>\n",
        "html",
        "keyword",
        "const",
    );
}

/// Workflow steps and container build steps embed shell.
#[test]
fn workflow_and_container_run_steps_are_painted_as_shell() {
    assert_reads_as(
        "/project/.github/workflows/ci.yml",
        "jobs:\n  test:\n    steps:\n      - run: echo cat\n",
        "github-action",
        "function",
        "echo",
    );
    assert_reads_as(
        "/project/Containerfile",
        "FROM fedora:44\nRUN echo cat\n",
        "dockerfile",
        "function",
        "echo",
    );
}
