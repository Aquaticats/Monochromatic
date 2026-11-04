# monochromatic-catalog-specifier

EXTREMELY UNSTABLE AND UNFINISHED DO NOT USE

Pure checker for the repository's catalog specifier rule:
every dependency specifier in a package manifest is `catalog:`, `catalog:<name>` or exactly `workspace:*`.

The crate reads no files and starts no processes.
A caller hands it a manifest path and the manifest's bytes and gets findings back,
so the `cli-git` commit policy `mono/catalog-specifiers`, the `pnpm-shadow` wrapper and any CI step
share one reading of the rule.

## Why

`pnpm-workspace.yaml` sets `catalogMode: strict`, yet pnpm consults that mode only for `pnpm add` and `pnpm update`.
`pnpm install` accepts a literal version such as `"zod": "^4.6.7"` without a word.
Upstream closed the request for install-time enforcement without implementing it
(pnpm issue 8644), so the repository checks the rule itself.
The record of the decisions is `doc/handover/catalog-specifier-policy.md`.

## Contract

This section is the normative byte-to-findings contract.
The vectors in `fixtures/vectors.json` are its executable form,
and a consumer must not re-implement any step of it.

### Input

- `path`: the manifest's repository-relative path with forward slashes.
  It is only used to name the file in findings and to detect unsupported manifest formats.
- `bytes`: the manifest file's bytes, from whichever snapshot the caller means to check
  (the staged blob for a commit policy, the working-tree file for an advisory scan).

### Unsupported manifest formats

A path whose last segment is exactly `package.json5`, `package.yaml` or `package.yml` yields one
`unsupported-manifest-format` finding, and its bytes are not read.
pnpm reads those formats too, and this crate has no parser for them,
so refusing the file name is the only way to keep a literal version from hiding there.
The comparison is case sensitive.
A name that merely contains one of those names (`my-package.yaml`, `package.yaml.bak`) is not unsupported.

### Reading the bytes

1. One leading UTF-8 byte order mark is removed, because pnpm removes it as well.
2. Bytes that are not valid UTF-8, empty input, text that `monochromatic-jsonc-edit` cannot parse
   and a root that is not an object each yield one `invalid-manifest` finding and nothing else.
3. The parser is `parse_jsonc` from `monochromatic-jsonc-edit`.
   It accepts `//` and `/* */` comments and trailing commas, which pnpm itself may reject,
   so a manifest this crate accepts can still fail in pnpm.
   JSON5-only forms are rejected.
4. Object members are kept in source order and duplicate keys stay visible.
   No consumer may parse the bytes into a map first, because a map drops duplicates.

### Sections

Four root members are checked: `dependencies`, `devDependencies`, `optionalDependencies` and `peerDependencies`.
Every other root member, including `resolutions`, `overrides`, `peerDependenciesMeta` and `pnpm`, is ignored.

- A section that is absent passes.
- A section whose value is not an object (`null`, an array, a string, a number or a boolean) yields one
  `malformed-section` finding naming the section; its contents are not read.
- An empty object passes.
- A root member named like a checked section that appears more than once yields one `duplicate-key` finding for every
  occurrence after the first, and every occurrence is still checked, so a literal in either copy is found.
- Inside a section, a dependency name that appears more than once yields one `duplicate-key` finding for every
  occurrence after the first, and every occurrence is still checked.
  pnpm keeps the last duplicate, so the first one may be the version that someone meant.

### Specifiers

A dependency value that is not a JSON string yields one `non-string-specifier` finding.
That includes arrays, objects, numbers, booleans and `null`; an array holding `"catalog:team"` is not a catalog reference.

For a string, the check is exact and case sensitive, with no trimming:

- `dependencies`, `devDependencies` and `optionalDependencies` accept `catalog:`, `catalog:<name>` and `workspace:*`.
- `peerDependencies` accept `*`, `catalog:` and `catalog:<name>`.
- `<name>` is one or more of the ASCII letters, digits, `.`, `_` and `-`.
  This is deliberately narrower than pnpm, which takes any text after `catalog:` and trims whitespace,
  so `catalog: `, `catalog:team `, `catalog:team:other` and `catalog:é` are findings here.
  The repository has no named catalogs today; widen the grammar here, with a new vector, if one needs more.
- Anything else yields one `disallowed-specifier` finding: digits, ranges, prereleases, tags such as `latest`,
  `workspace:^`, `workspace:~`, `workspace:*.0`, `link:`, `file:`, git and URL forms, `npm:` aliases and the empty string.
- A specifier containing an unpaired UTF-16 surrogate escape is a `disallowed-specifier`.

The check is about syntax only.
It does not look up a catalog entry, it does not read `pnpm-workspace.yaml`,
and it does not validate what a catalog entry itself says
(`pnpm add --workspace` can write `workspace:^1.0.0` into the catalog, which this check cannot see).

### Findings

Each finding carries a kind, the manifest `path`, and, where they apply, the section, the dependency name and the specifier.
Findings come in source order: section by section as the members appear, then entry by entry.
The `Display` text names the file, the section and the dependency, and states the remedy:
move the version into the `catalog:` section of `pnpm-workspace.yaml` and write `catalog:`,
or write `workspace:*` for a package of this workspace.
A dependency name that holds an unpaired surrogate is shown with the replacement character.

## Not covered

- Catalog indirection, `overrides`, package extensions and `.pnpmfile.mjs` rewrites.
- A tracked manifest outside the pnpm workspace gets no pnpm validation, only this check.
- Published manifests, which pnpm rewrites and which legitimately carry literal versions.
- The result is a preflight on the bytes given, not a postcondition: a script can write a literal afterwards.

## Checks

```bash
mise run //package/catalog-specifier/core-rust:lint
mise run //package/catalog-specifier/core-rust:lint:clippy
mise run //package/catalog-specifier/core-rust:test
```
