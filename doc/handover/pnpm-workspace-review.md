# pnpm workspace review handover

> Moved 2026-10-29 by the generic package category elimination
> (`doc/planning/generic-category-elimination.md`);
> paths and names below this note are historical:
> `package/dev-script/watch-restart` is now `package/watch-restart/cli`.

> Moved 2026-10-29 by the generic package category elimination
> (`doc/planning/generic-category-elimination.md`);
> paths and names below this note are historical:
> `package/module/numeric-format` is now `package/numeric-format/core`;
> `package/module/toml-edit` is now `package/toml-edit/core`;
> `package/module/or-throw` is now `package/or-throw/core`;
> `package/module/test` is now `package/test-harness/core`.

## Status

The user asked for a comprehensive review of `pnpm-workspace.yaml` after two insufficient passes.
Keep updating this file as evidence changes.

Current work is a review only,
 except this handover file is intentionally persisted at the user's request.
Do not edit `pnpm-workspace.yaml` unless the user later asks for fixes.

Unrelated dirty worktree state observed before creating this file:

- `mise.lock` was already modified.

## Suggested skills for next session

- `code-review`,
   for severity-ranked findings.
- `troubleshooting-doc`,
   only if diagnosing a new pnpm behavior or writing a troubleshooting doc.
- `handoff`,
   when updating or replacing this handover.

## Source files under review

- `pnpm-workspace.yaml`
- `pnpm-lock.yaml`
- Root `package.json`
- Active package manifests under `package/*/*/package.json`
- Deprecated workspace package manifests under `package-deprecated/*/*/package.json`
- Paused package manifests under `package-paused/*/*/package.json`,
   only to explain stale catalog entries

## Commands and checks already run

- `pnpm --version` returned `11.8.0`.
- `pnpm config list --location project` showed `managePackageManagerVersions: false`,
   no `pmOnFail`,
  `saveWorkspaceProtocol: false`,
   `linkWorkspacePackages: true`,
   and `preferWorkspacePackages: true`.
- `pnpm install --lockfile-only --frozen-lockfile` made no file changes but emitted a workspace cycle warning.
- `pnpm peers check` reported no peer dependency issues.
- `pnpm ignored-builds` reported no automatically ignored builds and explicit denies for
  `@vscode/ripgrep`,
   `core-js`,
   and `protobufjs`.
- `pnpm why @anthropic-ai/sdk --recursive` showed `@anthropic-ai/sdk@0.92.0`.
- `pnpm why postcss --recursive` showed active catalog consumers on `postcss@8.5.13`,
   while other
  graph paths have `postcss@8.5.15`.
- `pnpm why marked --recursive` showed `marked@18.0.5` via `@earendil-works/pi-tui@0.79.9`.
- `pnpm why ws --recursive` produced no installed `ws` package.
- `pnpm why @mitata/counters --recursive` showed a direct dev dependency from
  `@monochromatic-dev/test-fixture-file-enforcer-perf` and an injected dependency below `mitata`.
- A throwaway workspace verified `saveWorkspaceProtocol: false` plus `preferWorkspacePackages: true`
  saves a local workspace dependency as a plain version,
   not `workspace:*`.
- A throwaway workspace verified default `catalogMode` saves non-catalog additions as direct versions.
- From `package/pi-plugin/auto-mode`,
   importing `@earendil-works/pi-ai/google`,
   `/mistral`,
   and
  `/amazon-bedrock` fails with `ERR_MODULE_NOT_FOUND` because their provider dependencies are removed.
  Importing `@earendil-works/pi-ai/anthropic` succeeds.

## pnpm docs and source evidence already checked

- Official pnpm 11 settings docs say `pmOnFail` replaces `managePackageManagerVersions`,
  `packageManagerStrict`,
   and `packageManagerStrictVersion`.
- Installed pnpm source at
  `/var/home/user/.local/share/mise/installs/pnpm/11.8.0/dist/pnpm.mjs` contains `pmOnFail` handling
  and no active `managePackageManagerVersions` handling.
- Installed pnpm source shows default `minimum-release-age` is `24 * 60` minutes and
  `minimum-release-age-strict` gates resolution when `minimumReleaseAge` is truthy.
- Mitata package source in `node_modules/.pnpm/mitata@1.0.34/node_modules/mitata/src/main.mjs`
  wraps `import('@mitata/counters')` in platform checks and fallback handling.
