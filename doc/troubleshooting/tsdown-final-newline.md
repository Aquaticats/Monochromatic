# tsdown 0.22.4 minified Node and declaration builds omit final LF

## Symptom

A tsdown Node build can emit non-empty JavaScript and declaration files with no final LF.
Git then renders this diagnostic:

```text
\ No newline at end of file
```

The result depends on the output path through the toolchain:

- Minified JavaScript with `minify.codegen: true` ends with zero LF bytes.
- Declarations emitted by `dts: true` end with zero LF bytes whether JavaScript is minified or not.
- Unminified JavaScript ends with one LF in the verified fixture,
   but that does not repair declarations.
- Adding `footer: '\n'` to a minified build makes both files end with two LF bytes,
   not one.

This repository commits generated Node output.
The behavior was initially treated as rebuild drift,
then accepted as an intentional compact-output policy that saves one byte per generated file.

## Root cause

The source trace uses these exact releases:

- `rolldown/tsdown` tag `v0.22.4`,
   commit `434cfb3f563addffb88882b128b7a390f3434e94`.
- `rolldown/rolldown` tag `v1.1.5`,
   commit `f09947ab017d6df74299f691853dcfc4f4f0f86e`.
- `sxzz/rolldown-plugin-dts` tag `v0.27.1`,
   commit
  `f2d002e954a9b8cc14610ca6bc80548323043d22`.

First,
 tsdown delegates a non-watch build to Rolldown and receives already-written output.
`rolldown/tsdown` `src/build.ts:167-175` contains:

```ts
const configs = await initBuildOptions()
if (watch) {
  watcher = rolldownWatch(configs)
  handleWatcher(watcher)
} else {
  const outputs = await rolldownBuild(configs)
  for (const { output } of outputs) {
    chunks.push(...addOutDirToChunks(output, outDir))
  }
}
```

Second,
 Rolldown's convenience `build` API writes by default.
`rolldown/rolldown` `package/rolldown/src/api/build.ts:58-68` contains:

```ts
if (Array.isArray(options)) {
  return Promise.all(options.map((opts) => build(opts)));
} else {
  const { output, write = true, ...inputOptions } = options;
  const build = await rolldown(inputOptions);
  try {
    if (write) {
      return await build.write(output);
    } else {
      return await build.generate(output);
    }
```

Third,
 Rolldown joins generated sources with LF only between sources.
It does not append an LF after the final source.
`rolldown/rolldown` `crates/rolldown_sourcemap/src/source_joiner.rs:41-62` contains:

```rust
if !self.enable_sourcemap {
  for (index, source) in sources_iter {
    ret_source.push_str(source.content());
    if index < sources_len - 1 {
      ret_source.push('\n');
    }
  }
  return (ret_source, None);
}
```

The source-map branch has the same boundary rule in
`crates/rolldown_sourcemap/src/source_joiner.rs:72-81`:

```rust
ret_source.push_str(source.content());
if index < sources_len - 1 {
  ret_source.push('\n');
  line_offset += source.lines_count() + 1;
}
```

This explains both JavaScript outcomes.
An unminified generator result that already carries an LF keeps it.
A minified result without one gets no LF from `SourceJoiner`.
It also explains the rejected footer workaround:
`footer: '\n'` becomes another source,
 so Rolldown inserts one separator LF before the footer's own LF.

Declarations have a separate generator but the same pass-through behavior.
`rolldown-plugin-dts` `src/fake-js.ts:1-5` imports its parser and printer:

```ts
import { b, is } from 'yuku-ast'
import { isIdentifierName } from 'yuku-ast/identifier'
import { nameOf } from 'yuku-ast/utils'
import { print } from 'yuku-codegen'
import { parse, walk, type ParseResult } from 'yuku-parser'
```

Its declaration `renderChunk` returns printer output unchanged at
`src/fake-js.ts:546-557`:

```ts
const result = print(program, {
  comments: true,
  ...(sourcemap && {
    sourceMaps: {
      source: code,
      sourceFileName: chunk.fileName,
    },
  }),
})

return {
  code: result.code,
  map: (result.map ?? null) as SourceMapInput | null,
}
```

