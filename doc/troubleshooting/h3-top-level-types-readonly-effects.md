# H3 2.0.1-rc.24 top-level types with conditional exports left readonly effects opaque

## Symptom

Oxlint's semantic readonly rule reported H3 calls as unresolved package effects:

```text
The function input named "event" is used by these calls: getRouterParam.
The function input named "event" is used by these calls: serveStatic.
The function input named "app" is used as the object for these method calls: app.delete, app.post, app.put.
```

The failures appeared for imported root functions and generated HTTP-method members.
Other packages whose root `exports["."]` included a `types` condition resolved cleanly.

## Root cause

H3 publishes its root declaration separately from its conditional runtime exports.
H3 `package.json:17-27` at tag `v2.0.1-rc.24` contains:

```json
{
  "types": "./dist/_entries/generic.d.mts",
  "exports": {
    ".": {
      "deno": "./dist/_entries/deno.mjs",
      "bun": "./dist/_entries/bun.mjs",
      "workerd": "./dist/_entries/cloudflare.mjs",
      "browser": "./dist/_entries/service-worker.mjs",
      "node": "./dist/_entries/node.mjs",
      "default": "./dist/_entries/generic.mjs"
    }
  }
}
```

The package resolver recognized declaration targets inside an export condition map,
but did not first map a root declaration named by top-level `types` or `typings`.

A second mapping failure occurred after selecting H3's authored `node` runtime condition.
The entry imports `../h3.mjs`,
but TypeScript substitutes adjacent `h3.d.mts` during module resolution.
The generated semantic project therefore loaded declarations instead of the shipped implementation behind the runtime
re-export.
The package effect analyzer correctly failed closed instead of guessing from declarations.

The runtime behavior is not observational in every reported case.
H3 `src/utils/request.ts:330-336` delegates `getRouterParam` to `getRouterParams`:

```ts
export function getRouterParam(
  event: HTTPEvent,
  name: string,
  opts: { decode?: boolean } = {},
): string | undefined {
  const params = getRouterParams(event, opts);
  return params[name];
}
```

`getRouterParams` calls `getEventContext` at `src/utils/request.ts:182-195`.
`getEventContext` initializes missing request context at `src/utils/event.ts:27-34`:

```ts
if ((event as H3Event).context) {
  return (event as H3Event).context as T;
}
event.req.context ??= {};
return event.req.context as T;
```

`serveStatic` changes response state directly.
For example,
`src/utils/static.ts:65-85` writes response headers:

```ts
for (const [key, value] of entries) {
  event.res.headers.set(key, value);
}

if (event.req.method !== "GET" && event.req.method !== "HEAD") {
  if (options.fallthrough) {
    return;
  }
  event.res.headers.set("allow", "GET, HEAD");
  throw new HTTPError({ status: 405 });
}
```

H3 also installs HTTP-method helpers dynamically.
`src/h3.ts:235-244` forwards `post`,
`put`,
and `delete` to `on`,
while `src/h3.ts:178-193` adds each route and retained handler to application state.

The problem combined declaration-to-runtime mapping with two intentional fail-closed cases.
Bundled `serveStatic` loses enough primitive type information that `path.slice` remains unresolved,
and H3 creates HTTP-method members dynamically on the prototype.
Neither residual can be treated as observational.
This is not an H3 defect.

## Verification

The installed package was `h3@2.0.1-rc.24`.
The audited upstream tag resolves to commit `20bf346b7886d174625351db1f84d28a41d17853`.
The installed `dist/h3.mjs` SHA-256 digest was
`c3540d2742e64b094c7e4f5823bdc7c09b3fb67a6b1c9d0d3452e86e6e178611`.

Run the focused verification:

```sh
mise run //package/oxlint-plugin/prefer-readonly-parameter-type:build
mise run //package/oxlint-plugin/prefer-readonly-parameter-type:test:unit
OXLINT_THREADS=1 mise run //package/done/app:lint:oxlint
OXLINT_THREADS=1 mise run //package/done/variant-postcss:lint:oxlint
```

### Patterns that resolve

After the fix,
the package analyzer follows top-level root declarations into conditional runtime exports and follows explicit relative
runtime forwarding even when TypeScript selects an adjacent declaration.
`getRouterParam` resolves transitively to the mutation in `getEventContext`.

The disposable package fixture also resolves an imported runtime binding that is re-exported from a barrel while an
adjacent declaration shadows the runtime module under ordinary TypeScript resolution.

### Patterns that still fail closed

The resolver still rejects:

- `serveStatic` effects that originate from untyped bundled values such as `path.slice`;
- generated H3 members such as `post`,
  `put`,
  and `delete`,
  whose declarations do not correspond to authored class methods;
