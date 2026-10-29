# OCR GitHub Issue adapter handover

> Moved 2026-10-29 by the generic package category elimination
> (`doc/planning/generic-category-elimination.md`);
> paths and names below this note are historical:
> `@monochromatic-dev/module-logger` is now `@monochromatic-dev/logger-core`;
> `@monochromatic-dev/module-test` is now `@monochromatic-dev/test-harness-core`.

> Moved 2026-10-29 by the generic package category elimination
> (`doc/planning/generic-category-elimination.md`);
> paths and names below this note are historical:
> `package/cli/open-code-review-issue` is now `package/open-code-review-issue/cli`;
> `@monochromatic-dev/cli-open-code-review-issue` is now `@monochromatic-dev/open-code-review-issue-cli`.

## Status

The local adapter is implemented at `package/cli/open-code-review-issue`.
It builds the `open-code-review-issue` binary and exports its tested library seams.
The approved direct dependencies are installed through the pnpm catalog and package manifest.

The user requested and completed a design grilling before implementation.
Every identified design branch is resolved.
The user confirmed shared understanding and authorized implementation on 2026-08-16.
Follow the `grilling` skill:
ask one decision question at a time,
provide a recommended answer,
and do not enact the design until the user confirms shared understanding.

Keep this handover current as grilling settles decisions and as implementation progresses.

## User requirements

The adapter must:

- use TypeScript;
- support an interactive mode;
- support a non-interactive mode;
- study `pnpm update -r -i --no-save` as the interactive UX precedent;
- prefer the libraries pnpm uses for that interaction;
- ask the user separately before adding each new workspace dependency;
- accept OCR structured output from one required positional argument;
- allow interactive positional input as shell-quoted JSON or a named file;
- accept a JSONL file stored by OCR;
- implement the previously selected local-adapter architecture.

Human-readable terminal output,
piped standard input,
and TTY-pasted standard input are excluded from ingestion.

## Settled grilling decisions

### Adapter responsibility

The user chose an ingest-only adapter.
It must not invoke OCR.
Users run OCR separately,
then provide its structured output as interactive shell-quoted JSON or through a supported named file.

This removes OCR argument forwarding,
process signaling,
and OCR exit-code propagation from the adapter's scope.
The adapter owns parsing,
validation,
interactive triage,
security disclosure gating,
quarantine,
and GitHub issue creation.
It is create-only.
It must not synthesize persistent finding identities,
search older issues for cross-run deduplication,
suppress repeated-run duplicates,
update issues,
or reopen issues.
Repeated ingestion may create duplicate issues.
The only matching query is attempt reconciliation after an ambiguous create failure,
limited to new repository Issue or pull request numbers above a pre-request high-water mark.

### Structured input

The adapter must not parse OCR's human-readable terminal format.
This rule applies to interactive and non-interactive modes.
ANSI stripping and version-specific text parsing are out of scope.

Interactive shell-quoted JSON or a required named file accepts these OCR-native structured shapes:

- complete `ocr review --format json` or `ocr scan --format json` result object;
- bare comment array from `ocr session comments --json`;
- raw OCR session JSONL transcript from a named file.

Standard input is never an ingestion source.
A malformed or unsupported positional input reports its validation error and exits nonzero immediately.
If any record inside an otherwise recognized input has invalid types or structure,
the adapter must reject the entire input before performing any GitHub operation.
A finding is also invalid when OCR `content`,
`existing_code`,
and `suggestion_code` all lack a non-whitespace line.
That sparse record rejects the complete input under the same atomic rule.
It must not quarantine or skip malformed records and continue processing.
A structurally valid finding with absent classification metadata is not malformed under this rule;
the security policy still quarantines it.
Named files must be strict UTF-8 without a byte-order mark.
Malformed UTF-8 and a leading UTF-8 or UTF-16 byte-order mark reject the entire input before any GitHub operation.
The adapter must not perform lossy decoding or encoding auto-detection.

It must auto-detect among only these validated schemas.
It must not search arbitrary nested JSON for comment-like objects or accept individual comment fragments by accident.
All accepted shapes normalize into one internal finding collection before policy or publication logic runs.

### Mode selection

Mode selection must always be explicit.
Interactive mode uses `--interactive` with short form `-i`,
matching the explicit pnpm precedent.
Non-interactive mode requires `--non-interactive`.
Providing neither mode flag is an error rather than an inferred default.
Providing both contradictory mode flags is also an error.
Non-interactive mode must never prompt,
regardless of TTY detection.
A missing required non-interactive input or decision is an error rather than an invitation to prompt.
Without `--apply`,
non-interactive mode validates and emits an exact JSON publication plan to standard output
without GitHub mutations.
The plan includes the destination repository,
complete generated titles and bodies for normal issues,
label or title-prefix behavior,
and source-reference behavior.
Security findings appear only as a count and input record ordinals or JSONL line numbers.
Diagnostics go to standard error so standard output remains machine-readable.
After a non-interactive applied run completes or stops on a handled terminal failure,
standard output contains one final JSON result object and no progress events.
It records the outcome,
destination repository,
created Issue numbers and URLs with source input positions,
withheld-security positions,
and any stopping failure with its input position.
It does not repeat finding titles or bodies.
A partial or failed publication returns status one;
a complete publication returns zero.
Invalid command-line syntax,
flags,
or arguments return status two.
Validation,
preflight,
authentication,
GitHub operations,
and other handled runtime failures return status one.
The first handled post-publication Ctrl+C also returns status one.
A forced second Ctrl+C returns status 130 and may prevent final output.
`--non-interactive --apply` explicitly authorizes creation only when no security-gated finding is present.
If one is present,
bare `--apply` errors without creating any issue.
`--apply --non-security-only` creates all eligible non-security issues and withholds security-gated findings.
`--apply --all` creates both non-security and security-gated issues;
its help and preview must state that it authorizes public disclosure of every included security finding.
`--apply` is invalid in interactive mode,
which uses its explicit post-selection confirmation as the mutation boundary.

No mode may consume piped standard input.
The user explicitly rejected pipes and TTY-pasted ingestion for this adapter.
Every mode requires exactly one positional input:

- interactive mode accepts shell-quoted JSON or a named file;
- non-interactive mode accepts only a named file;
- omitting input is invocation misuse with status two before any prompt;
- a nonexistent path is a handled runtime failure with status one;
- multiple positional values are invocation misuse;
- `-` must not mean standard input.

