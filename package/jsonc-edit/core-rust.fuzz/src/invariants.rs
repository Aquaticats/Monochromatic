//! What:
//!      Property checks shared by every fuzz target in this sidecar.
//! Why:
//!       The contract the unit and fixture suites assert is the same contract fuzzing must
//!           assert,
//!  so it is written once here and every target calls it.

/// What:
///      Import the crate's public parse,
///  emit and model surface.
/// Why:
///       Invariants are stated over the same API a consumer uses,
///           never over internals,
///           so a refactor that preserves behavior cannot invalidate them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { emitJsoncValue, parseJsonc, JsoncKind, JsoncValue } from 'monochromatic-jsonc-edit';
/// ```
use monochromatic_jsonc_edit::{emit_jsonc_value, parse_jsonc, JsoncKind, JsoncValue};

/// What:
///      The accepted container nesting limit.
/// Why:
///       It is part of the published contract and the shared fixtures,
///           so the bound is asserted here rather than trusted.
const ACCEPTED_DEPTH: usize = 512;

/// What:
///      Parse one source,
///  emit it,
///  reparse the emission and emit again,
///  asserting stability.
/// Why:
///       Canonical output must be a fixed point:
///           if a second pass changed anything,
///           the emitter would not be deterministic and round-trip editing would drift.
///
/// # Arguments
///
/// * `source` - Document text that must already parse.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function assertCanonicalStability(source: string): void;
/// ```
pub fn assert_canonical_stability(source: &str) {
    let first = parse_jsonc(source).unwrap_or_else(|error| panic!("generated source must parse: {error}"));
    let once = emit_jsonc_value(&first);
    let second = parse_jsonc(&once).unwrap_or_else(|error| panic!("emission must reparse: {error}\n{once}"));
    let twice = emit_jsonc_value(&second);
    assert_eq!(once, twice, "canonical emission is not a fixed point");
    assert_trees_equal(&first, &second);
}

/// What:
///      Collect every comment body in a tree,
///  preorder.
/// Why:
///       Preservation is asserted per body,
///           and a tree walk that also counts owners would conflate two different failures.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function collectComments(value: JsoncValue): string[];
/// ```
pub fn collect_comments(value: &JsoncValue) -> Vec<String> {
    let mut found: Vec<String> = Vec::new();
    let mut stack: Vec<&JsoncValue> = vec![value];
    while let Some(current) = stack.pop() {
        if let Some(comment) = &current.comment {
            found.push(comment.text.clone());
        }
        if let JsoncKind::Record { entries } = &current.kind {
            for entry in entries.iter().rev() {
                if let Some(key_comment) = &entry.key.comment {
                    found.push(key_comment.text.clone());
                }
                stack.push(&entry.value);
            }
        }
        if let JsoncKind::Array { elements } = &current.kind {
            for element in elements.iter().rev() {
                stack.push(element);
            }
        }
    }
    return found;
}

/// What:
///      Assert every comment body in the tree survives into the emitted text.
/// Why:
///       Comments are data here,
///           so emission that quietly drops one is the worst failure this crate can have.
///           Each body line is matched as a substring,
///           because canonical emission splits a merged multi-line body into one `//` line per body
///           line and may re-render a single-line body with the other delimiter.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function assertCommentsPreserved(value: JsoncValue, emitted: string): void;
/// ```
pub fn assert_comments_preserved(value: &JsoncValue, emitted: &str) {
    for body in collect_comments(value) {
        // What: Check every body line rather than the whole body as one substring.
        // Why: Canonical emission renders a merged multi-line body as one `//` line per body line,
        //      with indentation between them, so the joined body is deliberately not a substring of
        //      the output. Content loss is still caught: a dropped line has no counterpart.
        let normalized = body.replace("\r\n", "\n");
        for line in normalized.split(['\n', '\r']) {
            assert!(emitted.contains(line), "emission dropped part of the comment body {body:?}:\n{emitted}");
        }
    }
}

