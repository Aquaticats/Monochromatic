# pi-guardrail

Tool-call guardrails for pi.

The extension blocks misleading `bun test` Bash tool calls and refuses `edit` or
`write` calls for protected paths matched with `.gitignore` semantics.

## Pi dependency trust

PJY:
 Minimize Pi dependencies,
 including in Pi and DSH integrations.
Pi's dependency hygiene and code quality have not been cleared for high trust.

## Installation

Add this package to global pi settings at `~/.pi/agent/settings.json`:

```json
{
  "packages": [
    "/var/home/user/Monochromatic/package/pi-plugin/guardrail"
  ]
}
```

Keep project `.pi/settings.json` package-free unless the guardrail becomes
shared project policy.

## Built-in behavior

### `bun test` Bash blocking

The guardrail blocks Bash tool calls that invoke `bun test` at a shell command
segment boundary.

Use package mise tasks instead:

```sh
mise run //package/<path>:test:unit
```

When no such task exists,
 add one to the target package's `mise.toml` first.
For ad-hoc single-file runs,
 use `node <file>` directly.

### `pnpm-lock.yaml` edit and write blocking

The guardrail ships this protected-path rule:

```json
{
  "pnpm-lock.yaml": "edit pnpm-workspace.yaml and package.json files then run `pnpm install`"
}
```

Because matching follows `.gitignore` semantics,
 `pnpm-lock.yaml` matches that
basename anywhere under pi's current working directory.

## Global config

Optional global config lives at:

```text
~/.pi/agent/extensions/pi-guardrail.json
```

Short form maps `.gitignore`-style patterns to refusal messages:

```json
{
  "Cargo.lock": "edit Cargo.toml then run `cargo update` intentionally",
  "package-lock.json": "edit package.json then run `npm install`"
}
```

Advanced form can also disable the `bun test` guard:

```json
{
  "blockBunTest": false,
  "pathRules": {
    "Cargo.lock": "edit Cargo.toml then run `cargo update` intentionally"
  }
}
```

Rules are appended after built-in defaults.
 Use `.gitignore` negation to unguard
a built-in rule:

```json
{
  "!pnpm-lock.yaml": ""
}
```

## Source structure

```text
src/
  index.ts                         # Extension entry point and tool_call registration
  bash-guard.ts                    # Shared analyzer backed `bun test` detection
  path-guard.ts                    # `ignore`-backed protected-path matching
  path-normalize.ts                # Tool path extraction and cwd-relative normalization
  config.ts                        # Global config loader orchestration
  config-file.ts                   # Optional JSON config reading
  config-normalize.ts              # Config shape validation and normalization
  config-paths.ts                  # Global config path helper
  constants.ts                     # Built-in rules and messages
  value.ts                         # Shared value-shape helpers
  types.ts                         # Shared config and decision types
  *.unit.test.ts                   # Unit tests
  mise.verify-extension.ts         # Built extension smoke verification
```

## Validation

Run package validation from the repository root:

```sh
mise run //package/pi-plugin/guardrail:build
mise run //package/pi-plugin/guardrail:test:unit
mise run //package/pi-plugin/guardrail:lint
mise run //package/pi-plugin/guardrail:verify:extension
```
