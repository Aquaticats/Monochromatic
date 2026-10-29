# Meta Package Manager 7.6.1 extends through manager definitions but TOML cannot forward restore versions

> Definition path note:
>  this record is the 7.6.1 investigation and stays historical as written.
>  In `meta-package-manager` 8.2.0,
>  the version this repository adopts,
>  trusted local definitions belong under `[mpm.overrides.<id>]`
>  (moved 2026-10-26),
>  so the `[mpm.managers.<id>]` blocks in this record are the historical path
>  (`package/dev-script/file-enforcer/src/toolchain/mpm-manifest.ts`,
>  `doc/planning/mpm-toolchain-migration.md`,
>  section `Implementation record (toolchain-core, 2026-10-26)`).

## Symptom

A Mise-removal assessment initially treated missing Meta Package Manager adapters as requiring commands outside MPM.
That wording understated MPM's extension surface.

Stable MPM 7.6.1 supports two manager implementation levels:

- trusted local TOML definitions for fixed command and parser shapes;
- full Python manager classes shipped in MPM for version pinning,
  stateful parsing,
  conditional behavior,
  and arbitrary adapter logic.

A TOML manager becomes a first-class runtime manager without modifying MPM.
It receives `--<id>` and `--no-<id>` selectors and participates in the operations it declares.

The boundary appears when a definition tries to forward the version read from an `mpm restore` entry:

```text
critical: Configuration validation error: mpm.managers.fixture.operations.install.args: unknown placeholder(s): {version}. Allowed: {package_id}.
```

Stable MPM does not expose a separately installed Python-plugin entry point for full-power out-of-tree managers.
That case requires an upstream or forked class,
or a direct consumer-side provider command.

## Root cause

### Configuration managers register before the CLI is built

`meta_package_manager/__main__.py:42-54` loads manager definitions before importing the Click command group.
That ordering lets custom manager IDs participate in dynamic CLI option construction:

```python
from meta_package_manager.config import register_eager_config_managers
from meta_package_manager.pool import pool

register_eager_config_managers(pool)

from meta_package_manager.cli import mpm

mpm(prog_name=mpm.name)
```

`meta_package_manager/config.py:739-779` rejects remote or unsafe definitions,
then builds and registers accepted local definitions:

```python
def register_config_managers(
    pool: ManagerPool,
    definitions: Mapping[str, ManagerDefinition],
    *,
    source: Path | None = None,
    source_is_url: bool = False,
) -> list[str]:
    # ...
    if source_is_url:
        logging.warning(
            f"Refusing to define manager {manager_id!r} from a remote config URL "
            "for safety (see docs/security.md).",
        )
        continue
    # ...
    pool.add_manager(build_manager_class(definition)())
```

`meta_package_manager/config.py:685-715` makes executable configuration local and trust-gated on POSIX:

```python
def config_file_is_trusted(path: Path) -> bool:
    # ...
    trusted_owners = {os.getuid(), 0}
    for target, is_dir in ((path, False), (path.parent, True)):
        # ...
        if stats.st_uid not in trusted_owners:
            return False
        writable_by_others = stats.st_mode & (stat.S_IWGRP | stat.S_IWOTH)
        sticky = stats.st_mode & stat.S_ISVTX
        if writable_by_others and not (is_dir and sticky):
            return False
```

### TOML definitions deliberately expose a bounded operation DSL

`meta_package_manager/definitions.py:705-737` computes placeholders in every operation and rejects any outside the
operation's allow-list:

```python
found_placeholders = {
    token for arg in args for token in ARG_PLACEHOLDER_REGEX.findall(arg)
}
allowed_placeholders = ALLOWED_ARG_PLACEHOLDERS.get(op_name, frozenset())
unknown_placeholders = found_placeholders - allowed_placeholders
if unknown_placeholders:
    # ...
    raise ValidationError(
        f"{path}.args",
        f"unknown placeholder(s): {listing}. {hint}.",
        code="invalid_value",
    )
```

`meta_package_manager/definitions.py:340-346` allows only `{package_id}` for install and single-package upgrade:

```python
ALLOWED_ARG_PLACEHOLDERS: Final[Mapping[str, frozenset[str]]] = {
    "install": frozenset({"package_id"}),
    "remove": frozenset({"package_id"}),
    "remove_orphan": frozenset({"package_id"}),
    "search": frozenset({"query"}) | SEARCH_REFINEMENT_KEYS,
    "upgrade_one": frozenset({"package_id"}),
}
```

