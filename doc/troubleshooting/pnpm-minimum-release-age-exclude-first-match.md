# pnpm 11.6.0 `minimumReleaseAgeExclude` honors only the first listed version per package name, so accumulated per-release entries silently fail to exempt newer versions

> Scratch-path note:
> `/tmp/agent` paths in this document are historical.
> Use `~/temp/agent` for current work.

## Symptom

A package version that is present in `minimumReleaseAgeExclude` still fails
the supply-chain age check during install:

```text
? Verifying lockfile against supply-chain policies (773 entries)...
[ERR_PNPM_MINIMUM_RELEASE_AGE_VIOLATION] 4 lockfile entries failed verification:
  @earendil-works/pi-agent-core@0.79.4 was published at 2026-06-15T08:32:28.000Z, within the minimumReleaseAge cutoff (2026-06-14T18:58:33.089Z)
  @earendil-works/pi-ai@0.79.4 was published at 2026-06-15T08:32:24.000Z, within the minimumReleaseAge cutoff (2026-06-14T18:58:33.089Z)
  @earendil-works/pi-coding-agent@0.79.4 was published at 2026-06-15T08:32:37.000Z, within the minimumReleaseAge cutoff (2026-06-14T18:58:33.089Z)
  @earendil-works/pi-tui@0.79.4 was published at 2026-06-15T08:32:32.000Z, within the minimumReleaseAge cutoff (2026-06-14T18:58:33.089Z)
```

The trigger is a `minimumReleaseAgeExclude` list that carries more than one
`name@version` entry for the same package name,
 for example the shape that
accumulates as each new release is added over time:

```yaml
minimumReleaseAgeExclude:
  - '@earendil-works/pi-ai@0.75.4'
  - '@earendil-works/pi-ai@0.76.0'
  # ...
  - '@earendil-works/pi-ai@0.79.4'   # the version you actually need exempt
```

The oldest entry (`0.75.4`) is honored;
 the newest (`0.79.4`),
 the one within
the age cutoff and the reason you added it,
 is not.
 The same install passes
the gate once the per-version entries for that package are replaced with a
single bare-name entry (`@earendil-works/pi-ai`) or a single `||` union entry.

`pnpm config list` shows every entry present in the merged config,
 so the
entries are read;
 they are just not all applied.

## Root cause

The lockfile verifier consults the exclude list,
 but the policy it builds
collapses multiple same-name entries to the first one.
 Source is the pnpm
11.6.0 bundle at
`~/.local/share/mise/installs/npm-pnpm/11.6.0/lib/node_modules/pnpm/dist/pnpm.mjs`
(line numbers are bundle offsets;
 the original modules are named in the bundle
comments).

The age verifier skips the check when the package is excluded
(`// ../resolving/npm-resolver/lib/createNpmResolutionVerifier.js`,
`dist/pnpm.mjs:64873`):

```js
const ageApplies = ageCheckActive && !isExcluded(excludePolicy, name, version2);
```

`isExcluded` calls the policy with the bare package name and checks whether the
returned version list contains the resolved version
(`dist/pnpm.mjs:65211`):

```js
function isExcluded(policy, name, version2) {
  if (!policy)
    return false;
  const result2 = policy(name);
  if (result2 === true)
    return true;
  if (Array.isArray(result2) && result2.includes(version2))
    return true;
  return false;
}
```

The policy is built one rule per exclude entry,
 in list order
(`// ../config/version-policy/lib/index.js`,
 `dist/pnpm.mjs:64603`):

```js
function createPackageVersionPolicy(patterns) {
  const rules = [];
  for (const pattern of patterns) {
    const parsed = parseVersionPolicyRule(pattern);
    rules.push({ nameMatcher: createMatcher(parsed.packageName), exactVersions: parsed.exactVersions });
  }
  return evaluateVersionPolicy.bind(null, rules);
}
```

Each `name@version` entry parses to a rule whose `exactVersions` is a single
element (`dist/pnpm.mjs:64652`).
 Multiple versions are expressed within one
entry via a `||` union,
 which is the hint that one package is meant to be one
entry (`dist/pnpm.mjs:64672`):

```js
function parseExactVersionsUnion(versionsStr) {
  const versions = [];
  for (const versionRaw of versionsStr.split("||")) {
    const version2 = import_semver11.default.valid(versionRaw);
    if (version2 == null) {
      return null;
    }
    versions.push(version2);
  }
  return versions;
}
```

The defect is the evaluator:
 it returns on the first rule whose name matches,
never consulting later rules for the same name
(`dist/pnpm.mjs:64640`):

```js
function evaluateVersionPolicy(rules, pkgName) {
  for (const { nameMatcher, exactVersions } of rules) {
    if (!nameMatcher(pkgName)) {
      continue;
    }
    if (exactVersions.length === 0) {
      return true;
    }
    return exactVersions;   // first name match wins; later same-name rules are dead
  }
  return false;
}
```