- a top-level declaration path that cannot be connected to one exact export;
- an installed version absent from the governing lockfile;
- an export without one supported shipped runtime target;
- a native or missing implementation;
- a runtime call whose effects remain unresolved after transitive analysis.

Complete boundary contracts account for the remaining H3 effects.
Both Done package runs then report only the separately unresolved `database.exec` calls.
Repeated unchanged runs complete below the 10-second acceptance limit.

## Verified workarounds

### Resolve declarations and explicit runtime forwarding separately

`packageModuleSpecifierForDeclaration` compares the exact root declaration against top-level `types` or `typings`
before walking subpath declaration conditions.
Runtime selection still follows package-authored condition order.

The generated implementation project loads shipped package runtime files as roots.
When TypeScript substitutes an adjacent declaration for a relative JavaScript import,
`runtimeForwardedExport` follows only explicit named import and export syntax to an exact package-local runtime path.
The runtime bytes remain effect authority.

The disposable package fixture in
`package/oxlint-plugin/prefer-readonly-parameter-type/src/effect-summaries.unit.test.ts`
uses top-level `types`,
separate runtime conditions,
a runtime barrel,
and an adjacent declaration that shadows the barrel's implementation dependency.
It proves observational,
mutating,
callback,
source-map,
and runtime-forward behavior.

Tradeoff:
the project loads every shipped runtime source file in the demanded package.
Dynamic module paths,
nonrelative forwarding,
and unsupported export syntax remain opaque.

### Keep contracts at an unresolved consumer boundary

A consumer can document residual H3 effects with complete `@mutates` contracts at the narrow boundary.
Done contracts `serveStatic` inside its dedicated handler and contracts route registration on the function receiving the
H3 application.

Tradeoff:
contracts preserve uncertainty rather than proving exact effects.
They must name every residual provenance and remain accurate after package upgrades.

## What does not work

### Treat H3 declarations as behavior evidence

The declarations expose signatures but cannot prove that `getRouterParam` initializes context,
that `serveStatic` writes response state,
or that generated route methods retain handlers.

### Add a handwritten H3 major-version catalog entry

A major-only entry would outlive the exact prerelease implementation and source digest.
It would bypass the installed-version,
lockfile,
runtime-export,
and shipped-source gates.

### Remove every contract after runtime entry resolution

Selecting a runtime entry does not prove every transitive call.
Bundled type erasure and dynamic prototype installation remain opaque after the entry itself is inspectable.
Removing those residual contracts restores diagnostics.

## Upstream filing decision

No `.out-of-scope/` entry matched H3 or package export mapping.
Open and closed H3 issue and pull-request searches for `source maps dist package` returned no matches.

1.  **Upstream fault**
    No.
    H3's top-level `types` and conditional runtime exports are a valid authored package layout.
    The project resolver omitted that composition.
2.  **Upstream fixability**
    H3 could duplicate a `types` condition inside `exports["."]`,
    but requiring that change would work around a downstream resolver limitation rather than fix H3 behavior.
3.  **Supported use case**
    H3 explicitly publishes declarations and environment-specific runtime entries.
4.  **Contribution policy**
    The audited tag contains no `CONTRIBUTING.md` or issue template,
    and its repository policy files contain no AI-assistance ban.
    This does not override the failed fault criterion.
5.  **Fix likelihood**
    Not applicable because no H3 defect is claimed.
6.  **Compatible prototype**
    The project-side resolver fix is implemented and covered by the disposable package fixture.
    An upstream prototype is neither necessary nor appropriate because criterion 1 fails.

There is no upstream issue or comment to file,
so the filing artifact is intentionally empty.

## Sources

- [H3 `v2.0.1-rc.24` package manifest][h3-package]
- [H3 event context source][h3-event]
- [H3 request utilities][h3-request]
- [H3 static serving source][h3-static]
- [H3 route registration source][h3-routes]

[h3-package]: https://github.com/h3js/h3/blob/20bf346b7886d174625351db1f84d28a41d17853/package.json
[h3-event]: https://github.com/h3js/h3/blob/20bf346b7886d174625351db1f84d28a41d17853/src/utils/event.ts
[h3-request]: https://github.com/h3js/h3/blob/20bf346b7886d174625351db1f84d28a41d17853/src/utils/request.ts
[h3-static]: https://github.com/h3js/h3/blob/20bf346b7886d174625351db1f84d28a41d17853/src/utils/static.ts
[h3-routes]: https://github.com/h3js/h3/blob/20bf346b7886d174625351db1f84d28a41d17853/src/h3.ts