`meta_package_manager/definitions.py:1034-1052` confirms that a restore version reaches the synthesized method,
produces a warning,
and is not passed into the command arguments:

```python
def _warn_version_unsupported(version: str | None) -> None:
    if version:
        logging.warning(
            "Configuration-defined managers do not support version pinning. "
            "Letting the package manager choose the version.",
        )


def _make_install(spec: OperationSpec) -> Callable[..., str]:
    def install(
        self: PackageManager, package_id: str, version: str | None = None
    ) -> str:
        _warn_version_unsupported(version)
        return self.run_cli(
            *_render_args(spec.args, package_id=package_id),
```

This does not prevent a fixed literal version or a version encoded into `package_id` from appearing in a custom command.
It prevents one reusable definition from forwarding each restore entry's separate version field.

### Full-power managers are class-based

The upstream manager-authoring guide at `.claude/skills/add-manager/SKILL.md:11-27` distinguishes the two paths:

```markdown
- **Class-based:** a Python module in `meta_package_manager/managers/`. Full power: multi-line or stateful output parsing,
  version pinning, per-operation search flags, conditional `sudo`, delegation, arbitrary logic.
- **Config-based:** a declarative `[mpm.managers.<id>]` block that `mpm` turns into a live manager at startup, with no
  Python. Quick to write, but constrained: each operation is a fixed argument list, and listings must parse either
  line-by-line with a single regex or as one flat top-level JSON array.
```

The same guide states that private config definitions require no source-tree change,
while bundled definitions and Python classes are shipped with MPM.

### There is no external Python-plugin loader in 7.6.1

A source search for Python package entry-point loading returned no matches:

```bash
rg --line-number \
  '(importlib\.metadata\.entry_points|entry_points\(|\[project\.entry-points)' \
  meta_package_manager pyproject.toml docs
```

A broader registration search found `ManagerPool.add_manager()` callers only in bundled and local configuration paths.
`meta_package_manager/pool.py:170-185` builds the initial pool from shipped classes and bundled TOML definitions,
and
`meta_package_manager/config.py:739-779` adds trusted config-defined managers.