The implementation must never inspect or read redirected or piped standard input as an ingestion source.
Piped bytes are ignored rather than detected or consumed.
Missing or nonexistent input diagnostics show commands for generating JSON,
the common OCR session location,
and the most recently modified discovered session JSONL path when present.
Discovery scans only file metadata under `~/.opencodereview/sessions` and never invokes OCR or reads transcript content.
The implementation must not reopen `/dev/tty`,
`CONIN$`,
or another controlling-terminal device.
Interactive mode requires both standard input and standard output to be TTYs.
It checks both after command-line validation and before reading input or opening a prompt.
A failed TTY preflight is a handled runtime failure with status one.
Prompts and human-readable interactive results use standard output;
diagnostics use standard error.
Interactive mode does not support redirecting its human-readable output.

### Configuration

The user chose CLI-only adapter configuration.
The adapter defines no project-local configuration file,
user-global configuration file,
or adapter-specific environment variables.
Every configurable publication decision is visible in the current invocation's arguments and input.
The adapter may invoke Git and GitHub CLI,
which retain their own configuration and authentication behavior,
but it must not parse or add hidden defaults from those tools beyond the settled repository-inference rule.

### Package task surface

Repository conventions determine the package's mise task surface without another preference decision.
The package defines `run`,
`build`,
`build:js`,
`build:js:node`,
`watch:build`,
`watch:build:js`,
`watch:build:js:node`,
`lint`,
`lint:types`,
`lint:oxlint`,
`format:oxlint`,
`test:unit`,
`test:integration`,
and `buildAndTest`.
Unit tests cover pure parser,
normalization,
policy,
rendering,
and prompt paths.
Integration tests invoke the built binary with disposable filesystem fixtures and a fake `gh` executable.
`buildAndTest` uses mise's array form to run build,
unit tests,
and integration tests in sequence.
No routine mise task performs live GitHub mutations;
any live disposable-repository verification requires its separately settled authorization and safeguards.

### Repository selection

An explicit `--repo https://github.com/OWNER/NAME` selects the destination in either mode.
The flag accepts only that canonical HTTPS GitHub repository URL shape,
not `OWNER/NAME` shorthand.
Without that flag,
the adapter infers the repository only when the process working directory is exactly the Git worktree root.
When run from a subdirectory inside a worktree,
it errors and instructs the user to rerun at the root
or pass `--repo https://github.com/OWNER/NAME`.
Outside a Git worktree,
it errors and requires `--repo https://github.com/OWNER/NAME`.
A repository that cannot be identified unambiguously also errors and suggests the explicit flag.
These cases print diagnostics and exit rather than opening another prompt.

### GitHub API boundary

The user chose `gh api` as the sole GitHub authentication and HTTP boundary.
The adapter requires GitHub CLI 2.97.0 or newer,
installed and authenticated.
Before its first `gh api` operation,
it parses `gh --version` and rejects a missing,
unparseable,
or older CLI as a handled runtime failure with status one.
The minimum matches the source-audited implementation;
the adapter does not accept older versions through capability probing or require an exact version pin.
It uses existing GitHub CLI credentials without extracting or accepting a token,
and invokes GitHub REST endpoints through non-paginated `gh api` subprocesses.
It must not use `gh issue create`,
a GitHub client library,
or direct authenticated `fetch` calls.

JSON request bodies use private named temporary files passed through `--input`.
Neither adapter input nor GitHub request bodies pass through standard input or process arguments.
The `gh` subprocess receives no inherited standard input.
The adapter launches `gh` through approved direct dependency `nano-spawn` 2.1.0 or newer,
with standard input set to `ignore` and both output streams explicitly set to `pipe`.
It requests response status and headers through `--include`,
parses those plus the response JSON,
and owns the explicit scheduling,
rate-limit delay,
ambiguous-failure reconciliation,
and bounded-retry policy.
Every `gh api` child receives a fixed one-minute deadline with no user override.
Each invocation receives a fresh `AbortSignal.timeout(60_000)` and `killSignal: 'SIGKILL'` through nano-spawn.
It passes no nano-spawn `timeout` option and shares no publication-interrupt signal with the deadline.
It must not rely on nano-spawn's `timeout` option or on an abort with default `SIGTERM`:
the first can resolve after its limit when a child handles termination,
and the second can reject while leaving that child alive.
The source trace and bounded reproduction are recorded in
[`doc/troubleshooting/nano-spawn-timeout-is-not-a-hard-deadline.md`][nano-spawn-timeout].
A timed-out creation is ambiguous and enters the settled reconciliation path;
a timed-out read-only operation fails its owning preflight or reconciliation step.
Temporary request files must be inaccessible to other users and removed after each invocation.

### Live verification repository

The user selected a supplied disposable repository for end-user GitHub verification:
`https://github.com/Aquaticat/issues-api`.
Read-only preflight on 2026-08-16 verified that it is a private,
unarchived repository with Issues enabled,
default branch `main`,
and authenticated administrative permission.
It contained no Issues or pull requests and no `needs-triage` label.
That starting state directly supports verification of the settled title-prefix fallback.
The user authorizes synthetic Issue creation there during implementation verification.
The adapter must not target another live repository for tests.
A separate verification harness closes every synthetic Issue it created after assertions complete.
It patches only the Issue numbers recorded from that verification run,
sets state to `closed` with reason `completed`,
and never adds cleanup behavior to the adapter.
Cleanup runs after successful assertions and after handled assertion failures when created numbers are known.
A closure failure fails verification and reports every affected URL for manual cleanup.
Closed Issue history and repository audit events remain.

### Issue rendering

Issue titles use the deterministic shape `[category] path: summary`.
The category is the normalized OCR category or `uncategorized` when absent.
The path comes from the OCR finding.
The summary is the first meaningful line of OCR content.
When content has no non-whitespace line,
the summary instead uses the first non-whitespace line of OCR `existing_code`,
trimmed of surrounding whitespace but otherwise unchanged.
If both content and `existing_code` lack a non-whitespace line,
the summary uses the first non-whitespace `suggestion_code` line under the same trimming rule.
These fallbacks can expose existing or proposed source code or secrets in the Issue title and notification surfaces;
help,
previews,
and publication confirmations must warn about that behavior.
When all three fields lack a non-whitespace line,
the finding is invalid and rejects the complete input before GitHub operations.
Issue bodies include both OCR `existing_code` and `suggestion_code` in separate,
clearly labeled code sections when those fields are present.
Each code section uses a GitHub Flavored Markdown indented code block:
a blank line follows the heading,
and every source line receives four leading spaces.
This keeps backticks,
tildes,
HTML,
and other Markdown literal without delimiter selection or HTML escaping.
The sections provide no inferred language tag or syntax-highlighting hint.
Model `thinking` remains excluded.
Source references always include the repository-relative OCR path and line range.
When the accepted input supplies a resolved head commit
and the adapter verifies that commit in the destination repository,
the source reference is a commit-pinned hyperlink.
Without that verified provenance,
it remains plain text rather than linking a mutable branch.
Every issue uses the existing `needs-triage` label when the destination defines it.
When that label is confirmed absent,
the adapter silently omits the label and prepends `[needs-triage] ` to the issue title instead.
It supports no additional label options and never creates labels.
A label-lookup failure is not proof of absence and remains an error before creation.

