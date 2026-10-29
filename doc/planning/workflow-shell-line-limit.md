# Workflow shell line limit

## Status

Grilling finished and the repository owner confirmed the design on 2026-10-29.
Nothing is implemented,
and the owner has not yet asked for implementation.
This records the decisions the repository owner has answered and the defaults adopted from them.

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
  across 13 of the 19 workflow files.
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
  blocks in the root `monochromatic-lint.config.jsonc`,
  generated once from a measurement of each file,
  then removed as a file is fixed.
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

## Open

None.
The owner confirmed the design is complete.