So `policy('@earendil-works/pi-ai')` returns `['0.75.4']` (the first entry),
and `['0.75.4'].includes('0.79.4')` is false,
 so the age check applies and
`0.79.4` is rejected.
 A bare-name entry parses to `exactVersions: []`,
 which
the evaluator treats as "all versions" via the `length === 0` branch,
 which is
why the bare-name workaround exempts every version.

This interacts badly with how the list grows.
 `pnpm audit --fix` /
`pnpm audit --fix=update` append a fresh `name@version` entry for each patched
release and never merge or clean them
(`dist/pnpm.mjs:170593`,
 and pnpm/pnpm#11668):

```js
const existing = manifest.minimumReleaseAgeExclude ?? [];
// ...
manifest.minimumReleaseAgeExclude = [...existing, ...newEntries];
```

The newest appended entry is the one a developer needs exempted right after a
release,
 and it is exactly the entry the first-match evaluator never reaches.

### Earlier wrong hypothesis

During investigation the first conclusion was "pnpm 11.6.0 ignores
`minimumReleaseAgeExclude` for the verification step (a plumbing or caching
bug)".
 That was wrong.
 `pnpm config list` showed all entries present in the
merged config,
 and `isExcluded` / `createNpmResolutionVerifier` above do
consult the exclude list.
 The exclude is honored,
 but only for the first
same-name rule.
 The disproving evidence was that replacing the four packages'
per-version entries with bare-name entries made a plain `pnpm install` pass
with no environment override,
 which a config-not-reaching-the-verifier bug
could not explain.

## Verification

Version under test:
 pnpm `11.6.0` (release tag `v11.6.0`),
 resolved by mise at
`~/.local/share/mise/installs/npm-pnpm/11.6.0`.
 `minimumReleaseAge` was active
(non-zero) in the environment under test,
 and `minimumReleaseAgeStrict` makes
the violation a hard install failure.

The behavior is reproducible from the pure version-policy logic,
 with no pnpm
build,
 by replicating the three functions above:

```js
// repro.mjs (Node) -- mirrors dist/pnpm.mjs createPackageVersionPolicy/evaluateVersionPolicy/parseVersionPolicyRule
import semver from 'semver';

const parse = (pattern) => {
  const isScoped = pattern.startsWith('@');
  const at = isScoped ? pattern.indexOf('@', 1) : pattern.indexOf('@');
  if (at === -1) return { name: pattern, versions: [] };
  return { name: pattern.slice(0, at), versions: pattern.slice(at + 1).split('||').map((v) => semver.valid(v.trim())) };
};
const build = (patterns) => patterns.map(parse);
const evaluate = (rules, name) => {
  for (const r of rules) {
    if (r.name !== name) continue;
    if (r.versions.length === 0) return true;
    return r.versions;            // first match wins
  }
  return false;
};
const isExcluded = (rules, name, version) => {
  const r = evaluate(rules, name);
  return r === true || (Array.isArray(r) && r.includes(version));
};

const perVersion = build(['@x/p@0.75.4', '@x/p@0.79.4']);
console.log('per-version, need 0.79.4:', isExcluded(perVersion, '@x/p', '0.79.4')); // false  <- bug
const bareName = build(['@x/p']);
console.log('bare-name,    need 0.79.4:', isExcluded(bareName, '@x/p', '0.79.4'));   // true
const union = build(['@x/p@0.75.4 || 0.79.4']);
console.log('union,        need 0.79.4:', isExcluded(union, '@x/p', '0.79.4'));      // true
```

Patterns that fail (newest version not exempt):

- two or more separate `@x/p@<version>` entries for the same name;
   only the
  first listed version is exempt.

Patterns that work (newest version exempt):

- one bare-name entry `@x/p` (all versions).
- one union entry `@x/p@0.75.4 || 0.79.4` (the listed versions).

End-to-end confirmation in this repo:
 with four `@earendil-works/pi-*`
packages each listed as eight separate `name@version` entries
(`0.75.4` through `0.79.4`),
 `pnpm install` failed with the symptom above on
the `0.79.4` entries.
 After collapsing each package to a bare-name entry,
 a
plain `pnpm install` (no `PNPM_CONFIG_MINIMUM_RELEASE_AGE` override) completed
and reported "Already up to date" with no violation.

## Verified workarounds

- **Bare-name entry** (`@earendil-works/pi-ai`):
   exempts every version of the
  package from the age gate.
   Tradeoff:
   the package is never age-checked again,
  including future versions,
   so a compromised or mistakenly-published release
  is not held back by `minimumReleaseAge`.
   Appropriate for first-party packages
  the consumer controls and upgrades deliberately;
   not for arbitrary
  third-party deps.
- **Single `||` union entry** (`@earendil-works/pi-ai@0.79.0 || 0.79.3 || 0.79.4`):
  keeps age-checking for unlisted versions while exempting the listed set.
  Tradeoff:
   every newly-needed version must be added to the union (not appended
  as a separate line,
   which re-triggers the bug),
   so it carries the same manual
  churn the per-version format had,
   minus the silent-drop failure.

## What does not work

- **Separate per-version entries** (`@x/p@a`,
   `@x/p@b`,
   ...):
   the bug itself.
  Only the first listed version is honored;
   this is also the shape
  `pnpm audit --fix` produces,
   so the list self-assembles into the broken form.
- `PNPM_CONFIG_TRUST_LOCKFILE=true`:
   skips lockfile verification,
   so the
  gate passes,
   but it also skips re-resolution.
   It cannot be used to also apply
  an unrelated resolution change (it left a pending `.pnpmfile.mjs` policy
  change unapplied with the lockfile unchanged).
   See
  `doc/troubleshooting/pnpm-minimum-release-age-trust-lockfile.md`.
- `pnpm install --force` alone:
   did not re-trigger the `.pnpmfile.mjs` hook
  on 11.6.0 while `pnpmfileChecksum` was unchanged,
   so it neither applied a
  pending policy change nor surfaced the exclude problem differently.
   It also
  re-runs the same first-match verifier,
   so it does not fix the exclude itself.
- **`PNPM_CONFIG_MINIMUM_RELEASE_AGE=0`** for a single install:
   passes the gate
  because it disables the age check entirely,
   but it disables it for every
  package for that invocation,
   not just the intended ones,
   and does nothing
  durable.
   Useful only as a one-shot to land an unrelated change,
   not as a fix.

## Upstream filing decision

`.out-of-scope/` was checked:
 no exemption for pnpm exists (the
`bun-install.md` entry concerns Bun versus pnpm as the package manager,
 not a
pnpm bug exemption),
 so upstream tracking is in scope.

Duplicate search (`gh search issues`/`gh search prs` on `pnpm/pnpm`,
 terms
drawn from symptom and cause):

- pnpm/pnpm#11881 (closed) and PR pnpm/pnpm#12010 (merged):
   "not respected in
  install or update" / "revalidate packages in minimumReleaseAgeExclude".
   A
  different mechanism:
   excluded packages were pinned to stale versions via a
  `publishedBy` mtime cache shortcut.
   Not the first-match evaluator.
- pnpm/pnpm#10361,
   #10120:
   exclude not respected during `--fix-lockfile` /
  `outdated`.
   Different commands,
   not the same root cause.
- pnpm/pnpm#11668:
   feature request to clean accumulated exclude entries.
  Describes the accumulation that creates the multiple-same-name shape,
   but not
  the first-match drop.

No existing issue describes the first-match evaluator dropping later same-name
entries.
 The contribution would be a new issue.

Six-constraint check:

1.  **Really upstream's fault?
    ** Yes.
     `evaluateVersionPolicy` returns on the
    first name match and silently ignores later same-name rules,
     with no
    warning,
     and pnpm's own `audit --fix` generates the multiple-same-name shape.
2.  **Can upstream fix it?
    ** Yes,
     and the fix is small and localized:
     merge
    rules with the same package name in `createPackageVersionPolicy` (concatenate
    their `exactVersions`,
     with any empty set meaning all versions),
     or emit a
    duplicate-name warning.
     (Prototype below.
    )
3.  **Supporting this use case?
    ** Yes.
     `minimumReleaseAgeExclude` is a supported,
    documented setting and `audit --fix` populates it.
4.  **Would the repo welcome the contribution?
    ** Likely yes.
     `CONTRIBUTING.md`
    exists;
     PR #12010 in this same area disclosed LLM assistance and was merged
    by the lead maintainer;
     no AI-filing ban was found in CONTRIBUTING or the
    issue templates checked.
     A filing must disclose AI assistance and name the
    reproduction,
     source trace,
     prototype,
     and workaround a human verified.
5.  **Will they likely fix it?
    ** Plausible.
     The area is actively maintained
    (multiple 2026 PRs touching `minimumReleaseAgeExclude`),
     related issues are
    open,
     and no won't-fix or non-goal signal was found.
6.  **Prototyped minimal fix?
    ** See "Prototype" below.

### Prototype

Cloned `pnpm/pnpm` (origin `https://github.com/pnpm/pnpm.git`,
 HEAD
`b6826b7d0b3e195bdfbb47233100419777360cb4`) into a disposable
`/tmp/agent/` directory.
 `config/version-policy/src/index.ts` matches the
11.6.0 bundle cited above.
 The minimal fix accumulates `exactVersions` across
every rule whose name matcher matches,
 instead of returning on the first match.
Accumulating in the evaluator (rather than merging by exact package name) is
the robust choice because `nameMatcher` comes from `@pnpm/config.matcher` and
may be a `*` glob,
 so two rules can match the same name without sharing a
literal `packageName`.

```diff
 function evaluateVersionPolicy (rules: VersionPolicyRule[], pkgName: string): boolean | string[] {
+  const matched: string[] = []
   for (const { nameMatcher, exactVersions } of rules) {
     if (!nameMatcher(pkgName)) {
       continue
     }
     if (exactVersions.length === 0) {
       return true
     }
-    return exactVersions
+    matched.push(...exactVersions)
   }
-  return false
+  return matched.length > 0 ? matched : false
 }
```

Verified with a targeted repro (`node repro.mjs`) that copies the real
`parseVersionPolicyRule`,
 `createPackageVersionPolicy`,
 and both the original
and fixed `evaluateVersionPolicy` verbatim,
 no pnpm build needed:

```text
two same-name per-version entries (the audit --fix shape)
  want 0.79.4 excluded -> original=false fixed=true
single union entry
  want 0.79.4 excluded -> original=true fixed=true
bare name
  want 0.79.4 excluded -> original=true fixed=true
single per-version entry (regression check, want 0.75.4)
  want 0.75.4 excluded -> original=true fixed=true

--- assertions ---
original drops newest same-name entry (bug present): true
fixed honors newest same-name entry: true
fixed still honors oldest entry (no regression): true

RESULT: bug reproduced under original, resolved under fix
```

The fix preserves the single-entry,
 `||`-union,
 and bare-name cases and
additionally honors every listed version when a name appears in multiple
entries.
 All six constraints now hold,
 so the draft below is fileable.
 Filing
posts to a third-party tracker (an outward-facing action),
 so it is left for a
maintainer to submit rather than filed automatically.

### Draft upstream issue

~~~md
**Title:** `minimumReleaseAgeExclude` only honors the first listed version when a package name appears in multiple entries

**Labels:** type: bug, area: supply chain security

**Description**

When `minimumReleaseAgeExclude` contains more than one `name@version` entry for
the same package name, only the first listed version is exempted from the
minimum-release-age check. Later same-name entries are silently ignored, with
no warning. This is the shape `pnpm audit --fix` produces over time (it appends
a new `name@version` entry per patched release, see #11668), so the list
self-assembles into a form where the newest entry, the one a developer just
added to unblock a fresh release, is the one that is dropped.

Root cause is `evaluateVersionPolicy` in `config/version-policy`, which returns
on the first rule whose name matches:

```js
function evaluateVersionPolicy(rules, pkgName) {
  for (const { nameMatcher, exactVersions } of rules) {
    if (!nameMatcher(pkgName)) continue;
    if (exactVersions.length === 0) return true;
    return exactVersions;   // first name match wins; later same-name rules never consulted
  }
  return false;
}
```

`createPackageVersionPolicy` builds one rule per entry, so two entries for the
same name produce two rules and the second is dead.

**Reproduction**

```yaml
# pnpm-workspace.yaml
minimumReleaseAge: <some non-zero value>
minimumReleaseAgeStrict: true
minimumReleaseAgeExclude:
  - 'my-scope/pkg@1.0.0'
  - 'my-scope/pkg@1.1.0'   # freshly published, within the age cutoff
```

With `my-scope/pkg@1.1.0` resolved in the lockfile, `pnpm install` fails with
`ERR_PNPM_MINIMUM_RELEASE_AGE_VIOLATION` for `1.1.0`, even though it is listed.
Replacing both lines with `my-scope/pkg@1.0.0 || 1.1.0` (one entry) or
`my-scope/pkg` (bare name) makes the install pass.

**Suggested fix**

In `createPackageVersionPolicy` (`config/version-policy`), merge rules that share
a package name before evaluation: union their `exactVersions`, and treat any
member with an empty `exactVersions` (a bare name) as "all versions". A lighter
alternative is to keep evaluation as-is but have `evaluateVersionPolicy`
accumulate `exactVersions` across all name matches instead of returning on the
first. Either keeps the existing single-entry and `||`-union inputs working
while fixing accumulated per-version entries. A duplicate-name warning at parse
time would also surface the footgun.

Note the blast radius: `evaluateVersionPolicy` / `createPackageVersionPolicy`
also back `trustPolicyExclude` (via `createExcludePolicy` in
`createNpmResolutionVerifier`), so this fix applies there too. That is a
consistent improvement (multiple same-name trust-exclude entries would all be
honored), not a regression. The reproduction above exercises exact-name
matches; the glob case (`nameMatcher` may be a `*` matcher) is reasoned, not
separately tested.

This report was prepared with AI assistance: the source trace, the
reproduction, the prototype fix, and the workarounds above were produced by an
AI agent. The submitter should review and confirm them before filing.
~~~
