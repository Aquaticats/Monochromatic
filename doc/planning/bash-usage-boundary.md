# Bash usage boundary and issue 308

## Status

Investigated 2026-09-09 at `3a4fdb7601c9644368b1d9185b9482c7c26e193d`.
This records findings and proposed issue framing,
not an adopted Bash exception or authorization to rewrite workflows.

Follow-up:
the user subsequently authorized switching issue 308's scope gate to Node and fixing it.
The adopted scope and implementation contract are recorded in
[the Node CI scope decision](../decision/toml-edit-ci-scope.md).
The broader language-policy proposal remains separate.

The question is whether Bash earns its place in repository-owned automation,
not which new shell or runtime to adopt.
No replacement dependency selection was undertaken.

## Finding

No project-specific requirement or accepted exception was found that justifies Bash
for the scope decision in [issue 308][].
Observed command-launch and externally defined interfaces do not by themselves justify
putting path classification and error policy in shell scripts.

The issue correctly identifies a fail-open defect.
Its acceptance criteria do not require a Bash implementation.
The missing architectural question is why this repo-owned decision is written in Bash at all.
Changing language alone would not fix the defect:
an implementation in any language can turn an unavailable comparison into a successful skip.

## What the history establishes

Commit `9e87f64d1d131a22a84167c12678886c52455d22` introduced the scope step on 2026-06-11.
Its message explains the need to skip unrelated merge-group verification while retaining a successful check,
and says it mirrors the changed-file guard in `forbidden-strings.yml`.
It does not explain choosing Bash.

The prohibition was already present:
`git grep --line-number 'Never write bash/powershell scripts' 9e87f64d1 -- AGENTS.md`
finds it at historical line 964.
This is not merely a new policy applied retroactively.

The comparison against issue 308's audit revision,
`4d1d66a0e2440f4402ea1eb454db6e49aacfbc48`,
shows that path renames and sidecar coverage changed,
but the failure-converting expression remains at
[`.github/workflows/toml-edit-fuzz.yml`](../../.github/workflows/toml-edit-fuzz.yml):71:

```bash
# .github/workflows/toml-edit-fuzz.yml
changed=$(git diff --name-only --diff-filter=ACMRT origin/main...HEAD || true)
```

No fix or fresh execution of this scope step was performed during this investigation.

## Evidence across the repository

### Workflow configuration

The workflow scan found explicit Bash selection in `cargo-publish.yml`,
explicit Node selection in `cli-git-performance.yml` (deleted on 2026-10-13 in commit `724d70bfa7`),
`cli-git-trust.yml` (deleted on 2026-10-13 in commit `642b6d72cb`),
and `kotlin-linter-publish.yml`,
and implicit interpreter selection in the toml-edit scope step.
Runner operating system and job defaults determine implicit execution.

This is an investigation of authored automation and issue 308,
not an exhaustive Bash-removal audit.
Personal shell configuration such as `.bashrc` and `.bash_profile`,
agent Bash-parsing plugins,
and third-party implementation languages would require separate consumer-level analysis.

### Parallel implementations and their contents

Line 37 of `.github/workflows/cli-git-performance.yml`
(deleted on 2026-10-13 in commit `724d70bfa7`; readable with
`git show 4c815fb13f5e31fc481d75dde215e5ef274e4ff1:.github/workflows/cli-git-performance.yml`)
already performed a pre-install decision under `shell: node {0}`.
Its body reads JSON,
invokes Git with an argument array,
validates inputs,
and writes `GITHUB_OUTPUT`.
This is relevant precedent for the same kind of scope decision,
not merely the presence of a Node dependency somewhere in the repo.

A real hosted run executed that step successfully before any mise or dependency installation.
The log records `shell: /usr/local/bin/node {0}`.
See the [runner source and execution evidence](../troubleshooting/github-actions-shell-selection.md).
Thus pre-install execution is a real constraint,
but it does not establish a Bash requirement on this runner.

[`.github/workflows/kotlin-linter-publish.yml`](../../.github/workflows/kotlin-linter-publish.yml):36
uses the same invocation for version detection;
its later Node steps validate artifacts and upload a publication.
Those are additional implementations,
not independently verified success claims in this investigation.

[`mise.no-env.toml`](../../mise.no-env.toml):258 documents the existing task boundary:
default platform shells for command launching,
explicit Node execution for logic.
The accepted [mise Node invocation decision](../decision/mise-task-node-invocations.md)
distinguishes pure forwarding from actual orchestration.
It governs mise tasks,
not an independently adopted Actions-wide policy.

### Comments and unfinished work

The comments in `toml-edit-fuzz.yml` explain the skip behavior and bootstrap order,
not an exception for the scripting language.
The release-archive comments in
[`.github/workflows/cargo-publish.yml`](../../.github/workflows/cargo-publish.yml):209
explain archive formats and optional README copying,
not why Bash must own those decisions.

A search of `.github` for `TODO`,
`FIXME`,
`HACK`,
`shellcheck`,
and `actionlint` found no matches.
This does not prove that workflow scripting is tested or that external checks are absent.

### Suppressions and exceptions

[`.github/workflows/logger-fuzz.yml`](../../.github/workflows/logger-fuzz.yml):67
contains the same `git diff ... || true` expression.
It is a separate affected workflow,
not evidence that changing toml-edit fixes logger.
Its remediation needs separate tracking;
this investigation does not extend issue 308 to cover it.

