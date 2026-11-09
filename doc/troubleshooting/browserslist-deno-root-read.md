# Browserslist 4.28.4 under Deno 2.8.3 requests root read during stats discovery

> Scratch-path note:
> `/tmp/agent` paths in this document are historical.
> Use `~/temp/agent` for current work.

## Symptom

Running the root file-enforcer config directly with Deno can eventually prompt for read access to `/`:

```text
Deno requests read access to "/".
Requested by `node:fs.existsSync()` API.
```

The prompt appears after Deno has already granted read access to the repo and its parent directories.
It is not emitted by this repo's `existsSync` usage in generated mise tasks.
It was emitted while resolving Browserslist targets from `file-enforcer.config.ts`.

## Current repo status

Fixed in `file-enforcer.config.ts`.
That file was deleted on 2026-10-31 in `e7b5130459`,
so the line citations below count from it,
not from the tree:
the rule is now the `browserslist` kind declared in
`file-enforcer.config.jsonc:200-207`
and implemented in `package/file-enforcer/cli/src/kind_browserslist.rs`.
The config now reads the checked-in `.browserslistrc` path directly:
`file-enforcer.config.ts:29-37`.

```ts
const BROWSERSLIST_CONFIG_PATH = './.browserslistrc';
```

It parses that content without Browserslist config discovery,
 selects a deterministic section,
 and passes direct
queries into Browserslist.
`file-enforcer.config.ts:200-219`:

```ts
function selectBrowserslistQueries(
  { config, }: { readonly config: browserslist.Config; },
): readonly string[] {
  return config[BROWSERSLIST_CONFIG_ENVIRONMENT]
    ?? config.defaults;
}
```

The Browserslist API call disables both path-based config discovery and stats-file discovery at the boundary.
`file-enforcer.config.ts:326-347`:

```ts
const browserslistConfig = resolveBrowserslist.parseConfig(
  await cat([BROWSERSLIST_CONFIG_PATH,],),
);
const targets = resolveBrowserslist(
  selectBrowserslistQueries({ config: browserslistConfig, },),
  {
    path: false,
    stats: EMPTY_BROWSERSLIST_STATS,
  },
);
```

`EMPTY_BROWSERSLIST_STATS` is an explicit empty stats object,
 so Browserslist has no reason to search ancestor
folders for `browserslist-stats.json`.
`file-enforcer.config.ts:51-60`:

```ts
const EMPTY_BROWSERSLIST_STATS: browserslist.Stats = {};
```

## Historical root cause

Before the repo-side fix,
 `file-enforcer.config.ts` called Browserslist with no explicit query and with `path` set
to the repo root:

```ts
const targets = resolveBrowserslist(
  undefined,
  { path: process.cwd(), },
);
```

The installed package under test is `browserslist@4.28.4`.
`pnpm-lock.yaml:4370-4371` pins the tarball and integrity:

- Integrity:
  `sha512-MTc8i/x9jBQd1iMw2CFGS+rwMa07eYjLR0CCTLDACl9xhxy+nIs3KeML/biicXtk9JrZ6dnnTatmc7ErPXIxqw==`.
- Tarball:
  `https://registry.npmjs.org/browserslist/-/browserslist-4.28.4.tgz`.

Browserslist first normalizes options and defaults `opts.path` to the current directory when none is given.
`node_modules/.pnpm/browserslist@4.28.4/node_modules/browserslist/index.js:372-379`:

```js
function prepareOpts(opts) {
  if (typeof opts === 'undefined') opts = {}

  if (typeof opts.path === 'undefined') {
    opts.path = path.resolve ? path.resolve('.') : '.'
  }

  return opts
}
```

It then loads config for missing queries,
 but after that it still checks custom usage stats.
`node_modules/.pnpm/browserslist@4.28.4/node_modules/browserslist/index.js:406-428`:

