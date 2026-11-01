# npm browserslist 4.29.5: semantics of the query subset this repository uses

## Purpose

The user decided, verbatim:
"for browserslist, we retrieve the data and write something conforming to the original browserslist ourselves,
only supporting the queries we use now,
throw on everything non-conforming with \"not implemented\"."

This report specifies exactly what npm `browserslist` 4.29.5 does for those queries,
so an in-repo Rust implementation can conform to it.
It is research only:
no product code was written or changed.
Every behavioral claim cites `path:line` with a quoted excerpt,
and every output claim was reproduced by running the real package in a container (see "Verification").

The queries are the seven lines of `.browserslistrc`
(156 bytes, newline-terminated, no sections, no comments, no commas):

```text
# .browserslistrc
last 1 Chrome versions
last 1 ChromeAndroid versions
last 1 Android versions
Firefox ESR
last 1 FirefoxAndroid versions
last 1 node major versions
not dead
```

## Sources and data revisions

Citation paths are relative to a package directory.
`browserslist/...` means
`node_modules/.pnpm/browserslist@4.29.5/node_modules/browserslist/...` in the repository.
`caniuse-lite/...` and `node-releases/...` are the sibling directories that pnpm links beside it.

Two installs exist and they carry different data.

The repository install is the one the real call path loads.
`node_modules/.pnpm/browserslist@4.29.5/node_modules/` links `caniuse-lite` to `caniuse-lite@1.0.30001820`
and `node-releases` to `node-releases@2.0.61` (`ls -l` shows
`caniuse-lite -> ../../caniuse-lite@1.0.30001820/node_modules/caniuse-lite`).
The committed lockfile pins the same pair:
`pnpm-lock.yaml:10015` `caniuse-lite@1.0.30001820:` and `pnpm-lock.yaml:10786` `node-releases@2.0.61:`
(identical at `HEAD` and in the working tree).
Byte sizes: `caniuse-lite/data/agents.js` 26304,
`caniuse-lite/data/browsers.js` 220,
`caniuse-lite/data/browserVersions.js` 2507,
`node-releases/data/processed/envs.json` 56329.

The gate-probe install named in the task is at `${HOME}/temp/agent/fe-rust/gates/p2/js/node_modules/`
with `caniuse-lite` 1.0.30001819 and `node-releases` 2.0.60.
Byte sizes: 26226, 220, 2498 and 56177.

The four browserslist source files `index.js`, `parse.js`, `node.js` and `error.js` are byte-identical across the two
installs (`cmp` reported no difference for each),
so one set of line citations serves both.
`browserslist/package.json:3` reads `"version": "4.29.5"` in both.

The data differ in two results that matter.
The repository install gives `and_ff 159` and a node list ending at `node 26.13.0`;
the gate-probe install gives `and_ff 158` and a node list ending at `node 26.12.0`.
The repository's current `.browserslistrc.resolved.local.json` (gitignored by `.gitignore:11` `*.local.*`)
equals the repository-install result byte for byte.
A conformance fixture must therefore bundle the three data files it was generated against
(or name the exact package versions) and never mix revisions.

The `index.js` header loads more than these queries use:

- `browserslist/index.js:1`: `var bbm = require('baseline-browser-mapping')`
- `browserslist/index.js:4`: `var e2c = require('electron-to-chromium/versions')`
- `browserslist/index.js:5`:
  `var jsEOL = require('node-releases/data/release-schedule/release-schedule.json')`

They are loaded at module initialization and never consulted by the seven queries
(a `require.cache` listing in the verification run confirms they are loaded;
the selectors in this report never reference `bbm`, `e2c` or `jsEOL`).

## The call path in this repository

- `file-enforcer.config.ts:57`: `const BROWSERSLIST_CONFIG_PATH = './.browserslistrc';`
- `file-enforcer.config.ts:69`: `const BROWSERSLIST_CONFIG_ENVIRONMENT = 'production';`
- `file-enforcer.config.ts:80`: `const EMPTY_BROWSERSLIST_STATS: browserslist.Stats = {};`
- `file-enforcer.config.ts:944-945`:
  `resolveBrowserslist.parseConfig(` / `await cat([BROWSERSLIST_CONFIG_PATH,],),`
- `file-enforcer.config.ts:662-664`:
  `return config[BROWSERSLIST_CONFIG_ENVIRONMENT]` / `?? config.defaults` / `?? [];`
- `file-enforcer.config.ts:951-956`: `resolveBrowserslist(` with the options
  `path: false,` and `stats: EMPTY_BROWSERSLIST_STATS,`
- `file-enforcer.config.ts:961-965`:
  `` content: `${JSON.stringify(targets, null, 2)}\n` `` (the four arguments sit on separate lines).
- `package/file-enforcer/core/src/io/cache.ts:57-59`: `readFile(absPath, 'utf8')`,
  so the file is decoded as UTF-8 (invalid bytes become U+FFFD, as Node does).
- `package/file-enforcer/core/src/io/cat.ts:169`: `return contents.join('\n',);`
  (one file, so the text is unchanged).

Consequences for the Rust implementation:

- The text goes through `parseConfig` first, then the `defaults` array (no `production` section exists) goes to the
  resolver as an array of strings, never as one string.
- The output is `JSON.stringify(targets, null, 2)` plus a newline:
  a JSON array of strings, two-space indent, one element per line, trailing newline;
  an empty result serializes as `[]`.
- `path: false` and `stats: {}` switch off config discovery and stats discovery
  (see "Options and environment that are inert here").

## Config text to query lines

`parseConfig` is `browserslist/node.js:375-408`.

- `browserslist/node.js:376-377`: `var result = { defaults: [] }` and `var sections = ['defaults']`.
- `browserslist/node.js:381`: `.replace(/#[^\n]*/g, '')` strips a `#` and everything to the end of the line
  (anywhere in a line, not only at its start).
- `browserslist/node.js:382`: `.split(/\n|,/)` splits on LF and on comma only.
  A lone CR or U+2028 does not split (verified: `last 1 Chrome versions\rFirefox ESR` stays one line and then fails as
  an unknown query).
- `browserslist/node.js:383-388`: each piece is `.trim()`med and empty pieces are dropped.
- `browserslist/node.js:8`: `var IS_SECTION = /^\s*\[(.+)]\s*$/`.
  A line matching it starts a section (`sections = line.match(IS_SECTION)[1].trim().split(' ')`, line 391);
  a repeated name throws `Duplicate section <name> in Browserslist config` (lines 393-396);
  this includes `[defaults]`, because `result.defaults` already exists.
  Any other line is pushed onto every current section (lines 401-403).
  A line such as `chrome [1]` is not a section header (verified).

For the seven-line file the result is exactly
`{ defaults: [the seven strings, in file order] }` (verified, see "Verification").

Because commas are consumed here as line separators,
the parser's own comma handling (see "Query parsing") is unreachable for config files.
A CRLF file works because `trim()` removes the CR.
A leading U+FEFF is removed by `trim()` (verified: `parseConfig('﻿last 1 Chrome versions\n')`
gives `defaults: ['last 1 Chrome versions']`).

