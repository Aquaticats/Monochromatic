# `Node.js` v26.5.0 `fs.promises.glob()` needs consumer sorting for stable mise PATH ordering

`Node.js` returns glob matches in filesystem directory-entry order.
The root `mise.toml` generator sorts and deduplicates workspace
`node_modules/.bin` directories before serializing them into `_.path`.
That makes configuration entries unique and ordering reproducible for the same
set of discovered directories.
Membership remains installation-dependent because a path is included only when
its `node_modules/.bin` directory exists.

## Symptom

Before commit `13dcfb114`,
 `file-enforcer.config.ts` copied raw
`fs.promises.glob()` output into the generated `mise.toml` `_.path` list.
A regeneration on another filesystem could move existing entries and change
which duplicate executable appeared first in `PATH`.

The fix landed in `file-enforcer.config.ts` and was regenerated into
`mise.toml` by commit `92a9499a4`.
The current generated ordering is stable for a fixed installed dependency tree.

## Root cause

### `Node.js` glob forwards unsorted directory entries

`nodejs/node@v26.5.0:lib/internal/fs/glob.js:210-217` obtains directory entries
with `readdir()` and caches the received array without sorting it:

```js
// nodejs/node@v26.5.0:lib/internal/fs/glob.js
async readdir(path) {
  const cached = this.#readdirCache.get(path);
  if (cached) {
    return cached;
  }
  const promise = PromisePrototypeThen(readdir(path, { __proto__: null, withFileTypes: true }), null, () => []);
  this.#readdirCache.set(path, promise);
  return promise;
}
```

`nodejs/node@v26.5.0:lib/internal/fs/glob.js:779-783` iterates that array by
index without an intervening sort:

```js
// nodejs/node@v26.5.0:lib/internal/fs/glob.js
children = await this.#cache.readdir(fullpath);

for (let i = 0; i < children.length; i++) {
  const entry = children[i];
```

`nodejs/node@v26.5.0:doc/api/fs.md:7225-7228` documents that directory entries
arrive in no particular order from underlying directory mechanisms:

```md
Directory entries returned by this function are in no particular order as
provided by the operating system's underlying directory mechanisms.
Entries added or removed while iterating over the directory might not be
included in the iteration results.
```

### The generator normalizes ordering and removes duplicate paths

`file-enforcer.config.ts:614-624` keeps the root `node_modules/.bin` directory
first,
 sorts discovered package bin directories,
 then constructs a `Set` before
creating TOML:

```ts
// file-enforcer.config.ts
[
  ...new Set([
    'node_modules/.bin',
    ...(await Array.fromAsync(glob('package/*/*/node_modules/.bin',),)).toSorted(),
  ]),
]
  .map(function quote(dir,): string {
    return `  "${dir}"`;
  },)
  .join(',\n',)
```

Mise preserves the `_.path` array's sequence when it constructs `PATH`.
Sorting controls observable package-local binary precedence;
the `Set` ensures no path is emitted twice.

## Verification

Versions under test:

- `Node.js` `v26.5.0`
- mise `2026.7.0`,
   Linux x64
- `nodejs/node` tag `v26.5.0`,
   commit `bebd1b8d92bf4cc917844d6335ed1ecf9c2a75fb`

The following checks passed on 2026-07-11:

- A disposable fixture created `zebra`,
   `amber`,
   `quartz`,
   then `birch` package
  directories.
  Raw `glob()` output was `zebra`,
   `quartz`,
   `birch`,
   then `amber`.
  `toSorted()` produced `amber`,
   `birch`,
   `quartz`,
   then `zebra`.
- The current repository's forty-two generated `_.path` entries matched
  the root path followed by sorted,
   deduplicated glob results.
  The configured path count and unique path count were both forty-two.
- A second `mise run sync:files` made no change to `mise.toml`.
- `mise env --json --locked --quiet` preserved all forty-two generated paths at
  the start of `PATH`.

## Implemented workaround

The consumer-side `toSorted()` plus `Set` construction is the durable workaround.
It is applied before `file-enforcer` writes generated `mise.toml`,
 so a later
synchronization cannot undo either normalization.

The intentional tradeoff is lexical path precedence for duplicate binaries.
The current installation has thirteen duplicate executable names among the
materialized package bins.
For example,
 the sorted order makes
`package/pi-plugin/advisor/node_modules/.bin` win for `pi` and
`package/git-clone-size/cli/node_modules/.bin` win for `tsc`.

## Regeneration on a partially installed workspace deletes entries

Membership is installation-dependent,
 so `mise run file-enforcer` on a workspace where some packages have no
materialized `node_modules/.bin` writes a shorter `_.path` list than the committed one.
The diff reads as
a deliberate removal and is not one.

Observed 2026-08-06 while regenerating `CLAUDE.md` after an `AGENTS.md` edit.
That run also rewrote
`mise.toml`,
 dropping seven entries including `package/git-policy/cli/node_modules/.bin` and
`package/config/oxlint/node_modules/.bin`,
 on a machine where those directories were absent.

This is the one place `WC2` in `AGENTS.md`,
 "managed -> edit source,
 run file-enforcer,
 commit output
as-is",
 needs a qualifier.
Commit the generated file the edit was for.
Check `git diff` for the generated
`mise.toml` separately,
 and restore it with `git checkout --` when the only change is `_.path` membership,
because those entries belong to whoever has the packages installed rather than to whoever last ran the
generator.

Staging explicit scoped pathspecs,
 which `CLG` already requires,
 is what keeps this from landing by
accident:
 a pathspec naming `AGENTS.md` and `CLAUDE.md` cannot carry a `mise.toml` rewrite with it.

## What does not work

- Re-running an unsorted generator on one machine does not prove portability.
  Unchanged directory entries can look stable while still lacking an ordering
  contract.
- Sorting only generated `mise.toml` is not durable.
  `file-enforcer.config.ts` owns and rewrites the `_.path` section.
- Sorting and deduplication do not fix membership drift.
  Different dependency installation states can produce a different set of
  existing `package/*/*/node_modules/.bin` directories.
  Canonical metadata,
   rather than the installed filesystem,
   would be needed if
  membership must also be identical across machines.
- Filing a `Node.js` defect would misidentify the cause.
  `Node.js` intentionally exposes native directory enumeration;
  the repository needed to normalize it before assigning `PATH` priority.

## Upstream filing decision

The `.out-of-scope/` search found no `Node.js` glob-order exemption.
A duplicate search on 2026-07-11 returned no issues or pull requests for either
`"fs.glob" order` or `glob deterministic` in `nodejs/node`.
There is nothing additive to post.

The filing gate is not met:

1. Upstream fault:
    no.
   `Node.js` exposes underlying directory order;
   the repository now normalizes it at the consumer boundary.
2. Upstream fixability:
    technically yes.
   A global sort would be an upstream behavior change,
    not a repair of a defect.
3. Supported use case:
    no documented `Node.js` guarantee covers deterministic
   glob ordering for generated configuration.
4. Contribution welcome:
    `nodejs/node@v26.5.0:CONTRIBUTING.md` welcomes
   contributions,
    but it requires explicit authorization before external
   automation interacts with the project.
   No such authorization exists for this diagnosis.
5. Likely upstream resolution:
    no basis exists to expect a change to documented
   native-order behavior.
6. Compatible upstream prototype:
    not applicable.
   The consumer-side fix is complete because the first constraint fails.

No issue,
 comment,
 or pull request draft is retained.
Do not file an upstream report as-is.
