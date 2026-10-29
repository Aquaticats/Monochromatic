# Slint 1.18.1 rejects `dialog` as an `accessible-role` value

## Symptom

The first search overlay declaration used `accessible-role: dialog`.
`slint-viewer 1.18.1 --check` rejected it with:

```text
Unknown unqualified identifier 'dialog'
```

The disposable minimal declaration also emitted:

```text
The `accessible-role` property must be a constant expression
```

This was an application declaration error,
not evidence that native key capture or search result rendering was broken.

## Root cause

The inspected upstream checkout is Slint v1.18.1,
commit `372cf0ee5577c3dfec309a45e7b778ba4e81b734`.
Its `internal/common/enums.rs:466` defines `AccessibleRole`.
The complete enumeration has no `Dialog` member.
`internal/common/enums.rs:525` through `530` includes the supported landmarks:

```rust
// Slint internal/common/enums.rs
/// Landmark: a generic section significant enough to be listed in a summary.
/// Use a more specific landmark if one applies.
Region,
/// Landmark: a region containing controls for searching or filtering content.
Search,
```

The identifier resolver in `internal/compiler/passes/resolving.rs:2597` reports an unresolved name:

```rust
// Slint internal/compiler/passes/resolving.rs
ctx.diag.push_error(
    format!("Unknown unqualified identifier '{}'{hint}", first.text()),
    &node,
);
```

Accessibility lowering in `internal/compiler/passes/lower_accessibility.rs:23` expects an enumeration value.
Its other branch emits the second diagnostic:

```rust
// Slint internal/compiler/passes/lower_accessibility.rs
if let Expression::EnumerationValue(val) = role.value_expression() {
    debug_assert_eq!(val.enumeration.name, "AccessibleRole");
    debug_assert_eq!(val.enumeration.values[0], "none");
    if val.value == 0 {
        return;
    }
} else {
    diag.push_error(
        "The `accessible-role` property must be a constant expression".into(),
        &*role,
    );
}
```

An unsupported role name does not become valid merely because its spelling is a constant token.
The selected search landmark describes this UI's purpose without inventing a toolkit enumeration member.

## Verification

Run the package-owned disposable markup controls:

```sh
# package/ide/app/mise.toml
mise run //package/ide/app:inspect:search-accessibility
```

Observed results with `slint-viewer 1.18.1`:

- `accessible-role: search`: status `0`, no diagnostic.
- `accessible-role: region`: status `0`, no diagnostic.
- `accessible-role: dialog`: status `1`, both diagnostics quoted in “Symptom”.

The recorded run is `/tmp/ide-search-accessibility-VVu3o7/results.json`.
The task prints its newly created artifact directory on each invocation.
The application also passes `lint:slint`,
and the native MCP element tree exposes the overlay as `Search`.
Native keyboard activation and result navigation passed in `proc_c872`.
This does not establish screen-reader modal semantics or an AccessKit focus trap.

## Verified workaround

Use the existing search landmark:

```diff
# package/ide/app/ui/search.slint
-    accessible-role: dialog;
+    accessible-role: search;
```

The search overlay separately manages keyboard focus,
Escape dismissal,
and restoration to its previous source/tree destination.
The semantic tradeoff is explicit:
a search landmark is not a modal-dialog accessibility role.
Do not claim the role supplies modal behavior.

## What does not work

- Using an unqualified `dialog` token fails identifier resolution.
- Treating the constant-expression diagnostic as the sole problem misses the unsupported enumeration member.
- Passing native headless tests does not replace the real nested-seat keyboard and clipboard checks.

## Upstream filing decision

Nothing to file or add as an upstream comment for this application declaration error.
The `.out-of-scope/` directory was checked;
no Slint exemption applies.

The issue searches `accessible dialog role` and `accessibility dialog` returned no matches.
The broader PR search `accessible role` found the related [landmark-role addition][landmarks].
Its body explicitly lists the added landmarks,
including `search`,
and contains no comments.
That is implementation precedent for the supported value,
not proof of a missing promised dialog feature.

1.  Upstream fault: not demonstrated.
    The application requested an absent enumeration member.
2.  Fixability: another role could be a feature proposal,
    but no compiler correction is required to use the existing search landmark.
3.  Supported use: the search landmark is explicitly supported by the enumeration and positive control.
    No claim of promised dialog-role support is made.
4.  Contribution policy: no feature proposal is being prepared,
    so contribution receptiveness was not reassessed for one.
5.  Corrective change: none is requested for rejecting this unsupported token.
6.  Prototype: none is warranted for an upstream defect because the first condition is unmet.
    The consumer correction is built and exercised instead.

[landmarks]: https://github.com/slint-ui/slint/pull/11831