## Query parsing

The array of lines enters `browserslist(queries, opts)` at `browserslist/index.js:424-427`:
`prepareOpts`, `prepareQueries`, then `checkQueries`.
`path: false` is kept as `false` because `browserslist/index.js:384` only defaults `opts.path` when it is
`typeof opts.path === 'undefined'`.
`prepareQueries` (`browserslist/index.js:391-402`) loads a config only when `queries` is `undefined` or `null`;
an empty array is a valid query list and resolves to `[]` (verified: `[]` and `''` both give `[]`).
`checkQueries` (`browserslist/index.js:404-410`) throws
`Browser queries must be an array or string. Got <typeof>.` for anything else.

### Joining lines into blocks

`browserslist/parse.js:150-173`:

- Lines of length zero are skipped: `if (block.length === 0) return` (line 157).
- A line is pushed to `pending` (line 158); if it does not end with an operator, `pending.join('\n')` becomes one
  block (lines 159-161).
  So a line ending with `or`, `and` or a comma joins the line that follows it (`endsWithOperator`,
  lines 3-23). Line 164 parses a dangling continuation too.
- None of the seven lines ends with an operator, so each line is its own block.
  `endsWithOperator('Firefox ESR')` is false: the lowercase three-character tail is `esr`, whose last two characters
  are `sr`, not `or` (`browserslist/parse.js:12-18`).

Every block is parsed independently (`browserslist/parse.js:166-172`) and the node lists are flattened in order
(`flatten`, lines 25-42).
The first clause of every block has `compose = 'or'` (`browserslist/parse.js:85`: `var compose = 'or'`),
so separate lines combine as a union, applied in order.

### Splitting a block into clauses

`parseBlock` (`browserslist/parse.js:79-148`) scans left to right.
Clause delimiters are a comma (`,` plus trailing whitespace, lines 91-100),
whitespace then `and` then whitespace (lines 108-123, `compose = 'and'`),
and whitespace then `or` then whitespace (lines 124-136, `compose = 'or'`).
The keyword test is ASCII case-insensitive and needs a whitespace character right after it
(`SPACE.test(block[q + 3])`, line 113; `SPACE.test(block[q + 2])`, line 128).
`SPACE` is `/\s/` (`browserslist/parse.js:1`).

None of the seven lines contains a delimiter,
but one line is a trap for hand-written scanners:
in `last 1 Android versions` the word `Android` begins with `And` after whitespace.
It is not a delimiter because the character after `And` is `r`, not whitespace.
Verified: `last 1 Android versions` resolves to `["android 156"]`.

Each clause is trimmed and matched (`browserslist/parse.js:68-72`:
`var node = matchQuery(all, text.trim())`).
A clause that is empty after trimming (verified with the whole-line query `' '`) matches no regexp and ends in the
`unknown` handler, which throws an unknown-query error whose quoted query text is empty.
A dangling `or` at the end of a line is not a delimiter, because the keyword test needs a whitespace character after
it, so the whole text, including the `or`, becomes one unknown query (verified).

### The not prefix

`browserslist/parse.js:46`: ``var not = query.match(/^not\s+(?!\s)/i)``.
If it matches, `node.not = true` and the prefix is removed (`browserslist/parse.js:47-50`).
`NOT   DEAD` and `not<TAB>dead` therefore parse as `not dead` (verified); `notdead` does not
(it is an unknown query, verified); `not not dead` is an unknown query (verified).

### Regexp matching order

`matchQuery` tries every entry of `QUERIES` in object key order and takes the first match
(`browserslist/parse.js:52-62`):
`for (var name in all) {` ... `var match = query.match(type.regexp)` ... `return node`.
The key order is the source order of `browserslist/index.js:689-1243`.
The captures are named by each entry's `matches` array (`browserslist/parse.js:57-59`).

Regexps relevant to the seven lines, in the order they are tried:

- `browserslist/index.js:692` `last_major_versions`: ``/^last\s+(\d+)\s+major\s+versions?$/i``
- `browserslist/index.js:706` `last_versions`: ``/^last\s+(\d+)\s+versions?$/i``
- `browserslist/index.js:720` `last_electron_major_versions`: ``/^last\s+(\d+)\s+electron\s+major\s+versions?$/i``
- `browserslist/index.js:730` `last_node_major_versions`: ``/^last\s+(\d+)\s+node\s+major\s+versions?$/i``
- `browserslist/index.js:741` `last_browser_major_versions`: ``/^last\s+(\d+)\s+(\w+)\s+major\s+versions?$/i``
- `browserslist/index.js:752` `last_electron_versions`: ``/^last\s+(\d+)\s+electron\s+versions?$/i``
- `browserslist/index.js:763` `last_node_versions`: ``/^last\s+(\d+)\s+node\s+versions?$/i``
- `browserslist/index.js:774` `last_browser_versions`: ``/^last\s+(\d+)\s+(\w+)\s+versions?$/i``
- `browserslist/index.js:1059` `browser_ray`: ``/^(\w+)\s*(>=?|<=?)\s*([\d.]+|esr)$/i``
- `browserslist/index.js:1081` `firefox_esr`: ``/^(firefox|ff|fx)\s+esr$/i``
- `browserslist/index.js:1164` `browser_version`: ``/^(\w+)\s+(tp|[\d.]+)$/i``
- `browserslist/index.js:1217` `dead`: ``/^dead$/i``
- `browserslist/index.js:1232` `unknown`: ``/^(\w+)$/i``

Which entry matches each line (verified by running `parse`, and by testing every regexp against every line, which
lists all matching keys in order):

- `last 1 Chrome versions` matches `last_browser_versions` with `versions = '1'`, `browser = 'Chrome'`.
  No other key matches.
- `last 1 ChromeAndroid versions` matches `last_browser_versions` with `browser = 'ChromeAndroid'`.
- `last 1 Android versions` matches `last_browser_versions` with `browser = 'Android'`.
- `Firefox ESR` matches `firefox_esr` (no captures). No other key matches.
- `last 1 FirefoxAndroid versions` matches `last_browser_versions` with `browser = 'FirefoxAndroid'`.
- `last 1 node major versions` matches both `last_node_major_versions` and `last_browser_major_versions`
  (the latter with `browser = 'node'`).
  The first wins only because of key order, and the second would throw `Unknown browser node`.
  A Rust matcher that tests `last N <word> major versions` before `last N node major versions` is wrong.
- `not dead`: after the `not` strip, `dead` matches `dead` and also `unknown`; `dead` wins by order.

The `unknown` handler (`browserslist/index.js:1230-1242`) throws
``Specify versions in Browserslist query for browser <query>`` when the lone word is a known browser,
else ``Unknown browser query `<query>`. Maybe you are using old Browserslist or made typo in query.``

### Regexp semantics that differ from Rust defaults

This is the main conformance trap.
All regexps quoted in this report are JavaScript regexps without the `u` flag.

