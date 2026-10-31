# npm browserslist 4.29.5 resolves `last 1 node major versions` to a Node.js pre-release once node-releases 2.0.62 lists one

Status note (2026-10-30):
npm `browserslist` 4.29.5 is the latest release (published 2026-10-26),
and `node-releases` 2.0.62 (published 2026-10-30) is the latest data.
With that pair,
`last 1 node major versions` returns only `node 27.0.0-alpha.0.0.0`
and hides every `26.x` release.
`package/browserslist/core-rust` drops Node.js pre-releases before evaluating the query,
by the user's decision ("filter pre-releases anyway"),
so its output differs from npm browserslist on purpose.
Nothing was filed upstream.

## Symptom

The repository's `.browserslistrc` carries `last 1 node major versions`.
Before 2026-10-30 it selected the 14 releases of the newest major:

```text
node 26.13.0, node 26.12.0, ... node 26.0.0
```

With node-releases 2.0.62 the same query selects one pre-release and nothing else:

```json
{"query":"last 1 node major versions","result":["node 27.0.0-alpha.0.0.0"]}
```

The whole repository resolution (`.browserslistrc.resolved.local.json`) then ends in
`node 27.0.0-alpha.0.0.0`,
and the Node.js target it names is an unreleased alpha.

Other Node.js queries read the same list and change too
(each was run,
see "Verification"):

- `last 1 node versions` returns `["node 27.0.0-alpha.0.0.0"]`.
- `last 2 node major versions` returns the alpha followed by the whole `26.x` list,
  so it covers one major fewer than intended.
- `node >= 26` includes the alpha as its newest entry.
- `maintained node versions` is unchanged (`node 26.13.0`,
  `node 24.22.0`,
  `node 22.23.0`),
  because it reads the release schedule,
  not `envs.json`.

No error or warning is printed.

## Root cause

Three steps produce it:
the Node.js release index lists a pre-release,
`node-releases` copies it into `envs.json`,
and `browserslist` treats every record of `envs.json` as a release.

### Step 1: nodejs.org lists the pre-release

`https://nodejs.org/dist/index.json` (878 entries when read on 2026-10-30)
lists `v27.0.0-alpha.0.0.0` first,
dated 2026-10-29.
It is the only entry with a pre-release part.

### Step 2: the node-releases build keeps it

`node-releases` regenerates `envs.json` nightly from that index
(`nightly-sync.yml`,
cron `0 3 * * *`,
running `npm run build`).
The build script filters on the patch number only:

```js
// scripts/build.js:13-28 (node-releases e27f3d6d00, version 2.0.62)
// Process Versions
const processed = dist
  .filter(env => semver.patch(env.version) === 0)
  .map(env => ({
    name: 'nodejs',
    version: env.version.substr(1),
```

`semver.patch('v27.0.0-alpha.0.0.0')` is `0`,
so the alpha passes the filter.
Measured with semver 7.6.3:
`prerelease('v27.0.0-alpha.0.0.0')` is `["alpha",0,0,0]`,
and `prerelease('v26.13.0')` is `null`.
The generated file therefore ends with:

```json
// data/processed/envs.json:3066-3073 (node-releases e27f3d6d00)
  {
    "name": "nodejs",
    "version": "27.0.0-alpha.0.0.0",
    "date": "2026-10-29",
    "lts": false,
    "security": false,
    "v8": "15.6.75.25"
  }
```

node-releases 2.0.61 (published 2026-10-29) holds 383 records and none has a dash;
2.0.62 holds 384 and the alpha is the only one with a dash.

### Step 3: browserslist reads every record as a release

`browserslist` loads the file at module initialization
and keeps the `version` of every record,
in file order:

```js
// index.js:1272-1274 (browserslist 4.29.5, 2e4b904b9a)
  browserslist.nodeVersions = jsReleases.map(function (release) {
    return release.version
  })
```

The selector hands that list to `getMajorVersions`:

```js
// index.js:728-738
  last_node_major_versions: {
    matches: ['versions'],
    regexp: /^last\s+(\d+)\s+node\s+major\s+versions?$/i,
    select: function (context, node) {
      return getMajorVersions(browserslist.nodeVersions, node.versions).map(
        function (version) {
          return 'node ' + version
        }
      )
    }
  },
```

