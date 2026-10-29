# Bun 1.3.10 `node:fs` `glob`/`globSync` silently skip dot files because the internal bridge omits `dot: true`

Status:
 **root cause identified**;
 upstream issue filed:
[oven-sh/bun#28021](https://github.com/oven-sh/bun/issues/28021).

## Symptom

`fs.glob` and `fs.globSync` from `node:fs/promises` / `node:fs`
never match dot files (files starting with `.`) when running on
Bun,
 regardless of pattern.
 Both wildcard patterns (`*`,
 `**/*`)
and explicit dot patterns (`.*`,
 `.hidden`) return empty results
for dot entries:

```ts
import { glob, } from 'node:fs/promises';

// Directory contains: visible, .hidden
for await (const match of glob('/path/*',))
  console.log(match,); // only "visible"; .hidden is silently skipped

for await (const match of glob('/path/.*',))
  console.log(match,); // no output at all
```

`Bun.Glob` with `dot: true` matches correctly under identical
conditions.

## Root cause

Bun's `fs.glob` delegates to `Bun.Glob.scan()` internally.
 The
bridge function `mapOptions` in `src/js/internal/fs/glob.ts`
builds scan options but never sets `dot: true`:

```ts
// src/js/internal/fs/glob.ts, lines 93-103 (Bun v1.3.10)
return {
  cwd: options?.cwd ?? process.cwd(),
  followSymlinks: true,
  onlyFiles: false,
  exclude,
};
// ^^^ no `dot` property; falls through to Bun.Glob default
```

The native `Bun.Glob` scan implementation in Zig defaults `dot` to
`false`:

```zig
// src/bun.js/api/glob.zig, line 70
var out: ScanOpts = .{
    .cwd = null,
    .dot = false,
    // ...
};
```

Since `mapOptions` never overrides this default,
 all dot files
are excluded from `fs.glob` results.

### Node.js comparison

Node.
js `fs.glob` has no `dot` option either,
 but its behaviour
differs from Bun's:

- **Explicit dot patterns** (`**/.gitignore`,
   `a/.b`):
   Node.
  js
  matches these;
   Bun does not.
- **Wildcard patterns** (`*`,
   `**/*`):
   neither Node.
  js nor Bun
  matches dot files.

Node.
js uses minimatch internally,
 where wildcards skip dot files
by default (standard Unix glob behaviour).
 Node.
js correctly
matches dot files when the dot is literal in the pattern.
 Bun
skips dot files unconditionally because `Bun.Glob` with `dot:
false` ignores them even in literal positions.

Node.
js has a related but distinct inconsistency:
 globstar (`**`)
prevents dot-file matching even with literal dot prefixes.
 See
[nodejs/node#56321](https://github.com/nodejs/node/issues/56321)
(filed Dec 2024,
 closed Dec 2025 without adding a `dot` option).

### Why this has not been widely reported

1. `fs.glob` only stabilised in Node.
   js v22.17.0 (June 2025):
   most projects still use third-party packages.
2. Third-party glob packages dominate:
    `glob` (312M/week),
   `fast-glob` (102M/week),
    `tinyglobby` (77M/week) as of
   March 2026.
3. Bun's `fs.glob` has only 4 commits since its introduction in
   January 2025;
    the community is still finding more fundamental
   compat gaps (`withFileTypes` missing,
    directory matching
   broken,
    array patterns broken).
4. The intersection of Bun users who use `fs.glob` and need
   dot-file matching is nearly zero.

## Verification

Version under test:

- Bun 1.3.10 (release tarball)
- Bun `main` branch source cloned March 2026 (for citing
  `src/js/internal/fs/glob.ts` and `src/bun.js/api/glob.zig`)

Reproduce:

```ts
import {
  mkdir,
  writeFile,
} from 'node:fs/promises';
import { glob, } from 'node:fs/promises';

await mkdir('/tmp/dotfile-test', { recursive: true, },);
await writeFile('/tmp/dotfile-test/visible', '',);
await writeFile('/tmp/dotfile-test/.hidden', '',);

const matches: string[] = [];
for await (const m of glob('/tmp/dotfile-test/*',))
  matches.push(m,);
console.log(matches,);
// Bun 1.3.10: ['/tmp/dotfile-test/visible']
// Node.js v22.17+: ['/tmp/dotfile-test/visible']
//   (Node also skips wildcard dot matches; that part agrees)

const dotMatches: string[] = [];
for await (const m of glob('/tmp/dotfile-test/.*',))
  dotMatches.push(m,);
console.log(dotMatches,);
// Bun 1.3.10: []                              ← bug
// Node.js v22.17+: ['/tmp/dotfile-test/.hidden']  ← correct
```

## Impact on this codebase

Two files used `Bun.Glob`:

- `package/file-enforcer/core/src/io/glob.ts`:
  `expandGlob()` requires `dot: true` to match hidden files.
  **Cannot migrate** to `fs.glob` due to this bug.
   Kept on
  `Bun.Glob`.
- `package/claude-code-plugin/session-start-housekeeping/src/index.ts`
  ;
   matches named directories only
  (`package/*/*/dist/final`);
   no dot-file concern.
   **Migrated**
  to `node:fs/promises` glob.

## Verified workarounds

### Use `Bun.Glob` with `dot: true`

For Bun-specific code paths that need dot-file matching,
 use
`Bun.Glob` directly rather than `node:fs/promises` glob:

```ts
const glob = new Bun.Glob('/path/*',);
for await (const m of glob.scan({ cwd: '/path', dot: true, },)) {
  // matches dot files
}
```

Tradeoff:
 code becomes Bun-specific and cannot run unchanged on
Node.
js.
 `AGENTS.md` prefers cross-runtime patterns;
 accept the
exception here because `fs.glob` is broken in Bun for this case.

### Restructure to avoid dot-file globs

When the use case is "match a specific named file",
 use a direct
`readdir`/`stat` rather than a glob:

```ts
import { readdir, } from 'node:fs/promises';
const entries = await readdir('/path', { withFileTypes: true, },);
for (const e of entries)
  if (e.name.startsWith('.',))/* ... */ ;
```

Tradeoff:
 more verbose;
 no pattern flexibility.
 Appropriate when
the dot-file set is small and known.

## What does not work

- Passing a `dot` option to `node:fs` glob:
   the option is not in
  the `node:fs` API surface;
   the underlying `mapOptions` bridge
  is where the missing flag lives.
- Setting `BUN_GLOB_DOT=true` or any env var:
   no such switch
  exists in Bun.
- Wrapping the call in a custom function that scans with a
  broader pattern:
   the scanner itself does not emit dot entries;
  no client-side filter can recover what the scanner never
  produced.

## Upstream fix (already filed)

Add `dot: true` to the `mapOptions` return in
`src/js/internal/fs/glob.ts`:

```ts
return {
  cwd: options?.cwd ?? process.cwd(),
  followSymlinks: true,
  onlyFiles: false,
  dot: true,
  exclude,
};
```

This aligns with Node.
js behaviour where explicit dot patterns
match dot files.
 Wildcard exclusion of dot files is handled by
minimatch/glob pattern semantics,
 not by the `dot` flag:
`Bun.Glob`'s `dot` flag controls whether the scanner emits dot
entries at all,
 which is a lower level than pattern matching.

## Source evidence

- `src/js/internal/fs/glob.ts`:
   `mapOptions` function,
   missing
  `dot: true`.
- `src/bun.js/api/glob.zig:70`:
   `ScanOpts` defaults `.dot =
  false`.
- `package/bun-types/bun.d.ts:7646-7650`:
   type definition
  confirms `dot` defaults to `false`.
- `src/js/node/fs.ts`:
   `node:fs` module imports and exposes
  `glob`/`globSync` from the internal module.

## Why we did file this upstream

All 5 constraints hold:

1. **Is it really upstream's fault?
   ** Yes;
    the missing `dot:
   true` in the bridge is a code defect.
2. **Can upstream fix it?
   ** Yes;
    a one-line change to the bridge
   options.
3. **Are they supporting this use case?
   ** Yes;
    `node:fs/promises`
   glob compatibility is an explicit Bun goal.
4. **Will they likely fix it?
   ** Bun has merged similar
   one-line compatibility patches quickly in recent releases.
5. **Have we prototyped a minimal fix?
   ** Yes;
    the patch is
   in-line above and trivially testable.

The issue is filed at oven-sh/bun#28021 and includes the source
locations and patch.

## Draft upstream issue (kept as reference)

````md
**Title**: `node:fs/promises` `glob` silently skips dot files because internal bridge omits `dot: true`

**Labels**: bug, node-compat, fs

**Description**:

`fs.glob` from `node:fs/promises` never matches dot files on Bun 1.3.10, even with explicit dot patterns like `.*` or `**/.gitignore`. Node.js matches dot files in those cases.

Source trace:

- `src/js/internal/fs/glob.ts:93-103`: `mapOptions` builds scan options without `dot: true`:

```ts
return {
  cwd: options?.cwd ?? process.cwd(),
  followSymlinks: true,
  onlyFiles: false,
  exclude,
};
```

- `src/bun.js/api/glob.zig:70`: `ScanOpts` defaults `.dot = false`:

```zig
var out: ScanOpts = .{ .cwd = null, .dot = false, ... };
```

Because `mapOptions` never overrides the default, all dot files are excluded.

**Reproduction**:

```ts
import { glob, } from 'node:fs/promises';
import {
  mkdir,
  writeFile,
} from 'node:fs/promises';

await mkdir('/tmp/dotfile-test', { recursive: true, },);
await writeFile('/tmp/dotfile-test/.hidden', '',);

const matches: string[] = [];
for await (const m of glob('/tmp/dotfile-test/.*',))
  matches.push(m,);
console.log(matches,); // [] on Bun; ['/tmp/dotfile-test/.hidden'] on Node v22.17+
```

**Suggested fix**: add `dot: true` to the `mapOptions` return in `src/js/internal/fs/glob.ts`:

```ts
return {
  cwd: options?.cwd ?? process.cwd(),
  followSymlinks: true,
  onlyFiles: false,
  dot: true,
  exclude,
};
```

This aligns with Node.js: explicit dot patterns match dot files; wildcard exclusion is still handled by the glob pattern semantics, not by the scanner-level `dot` flag.
````
