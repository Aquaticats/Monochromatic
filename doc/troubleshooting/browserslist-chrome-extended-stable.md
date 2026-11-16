# Browserslist 4.29.5 rejects Chrome Extended Stable channel queries

## Investigation status at 2026-11-15T19:16:52-05:00

Agent investigation,
not final or owner-approved.
Codex inspected upstream source and exercised Browserslist's JavaScript API.
No browser targets or implementation changed.
Chrome Extended Stable has no built-in selector analogous to `Firefox ESR`.
Use its current Chrome major explicitly,
or obtain that major outside Browserslist.

## Symptom

Browserslist 4.29.5 rejects `Chrome ESR`,
`Chrome extended stable`,
and `last 1 Chrome extended stable version` with:

```text
Unknown browser query `<query>`. Maybe you are using old Browserslist or made typo in query.
```

`Chromium 156` and `last 1 Chromium versions` instead produce `Unknown browser Chromium`.
Use the `Chrome` target for this engine version.

## Root cause

Source inspected:
upstream commit `8253b7071aae92f6ea755aee54bc93dee47048a1`,
whose `package.json` declares version `4.29.5`.
The [documented queries][documented-queries] list Chrome version counts and `Firefox ESR`,
without a Chrome Extended Stable selector.

In [`index.js:772` to `780`][last-browser-selector],
the Chrome version-count query selects from released versions:

```js
// browserslist/index.js:772 to 780
last_browser_versions: {
  matches: ['versions', 'browser'],
  regexp: /^last\s+(\d+)\s+(\w+)\s+versions?$/i,
  select: function (context, node) {
    var data = checkName(node.browser, context)
    var list = data.released.slice(-node.versions).map(nameMapper(data.name))
    list = filterJumps(list, data.name, node.versions, context)
    return list
  }
```

The dedicated ESR selector is restricted to Firefox in
[`index.js:1079` to `1085`][firefox-selector]:

```js
// browserslist/index.js:1079 to 1085
firefox_esr: {
  matches: [],
  regexp: /^(firefox|ff|fx)\s+esr$/i,
  select: function () {
    return ['firefox ' + FIREFOX_ESR_VERSION]
  }
}
```

These selectors explain why a Chrome version-count query does not track the Extended Stable channel.

## Verification

Tested Browserslist `4.29.5` with `caniuse-lite` `1.0.30001826` on Node.js `26.13.1`.
Run this from a project where `browserslist` resolves:

```sh
# Consumer-side API probe, without changing browser configuration.
node --input-type=module -e '
import { createRequire } from "node:module";
const load = createRequire(import.meta.url);
const browserslist = load("browserslist");
for (const query of ["Firefox ESR", "last 1 Chrome versions", "last 2 Chrome versions", "Chrome 156", "Chrome >= 156", "Chrome ESR", "Chrome extended stable", "last 1 Chrome extended stable version", "Chromium 156", "last 1 Chromium versions"]) {
  try { console.log(JSON.stringify({ query, result: browserslist(query) })); }
  catch (error) { console.log(JSON.stringify({ query, error: error.message })); }
}'
```

Accepted-query catalog:

- `Firefox ESR` returned `firefox 153`.
- `last 1 Chrome versions` returned `chrome 157`.
- `last 2 Chrome versions` returned `chrome 157` and `chrome 156`.
- `Chrome 156` returned `chrome 156`.
- `Chrome >= 156` returned `chrome 157` and `chrome 156`.

Rejected-query catalog:
the Chrome channel spellings produced the unknown-query diagnostic in `Symptom`;
the Chromium spellings produced its unknown-browser diagnostic.

The live [Windows Extended Stable endpoint][extended-windows] returned milestone `156`,
version `156.0.8078.100`.
The same request with `platform=Mac` returned the same version.
The [Windows Stable endpoint][stable-windows] returned milestone `158`,
version `158.0.8127.37`.
The installed compatibility dataset therefore lagged the live Stable channel in this probe.

## Verified workarounds

Use `Chrome 156` to select the currently observed Extended Stable major.
Use `Chrome >= 156` when the intended policy includes subsequent released versions in the installed dataset.
Both queries were exercised successfully.
The tradeoff is that the numeric floor must change when the channel advances;
neither query discovers the channel or its patch version.

For automatic channel tracking,
fetch the Extended endpoint and construct `Chrome <milestone>` externally.
The endpoint request and explicit query were verified separately;
no automatic updater was implemented or exercised end to end.

## What does not work

- Chrome ESR and Extended Stable spellings fail as recorded in `Symptom`.
- A `Chromium` target fails as recorded in `Symptom`.
- `last 2 Chrome versions` happened to include `156` in the installed dataset,
  but selects a count of releases and does not express Extended Stable membership.
- Treating the installed dataset as a live release-channel service misses the observed Stable milestone `158`.

## Upstream filing decision

Nothing was filed,
and there is no actionable bug report to add from this capability question.
The `.out-of-scope/` filenames contained no Browserslist exemption.
`gh search issues '"extended stable"' --repo browserslist/browserslist`
and the equivalent pull-request search returned no matches on the investigation date.
A wider issue search for `extended` returned Firefox ESR and unrelated configuration threads.

1.  Upstream fault:
    no violation of a documented Chrome channel selector was established.
2.  Feasibility:
    adding channel metadata could be possible,
    but was not designed or prototyped.
3.  Supported use case:
    the inspected query catalog does not document Chrome Extended Stable selection.
4.  Contribution acceptance:
    not assessed because no upstream filing is proposed.
5.  Maintainer intent:
    the tracker searches establish no decision about this feature.
6.  Prototype:
    none;
    this request asks whether the capability exists.

The automatic prototype condition does not hold because the documented-use-case condition fails.
No recommendation to file is made.

## Check limits

Source selections were checked against the clone.
Browser confirmation initially failed with agent-browser's
`Failed to read: Resource temporarily unavailable (os error 11)` diagnostic;
see [the existing diagnostic investigation](agent-browser-eagain.md).
This does not affect the successful Browserslist API or release-endpoint probes.

[documented-queries]: https://github.com/browserslist/browserslist/blob/8253b7071aae92f6ea755aee54bc93dee47048a1/README.md#L235-L257
[last-browser-selector]: https://github.com/browserslist/browserslist/blob/8253b7071aae92f6ea755aee54bc93dee47048a1/index.js#L772-L780
[firefox-selector]: https://github.com/browserslist/browserslist/blob/8253b7071aae92f6ea755aee54bc93dee47048a1/index.js#L1079-L1085
[extended-windows]: https://chromiumdash.appspot.com/fetch_releases?channel=Extended&platform=Windows&num=1
[stable-windows]: https://chromiumdash.appspot.com/fetch_releases?channel=Stable&platform=Windows&num=1