```js
// index.js:50-67
function getMajor(version) {
  return parseInt(version.split('.')[0])
}

function getMajorVersions(released, number) {
  if (released.length === 0) return []
  var majorVersions = uniq(released.map(getMajor))
  var minimum = majorVersions[majorVersions.length - number]
  if (!minimum) {
    return released
  }
  var selected = []
  for (var i = released.length - 1; i >= 0; i--) {
    if (minimum > getMajor(released[i])) break
    selected.unshift(released[i])
  }
  return selected
}
```

`parseInt('27.0.0-alpha.0.0.0'.split('.')[0])` is `27`,
so the newest major becomes 27,
`minimum` is 27,
and the backward walk keeps the alpha and stops at `26.13.0`
because `27 > 26`.
Nothing in the path looks at the pre-release part.
`last_node_versions` (index.js:761-771) slices the same list,
and `node_range` and `node_ray` filter it (`browserslist.nodeVersions` at index.js:1013 and index.js:1050),
which is why those queries change too.

The documentation calls the result releases
(`last 2 node major versions`:
"select 2 latest major-version Node.js releases",
README.md:247),
and describes alpha versions as the opposite for browsers
(`unreleased versions`:
"alpha and beta versions",
README.md:278-279).

### Readings that were wrong or too narrow

- The browserslist subset specification
  (`doc/research/browserslist-subset-semantics.md`)
  records `envs.json` as "all `x.y.z`" and the selector as "every `26.x.y`".
  That held for node-releases 2.0.61 and fails from 2.0.62.
  The specification's reading of the algorithm is correct;
  its claim about the data was a property of one revision.
- Browserslist is not the source of the alpha.
  The file `scripts/build.js` shows it comes from the nodejs.org index through a filter that never looks at
  pre-releases.
  Fixing only browserslist hides it from one consumer,
  not from the data.

## Verification

Versions under test
(registry `dist.integrity`,
read 2026-10-30):

- `browserslist@4.29.5`,
  `sha512-khTmKu5NGYTotqcYo8Bew2k4RAE9PuXRSFRXG2lQBrCEQHyeQ/vX6gKMxfokwywBsLiXZMnHsmWMG1pnO8J7uQ==`,
  git tag `4.29.5` at commit `2e4b904b9a971efb8aeb934847152454113e5aab`.
- `node-releases@2.0.62`,
  `sha512-A0Eq+qVDQI97IbAD6g/HriIuv256el9N8EwF4+xqJ49pwrdRYpdCSmLk0sO4I3fnS6hpYE5v22B73Bk+WArEoQ==`,
  repository commit `e27f3d6d00394966bfebae81817ea937271bab4c`.
- `node-releases@2.0.61` (control),
  `sha512-5NUN6QCxNFrS9oEA9RPS+d0AP+ooGBzzovY2JhIpDRHacPvOhHjXjUsUMX9qp4nLHZ+5jVgXB2jj/P4arGOE/g==`.
- `caniuse-lite@1.0.30001820` and the other transitive dependencies as resolved by `npm install` that day;
  none of them enters the Node.js queries.

### Harness

The reproduction runs in a bounded container
(`--memory=2g --cpus=2`),
with a network for the install only and none for the run.
The file `repro.js` prints the installed versions and the answers:

```js
// repro.js
const browserslist = require('browserslist')

const queries = [
  'last 1 node major versions',
  'last 2 node major versions',
  'last 1 node versions',
  'node >= 26',
  'maintained node versions'
]

console.log(
  JSON.stringify({
    browserslist: require('browserslist/package.json').version,
    nodeReleases: require('node-releases/package.json').version,
    newestNodeRecord: require('node-releases/data/processed/envs.json').at(-1).version
  })
)
for (const query of queries) {
  console.log(JSON.stringify({ query, result: browserslist(query, { path: false, stats: {} }) }))
}
```

```sh
# positive: node-releases 2.0.62 (positive/ and control/ each hold a copy of repro.js)
podman run --rm --memory=2g --cpus=2 --security-opt label=disable \
  --volume "$PWD/positive:/work" --workdir /work docker.io/library/node:26.12.0-trixie-slim \
  npm install --no-save --ignore-scripts --no-audit --no-fund browserslist@4.29.5 node-releases@2.0.62
podman run --rm --memory=2g --cpus=2 --network=none --security-opt label=disable \
  --volume "$PWD/positive:/work:ro" --workdir /work docker.io/library/node:26.12.0-trixie-slim node repro.js

# control: the same with node-releases@2.0.61 in control/
```