Historical issue [kdeldycke/meta-package-manager#276][issue-276]
asked for extension documentation and mentioned plugins.
The maintainer rejected external plugins in 2021 because the project preferred a monolithic source and test surface.
The later config-defined manager feature supplies a lightweight out-of-tree extension path,
but 7.6.1 still has no full-power external class-plugin mechanism.

## Verification

### Version and artifact

- Meta Package Manager version:
  `7.6.1`.
- Source tag:
  `v7.6.1`.
- Source commit:
  `d0404ee2f93f95dac9329e41e47511ef6da4132b`.
- Linux x64 binary SHA-256:
  `de58308be281c46cd515ce01af4932ea3b4b5fcbec13d211d987cf463e4014be`.
- Source clone:
  `~/temp/agent/meta-package-manager-2026-09-12`.
- Fixture root:
  `~/temp/agent/mpm-extensibility.ii6hE2wS`.

The downloaded binary's SHA-256 matched GitHub's release metadata.
Candidate execution used `debian:bookworm-slim` with 2 GiB memory,
two CPUs,
a 128-process limit,
no network,
a read-only root,
a read-only fixture mount,
and a temporary home.
The repository was not mounted.

`gh attestation verify` did not accept the release attestation
under its default GitHub Actions issuer policy and emitted:

```text
Error: verifying with issuer "sigstore.dev"
```

The runtime probe therefore relies on the matching published checksum plus process isolation,
not a claim that GitHub's attestation verifier established provenance.

### Working catalog

The working fixture defines a manager over `/usr/bin/echo`:

```toml
[mpm.managers.fixture]
name = "Fixture manager"
platforms = ["all_platforms"]
cli_names = ["echo"]
version_regexes = ['echo \(GNU coreutils\) (?P<version>\S+)']

[mpm.managers.fixture.operations.installed]
args = ["fixture-package 1.2.3"]
regex = '^(?P<package_id>\S+) (?P<installed_version>\S+)$'

[mpm.managers.fixture.operations.install]
args = ["install", "{package_id}"]
```

This command registered the manager:

```bash
/fixture/mpm --config /fixture/good.toml --fixture managers
```

Relevant output:

```text
Manager ID  Name             Supported  CLI              Executable  Version
fixture     Fixture manager  ✓          ✓ /usr/bin/echo  ✓           ✓ 9.1
```

This command exercised its `installed` parser:

```bash
/fixture/mpm --config /fixture/good.toml --fixture installed
```

Relevant output:

```text
Package ID       Manager  Installed version
fixture-package  fixture  1.2.3
1 package total (fixture: 1).
```

Both commands exited 0.

### Failing catalog

Adding `{version}` to the install arguments fails configuration validation:

```toml
[mpm.managers.fixture.operations.install]
args = ["install", "{package_id}", "{version}"]
```

```bash
/fixture/mpm --config /fixture/bad-version.toml managers
```

The command exited 1 with the diagnostic quoted in the Symptom section.

Remote custom definitions and unsafe-permission local definitions also fail registration by construction in
`meta_package_manager/config.py:739-779`.
They were not exercised because the source branch is explicit and the migration should use a trusted generated local
configuration.

## Verified workarounds

### Use a trusted TOML definition for simple providers

A project-generated local definition can adapt a missing provider when:

- operations are fixed argument vectors;
- package records are one line each or a flat JSON array;
- one command supplies each record;
- dynamic restore-version forwarding is unnecessary;
- unconditional per-operation privilege is sufficient.

The fixture verifies runtime registration and parsing.

Tradeoff:
the definition is executable configuration and must remain local and trust-gated.
It cannot represent stateful parsing,
pagination,
dynamic versions,
or arbitrary option models.

### Contribute or carry a Python manager class for full-power behavior

A Python class can implement version pinning,
stateful or multi-line parsing,
multiple subprocesses,
conditional behavior,
and provider-specific options.
This is the documented MPM architecture used by its built-in class managers.

Tradeoff:
a contributed class waits for an MPM release.
A fork receives immediate control but also owns rebases,
security updates,
release artifacts,
and the complete adapter test surface.
Stable 7.6.1 does not load such classes from separately installed plugin packages.

### Keep provider-specific execution at the repository boundary

When an adapter is missing and an MPM fork is not justified,
file-enforcer or a pnpm script can invoke the provider directly while MPM continues to manage the adapters it handles
well.

Tradeoff:
the tool manifest has more than one execution route.
The benefit is preserving provider-specific versions,
features,
components,
and interactive setup without weakening MPM's generic schema.

## What does not work

### Treat `{version}` as a TOML operation placeholder

Stable 7.6.1 rejects it during validation.
The allowed install placeholder is `{package_id}`.

### Expect a Python package entry point to register a full manager

Stable source has no package-entry-point discovery path for manager classes.
Installing a package that merely exports a `PackageManager` subclass does not make MPM discover it.

### Force stateful providers into the TOML DSL

The upstream authoring guide routes multi-line records,
pagination,
per-package follow-up calls,
version pinning,
conditional privilege,
and arbitrary logic to class-based managers.
A wrapper executable can flatten those semantics first,
but then the wrapper owns the complexity rather than MPM's definition.

### Load executable manager definitions from a remote URL

MPM refuses remote definitions by design.
Remote built-in overrides remain a backward-compatibility surface,
not an extension route for a new manager.

## Upstream filing artifact

Nothing to file.
The current behavior is documented and the user question is answered by existing extension mechanisms.

### Upstream filing decision

1. Upstream fault:
   no.
   MPM deliberately provides bounded TOML definitions and full-power shipped classes.
2. Upstream fixability:
   yes in principle.
   MPM could add external class discovery or extend the TOML schema,
   but neither is required to correct a defect.
3. Supported use case:
   yes for trusted config-defined managers and upstream class contributions;
   no stated support for separately installed full-power plugins.
4. Contribution policy:
   `docs/contributing.md`,
   `.claude/skills/add-manager/SKILL.md`,
   `claude.md`,
   and the issue templates invite manager reports and pull requests.
   No AI-assistance prohibition was found.
5. Likely upstream action:
   not established for schema expansion,
   and historically negative for external plugins.
   [Issue 276][issue-276] records the maintainer's preference for a monolithic tested manager set.
6. Minimal prototype:
   not applicable because this report proposes no upstream change and constraints 1,
   3,
   and 5 do not support filing an external-plugin feature request.

`.out-of-scope/` contains no MPM-specific exemption.
Tracker searches for `manager plugin`,
`config-defined manager`,
and `version pinning` found issue 276 and no current duplicate requiring an additive comment.
The issue is locked,
and current documentation already contains the new information that would have been additive to its 2021 discussion.

[issue-276]: https://github.com/kdeldycke/meta-package-manager/issues/276
