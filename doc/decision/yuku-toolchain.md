# Yuku toolchain adoption

Adopted 2026-07-22 on the maintainer's direct instruction
("Do #2,
 #3,
 #4",
"I approve using everything under yuku-*.
Own as little code ourselves as possible"),
which waived the full `choosing-technology` vetting workflow;
the maintainer overruled the pre-1.0 maturity concern because this
repository is itself pre-1.0,
and overruled the trust-boundary auditability concern because a single
parser family across the repo beats per-package parser diversity.

## Adopted components

- `yuku-parser` (npm,
   native bindings):
   ESTree/TS-ESTree parsing with
  UTF-16 span semantics,
   probe-verified on astral characters.
- `yuku-ast`:
   structural walk (`walk`,
   `WalkContext`),
   node guards
  (`is`),
   used wherever repository code traverses parse output.
- `yuku-analyzer`:
   cross-file semantic model powering
  `@monochromatic-dev/mutation-test-cli-unused-export`.
- Catalog floors sit at `>=0.7.3` because 0.7.4 was younger than pnpm's
  `minimumReleaseAge` at adoption time;
  the range adopts newer versions as they age past the cutoff.
  See `doc/troubleshooting/pnpm-minimum-release-age-trust-lockfile.md`
  for the mechanism.

## Displaced incumbents

- `oxc-parser` left `package/mutation-test/cli` together with the
  package's hand-rolled walker;
  yuku-ast's walk added type-erased-subtree skipping,
  so mutants inside erased TypeScript positions no longer waste
  per-mutant compile checks.
- `acorn` left `package/git-policy/cli`'s MJS trust validator and its
  packed-dependency fixtures;
  the validator now walks typed visitors instead of two hand-rolled
  recursive scans and keeps rejecting non-literal dynamic imports.
- Consumer-less `oxc-minify` and `oxc-resolver` catalog entries were
  pruned in the same change.

## Integration boundary

- Repository code imports `yuku-parser`,
  `yuku-ast`,
  and `yuku-analyzer` through the pnpm catalog only.
- The readonly-parameter lint rule's audited-call catalogue pins the
  shipped `yuku-ast` 0.7.3 bundle by sha256
  (`package/oxlint-plugin/prefer-readonly-parameter-type/src/prefer-readonly-parameter-types/yuku-ast-package-effect-catalog.ts`);
  version bumps fail the evidence validation loudly and force a
  re-audit,
  the same contract the dot-prop and postcss audits use.
- `rolldown-plugin-dts` already depended on yuku before this adoption,
  so the build chain exercised the parser on every build regardless.

## Rollback

Reverse the three adoption commits
(mutation-test swap,
 git-policy swap,
 cli-unused-export addition),
restore the oxc/acorn catalog entries,
and drop the yuku audited-call catalogue;
no other component depends on yuku directly.

## Revisit triggers

- A yuku minor release removes the deprecated `yuku-parser` walk
  re-export;
   repository code already imports walk from `yuku-ast`,
  so only the audit catalogue digest needs re-pinning.
- Yuku 1.0 or a security advisory on the native bindings warrants the
  full `choosing-technology` vet that adoption skipped.
- `parse(source, { sourceType: 'script' })` accepted `export` syntax
  without diagnostics during probing;
  if script-mode validation ever matters to a consumer,
  investigate before relying on it.