- `\s` is the ECMAScript WhiteSpace plus LineTerminator set, measured in Node 26.12.0 as
  `[\t\n\v\f\r    -     　﻿]`.
  It includes U+FEFF and excludes U+0085 (NEL).
  `String.prototype.trim` strips exactly the same set (0 differences over all 65536 code units).
- Rust `char::is_whitespace` (rustc 1.101.0-nightly, measured over the BMP with `ws.rs`) is
  `9-d 20 85 a0 1680 2000-200a 2028-2029 202f 205f 3000`.
  It includes U+0085 and excludes U+FEFF, so the two sets differ on exactly those two characters.
- `\w` is `[A-Za-z0-9_]` and `\d` is `[0-9]` (verified: `/\w/.test('é')`, `/\d/.test('٣')` and `/\w/.test('ſ')`
  are all false).
  The Rust `regex` crate defaults to Unicode-aware `\w`, `\d` and `\s`
  (`regex-1.13.1/src/lib.rs:571`:
  "The character classes `\w`, `\d` and `\s` are all Unicode-aware by default.").
- The `i` flag without `u` does not fold non-ASCII to ASCII (verified: `/s/i.test('ſ')` and `/k/i.test('K')`
  are false). The Rust `regex` crate's case-insensitive matching is Unicode-aware
  (`regex-1.13.1/src/lib.rs:580`: "Case insensitive searching is Unicode-aware and uses simple case folding."),
  and Unicode simple case folding maps U+017F to `s` and U+212A to `k`, so it would match them.

Verified conformance cases, all through the real `browserslist(..., { path: false, stats: {} })` call:

- `last 1 Chrome versions` (NBSP) gives `["chrome 156"]`.
- `last﻿1 Chrome versions` gives `["chrome 156"]`.
- `last　1 Chrome versions` (ideographic space) gives `["chrome 156"]`.
- `last 1 Chrome versions ` (trailing NBSP) gives `["chrome 156"]`.
- `last\u00851 Chrome versions` (NEL) throws an unknown-query error (NEL is not whitespace;
  the message shows the invisible NEL as `last<NEL>1 Chrome versions`).
- `last 1 Chrome versions\u0085` (trailing NEL) throws an unknown-query error (trim does not strip NEL).
- `last 1 Chrome verſions` (long s) throws an unknown-query error.
- `Firefox EſR` throws an unknown-query error.
- ``last １ Chrome versions`` (fullwidth digit) throws an unknown-query error.

Recommendation for Rust:
implement an explicit whitespace predicate with the JavaScript set,
ASCII-only word and digit classes,
and ASCII-only case-insensitive comparison (`eq_ignore_ascii_case`),
rather than the default `regex` configuration.

### Case and whitespace of the seven forms

Every regexp for the seven forms carries the `i` flag.
Verified results: `LAST 1 CHROME VERSION`, `last   1<TAB>Chrome  versions`, `last 1 chrome versions`,
`firefox esr`, `FX ESR`, `ff   esr`, `Firefox<TAB>ESR` all resolve (to `chrome 156` or `firefox 153`);
`last 1 chromeandroid version`, `last 1 ANDROID versions`, `last 1 firefoxandroid versions`,
`last 1 NODE MAJOR VERSIONS` resolve to the same values as their canonical spellings.
The singular `version` is accepted because every regexp ends in `versions?`.
Leading zeros in the count are accepted (`last 01 Chrome versions` gives `["chrome 156"]`).
Leading and trailing whitespace is removed by `trim` (`browserslist/parse.js:69`).
The output never echoes the input spelling: names come from the data (`data.name`).

## Selector semantics

### Browser names, aliases and the data table

`byName` (`browserslist/index.js:252-266`):

```js
// browserslist/index.js:252-254, 265
function byName(name, context) {
  name = name.toLowerCase()
  name = browserslist.aliases[name] || name
  ...
  return browserslist.data[name]
```

- `browserslist/index.js:519-532` (`browserslist.aliases`) includes
  `chromeandroid: 'and_chr'` (line 528) and `firefoxandroid: 'and_ff'` (line 529).
  `android` and `chrome` are not aliases; they are `data` keys.
- `checkName` (`browserslist/index.js:301-305`) throws ``'Unknown browser ' + name`` when `byName` is falsy, using the
  spelling the user wrote.

So the three browser captures resolve as: `Chrome` to `chrome`, `ChromeAndroid` to `and_chr`,
`Android` to `android`, `FirefoxAndroid` to `and_ff`.
The result strings use the resolved key (`data.name`), for example `and_chr 156`, not the written spelling.

### Released versions

`browserslist/index.js:1247-1275` builds `browserslist.data[name]` for each caniuse agent:

```js
// browserslist/index.js:1252-1254
versions: normalize(agents[name].versions),
released: normalize(agents[name].versions.slice(0, -3)),
releaseDate: agents[name].release_date
```

- `normalize` (`browserslist/index.js:30-34`) keeps only elements with `typeof version === 'string'`,
  which removes the `null` placeholders.
- The `slice(0, -3)` is positional on the raw array (the one that still contains `null`s),
  so it drops the last three slots of the 163-slot array, and only then are `null`s removed.
  Every agent's array ends in exactly three slots reserved for unreleased versions.
  For Chrome those three slots are 157, 158 and 159, so `released` ends at 156.
  For `and_chr`, `and_ff`, `android` and the dead browsers the last three slots are all `null`.

Facts about the repository-install data for the relevant keys (verified from `browserslist.data`):

- `chrome`: 155 versions, 152 released, last released `156`.
- `and_chr`: `versions = ["156"]`, `released = ["156"]`.
- `android`: `["2.1","2.2","2.3","3","4","4.1","4.2-4.3","4.4","4.4.3-4.4.4","156"]`, all released.
- `firefox`: last released `159`; unreleased 160, 161, 162.
- `and_ff`: `["159"]`.

### last N browser versions

`browserslist/index.js:772-781`:

```js
// browserslist/index.js:775-780
select: function (context, node) {
  var data = checkName(node.browser, context)
  var list = data.released.slice(-node.versions).map(nameMapper(data.name))
  list = filterJumps(list, data.name, node.versions, context)
  return list
}
```

- `nameMapper` (`browserslist/index.js:44-48`) produces `name + ' ' + version`.
- `node.versions` is the captured digit string; `slice(-'1')` coerces to `slice(-1)`, the last released entry.
- For N = 1 the result is the single last released version of the browser.
  Results with the repository data: Chrome `chrome 156`, ChromeAndroid `and_chr 156`, Android `android 156`,
  FirefoxAndroid `and_ff 159`.
- Generalization (not needed for N = 1, but recorded because a Rust implementation that accepts other counts must
  match it):
  `slice(-N)` returns the last N released entries in ascending order;
  N larger than the list returns the whole list;
  `N = 0` gives `slice(-0)` which is `slice(0)`, the whole list (verified: `last 0 Chrome versions` returns every
  released Chrome version);
  a count of 20 digits, and even 400 digits, also returns the whole list (verified), so a Rust port needs saturating
  parsing of the digit string.