```js
function browserslist(queries, opts) {
  opts = prepareOpts(opts)
  queries = prepareQueries(queries, opts)
  checkQueries(queries)

  var needsPath = parseQueries(queries).some(function (node) {
    return QUERIES[node.type].needsPath
  })
  var context = {
    ignoreUnknownVersions: opts.ignoreUnknownVersions,
    dangerousExtend: opts.dangerousExtend,
    throwOnMissing: opts.throwOnMissing,
    mobileToDesktop: opts.mobileToDesktop,
    env: opts.env
  }
  // Removing to avoid using context.path without marking query as needsPath
  if (needsPath) {
    context.path = opts.path
  }

  env.oldDataWarning(browserslist.data)
  var stats = env.getStat(opts, browserslist.data)
```

With no `opts.stats` and no `BROWSERSLIST_STATS`,
 `getStat` searches every ancestor for
`browserslist-stats.json`.
`node_modules/.pnpm/browserslist@4.28.4/node_modules/browserslist/node.js:290-304`:

```js
  getStat: function getStat(opts, data) {
    var stats
    if (opts.stats) {
      stats = opts.stats
    } else if (process.env.BROWSERSLIST_STATS) {
      stats = process.env.BROWSERSLIST_STATS
    } else if (opts.path && path.resolve && fs.existsSync) {
      stats = eachParent(
        opts.path,
        function (dir) {
          var file = path.join(dir, 'browserslist-stats.json')
          return isFile(file) ? file : undefined
        },
        statCache
      )
```

The ancestor walk runs until the filesystem root unless `BROWSERSLIST_ROOT_PATH` stops it.
`node_modules/.pnpm/browserslist@4.28.4/node_modules/browserslist/node.js:75-95`:

```js
  do {
    if (!pathInRoot(loc)) {
      break
    }
    if (cache && loc in cache) {
      result = cache[loc]
      break
    }
    pathsForCacheResult.push(loc)

    if (!isDirectory(loc)) {
      continue
    }

    var locResult = callback(loc)
    if (typeof locResult !== 'undefined') {
      result = locResult
      break
    }
  } while (loc !== (loc = path.dirname(loc)))
```

The root check itself is a Node-compat `fs.existsSync` call.
`node_modules/.pnpm/browserslist@4.28.4/node_modules/browserslist/node.js:42-70`:

```js
function getPathType(filepath) {
  var stats
  try {
    stats = fs.existsSync(filepath) && fs.statSync(filepath)
  } catch (err) {
    /* c8 ignore start */
    if (
      err.code !== 'ENOENT' &&
      err.code !== 'EACCES' &&
      err.code !== 'ERR_ACCESS_DENIED'
    ) {
      throw err
    }
    /* c8 ignore end */
  }

  if (stats && stats.isDirectory()) return PATHTYPE_DIR
  if (stats && stats.isFile()) return PATHTYPE_FILE

  return PATHTYPE_UNKNOWN
}

function isFile(file) {
  return getPathType(file) === PATHTYPE_FILE
}

function isDirectory(dir) {
  return getPathType(dir) === PATHTYPE_DIR
}
```

`BROWSERSLIST_ROOT_PATH` is the intended stop condition.
`node_modules/.pnpm/browserslist@4.28.4/node_modules/browserslist/node.js:105-112`:

```js
function pathInRoot(p) {
  if (!process.env.BROWSERSLIST_ROOT_PATH) return true
  var rootPath = path.resolve(process.env.BROWSERSLIST_ROOT_PATH)
  if (path.relative(rootPath, p).substring(0, 2) === '..') {
    return false
  }
  return true
}
```

So the root read is Browserslist checking whether `/` is a directory while looking for an optional
`browserslist-stats.json` file.
Deno's permission layer sees that as `node:fs.existsSync('/')` and asks for read access to `/`.

## Verification

Version and source checks:

- `deno --version`:
   `deno 2.8.3`,
   `typescript 6.0.3`.
- Package version command:

  ```sh
  node <<'JS'
  const p = require(
    './node_modules/.pnpm/browserslist@4.28.4/node_modules/browserslist/package.json',
  );
  console.log(p.version);
  JS
  ```

  Output:
   `4.28.4`.
- `test -e browserslist-stats.json && echo has-stats || echo no-stats`:
   `no-stats`.
