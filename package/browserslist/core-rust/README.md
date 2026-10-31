# monochromatic-browserslist

Resolves this repository's `.browserslistrc` to the same target list npm `browserslist` 4.29.5 produces,
from `caniuse-lite` and `node-releases` data pinned by hash in a committed lock.
It supports only the seven query lines the repository uses.
Every other query form or config feature fails with an error whose message starts with `not implemented`.

## Why this crate exists

The user decided on 2026-10-30:
"for browserslist,
we retrieve the data and write something conforming to the original browserslist ourselves,
only supporting the queries we use now,
throw on everything non-conforming with \"not implemented\"."
The decision record is `doc/decision/file-enforcer-rust-rewrite.md` (bullet "Browserslist").
The semantics being matched are specified,
with source citations,
in `doc/research/browserslist-subset-semantics.md`.

## Supported input

The `.browserslistrc` text is accepted when every line,
after removing `#` comments and trimming spaces and tabs,
is blank or exactly one of these:

- `last 1 Chrome versions`
- `last 1 ChromeAndroid versions`
- `last 1 Android versions`
- `Firefox ESR`
- `last 1 FirefoxAndroid versions`
- `last 1 node major versions`
- `not dead`

Anything else is refused with a `not implemented` error that names the query and the line.
That includes other spellings of the seven lines (case,
spacing,
singular `version`,
leading zeros),
every count other than 1,
`or` and `and`,
commas as separators,
CR line endings,
`[section]` headers,
`extends`,
and every other browserslist query family.
Only the count 1 is supported,
so browserslist's `last 0` and large-count quirks are unreachable.

`not dead` evaluates browserslist's own `dead` expansion (`Baidu >= 0`,
`ie <= 11`,
`ie_mob <= 11`,
`bb <= 10`,
`op_mob <= 12.1`,
`samsung 4`) against the locked data and subtracts it from everything selected before it.
As in browserslist,
it is an error as the first query:
``Write any browsers query (for instance, `defaults`) before `not dead` ``.

The output is sorted and deduplicated exactly as browserslist does it (names ascending,
versions descending).

## Library API

The file-enforcer engine calls these (all re-exported at the crate root):

```text
// package/browserslist/core-rust/src/lib.rs
pub fn resolve_browserslistrc(config_text: &str, lock: &DataLock, source: &DataSource)
    -> Result<Vec<String>, BrowserslistError>;
pub fn render_browserslistrc(config_text: &str, lock: &DataLock, source: &DataSource)
    -> Result<String, BrowserslistError>;
pub fn render_resolved_json(targets: &[String]) -> String;
impl DataLock { pub fn from_json(text: &str) -> Result<DataLock, BrowserslistError>; }
impl DataSource { pub fn user_default() -> Result<DataSource, BrowserslistError>; }
```

`render_browserslistrc` returns the exact content of `.browserslistrc.resolved.local.json`:
`JSON.stringify(targets, null, 2)` plus a newline,
with `[]` for an empty result.
Config and query errors are reported before any cache read or download.
`DataSource::user_default()` uses the platform user cache directory (through the `directories` crate)
and `https://registry.npmjs.org`;
tests and tools build a `DataSource` with other values.

Calls are synchronous.
The async HTTP stack runs on a private thread and runtime,
and only when a pinned tarball is missing from the cache,
so the call works inside or outside an async runtime.

## Data lock and cache

`browserslist-data.lock.json` pins `caniuse-lite` and `node-releases` by version and npm `dist.integrity` (sha-512),
and the Firefox ESR value from Mozilla product-details (for example `153.6.0esr`).
The library only reads the lock;
it never picks versions.

The cache holds the verified tarballs,
named by the hex SHA-512 of the pinned integrity,
under the user cache directory.
Every read re-verifies the bytes.
A cache miss,
or an entry that fails verification,
fetches exactly the pinned tarball from
`<registry>/<name>/-/<name>-<version>.tgz`,
verifies it,
and writes it atomically.
Redirects are not followed,
and plain `http` is accepted only for loopback hosts (the tests' fake registry).

## Updating the lock

`mise run //package/browserslist/core-rust:update` runs the binary's `update` command.
It asks the registry for the newest published `caniuse-lite` and `node-releases`,
reads `FIREFOX_ESR` from `https://product-details.mozilla.org/1.0/firefox_versions.json`
(a value not shaped `<major>.<minor>.<patch>esr` fails with a diagnostic),
downloads and verifies the tarballs,
proves they decode,
and rewrites the lock.
Then run `mise run //package/browserslist/core-rust:conformance:regenerate` and review the lock and fixture diff.

The binary `monochromatic-browserslist` has three commands:

- `update` rewrites the lock.
- `export --out DIR` writes the verified pinned tarballs,
  which the conformance harness uses.
- `resolve --config PATH` prints the rendered resolved JSON for a config file.

Each takes `--lock PATH` (default `browserslist-data.lock.json`),
`--cache-dir DIR` and `--registry URL`.

## Data files read

The parser for the packed `caniuse-lite` files (`data/agents.js`,
`data/browsers.js`,
`data/browserVersions.js`)
is owned by this crate and accepts exactly the JavaScript literal subset those files use,
in one linear pass with a bounded nesting depth.
Anything outside the subset is an error naming the file and the byte offset.
From `node-releases`,
only the `version` field of `data/processed/envs.json` is read,
in file order.

## Tests

- `mise run //package/browserslist/core-rust:test` runs everything offline with nextest.
  Network behavior is tested against a fake registry on a loopback port.
- `tests/conformance.rs` compares the resolver with the answers npm `browserslist` 4.29.5 gave in a container,
  over the seven lines together and each alone,
  orderings,
  comments,
  Node.js version variants,
  and representative
  refusals.
  The Firefox ESR entry is compared against the locked Mozilla value,
  not npm's constant.
- `mise run //package/browserslist/core-rust:test:live` runs the opt-in live test against the real registry and Mozilla.
- `mise run //package/browserslist/core-rust:conformance:regenerate` rebuilds `fixtures/data` and
  `conformance/expected.json` from the locked data with a bounded `podman` container (`--memory=2g --cpus=2`).

## Dependencies

`serde_json`,
`sha2`,
`base64`,
`directories`,
`tracing`,
`flate2`,
`tar`,
and the approved HTTP stack
(`hyper`,
`hyper-util`,
`http-body-util`,
`hyper-rustls`,
`tokio`).
The decision record approves `flate2` and `tar` (own registry fetch) and the HTTP stack.

## License

LGPL-3.0-or-later.
