# Slint 1.18.1 rejects component properties named `padding` or `maximum-width`, and a `text` property hides `accessible-role: text`

## Symptom

While building the IDE's diagnostic card (`package/ide/app/ui/annotation.slint`),
three declarations failed `slint-viewer 1.18.1 --check`.

A component inheriting `Rectangle` that declares `private property <length> padding: 8px;` fails with:

```text
error: Cannot override property 'padding'
```

A component inheriting `Rectangle` that declares `in property <length> maximum-width: 560px;` fails with:

```text
error: Cannot override property 'maximum-width'
```

A component that declares `in property <string> text;`,
and sets `accessible-role: text;` on any element inside it,
fails with two errors:

```text
error: Cannot convert string to AccessibleRole
error: The `accessible-role` property must be a constant expression
```

`Rectangle` has no `padding` or `maximum-width` property in the documentation of its own properties,
and `text` is a documented `AccessibleRole` value,
so none of the three names looks taken.

## Root cause

Source is the published crate `i-slint-compiler` 1.18.1
(registry archive SHA-256 `f55d48ebef05e2bf09ba5a1fc67bcb11033b9e12a2a50ed3105c7d25e2b539d0`).

### Reserved layout properties

Every element carries the layout properties of `RESERVED_LAYOUT_PROPERTIES`,
whether or not it is inside a layout,
and `padding` is one of them
(`typeregister.rs:28` and `typeregister.rs:33`):

```rust
// i-slint-compiler-1.18.1/typeregister.rs
pub const RESERVED_LAYOUT_PROPERTIES: &[(&str, Type)] = &[
    ("min-width", Type::LogicalLength),
    // ...
    ("padding", Type::LogicalLength),
```

A property declaration whose name already resolves to a member is a conflict,
reported with the override message (`object_tree.rs:1755` and `object_tree.rs:1766`):

```rust
// i-slint-compiler-1.18.1/object_tree.rs
if let MemberDeclaration::Conflict { existing_type, declared_in } = &declaration {
    match existing_type {
        // callback and function cases elided
        _ => diag.push_error(
            cannot_override_message(Some("property"), &unresolved_prop_name, declared_in),
            &name_token,
        ),
```

### Deprecated size aliases

`maximum-width` is not a property of its own.
The property lookup maps the old spellings `minimum-width`, `maximum-width`, `minimum-height`, and `maximum-height`
onto `min-width` and the others (`typeregister.rs:400` to `typeregister.rs:409`):

```rust
// i-slint-compiler-1.18.1/typeregister.rs
// Report deprecated known reserved properties (maximum_width, minimum_height, ...)
for pre in &["min", "max"] {
    if let Some(a) = name.strip_prefix(pre) {
        for suf in &["width", "height"] {
            if let Some(b) = a.strip_suffix(suf)
                && b == "imum-"
            {
                return PropertyLookupResult {
                    property_type: Type::LogicalLength,
                    resolved_name: format!("{pre}-{suf}").into(),
```