### Catalog: answers that fail (positive run, 2.0.62)

```text
{"browserslist":"4.29.5","nodeReleases":"2.0.62","newestNodeRecord":"27.0.0-alpha.0.0.0"}
{"query":"last 1 node major versions","result":["node 27.0.0-alpha.0.0.0"]}
{"query":"last 2 node major versions","result":["node 27.0.0-alpha.0.0.0","node 26.13.0", ... ,"node 26.0.0"]}
{"query":"last 1 node versions","result":["node 27.0.0-alpha.0.0.0"]}
{"query":"node >= 26","result":["node 27.0.0-alpha.0.0.0","node 26.13.0", ... ,"node 26.0.0"]}
```

### Catalog: answers that work (control run, 2.0.61, and the unaffected query)

```text
{"browserslist":"4.29.5","nodeReleases":"2.0.61","newestNodeRecord":"26.13.0"}
{"query":"last 1 node major versions","result":["node 26.13.0","node 26.12.0", ... ,"node 26.0.0"]}
{"query":"last 1 node versions","result":["node 26.13.0"]}
{"query":"node >= 26","result":["node 26.13.0", ... ,"node 26.0.0"]}
{"query":"maintained node versions","result":["node 26.13.0","node 24.22.0","node 22.23.0"]}
```

The control returns 14 entries for the first query and the positive run returns 1,
so the harness can tell the two apart.
`maintained node versions` gives the same answer in both runs.

### This repository's conformance run

The conformance oracle
(`package/browserslist/core-rust/bin/oracle-container.ts`)
runs npm browserslist 4.29.5 over the locked data twice per case:
as shipped,
and with pre-release Node.js versions removed from `browserslist.nodeVersions`.
For the repository's seven lines the first answer ends in `node 27.0.0-alpha.0.0.0`
and the second ends in the `26.x` list the crate returns.
`tests/conformance.rs` treats the cases where the two differ as the named exception "node pre-releases",
checks that they differ only in the pre-release Node.js entries,
and requires every other supported case to equal plain npm
(apart from the Firefox ESR entry,
the other accepted exception).

## Verified workarounds

### Drop pre-release Node.js versions before evaluating the query (used here)

`package/browserslist/core-rust/src/select.rs`
(`has_prerelease` and `last_node_major`)
removes every version with a dash,
after removing any `+build` suffix,
from the list before the major-version walk.
Verified by the unit tests in `select_tests.rs`
and by the conformance exception above:
with node-releases 2.0.62 the repository's `.browserslistrc` resolves to
`and_chr 156`,
`and_ff 159`,
`android 156`,
`chrome 156`,
`firefox 153`,
and `node 26.13.0` down to `node 26.0.0`.

Tradeoffs:

- The crate's output differs from npm browserslist 4.29.5 for as long as node-releases lists a pre-release.
  The repository's TypeScript file-enforcer,
  which calls npm browserslist,
  still writes the alpha until it is retired.
- A project that wants to target a Node.js alpha cannot do so through this crate.
- The rule is the semver definition (a dash before any `+build`),
  not a list of names,
  so a future `-rc` or `-nightly` entry is also dropped.
- Only the count 1 is implemented,
  so the behavior of `last 2 node major versions` and similar is not decided here.

### Pin `node-releases@2.0.61` for an npm consumer