/// What:
///      Measure container nesting with an explicit stack,
///  counting containers only.
/// Why:
///       A recursive walk would itself overflow on the deepest accepted documents,
///           which is the same trap the crate's edit path already avoids.
///  Scalars are not counted,
///           so a flat record measures 1 and `[[[0]]]` measures 3,
///  matching the parser's bound.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function depthOf(value: JsoncValue): number;
/// ```
pub fn depth_of(value: &JsoncValue) -> usize {
    let mut deepest = 0;
    let mut stack: Vec<(usize, &JsoncValue)> = vec![(1, value)];
    while let Some((depth, current)) = stack.pop() {
        // Only containers count toward the nesting bound: a scalar member sits inside its parent's
        // container rather than opening another one, which is how the parser counts too.
        match &current.kind {
            JsoncKind::Record { entries } => {
                deepest = deepest.max(depth);
                for entry in entries {
                    stack.push((depth + 1, &entry.value));
                }
            }
            JsoncKind::Array { elements } => {
                deepest = deepest.max(depth);
                for element in elements {
                    stack.push((depth + 1, element));
                }
            }
            _ => {}
        }
    }
    return deepest;
}

/// What:
///      Assert the parsed tree respects the published nesting bound.
/// Why:
///       The limit is enforced during parsing,
///           so a tree deeper than the bound means the check moved or broke.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function assertDepthBound(value: JsoncValue): void;
/// ```
pub fn assert_depth_bound(value: &JsoncValue) {
    let measured = depth_of(value);
    assert!(measured <= ACCEPTED_DEPTH, "parsed depth {measured} exceeds the accepted bound {ACCEPTED_DEPTH}");
}

/// What:
///      Assert two trees carry the same shape,
///  scalars,
///  keys and comment bodies.
/// Why:
///       Round-trip and immutability checks both need structural comparison,
///           and the crate deliberately does not implement `PartialEq` on its model.
///           Comment bodies are compared but comment styles are not,
///           because canonical emission re-renders a single-line block comment in `//` form and a
///           reparse then reports the other style;
///           the body is the contract,
///           the delimiter is presentation.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function assertTreesEqual(left: JsoncValue, right: JsoncValue): void;
/// ```
pub fn assert_trees_equal(left: &JsoncValue, right: &JsoncValue) {
    let mut stack: Vec<(&JsoncValue, &JsoncValue)> = vec![(left, right)];
    while let Some((mine, theirs)) = stack.pop() {
        assert_eq!(
            mine.comment.as_ref().map(|comment| return comment.text.clone()),
            theirs.comment.as_ref().map(|comment| return comment.text.clone()),
            "comment bodies differ"
        );
        match (&mine.kind, &theirs.kind) {
            (JsoncKind::Null, JsoncKind::Null) => {}
            (JsoncKind::Boolean { value: one }, JsoncKind::Boolean { value: two }) => assert_eq!(one, two, "boolean values differ"),
            (JsoncKind::Number { identity: one, .. }, JsoncKind::Number { identity: two, .. }) => assert_eq!(one, two, "number identities differ"),
            (JsoncKind::Text { units: one, .. }, JsoncKind::Text { units: two, .. }) => assert_eq!(one, two, "text code units differ"),
            (JsoncKind::Array { elements: one }, JsoncKind::Array { elements: two }) => {
                assert_eq!(one.len(), two.len(), "array lengths differ");
                for index in 0..one.len() {
                    stack.push((&one[index], &two[index]));
                }
            }
            (JsoncKind::Record { entries: one }, JsoncKind::Record { entries: two }) => {
                assert_eq!(one.len(), two.len(), "record member counts differ");
                for index in 0..one.len() {
                    assert_eq!(one[index].key.units, two[index].key.units, "member keys differ at position {index}");
                    assert_eq!(
                        one[index].key.comment.as_ref().map(|comment| return comment.text.clone()),
                        two[index].key.comment.as_ref().map(|comment| return comment.text.clone()),
                        "key comment bodies differ at position {index}"
                    );
                    stack.push((&one[index].value, &two[index].value));
                }
            }
            _ => panic!("value kinds differ: {} versus {}", kind_name(&mine.kind), kind_name(&theirs.kind)),
        }
    }
}

/// What:
///      Name one payload variant for a mismatch message.
/// Why:
///       A failure that says which shapes collided is diagnosable;
///           one that prints two debug trees is not.
fn kind_name(kind: &JsoncKind) -> &'static str {
    if matches!(kind, JsoncKind::Null) {
        return "null";
    }
    if matches!(kind, JsoncKind::Boolean { .. }) {
        return "boolean";
    }
    if matches!(kind, JsoncKind::Number { .. }) {
        return "number";
    }
    if matches!(kind, JsoncKind::Text { .. }) {
        return "text";
    }
    if matches!(kind, JsoncKind::Array { .. }) {
        return "array";
    }
    if matches!(kind, JsoncKind::Record { .. }) {
        return "record";
    }
    return "unknown";
}
