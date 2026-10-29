# Workflow shell line limit

## Status

Grilling finished and the repository owner confirmed the design on 2026-10-29;
the owner then asked for the implementation,
and it landed the same day in `package/linter/monochromatic-lint`.
The linter side is complete and verified.
The ban takes effect in the repository's own lint run only once the pinned `cargo:monochromatic-lint` is a
release that contains these rules (see "Enforcement starts at the next pin").
This records the decisions the repository owner has answered,
the defaults adopted from them,
and what the implementation found.

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
- `.github/workflows` holds 20 `.yml` files and no `.yaml` file.
  They contain 142 `run: |` blocks.
  Measured by a scratch script on 2026-10-29 (raw lines,
  blank and comment lines included):
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

- **Measure:**
  line count of the run body,
  skipping blank and comment-only lines,
  with a configurable `max`.
  A control-flow ban was offered and not chosen.
- **Scope:**
  a step counts when its effective shell is one of bash,
  sh,
  cmd,
  pwsh,
  zsh or fish.
  Steps whose effective shell is `node {0}` or `python` do not count.
  Effective shell is the step's `shell:`,
  else `defaults.run.shell` of the job or workflow,
  else the runner default.
- **Rollout:**
  the rule is `error` repository-wide.
  Each workflow file that already exceeds the default gets a configuration block
  setting `max` to its current longest block,
  so no block in it can grow.
- **Shape:**
  each shell `run:` body becomes a virtual file in the embedded-source style,
  such as `ci.yml/42.sh`,
  and a rule lints it.
  The Markdown-fence processor is the model.

- **Rule name:**
  `shell/max-lines`.
  Virtual files take a per-dialect suffix:
  `.sh` (bash,
  sh,
  zsh),
  `.fish`,
  `.ps1` (pwsh) and `.cmd` (cmd).
- **Default `max`:**
  2 lines.
  The repository owner's reason:
  any longer and TypeScript should be used.
- **Banned constructs:**
  the repository owner also wants these banned,
  by a string scan,
  because they are tell-tale signs of too much shell:
  `|`,
  `&&`,
  `||`,
  `|&`,
  `;`,
  `$(...)` and backticks.
  The owner chose this widest option over pipes and `&&` alone.
- **Scan precision:**
  the scan skips `${{ }}` expressions,
  quoted strings and comments.
  Adopted consequence,
  open to veto:
  inside double quotes Bash and Zsh still execute `$(...)` and backticks,
  so those two are flagged there,
  while `|`,
  `&&` and `;` inside any quotes are not.
- **Ratchet for the banned constructs:**
  the rule takes an occurrence `max` per run body,
  default 0,
  and a ratchet block sets it to the file's current worst body.