### Android and filterJumps

`browserslist/index.js:316-337`:

```js
// browserslist/index.js:318-337 (android branch and tail)
function filterJumps(list, name, nVersions, context) {
  var jump = 1
  switch (name) {
    case 'android':
      if (context.mobileToDesktop) return list
      var released = browserslist.data.chrome.released
      jump = released.length - released.indexOf(ANDROID_EVERGREEN_FIRST)
      break
    ...
    default:
      return list
  }
  if (nVersions <= jump) {
    return list.slice(-1)
  }
  return list.slice(jump - 1 - nVersions)
}
```

- `browserslist/index.js:13`: `var ANDROID_EVERGREEN_FIRST = '37'`.
- Only the key `android` is adjusted here.
  The `op_mob` case (lines 326-329, using `OP_MOB_BLINK_FIRST = 14` from line 14) is not reachable from these queries.
  `chrome`, `and_chr` and `and_ff` take `default: return list`, unchanged.
- With the repository data, Chrome `released` has 152 entries and `indexOf('37')` is 33, so `jump` is 119.
  For N = 1, `1 <= 119`, and the result is `list.slice(-1)`, the last element of a one-element list.
  So for N = 1 the Android answer is the last released Android entry and does not depend on Chrome data.
- For N greater than `jump`, the result is `list.slice(jump - 1 - N)`,
  the last `N - jump + 1` entries of the N-entry list.
  Verified: `last 119 Android versions` gives `["android 156"]`, and `last 120 Android versions` gives
  `["android 156","android 4.4.3-4.4.4"]`.
  The comparison `nVersions <= jump` coerces the digit string to a number, so N = 0 also takes the `slice(-1)` branch
  (verified: `last 0 Android versions` gives `["android 156"]`, while `last 0 Chrome versions` returns all 152 released
  Chrome versions).

### mobileToDesktop

`mobileToDesktop` is read from `opts` at `browserslist/index.js:436`
(`mobileToDesktop: opts.mobileToDesktop`) and is used in `byName` (line 255) and `filterJumps` (line 322).
The file-enforcer call passes no such option (`file-enforcer.config.ts:951-956`), so it is `undefined`, which is falsy,
and `byName` returns `browserslist.data[name]` unchanged (line 265).
The `normalizeAndroidVersions` and `normalizeAndroidData` helpers (`browserslist/index.js:268-299`)
are called only from the `mobileToDesktop` branch of `byName` (line 258) and are never reached.
Positive evidence that the option is inert for these seven lines anyway:
running the seven lines with `mobileToDesktop: true` returned an identical array.
Because the default is false, the Rust implementation should throw "not implemented" if it ever receives the option.

### last N node major versions

`browserslist/index.js:728-738`:

```js
// browserslist/index.js:732-736
return getMajorVersions(browserslist.nodeVersions, node.versions).map(
  function (version) {
    return 'node ' + version
  }
)
```

`getMajorVersions` (`browserslist/index.js:54-67`):

