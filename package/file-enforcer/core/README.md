# file-enforcer

Declarative TypeScript tool for keeping derived files in sync across a monorepo.
Uses direct async function calls instead of a descriptor/engine pattern;
 each call reads and writes immediately.

## Motivation

Two problems prompted this package:

- Claude Code's `@AGENTS.md` include syntax is unreliable,
   so `CLAUDE.md` must be a literal copy of `AGENTS.md`
- Oxlint config files live in a config package but need to appear at the monorepo root

Generic file-sync GitHub Actions exist,
 but they target cross-repo sync and lack operations like concatenation,
 deduplication,
 and property extraction.

## Usage

Create a `file-enforcer.config.ts` at the monorepo root:

```ts
import {
  cat,
  overwrite,
} from '@monochromatic-dev/file-enforcer-core/ts';

await overwrite({
  dest: './CLAUDE.md',
  content: await cat(['./AGENTS.md',],),
});
```

Run it directly or through the CLI:

```bash
# Direct execution
node file-enforcer.config.ts

# CLI (finds the config in the nearest ancestor of cwd)
node package/file-enforcer/core/src/index.ts

# Watch mode -- re-runs on source changes, protects managed destinations
node package/file-enforcer/core/src/index.ts --watch
```

## API

### Reading

- `cat(files: string[])`:
   reads and concatenates files into a single string;
   paths containing `*` or `?` are auto-expanded as globs
- `cat(glob: string)`:
   reads files matching a glob pattern,
   returns `GlobResults` (a `GlobResult[]` carrying the source pattern)

### Writing

- `overwrite({ dest, content })`:
   writes content to dest;
   skips when existing content is identical.
- `overwriteIfNotExists({ dest, content })`:
   writes only if the file does not exist
- `overwriteEach({ destGlob, files })`:
   mirrors each `GlobResults` entry to a destination using positional
  wildcard substitution;
   source glob is read from the array.
- `overwriteTomlKey({ dest, path, value })`:
   updates a single key in an existing TOML file,
   preserving comments
  and unmutated whitespace via splice mode;
   throws when dest does not exist
- `manageMiseManifests({ manifestGlobs })`:
   normalizes `mise.toml` task-header key quoting across every manifest matched
  by the caller's globs,
   rewriting literal-quoted header keys as basic strings while
  preserving every other byte;
   canonical manifests produce no write

### Staleness cache

One-shot runs persist a manifest at `node_modules/.cache/file-enforcer/staleness-manifest.json`.
The default manifest path walks up from the current directory until it finds `node_modules`,
 then writes under that root.
The manifest records the config file,
 files read through `cat()` or `addWatchedPaths()`,
 glob expansions,
and managed destination metadata plus content hashes.
Config code still executes on every run so new or untracked effects can be discovered.
Lazy `overwrite()` and `overwriteEach()` builders can skip their content callbacks when captured sources,
glob expansions,
 and destination hashes still match.

Pass `manifestPath` only for throwaway fixtures or benchmarks that should not share the default cache.

### Transforms

- `dedup(content)`:
   removes duplicate lines,
   preserving first occurrence order
- `getJsonProperty({ path, content })`:
   extracts a nested value from JSON;
   `path` is an array of segments (string keys and numeric array indices),
   avoiding ambiguity for keys that contain literal dots
- `getTomlProperty({ path, content })`:
   same shape as `getJsonProperty` but for TOML;
   backed by `@monochromatic-dev/toml-edit-core`,
   supports section headers,
   inline tables,
   and array-of-tables indexing
- `editTomlKey({ content, path, value })`:
   returns TOML text with a single key set or replaced;
   splice mode keeps unmutated regions byte-identical.
   Each call parses and stringifies,
   so for multiple edits to the same source,
   compose `parseTomlEdit + tomlSet + tomlStringify` from `@monochromatic-dev/toml-edit-core` directly
- `exec(cmd, args)`:
   runs a command and captures stdout