- Upstream source clone for comparison:
  `https://github.com/browserslist/browserslist.git` at
  `7cc569488762ed453b68f874402687ae2bae6422` under
  `/tmp/agent/browserslist-deno-root-read-20260626`.

Failing catalog:

```sh
DENO_TRACE_PERMISSIONS=1 deno run \
  --allow-env \
  --allow-read=/var/home/user/Monochromatic,/var/home/user,/var/home,/var \
  - <<'TS'
import { createRequire } from 'node:module';
const require = createRequire(import.meta.url);
const browserslist = require(
  '/var/home/user/Monochromatic/node_modules/.pnpm/browserslist@4.28.4/node_modules/browserslist',
);
browserslist(undefined, { path: '/var/home/user/Monochromatic' });
TS
```

Observed failure,
 with `/var/home/user/Monochromatic` abbreviated to `$REPO` in stack paths:

```text
error: Uncaught (in promise) NotCapable: Requires read access to "/", run again with the --allow-read flag
    stats = fs.existsSync(filepath) && fs.statSync(filepath)
               ^
    at Object.existsSync (ext:deno_node/_fs/_fs_exists.ts:55:10)
    at getPathType (file://$REPO/node_modules/.pnpm/browserslist@4.28.4/node_modules/browserslist/node.js:45:16)
    at isDirectory (file://$REPO/node_modules/.pnpm/browserslist@4.28.4/node_modules/browserslist/node.js:69:10)
    at eachParent (file://$REPO/node_modules/.pnpm/browserslist@4.28.4/node_modules/browserslist/node.js:86:10)
    at Object.getStat (file://$REPO/node_modules/.pnpm/browserslist@4.28.4/node_modules/browserslist/node.js:297:15)
    at browserslist (file://$REPO/node_modules/.pnpm/browserslist@4.28.4/node_modules/browserslist/index.js:427:19)
```

This also fails when the config file is explicit,
 because `prepareOpts` still supplies a default `path`,
and `getStat` still searches ancestors:

```sh
DENO_TRACE_PERMISSIONS=1 deno run \
  --allow-env \
  --allow-read=/var/home/user/Monochromatic,/var/home/user,/var/home,/var \
  - <<'TS'
import { createRequire } from 'node:module';
const require = createRequire(import.meta.url);
const browserslist = require(
  '/var/home/user/Monochromatic/node_modules/.pnpm/browserslist@4.28.4/node_modules/browserslist',
);
browserslist(undefined, { config: '/var/home/user/Monochromatic/.browserslistrc' });
TS
```

Working catalog for the current repo config:

```sh
cd /tmp/agent/monochromatic-file-enforcer-browserslist-20260626
XDG_CONFIG_HOME=/tmp/agent/monochromatic-file-enforcer-browserslist-20260626/xdg-config \
  DENO_TRACE_PERMISSIONS=1 deno run \
  --no-prompt \
  --allow-env \
  --allow-sys=homedir \
  --allow-read=/tmp/agent/monochromatic-file-enforcer-browserslist-20260626,/var/home/user/Monochromatic/node_modules \
  --allow-write=/tmp/agent/monochromatic-file-enforcer-browserslist-20260626 \
  file-enforcer.config.ts
```

The command exits successfully without read access to `/`.
It generated the same `.browserslistrc.resolved.local.json` content as the main worktree:

```text
c41308d4c1442e25524d0d1732ea3472cf0ac8ddc82e72b3dbf9e6d76c4e79cc  .browserslistrc.resolved.local.json
```

Working catalog for the upstream-provided environment boundary:

```sh
BROWSERSLIST_ROOT_PATH=/var/home/user/Monochromatic \
  DENO_TRACE_PERMISSIONS=1 deno run \
  --allow-env \
  --allow-read=/var/home/user/Monochromatic \
  - <<'TS'
import { createRequire } from 'node:module';
const require = createRequire(import.meta.url);
const browserslist = require(
  '/var/home/user/Monochromatic/node_modules/.pnpm/browserslist@4.28.4/node_modules/browserslist',
);
browserslist(undefined, { path: '/var/home/user/Monochromatic' });
TS
```