Issue bodies use this section order:

1. `Finding`,
   containing OCR content.
2. `Source`,
   containing the verified permalink or plain path and line range,
   followed by category and severity metadata.
3. `Existing code`,
   when OCR supplied `existing_code`.
4. `Suggested code`,
   when OCR supplied `suggestion_code`.
5. A plain attribution that the issue was generated from OpenCodeReview output.

Missing category renders as `uncategorized` and missing severity as `unspecified`.
Absent code fields omit their sections.
Model `thinking` and raw record JSON are never included.
OCR `content` is inserted as Markdown unchanged.
The adapter must not sanitize,
escape,
demote,
contain,
or neutralize its headings,
fences,
HTML,
links,
or mentions.
Documentation and publication previews must state that OCR content retains active GitHub Markdown behavior,
including notifications caused by mentions.
In interactive mode,
the normal picker and final batch summary show normal issue titles rather than full bodies.
Before confirming each selected security finding,
the adapter displays its complete generated title and Markdown body.
GitHub's current REST documentation and OpenAPI schema publish no title-length maximum or counting unit;
see
[`doc/troubleshooting/github-issue-title-length.md`][github-issue-title-length].
The adapter owns a conservative 256-byte UTF-8 cap on the final title,
including a `[needs-triage] ` fallback prefix when present.
A title within the cap remains unchanged.
For an overlength title,
it retains the longest valid UTF-8 prefix fitting within 253 bytes,
trims trailing whitespace,
and appends the three-byte `…` character.
The complete finding remains in the body.
This is an adapter contract,
not a documented GitHub limit.

## Settled prior findings

The installed command is OpenCodeReview `v1.9.4`,
commit `f7344e79`.

OCR has no native regular GitHub Issue publisher.
Its Action calls pull request review and issue-comment APIs,
and `route_categories` only moves findings into the pull request summary.
The source audit and integration options are recorded in
[`doc/troubleshooting/open-code-review-github-issue-routing.md`][ocr-routing].

The user selected the local adapter over the private-intake Action and direct-public Action alternatives.
The intended boundary is a separate adapter command,
not an alias that shadows `ocr`,
not a write-capable OCR MCP tool,
and not SARIF.

## Security constraints

This repository is public.
[`SECURITY.md`](../../SECURITY.md) requires suspected vulnerabilities to use private vulnerability reporting
and forbids opening a public issue for a suspected vulnerability.
An OCR security classification does not by itself prove that a finding describes a currently exploitable
or unresolved vulnerability.
A finding may already be fixed and may be published later for accountability or as a durable record.
The adapter must explain this distinction and its disclosure behavior in every relevant CLI help entry,
prompt,
preview,
error,
and package document.

Interactive mode places security-gated findings in a separate picker from other findings.
The normal picker initially selects every finding.
The security picker initially selects no finding.
Both use square checkbox indicators rather than circular radio-style indicators:
`☐` for unchecked and `☑` for checked.
Disabled variants,
if rendered,
retain the same square shapes with disabled styling.
The normal picker retains Inquirer's select-all and invert shortcuts.
The security picker disables both shortcuts,
so each security selection requires direct navigation and toggling.
After both picker stages,
the combined selection must contain at least one finding.
An empty combined selection keeps the selection flow active until the user selects a finding or cancels explicitly.
Either individual picker may be empty,
so security-only publication remains possible.
Pressing Ctrl+C during any prompt or answering No at the final confirmation before creation starts
prints `Issue creation canceled.` and returns status zero without GitHub mutations.
After creation starts,
interactive mode prints human-readable progress and ends with a human-readable list of created Issue URLs.
A handled partial failure also identifies the stopping failure and returns nonzero.
After creation starts,
the first Ctrl+C prevents every later creation but allows an active `gh api` creation to finish
within its normal request timeout.
An ambiguous completion still receives the settled read-only reconciliation,
with no retry after the interrupt.
If only pacing or retry delay is active,
the first Ctrl+C stops that wait immediately.
The adapter then emits the partial applied-run result and returns nonzero.
A second Ctrl+C terminates immediately and may prevent final output.
That picker must use red styling when color is available and an explicit textual `SECURITY` marker
so color is never the only signal.
Custom styles use Node's native `styleText()` stream validation against standard output.
They honor terminal capability,
`NO_COLOR`,
`NODE_DISABLE_COLORS`,
and `FORCE_COLOR` rather than defining an adapter-specific color flag.
A Node probe verified plain text under both disable variables and ANSI red under `FORCE_COLOR`.
Square `☐` and `☑` indicators and textual markers retain state and security meaning when color is disabled.
Every security-gated finding selected for publication requires its own explicit safe-to-disclose confirmation.
The confirmation has no default;
empty input reprompts.
Declining confirmation withholds that finding without approving its publication.
Withheld security findings are not persisted to another file or destination.
The adapter reports only their count and input record ordinals or JSONL line numbers,
never their titles,
paths,
code,
or content.
The original named OCR input remains the authoritative retained copy.

Non-interactive mode uses an explicit authority ladder:

- bare `--apply` errors before all GitHub creation when any security-gated finding is present;
- `--apply --non-security-only` creates eligible non-security issues and withholds security-gated findings;
- `--apply --all` includes security-gated issues and explicitly represents that they are safe for public disclosure.

A finding is security-gated only when its normalized OCR category is exactly `security`.
No content scanner,
secret detector,
severity value,
missing metadata,
or other category may place a finding in the security gate.
Consequently,
`--non-security-only` excludes exactly the `security` category,
and `--all` adds exactly that category to otherwise eligible publication.
The explicit OCR category `other` is an ordinary publishable candidate.
Interactive lists and previews must visibly mark it as `OTHER`,
but it receives no additional confirmation or non-interactive authority gate.
A structurally valid finding with missing category metadata is also an ordinary publishable candidate.
Interactive lists and previews must visibly mark it as `UNCATEGORIZED`.
Missing metadata does not add a confirmation or non-interactive authority gate.

The publication boundary also requires that:

- model-provided `thinking` must never be published;
- `existing_code` and `suggestion_code` require deliberate treatment
  because they can contain secrets or exploit details;