```js
// browserslist/index.js:54-67
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

- `getMajor` is ``parseInt(version.split('.')[0])`` (line 51); `uniq` (lines 69-75) keeps the first occurrence.
- `browserslist.nodeVersions` is the `version` field of every record of
  `node-releases/data/processed/envs.json`, in file order (`browserslist/index.js:1272-1274`:
  ``browserslist.nodeVersions = jsReleases.map(function (release) { return release.version })``).
- The algorithm never sorts.
  It relies on the file being ascending.
  The repository-install `envs.json` has 383 records, all `x.y.z`, strictly ascending from `0.2.0` to `26.13.0`
  (verified), with 24 distinct majors: 0, 4 to 26.
  An implementation must preserve file order and must not sort or dedupe versions itself.
- For N = 1: `minimum` is the last distinct major, 26; the loop walks from the end and keeps every release whose major
  is at least 26. The result is every `26.x.y` in ascending order, mapped to `node <version>` (the final sort
  reorders them). With the repository data that is 14 versions, `26.0.0` to `26.13.0`.
  It is not "the latest version only".
- The falsy test `!minimum` covers both `undefined` (index out of range, which happens when N is larger than the
  number of majors, or when N = 0 because `majorVersions[length - 0]` is `undefined`) and a minimum of major `0`.
  In both cases the whole list is returned.
  Verified: `last 0 node major versions` returns the whole list; `last 2 node major versions` adds the `25.x` line;
  `last 30 node major versions` and a 400-digit count return all 383 versions.
  The regexp accepts a digit string of any length, so a Rust port needs saturating parsing.

### Firefox ESR

`browserslist/index.js:15`: `var FIREFOX_ESR_VERSION = '153'`.
`browserslist/index.js:1079-1085`:

```js
// browserslist/index.js:1079-1085
firefox_esr: {
  matches: [],
  regexp: /^(firefox|ff|fx)\s+esr$/i,
  select: function () {
    return ['firefox ' + FIREFOX_ESR_VERSION]
  }
},
```

The result is the constant `firefox 153`.
It is not looked up in caniuse data, is not checked against the released list, and does not change with the data
(it is the same under both data revisions).
`browserslist/index.js:1277` also sets ``browserslist.versionAliases.firefox.esr = FIREFOX_ESR_VERSION``,
used by the `firefox >= esr` ray form only, which is not part of these queries.
The constant lives in browserslist's JavaScript source, not in caniuse-lite or node-releases.
Updating it is a browserslist release change; a Rust implementation has to source it separately
(see "Open decisions").

### dead, and how not subtracts it

`browserslist/index.js:1215-1229`:

```js
// browserslist/index.js:1215-1228
dead: {
  matches: [],
  regexp: /^dead$/i,
  select: function (context) {
    var dead = [
      'Baidu >= 0',
      'ie <= 11',
      'ie_mob <= 11',
      'bb <= 10',
      'op_mob <= 12.1',
      'samsung 4'
    ]
    return resolve(dead, context)
  }
},
```

`resolve(dead, context)` parses the six strings as six blocks, each of which is one clause with `compose = 'or'`,
and evaluates them with the same `resolve` function.
Two internal forms are used:

Form `browser_ray` (`browserslist/index.js:1057-1078`) for `Baidu >= 0`, `ie <= 11`, `ie_mob <= 11`, `bb <= 10`
and `op_mob <= 12.1`:

- `checkName(node.browser)` resolves the name case-insensitively, so `Baidu` is `baidu`.
- Numeric versions are kept as written (`browserslist/index.js:1066`: ``if (alias && !/^[\d.]+$/.test(version))``).
- It returns `data.released.filter(generateFilter(sign, version))` mapped to ``data.name + ' ' + v``.
- `generateFilter` (`browserslist/index.js:103-121`):
  `>=` keeps ``parseLatestFloat(v) >= parseLatestFloat(version)``;
  `<=` keeps ``parseFloat(v) <= parseFloat(version)``.
  `parseLatestFloat` (lines 123-125) is ``parseFloat(v.split('-')[1] || v)``, so for a joined range like `15.2-15.3`
  it compares the upper end; for plain versions it equals `parseFloat`.
  `parseFloat` parses the longest numeric prefix of the string (`5.5`, `11.1`, `12.1`).

Form `browser_version` (`browserslist/index.js:1162-1191`) for `samsung 4`:

- `normalizeVersion(data, '4')` (lines 219-228) succeeds when `'4'` is in `data.versions` (`resolveVersion`, lines
  209-217), and the result is `['samsung 4']`.
- If it were absent, the code tries `4.0`; if that fails too it throws ``Unknown version 4 of samsung``
  (lines 1181-1187), unless `ignoreUnknownVersions` is set.
  `normalizeVersion` also returns the only version of a one-version browser as a fallback (line 223-224).

Expansion of the six strings with the repository data (verified, each run alone):

- `Baidu >= 0`: `baidu 13.52`, because `parseLatestFloat('13.52') >= 0`; every released Baidu version qualifies.
- `ie <= 11`: `ie 11`, `ie 10`, `ie 9`, `ie 8`, `ie 7`, `ie 6`, `ie 5.5` (all seven released IE versions).
- `ie_mob <= 11`: `ie_mob 11`, `ie_mob 10`.
- `bb <= 10`: `bb 10`, `bb 7`.
- `op_mob <= 12.1`: `op_mob 12.1`, `op_mob 12`, `op_mob 11.5`, `op_mob 11.1`, `op_mob 11`, `op_mob 10`
  (`op_mob 80` is excluded).
- `samsung 4`: `samsung 4`.

Together that is 19 strings.

`not` subtraction is in `resolve` (`browserslist/index.js:346-379`):

```js
// browserslist/index.js:346-379 (abridged to the load-bearing statements)
function resolve(queries, context) {
  return parseQueries(queries).reduce(function (result, node, index) {
    if (node.not && index === 0) {
      throw new BrowserslistError(
        'Write any browsers query (for instance, `defaults`) ' + 'before `' + node.query + '`'
      )
    }
    ...
    if (!node.not && node.compose === 'or') {
      for (var i = 0; i < array.length; i++) { result.push(array[i]) }
      return result
    }
    var filter = new Set(array)
    var filterKeep = !node.not
    return result.filter(function (j) { return filter.has(j) === filterKeep })
  }, [])
}
```

- A plain `or` clause appends its strings to the accumulator (duplicates allowed until the final `uniq`).
- A `not` clause (any compose) removes from the accumulator every string that is in the clause's set,
  by exact string equality, preserving order.
  An `and` clause (not used here) keeps only strings in the set.
- A `not` node at flattened index 0 throws, with the query text as written after trimming:
  ``Write any browsers query (for instance, `defaults`) before `not dead` ``
  (verified; also when `not dead` is the first
  line of a longer file).
- The strings produced by a selector pass through a mapper (`browserslist/index.js:357-364`) that rewrites
  ``<name> 0`` to the first version of the browser (`if (parts[1] === '0')`, line 359).
  No selected string in these queries has the version `0`, so this rewrite never fires.
- The subtraction applies to everything accumulated so far, across line boundaries, not only to the adjacent line.

## Result post-processing

`browserslist/index.js:456-473`:

```js
// browserslist/index.js:456-469
var result = uniq(resolve(queries, context)).sort(function (name1, name2) {
  name1 = name1.split(' ')
  name2 = name2.split(' ')
  if (name1[0] === name2[0]) {
    // assumptions on caniuse data
    // 1) version ranges never overlaps
    // 2) if version is not a range, it never contains `-`
    var version1 = name1[1].split('-')[0]
    var version2 = name2[1].split('-')[0]
    return compareSemver(version2.split('.'), version1.split('.'))
  } else {
    return compare(name1[0], name2[0])
  }
})
```

- Duplicates are removed first, keeping the first occurrence (`uniq`, `browserslist/index.js:69-75`).
- Browser names sort ascending with JavaScript string comparison (`compare`, lines 158-162, uses `<` and `>`),
  which compares UTF-16 code units. For ASCII names this is byte order, so `_` (0x5F) sorts before letters:
  `and_chr`, `and_ff`, `android`, `chrome`, `firefox`, `node`.
- Within one name, versions sort descending: `compareSemver(version2.split('.'), version1.split('.'))`
  with `compareSemver` (lines 164-170) comparing `parseInt` of the first three dot-separated components, with
  missing components as 0. The part after a `-` is ignored (`split('-')[0]`).
  A non-numeric component such as `TP` parses to `NaN`, `compare` returns 0 for `NaN`, and the stable sort
  (`Array.prototype.sort` is stable) keeps the incoming order. None of the selected versions here is non-numeric.
- Each output element is `<name> <version>` with a single ASCII space.
- The result is cached in a process-local map keyed by `JSON.stringify([queries, context])`
  (`browserslist/index.js:453-454`, `470-472`); the cache has no effect on the value.

## Data inputs actually read

### caniuse-lite files

Loaded and used by these queries, via `browserslist/index.js:3`
(``var agents = require('caniuse-lite/dist/unpacker/agents').agents``):

- `caniuse-lite/data/agents.js` (26304 bytes in the repository install)
- `caniuse-lite/data/browsers.js` (220 bytes)
- `caniuse-lite/data/browserVersions.js` (2507 bytes)
- `caniuse-lite/dist/unpacker/agents.js` (the decoder; 1439 bytes)

`caniuse-lite/dist/unpacker/browsers.js` and `browserVersions.js` are one-line wrappers
(``module.exports.browsers = require('../../data/browsers')``).

Of `agents.js`, these queries read only the version list (entry `C`) of the browsers `chrome`, `and_chr`, `android`,
`firefox`, `and_ff`, and for `dead` also `baidu`, `ie`, `ie_mob`, `bb`, `op_mob`, `samsung`.
The entries `A` (usage), `B` (prefix), `D` (prefix exceptions), `E` (display name) and `F` (release dates)
are decoded at load time but unused by these queries.

### Packed grammar

All three data files have the same envelope: the exact bytes `module.exports=`, one value, `;`, and a final newline
(measured: prefix `module.exports=`, suffix `;\n`, no other whitespace, no non-ASCII bytes, no escapes in strings,
no trailing commas, in both installs).
The JavaScript literal subset in use, measured by a strict tokenizer over the repository-install files:

- Objects `{ key: value, ... }` and arrays `[ value, ... ]`, with comma separators only.
  There are three container levels: the top object, the browser object, and the `C` array or `A`/`D`/`F` object.
- Object keys are either unquoted identifiers (`[A-Za-z][A-Za-z0-9]*`, 1352 occurrences in `agents.js`)
  or double-quoted strings (200 occurrences).
  The rule is: a key is quoted exactly when it starts with a digit
  (verified: every quoted key starts with a digit and no
  unquoted key does). Quoted keys are `"0"` to `"9"` and codes such as `"0B"`.
- String values are double-quoted with no escape sequences.
  Their characters are letters, digits, `.`, `-`, `_` (in `browsers.js`) and a space (display names in `agents.js`).
  The empty string `""` is used 2390 times in `agents.js`, always as a slot in a `C` array (verified: none elsewhere).
- Numbers are non-negative integers (`0`) or plain decimals (`0.211788`).
  No exponent form, no leading-dot form and no sign appears (verified).
- `null` appears 11 times in `agents.js`, only as a value in the `F` (release date) objects of unreleased versions
  (Firefox 160 to 162, Chrome 157 to 159, Safari 27.1, 27.2 and `TP`, iOS Safari 27.1 and 27.2).
  The `C` arrays contain only strings.
- No `true`, `false`, `undefined`, comments, template strings or computed keys appear.

Top-level shape of `agents.js` (the tokenizer's per-browser listing):
19 browser entries keyed `A` to `S`, each an object with keys `A`, `B`, `C`, `E`, `F` and, for four browsers, `D`.
Every `C` array has exactly 163 entries in the repository install (162 in the gate-probe install).

Head of the Chrome entry, the start and the end of its `C` array:

```text
// caniuse-lite/data/agents.js:1 (excerpts; the file is a single line)
,D:{A:{"0":0.074518,"1":0.219632,...
C:["","","","","","","","","I","oB","K",...
...,"kB","lB","mB","nB","QB","lC","mC","aC"],E:"Chrome",F:{"0":1694476800,"1":1696896000,...
```

`browsers.js` (the whole file):

```text
// caniuse-lite/data/browsers.js:1 (one line, wrapped here at commas for reading)
module.exports={A:"ie",B:"edge",C:"firefox",D:"chrome",E:"safari",F:"opera",G:"ios_saf",H:"op_mini",
I:"android",J:"bb",K:"op_mob",L:"and_chr",M:"and_ff",N:"ie_mob",O:"and_uc",P:"samsung",Q:"and_qq",
R:"baidu",S:"kaios"};
```

`browserVersions.js` maps 263 distinct codes to version strings, for example
`"0":"117"`, `QB:"156"`, `aC:"159"`, `ME:"13.52"`, `NB:"28"`, `pC:"15.2-15.3"`, `YD:"TP"`.
All 263 values are distinct, and 102 of them are not plain integers (for example ranges and decimals).
Every code used in any `C`, `A` or `F` is a key of this table (0 unknown codes, verified).

### How the unpacker decodes them

`caniuse-lite/dist/unpacker/agents.js`:

- Lines 3-5 load `browsers`, `browserVersions` and `data/agents`.
- Line 14, ``module.exports.agents = Object.keys(agentsData).reduce((map, key) => {``, and line 16,
  ``map[browsers[key]] = ...``: the one-letter browser code becomes the browser name key.
- Lines 19-27, the `C` branch:

```js
// caniuse-lite/dist/unpacker/agents.js:19-27
} else if (entry === 'C') {
  data.versions = versionsData[entry].reduce((list, version) => {
    if (version === '') {
      list.push(null)
    } else {
      list.push(versions[version])
    }
    return list
  }, [])
```

So `""` becomes `null`, any other string is looked up in `browserVersions`, and array order is preserved.
A code absent from the table would push `undefined`, which `normalize` then drops silently.
A Rust decoder should treat an unknown code as an error instead
(no code in the measured data is unknown).
Entries `A`, `D` and `F` are re-keyed by the decoded version string (`unpackBrowserVersions`, lines 7-12;
the `F` branch is lines 32-39); that matters only for features these queries do not use.

### node-releases fields

`browserslist/index.js:2`: ``var jsReleases = require('node-releases/data/processed/envs.json')``.
Only `version` is read (`browserslist/index.js:1272-1274`).
Each record has the keys `name`, `version`, `date`, `lts`, `security`, `v8`:

```jsonc
// node-releases/data/processed/envs.json:2-9
{
  "name": "nodejs",
  "version": "0.2.0",
  "date": "2011-08-26",
  "lts": false,
  "security": false,
  "v8": "2.3.8.0"
}
```

`lts` is `false` or a codename string (the `26.13.0` record has `"lts": "Lithium"`), so do not type it as a boolean
if the file is parsed with a typed schema.
The file is a plain JSON array of objects (383 records, 3066 lines in the repository install).

### Data these queries never touch

- Region usage data (`caniuse-lite/data/regions/*.js`): loaded only by `env.loadCountry`, called from `cover`
  and `popularity_in_place` queries.
- Feature data (`caniuse-lite/data/features/*.js`, `features.js`): loaded only by `env.loadFeature`, called from the
  `supports` query.
- Global usage statistics: filled at load time from `A` (`browserslist/index.js:1256`), read only by popularity and
  coverage queries.
- `electron-to-chromium/versions`, `baseline-browser-mapping`,
  `node-releases/data/release-schedule/release-schedule.json`:
  required at module initialization, used only by the electron, `baseline` and `maintained node versions` queries.
- The `feature` and `region` unpackers are required by `browserslist/node.js:1-2`
  (the `require.cache` listing shows `feature.js` pulling in `statuses.js`, `supported.js` and `versionGroups.js`)
  and are never called.
- `release_date` (`F`): read only by `last N years` and `since` queries.

### Constants that live in browserslist source, not in data

- `FIREFOX_ESR_VERSION = '153'` (`browserslist/index.js:15`).
- The `dead` list of six strings (`browserslist/index.js:1219-1226`).
- `ANDROID_EVERGREEN_FIRST = '37'` (`browserslist/index.js:13`), used only for the Android jump when N exceeds 1.
- `OP_MOB_BLINK_FIRST = 14` (`browserslist/index.js:14`), not reachable from these queries.

## Options and environment that are inert here

With explicit queries, `path: false` and `stats: {}`:

- `getStat` (`browserslist/node.js:297-321`) takes `opts.stats` first (`if (opts.stats) { stats = opts.stats }`).
  An empty object is truthy, so `BROWSERSLIST_STATS` and the parent-directory search for `browserslist-stats.json`
  are skipped. The result only feeds `context.customUsage`, which none of the seven queries read.
- `loadConfig` (`browserslist/node.js:323-334`), the only reader of `BROWSERSLIST`, `BROWSERSLIST_CONFIG`,
  `BROWSERSLIST_ENV`, `NODE_ENV`, `package.json` `browserslist` keys and `browserslist` files, runs only when
  `queries` is `undefined` or `null` (`browserslist/index.js:392-393`).
- `BROWSERSLIST_DISABLE_CACHE` only controls the in-process cache (`browserslist/index.js:470`).
- `BROWSERSLIST_IGNORE_OLD_DATA` and `BROWSERSLIST_TRACE_WARNING` affect only the stderr warning in
  `oldDataWarning` (`browserslist/node.js:478-502`), which prints
  ``Browserslist: browsers data (caniuse-lite) is N months old`` when the newest release date is six or more months
  old. It never changes the result and it did not fire in any verification run (stderr was empty each time).
- `BROWSERSLIST_ROOT_PATH` is read only by `pathInRoot`, called while searching parent directories.
- `BROWSERSLIST_DANGEROUS_EXTEND` is read only when an `extends` query runs.

Verified: a container run with `BROWSERSLIST='ie 6'`, `BROWSERSLIST_ENV=nope`, `NODE_ENV=nope`,
`BROWSERSLIST_STATS=/nonexistent.json`, `BROWSERSLIST_CONFIG=/nonexistent`, `BROWSERSLIST_ROOT_PATH=/nonexistent`,
`BROWSERSLIST_DISABLE_CACHE=1` and `BROWSERSLIST_DANGEROUS_EXTEND=1` returned the identical 19-element array.

## What an implementation must reject as not implemented

The user's rule throws "not implemented" on everything outside the seven forms.
This list is what the current call path does not exercise
(each item is a browserslist feature with behavior the Rust code must not silently approximate):

- Any query whose text, after the lenient normalization of "Case and whitespace of the seven forms",
  is not one of the seven forms.
  That includes all other `QUERIES` entries (`browserslist/index.js:689-1243`): other `last` forms
  (`last N versions`, `last N electron ...`, `last N node versions`, `last N years`,
  `last N <browser> major versions`),
  `unreleased ...`, `since ...`, `baseline ...`,
  popularity (`> 0.5%`, `in my stats`, `in <config> stats`, `in <region>`),
  `cover ...`, `supports ...`, ranges and rays (`electron`, `node`, browser forms), single-version forms
  (`node 18`, `chrome 100`, `phantomjs`), `current node`, `maintained node versions`, `defaults`,
  `browserslist config`, `extends <config>`, and `dead` without a `not` prefix
  (it is needed only as the operand of `not`).
- Composition inside a line: comma, `or`, `and`, and `not` anywhere except as the prefix of `dead`.
  The parser supports them (`browserslist/parse.js:79-148`) and verified results exist
  (`a, b` and `a or b` give the union; `a and b` gives the intersection, which is `[]` for Chrome and Android;
  `a and not b` subtracts), but the seven lines do not use them.
  The config reader turns commas into line breaks before parsing (`browserslist/node.js:382`),
  so a comma in `.browserslistrc` is a line separator; a line ending in `or` or `and` joins the line that follows it.
- Config sections (`[production]`, `[defaults]`, `[a b]`; `browserslist/node.js:8`, `390-399`),
  including the `production` preference at `file-enforcer.config.ts:662`, and duplicate-section errors.
- Comments are handled by `parseConfig` (`browserslist/node.js:381`) and the repository file has none;
  accepting them is a choice, rejecting them is conforming to the user's rule.
- `extends` (`browserslist/node.js:259-282`, `checkExtend` lines 24-46), `browserslist config`.
- The `browserslist` key in `package.json` (`parsePackage`, `browserslist/node.js:157-181`) and the `browserslist` file
  name (`findConfigFile`, lines 418-461).
- `BROWSERSLIST*` environment variables, `NODE_ENV`, and the `env`, `config`, `path`, `throwOnMissing` options.
- `stats` other than the empty object, `my stats`, `browserslist-stats.json`.
- The options `mobileToDesktop`, `ignoreUnknownVersions` and `dangerousExtend`
  (`browserslist/index.js:433-437`).
- Browsers outside the ten that the supported queries and `dead` name,
  and counts other than the ones the user chooses to support (see "Open decisions").

### Browserslist's own errors within the supported forms

These are real browserslist errors the Rust implementation can reproduce or deliberately map to "not implemented".
All were reproduced with the real package.

- ``Unknown browser Foo`` for `last 1 Foo versions` and `last 1 Foo major versions`
  (`checkName`, `browserslist/index.js:301-305`). Prototype-member names (`__proto__`, `constructor`, `toString`) give
  the same message, not a `TypeError`.
- ``Write any browsers query (for instance, `defaults`) before `not dead` `` when `not dead` is the first query node,
  including a first line, or the only line (`browserslist/index.js:348-354`).
- ``Specify versions in Browserslist query for browser Chrome`` for the lone word `Chrome`
  (`browserslist/index.js:1234-1237`).
- ``Unknown browser query `foo`. Maybe you are using old Browserslist or made typo in query.`` for unrecognized text
  (`browserslist/index.js:307-314`, `1239`), including a whitespace-only entry such as `' '`, which reports an empty query text.
- ``Browser queries must be an array or string. Got number.`` for a wrong argument type
  (`browserslist/index.js:404-410`).
- The `browser_version` and `node_*` single-version forms throw ``Unknown version X of Y`` for absent versions
  (`browserslist/index.js:1181-1187`; the Node form is `browserslist/index.js:603-610`); only `samsung 4` inside `dead`
  reaches this code with the supported queries.

## Verification

### Method

All runs used the real npm package inside a container:
`podman run --rm --memory=2g --cpus=2 --network=none --security-opt label=disable`,
image `docker.io/library/node:26.12.0-trixie-slim`,
the repository's `node_modules/.pnpm` mounted read-only (so the relative pnpm links resolve),
the gate-probe `node_modules` mounted read-only at `/p2/node_modules`,
the probe scripts mounted read-only, and `.browserslistrc` mounted read-only.
Each container ran one short Node script and exited (`--rm`); no long-lived process or `systemd-run` unit was started,
and no synthetic load was generated.
The scripts live in `${HOME}/temp/agent/browserslist-spec/` (`probe.cjs`, `probe2.cjs`, `probe3.cjs`, `probe4.cjs`,
`probe5.cjs`, `probe6.cjs`, `grammar.cjs`, `grammar2.cjs`, `ws.cjs`, `ws.rs`, `run.sh`), with outputs in `out/`.
Scratch is wiped by reboots; the results that matter are recorded in this report.

Minimal reproduction of the main result:

```js
// ${HOME}/temp/agent/browserslist-spec/probe.cjs (core)
const bl = require(root + '/browserslist') // root = the node_modules directory holding browserslist
const fs = require('fs')
const config = bl.parseConfig(fs.readFileSync('/repo/.browserslistrc'))
console.log(JSON.stringify(bl(config.defaults, { path: false, stats: {} })))
```

### The seven lines together and each alone

Repository install (caniuse-lite 1.0.30001820, node-releases 2.0.61); identical to the current
`.browserslistrc.resolved.local.json`:

```jsonc
// out/probe-pnpm.json, "all"
["and_chr 156", "and_ff 159", "android 156", "chrome 156", "firefox 153",
 "node 26.13.0", "node 26.12.0", "node 26.11.0", "node 26.10.0", "node 26.9.0", "node 26.8.0", "node 26.7.0",
 "node 26.6.0", "node 26.5.0", "node 26.4.0", "node 26.3.0", "node 26.2.0", "node 26.1.0", "node 26.0.0"]
```

Gate-probe install (caniuse-lite 1.0.30001819, node-releases 2.0.60):

```jsonc
// out/probe-p2.json, "all"
["and_chr 156", "and_ff 158", "android 156", "chrome 156", "firefox 153",
 "node 26.12.0", "node 26.11.0", "node 26.10.0", "node 26.9.0", "node 26.8.0", "node 26.7.0", "node 26.6.0",
 "node 26.5.0", "node 26.4.0", "node 26.3.0", "node 26.2.0", "node 26.1.0", "node 26.0.0"]
```

Each line alone, repository install (the gate-probe install differs only in `and_ff 158` and the node list ending at
`26.12.0`):

- `last 1 Chrome versions` gives `["chrome 156"]`.
- `last 1 ChromeAndroid versions` gives `["and_chr 156"]`.
- `last 1 Android versions` gives `["android 156"]`.
- `Firefox ESR` gives `["firefox 153"]`.
- `last 1 FirefoxAndroid versions` gives `["and_ff 159"]`.
- `last 1 node major versions` gives the 14 strings `node 26.13.0` down to `node 26.0.0`.
- `not dead` alone throws
  ``BrowserslistError: Write any browsers query (for instance, `defaults`) before `not dead` ``.

Sort order check: feeding the same lines in reverse file order produces the identical sorted array (verified).
Listing the same node query twice yields no duplicates (verified).
Running the seven lines with `mobileToDesktop: true`, and inside the environment-variable container, returned the same
array. A two-line query (`last 1 Chrome versions`, `Firefox ESR`) passed with no options at all returned the expected
`["chrome 156","firefox 153"]`.
The sort is stable (`Array.prototype.sort` with an always-zero comparator preserved the order of 2000 elements).

### not dead removes nothing from the current selection

- The set `dead` (the 19 strings listed in the `dead` section, sorted):
  `baidu 13.52`, `bb 10`, `bb 7`, `ie 11`, `ie 10`, `ie 9`, `ie 8`, `ie 7`, `ie 6`, `ie 5.5`, `ie_mob 11`, `ie_mob 10`,
  `op_mob 12.1`, `op_mob 12`, `op_mob 11.5`, `op_mob 11.1`, `op_mob 11`, `op_mob 10`, `samsung 4`.
- The six positive queries without `not dead` give the same 19 strings as the seven lines together
  (array equality verified for both installs).
- The intersection of the six-query selection with the `dead` set is empty (verified for both installs).

So `not dead` is a no-op for the current data, and its subtraction path is exercised by no result in this repository.
A conforming implementation must still implement it
(the subtraction would start to matter if any selected browser entered the `dead` set),
and it needs a synthetic unit test of the subtraction itself.

### Positive controls

The control must fail when `not dead` is skipped,
and it distinguishes "subtracts" from "does nothing":

- `["last 1 IE versions"]` returns `["ie 11"]`.
  `["last 1 IE versions", "not dead"]` returns `[]`.
  An implementation that ignores `not dead` returns `["ie 11"]` and fails this control.
- A synthetic accumulator vector, produced by the real package from literal single-version queries
  (`chrome 156`, `ie 11`, `ie 5.5`, `op_mob 12.1`, `op_mob 80`, `samsung 4`, `samsung 30`, `baidu 13.52`, `bb 10`,
  `bb 7`, `ie_mob 11`, `ie_mob 10`, `op_mob 10`, `ie 10`):

```jsonc
// out/probe2-pnpm.json, "synthetic before" then "synthetic after not dead"
// before (sorted, after uniq):
["baidu 13.52","bb 10","bb 7","chrome 156","ie 11","ie 10","ie 5.5","ie_mob 11","ie_mob 10",
 "op_mob 80","op_mob 12.1","op_mob 10","samsung 30","samsung 4"]
// after appending the query "not dead":
["chrome 156","op_mob 80","samsung 30"]
```

  This is the unit-level test vector for "dead expansion plus set subtraction":
  inputs are the strings before `not dead`,
  the output keeps exactly the non-dead strings,
  and the check that `op_mob 80` and `samsung 30` survive guards against over-removing.
  The sort puts `op_mob 80` before `op_mob 12.1` because versions descend numerically (80 is greater than 12.1 as a
  number; as strings `12.1` would sort first).
- The guard that proves the harness can see a difference:
  any test of the subtraction must fail when `not dead` is removed from the vector,
  because `["chrome 156","op_mob 80","samsung 30"]` differs from the 14-string vector.

### Additional verified behaviors

Config text (`parseConfig`):

- `'last 1 Chrome versions # c\r\n\r\nFirefox ESR, not dead\r\n'` gives
  `defaults: ['last 1 Chrome versions', 'Firefox ESR', 'not dead']`.
- `'last 1 Chrome versions\n[production]\nFirefox ESR\n[dev test]\nnot dead\n'` gives
  `defaults: ['last 1 Chrome versions']`,
  `production: ['Firefox ESR']`, `dev: ['not dead']`, `test: ['not dead']`.
- `''` and a comment-only text give `{ defaults: [] }`.

Composition and edge cases, through the full call:

- `last 1 Chrome versions or` then `last 1 Android versions` as two config lines gives `["android 156","chrome 156"]`
  (joined as a union); with `and` instead of `or` it gives `[]`.
- `last 1 Chrome versions # why` then `Firefox ESR` gives `["chrome 156","firefox 153"]`.
- `last 1 Chrome versions`, `last 1 ChromeAndroid versions`, `not last 1 Chrome versions` gives `["and_chr 156"]`.
- A leading empty array element is skipped (`['', 'last 1 Chrome versions']` gives `["chrome 156"]`).

## Open decisions for the user

These need a user choice; the report does not make them.

- Count policy.
  Accept any `\d+` count with the semantics recorded in this report (including the whole-list results for a count of
  0 and for counts reaching major 0),
  or throw "not implemented" for any count other than 1.
  The second choice removes every quirk (`!minimum`, `slice(-0)`, Android `jump`) at no cost for the current file.
- The Firefox ESR constant.
  It is only in `browserslist/index.js:15`.
  The options are: read it from the installed package source at generation time,
  pin it in Rust with a drift test against the package,
  or fail when the installed browserslist version differs from 4.29.5.
- The data pin.
  The repository install and the gate-probe install disagree on `and_ff` and the node list,
  so the fixture must name one revision.
  The repository lockfile pins caniuse-lite 1.0.30001820 and node-releases 2.0.61.
- How `dead` is implemented.
  Either hard-code the six expansions' rules (all released Baidu; IE and IE Mobile released versions up to 11;
  BlackBerry up to 10; Opera Mobile released versions at most 12.1; Samsung 4),
  or implement the two internal single-version and ray forms.
  The first needs less code; both must reproduce `parseFloat` comparisons exactly.
- Which spellings of the seven forms to accept:
  the full lenient normalization of this report (ASCII case-insensitive, JavaScript whitespace, optional plural,
  leading zeros), or only the exact text of the current file.
- Whether `parseConfig` features (comments, commas as separators, CRLF) are replicated or rejected.
  The repository file uses none of them.

`doc/research/` has no `README.md` index, so there is nothing to register this report in.