- `exec(platformCommands)`:
   platform-aware exec;
   evaluates `[predicate, command]` tuples top-to-bottom and runs the first match
   (see [Platform-aware exec](#platform-aware-exec) below)
- `evaluatePredicate(predicate)`:
   runs a predicate and returns whether it succeeded (exit 0);
   results cached per-session
- `inspect(value)`:
   debug tap that logs and returns the value unchanged

### Watch mode utilities

- `addWatchedPaths(paths)`:
   registers additional paths for watch mode to monitor (for `exec()` dependencies)
- `invalidatePaths(paths)`:
   surgically removes specific entries from the in-memory read cache
- `reset()`:
   clears read/write tracking sets between re-runs (preserves cache and write timestamps)

## Platform-aware exec

`exec()` takes one object.
The platform-aware form is `{ platformCommands }`,
 whose value is an array of `[predicate, command]` tuples for platform-dependent command dispatch.
Predicates are direct commands (no shell involved);
 exit code 0 means the predicate matched.
Tuples are evaluated top-to-bottom;
 the first match wins.

```ts
import { exec, } from '@monochromatic-dev/file-enforcer-core/ts';

// Install git via the first available package manager
const output = await exec({
  platformCommands: [
    [['mise', '--version',], ['mise', 'use', 'git',],],
    [['brew', '--version',], ['brew', 'install', 'git',],],
    [['dnf', '--version',], ['dnf', 'install', 'git',],],
  ],
},);
```

### Checking tool capabilities

Predicates can verify not just tool presence but specific capabilities.
Use `evaluatePredicate()` to combine multiple checks:

```ts
// Can mise manage git at all?
const hasMise = await evaluatePredicate(['mise', '--version',],);
const miseCanManageGit = hasMise
  && await evaluatePredicate(['mise', 'registry', 'git',],);

if (miseCanManageGit)
  await exec({ cmd: 'mise', args: ['use', 'git',], },);
```

### Reusable predicates

Define predicates as constants to share across multiple `exec()` calls:

```ts
const HAS_MISE = ['mise', '--version',] as const;

await exec({ platformCommands: [[HAS_MISE, ['mise', 'exec', '--', 'git', 'pull',],],], },);
await exec({ platformCommands: [[HAS_MISE, ['mise', 'exec', '--', 'git', 'status',],],], },);
```

### Manual dispatch with `evaluatePredicate()`

For complex logic that doesn't fit the tuple pattern,
 use `evaluatePredicate()` directly:

```ts
import {
  evaluatePredicate,
  exec,
} from '@monochromatic-dev/file-enforcer-core/ts';

const hasMise = await evaluatePredicate(['mise', '--version',],);
const hasBrew = await evaluatePredicate(['brew', '--version',],);

if (hasMise)
  await exec({ cmd: 'mise', args: ['use', 'git',], },);
if (hasBrew)
  await exec({ cmd: 'brew', args: ['install', 'git',], },);
```

### Nested platform dispatch

The command slot accepts another `PlatformCommands` for multi-level dispatch.
The first element of the inner array being an array (not a string) triggers recursive evaluation.

```ts
await exec({
  platformCommands: [
    [['mise', '--version',], [
      [['mise', 'where', 'python@3.12',], ['mise', 'exec', 'python@3.12', '--',
        'script.py',],],
      [['mise', 'where', 'python@3.11',], ['mise', 'exec', 'python@3.11', '--',
        'script.py',],],
    ],],
    [['python3', '--version',], ['python3', 'script.py',],],
  ],
},);
```

Nested command literals require `as const` to satisfy the recursive type.
Extract them into a typed constant to keep the call site readable:

```ts
const miseDispatch = [
  [['mise', 'where', 'python@3.12',], ['mise', 'exec', 'python@3.12', '--',
    'script.py',],],
  [['mise', 'where', 'python@3.11',], ['mise', 'exec', 'python@3.11', '--',
    'script.py',],],
] as const;

await exec({
  platformCommands: [
    [['mise', '--version',], miseDispatch,],
    [['python3', '--version',], ['python3', 'script.py',],],
  ],
},);
```

### Negation via noop fallthrough

Predicates only express positive checks (exit 0 = match).
To express "if X is NOT available,
 do Y",
 match the positive case with a noop command and let the negative case fall through:

```ts
await exec({
  platformCommands: [
    [['python3', '--version',], ['true',],], // python found → noop
    [['true',], installPython,], // fallthrough → install
  ],
},);
```

For complex negation logic,
 use `evaluatePredicate()` with control flow instead:

```ts
const hasPython = await evaluatePredicate(['python3', '--version',],);
if (!hasPython)
  await exec({ cmd: 'apt-get', args: ['install', '--yes', 'python3',], },);
```

## Mutation testing

Mutation testing is opt-in because it builds a local Podman runtime image and runs
one Stryker session per production source file.

```bash
mise run //package/file-enforcer/core:test:mutation -- src/io/glob-mirror.ts
mise run //package/file-enforcer/core:test:mutation -- --full-suite
```

The task dynamically enumerates production `src/**/*.ts` files,
 excludes tests,
declaration files,
 and fixtures,
 and prints the resolved count.
 Reports are written
to a host temp directory printed by the run.
 The aggregate score is weighted from
raw Stryker mutant counts.

## Package management

`ensurePackage()` ensures a system-level binary exists on the current machine,
installing it via the platform's native package manager if absent.
Designed for packages that mise cannot manage (system libraries,
 servers,
 desktop apps,
 core OS utilities).

### Basic usage

The bundled package index lives at `src/data/packages.ts`,
which the manifest does not export,
so a consumer registers its own entries.
Whether to export that data is issue #554.

```ts
import {
  ensurePackage,
  p,
  registerPackages,
} from '@monochromatic-dev/file-enforcer-core/ts';

registerPackages([p('curl',), p({ bin: 'rg', effname: 'ripgrep', },),],);

await ensurePackage('curl',); // already installed → noop
await ensurePackage('rg',); // not found → installs ripgrep via detected manager
```

### How it works

1. Check if the binary exists on PATH (runs `<binary> --version` or a custom check flag)
2. Look up the binary in the registered package index
3. Detect the system's package manager (apt,
    dnf,
    pacman,
    apk,
    zypper,
    brew,
    winget,
    scoop,
    choco)
4. Verify the package exists in the manager's repository (`apt-cache show`,
    `dnf info`,
    etc.)