So `maximum-width` resolves to an existing member and takes the same conflict path.
Slint issue [#6324](https://github.com/slint-ui/slint/issues/6324) is the same mechanism for `color`,
an old alias of `Rectangle`'s `background`;
it was closed after a maintainer explained the alias.

### Lookup order for an unqualified enumeration value

An unqualified identifier is looked up through a fixed chain,
and properties in scope come before the values of the expected enumeration
(`lookup.rs:1107` to `lookup.rs:1120`):

```rust
// i-slint-compiler-1.18.1/lookup.rs
pub fn global_lookup() -> impl LookupObject {
    (
        LocalVariableLookup,
        (
            ArgumentsLookup,
            (
                SpecialIdLookup,
                (
                    IdLookup,
                    (
                        InScopeLookup,
                        (
                            LookupType,
                            (BuiltinNamespaceLookup, (TypeSpecificLookup, BuiltinFunctionLookup)),
```

In a component written in the current syntax,
`InScopeLookup` finds the properties declared on elements in scope,
the component root among them,
through `declaration(name)` (`lookup.rs:495` to `lookup.rs:513`):

```rust
// i-slint-compiler-1.18.1/lookup.rs
|elem| {
    let elem_borrow = elem.borrow();
    elem_borrow.declaration(name).map(|(internal_name, prop)| {
        expression_from_reference(
            NamedReference::new(elem, internal_name.clone()),
            &prop.property_type,
            None,
        )
    })
},
```

A declared `in property <string> text` therefore wins over the `AccessibleRole` value `text`,
the string cannot convert to the enumeration,
and accessibility lowering then reports the non-constant role.
A builtin `Text` element's own `text` property is not a declaration,
so `Text { accessible-role: text; }` alone still compiles;
only a declared property named like an enumeration value hides it.

## Verification

`slint-viewer 1.18.1` (`slint-viewer --version`) checked one disposable file per case
(`~/temp/agent/slint-names.dZaxzakn/results.json`):

```js
// ~/temp/agent/slint-names.dZaxzakn, run with node --input-type=module
const cases = {
  'text-role-in-text': 'export component Case inherits Window { Text { text: "x"; accessible-role: text; } }',
  'text-role-qualified-in-text': 'export component Case inherits Window { Text { text: "x"; accessible-role: AccessibleRole.text; } }',
  'text-role-in-rectangle': 'export component Case inherits Window { Rectangle { accessible-role: text; } }',
  'text-role-in-component-with-text-property': 'component Card inherits Rectangle { in property <string> text; Rectangle { accessible-role: text; } }\nexport component Case inherits Window { Card { } }',
  'padding-property': 'component Padded inherits Rectangle { private property <length> padding: 8px; }\nexport component Case inherits Window { Padded { } }',
  'maximum-width-property': 'component Widest inherits Rectangle { in property <length> maximum-width: 560px; }\nexport component Case inherits Window { Widest { } }',
  'renamed-properties': 'component Renamed inherits Rectangle { in property <length> widest: 560px; private property <length> inset: 8px; }\nexport component Case inherits Window { Renamed { } }',
};
// Each body is written to <name>.slint and checked with: slint-viewer --check <name>.slint
```

Declarations that compile:

- `Text { text: "x"; accessible-role: text; }`
- `Text { text: "x"; accessible-role: AccessibleRole.text; }`
- `Rectangle { accessible-role: text; }`
- properties named `widest` and `inset` on a component inheriting `Rectangle`

Declarations that fail with `Cannot override property '<name>'`:

- `private property <length> padding` on a component inheriting `Rectangle`
- `in property <length> maximum-width` on a component inheriting `Rectangle`

Declarations that fail with `Cannot convert string to AccessibleRole`
and `The accessible-role property must be a constant expression`:

- `accessible-role: text` inside a component that declares `in property <string> text`

## Verified workarounds

- Name component properties outside the reserved layout set and the deprecated `minimum-` and `maximum-` spellings:
  the card uses `widest` and `inset`.
  The cost is a less conventional name.
- Qualify the enumeration value as `accessible-role: AccessibleRole.text;`,
  which bypasses the in-scope lookup.
  The card's `Text` uses this form;
  it costs only verbosity and stays correct if a `text` property is added later.

## What does not work

- Fixing one of the two names is not enough:
  the first check of the card reported both `Cannot override` errors at once,
  because the alias is resolved separately from the reserved table.
- Setting `accessible-role: text` on a child element does not help:
  the component root's declared `text` is in scope for every element inside the component,
  as the `text-role-in-component-with-text-property` case shows with a child `Rectangle`.

## Upstream filing decision

`.out-of-scope/` has no Slint entry,
so the filing check applies.
Searches of `slint-ui/slint` issues for `Cannot override property`, `maximum-width deprecated`,
`Cannot convert string to AccessibleRole`, `property shadows enum`, and `padding reserved property component`
found only #6324.

1. Upstream's fault: no for the behavior.
   The reserved names, the alias mapping, and the lookup order are deliberate,
   and #6324 was answered as intended behavior.
   At most the messages could name the colliding alias or the shadowing property.
2. Upstream can fix it: yes for the message wording.
3. Supported use case: yes, components declaring their own properties are the main use.
4. Contribution welcome: not checked further, because constraint 1 fails.
5. Likely to fix: leaning no for the behavior, per #6324.
6. Prototype: not attempted, because constraint 1 fails.

Do not file.
The draft is kept as a record.

~~~md
Title: Name the alias or shadowing property in "Cannot override property" and enum conversion errors

Labels: a:compiler, enhancement

Declaring `in property <length> maximum-width` on a component inheriting `Rectangle` reports
`Cannot override property 'maximum-width'`, although `Rectangle` documents no such property;
it is the deprecated spelling of `max-width` (`typeregister.rs:400`). A component that declares
`in property <string> text` makes `accessible-role: text` in any of its elements resolve to that property
(`lookup.rs:1107`, `InScopeLookup` before `TypeSpecificLookup`), reported only as
`Cannot convert string to AccessibleRole`.

Suggested fix: when the conflict comes from the deprecated-alias branch, say which property it aliases;
when an unqualified identifier in an enumeration-typed binding resolves to a property while the enumeration
has a value of the same name, add a hint to write `AccessibleRole.text`.
~~~
