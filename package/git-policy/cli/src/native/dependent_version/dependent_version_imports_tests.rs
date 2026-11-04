//! What: Specifier detection, with every `importsPackage` case of
//!       `source-imports.unit.test.ts` and each boundary of the keyword and quote rules.
//! Why: A missed import drops a bundled edge; a false one adds a bump.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(importsPackage({ sourceText: "import { a } from '@scope/b';", packageName: '@scope/b' })).toBe(true);
//! ```

/// The scan under test.
use super::imports_package;

/// Whether the text imports `@scope/b`.
fn imports(text: &str) -> bool {
    return imports_package(text, "@scope/b");
}

/// Ported: every specifier form the incumbent detects.
#[test]
fn detects_the_incumbent_forms() {
    for text in [
        "import { a } from '@scope/b';",
        "import type { A } from \"@scope/b/ts\";",
        "import '@scope/b';",
        "export * from '@scope/b';",
        "const m = await import('@scope/b');",
        "const m = await import ( `@scope/b/sub` );",
        "const m = require('@scope/b');",
        "import { a }\nfrom\n'@scope/b';",
    ] {
        assert!(imports(text), "{text}");
    }
}

/// Ported: every non-specifier the incumbent ignores.
#[test]
fn ignores_the_incumbent_non_specifiers() {
    for text in [
        "import { a } from '@scope/bc';",
        "const name = '@scope/b';",
        "reimport '@scope/b';",
        "myrequire('@scope/b');",
        "import { a } from 'x@scope/b';",
        "@scope/b",
        "",
    ] {
        assert!(!imports(text), "{text}");
    }
}

/// Ported: a later specifier is found after an earlier mention.
#[test]
fn finds_a_later_specifier_after_an_earlier_mention() {
    assert!(imports(
        "const note = '@scope/b';\nimport { a } from '@scope/b';"
    ));
}

/// Each identifier character before a keyword makes it part of a longer word.
#[test]
fn requires_a_whole_keyword() {
    for prefix in ["a", "Z", "7", "_", "$"] {
        assert!(!imports(&format!("{prefix}from '@scope/b'")), "{prefix}");
        assert!(
            !imports(&format!("{prefix}require('@scope/b')")),
            "{prefix}"
        );
    }
    assert!(imports("-from '@scope/b'"));
    assert!(imports("from'@scope/b'"));
    assert!(imports("x=require\t(\r'@scope/b')"));
}

/// The quotes must match, or the name must continue as a subpath.
#[test]
fn requires_a_closing_quote_or_a_subpath() {
    assert!(!imports("import '@scope/b\";"));
    assert!(!imports("import '@scope/b"));
    assert!(imports("import \"@scope/b\""));
    assert!(imports("import `@scope/b/x`"));
    assert!(!imports("import (@scope/b)"));
}

/// A literal without a keyword or callee before it is not a specifier.
#[test]
fn requires_specifier_position() {
    assert!(!imports("'@scope/b'"));
    assert!(!imports("('@scope/b')"));
    assert!(!imports("x('@scope/b')"));
    assert!(!imports("from('@scope/b')"));
    assert!(imports("import('@scope/b')"));
    // A callee keyword followed by one more letter is neither a keyword nor a call.
    assert!(!imports("requirex '@scope/b'"));
    assert!(!imports("importx '@scope/b'"));
}

/// An empty name follows the stated rule instead of looping as the incumbent does.
#[test]
fn applies_the_rule_to_an_empty_name() {
    assert!(imports_package("import ''", ""));
    assert!(imports_package("require('/x')", ""));
    assert!(!imports_package("const a = ''", ""));
}