The verified printer output had no final LF.
Neither the plugin return nor Rolldown's join step adds one.

Finally,
 Rolldown writes chunk bytes without a final-newline transformation.
`rolldown/rolldown` `crates/rolldown/src/bundle/bundle.rs:216-223` contains:

```rust
self
  .fs
  .write(&dest, chunk.content_as_bytes())
  .with_context(|| format!("Failed to write file in {}", dest.display()))?;
```

An earlier idea was that tsdown itself removed a final LF after Rolldown wrote the files.
The call chain disproves that reading:
 tsdown delegates writing to `rolldownBuild`,
 and Rolldown writes each chunk's
existing bytes unchanged.

## Verification

### Versions and source identity

The behavior was reproduced on 2026-07-09 with:

```text
tsdown v0.22.4
rolldown v1.1.5
rolldown-plugin-dts v0.27.1
Node.js v26.4.0
```

`gh release list --repo rolldown/tsdown` reported `v0.22.4` as the latest release.
The source tags and commits are recorded in the root-cause section.

### Runnable harness

Create `/tmp/tsdown-final-newline/src/index.ts`:

```ts
export const value: number = 1;
```

Create `/tmp/tsdown-final-newline/tsdown.config.mjs`:

```js
import { glob, readFile, writeFile } from 'node:fs/promises'
import { join } from 'node:path'

const packageDir = process.env.REPRO_PACKAGE_DIR
const outputRoot = process.env.REPRO_OUTPUT_DIR
const entry = process.env.REPRO_ENTRY

if (!packageDir || !outputRoot || !entry)
  throw new Error('Set all REPRO_* variables.')

const base = {
  cwd: packageDir,
  entry: [entry],
  dts: true,
  format: 'esm',
  clean: true,
}
const minify = {
  compress: true,
  mangle: false,
  codegen: true,
}

function outputDir(name) {
  return join(outputRoot, name)
}

function normalizeFinalLf(content) {
  if (content.length === 0)
    return content

  let end = content.length
  while (end > 0 && content.charAt(end - 1) === '\n')
    end -= 1
  return `${content.slice(0, end)}\n`
}

const normalizeHook = {
  'build:done': async ({ options }) => {
    const pattern = '**/*.{js,mjs,cjs,ts,mts,cts}'
    for await (const relativePath of glob(pattern, { cwd: options.outDir })) {
      const path = join(options.outDir, relativePath)
      const content = await readFile(path, 'utf8')
      const normalized = normalizeFinalLf(content)
      if (normalized !== content)
        await writeFile(path, normalized, 'utf8')
    }
  },
}

export default [
  { ...base, outDir: outputDir('minified'), minify },
  { ...base, outDir: outputDir('unminified') },
  {
    ...base,
    outDir: outputDir('codegen-disabled'),
    minify: { ...minify, codegen: false },
  },
  { ...base, outDir: outputDir('footer'), minify, footer: '\n' },
  { ...base, outDir: outputDir('hook'), minify, hooks: normalizeHook },
]
```

Run from this repository root.
The existing package supplies a working TypeScript project for declaration generation;
 the entry itself remains the
single-line scratch fixture:

```sh
mkdir --parents /tmp/tsdown-final-newline/src
printf '%s\n' 'export const value: number = 1;' > /tmp/tsdown-final-newline/src/index.ts
REPRO_PACKAGE_DIR="$PWD/packages/claude-code-plugin/prompt-time" \
REPRO_OUTPUT_DIR=/tmp/tsdown-final-newline/dist \
REPRO_ENTRY=/tmp/tsdown-final-newline/src/index.ts \
mise exec -- tsdown --config /tmp/tsdown-final-newline/tsdown.config.mjs
node -e "const {readFileSync,readdirSync}=require('node:fs');const root='/tmp/tsdown-final-newline/dist';for(const variant of readdirSync(root).sort())for(const file of readdirSync(root+'/'+variant).sort()){const bytes=readFileSync(root+'/'+variant+'/'+file);let count=0;for(let i=bytes.length-1;i>=0&&bytes[i]===10;i-=1)count+=1;console.log(variant+'/'+file+': '+count)}"
```

