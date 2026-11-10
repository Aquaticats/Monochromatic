# Oxlint 1.73 JavaScript plugins can emit safe fixes or suggestions but cannot label a fix as dangerous

## Symptom

A project-owned Oxlint JavaScript rule needs to offer a semantic signature rewrite without allowing ordinary
`oxlint --fix` to apply it.
Oxlint itself distinguishes safe fixes,
suggestions,
and dangerous fixes,
but the JavaScript plugin API exposes only direct fixes and suggestions.

For the readonly-parameter replacement,
a type rewrite must therefore be emitted as a suggestion if it should require an explicit non-safe fix mode.

## Root cause

Verified against `oxc-project/oxc` commit `8de6fcaac7037d37e7f971e67a474b3ae442513a` and the installed
`@oxlint/plugins` 1.73.0 declarations.

The JavaScript rule metadata accepts `fixable: "code" | "whitespace"` and `hasSuggestions`.
It has no dangerous-fix field.
`apps/oxlint/src-js/plugins/rule_meta.ts:29-38`:

```typescript
/**
 * Type of fixes that the rule provides.
 * Must be `'code'` or `'whitespace'` if the rule provides fixes.
 */
fixable?: "code" | "whitespace" | null | undefined;
/**
 * Specifies whether rule can return suggestions.
 * Must be `true` if the rule provides suggestions.
 * @default false
 */
hasSuggestions?: boolean;
```

The diagnostic shape likewise offers one direct `fix` and a `suggest` array,
without a fix-kind field.
The installed `@oxlint/plugins/index.d.ts:3354-3367` declaration is:

```typescript
interface DiagnosticBase {
  message?: string | null | undefined;
  messageId?: string | null | undefined;
  node?: Ranged;
  loc?: LocationWithOptionalEnd | LineColumn;
  data?: DiagnosticData | null | undefined;
  fix?: FixFn;
  suggest?: Suggestion[] | null | undefined;
}
```

The Rust bridge assigns the kind unconditionally.
A direct JavaScript fix becomes `FixKind::Fix`,
and each JavaScript suggestion becomes `FixKind::Suggestion`.
`crates/oxc_linter/src/lib.rs:870-884`:

```rust
// Convert fix
let fix = diagnostic.fixes.and_then(|fixes| create_fix(fixes, FixKind::Fix));

// Convert suggestions (only if fix kind allows suggestions), and combine with fix
let possible_fixes = if let Some(suggestions) = diagnostic.suggestions
    && ctx_host.fix.can_apply(FixKind::Suggestion)
{
    // ...
    let suggestions = suggestions.into_iter().filter_map(|suggestion| {
        create_fix(suggestion.fixes, FixKind::Suggestion)
            .map(|fix| fix.with_message(suggestion.message))
    });
```

A JavaScript plugin cannot send the `Dangerous` bit through this protocol.

## Verification

Versions:

- Oxlint 1.73.0;
- `@oxlint/plugins` 1.73.0;
- `oxlint-tsgolint` 0.24.0.

A disposable plugin emitted one direct fix and one suggestion.
The input was:

```typescript
export const safeCandidate = 1;
export const suggestionCandidate = 2;
```

The direct rule used:

```typescript
context.report({
  node,
  message: 'direct',
  fix(fixer,) {
    return fixer.replaceText(node, 'safeApplied',);
  },
},);
```

The suggestion rule used:

```typescript
context.report({
  node,
  message: 'suggested',
  suggest: [{
    desc: 'Apply suggestion',
    fix(fixer,) {
      return fixer.replaceText(node, 'suggestionApplied',);
    },
  },],
},);
```

Observed catalogs:

- `--fix` changed only `safeCandidate`;
- `--fix-suggestions` changed both the direct fix and suggestion;
- `--fix-dangerously` changed both the direct fix and suggestion;
- no flag left both unchanged and reported diagnostics.

The relevant outputs were:

```text
--fix:
export const safeApplied = 1;
export const suggestionCandidate = 2;

--fix-suggestions:
export const safeApplied = 1;
export const suggestionApplied = 2;

--fix-dangerously:
export const safeApplied = 1;
export const suggestionApplied = 2;
```

## Verified workaround

Emit semantic readonly rewrites through `Diagnostic.suggest` and set `meta.hasSuggestions: true`.
Do not provide `Diagnostic.fix` for the same rewrite.

Tradeoff:
`--fix-suggestions` and `--fix-dangerously` can apply the change,
while ordinary `--fix` cannot.
The JavaScript rule cannot distinguish those two explicit modes.

Editor integrations can present the suggestion as a code action.
The repository's normal `format:oxlint` path currently passes only `--fix`,
so it will not apply the semantic rewrite.
A separate explicit task may expose suggestion application after the implementation is verified.

## What does not work

### Mark `meta.fixable` as dangerous

The metadata union accepts only `code`,
`whitespace`,
null,
or absence.
There is no dangerous value.

### Add a dangerous field to a diagnostic

The JavaScript diagnostic protocol does not declare or serialize such a field.
The Rust bridge derives fix kind from whether the payload came from `fix` or `suggest`.

### Use a direct fix and rely on `--fix-dangerously`

A direct JavaScript fix is tagged `FixKind::Fix`,
so ordinary `--fix` applies it too.
That violates the requirement that semantic signature rewrites require an explicit non-safe mode.

## Upstream filing decision

Oxlint documents three user-facing fix modes,
and documents that JavaScript plugins can provide fixes and suggestions.
It does not promise that JavaScript plugins can label dangerous fixes.
No matching open or closed issue or pull request was found for `JS plugin dangerous fix`.

The filing constraints resolve as follows:

1.  Upstream fault:
    no.
    The supported JavaScript API behaves as declared.
2.  Upstream can fix it:
    yes.
    The protocol could gain an explicit fix kind.
3.  Supported use case:
    no documented dangerous-plugin API exists.
4.  Contribution welcome:
    conditional yes under Oxc's disclosed and reviewed AI-assistance policy.
5.  Likely upstream action:
    unknown;
    no matching tracker signal was found.
6.  Compatible minimal fix prototyped:
    no.
    Suggestions already satisfy the repository requirement,
    so an upstream protocol change is out of scope.

No issue or comment should be filed.
There is nothing additive to post upstream,
and no draft is retained.
