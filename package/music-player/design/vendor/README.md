# Vendored builds

The three files here are the browser builds the design pages load. They are committed so a page opened from a
checkout renders with no network, and so the page reads the same bytes every time.

- `react-18.3.1.production.min.js`: React's `umd/react.production.min.js` from the `react` package at 18.3.1.
- `react-dom-18.3.1.production.min.js`: ReactDOM's `umd/react-dom.production.min.js` from the `react-dom` package at
  18.3.1.
- `babel-standalone-7.29.0.min.js`: `babel.min.js` from the `@babel/standalone` package at 7.29.0. The pages
  themselves define their logic in plain JavaScript; Babel is here because the Design Component runtime compiles an
  `x-import` of JSX on demand, and that path must not reach a network either.

Both packages are MIT licensed, and each licence text travels with its build: `react.LICENSE.txt` covers React and
ReactDOM, `babel.LICENSE.txt` covers Babel.

`builds.json` records, per file, the package, version, in-package path, licence, licence file and SHA-384 digest. The
digests are the same values the pages pinned through subresource integrity before the builds were vendored, so a file
here is byte-identical to the one every page used to fetch.

## The guards that read this directory

`mise run //package/music-player/design:test:offline-pages` holds every page and this record to the same tree: a
recorded build whose digest no longer matches, a file with no record, a missing licence file, and a runtime that asks
a network instead of this directory are each reported.

`mise run //package/music-player/design:test:offline-pages:render` renders every page in a headless browser and fails
when any request leaves the loopback server it serves the tree on.

## Changing a build

1. Replace the file, keeping the `<package>-<version>` naming, and update its `builds.json` entry.
2. Compute the new digest with `openssl dgst -sha384 -binary <file> | openssl base64 -A` and prefix `sha384-`.
3. Run both guard tasks above.

The runtime resolves these files relative to its own script URL, through the `VENDOR_BASE` line at the top of its
`src/cdn.ts` section. One runtime copy sits beside this directory and reaches it with `./vendor/`; the copy under
`candidates/` reaches it with `../vendor/`. Those two lines are the only difference between the copies, and a guard
compares them with that line normalized.
