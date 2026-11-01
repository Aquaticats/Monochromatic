# npm 11.20.0 selects a parent node_modules directory when a scratch consumer lacks a manifest

## Symptom

On 2026-10-16,
`npm install @monochromatic-dev/fs-path-core@0.2.0` ran from a new scratch child directory
without `package.json`.
npm printed `added 1 package, and removed 120 packages`.
It installed into the parent scratch directory,
created parent `package.json` and `package-lock.json`,
and removed the parent's extraneous Markdown tooling dependencies.
The repository's dependency tree was not affected.

The first import-only check resolved the parent's installation and passed,
so it did not establish that the fixture was isolated.
A later explicit read of the fixture's own installed manifest failed with `ENOENT`.

## Root cause

Installed npm 11.20.0 contains the deciding code in
`node_modules/@npmcli/config/lib/index.js:696`,
inside `loadLocalPrefix()`.
It honors an explicit CLI prefix first,
then walks ancestors from the working directory.
Lines 713 to 717 select the first directory holding either a manifest or `node_modules`:

```js
// npm/node_modules/@npmcli/config/lib/index.js
const hasPackageJson = await fileExists(p, 'package.json')
if (!this.localPrefix && (hasPackageJson || await dirExists(p, 'node_modules'))) {
  this.localPrefix = p
  this.localPackage = hasPackageJson
}
```

The npm debug log independently recorded
`no package.json at prefix`
and the parent scratch path.
Changing only the shell working directory did not constrain npm's installation root.

## Verification

- Failing case: child without a manifest beneath an existing `node_modules` directory.
  npm selected the parent,
  as recorded in `2026-10-16T01_46_29_297Z-debug-0.log`.
- Working case: child with a private `package.json` before installation.
  npm installed locally,
  and the verification read `node_modules/@monochromatic-dev/fs-path-core/package.json`
  before exercising root discovery and the Node helpers.
- Recovery: npm's removal log identified 120 package paths.
  Cached archives were installed into a separate staging project with `--prefix`,
  `--offline`,
  and `--ignore-scripts`.
  All removed package paths were restored without overwriting surviving packages,
  and `@mdx-js/mdx` compiled a Markdown document successfully.

Where multiple cached versions existed,
recovery selected the August 2026 installation cohort.
The removed tree had no retained manifest or lockfile;
the recovery proves restored package paths and working compilation,
not byte-identical versions of the former tree.
The accidentally created parent manifests and fs-path installation were removed.

## Verified workarounds

Create a private manifest before installing into a consumer fixture.
Also pass an explicit `--prefix` when the installation root is independently specified.
These preserve isolation but require establishing the fixture before installation.
Verify the installed manifest at the intended path;
successful imports alone can resolve an ancestor's package.

## What does not work

- Setting the working directory without creating a manifest did not isolate installation.
- Installing a raw npm cache content file without a `.tgz` filename failed with `ENOTDIR`.
  Recovery copied cached archives to named `.tgz` files before the offline install.

## Upstream filing decision

- Fault: the fixture omitted its installation boundary.
- Upstream fixability: no npm defect was established.
- Supported usage: explicit prefix and local project manifests are handled by the installed source.
- Contribution acceptance: no upstream change is proposed.
- Upstream direction: ancestor discovery is explicit in the installed implementation.
- Prototype: the corrected consumer and staged recovery both ran successfully.

No upstream issue or comment is warranted.
Nothing additive is proposed for upstream.
