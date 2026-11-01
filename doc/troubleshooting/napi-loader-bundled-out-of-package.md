# Sätteri 0.10.5 throws `Cannot find native binding`
when rolldown inlines its napi loader into another package's `dist/final/node/`,
because the loader resolves bindings from `import.meta.url`

A napi loader finds its platform binding relative to its own file.
Bundling copies that file somewhere else.
Once copied,
 the loader looks for `@bruits/satteri-<platform>` beside the copy,
where pnpm has linked nothing,
 and throws a message naming npm and optional dependencies.
Both of those are red herrings.

This is not specific to Sätteri.
Any napi package that the shared Node rolldown config inlines reproduces it,
because the config inlines every `@monochromatic-dev/**` workspace package
and every undeclared transitive those packages reach.

## Symptom

Importing a workspace package that imports `@monochromatic-dev/cli-markdown-lint`
throws before a line of the importing package's own code runs:

```text
Cannot find native binding. npm has a bug related to optional dependencies
(https://github.com/npm/cli/issues/4828). Please try `npm i` again after
removing both package-lock.json and node_modules directory.
```

Three properties make this hard to place:

- The message names npm,
   and this repository uses pnpm.
- The message names optional dependencies,
   and the bindings are installed correctly.
- It fails at import time,
   not build time,
  so the build that created the broken artifact reports success.

The trigger is importing the package at all.
It does not matter which export the consumer uses:
`@monochromatic-dev/module-translation-repair` reaches only for
`fixSource`,
 `Rule`,
 and `rulesById`,
 none of which parse anything,
and still throws.

## Root cause

### The cause in one sentence

Inlining a package's source carries its imports but not its declarations.

A build asks one question of its own `package.json`:
what do you declare?
Declared names stay as `import` statements in the output.
Undeclared names are copied into it.

`package/cli/markdown-lint/package.json` declares `satteri`,
so every build of the linter emits an import rather than a copy.
Calling `packageExternals` against that manifest directly shows it:

```text
markdown-lint externals: /^node:/u /^ignore(\/|$)/u /^satteri(\/|$)/u /^type-fest(\/|$)/u
satteri external? true
```

That covers both of the linter's artifacts,
the `bin` and the library entry,
so the linter is not the problem and never was.

The consumer inlines the linter's source,
 not its manifest.
The copied source still carries `import { markdownToMdast } from 'satteri'`,
but the build now asks the consumer's manifest whether `satteri` is declared,
gets no,
 and copies Sätteri in as well.
The declaration and the code that needs it have been separated.

Two consequences worth stating,
 because both were misread during this
investigation.

The linter being a CLI does not matter.
General guidance splits bundling by artifact kind,
 tools versus libraries
("Prior art on bundling a native dependency"),
and the linter is both,
 having a `bin` and an `exports` map.
That split decides nothing here,
because the linter's own build externalizes `satteri` either way.
The artifact-kind question applies to the consumer,
which is a published library inlining another package's source.

Sätteri being native is not the cause either.
It is what makes the failure loud.
Any inlined package is separated from its manifest the same way;
a pure-JavaScript one is silently duplicated into the bundle instead of throwing,
which is the de-duplication cost appearing as a quiet problem rather than a crash.

### The loader resolves relative to itself

`node_modules/.pnpm/satteri@0.10.5/node_modules/satteri/index.js:6`:

```js
import { createRequire } from 'module'
const require = createRequire(import.meta.url)
```

Every binding lookup goes through that `require`,
 so every lookup is anchored at
whatever file the loader currently sits in.
The Linux x64 gnu branch,
 `index.js:292`:

```js
const binding = require('@bruits/satteri-linux-x64-gnu')
```

Failures accumulate in `loadErrors` rather than throwing at the site,
and the message is assembled once at the end,
 `index.js:583`:

```js
if (!nativeBinding) {
  if (loadErrors.length > 0) {
    const error = new Error(
      `Cannot find native binding. ` +
        `npm has a bug related to optional dependencies (https://github.com/npm/cli/issues/4828). ` +
        'Please try `npm i` again after removing both package-lock.json and node_modules directory.',
    )
