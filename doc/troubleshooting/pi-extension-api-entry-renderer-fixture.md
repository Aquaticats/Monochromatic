# Pi 0.80.6 requires `registerEntryRenderer` on ExtensionAPI verification fixtures

## Symptom

`mise run //package/search-fetch/pi-plugin:lint:types` failed while checking the built-extension verifier:

```text
src/mise.verify-extension.ts(154,9): error TS2741: Property 'registerEntryRenderer' is missing in type '{ ... }'
but required in type 'ExtensionAPI'.
```

The same omission made `mise run //package/search-fetch/pi-plugin:lint:oxlint` fail with
`typescript(TS2741)`.

The trigger is an object literal explicitly typed as `ExtensionAPI` that implements prior API members but omits
`registerEntryRenderer`.

## Root cause

Pi's `ExtensionAPI` declares `registerEntryRenderer` as a required method.
 The Pi 0.80.6 source clone at commit
`bc469b03389135edf5d179ab7718c2085cdfd3a9` has this contract in
`package/coding-agent/src/core/extensions/types.ts:1258-1259`:

```ts
/** Register a custom renderer for CustomEntry. Custom entries do not participate in LLM context. */
registerEntryRenderer<T = unknown>(customType: string, renderer: EntryRenderer<T>): void;
```

Pi's API loader provides the method in
`package/coding-agent/src/core/extensions/loader.ts:273-276`,
 so it is an implemented host API rather than an
optional extension capability:

```ts
registerEntryRenderer<T>(customType: string, renderer: EntryRenderer<T>): void {
  runtime.assertActive();
  extension.entryRenderers ??= new Map();
  extension.entryRenderers.set(customType, renderer as EntryRenderer);
},
```

`package/search-fetch/pi-plugin/src/mise.verify-extension.ts:154` assigns the fake API object to
`ExtensionAPI`.
 That structural assignment exposed the stale fixture.
 The consumer-side repair in `package/search-fetch/pi-plugin/src/mise.verify-extension.ts`
now supplies the required no-op registration method:

```ts
registerEntryRenderer(customType: string,) {
  registrations.push(`entry-renderer:${customType}`,);
},
```

## Verification

Version under test:

- `@earendil-works/pi-coding-agent@0.80.6`
- Pi source clone:
   `https://github.com/earendil-works/pi.git`,
   commit
  `bc469b03389135edf5d179ab7718c2085cdfd3a9`

The failing catalogue contains a fake `ExtensionAPI` object that omits `registerEntryRenderer`.
 Its observed
TypeScript result is `TS2741`,
 quoted in [Symptom](#symptom).

The passing catalogue includes the repaired verifier fixture,
 which retains the required method while the
extension itself registers only its two tools:

```sh
# /var/home/user/Monochromatic
mise run //package/search-fetch/pi-plugin:lint:types
mise run //package/search-fetch/pi-plugin:lint:oxlint
mise run //package/search-fetch/pi-plugin:verify:extension
```

Observed output ends with:

```text
Found 0 warnings and 0 errors.
Pi Search Fetch extension verified: tool:web_search, tool:web_fetch
```

## Verified workaround

Add a method named `registerEntryRenderer` to every full `ExtensionAPI` fake and record its custom type if the
fixture records registrations.
 This restores structural compatibility with Pi 0.80.6 and keeps later required API
additions visible to TypeScript.

Tradeoff:
 the fake accepts registration but does not exercise entry-renderer behavior.
 That is sufficient here
because Pi Search Fetch does not register an entry renderer.
 An extension using one needs a focused verifier for its
renderer registration and invocation path.

## What does not work

- Leaving `registerEntryRenderer` out fails TypeScript and Oxlint with `TS2741`.
- Casting an incomplete object to `ExtensionAPI` would hide required API additions and remove the compatibility check
  that detected this drift.
   It was not adopted.

## Upstream filing artifact

### Upstream filing decision

`.out-of-scope/pi-gpt55-long-context.md` was checked.
 It excludes Pi GPT-5.5 context-capacity work,
 not
ExtensionAPI fixture maintenance.

GitHub searches for `registerEntryRenderer` in `earendil-works/pi` found no matching report.
 The closed issue
`#6326`,
 about custom-message compaction,
 and closed pull request `#5678`,
 about excluding custom messages from
context,
 do not concern `ExtensionAPI` fixture compatibility.

1. **Is it really upstream's fault?
   ** No. Pi declares and implements the required method;
    the consumer fake was
   incomplete.
2. **Can upstream fix it?
   ** Pi could make the method optional,
    but that would weaken its declared API contract and
   would not correct the consumer fixture.
3. **Are they supporting this use case?
   ** Yes.
    Pi documents entry renderers and includes an extension example that
   calls `registerEntryRenderer`.
4. **Would the repo welcome our contribution?
   ** Not for this case.
    Pi's `CONTRIBUTING.md` permits agent assistance
   only with human understanding and disclosure,
    and new-contributor issues are auto-closed by default.
5. **Will they likely fix it?
   ** No upstream fix is appropriate because the observed failure is consumer-owned.
6. **Have we prototyped a minimal fix compatible with their architecture?
   ** No upstream prototype is appropriate.
   The consumer-side method was verified by the package type check,
    Oxlint,
    and built-extension verifier.

Nothing is added to the upstream tracker.
