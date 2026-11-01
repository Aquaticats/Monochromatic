//! What:
//!  Real semantic conformance cases for explicit Rust declarations and generic arguments.
//! Why:
//!  Passing a typed example is not evidence that omitted arguments or nameable inference holes are rejected.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // Reuse one loaded fixture, alternating passing and failing source to exercise cache invalidation.
//! ```

/// Import actual findings and severities.
use crate::diagnostic::{Diagnostic, Severity};
/// Import the disposable Cargo-backed fixture,
///  not a test-only implementation of the rule.
use crate::rust_semantic_test_support::SemanticFixture;

/// What:
///  Authored source and independently expected policy-finding count.
/// Why:
///  The expectation does not depend on the production rule's traversal or resolver output.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Case = { name: string; source: string; count: number };
/// ```
struct Case {
    /// Failure context printed by assertions.
    name: &'static str,
    /// Complete standalone Rust input for the owned fixture.
    source: &'static str,
    /// Expected ordinary findings,
    ///  with semantic-processing errors prohibited separately.
    count: usize,
}

/// Check exact finding counts,
///  severity and the distinction from unavailable semantic information.
fn assert_case(fixture: &mut SemanticFixture, case: &Case) {
    let findings: Vec<Diagnostic> = fixture.check(case.source, Severity::Warn);
    assert_eq!(findings.len(), case.count, "{}: {findings:?}", case.name);
    for finding in &findings {
        assert!(!finding.processing_failure, "{}: {finding:?}", case.name);
        assert_eq!(finding.code, "rust/require-explicit-types", "{}", case.name);
        assert_eq!(finding.severity, Severity::Warn);
        assert!(finding.fix.is_none());
    }
}

