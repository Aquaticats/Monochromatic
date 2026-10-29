# Workflow shell line limit

## Status

Grilling in progress; started 2026-10-29.
Nothing is implemented.
This records the decisions the repository owner has answered and the questions still open.

## Request

Make `monochromatic-lint` ban long shell in `.github/workflows/*.{yml,yaml}`.

## Evidence

- `package/linter/monochromatic-lint` lints Rust and Markdown (including MDX) only.
  It has no YAML dependency and no YAML host.
- `ProcessorLanguage` in `package/linter/monochromatic-lint/src/processors_model.rs` has the variants `Rust`,
  `Markdown` and `Mdx`.
  The processor files total about 1100 lines.
  Fenced Rust in Markdown is already linted as a virtual file such as `guide.md/3.rs`,
  with `MappedLine` records carrying container prefixes back to host positions.
- `.github/workflows` holds 19 `.yml` files and no `.yaml` file.
  They contain 142 `run: |` blocks.
  Measured by a scratch script on 2026-10-29 (raw lines, blank and comment lines included):
  114 blocks are longer than 5 lines,
  77 longer than 10,
  54 longer than 15,
  40 longer than 20,
  32 longer than 30,
  and 16 longer than 50.
  The longest is 351 lines at `.github/workflows/music-player-release.yml:2164`.
- [`bash-usage-boundary.md`](bash-usage-boundary.md) records that `SCR` already prohibits Bash scripts
  and that `shell: node {0}` is the repository's established alternative.
  It adopts no exception and no workflow-wide rule.

## Decisions

- **Measure:** line count of the run body, skipping blank and comment-only lines,
  with a configurable `max`.
  A control-flow ban was offered and not chosen.
- **Scope:** a step counts when its effective shell is one of bash, sh, cmd, pwsh, zsh or fish.
  Steps whose effective shell is `node {0}` or `python` do not count.
  Effective shell is the step's `shell:`,
  else `defaults.run.shell` of the job or workflow,
  else the runner default.
- **Rollout:** the rule is `error` repository-wide.
  Each workflow file that already exceeds the default gets a configuration block
  setting `max` to its current longest block,
  so no block in it can grow.
- **Shape:** each shell `run:` body becomes a virtual file in the embedded-source style,
  such as `ci.yml/42.sh`,
  and a rule lints it.
  The Markdown-fence processor is the model.

## Open

- Rule name and virtual file suffixes now that scope covers more than Bash.
- Default `max`.
- Handling of shell templates such as `bash -e {0}` and of expression-valued shells or runners.
- Which files the YAML host recognises,
  and whether composite actions under `.github/actions` are in scope.
- Choice of YAML parser,
  which must expose byte offsets of block scalar content.
- Whether inline directives (`#` comments) can suppress the rule in YAML.