No `continue-on-error` entries were found in `.github`.
The error conversion in issue 308 happens inside the script instead.

[`package/file-enforcer/core/generator/entrypoint.sh`](../../package/file-enforcer/core/generator/entrypoint.sh)
is repo-authored Bash that starts PostgreSQL,
initializes a schema,
and dispatches a Python program.
Its [`Containerfile`](../../package/file-enforcer/core/generator/Containerfile)
copies and selects that script as the entrypoint.
It is another first-party scripting island,
not automatically an externally imposed exception just because its commands are third-party programs.

Paused scripts and historical reproduction scripts also exist.
Their presence is inventory evidence,
not approval to use them as templates for new automation.

### Stated policy and existing documentation

[`AGENTS.md`](../../AGENTS.md) `SCR` prohibits Bash/PowerShell scripts.
`CM2` says mise task logic uses Node.
Neither rule supplies a Bash exception for CI.

[`doc/todo/cli-tools.md`](../todo/cli-tools.md):186 explicitly rejects shell scripts.
Its suggested `mise.<action>.ts` placement is historical and contradicts current `SCR`;
it must not be copied as current implementation guidance.

[`doc/todo/forbidden-strings.md`](../todo/forbidden-strings.md):237
also rejects a native Git hook plus grep script because of the Bash prohibition.

[`doc/philosophy/agents.md`](../philosophy/agents.md):114 discusses an agent's command-shell assumptions.
[`doc/troubleshooting/bash.md`](../troubleshooting/bash.md) explains redirect ordering.
Neither is a rationale for authoring CI decision logic in Bash.

Searches covered `doc/decision`,
the broader `doc` tree,
`AGENTS.md`,
workflow contents and history,
using Bash,
shell,
POSIX,
bootstrap,
CI scripting,
and exception terms.
The conclusion is no justification found in those sources,
not that undocumented author intent can be disproved.

## Which reasons survive scrutiny?

- **Host command launching:**
  a workflow step such as `run: mise install node pnpm` can leave the host's launcher alone.
  Its tradeoff is continued dependence on that host interface.
  It does not require adopting Bash as the language for decisions.
- **Bootstrap availability:**
  avoid requiring project dependencies before deciding whether to install them.
  This is legitimate,
  but the hosted Node execution evidence shows it does not force Bash here.
  A replacement must still declare its runtime floor and avoid premature workspace imports.
- **Existing shell interfaces:**
  interactive shell configuration and third-party launcher contracts are different from
  repository-owned workflow algorithms.
  Retention would require naming the actual boundary,
  not labeling every subprocess call unavoidable.
- **Familiar CI conventions and copied examples:**
  these explain how the current code arose.
  They do not override the documented scripting prohibition.
- **Ubuntu-only execution:**
  portability is not a functional requirement of this particular job.
  That removes one possible objection to Bash,
  but supplies no positive need for it.

No blanket claim that Bash must disappear from the operating system,
agent tools,
containers,
or transitive dependencies follows from these findings.

## Proposed issue framing

Retain issue 308 as the fail-open bug,
and track the repository-wide language boundary separately.
A scope-step implementation should explain how it satisfies the existing scripting policy.
The implementation framing is:

> Make toml-edit scope detection a tested repo-owned program,
> not inline shell decision logic.
> An unavailable change comparison must fail the check,
> never become a successful no-op.

The evidence supports applying the existing no-shell-script policy,
not inventing an accepted exception to preserve this implementation.
The exact program location and bootstrap-compatible invocation remain implementation work.
The already-used Node boundary is evidence against necessity,
not a completed new technology selection.

Preserve the current acceptance criteria:

- A failed comparison fails the step and workflow.
- Only a successful comparison proving no relevant paths may skip verification.
- Validate or fetch the intended base explicitly.
- Cover relevant changes,
  irrelevant changes,
  and an unavailable base.
- A genuine no-op leaves the required merge-queue check successful.

Proposed additional acceptance criteria for a scope-step rewrite:

- Document and verify the intended merge-group base identity.
- Cover relevant deletions and renames into and out of scope.
- Cover unusual path bytes and exact agreement with the workflow's declared path set.
- Verify the entrypoint through the workflow boundary after local regression coverage.

These are proposed additions,
not claims that the current issue already specifies them.
Do not assume a runtime change repairs these semantics.

## Proposed documentation clarification

The concurrently authored
[load-bearing code language proposal](load-bearing-code-languages.md)
already addresses embedded programs and direct invocation glue.
Use that proposal as the policy discussion location rather than adopting competing wording here.

`SCR` currently prohibits scripts without explicitly defining an invocation-glue exception.
Making that boundary explicit requires accepted wording,
not silently treating this investigation as authorization.
Inline YAML/TOML bodies and shell command strings that implement logic must be addressed,
not just files named `.sh`.

Any requested exception should document its exact consumer,
why the permitted program cannot meet that consumer's requirements,
verification,
and when to revisit it.
No such exception is adopted here.

The issue,
workflow code,
and agent instructions are unchanged by this investigation.

[issue 308]: https://github.com/Aquaticat/Monochromatic/issues/308
