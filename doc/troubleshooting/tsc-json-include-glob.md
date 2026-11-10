# TypeScript 7.0.1-rc: a `src/**/*.json` include glob plus `resolveJsonModule` strict-parses every matched JSON file, so a raw-text file named `.json` fails the type check

An explicit `*.json` entry in a tsconfig `include` array makes `tsc` treat every matched
`.json` file as a program root.
With `resolveJsonModule` enabled,
 those roots are parsed against the strict JSON grammar,
even when no module ever imports them.
A file that carries a `.json` extension but holds non-JSON content (for example a raw
comma-separated list) then fails the type check with a burst of parser diagnostics,
despite being valid input for whatever actually reads it.

## Symptom

`mise run //package/<pkg>:lint:types` (and the type-aware `lint:oxlint`,
 which drives
`oxlint-tsgolint@0.24.0`) fail on a `.json` file that nothing imports,
 with a cascade of
parser errors anchored to `line 1`:

```txt
src/cache_AS41231.json(1,7): error TS1012: Unexpected token.
src/cache_AS41231.json(1,12): error TS1005: '{' expected.
src/cache_AS41231.json(1,13): error TS1327: String literal with double quotes expected.
src/cache_AS41231.json(1,15): error TS1328: Property value can only be string literal, numeric literal, 'true', 'false', 'null', object literal or array literal.
```

The trigger is a `.json` file whose bytes are not strict JSON,
 placed anywhere under a
directory that a tsconfig `include` glob covers with an explicit `*.json` pattern.
In this repo it surfaced when tofu external-data caches (raw comma-joined CIDR strings
written as `writeFile(CACHE_FILE, result)` in `package/config/tofu/src/fetch_ips.ts:418`)
were relocated under `src/`,
 where the shared include glob matched them for the first time.

## Root cause

Two conditions combine.

First,
 the shared tsconfig carried an explicit JSON glob.
`package/config/typescript/tsconfig.options.json` listed,
 before this fix:

```jsonc
"include": [
  "${configDir}/src/**/*.ts",
  "${configDir}/src/**/*.astro",
  "${configDir}/src/**/*.mdx",
  "${configDir}/src/**/*.json",
  "${configDir}/*.config.ts",
  "${configDir}/.astro/types.d.ts"
]
```

Files matched by `include` become program roots,
 so `tsc` loads and parses each matched
`.json` file whether or not any module imports it.

Second,
 `resolveJsonModule` is on:

```jsonc
// package/config/typescript/tsconfig.options.json:67
"resolveJsonModule": true,
```

That makes `tsc` parse `.json` roots with its strict JSON grammar (double-quoted keys and
strings,
 no bare tokens).
A `.json` file holding a raw comma-separated list is not strict JSON,
 so the parse fails at
the first bare token and emits TS1012/TS1005/TS1327/TS1328.

The glob was not deliberate.
`git blame` puts the `src/**/*.json` line in the omnibus commit `7721c72cc`
(2025-06-26),
 which mechanically expanded a single bare directory entry into explicit
per-extension globs:

```diff
-    "${configDir}/src",
+    "${configDir}/src/**/*.ts",
+    "${configDir}/src/**/*.astro",
+    "${configDir}/src/**/*.mdx",
+    "${configDir}/src/**/*.json",
```

A bare directory in `include` only matches TypeScript's default extensions
(`.ts`,
 `.tsx`,
 `.d.ts`,
 plus `.js`/`.jsx` under `allowJs`);
 `.json` is not among them,
even with `resolveJsonModule`.
So the old bare `src` never pulled JSON in.
The `.astro` and `.mdx` globs were genuine additions the SSG packages need;
 the `.json`
glob was added alongside them with no consumer.
A repo-wide search found zero `from '….json'` module imports and zero
`with { type: 'json' }` imports under any `src/`,
 and at the time the fix landed zero
`.json` files existed under any `src/`,
 so the glob had matched nothing since it was
introduced.

Earlier readings that were wrong,
 recorded so they are not re-derived:

- "The `lib` target is too old for `Error.isError`."
   The lib is already `ESNext` and
  declares `Error.isError`;
   that first failure was a separate script-outside-`src` issue.
- "tsgolint bundles an older lib without `Error.isError`."
   Its embedded
  `lib.esnext.d.ts` references `esnext.error`,
   which declares `isError`.