- model category alone is not a safe disclosure boundary;
- generated public issues should begin with `needs-triage`,
  not `ready-for-agent`;
- omission from a later OCR run must not automatically close an issue.

Grilling can revise the candidate policy only through an explicit user decision.
Do not silently weaken it during implementation.
Actual unresolved suspected vulnerabilities remain private under `SECURITY.md`;
a publication confirmation or flag is an assertion that selected material is safe for disclosure,
not permission to publish an unresolved vulnerability.

## Verified technical evidence

The installed OCR binary supports:

```text
ocr review --format json --audience agent
```

A disposable empty repository verified that the installed artifact emits a top-level JSON object
with a `comments` array.
The source model defines comment fields including:

- `path`;
- `content`;
- `suggestion_code`;
- `existing_code`;
- `start_line`;
- `end_line`;
- `thinking`;
- `category`;
- `severity`.

Individual comment records have no source commit or repository URL.
Complete JSON output and completed JSONL transcripts can carry a run manifest with a resolved head commit,
but bare comment arrays do not.
The manifest stores repository identity only as a SHA-256 value rather than a repository URL.

OCR also persists sessions and exposes `ocr session comments --json`.
Source inspection confirms that session JSONL is an event transcript,
not final review output repeated one object per line.
Findings are embedded in `comments` arrays on `review_item_done` and `review_item_reused` records.
A later checkpoint with the same fingerprint supersedes the earlier checkpoint,
and a later `review_item_failed` record for that fingerprint removes its findings.
The final `session_end` record can contain `run_manifest`;
a killed run may leave a partial transcript without `session_end`.
Other event types can contain prompts,
code,
tool calls,
and model output and must not be treated as findings.

A disposable classifier prototype under the agent scratch directory
verified the previously proposed fail-closed partition against synthetic findings.
That prototype is evidence only,
not production code and not part of this repository.

The installed GitHub CLI is `gh` 2.97.0,
release commit `55dbb4dc6b7edb10b48e3d7fc5bccd32318d1b55`.
Its `gh api` command delegates authentication to GitHub CLI configuration
or the documented token environment variables,
accepts a named JSON request file through `--input`,
and can include response status and headers with `--include`.
The adapter would never use either option's `-` standard-input form.
Source inspection of `pkg/cmd/api/api.go` and `pkg/cmd/api/http.go`
found one `client.Do` path per non-paginated invocation and no API-command retry loop.
The underlying `cli/go-gh` 2.13.0 client uses the Go default transport rather than a retry transport.
Its client options document no timeout as the default,
and `gh api` 2.97.0 supplies no timeout value or timeout flag.
The adapter therefore applies a fixed one-minute child-process deadline to every GitHub CLI invocation,
with no timeout flag or configuration override.
The workspace already invokes `gh api` from active packages and has no direct Octokit dependency.
The user selected this boundary instead of a GitHub client library or direct authenticated HTTP.

GitHub REST API documentation version 2026-03-10
and `github/rest-api-description` commit `67c14c7efb01cdeeac0ecd8cee9fae8d7a80e2aa`
require a Create an issue title but specify no `minLength`,
`maxLength`,
or counting unit.
A community web-interface report mentions 256 characters,
but no authorized disposable API mutation verified that undocumented boundary.

The published GitHub Flavored Markdown specification is version 0.29-gfm.
Its indented-code-block rule removes four leading spaces from each non-blank line
and treats the remaining contents literally.
Its fenced-code-block rule instead closes on a same-character fence
at least as long as the opening fence.
No rendering choice has been made from this evidence yet.

The pnpm `v11.8.0` source tag resolves to commit `93458600a8498412f85316d054e033319ba31ed6`.
Its `installing/commands/src/update/index.ts` implementation uses `checkbox` and `Separator`
from `@inquirer/prompts` for `pnpm update -i`.
The prompt groups choices with separators,
uses a terminal-dependent page size,
requires at least one choice,
shows explicit selection and cancellation instructions,
uses custom checked,
unchecked,
and cursor icons,
enables vim keybindings,
and catches Inquirer's `ExitPromptError` as a clean cancellation.
A completed pairwise vet now compares `@clack/prompts` 1.7.0
with `@inquirer/prompts` 8.5.2 in
[`doc/audit/tech-clack-versus-inquirer-for-a-standalone-type-vet-2026-08-16.md`][prompt-vet].
The no-context equal-weight baseline places Inquirer first,
but dependency-surface weighting reverses the order,
so the formal context-free result is conditional.
The adapter's known requirements supply the deciding context:
Inquirer provides per-prompt themes for the red security picker,
preselected checkbox choices,
custom streams,
and the pnpm interaction precedent.
Clack's high-level active and selected colors are fixed in version 1.7.0,
and its custom-stream path still reads global stdout width.
Both exact packages passed the adapter-focused consumer fixture.
Inquirer's complete CI-equivalent,
ESM,
and CommonJS validation passed.
Clack's target prompt suites passed,
but its full suite exposed a supported-runtime nested-color mismatch documented in
[`doc/troubleshooting/clack-note-nested-styletext-node-floor.md`][clack-note-node-floor].

The recommendation is Inquirer 8.5.2 for this adapter.
The user approved Inquirer on 2026-08-16,
with the direction to cherry-pick only necessary package parts.
Treat this as approval of the Inquirer technology family,
not approval to install its umbrella package unnecessarily.
The completed subpackage audit proposes only:

- `@inquirer/checkbox` 5.2.1 for both finding pickers;
- `@inquirer/input` 5.1.2 for validated explicit decisions.

Checkbox re-exports `Separator`,
so no direct core dependency is needed.
The direct pair installs 12 package nodes;
input contributes only one node beyond checkbox's graph.
A direct-package fixture passed custom streams,
preselection,
separators,
red security theming,
validation,
Ctrl+C,
and cleanup.
The umbrella `@inquirer/prompts` package is rejected because it adds unused prompt and editor paths.
The user separately approved `@inquirer/checkbox` 5.2.1
and `@inquirer/input` 5.1.2 on 2026-08-16.

CLI packages under `package/cli/<name>` use package names shaped
`@monochromatic-dev/cli-<name>` and usually expose an unscoped `<name>` binary.
No repository text or executable on the current `PATH` uses
`ocr-github-issue`,
`ocr-issue`,
or `open-code-review-issue`.
No package naming decision has been made from this evidence yet.

## Implementation and verification result

The package implements the complete settled contract:

- strict complete-result,
  comment-array,
  and JSONL ingestion from named files plus interactive shell-quoted JSON;