5. Install the package (with auto-detected privilege escalation)
6. Verify the binary now exists on PATH

### Package index

The index ships as TypeScript source using the `p()` builder:

```ts
p('curl',); // binary = effname = package name everywhere
p({ bin: 'rg', effname: 'ripgrep', },); // binary differs from package name
p({ effname: 'wget', winget: 'JernejSimoncic.Wget', },); // per-manager override
p({ bin: 'openssl', check: 'version', effname: 'openssl', },); // custom existence check
```

The index is split into two files:

- `data/packages.generated.ts`:
   auto-generated from Repology database dump (7,332 entries).
  Freely rebuilt;
   contains effname and per-manager package name mappings.
- `data/packages.overrides.ts`:
   hand-maintained.
  Contains binary names,
   custom check flags,
   and corrections that Repology cannot infer.
  Survives index regeneration.
- `data/packages.ts`:
   merges both files into the final index.

### Supported package managers

<table>
<thead>
<tr>
<th>Manager</th>
<th>Detection</th>
<th>Search</th>
<th>Install</th>
<th>Privilege</th>
</tr>
</thead>
<tbody>
<tr>
<td>apt</td>
<td>`apt-get --version`</td>
<td>`apt-cache show`</td>
<td>`apt-get install --yes`</td>
<td>sudo</td>
</tr>
<tr>
<td>dnf</td>
<td>`dnf --version`</td>
<td>`dnf info`</td>
<td>`dnf install --assumeyes`</td>
<td>sudo</td>
</tr>
<tr>
<td>pacman</td>
<td>`pacman --version`</td>
<td>`pacman -Si`</td>
<td>`pacman -S --noconfirm`</td>
<td>sudo</td>
</tr>
<tr>
<td>apk</td>
<td>`apk --version`</td>
<td>`apk info --description`</td>
<td>`apk add`</td>
<td>sudo</td>
</tr>
<tr>
<td>zypper</td>
<td>`zypper --version`</td>
<td>`zypper info`</td>
<td>`zypper install --non-interactive`</td>
<td>sudo</td>
</tr>
<tr>
<td>brew</td>
<td>`brew --version`</td>
<td>`brew info`</td>
<td>`brew install`</td>
<td>user</td>
</tr>
<tr>
<td>winget</td>
<td>`winget --version`</td>
<td>`winget show --exact`</td>
<td>`winget install --id --exact`</td>
<td>user</td>
</tr>
<tr>
<td>scoop</td>
<td>`scoop --version`</td>
<td>`scoop info`</td>
<td>`scoop install`</td>
<td>user</td>
</tr>
<tr>
<td>choco</td>
<td>`choco --version`</td>
<td>`choco info`</td>
<td>`choco install --yes`</td>
<td>admin</td>
</tr>
</tbody>
</table>