The command exits successfully and emits no root-read prompt.

Passing empty stats also bypasses stats-file discovery:

```sh
DENO_TRACE_PERMISSIONS=1 deno run \
  --allow-env \
  --allow-read=/var/home/user/Monochromatic \
  - <<'TS'
import { createRequire } from 'node:module';
const require = createRequire(import.meta.url);
const browserslist = require(
  '/var/home/user/Monochromatic/node_modules/.pnpm/browserslist@4.28.4/node_modules/browserslist',
);
browserslist(undefined, { path: '/var/home/user/Monochromatic', stats: {} });
TS
```

That command also exits successfully.

## Verified workarounds

This repo now uses the consumer-side code fix from "Current repo status" rather than requiring a shell
workaround.

For callers that still use Browserslist discovery,
 set `BROWSERSLIST_ROOT_PATH`:

```sh
BROWSERSLIST_ROOT_PATH=. deno file-enforcer.config.ts
```

Tradeoff:
 this preserves Browserslist's config and stats discovery inside the repo,
 but it intentionally prevents
Browserslist from finding `.browserslistrc`,
 `package.json` `browserslist`,
 `browserslist`,
 or
`browserslist-stats.json` files above the repo root.
That is the desired boundary for this monorepo.

Consumer code can also pass `stats: {}` to Browserslist:

```ts
const targets = resolveBrowserslist(
  undefined,
  {
    path: process.cwd(),
    stats: {},
  },
);
```

Tradeoff:
 this disables automatic `browserslist-stats.json` discovery for that call.
It is safe only while this repo's `.browserslistrc` avoids `my stats` queries.

## What does not work

Passing only `config: '/var/home/user/Monochromatic/.browserslistrc'` does not stop the prompt.
Browserslist still calls `prepareOpts`,
 still defaults `opts.path`,
 and still calls `getStat`.
The verification command above reproduces the same `NotCapable: Requires read access to "/"` stack.

Granting `--allow-read=/` also stops the prompt,
 but it broadens the Deno sandbox to full filesystem read access.
That avoids the symptom by granting the exact permission Deno asked for,
 not by narrowing Browserslist's search.

## Upstream filing artifact

### Upstream filing decision

`.out-of-scope/` was checked with `grep` for `browserslist`,
 `deno`,
 and `permission`.
No exemption matched.

A duplicate upstream issue already exists:
[browserslist/browserslist#813](https://github.com/browserslist/browserslist/issues/813),
"Browserslist assumes it has read access to all directories in path".
The issue explicitly names Deno restrictive permissions,
 ancestor config search,
 ancestor stats search,
and the proposed `BROWSERSLIST_ROOT_PATH` stop condition.

The fix already landed in
[browserslist/browserslist#819](https://github.com/browserslist/browserslist/pull/819),
"feat:
 add BROWSERSLIST_ROOT_PATH".
The maintainer comment says it was released in `4.23`,
 and this repo's installed `4.28.4` contains
`pathInRoot` and `BROWSERSLIST_ROOT_PATH` support.

The filing constraints:

- Is it really upstream's fault?
  Yes for the ancestor-walk behavior,
   but upstream already treats it as valid and fixed.
- Can upstream fix it?
  Yes,
   they added `BROWSERSLIST_ROOT_PATH`.
- Are they supporting this use case?
  Yes,
   the README documents `BROWSERSLIST_ROOT_PATH` as preventing reads above the path.
- Would the repo welcome our contribution?
  Historical evidence says yes for this exact issue,
   because PR #819 was merged.
- Will they likely fix it?
  Already fixed.
- Have we prototyped a minimal fix compatible with their architecture?
  Not needed for a new upstream filing,
   because the exact fix is already released and verified locally.

Decision:
 do not file a new issue and do not comment on #813.
There is nothing additive:
 the upstream issue and PR already contain the Deno failure mode,
 stats search,
configuration search,
 and the implemented workaround.

~~~md
No upstream comment to post.
~~~