- missing-input generation guidance and latest persisted-session discovery;
- atomic validation and strict BOM-free UTF-8 named files;
- deterministic titles,
  bodies,
  category markers,
  source references,
  and security-redacted plans;
- direct Inquirer checkbox and input prompts with red `SECURITY` styling,
  square `☐` and `☑` indicators,
  and disabled security bulk shortcuts;
- bounded `gh api --include` process requests with private request files;
- serial pacing,
  retry classes,
  high-water reconciliation,
  and positioned partial results;
- CLI-only explicit modes,
  compact exit statuses,
  and two-stage publication interruption.

Verification completed on 2026-08-16:

- `mise run //package/cli/open-code-review-issue:buildAndTest` passed all unit and fake-`gh` integration paths;
- `mise run //package/cli/open-code-review-issue:lint:types` passed;
- `mise run //package/cli/open-code-review-issue:lint:oxlint` reported zero warnings and zero errors;
- package and design Markdown lint passed;
- the built CLI emitted separate library and executable bundles with the required Node shebang;
- PTY-driven ordinary and security flows exercised the actual Inquirer rendering,
  explicit decisions,
  red security picker,
  square checkbox states,
  absent security bulk shortcuts,
  and final fake publication;
- final live verification created
  [`Aquaticat/issues-api#2`](https://github.com/Aquaticat/issues-api/issues/2),
  verified its exact generated title and body,
  then the separate harness closed it with reason `completed`.

The disposable repository was empty and had no `needs-triage` label,
so final live verification exercised plain source references and the title-prefix label fallback.
Commit-pinned source links and existing-label behavior remain covered by artifact-importing unit tests.

## Repository state and commits

The capability investigation and troubleshooting document landed in:

```text
cddac7bd6 docs(*): document OCR issue routing
```

At handover creation,
`main` and `origin/main` had advanced to:

```text
7a20add13 docs(kotlin-linter): record signing key rotation
```

The worktree was clean before this handover was created.

## Dependency approval gate

No new dependency may be added to the workspace until all of these steps occur for that dependency:

1. Inspect current pnpm source and package metadata to identify what pnpm uses for the relevant interaction.
2. Inspect this workspace for an existing dependency or utility that already meets the need.
3. Present the proposed dependency to the user with its role,
   alternatives,
   pros,
   cons,
   and personal ranking.
4. Wait for explicit approval for that dependency.
5. Add it through the pnpm catalog and package manifest using repository dependency policy.

Approval for one external dependency does not approve another.
A transitive dependency does not require separate approval unless the implementation proposes making it direct.
On 2026-08-16,
the user granted standing approval for any workspace-owned dependency.
Those packages no longer require individual approval,
but each must still have a concrete role and remain direct only when imported by this package.
Record each external approval or rejection and every selected workspace-owned dependency in this handover.

The user initially approved `p-limit` as a direct adapter dependency for bounded issue creation.
The workspace already catalogs `p-limit` at `>=7.3.1`,
and multiple active packages consume it through `catalog:`.
Inspection of
`node_modules/.pnpm/p-limit@7.3.1/node_modules/p-limit/index.js`
and its adjacent `index.d.ts` confirms a concurrency cap and `clearQueue()`.
With `rejectOnClear: true`,
clearing rejects queued tasks with `AbortError`,
but it cannot cancel tasks already running.

The user separately approved these external runtime dependencies:

- `@inquirer/checkbox` 5.2.1;
- `@inquirer/input` 5.1.2;
- `nano-spawn` 2.1.0 or newer through the existing catalog entry.

The adapter uses these workspace-owned direct packages:

- `@monochromatic-dev/module-logger` at runtime for tagged lifecycle,
  retry,
  interruption,
  and error logging;
- `@monochromatic-dev/config-rolldown` for the Node build;
- `@monochromatic-dev/config-typescript` for type checking;
- `@monochromatic-dev/module-test` for unit and integration tests.

Logger calls must never contain finding content,
titles,
bodies,
code,
paths,
or secrets.
They may record branch names,
counts,
input positions,
HTTP status classes,
and retry attempts.
Lifecycle logs use debug level so non-interactive standard output remains the single settled JSON object.

No direct timing package is needed.
Abortable pacing and retry delays use `node:timers/promises` with the publication interrupt signal.
Nano-spawn owns subprocess promise and output capture behavior;
Node `AbortSignal` supplies its forceful one-minute deadline.
The user will not approve another subprocess dependency.
The verified composition force-terminated the direct child,
preserved partial stdout and stderr,
and allowed a successful child to exit without waiting for its unused deadline timer.
If nano-spawn fails a required supported-platform behavior despite that composition,
the only fallback is direct `node:child_process` use.
No direct command-line parser is needed because Node's argument-parsing API supplies that boundary.
Additional workspace-owned packages may become direct only if implementation imports them for a demonstrated role.

The user then selected provider-aligned serial creation after reviewing
[GitHub's current REST best practices][github-rest-best-practices].
Issue creation runs one request at a time and waits at least one second between mutative requests.
The adapter therefore will not add or use `p-limit`.
No manifest or lockfile change has been made.
The source trace and scheduling rationale are recorded in
[`doc/troubleshooting/github-issue-creation-concurrency.md`][github-issue-concurrency].

## Resolved design tree pending confirmation

The following branches were resolved one at a time in dependency order:

1. Command responsibility:
   settled as ingest-only.
   The adapter never launches OCR.
2. Publication timing:
   interactive mode shows one final summary after selection,
   then requires an explicit yes or no before GitHub mutations.
   Empty input has no default and reprompts.
   The summary names the destination repository and each selected issue title.
   Every selected operation is creation.
   It reports quarantined findings only as a count and never displays their sensitive content.
3. Input contracts:
   Accepted shapes are the complete review or scan result object,
   `ocr session comments --json` comment array,
   and raw OCR session JSONL transcript from a named file.
   Every mode requires exactly one positional input.
   Interactive mode accepts shell-quoted JSON or a named file;
   non-interactive mode accepts only a named file.
   Omitting input or supplying multiple values is invocation misuse with status two before any prompt.
   `-` never means standard input.
   Redirected,
   piped,
   and TTY-pasted standard input are never inspected or read as ingestion.
   Interactive terminal prompts use TTY standard input only for finding selection and explicit decisions.
   Missing and nonexistent input diagnostics show generation commands,
   the common persisted-session path,
   and a specifically discovered latest JSONL path when present.
   A malformed record rejects the entire input before any GitHub operation.
   Structurally valid findings with missing category metadata remain ordinary publishable candidates.
   Named files require strict UTF-8 without any byte-order mark;
   malformed bytes or a byte-order mark reject the input before GitHub operations.
4. Non-interactive authority:
   mode selection always requires exactly one explicit flag:
   `--interactive` or `-i`,
   or `--non-interactive`.
   Neither or both is an error.
   `--non-interactive` without `--apply` validates and prints a preview without GitHub mutations.
   Bare `--non-interactive --apply` errors before all creation if any security-gated finding exists.
   `--apply --non-security-only` creates only eligible non-security issues.
   `--apply --all` includes security-gated issues under an explicit safe-disclosure assertion.
   `--apply` is invalid with interactive mode,
   whose post-selection yes-or-no confirmation is its mutation boundary.
   Preview mode emits an exact JSON publication plan to standard output.
   It contains complete normal issue titles and bodies,
   repository,
   label fallback,
   source-reference behavior,
   and only counts and input positions for security findings.
   Diagnostics use standard error.
   Non-interactive applied runs emit one final JSON result object after completion or a handled terminal failure,
   with no standard-output progress events.
   The result contains outcome,
   repository,
   created Issue numbers and URLs with input positions,
   withheld-security positions,
   and a positioned stopping failure without repeating titles or bodies.
   Interactive applied runs use human-readable progress and finish with created Issue URLs.
   Successful preview,
   successful publication,
   help,
   and pre-publication cancellation return status zero.
   Invalid command-line syntax,
   flags,
   or arguments return status two.
   Validation,
   preflight,
   authentication,
   GitHub operations,
   handled post-publication interruption,
   partial publication,
   and other runtime failures return status one.
   A forced second Ctrl+C returns status 130.
   Issue creation is serial with at least one second between mutative requests.
   Rate-limit rejections,
   network failures,
   timeouts,
   and `5xx` responses are retryable.
   Every retryable failure permits at most three retries after the initial request.
   Rate limits honor `retry-after` or `x-ratelimit-reset`;
   without either,
   delays start at sixty seconds and double.
   Network,
   timeout,
   and `5xx` delays start at one second and double.
   Before each initial create request,
   the adapter queries the greatest repository Issue or pull request number as a high-water mark.
   After an ambiguous failure,
   it queries every newer number and compares the exact generated title and body before retrying.
   One exact match proves the request result for adapter purposes and suppresses the retry.
   No match permits the next retry.
   More than one exact match is a terminal error:
   the adapter reports every matching URL,
   creates no later issue,
   and performs no automatic closure or cleanup.
   A failed reconciliation query stops processing rather than risking a blind retry.
   Retrying after a successful no-match reconciliation can still create a duplicate
   because GitHub documents no read-after-write consistency guarantee;
   the tool must state that residual risk.
   After a failure exhausts its retries,
   no later issue is attempted.
5. Security disclosure and quarantine:
   security-gated findings use a separate red and text-marked interactive picker.
   Each selected security finding requires an explicit confirmation with no default.
   The security gate contains only findings whose normalized OCR category is exactly `security`.
   Content signals,
   severity,
   `other`,
   and missing category metadata do not enter that gate.
   Explicit category `other` is an ordinary candidate visibly marked `OTHER`.
   Missing category metadata is an ordinary candidate visibly marked `UNCATEGORIZED`.
   Neither receives an additional authority gate.
   Withheld security findings are not persisted separately.
   Output reports only their count and input record ordinals or JSONL line numbers;
   users inspect the original OCR input.
6. Repository selection:
   explicit `--repo https://github.com/OWNER/NAME` overrides repository inference.
   The shorthand `OWNER/NAME` is invalid.
   Without `--repo`,
   the adapter infers the destination only when the process working directory is exactly a Git worktree root.
   From a subdirectory inside a worktree,
   it errors and instructs the user to rerun at the root
   or pass `--repo https://github.com/OWNER/NAME`.
   Outside a Git worktree,
   it errors and requires `--repo https://github.com/OWNER/NAME`.
   These are diagnostics followed by exit,
   not prompts;
   non-interactive mode never prompts.
   Failure or ambiguity while identifying the root repository also errors and suggests `--repo`.
7. Issue rendering:
   issue titles use `[category] path: summary`.
   `category` is the normalized OCR category or `uncategorized` when absent.
   `path` is the OCR finding path.
   `summary` is the first meaningful line of OCR content.
   When content has none,
   the first non-whitespace `existing_code` line becomes the summary after trimming surrounding whitespace.
   When neither has one,
   the first non-whitespace `suggestion_code` line becomes the summary under the same trimming rule.
   Interfaces warn that these fallbacks can expose existing or proposed code or secrets in titles and notifications.
   A finding for which all three fields lack a non-whitespace line is invalid
   and atomically rejects the complete input.
   Bodies include separate existing-code and suggested-code sections when OCR supplies those fields.
   Model `thinking` is always omitted.
   Source references always show path and line range.
   They become commit-pinned links only when input supplies a resolved head commit
   that is verified in the destination repository;
   otherwise they remain plain text.
   Every issue receives the existing `needs-triage` label when available.
   When it is confirmed absent,
   the adapter silently prepends `[needs-triage] ` to the generated title instead.
   It never creates labels and supports no additional label option.
   Body section structure is settled as Finding,
   Source with category and severity,
   optional Existing code,
   optional Suggested code,
   and OpenCodeReview attribution.
   Missing metadata uses `uncategorized` and `unspecified`.
   OCR content is inserted as unchanged Markdown with active headings,
   fences,
   HTML,
   links,
   and mentions.
   Interactive normal candidates and the final batch summary show titles only.
   Each selected security candidate shows its complete generated title and body
   before its individual disclosure confirmation.
   The final title,
   including a fallback `[needs-triage] ` prefix,
   has an adapter-owned 256-byte UTF-8 cap.
   Overlength titles retain the longest valid prefix within 253 bytes,
   trim trailing whitespace,
   and append `…`.
   Existing code and Suggested code use indented code blocks,
   with a blank line after the heading and four added spaces on every code line.
   They use no delimiter,
   inferred language tag,
   or syntax-highlighting hint.
8. Identity and lifecycle:
   settled as create-only.
   The adapter has no persistent synthetic identity,
   cross-run existing-issue lookup,
   repeated-run duplicate suppression,
   update,
   reopen,
   or automatic closure behavior.
   Repeated ingestion may create duplicates.
   Attempt reconciliation is the sole lookup exception:
   it scans only Issue or pull request numbers above a pre-request high-water mark
   after an ambiguous create failure and compares exact generated title and body.
   One match is success,
   no match allows retry,
   and multiple matches stop with every matching URL reported and no automatic cleanup.
9. Interactive mechanics:
   the normal picker initially selects all findings.
   The separate security picker initially selects none.
   Both use `☐` for unchecked and `☑` for checked rather than circular indicators.
   The normal picker retains select-all and invert shortcuts;
   the security picker disables both.
   The combined result must select at least one issue;
   otherwise selection remains active until a finding is selected or the user cancels.
   Either individual picker may be empty.
   Inquirer direct subpackages implement pnpm-style prompting.
   Post-creation interrupt and multi-select behavior are settled.
   Interactive mode requires TTY standard input and TTY standard output.
   Native Node color controls govern ANSI styling;
   square indicators and textual security markers prevent color-only meaning.
10. GitHub boundary and package interface:
    GitHub operations use non-paginated `gh api --include` subprocesses
    with private named request-body files and no inherited standard input.
    The adapter reuses GitHub CLI authentication and owns retry decisions.
    Direct authenticated HTTP,
    GitHub client libraries,
    and `gh issue create` are excluded.
    Every GitHub CLI invocation has a fixed one-minute child-process deadline with no user override.
    After publication starts,
    the first Ctrl+C stops future creation while allowing an active bounded creation to settle;
    a second Ctrl+C terminates immediately.
    The package lives at `package/cli/open-code-review-issue`,
    is named `@monochromatic-dev/cli-open-code-review-issue`,
    and exposes the `open-code-review-issue` binary.
    Configuration is CLI-only,
    with no project,
    user,
    or environment configuration surface.
    GitHub CLI compatibility requires parseable version 2.97.0 or newer.
    Mise tasks follow the settled package task surface,
    including unit and built-CLI integration suites.
11. Verification:
    parser fixtures,
    prompt interaction tests,
    mocked `gh` boundaries,
    and end-user CLI exercise.
    Live verification uses only the user-supplied private repository
    `https://github.com/Aquaticat/issues-api`.
    Read-only preflight confirmed administrative access,
    Issues enabled,
    an empty Issue history,
    and no `needs-triage` label.
    A separate harness closes only the synthetic Issues recorded from its own run,
    including after handled assertion failures.
    Cleanup failure fails verification and reports affected URLs.

## Immediate next action

No implementation action remains.
Do not replace the direct Inquirer subpackages with the umbrella package,
add a direct Inquirer core dependency,
or add another subprocess dependency.
Future behavior changes must update this contract and repeat package,
built-CLI,
interactive,
and disposable live verification.
Ask one question only,
include the recommended answer with its pros and cons,
and wait for the user's response.
Do not inspect or add candidate dependencies until the relevant design branch makes their role concrete.

## Update history

- 2026-08-16:
  created before grilling,
  recording the selected local adapter,
  user requirements,
  dependency approval gate,
  prior evidence,
  and unresolved design tree.
- 2026-08-16:
  recorded the user's ingest-only decision and removed OCR process orchestration from scope.
- 2026-08-16:
  recorded structured-JSON-only pasted input and excluded human-readable OCR text parsing.
- 2026-08-16:
  recorded the three accepted OCR-native JSON shapes and rejection of arbitrary JSON fragments.
- 2026-08-16:
  recorded explicit `--interactive` or `-i` mode selection and non-interactive default behavior.
- 2026-08-16:
  excluded piped standard input from interactive mode and controlling-terminal reopening from implementation.
- 2026-08-16:
  expanded the pipe exclusion to every mode at the user's direction.
  Non-interactive ingestion now requires a named file.
- 2026-08-16:
  recorded positional file input,
  interactive no-path paste,
  non-interactive no-path failure,
  and rejection of `-` as a standard-input sentinel.
- 2026-08-16:
  recorded that the adapter never ingests redirected or piped standard input and ignores piped bytes.
- 2026-08-16:
  recorded the pnpm `v11.8.0` interactive-update source path and its use of `@inquirer/prompts` checkbox UI.
- 2026-08-16:
  recorded single-line interactive paste with no multiline handling;
  multiline JSON and JSONL require a named file.
- 2026-08-16:
  recorded that invalid interactive paste reports its error and exits nonzero without retrying.
- 2026-08-16:
  recorded atomic rejection before GitHub operations when any input record is malformed.
- 2026-08-16:
  recorded strict UTF-8 named files without byte-order-mark support or encoding auto-detection.
- 2026-08-16:
  recorded one post-selection summary and an explicit yes-or-no confirmation with no default.
- 2026-08-16:
  superseded the non-interactive default with required `--non-interactive` mode selection;
  neither or both mode flags is an error.
- 2026-08-16:
  reopened adapter-owned identity and deduplication after the user clarified that OCR supplies no issue identity.
- 2026-08-16:
  settled create-only publication with no synthetic identity,
  existing-issue lookup,
  duplicate suppression,
  updates,
  reopens,
  or automatic closure.
- 2026-08-16:
  settled non-interactive preview by default and required `--apply` for non-interactive issue creation.
- 2026-08-16:
  settled a separate red and text-marked interactive security picker,
  per-finding explicit disclosure confirmation,
  and the non-interactive `--apply`,
  `--apply --non-security-only`,
  and `--apply --all` authority ladder.
- 2026-08-16:
  clarified that security-classified findings may document remediated or non-exploitable concerns;
  the tool must explain disclosure semantics in every relevant interface.
- 2026-08-16:
  limited the security gate to normalized OCR category `security` only;
  content signals,
  severity,
  and classification uncertainty do not enter it.
- 2026-08-16:
  made explicit OCR category `other` an ordinary candidate with a visible `OTHER` marker.
- 2026-08-16:
  made missing category metadata an ordinary candidate with a visible `UNCATEGORIZED` marker.
- 2026-08-16:
  rejected separate persistence for withheld security findings;
  only counts and input positions are reported.
- 2026-08-16:
  settled root-only Git repository inference with explicit
  `--repo https://github.com/OWNER/NAME` override;
  subdirectories and non-repository directories error with rerun instructions when the flag is absent.
- 2026-08-16:
  settled deterministic issue titles using normalized category,
  OCR path,
  and the first meaningful content line.
- 2026-08-16:
  included both OCR `existing_code` and `suggestion_code` in separate issue-body sections when present.
- 2026-08-16:
  settled plain path-and-line source references with commit-pinned hyperlinks only after destination verification.
- 2026-08-16:
  applied `needs-triage` when available and silently fell back to a `[needs-triage] ` title prefix when absent;
  no other label options are supported.
- 2026-08-16:
  settled structured issue bodies with Finding,
  Source and metadata,
  optional existing and suggested code,
  and OpenCodeReview attribution sections.
- 2026-08-16:
  preserved OCR content as unchanged active Markdown without sanitizing headings,
  fences,
  HTML,
  links,
  or mentions.
- 2026-08-16:
  limited complete interactive body previews to selected security findings before their individual confirmations.
- 2026-08-16:
  preselected all normal findings and left all security findings unselected in their separate interactive pickers.
- 2026-08-16:
  required at least one combined interactive selection while allowing either individual picker to be empty.
- 2026-08-16:
  made pre-publication Ctrl+C and final-confirmation rejection successful cancellation with status zero.
- 2026-08-16:
  made non-interactive preview output an exact machine-readable JSON plan on standard output,
  with diagnostics on standard error.
- 2026-08-16:
  recorded explicit approval for existing catalog dependency `p-limit` and its queue-clearing semantics.
- 2026-08-16:
  recorded that requested concurrency five conflicts with GitHub's serial mutative-request guidance;
  final creation scheduling remains open.
- 2026-08-16:
  selected serial Issue creation with at least one second between mutations;
  the adapter will not add or use `p-limit`.
- 2026-08-16:
  limited explicit repository syntax to canonical `https://github.com/OWNER/NAME` URLs.
- 2026-08-16:
  made rate-limit,
  network,
  timeout,
  and `5xx` Issue-creation failures retryable despite acknowledged duplicate risk.
- 2026-08-16:
  allowed three retries per retryable failure with provider-aware exponential delays.
- 2026-08-16:
  added high-water Issue-number reconciliation before ambiguous retries,
  using exact generated title and body matches only among newly numbered items.
- 2026-08-16:
  made multiple post-high-water exact matches terminal,
  with every URL reported and no automatic closure or cleanup.
- 2026-08-16:
  selected `gh api` as the sole GitHub authentication and HTTP boundary,
  using private named request files and caller-owned retry orchestration.
- 2026-08-16:
  selected human-readable interactive applied-run output
  and one final JSON result object for completed or handled non-interactive applied runs.
- 2026-08-16:
  made the first post-publication Ctrl+C stop after an active bounded creation settles,
  while a second Ctrl+C terminates immediately.
- 2026-08-16:
  applied a fixed one-minute child-process deadline to every `gh api` invocation,
  with no user override.
- 2026-08-16:
  selected compact exit statuses:
  zero for success and clean cancellation,
  one for runtime and handled publication failures,
  two for invocation misuse,
  and 130 for forced second interrupt.
- 2026-08-16:
  selected the first meaningful `existing_code` line as the title-summary fallback
  when OCR content has no meaningful line.
- 2026-08-16:
  selected the first meaningful `suggestion_code` line as the next title-summary fallback.
- 2026-08-16:
  made a finding invalid when content,
  existing code,
  and suggested code all lack a meaningful line;
  one such record atomically rejects the input.
- 2026-08-16:
  capped final titles at 256 UTF-8 bytes
  and selected deterministic end truncation with a trailing ellipsis.
- 2026-08-16:
  rendered adapter-owned Existing code and Suggested code sections as delimiter-free indented code blocks.
- 2026-08-16:
  selected `package/cli/open-code-review-issue`,
  package `@monochromatic-dev/cli-open-code-review-issue`,
  and binary `open-code-review-issue`.
- 2026-08-16:
  completed the Clack versus Inquirer vet;
  adapter-specific interaction requirements recommend Inquirer.
- 2026-08-16:
  approved the Inquirer technology family,
  with only necessary package parts to become direct dependencies.
- 2026-08-16:
  validated `@inquirer/checkbox` plus `@inquirer/input` as the minimal library-backed set;
  rejected the umbrella package.
- 2026-08-16:
  separately approved `@inquirer/checkbox` 5.2.1 as a direct dependency.
- 2026-08-16:
  separately approved `@inquirer/input` 5.1.2 as a direct dependency.
- 2026-08-16:
  selected CLI-only adapter configuration,
  with no adapter configuration files or environment variables.
- 2026-08-16:
  selected GitHub CLI 2.97.0 as the minimum supported version.
- 2026-08-16:
  settled the standard mise task surface,
  including unit and built-CLI integration suites.
- 2026-08-16:
  selected `https://github.com/Aquaticat/issues-api` as the supplied disposable live-verification repository;
  read-only preflight confirmed that it is private,
  empty,
  Issue-enabled,
  and administratively writable.
- 2026-08-16:
  disabled select-all and invert shortcuts only in the security picker.
- 2026-08-16:
  required square checkbox indicators rather than circular indicators.
- 2026-08-16:
  selected `☐` and `☑` as the unchecked and checked indicators.
- 2026-08-16:
  required TTY standard input and TTY standard output for interactive mode.
- 2026-08-16:
  selected Node's native terminal and environment color policy,
  with no adapter-specific color flag.
- 2026-08-16:
  authorized the separate verification harness to close only synthetic Issues created during its own run.
- 2026-08-16:
  granted standing approval for any workspace-owned dependency;
  selected logger,
  build configuration,
  TypeScript configuration,
  and test harness packages for concrete adapter roles.
- 2026-08-16:
  approved nano-spawn as the direct process dependency;
  selected abort deadline plus forceful kill after verifying that its timeout option is not a hard deadline.
  Rejected adding any alternative subprocess dependency;
  raw `node:child_process` is the sole fallback.
- 2026-08-16:
  confirmed shared understanding and authorized implementation.
- 2026-08-16:
  implemented the adapter,
  passed package lint and complete fake-`gh` suites,
  exercised both interactive picker classes,
  and created then closed final live verification Issue `Aquaticat/issues-api#2`.
- 2026-08-17:
  removed interactive prompted JSON ingestion and required one positional input in every mode.
- 2026-08-17:
  restored shell-safe interactive inline JSON as a quoted positional value,
  retained named-file input,
  and added actionable generation plus latest-session JSONL guidance.

[clack-note-node-floor]: ../troubleshooting/clack-note-nested-styletext-node-floor.md
[github-issue-concurrency]: ../troubleshooting/github-issue-creation-concurrency.md
[github-issue-title-length]: ../troubleshooting/github-issue-title-length.md
[nano-spawn-timeout]: ../troubleshooting/nano-spawn-timeout-is-not-a-hard-deadline.md
[github-rest-best-practices]: https://docs.github.com/en/rest/using-the-rest-api/best-practices-for-using-the-rest-api
[ocr-routing]: ../troubleshooting/open-code-review-github-issue-routing.md
[prompt-vet]: ../audit/tech-clack-versus-inquirer-for-a-standalone-type-vet-2026-08-16.md