The control run shows the old answer with `node-releases@2.0.61` next to `browserslist@4.29.5`.
A package manager override (for example pnpm's `overrides`) would apply it.
Only the install was verified,
not an override setting.

Tradeoffs:
the Node.js release data stops moving,
so new releases are missing until the pin is removed,
and the pin must be dropped once the alpha leaves the data
(the next published node-releases after Node.js removes or promotes it).

### Prototype fixes for upstream (disposable clones, not applied here)

Both were checked with the harness above;
neither ran the upstream test suites.

Fix in node-releases:
the generated data stops containing pre-releases.

```diff
--- a/scripts/build.js
+++ b/scripts/build.js
@@ -12,3 +12,3 @@
 const processed = dist
-  .filter(env => semver.patch(env.version) === 0)
+  .filter(env => semver.patch(env.version) === 0 && semver.prerelease(env.version) === null)
   .map(env => ({
```

Run in a container with the clone mounted (`npm ci`,
then `node scripts/build.js`,
network on),
before the patch the output had 384 records with one pre-release (`27.0.0-alpha.0.0.0`);
after the patch it had 383 records,
none with a dash,
ending at `26.13.0`.
Copying that `envs.json` over the installed file of the positive run made
`last 1 node major versions` return the 14 `26.x` entries again.

Fix in browserslist:
the loader ignores pre-release records.

```diff
--- a/index.js
+++ b/index.js
@@ -1272,3 +1272,7 @@
-  browserslist.nodeVersions = jsReleases.map(function (release) {
-    return release.version
-  })
+  browserslist.nodeVersions = jsReleases
+    .filter(function (release) {
+      return release.version.indexOf('-') === -1
+    })
+    .map(function (release) {
+      return release.version
+    })
```

Applied to the installed `index.js` of the positive run (node-releases 2.0.62 unchanged),
`last 1 node major versions` and `last 1 node versions` return the `26.x` list and `["node 26.13.0"]`.

## What does not work

- `last 2 node major versions` instead of `last 1`:
  it still starts with the alpha and now also covers `26.x`,
  so it is a different set,
  not a workaround.
- `last 1 node versions`:
  it returns only the alpha.
- `node >= 26`:
  it includes the alpha and everything from 26.
- Expecting `maintained node versions` to show the problem:
  it reads the release schedule and is unaffected.
- Reading the alpha as a browserslist-only bug:
  the data file already holds it (Step 2),
  so a fix in browserslist alone leaves every other consumer of `envs.json` exposed.
- Treating the first spec reading ("every record is `x.y.z`") as a guarantee:
  it was a property of 2.0.61.

## Upstream filing artifact

### Upstream filing decision

Nothing was filed.
The user asked that drafts stay local.

`.out-of-scope/` was checked on 2026-10-30:
a search of the whole directory for `browserslist`,
`node-releases` and `caniuse` found no match,
so no exemption applies.

#### 1. Is it really upstream's fault?

Yes for the data file,
with a soft part.
The behavior is a pre-release in a file that the README describes as "node.js releases data preprocessed to be used by
Browserslist" ([node-releases README.md:8][nr-readme]),
and the build filter that admits it never looks at pre-releases (Step 2).
The soft part:
neither project states that pre-releases are excluded or included,
so a maintainer could call the alpha intended.
Browserslist shares the fault:
it documents the result as "releases" ([README.md:246-247][bl-readme-node])
and assumes every record is one (Step 3).
This is behavior,
not wording and not an architectural restriction.

#### 2. Can upstream fix it?

Yes.
Each prototype above is a few lines in one file.

#### 3. Are they supporting this use case?

Yes.
`last N node major versions` is documented ([README.md:247][bl-readme-node]),
and node-releases exists to feed Browserslist ([README.md:8][nr-readme]).
Browserslist's own wording treats alpha as unreleased ([README.md:278-279][bl-readme-unreleased]).

#### 4. Would the repository welcome our contribution?

No policy against it was found.
Checked on 2026-10-30:
the repository roots and `.github` of both projects
(browserslist:
`FUNDING.yml`,
`workflows/release.yml`,
`workflows/test.yml`;
node-releases:
`dependabot.yml`,
`workflows/nightly-sync.yml`);
the GitHub community profiles of both
(no contributing guide,
no issue template,
no pull request template,
browserslist health 50%);
and both READMEs (no contribution or AI-assistance text).
Neither project has a `CONTRIBUTING.md`.
Absence of a policy is not a failure.
Both projects accept issues (`has_issues` is true for both).

#### 5. Will they likely fix it?

No signal against.
No existing issue or pull request mentions the behavior.
Searches run on 2026-10-30 with `gh search issues` and `gh search prs`,
open and closed:

- `browserslist/browserslist`:
  "alpha node" (0 hits),
  "prerelease" (1 hit,
  unrelated by title:
  #326),
  "pre-release" (4 unrelated:
  #842,
  #351,
  #484,
  #483),
  "node major versions" (1 unrelated:
  #860),
  "27.0.0-alpha" (0),
  "node-releases" (5 unrelated,
  for example #468,
  #680),
  "last 1 node" (3 unrelated:
  #1,
  #410,
  #350),
  pull requests "node prerelease" (0) and "getMajorVersions" (0).
- `chicoxyzzy/node-releases`:
  "alpha" (0),
  "prerelease" (0),
  "pre-release" (1 unrelated:
  #18),
  "27.0.0" (0),
  "envs.json" (0),
  pull requests "alpha" (0) and "build script" (1 unrelated,
  merged:
  #33).

The titles of every hit were read;
none concerns a pre-release Node.js version,
so there is no thread to comment on.

Maintainer activity informs the draft but does not gate it.
Browserslist's `index.js` last changed 2026-10-23 and its default branch head is the 4.29.5 tag commit.
Node-releases' `scripts/build.js` last changed 2026-05-12;
its recent commits are the nightly bot,
and its three open pull requests (#43,
#44,
#45) were last updated between 2026-07-01 and 2026-09-21.
The user also told the coordinator that a search of both trackers for 'node alpha' and 'prerelease node' found nothing;
the searches above agree.

Work in progress:
`gh pr list --state all` on both projects showed no open or recent pull request touching the loader or the build filter
(browserslist's open pull requests are #511 and #513,
about normalizers and semver queries;
node-releases' are dependabot and "remove engines requirement");
browserslist has branches `main`,
`gh-pages` and `chore/pnpm-latest` only;
no comment claims the work.

#### 6. Have we prototyped a minimal fix compatible with their architecture?

Yes,
twice (see "Prototype fixes for upstream").
The node-releases fix was built and run against the live nodejs.org index,
and its output was fed to browserslist 4.29.5;
the browserslist fix was run against the installed file.
The upstream test suites were not run.

#### Risk to the poster

Filing puts the poster's GitHub account (the user's) in front of strangers.
No trust or blocklist system was found:
a code search for `vouch` in both repositories returned nothing,
and neither workflow list contains an intake or auto-close workflow
(browserslist:
`release.yml`,
`test.yml`;
node-releases:
`nightly-sync.yml`).
That search is dated 2026-10-30 and must be repeated on the day of posting.
A report written with an agent's help and judged low-effort is the case such systems punish;
the reproduction,
source trace and verified prototypes lower that risk and do not remove it.
Filing is the poster's decision.

#### Readiness check

Run on 2026-10-30 unless noted;
items not listed as run were not run.

- Latest release and other maintained lines:
  browserslist 4.29.5 is the latest (`dist-tags.latest`) and its tag commit is the head of `main`;
  node-releases 2.0.62 is the latest and its commit is the head of `master`.
  Older browserslist majors were not tested.
- Prior art:
  no merged or closed pull request or issue about the behavior was found by the searches above;
  the symbols `getMajorVersions` and `nodeVersions` were searched (pull requests,
  0 hits).
  Pull requests that changed the node-releases build filter were not read beyond #33's title.
- Intake:
  no template or contribution policy exists for either project.
- Automation:
  node-releases publishes through a nightly workflow
  (build,
  commit "feat:
  Nightly Sync",
  version bump,
  npm publish);
  no triage bot was found for a new issue.
- Duplicate,
  work-in-progress and trust searches:
  run above;
  they must be re-run on the day of posting.
- The draft's own reproduction was run as written for the positive and control cases and for both prototypes.
- A clean-runner repository with a CI workflow:
  not run.
- Verdict:
  no recommendation to file is made.
  Missing is the clean-runner reproduction and the day-of-posting re-run.

### Draft

The draft targets node-releases because the data file is where the pre-release enters.
Its title and body are ready to copy once the missing readiness items are done;
if the poster files it,
the body must say that an AI assistant helped produce the reproduction and trace,
and that the poster re-ran them.

~~~md
Title: `envs.json` lists the Node.js pre-release `27.0.0-alpha.0.0.0`, so Browserslist's `last 1 node major versions` resolves to only that alpha

Labels: none available (no issue template)

## Description

Since 2.0.62, `data/processed/envs.json` ends with a pre-release:

    { "name": "nodejs", "version": "27.0.0-alpha.0.0.0", "date": "2026-10-29", "lts": false, ... }

Browserslist 4.29.5 reads every record of this file as a release
(`index.js:1272-1274`, `browserslist.nodeVersions = jsReleases.map(...)`)
and `last 1 node major versions` keeps every record whose major is at least the newest major
(`getMajorVersions`, `index.js:50-67`). With the alpha present the answer is:

    ["node 27.0.0-alpha.0.0.0"]

and with 2.0.61 it was the 14 releases `26.13.0` to `26.0.0`.

## Cause

`scripts/build.js:15` keeps an entry when `semver.patch(env.version) === 0`.
`semver.patch('v27.0.0-alpha.0.0.0')` is `0`, so the entry nodejs.org lists first in `dist/index.json`
(the only pre-release among its 878 entries on 2026-10-30) passes.
`semver.prerelease('v27.0.0-alpha.0.0.0')` is `["alpha",0,0,0]` and `semver.prerelease('v26.13.0')` is `null`.

## Reproduction

    # repro.js
    const browserslist = require('browserslist')
    console.log(require('node-releases/package.json').version)
    console.log(JSON.stringify(browserslist('last 1 node major versions', { path: false, stats: {} })))

    npm install --no-save browserslist@4.29.5 node-releases@2.0.62 && node repro.js
    # 2.0.62
    # ["node 27.0.0-alpha.0.0.0"]

    npm install --no-save browserslist@4.29.5 node-releases@2.0.61 && node repro.js
    # 2.0.61
    # ["node 26.13.0","node 26.12.0", ... ,"node 26.0.0"]

## Suggested fix

In `scripts/build.js`, keep only releases without a pre-release part:

    -  .filter(env => semver.patch(env.version) === 0)
    +  .filter(env => semver.patch(env.version) === 0 && semver.prerelease(env.version) === null)

Run against the live index this gives 383 records, none with a dash, ending at `26.13.0`;
feeding that file to Browserslist 4.29.5 restores the 14-entry answer.
If pre-releases in the file are intended, Browserslist's loader (`index.js:1272`) is the place to ignore them instead,
and this report belongs there.
~~~

[bl-readme-node]: https://github.com/browserslist/browserslist/blob/2e4b904b9a971efb8aeb934847152454113e5aab/README.md?plain=1#L246-L247
[bl-readme-unreleased]: https://github.com/browserslist/browserslist/blob/2e4b904b9a971efb8aeb934847152454113e5aab/README.md?plain=1#L278-L279
[nr-readme]: https://github.com/chicoxyzzy/node-releases/blob/e27f3d6d00394966bfebae81817ea937271bab4c/README.md?plain=1#L8

## Source links

Every source this repository does not own that is cited above is linked here by commit-pinned line range.
Each link was opened in `agent-browser` on 2026-10-30 and the first and last lines of the range were read back.

- browserslist `index.js` at `2e4b904b9a`:
  [line 2][bl-2] (`jsReleases` require),
  [lines 50-67][bl-50] (`getMajor`,
  `getMajorVersions`),
  [lines 728-738][bl-728] (`last_node_major_versions`),
  [lines 761-771][bl-761] (`last_node_versions`),
  [lines 1272-1274][bl-1272] (`browserslist.nodeVersions`).
- node-releases at `e27f3d6d00`:
  [`scripts/build.js` lines 13-28][nr-build],
  [`data/processed/envs.json` lines 3066-3073][nr-envs].
- nodejs.org `dist/index.json` has no line anchor;
  it was read with `curl` on 2026-10-30 (878 entries,
  `v27.0.0-alpha.0.0.0` first,
  dated 2026-10-29).

[bl-2]: https://github.com/browserslist/browserslist/blob/2e4b904b9a971efb8aeb934847152454113e5aab/index.js#L2
[bl-50]: https://github.com/browserslist/browserslist/blob/2e4b904b9a971efb8aeb934847152454113e5aab/index.js#L50-L67
[bl-728]: https://github.com/browserslist/browserslist/blob/2e4b904b9a971efb8aeb934847152454113e5aab/index.js#L728-L738
[bl-761]: https://github.com/browserslist/browserslist/blob/2e4b904b9a971efb8aeb934847152454113e5aab/index.js#L761-L771
[bl-1272]: https://github.com/browserslist/browserslist/blob/2e4b904b9a971efb8aeb934847152454113e5aab/index.js#L1272-L1274
[nr-build]: https://github.com/chicoxyzzy/node-releases/blob/e27f3d6d00394966bfebae81817ea937271bab4c/scripts/build.js#L13-L28
[nr-envs]: https://github.com/chicoxyzzy/node-releases/blob/e27f3d6d00394966bfebae81817ea937271bab4c/data/processed/envs.json#L3066-L3073