- Mitata README documents `@mitata/counters` as an optional hardware counters extension.

## Findings collected so far

### Blockers

- `pnpm-workspace.yaml:14` and `pnpm-workspace.yaml:198` conflict.
  Catalog requires `@anthropic-ai/sdk >=0.99.0`,
   but the override allows `>=0.91.1`.
  The lock resolves `0.92.0`,
   below the catalog floor.
- `pnpm-workspace.yaml:105` and `pnpm-workspace.yaml:228` conflict.
  Catalog requires `postcss >=8.5.14`,
   but the override allows `>=8.5.10`.
  Active catalog consumers resolve `8.5.13`,
   below the catalog floor.
- `pnpm-workspace.yaml:302-304` remove provider dependencies from `@earendil-works/pi-ai` while the
  package still exports provider subpaths that statically import those dependencies.
  Verified imports for Google,
   Mistral,
   and Amazon Bedrock fail from an active package context.

### Warnings

- `pnpm-workspace.yaml:177-187` misuses `packageExtensions` for `mitata`.
  It injects `@mitata/counters` as a hard dependency into every `mitata` version,
  even though mitata treats counters as an optional extension and wraps import failures.
  Suggested shape if retained:

  ```yaml
  packageExtensions:
    mitata@1.0.34:
      optionalDependencies:
        '@mitata/counters': '>=0.0.8'
  ```

  Do not use `catalog:` inside this `packageExtensions` entry.
   A throwaway fixture showed pnpm fails
  with `ERR_PNPM_SPEC_NOT_SUPPORTED_BY_ANY_RESOLVER` when `packageExtensions` injects a
  `catalog:` dependency into an external package manifest.
   Remove the extension if hardware counters
  are not required by the benchmark fixture.
- `pnpm-workspace.yaml:162` uses removed pnpm 11 config `managePackageManagerVersions: false`.
  Replace with `pmOnFail: ignore` if mise owns pnpm version management.
- `pnpm-workspace.yaml:357` combines `resolutionMode: highest` with many open-ended `>=` catalog ranges.
  Lock regeneration or update can select new majors unless another process tightens them.
- Missing `catalogMode: strict` means new external deps can bypass the catalog.
  Throwaway verification saved a non-catalog package as a direct version.
- `pnpm-workspace.yaml:355` forces `packageImportMethod: hardlink`.
  pnpm docs describe `clone` as safer because edits in `node_modules` do not modify the central store.
  The repo and pnpm store are on Btrfs,
   and a reflink test under `/var/home/user` succeeded,
   so `auto`
  should be able to use clone semantics on this host.
- `pnpm-workspace.yaml:356` and `pnpm-workspace.yaml:360` combine `preferWorkspacePackages: true`
  with `saveWorkspaceProtocol: false`.
   This makes local workspace packages preferred even without a
  `workspace:` dependency specifier,
   and newly added local deps are saved as plain versions.
  Throwaway verification saved local workspace deps as plain versions rather than `workspace:*`.
- Publishable packages depend on private workspace packages.
   Verified manifests:
  `package/dev-script/watch-restart/package.json` depends on private `module-logger`,
  `package/module/test/package.json` depends on private `module-fs-path` and `module-logger`,
   and
  `package/module/toml-edit/package.json` depends on private `module-logger`.
  pnpm converts `workspace:*` before publish,
   so those public packages would point consumers at
  unpublished private packages.
- `pnpm install --lockfile-only --frozen-lockfile` warns about workspace cycles involving
  `package/config/tsdown`,
   `package/module/test`,
   `package/module/numeric-format`,
   and
  `package/module/or-throw`.

### Nits or cleanup candidates