```

That text is napi-rs generated boilerplate.
It describes one historical cause of a missing binding and hard-codes it as the only cause,
which is why it points away from the real one.

### The build copies the loader out of its package

`package/config/rolldown/src/index.node.ts` inlines every workspace package:

```ts
export const NODE_ALWAYS_BUNDLE: readonly string[] = [
  '@monochromatic-dev/**',
  'nano-spawn',
];
```

`package/config/rolldown/src/package-externals.ts` then keeps external only what the
consuming manifest declares,
 and states the consequence in its own TSDoc:

```text
every declared dependency and peer dependency stays external
unless a bundle pattern forces it inline,
`node:` builtins stay external,
and undeclared bare imports (transitives of inlined workspace source)
bundle by omission so artifacts stay self-contained outside the monorepo.
```

The filter that implements it:

```ts
const externalNames = Object.keys({
  ...manifest.dependencies,
  ...manifest.peerDependencies,
})
  .filter(function isExternalName(name: string): boolean {
  return !alwaysBundle.some(/* ... */);
});
```

`satteri` is a transitive of inlined workspace source and is not in the consumer's
manifest,
 so it is not in `externalNames`,
 so it bundles by omission.
Bundling by omission is the deliberate design,
 not an oversight:
it is what makes built artifacts self-contained outside the monorepo.

### The reach from the consumer to the loader

`package/cli/markdown-lint/src/parse.ts:5` is the only file in the linter that
imports `satteri`.
The consumer still reaches it through the barrel,
`package/cli/markdown-lint/src/index.ts:15`:

```ts
export { parse, } from './parse.ts';
```

Tree-shaking does not remove it,
 because
`package/cli/markdown-lint/package.json` declares `"sideEffects": true`.

So the loader's bytes are written into the consumer's `dist/final/node/`.
`import.meta.url` in the copy names that directory.
Resolution walks up from there and finds no `@bruits/*`,
because `hoist: false` and `nodeLinker: isolated` link those under Sätteri's own
location and nowhere else.

### Readings that were wrong

**"npm and optional dependencies."**
The message says so and it is wrong twice over.
The bindings are installed:
 `node_modules/.pnpm` carries
`@bruits+satteri-linux-x64-gnu` and its siblings for every architecture in
`supportedArchitectures`.
The text is a fixed string in the generated loader,
 not a diagnosis of the actual failure.

**"`packageExtensions` will fix it."**
`packageExtensions` adds dependencies to another package's manifest so pnpm links them
under that package.
It cures a package importing something it never declared,
which under isolated linking is unresolvable from that package's own location.
`pnpm-workspace.yaml` carries exactly one such entry,
 `mitata` reaching for
`@mitata/counters`.
This failure has the opposite shape.
Nothing is missing from Sätteri's location.
The code was copied out of that location,
 and the copy never consults Sätteri's manifest.

## Verification

Version under test:
 `satteri@0.10.5`,
 at
`node_modules/.pnpm/satteri@0.10.5/node_modules/satteri/`.
The failure was first seen on `0.9.5`.
It was re-verified on `0.10.5` because the mechanism is version independent:
it depends on `createRequire(import.meta.url)`,
 which every napi-rs loader emits.

The harness is a matched pair.
The same bytes run from two directories,
 with opposite outcomes,
which isolates location as the only variable and needs no build.

Positive control,
 the loader in its own package:

```bash
cd package/cli/markdown-lint
node --input-type=module -e "const m = await import('satteri'); console.log('OK:', typeof m.markdownToMdast);"
```

```text
OK: function
```

Failing case,
 the same file copied elsewhere,
which is what the bundler does:

```bash
mkdir --parents "${HOME}/temp/agent"
chmod 700 "${HOME}/temp/agent"
probe="$(mktemp --directory "${HOME}/temp/agent/issue447-probe.XXXXXXXX")"
cp node_modules/.pnpm/satteri@0.10.5/node_modules/satteri/index.js "${probe}/copied-loader.mjs"
node "${probe}/copied-loader.mjs"
```

```text
Error: Cannot find native binding. npm has a bug related to optional dependencies
(https://github.com/npm/cli/issues/4828). Please try `npm i` again after removing
both package-lock.json and node_modules directory.
    at file:///.../issue447-probe.AupOsxC7/copied-loader.mjs:585:19
```

What works:

- Importing `satteri` from any package that declares it,
   so pnpm links it and its
  `@bruits/*` bindings under that package.
- Running the linter's own binary,
   `package/cli/markdown-lint/dist/final/node/cli.mjs`,
  which sits inside the package whose manifest declares `satteri`.
- Spawning that binary from another package,
   which is what
  `package/git-policy/markdown-lint` does,
   and why it has never hit this.

What fails:

- Any file containing the loader's source that lives outside a directory from which
  `@bruits/satteri-<platform>` resolves.
  That is the whole failing set,
   and the copied-loader probe is its minimal case.

## Verified workarounds

**Ship the binding beside the bundle,
 which keeps the artifact self-contained.**
Each platform arm of the loader tries a sibling file before it tries the binding package.
`index.js:287`,
 ahead of the `@bruits/*` require at `index.js:292`:

```js
return require('./satteri_napi.linux-x64-gnu.node')
```

So a bundle that inlines the loader works as long as the matching `.node` sits next to
the emitted file.
Verified in the same probe directory that produced the failure:
dropping `satteri_napi.linux-x64-gnu.node` beside `copied-loader.mjs` turns the throw
into working exports.

```bash
cp node_modules/.pnpm/@bruits+satteri-linux-x64-gnu@0.10.5/node_modules/@bruits/satteri-linux-x64-gnu/satteri_napi.linux-x64-gnu.node "${probe}/"
node --input-type=module -e "const m = await import('${probe}/copied-loader.mjs'); console.log(typeof m.parseExpression, typeof m.walkMdastHandle);"
```

```text
function function
```

This is the only workaround that needs nothing from the consumer:
no manifest declaration,
no external import,
and no knowledge of which parser the linter uses.

Tradeoff is artifact size,
and it is set by how many architectures the artifact must run on.
Measured at `0.10.5`,
 the bindings run 3.4M to 4.2M each,
and the six that `supportedArchitectures` installs
(`x64`,
 `arm64`,
 `current` across `glibc`,
 `musl`,
 `current` on `darwin`,
 `linux`,
`current`) total 23M.
Shipping only the build host's binding costs 4.2M and makes the artifact host specific,
which is fine for a private consumer and wrong for a published one.

Do not reach for base64 inlining to avoid the sibling file.
The bytes can ride inside the JavaScript,
but `process.dlopen` accepts only a filesystem path,
so an inlined binding still has to be written to a real file before it loads.
That adds an import-time write,
 a cache directory to manage,
 and a failure mode on
`noexec` mounts,
 to arrive where the sibling file already is.

**Declare the transitive in the consuming manifest.**
Adding `"satteri": "catalog:"` to the consumer moves it into `externalNames`,
so the bundle emits a runtime `import` instead of a copy,
and node resolves it from the consumer's own `node_modules`,
where Sätteri sits beside the `@bruits/*` links its loader needs.
Verified by reading the built bundle:
 one match for `from"satteri"`,
zero files containing `Cannot find native binding`,
zero matches for `@bruits/satteri-[a-z0-9-]*`.

Tradeoff:
 the consumer's manifest names a native package its own source never mentions,
and the requirement is discoverable only by hitting the crash.
For a package that is published,
 such as
`@monochromatic-dev/module-translation-repair` (`"private": false`),
the declaration is also simply accurate,
because that package really does need Sätteri present at runtime.
For a private consumer it is pure noise.

**Spawn the CLI instead of importing the package.**
`package/git-policy/markdown-lint` runs `cli-markdown-lint --fix` as a subprocess.
Tradeoff:
 process startup per invocation and no in-process API,
in exchange for total immunity,
 since the binary never leaves its own package.

**`NAPI_RS_NATIVE_LIBRARY_PATH`.**
The loader honours it at `index.js:70`:

```js
return require(process.env.NAPI_RS_NATIVE_LIBRARY_PATH);
```

Tradeoff:
 an absolute path fixed per machine and per architecture.
Usable to confirm a diagnosis,
 not shippable.

## What does not work

**`packageExtensions`.**
Wrong shape,
 for the reason walked in "Readings that were wrong".

**Relying on tree-shaking.**
The consumer touches no parsing export,
 but
`package/cli/markdown-lint/package.json` sets `"sideEffects": true`,
so rolldown keeps the `parse` re-export at
`package/cli/markdown-lint/src/index.ts:15` and everything it reaches.

**Carving `@monochromatic-dev/cli-markdown-lint` out of `NODE_ALWAYS_BUNDLE`,
while the linter stays private.**
Making the linter external emits `import ... from "@monochromatic-dev/cli-markdown-lint"`,
which resolves to the linter's own directory,
where `satteri` is declared and its bindings link correctly.
That genuinely fixes the resolution,
and it is what the prior art recommends.
It is blocked only by publication status:
`package/cli/markdown-lint/package.json:3` sets `"private": true`,
so a package with `"private": false` cannot carry a runtime dependency on it,
and the consumer must inline it or not use it.
Once the linter publishes,
 this stops being blocked and becomes the correct fix.

**Injecting the parser as a function parameter.**
Moving the `parse` call out of `runRules` (`package/cli/markdown-lint/src/lint.ts:2`)
and into a caller-supplied argument does remove the import edge,
and therefore does remove the failure.
It was rejected on API grounds:
it puts the choice of parser into every consumer's call site,
so each consumer must know which parsers the linter supports
in order to use a linter rule.
Recorded here so it is not re-proposed as the obvious fix.

## Prior art on bundling a native dependency

Both the vendor of the native package and the general library guidance say the same
thing,
 and they say the opposite of what this repository's Node config currently does
for library builds.

napi-rs,
 the toolchain that generates this loader,
 states it directly in its
troubleshooting guide (<https://napi.rs/docs/more/troubleshooting>):

> Keep native packages external to server bundles.

The same page explains why the message we saw is uninformative,
and what to do instead:

> The generated loader records native-candidate load failures in an error `cause` chain.
> Print it instead of reporting only "Cannot find native binding".

The e18e writeup on bundling dependencies
(<https://e18e.dev/blog/bundling-dependencies>) draws the line by artifact kind:

> If you're building a library,
>  you generally should not be bundling your dependencies.

> If you're building a developer tool (e.g. a CLI),
>  it may make sense to bundle
> long-term.

It names three costs a bundled library imposes on its consumers:
npm de-duplication stops working because the installer cannot see the bundled copies,
the bundled dependencies freeze at their bundling-time versions so shipping their
security updates becomes our burden,
and the dependency tree is hidden rather than reduced.

Measured against this repository:
eight packages emit to `bundle/node`,
all of them under `package/claude-code-plugin/`,
and those are the developer-tool case where bundling is correct.
The other seventy-eight `rolldown.node.config.ts` files build libraries to
`dist/final/node`,
where the guidance says workspace and native dependencies should stay external.
`NODE_ALWAYS_BUNDLE` currently applies the tool rule to all of them.

This only became visible now because most packages are private
(124 of 159 manifests set `"private": true`),
so the costs the guidance names have had no consumers to land on yet.


## Upstream filing decision

`.out-of-scope/` was checked for Sätteri,
 napi-rs,
 rolldown,
 and pnpm.
No exemption matched
(`bun-install.md`,
 `cargo-workspace.md`,
 `claude-code-upstream-bugs.md`,
`codex-harness.md`,
 `jsr.md`,
 `lightningcss.md`,
`low-impact-typescript-formatting.md`,
 `module-es-monolith.md`,
`pi-gpt55-long-context.md`,
 `terminal-title-fork-parity-tests.md`,
`typescript-project-references.md`).

Duplicate search:
 `gh search issues --repo napi-rs/napi-rs "Cannot find native binding"`
returns napi-rs#2934,
 closed,
 on the same message text.

1.  **Is it really upstream's fault?**
     No.
    `createRequire(import.meta.url)` is the only anchor a napi loader has,
    and it behaves correctly wherever it legitimately sits.
    Our build config moved the file.
    The one genuinely upstream-owned defect is the message wording,
    which asserts an npm-specific cause for a package-manager-independent failure.
2.  **Can upstream fix it?**
     Partly.
    The wording is a generated template and could name the resolution base and the
    specifiers tried instead of naming npm.
    The resolution behavior itself cannot change.
3.  **Are they supporting this use case?**
     Not applicable.
    Bundling a napi loader out of its package is not a use case anyone supports,
    and should not be.
4.  **Would the repo welcome our contribution?**
     Not assessed,
    because constraint 1 already fails.
5.  **Will they likely fix it?**
     napi-rs#2934 is closed on this message text
    without a wording change,
     which is mild evidence against.
6.  **Have we prototyped a minimal fix?**
     No,
     and deliberately not.
    The fix belongs in this repository's build configuration,
     not in Sätteri or napi-rs.

Decision:
 file nothing.
Constraint 1 fails,
 so the draft below is kept as an auditable record only.
**Do not file as-is.**

~~~md
Title: `Cannot find native binding` message names npm as the cause for any missing binding

The generated loader assembles one fixed message for every binding-resolution failure:

    Cannot find native binding. npm has a bug related to optional dependencies
    (https://github.com/npm/cli/issues/4828). ...

The text names npm and optional dependencies unconditionally.
A loader whose file has been relocated by a bundler,
or one resolving under a non-hoisting package manager,
produces the same message, which points at neither.

Suggested fix: include the resolution base (`import.meta.url`) and the specifiers
tried in the message, and demote the npm advice to one possible cause among several.
~~~

Nothing in that draft is absent from napi-rs#2934's thread,
so there is also no additive comment to post there.