Privilege escalation is auto-detected:
 `sudo` is prepended for managers that need root,
skipped when already running as root (UID 0 / container context).

## Architecture

### Direct execution

The config file is a plain TypeScript script with top-level `await`.
Each function call reads from disk (or cache) and writes immediately.
There is no descriptor collection phase and no engine interpreter.
Users control sequencing and parallelism with `await` and `Promise.all`.

### In-memory read cache

After the first run,
 file contents are cached in memory.
On watch-mode re-runs,
 only the file that triggered the event is invalidated and re-read.
All other files return cached content,
 turning ~300 file reads into ~1.

### Content-based write skipping

`overwrite()` reads the existing destination content before writing.
If the content is identical,
 the write is skipped entirely.
This makes full re-runs cheap even without knowing which source changed.

### Watch mode

The CLI's `--watch` flag uses `fs.watch` on directories derived from tracked reads and writes.
Events are classified into three categories:

- **source**:
   a tracked source file or the config changed;
   triggers re-run
- **protected**:
   a managed destination was modified externally;
   triggers re-run + system notification via `notify-send`
- **ignore**:
   unrelated file or our own write echoing through `fs.watch`

Echo detection compares the file's `mtime` against the recorded write timestamp.

### Mirror-glob expansion

`overwriteEach` maps wildcards positionally between source and destination patterns.
`package/*/src/index.ts` -> `dist/*/index.ts` substitutes each captured segment into the corresponding position.

## Source files

All production source files are under 100 lines per the monorepo coding guidelines.

- `cache.ts`:
   in-memory read cache with invalidation and post-write updates
- `cat.ts`:
   overloaded file reading (array concatenation vs glob expansion)
- `ensure-package.ts`:
   `ensurePackage()` function,
   index lookup,
   binary check,
   install dispatch
- `evaluate-predicate.ts`:
   shell predicate evaluation with per-session caching
- `exec.ts`:
   child process execution with stdout capture and platform-aware tuple dispatch
- `glob.ts`:
   glob expansion and mirror-glob path mapping
- `inspect.ts`:
   generic debug tap
- `manager.ts`:
   package manager detection,
   `binaryExists`,
   `canProvide`,
   `installPackage`
- `merge.ts`:
   merge generated index with hand-maintained overrides
- `mise/manage-mise-manifests.ts`:
   `manageMiseManifests()` normalizes mise task-header key quoting across
  caller-supplied manifest globs,
   reusing the byte-preserving toml-edit normalizer
- `mod.ts`:
   re-exports for the public API
- `index.ts`:
   CLI entry point with upward config discovery and --watch flag
- `notify.ts`:
   terminal warning + platform-aware desktop notification dispatch
- `p.ts`:
   `p()` builder for `PackageEntry` values
- `tracker.ts`:
   read/write/timestamp tracking for watch mode
- `toml.ts`:
   getTomlProperty and editTomlKey backed by @monochromatic-dev/toml-edit-core
- `transform.ts`:
   dedup and dot-prop getJsonProperty (array-path form)
- `types.ts`:
   `PackageManager`,
   `PackageSpec`,
   `PackageEntry`,
   and the local `Path` alias
- `watch.ts`:
   main watch loop with debounce and cache-busting re-import
- `watch-dir.ts`:
   per-directory fs.
  watch wrapper with AbortController
- `watch-filter.ts`:
   event classification (source/protected/ignore)
- `write.ts`:
   overwrite,
   overwriteIfNotExists,
   overwriteEach,
   readExisting with content-skip
- `write-toml.ts`:
   overwriteTomlKey -- read existing TOML,
   apply one key edit,
   write back through writeIfChanged

## Tests

145 unit and integration tests covering all modules.
 Run via the mise task (preferred) or by invoking individual `*.unit.test.ts` files directly with `bun`:

```bash
mise run //package/file-enforcer/core:test:unit
```