- Parent-scoped removal overrides that are stale or ineffective against current production manifests:
  `@earendil-works/pi-ai>chalk`,
   `@earendil-works/pi-ai>proxy-agent`,
  `@earendil-works/pi-ai>undici`,
   and `@earendil-works/pi-ai>zod-to-json-schema`
  target no current `dependencies`,
   `optionalDependencies`,
   or `peerDependencies` entry on
  `@earendil-works/pi-ai@0.79.9`.
  `@earendil-works/pi-coding-agent>marked` targets no current production dependency on
  `@earendil-works/pi-coding-agent@0.79.9`;
   `marked` remains installed via
  `@earendil-works/pi-tui@0.79.9`.
  `@earendil-works/pi-tui>chalk`,
   `@earendil-works/pi-tui>koffi`,
   and
  `@earendil-works/pi-tui>mime-types` target no current production dependency on
  `@earendil-works/pi-tui@0.79.9`.
  Re-check before final because some may be intentional future guards.
  Current recommendation depends on intent.
   If the goal is current install pruning,
   delete the no-op
  entries and replace `@earendil-works/pi-coding-agent>marked` with `@earendil-works/pi-tui>marked`.
  Current installed manifests show `marked` belongs to `@earendil-works/pi-tui@0.79.9`,
   while
  `@earendil-works/pi-ai@0.79.9` now uses `http-proxy-agent` and `https-proxy-agent`,
   not
  `proxy-agent`.
   If the goal is future guardrails,
   keep no-op parent-scoped removals only with comments
  saying they are future guards,
   not current graph pruning.
   For the specific policy "drop koffi and
  mime-types for pi",
   current `pnpm why koffi mime-types --recursive` is empty,
   so the existing
  `@earendil-works/pi-tui>koffi` and `@earendil-works/pi-tui>mime-types` entries are only future
  guards against those deps returning to `pi-tui`;
   keep them with that comment if that is intentional.
  If the goal is global absence,
   use global removal overrides,
   but that has broader breakage risk than
  parent-scoped removals.
- `ws` override is currently unused.
- `node-domexception` shim override is currently unused.
- 34 catalog entries are unused by active workspace/root packages.
  Most correspond to `package-paused`,
   which is intentionally outside `packages:` globs.
- `minimumReleaseAgeStrict: true` relies on pnpm 11 default `minimumReleaseAge: 1440`.
  If one-day strict maturity is policy,
   make `minimumReleaseAge: 1440` explicit.
- `allowBuilds` has deny entries for packages not currently in the lock,
   including
  `@vscode/ripgrep` and `protobufjs`.
   Comment as guardrails or remove if stale.

## Evidence scripts worth re-running

To find catalog entries unused by active packages:

```bash
python3 - <<'PY'
from pathlib import Path
import json, yaml
catalog = yaml.safe_load(Path('pnpm-workspace.yaml').read_text()).get('catalog', {})
usage = {name: [] for name in catalog}
for package_path in [Path('package.json'), *sorted(Path('packages').glob('*/*/package.json')), *sorted(Path('package-deprecated').glob('*/*/package.json'))]:
    data = json.loads(package_path.read_text())
    for section in ['dependencies', 'devDependencies', 'optionalDependencies', 'peerDependencies']:
        for name, spec in data.get(section, {}).items():
            if name in usage and spec == 'catalog:':
                usage[name].append(f'{package_path}:{section}')
for name, hits in usage.items():
    if not hits:
        print(name)
PY
```

To inspect parent-scoped override targets against installed manifests:

```bash
python3 - <<'PY'
from pathlib import Path
import json, yaml

def split_parent(selector: str) -> str:
    parent = selector.split('>', 1)[0]
    if parent.startswith('@'):
        return '@' + parent.split('@')[1]
    return parent.split('@', 1)[0]

workspace = yaml.safe_load(Path('pnpm-workspace.yaml').read_text())
package_json_by_name = {}
for package_json in Path('node_modules/.pnpm').glob('*/node_modules/**/package.json'):
    try:
        data = json.loads(package_json.read_text())
    except Exception:
        continue
    name = data.get('name')
    if name and name not in package_json_by_name:
        package_json_by_name[name] = data
for key in [key for key in workspace.get('overrides', {}) if '>' in key]:
    parent_name = split_parent(key)
    dep = key.split('>', 1)[1]
    data = package_json_by_name.get(parent_name)
    deps = {}
    for section in ['dependencies', 'optionalDependencies', 'peerDependencies']:
        deps.update(data.get(section, {}) if data else {})
    print(f'{key}: current-prod-spec={deps.get(dep)}')
PY
```

## Next steps

- Continue auditing `pnpm-workspace.yaml` from line 281 onward in detail.
- Re-check whether each stale-looking override is intentional policy or accidental dead config.
- Ask an independent reviewer before final,
   because this session already missed findings twice.
- Final response should include severity sections and concrete fixes only,
   not implementation changes.