The final number on each line is its final-LF byte count.

### Patterns that work cleanly

```text
hook/index.d.mts: 1
hook/index.mjs: 1
unminified/index.mjs: 1
codegen-disabled/index.mjs: 1
```

### Patterns that fail with no final LF

```text
minified/index.d.mts: 0
minified/index.mjs: 0
unminified/index.d.mts: 0
codegen-disabled/index.d.mts: 0
```

### Patterns that fail with multiple final LF bytes

```text
footer/index.d.mts: 2
footer/index.mjs: 2
```

## Repository decision

The shared tsdown producer remains byte-transparent.
All 18 tracked files under `package/claude-code-plugin/*/dist/final/node/` have zero final LF bytes,
which saves 18 bytes compared with mandatory one-LF output.

Cli-git's core `final-newline` policy excludes `**/dist/final/node/**` from commit,
manual-push,
direct-check,
and direct-fix behavior.
Packed lifecycle fixtures prove the exclusion applies at each policy boundary.
Issue `#357` removed the superseded hk/Pkl layer.
The producer normalizer and its tests were removed.

The exception is directory-scoped rather than suffix-scoped because `dist/final/node` is the shared tsdown producer
boundary.
Moving a generated file outside that directory makes normal newline enforcement apply again.

## Verified workarounds

### Normalize owned text outputs in `build:done`

Projects that prefer canonical final LF can use the `normalizeHook` in the runnable harness.
It turns both minified JavaScript and declarations into exactly one final LF,
leaves empty files empty,
and avoids rewriting content that is already canonical.

A repository-local prototype also verified recursive suffix filtering,
idempotent writes,
and multi-entry completion coordination before it was removed in favor of compact output.

This hook is supported by tsdown's documented lifecycle.
`rolldown/tsdown` `src/build.ts:306-317` invokes it after copy,
 executable processing,
 and completion work:

```ts
async function postBuild() {
  await copy(config)
  await buildExe(config, chunks)

  if (!hasBuilt) {
    await done(bundle)
  }

  await hooks.callHook('build:done', { ...context, chunks })
  hasBuilt = true
```

Tradeoffs:

- Every build performs a recursive directory scan and reads owned JavaScript and declaration outputs.
- Canonicalization adds one byte to each currently tracked non-empty artifact.
- A suffix allowlist can exclude source maps and copied assets,
   but then those bytes remain producer-owned.
- A per-entry tsdown config array invokes `build:done` for each config.
  Shared output directories therefore need a completion gate to avoid duplicate scans and concurrent writes.
- Filesystem failures should fail the build rather than leave partially normalized output.
- This is a consumer-side canonicalization policy,
   not a claim that upstream output is invalid.

The prototype normalized a repeated build of the two-entry `bash-output-filter` package once and produced no
newline-only drift.
The current repository intentionally chooses the opposite byte policy and does not install this workaround.

## What does not work

### Add `footer: '\n'`

The fixture produced two final LF bytes in both minified JavaScript and declarations.
Rolldown inserts one LF between the generated chunk and footer,
 then preserves the footer's own LF.

### Disable minifier code generation

`minify.codegen: false` produced one final LF in JavaScript but still produced zero in declarations.
It also changes JavaScript formatting,
 so it is not a final-newline-only correction.

### Disable minification

Unminified JavaScript happened to carry one final LF,
 while its declaration still carried zero.
Disabling minification changes artifact size and formatting without solving every output path.

### Edit committed output only

The next tsdown build regenerates the old byte endings.
If canonical final LF is desired,
the producer must own normalization.
For this repository,
the correct response is instead to keep producer output and exclude its directory from newline enforcement.

### Run the same postprocessor independently for every entry

`perEntryNodeConfig` builds entries as separate configs sharing one output directory.
An initial hook ran once per entry,
 scanned the same files twice,
 and logged the same four rewrites twice.
A shared completion gate now runs normalization after the final entry completion and resets for watch-mode rebuilds.

## Upstream filing decision

No matching exemption exists in the 11 files under `.out-of-scope/`.
The only tsdown mention is the unrelated TypeScript project-reference discussion in
`.out-of-scope/typescript-project-references.md`.

