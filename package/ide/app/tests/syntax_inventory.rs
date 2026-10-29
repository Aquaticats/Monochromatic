//! Every language measured in the repository is recognized and painted by the bundled runtime.
//! Paths mirror real repository file shapes;
//!  the selection evidence is recorded in
//! `doc/planning/slint-ide-runtime-languages.md`.

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
///   TypeScript source is the largest measured language.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// test('typescript source is read as typescript', () => { /* ... */ });
/// ```
#[test]
fn typescript_source_is_read_as_typescript() {
    assert_reads_as(
        "/project/src/index.ts",
        "const cat: string = 'cat';\n",
        "typescript",
        "keyword",
        "const",
    );
}

/// JavaScript modules use the `.mjs` and `.cjs` extensions as well as `.js`.
#[test]
fn javascript_module_is_read_as_javascript() {
    assert_reads_as(
        "/project/bin/tool.mjs",
        "const value = 42;\n",
        "javascript",
        "constant",
        "42",
    );
}

/// TSX stays bundled from the initial slice although no `.tsx` file is currently measured.
#[test]
fn tsx_component_is_read_as_tsx() {
    assert_reads_as(
        "/project/App.tsx",
        "const view = <div className=\"a\">cat</div>;\n",
        "tsx",
        "tag",
        "div",
    );
}

/// Rust source.
#[test]
fn rust_source_is_read_as_rust() {
    assert_reads_as(
        "/project/src/main.rs",
        "fn main() {}\n",
        "rust",
        "keyword",
        "fn",
    );
}

/// Ordinary JSON data such as `package.json`.
#[test]
fn json_data_is_read_as_json() {
    assert_reads_as(
        "/project/package.json",
        "{\"name\": \"cat\", \"private\": true}\n",
        "json",
        "constant",
        "true",
    );
}

/// `tsconfig.json` is JSON with comments,
///  a separate Helix language on the JSON grammar.
#[test]
fn typescript_project_configuration_is_read_as_jsonc() {
    assert_reads_as(
        "/project/tsconfig.json",
        "{\n  // note\n  \"strict\": true\n}\n",
        "jsonc",
        "comment",
        "// note",
    );
}

/// TOML manifests such as `Cargo.toml`.
#[test]
fn toml_manifest_is_read_as_toml() {
    assert_reads_as(
        "/project/Cargo.toml",
        "[package]\nname = \"ide\"\n",
        "toml",
        "type",
        "package",
    );
}

/// `mise.toml` is a separate Helix language on the TOML grammar.
#[test]
fn mise_configuration_is_read_as_miseconfig() {
    assert_reads_as(
        "/project/package/mise.toml",
        "[vars]\nimage = \"ide\"\n",
        "miseconfig",
        "type",
        "vars",
    );
}

/// Ordinary YAML data such as `pnpm-workspace.yaml`.
#[test]
fn yaml_data_is_read_as_yaml() {
    assert_reads_as(
        "/project/pnpm-workspace.yaml",
        "packages:\n  - 'package/*'\ncount: 3\n",
        "yaml",
        "constant",
        "3",
    );
}

/// Workflow files are a separate Helix language on the YAML grammar.
#[test]
fn workflow_is_read_as_github_action() {
    assert_reads_as(
        "/project/.github/workflows/ci.yml",
        "name: \"ci\"\non: push\n",
        "github-action",
        "string",
        "\"ci\"",
    );
}

/// Compose files are a separate Helix language on the YAML grammar.
#[test]
fn compose_file_is_read_as_docker_compose() {
    assert_reads_as(
        "/project/docker-compose.yml",
        "services:\n  web:\n    image: \"nginx\"\n",
        "docker-compose",
        "string",
        "\"nginx\"",
    );
}

/// HTML pages.
#[test]
fn html_page_is_read_as_html() {
    assert_reads_as(
        "/project/index.html",
        "<p class=\"a\">cat</p>\n",
        "html",
        "attribute",
        "class",
    );
}

/// CSS stylesheets.
#[test]
fn stylesheet_is_read_as_css() {
    assert_reads_as(
        "/project/site.css",
        "a { color: red; }\n",
        "css",
        "tag",
        "a",
    );
}