/// Declaration requirements,
///  genuine generic arity,
///  aliases,
///  constructors and unnameable exceptions work together.
#[test]
fn semantic_conformance_and_source_overlay_controls() {
    let mut fixture: SemanticFixture = SemanticFixture::new();
    let cases: &[Case] = &[
        Case {
            name: "typed nongeneric code",
            source: "fn main() { let value: u16 = 1_u16; }",
            count: 0,
        },
        Case {
            name: "missing declaration annotation despite explicit initializer",
            source: "fn main() { let value = 1_u16; }",
            count: 1,
        },
        Case {
            name: "named generic function call needs arguments",
            source: "fn identity<T>(value: T) -> T { return value; } fn main() { let value: u16 = identity(1_u16); }",
            count: 1,
        },
        Case {
            name: "generic function item selection needs arguments before indirect call",
            source: "fn identity<T>(value: T) -> T { return value; } fn main() { let selected: _ = identity; let value: u16 = selected(1_u16); }",
            count: 1,
        },
        Case {
            name: "monomorphized function item may use its unnameable type",
            source: "fn identity<T>(value: T) -> T { return value; } fn main() { let selected: _ = identity::<u16>; let value: u16 = selected(1_u16); }",
            count: 0,
        },
        Case {
            name: "same-named generic and nongeneric methods differ",
            source: "struct Generic; impl Generic { fn parse<T>(&self, value: T) -> T { return value; } } struct Plain; impl Plain { fn parse(&self, value: u16) -> u16 { return value; } } fn main() { let first: u16 = Generic.parse(1_u16); let second: u16 = Plain.parse(2_u16); }",
            count: 1,
        },
        Case {
            name: "typed generic method passes",
            source: "struct Generic; impl Generic { fn parse<T>(&self, value: T) -> T { return value; } } fn main() { let value: u16 = Generic.parse::<u16>(1_u16); }",
            count: 0,
        },
        Case {
            name: "nameable scalar inference hole fails",
            source: "fn main() { let value: _ = 1_u16; }",
            count: 1,
        },
        Case {
            name: "nameable generic argument hole fails",
            source: "fn identity<T>(value: T) -> T { return value; } fn main() { let value: u16 = identity::<_>(1_u16); }",
            count: 1,
        },
        Case {
            name: "function pointer is nameable unlike its function item",
            source: "fn named(value: u16) -> u16 { return value; } fn main() { let pointer: _ = named as fn(u16) -> u16; }",
            count: 1,
        },
        Case {
            name: "explicit function pointer passes",
            source: "fn named(value: u16) -> u16 { return value; } fn main() { let pointer: fn(u16) -> u16 = named; }",
            count: 0,
        },
        Case {
            name: "reference around an unnameable item stays explicit",
            source: "fn named() {} fn main() { let value: _ = &named; }",
            count: 1,
        },
        Case {
            name: "explicit reference can retain unnameable leaf",
            source: "fn named() {} fn main() { let value: &_ = &named; }",
            count: 0,
        },
        Case {
            name: "enum arguments may be attached to enum or variant",
            source: "enum Choice<T> { Value(T), Empty } fn main() { let first: Choice<u16> = Choice::<u16>::Value(1_u16); let second: Choice<u16> = Choice::Value::<u16>(2_u16); let empty: Choice<u16> = Choice::<u16>::Empty; }",
            count: 0,
        },
        Case {
            name: "enum and variant are not double-reported",
            source: "enum Choice<T> { Value(T) } fn main() { let value: Choice<u16> = Choice::Value(1_u16); }",
            count: 1,
        },
        Case {
            name: "fixed alias binds underlying enum arguments",
            source: "enum Choice<T> { Value(T) } type Fixed = Choice<u16>; fn main() { let value: Fixed = Fixed::Value(1_u16); }",
            count: 0,
        },
        Case {
            name: "generic alias requires its own arguments",
            source: "enum Choice<T> { Value(T) } type Alias<T> = Choice<T>; fn main() { let value: Alias<u16> = Alias::Value(1_u16); }",
            count: 1,
        },
        Case {
            name: "constructor type arguments stay explicit",
            source: "struct Holder<T> { value: T } fn main() { let value: Holder<u16> = Holder { value: 1_u16 }; }",
            count: 1,
        },
        Case {
            name: "type and constant defaults remain defaults",
            source: "struct Holder<T = (), const N: usize = 3> { value: T } fn main() { let value: Holder = Holder { value: () }; }",
            count: 0,
        },
        Case {
            name: "constant generic argument is required",
            source: "fn build<const N: usize>() -> [u8; N] { return [0_u8; N]; } fn main() { let values: [u8; 3] = build(); }",
            count: 1,
        },
        Case {
            name: "enum qualifier constant hole retains its constant parameter",
            source: "enum Choice<const N: usize> { Value } fn main() { let value: Choice<3> = Choice::<_>::Value; }",
            count: 1,
        },
        Case {
            name: "enum variant constant hole retains its constant parameter",
            source: "enum Choice<const N: usize> { Value } fn main() { let value: Choice<3> = Choice::Value::<_>; }",
            count: 1,
        },
        Case {
            name: "mixed type and constant slots retain their positions",
            source: "fn build<T: Copy, const N: usize>(value: T) -> [T; N] { return [value; N]; } fn main() { let values: [u8; 3] = build::<u8, _>(1_u8); }",
            count: 1,
        },
        Case {
            name: "imported variant uses the enum's parameters",
            source: "enum Choice<T> { Value(T) } use Choice::Value; fn main() { let value: Choice<u16> = Value(1_u16); }",
            count: 1,
        },
        Case {
            name: "Self is fixed by its enclosing generic implementation",
            source: "struct Holder<T>(T); impl<T> Holder<T> { fn new(value: T) -> Self { return Self(value); } } fn main() { let value: Holder<u16> = Holder::<u16>::new(1_u16); }",
            count: 0,
        },
        Case {
            name: "explicit constant generic argument passes",
            source: "fn build<const N: usize>() -> [u8; N] { return [0_u8; N]; } fn main() { let values: [u8; 3] = build::<3>(); }",
            count: 0,
        },
        Case {
            name: "constant inference does not qualify as an unnameable-type exception",
            source: "fn build<const N: usize>() -> [u8; N] { return [0_u8; N]; } fn main() { let values: [u8; 3] = build::<_>(); }",
            count: 1,
        },
        Case {
            name: "nameable array length stays explicit",
            source: "fn main() { let values: [u8; _] = [0_u8; 3]; }",
            count: 1,
        },
        Case {
            name: "elided lifetimes do not become type-argument requirements",
            source: "fn borrow<'a, T>(value: &'a T) -> &'a T { return value; } fn main() { let value: u16 = 1_u16; let reference: &u16 = borrow::<u16>(&value); }",
            count: 0,
        },
        Case {
            name: "import alias retains the generic function identity",
            source: "mod source { pub fn identity<T>(value: T) -> T { return value; } } use source::identity as chosen; fn main() { let value: u16 = chosen(1_u16); }",
            count: 1,
        },
        Case {
            name: "fully qualified generic method retains its requirement",
            source: "struct Item; impl Item { fn method<T>(&self, value: T) -> T { return value; } } fn main() { let value: u16 = Item::method(&Item, 1_u16); }",
            count: 1,
        },
        Case {
            name: "trait method generic arguments are checked",
            source: "trait Identity { fn identity<T>(&self, value: T) -> T; } struct Item; impl Identity for Item { fn identity<T>(&self, value: T) -> T { return value; } } fn main() { let value: u16 = Item.identity(1_u16); }",
            count: 1,
        },
        Case {
            name: "function-trait shorthand supplies its tuple argument",
            source: "fn named(value: u16) -> u16 { return value; } fn apply(value: impl Fn(u16) -> u16) -> u16 { return value(1_u16); } fn main() { let result: u16 = apply(named); }",
            count: 0,
        },
        Case {
            name: "named generic parameter is not an opaque exception",
            source: "fn keep<T>(value: T) { let saved: _ = value; } fn main() {}",
            count: 1,
        },
        Case {
            name: "impl-Trait parameter cannot be named in a local annotation",
            source: "fn keep(value: impl Copy) { let saved: _ = value; } fn main() {}",
            count: 0,
        },
        Case {
            name: "opaque async result is an unnameable leaf",
            source: "async fn operation() -> u16 { return 1_u16; } fn main() { let future: _ = operation(); }",
            count: 0,
        },
        Case {
            name: "async block creates an unnameable coroutine, not an anonymous function",
            source: "fn main() { let future: _ = async { 1_u16 }; }",
            count: 0,
        },
        Case {
            name: "tuple structure must remain explicit",
            source: "fn named() {} fn main() { let pair: _ = (named, 1_u16); }",
            count: 1,
        },
        Case {
            name: "explicit tuple may retain an unnameable leaf",
            source: "fn named() {} fn main() { let pair: (_, u16) = (named, 1_u16); }",
            count: 0,
        },
    ];
    for case in cases {
        assert_case(&mut fixture, case);
    }
    // Revisit the original good input after every changed-source case: stale cached failures are also failures.
    assert_case(&mut fixture, &cases[0]);
    assert!(fixture.directory.path.is_dir());
    let disk_source: String =
        std::fs::read_to_string(&fixture.source_path).expect("read untouched physical fixture");
    assert_eq!(
        disk_source, "fn main() {}\n",
        "semantic overlays must not rewrite the user file"
    );
    let absent: std::path::PathBuf = fixture.source_path.with_file_name("absent.rs");
    let missing: crate::rust_semantic_error::SemanticError = fixture
        .session
        .check_file(&absent, "fn main() {}", "absent.rs", Severity::Error)
        .expect_err("unloaded source cannot be verified");
    assert!(missing.message.contains("was not loaded"));
    let relative: crate::rust_semantic_error::SemanticError = fixture
        .session
        .check_file(
            std::path::Path::new("main.rs"),
            "fn main() {}",
            "relative.rs",
            Severity::Error,
        )
        .expect_err("relative source path is not implicitly rebased");
    assert!(relative.message.contains("absolute path"));
    assert_case(&mut fixture, &cases[0]);
    let unavailable: Vec<Diagnostic> = fixture.check("fn main() { unresolved(); }", Severity::Warn);
    assert!(
        !unavailable.is_empty(),
        "unresolved context is not a clean result"
    );
    for finding in &unavailable {
        assert!(finding.processing_failure);
        assert_eq!(finding.code, "core/rust-type-resolution");
        assert_eq!(finding.severity, Severity::Error);
        assert!(finding.fix.is_none());
    }
    assert_case(&mut fixture, &cases[0]);
}