- **Line counting:**
  every physical non-blank,
  non-comment line counts,
  including `\` continuations and heredoc content.
- **Rule structure:**
  one rule per banned construct,
  each with an occurrence `max` defaulting to 0.
  The owner chose this over one combined rule.
- **Dialects:**
  a per-dialect table says which constructs apply,
  so a PowerShell backtick is not flagged
  as command substitution and cmd is not scanned for `;`.
  The exact fish and pwsh entries must be checked against their manuals before coding.
- **Directives:**
  both families of rule refuse inline directives;
  the only exemption is a ratchet block in `monochromatic-lint.config.jsonc`.
- **YAML parsing:**
  a hand-written minimal extractor,
  not a crate.
  The owner chose this over running the `choosing-technology` process.
  The extractor can miss forms it does not understand,
  so any `run:` value it cannot read is a `core/processing-failure`,
  never a skip.
- **Unresolvable `shell:`:**
  a `shell:` value that is an expression,
  such as `${{ matrix.shell }}`,
  is a `core/processing-failure` finding at the value.
- **Host files:**
  `.github/workflows/*.{yml,yaml}` and composite actions
  (`.github/actions/**/action.{yml,yaml}`,
  steps under `runs.steps`).
  The request named workflows only;
  the owner widened it to composite actions.

## Measurements

Counting only non-blank,
non-comment lines,
measured 2026-10-29 with a scratch script:

- 230 `run:` steps in total;
  133 have more than 2 code lines,
  across 13 of the 20 workflow files.
- 68 steps contain a single `|` or `&&` outside `${{ }}`:
  59 with a pipe and 24 with `&&`,
  across 11 files.
- `shell:` values:
  56 `bash`,
  5 `node {0}`,
  1 `pwsh`,
  none an expression.
- `.github/actions` holds one file,
  `task-cli-setup/action.yml`.
- `run:` value forms across the workflows and that action,
  by a scratch script:
  147 block literal (`|`) and 86 plain single-line.
  No folded,
  quoted,
  multi-line plain,
  anchor,
  alias,
  merge key,
  tag,
  flow-style steps or multi-document file.
  `defaults.run.shell` (a mapping named `run`,
  not a step) occurs once,
  at
  `.github/workflows/music-player-release.yml:1539`,
  so the extractor must tell a step's `run` from it.

- **`run:` forms read:**
  block literal (`run: |`) and plain single-line only.
  Every other form is a `core/processing-failure` at the value.
- **Single `&`:**
  its own rule,
  `shell/no-ampersand`
  (cmd command separator,
  POSIX background operator).
  The owner chose this over folding it into a separator rule.

## Adopted without a question

These follow from the answers above and stay open to veto.

- **Rule set:**
  `shell/max-lines` plus one rule per construct:
  `shell/no-pipe`,
  `shell/no-and-chain`,
  `shell/no-or-chain`,
  `shell/no-pipe-stderr`,
  `shell/no-semicolon`,
  `shell/no-dollar-paren`,
  `shell/no-backtick` and `shell/no-ampersand`.
  All are `error`.
  The per-dialect table decides which constructs each dialect is scanned for.
- **Dialect from `shell:`:**
  the first word of the value,
  without directory or `.exe`,
  names the dialect.
  Dialects in scope:
  bash,
  sh,
  dash,
  ash,
  ksh,
  zsh,
  fish,
  pwsh,
  powershell and cmd.
  Any other first word (`node`,
  `python`,
  `perl`) is out of scope,
  so its step is not checked.
- **Effective shell:**
  the step's `shell:`,
  else the job's `defaults.run.shell`,
  else the workflow's `defaults.run.shell`,
  else an in-scope default shell.
  The runner is never resolved,
  because every runner's default shell is in scope.
- **Composite actions:**
  `shell:` is required there;
  a step without it is a `core/processing-failure`.
- **Ratchet location:**
  two nested configurations,
  `.github/workflows/monochromatic-lint.config.jsonc` and `.github/actions/monochromatic-lint.config.jsonc`,
  not the root configuration.
  The nearest configuration governs a file,
  so the root one does not apply under them,
  and the pinned binary never plans a `.yml` file,
  so it never loads them.
  A root-configuration entry would instead make the pinned binary reject the unknown `shell/` rules
  and fail every lint run until a release is pinned.
  Both directories hold only YAML;
  the one Markdown file,
  `.github/COPILOT_REVIEW.md`,
  sits above them.
  The blocks were generated once from the linter's own findings,
  and each is removed once its file passes the default budgets.
  Existing offending workflows are not rewritten by this work.
- **Done means:**
  a `README.md` section,
  tests for every branch of the extractor and each rule
  (including the processing-failure forms and adversarial quoting),
  a guard test shown to fail with its guard removed,
  and a run over the real workflows whose per-file counts match the numbers in Measurements.

## Settled while implementing

These follow from the design and from reading the linter source,
and stay open to veto.

- **Directives:**
  the nine rules are declared unsuppressible in the rule listing,
  and a workflow has no directive form at all (its directive scan returns nothing),
  as MDX has none.
  This replaces the earlier wording that a directive in a run body would itself be reported.
- **Tokenization:**
  the longest match wins for `&&`,
  `||` and `|&`.
  `>&`,
  `<&`,
  `&>`,
  `&>>` and `>|` are redirections,
  not operators.
  `$((` is arithmetic,
  not `$(`.
  A here-document body is data up to its end word,
  and its lines still count toward `shell/max-lines`.
  In PowerShell a leading `&` is the call operator,
  and a here-string is data except `$(` inside `@"`.
- **`shell:` first word:**
  the word after `env` (and its options and `NAME=value` words) names the interpreter,
  so `env bash {0}` is bash.
  A quoted first word may contain spaces.
  Listed non-shell interpreters (`node`,
  `python`,
  `deno`,
  `bun`,
  `perl`,
  `ruby`,
  `php`,
  `lua`,
  `julia`
  and similar) are out of scope;
  any other word is a processing failure.
- **No `shell:`:**
  a job whose `runs-on` text contains `windows` defaults to PowerShell,
  and every other job (including an expression such as `${{ matrix.runner }}`) defaults to a POSIX shell.
  An expression runner is therefore scanned as POSIX,
  which can over-report a PowerShell backtick but not under-report.
- **Hosts:**
  a path is a host only when it is `.github/workflows/<file>.yml` or `.yaml`
  (direct children),
  or `action.yml` or `action.yaml` anywhere under `.github/actions`.
  Other YAML is never read,
  so a stray `run:` key elsewhere cannot fail a run.
- **Virtual file numbering:**
  the number in `<workflow>/<n>.<suffix>` counts the extracted steps
  in file order,
  not the line,
  as for fenced Rust.
- **Dialect sources** (fetched 2026-10-29):
  the fish language manual lists `|`,
  `&&`,
  `||`,
  `;`,
  `$()`,
  the `|&` alias and a trailing `&` for background jobs,
  and does not mention backticks;
  the PowerShell `about_Pipeline_Chain_Operators` page says `&&` and `||` begin in PowerShell 7,
  and `about_Operators` documents `$()`,
  the call operator `&`,
  the background operator `&` and `|`;
  the `cmd` command reference documents `|`,
  `||`,
  `&&` and `&` as the chaining and piping operators.
  Each rule's dialect table in `package/linter/monochromatic-lint/src/shell_dialect.rs` follows these pages.
- **Counts:**
  the ratchet values come from the linter's own run over `.github`,
  not from the scratch script,
  because the script counted raw lines and a regex for pipes.
  The Measurements numbers stay a sanity bound,
  and each difference is explained when the ratchet is generated.

## Results

Measured on 2026-10-29 with the debug build of the implementation.

- The linter reads all 20 workflows and the one composite action with no processing failure.
- Steps over 2 code lines: 133.
  This equals the scratch count by coincidence of two offsetting differences:
  the 5 steps under `shell: node {0}` (in `kotlin-linter-publish.yml`,
  `toml-edit-fuzz.yml`
  and `cache-prune.yml`) are out of scope,
  and the 5 steps of the composite action are in scope.
- Files with a step over 2 lines:
  12,
  including the composite action.
  The scratch script counted 13 workflow files,
  because it counted the node-shell steps.
- Files with any banned construct: 10.
  Per-rule file counts:
  `shell/max-lines` 12,
  `shell/no-semicolon` 8,
  `shell/no-pipe` 8,
  `shell/no-dollar-paren` 9,
  `shell/no-or-chain` 6,
  `shell/no-and-chain` 3,
  `shell/no-ampersand` 1.
  `shell/no-pipe-stderr` and `shell/no-backtick` have no occurrence.
- The ratchet is 13 blocks:
  12 in the workflows configuration and 1 in the actions configuration.
- The first run refused `music-player-release.yml` at line 306,
  because the build matrix writes `- { os: linux, … }` flow mappings.
  The refusal of unread key forms now applies only to a job id,
  a step key or a structural key,
  and a test covers the matrix form.
- Checks run:
  the whole package suite (693 tests pass),
  `lint:types` (clippy with warnings denied),
  `lint:rustfmt`,
  the linter's own Rust rules over its sources (no finding after two files were split under the line limit),
  and the pinned binary over `.github` (exit 0,
  unaffected).
- Negative controls in a scratch copy:
  a new workflow with a three-line step containing `&&` is reported for
  `shell/max-lines` and `shell/no-and-chain`,
  and a third pipe added to a ratcheted workflow is reported.
- Guard check:
  with the path guard in `is_actions_yaml` removed,
  `other_yaml_is_never_read`,
  `only_workflows_and_composite_actions_are_yaml_hosts`
  and a discovery test fail;
  with it restored they pass.

## Enforcement starts at the next pin

`toolchain.lock.json` pins `cargo:monochromatic-lint` at the published 0.3.0,
which does not read YAML,
and `package/linter/monochromatic-lint/Cargo.toml` is bumped to 0.4.0 (a new feature in a 0.x crate),
together with its fuzz crate's `=0.4.0` dependency and both lockfiles,
but unpublished.
The lockfiles were refreshed by the new `lock:version` task,
which updates only the crate's own entry;
the existing `lock` task re-resolves the whole dependency graph offline and changed several hundred lockfile lines,
so its output was discarded (saved as a patch in the session scratch directory,
not kept in the repository).
Until a release that contains them is published and pinned (`toolchain:update`,
whose lock diff is the review),
the repository's lint run does not check workflows.
Publishing and pinning are the repository owner's steps and were not done.
After the pin moves,
`mise run lint:monochromatic-lint .github` enforces the ban.

## Open

None for the design.
The release and pin above are the remaining steps.