- "The JSON glob is load-bearing."
   No module imports any `.json`;
   the glob only
  force-parses matched files.

## Verification

Version under test:
 `typescript@7.0.1-rc` (native,
 `Version 7.0.1-rc`),
 the binary at
`node_modules/.pnpm/@typescript+typescript-linux-x64@7.0.1-rc/node_modules/@typescript/typescript-linux-x64/lib/tsc`.
The type-aware linter that first surfaced it is `oxlint-tsgolint@0.24.0`.

Minimal self-contained harness (run against the tsc binary above,
 referenced as `$TSC`):

```jsonc
// tsconfig.json
{
  "compilerOptions": {
    "module": "nodenext",
    "moduleResolution": "nodenext",
    "resolveJsonModule": true,
    "noEmit": true,
    "skipLibCheck": true
  },
  "include": ["src/**/*.ts", "src/**/*.json"]
}
```

```ts
// src/index.ts
export const x = 1;
```

Failing catalog:
 `src/raw.json` holding raw text (no import references it):

```txt
1.2.3.4/24,5.6.7.8/32,2001:db8::/48
```

```sh
"$TSC" --project tsconfig.json
# src/raw.json(1,4): error TS1012: Unexpected token.
# src/raw.json(1,8): error TS1005: '{' expected.
# src/raw.json(1,9): error TS1327: String literal with double quotes expected.
# src/raw.json(1,11): error TS1328: Property value can only be string literal, ...
```

Working catalog A,
 drop the `*.json` glob (`"include": ["src/**/*.ts"]`),
 same raw file:

```sh
"$TSC" --project tsconfig.json   # exit 0
```

Working catalog B,
 keep the glob but make the file strict JSON
(`{"ips":"1.2.3.4/24,5.6.7.8/32,2001:db8::/48"}`):

```sh
"$TSC" --project tsconfig.json   # exit 0
```

The same failure and both fixes reproduced in-repo:
 relocating the raw tofu caches under
`src/` broke `//package/config/tofu:lint:types`,
 and removing the shared `*.json` glob
returned it to exit 0 with no other package affected.

## Verified workarounds

- Remove the `src/**/*.json` glob from the shared include (the fix applied here).
  Tradeoff:
   a `.json` genuinely imported as a module is still type-checked,
   because the
  import pulls it in under `resolveJsonModule`;
   only never-imported JSON stops being
  parsed.
   No package depended on the glob,
   so the blast radius was zero.
   This is the right
  default:
   `include` should list source the program owns,
   not sweep arbitrary data files.
- Keep the glob but ensure every `.json` under `src/` is strict JSON.
  Tradeoff:
   brittle.
   Any future raw-text file named `.json` (a cache,
   a fixture,
   a fetched
  blob) reintroduces the failure,
   far from the include line that causes it.
- Do not place non-JSON payloads under `src/` with a `.json` extension.
  Tradeoff:
   sound hygiene,
   but it only removes the trigger locally and leaves the
  over-broad glob in place for the next author to trip.

## What does not work

- Reverting `include` to the bare `"${configDir}/src"` directory entry.
  It would stop matching `.json` (default extensions only),
   but it would also drop the
  `.astro` and `.mdx` globs the SSG packages rely on,
   breaking their type resolution.
- Excluding the specific files via `exclude`.
  It patches one package while leaving the footgun armed for every other `src/` tree,
   and
  couples the shared config to one package's data-file names.

## Upstream filing decision

Default policy is do not file,
 and this does not clear constraint 1,
 so the audit stops
there.

1.  Is it really upstream's fault?
     No. `include` defining program roots and
    `resolveJsonModule` parsing `.json` roots against the strict JSON grammar are both
    documented,
     intended TypeScript behaviors.
     A `.json` file containing non-JSON should
    fail to parse.
     The defect was our config's over-broad glob plus a raw-text payload
    misnamed `.json`,
     not a `tsc` bug.

`.out-of-scope/` was checked for a matching exemption;
 the TypeScript entries there
(`typescript-project-references.md`,
 `low-impact-typescript-formatting.md`) cover
unrelated topics,
 so none applies.
 No upstream issue or comment is warranted,
 and no
duplicate search is needed because there is nothing to file.

Decision:
 do not file.