Searches covered open and closed issues and pull requests in `rolldown/tsdown` for `final newline`,
`trailing newline`,
 `EOF newline`,
 and `footer newline`.
No matching report exists.
Pull request [#932](https://github.com/rolldown/tsdown/pull/932) matched a broad trailing-newline search but concerns
shim banner placement,
 not final output bytes.

The six filing constraints resolve as follows:

1.  **Is it really upstream's fault?
    ** No.
    tsdown and Rolldown document JavaScript and declaration generation,
     but no reviewed source promises exactly one
    final LF.
    The observed bytes are consistent with Rolldown preserving generator output.
2.  **Can upstream fix it?
    ** Yes.
    A canonicalization option could live in tsdown post-processing,
     or Rolldown could add an explicit output policy.
    A global Rolldown change would need to account for consumers that rely on exact output bytes.
3.  **Are they supporting this use case?
    ** Partly.
    The README explicitly supports library bundles and declaration generation,
     and the hook documentation supports
    post-processing.
    Exact final-LF canonicalization is not documented.
4.  **Would the repo welcome our contribution?
    ** Yes,
     with conditions.
    `.github/ISSUE_TEMPLATE/bug_report.yml` accepts bug reports only with a minimal reproduction.
    `.github/PULL_REQUEST_TEMPLATE.md` asks contributors to avoid duplicate work and include regression tests.
    It permits AI-generated core code only after careful human line-by-line review and requires that fact to be
    checked in the template.
    No policy banning AI-assisted issue reports was found.
    The linked `sxzz/contribute` guide asks contributors to discuss features first and use Conventional Commits.
5.  **Will they likely fix it?
    ** Unknown,
     with no active rejection.
    `v0.22.4` is the latest release,
     tracker searches found no decision,
     and recent `src/build.ts` history concerns
    config dependency tracking rather than byte endings.
    No documented non-goal or maintainer rejection was found.
6.  **Have we prototyped a minimal fix compatible with their architecture?
    ** No upstream patch.
    Constraints 1 and 3 do not hold,
     so the automatic upstream prototype gate does not trigger.
    The verified `build:done` consumer workaround proves the documented extension point is available,
    but this repository intentionally keeps the smaller producer-native output instead.

The default no-filing policy applies.
Nothing was posted upstream.
The following draft is retained for audit only and must not be filed as-is.

~~~md
Title: Add an explicit final-newline policy for generated JavaScript and declarations

Labels: pending triage

## Description

With tsdown 0.22.4, Rolldown 1.1.5, and rolldown-plugin-dts 0.27.1:

- minified JavaScript with `minify.codegen: true` ends with zero LF bytes;
- declarations from `dts: true` end with zero LF bytes;
- adding `footer: '\n'` makes both outputs end with two LF bytes.

This may be intentional byte preservation rather than a bug because the current docs do not promise a final LF.
I am opening this only as a design question: should tsdown offer an explicit canonical final-newline option?

## Reproduction

Use the matrix fixture and command from `doc/troubleshooting/tsdown-final-newline.md`.
The observed final-LF counts are:

```text
minified/index.d.mts: 0
minified/index.mjs: 0
footer/index.d.mts: 2
footer/index.mjs: 2
```

## Source trace

`src/build.ts` delegates writing to `rolldownBuild`.
Rolldown's `SourceJoiner::join` inserts LF only between sources, and `bundle_write` writes
`chunk.content_as_bytes()` unchanged.
rolldown-plugin-dts returns `yuku-codegen`'s `result.code` unchanged from `renderChunk`.

## Suggested fix

First decide whether exact final-LF canonicalization belongs in tsdown or Rolldown.
If it belongs in tsdown, add an opt-in output policy after Rolldown writing and before `build:done`, covering both
JavaScript and declaration suffixes, empty-file behavior, source maps, watch rebuilds, and multi-config output
sharing.
Add regression cases for minified JavaScript, declarations, footer interaction, empty output, and idempotence.

This draft was prepared with AI assistance from a reproduction, source trace, and verified consumer workaround.
No upstream patch has received the human line-by-line review required by the pull request template.
~~~