/// Kotlin source.
#[test]
fn kotlin_source_is_read_as_kotlin() {
    assert_reads_as(
        "/project/Main.kt",
        "fun main() { val cat = \"cat\" }\n",
        "kotlin",
        "keyword",
        "fun",
    );
}

/// Slint markup,
///  the language of this application's own interface.
#[test]
fn slint_markup_is_read_as_slint() {
    assert_reads_as(
        "/project/ui/app.slint",
        "export component App inherits Window { }\n",
        "slint",
        "keyword",
        "component",
    );
}

/// QML markup uses the `qmljs` grammar under the `qml` language name.
#[test]
fn qml_markup_is_read_as_qml() {
    assert_reads_as(
        "/project/Main.qml",
        "import QtQuick\nItem { width: 100 }\n",
        "qml",
        "keyword",
        "import",
    );
}

/// OpenTofu and Terraform configuration.
#[test]
fn terraform_configuration_is_read_as_hcl() {
    assert_reads_as(
        "/project/main.tf",
        "resource \"server\" \"web\" {\n  count = 1\n}\n",
        "hcl",
        "constant",
        "1",
    );
}

/// SQL schema and queries.
#[test]
fn sql_schema_is_read_as_sql() {
    assert_reads_as(
        "/project/schema.sql",
        "SELECT id FROM cats WHERE id = 1;\n",
        "sql",
        "keyword",
        "SELECT",
    );
}

/// Shell scripts with an extension.
#[test]
fn shell_script_is_read_as_bash() {
    assert_reads_as(
        "/project/run.sh",
        "echo \"cat\" # note\n",
        "bash",
        "comment",
        "# note",
    );
}

/// The repository's extensionless Gradle wrappers are recognized only by their shebang.
#[test]
fn extensionless_shell_wrapper_is_read_as_bash_by_shebang() {
    assert_reads_as(
        "/project/gradlew",
        "#!/bin/sh\necho cat\n",
        "bash",
        "function",
        "echo",
    );
}

/// C source.
#[test]
fn c_source_is_read_as_c() {
    assert_reads_as(
        "/project/probe.c",
        "int main(void) { return 0; }\n",
        "c",
        "keyword",
        "return",
    );
}

/// C++ source inherits the C highlighting rules.
#[test]
fn cpp_source_is_read_as_cpp() {
    assert_reads_as(
        "/project/bridge.cpp",
        "class Cat { public: int age() { return 1; } };\n",
        "cpp",
        "keyword",
        "class",
    );
}

/// Windows batch wrappers.
#[test]
fn batch_wrapper_is_read_as_batch() {
    assert_reads_as(
        "/project/gradlew.bat",
        "@echo off\nset NAME=cat\n",
        "batch",
        "keyword",
        "set",
    );
}

/// Dockerfiles and both Containerfile spellings use the Dockerfile grammar.
#[test]
fn container_definitions_are_read_as_dockerfile() {
    let source = "FROM fedora:44\nRUN echo cat\n";
    assert_reads_as(
        "/project/Dockerfile",
        source,
        "dockerfile",
        "keyword",
        "FROM",
    );
    assert_reads_as(
        "/project/Containerfile",
        source,
        "dockerfile",
        "keyword",
        "FROM",
    );
    assert_reads_as(
        "/project/test.Containerfile",
        source,
        "dockerfile",
        "keyword",
        "FROM",
    );
}

/// XML documents and SVG images share the XML grammar.
#[test]
fn xml_and_svg_are_read_as_xml() {
    let source = "<a b=\"c\">cat</a>\n";
    assert_reads_as("/project/layout.xml", source, "xml", "tag", "a");
    assert_reads_as("/project/icon.svg", source, "xml", "tag", "a");
}

/// Markdown and MDX share the Markdown language;
///  a fenced block is painted by its own grammar.
#[test]
fn markdown_and_mdx_are_read_as_markdown() {
    let source = "# Title\n\n```rust\nfn main() {}\n```\n";
    assert_reads_as("/project/README.md", source, "markdown", "keyword", "fn");
    assert_reads_as("/project/page.mdx", source, "markdown", "keyword", "fn");
}
