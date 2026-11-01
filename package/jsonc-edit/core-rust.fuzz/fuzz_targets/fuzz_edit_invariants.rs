//! What:
//!      Edit target:
//!  every edit must be immutable,
//!  addressable and canonical afterwards.
//! Why:
//!       The crate promises that edits return new documents and leave the previous one usable,
//!           that comments stay owned by the address they were attached to,
//!  and that emission stays
//!           a fixed point after any edit.
//!  Those promises are exactly what a fuzzer should attack
//!           with random addresses and random replacement values.

#![no_main]

/// What:
///      Import the harness macro.
/// Why:
///       Every target in this sidecar is a libFuzzer entry point.
use libfuzzer_sys::fuzz_target;
/// What:
///      Import the byte-budget reader.
/// Why:
///       Address selection and the action choice both come from fuzzer bytes.
use arbitrary::Unstructured;
/// What:
///      Import the replacement-value generator.
/// Why:
///       Edits must write the same difficult scalars the document generator produces.
use jsonc_edit_fuzz::generators::replacement_value;
/// What:
///      Import the shared invariants and address selection.
/// Why:
///       Edits are only meaningful at addresses that resolve,
///  and only checkable against the
///           same properties the other targets assert.
use jsonc_edit_fuzz::{
    assert_canonical_stability, assert_comments_preserved, assert_depth_bound, assert_trees_equal, random_path,
    GeneratedDocument,
};
/// What:
///      Import the crate's edit,
///  query and emission surface.
/// Why:
///       The target exercises the API a consumer uses,
///  not internals.
use monochromatic_jsonc_edit::{
    emit_jsonc_value, jsonc_comment, jsonc_delete, jsonc_has, jsonc_key_comment, jsonc_lookup, jsonc_set,
    jsonc_set_comment, jsonc_set_key_comment, parse_jsonc, parse_jsonc_edit, JsoncComment, JsoncCommentKind,
    JsoncPathSegment,
};

fuzz_target!(|input: (GeneratedDocument, Vec<u8>)| {
    let (document, noise) = input;
    let mut unstructured = Unstructured::new(&noise);

    let state = parse_jsonc_edit(&document.source)
        .unwrap_or_else(|error| panic!("generated document rejected: {error}\n{:?}", document.source));
    // A second, independent parse is the reference copy for the immutability assertion.
    let reference = parse_jsonc(&document.source).expect("generated document parses twice");

    let path = random_path(&state.root, &mut unstructured).unwrap_or_default();
    let action = unstructured.int_in_range(0..=4).unwrap_or(0);

    // The value written by a set, kept so the assertion after the match can compare against it.
    let mut written: Option<monochromatic_jsonc_edit::JsoncValue> = None;

    // Every action starts from the untouched state and must leave it byte-identical.
    let edited = match action {
        0 => {
            // Set: write a generated value at the address, carrying the existing comment across so
            // comment preservation can be asserted on the result.
            let Ok(target) = jsonc_lookup(&state.root, &path) else { return };
            // An exhausted byte budget is not a defect, so the input is skipped rather than failed.
            let Ok(mut replacement) = replacement_value(&mut unstructured) else { return };
            replacement.comment = target.comment.clone();
            written = Some(replacement.clone());
            match jsonc_set(&state.root, &path, replacement) {
                Ok(edited) => edited,
                Err(error) => {
                    // The only refusal this target may provoke is the root shape guard: a scalar at
                    // the root would break the container-root contract, so the crate refuses it.
                    assert!(path.is_empty(), "set at a resolved address was refused: {error}");
                    return;
                }
            }
        }
        1 => {
            // Delete: only meaningful away from the root, which the contract refuses to remove.
            if path.is_empty() {
                return;
            }
            jsonc_delete(&state.root, &path).expect("delete at a resolved address succeeds")
        }
        2 => {
            // Attach a multi-line value comment, which forces leading placement during emission.
            let comment = JsoncComment { kind: JsoncCommentKind::Block, text: " fuzz\nnote ".to_string() };
            jsonc_set_comment(&state.root, &path, Some(comment)).expect("comment attaches")
        }
        3 => {
            // Attach a key comment, which only exists where the final segment names a member.
            let Some(JsoncPathSegment::Key { .. }) = path.last() else { return };
            let comment = JsoncComment { kind: JsoncCommentKind::Line, text: " fuzzkey ".to_string() };
            jsonc_set_key_comment(&state.root, &path, Some(comment)).expect("key comment attaches")
        }
        _ => {
            // Clear the value comment at the address.
            jsonc_set_comment(&state.root, &path, None).expect("comment clears")
        }
    };

    // Immutability: the state the edit was taken from must be unchanged.
    assert_trees_equal(&reference, &state.root);

    // The edited document must satisfy the same properties an untouched one does.
    assert_depth_bound(&edited);
    let emitted = emit_jsonc_value(&edited);
    let reparsed = parse_jsonc(&emitted).unwrap_or_else(|error| panic!("edited emission rejected: {error}\n{emitted}"));
    assert_depth_bound(&reparsed);
    assert_comments_preserved(&edited, &emitted);
    assert_canonical_stability(&emitted);

    if let Some(expected) = &written {
        // A set must be visible at the address it was written to, with the value that was written.
        let found = jsonc_lookup(&edited, &path).expect("address still resolves after set");
        assert_eq!(
            format!("{:?}", found.kind).split(' ').next().unwrap_or_default(),
            format!("{:?}", expected.kind).split(' ').next().unwrap_or_default(),
            "set wrote a different kind of value than the address now holds"
        );
    }
    if action == 1 {
        // What: Assert what deletion actually guarantees, per segment kind.
        // Why: Deleting a member removes its name, so the address must stop resolving. Deleting an
        //      element shifts the ones after it, so the same index can still resolve to a different
        //      element; what must hold there is that the parent lost exactly one element.
        let parent = &path[..path.len() - 1];
        let before = jsonc_lookup(&state.root, parent).expect("parent resolves before the delete");
        let after = jsonc_lookup(&edited, parent).expect("parent resolves after the delete");
        match path.last() {
            Some(JsoncPathSegment::Key { .. }) => {
                assert!(!jsonc_has(&edited, &path), "deleted member still resolves");
                let siblings_before = before.entries().expect("record parent").len();
                let siblings_after = after.entries().expect("record parent").len();
                assert_eq!(siblings_after + 1, siblings_before, "member delete did not remove exactly one member");
            }
            Some(JsoncPathSegment::Index { .. }) => {
                let elements_before = before.elements().expect("array parent").len();
                let elements_after = after.elements().expect("array parent").len();
                assert_eq!(elements_after + 1, elements_before, "element delete did not remove exactly one element");
            }
            None => panic!("the root address returns before any delete"),
        }
    }
    if action == 2 {
        let found = jsonc_comment(&reparsed, &path).expect("comment query on reparsed document");
        assert_eq!(found.map(|comment| return comment.text.as_str()), Some(" fuzz\nnote "), "value comment did not survive the round trip");
    }
    if action == 3 {
        let found = jsonc_key_comment(&reparsed, &path).expect("key comment query on reparsed document");
        assert_eq!(found.map(|comment| return comment.text.as_str()), Some(" fuzzkey "), "key comment did not survive the round trip");
    }
});
