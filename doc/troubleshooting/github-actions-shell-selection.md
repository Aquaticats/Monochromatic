# GitHub Actions: implicit Bash execution does not require Bash decision logic

## Symptom

The scope step in [issue 308][] runs before mise and project dependency installation.
That ordering can suggest that Bash is required for bootstrap logic.
The workflow does not name a shell,
and GitHub's `shell` field can misleadingly sound restricted to shell languages.

There is no missing-runtime diagnostic in this investigation.
The question is which execution boundary is required,
not a newly diagnosed runner defect.
The [Bash usage assessment](../planning/bash-usage-boundary.md) separates this capability finding
from the repository policy decision.
The subsequently authorized implementation is recorded in
[the Node CI scope decision](../decision/toml-edit-ci-scope.md).

## Root cause of the assumption

[GitHub's workflow syntax documentation][syntax] defines custom execution as
`command [options] {0} [more_options]`.
The command must be installed on the runner.
The field name does not require Bash.

Source inspected on 2026-09-09:

- Repository:
  <https://github.com/actions/runner>.
- Revision:
  `0e345bcaa292fea27a2f218c0edab3d3b93e1060`.
- Read-only clone:
  `~/temp/agent/actions-runner-bash-rationale-20260908`.
- Observed hosted runner:
  `2.337.0`.
  The source snapshot is a separately identified upstream revision,
  not a claim that the hosted binary was built from that exact commit.

### The default selects Bash when available

`src/Runner.Worker/Handlers/ScriptHandler.cs:198-203` chooses the Unix executable
when no shell is specified:

```csharp
// src/Runner.Worker/Handlers/ScriptHandler.cs
shellCommand = "sh";
commandPath = WhichUtil.Which("bash", false, Trace, prependPath) ?? WhichUtil.Which("sh", true, Trace, prependPath);
```

This explains the executable used by an implicit Ubuntu `run:` step.
It is a default,
not a restriction on our implementation language.

### Custom commands bypass that choice

`src/Runner.Worker/Handlers/ScriptHandler.cs:224-228` resolves the requested command:

```csharp
// src/Runner.Worker/Handlers/ScriptHandler.cs
var parsed = ScriptHandlerHelpers.ParseShellOptionString(shell);
shellCommand = parsed.shellCommand;
// For non-ContainerStepHost, the command must be located on the host by Which
commandPath = WhichUtil.Which(parsed.shellCommand, !isContainerStepHost, Trace, prependPath);
argFormat = $"{parsed.shellArgs}".TrimStart();
```

`src/Runner.Worker/Handlers/ScriptHandlerHelpers.cs:70-75` splits the command from its argument template:

```csharp
// src/Runner.Worker/Handlers/ScriptHandlerHelpers.cs
var shellStringParts = shellOption.Split(" ", 2);
if (shellStringParts.Length == 2)
{
    return (shellCommand: shellStringParts[0], shellArgs: shellStringParts[1]);
}
```

`ScriptHandler.cs:267` substitutes the temporary script path for `{0}`,
and `ScriptHandler.cs:284` writes the workflow body to that file:

```csharp
// src/Runner.Worker/Handlers/ScriptHandler.cs
var arguments = string.Format(argFormat, resolvedScriptPath);
```

```csharp
// src/Runner.Worker/Handlers/ScriptHandler.cs
File.WriteAllText(scriptFilePath, contents, encoding);
```

Consequently,
inline workflow decision logic is still a script even when no `.sh` file is checked in.

### Failure policy remains the program's responsibility

`ScriptHandler.cs:352-355` maps a nonzero process status to a failed step:

```csharp
// src/Runner.Worker/Handlers/ScriptHandler.cs
if (exitCode != 0)
{
    ExecutionContext.Error($"Process completed with exit code {exitCode}.");
    ExecutionContext.Result = TaskResult.Failed;
}
```

The runner cannot recover an error that the program itself converts into success.
This is why switching the interpreter alone is not a fix for issue 308.

## Verification

The evidence crosses the hosted runner boundary,
not just a local syntax check:

- [Workflow run 34060935175][run],
  job `101561212817`,
  `detect-version-bump`.
- Source revision:
  `4c815fb13f5e31fc481d75dde215e5ef274e4ff1`.
- Runner `2.337.0`,
  Ubuntu `24.04.4`,
  image `20260831.293.1`.
- `Detect cli-git version bump` completed successfully.
- Its job performed checkout then the Node decision step,
  without a Node setup,
  mise setup,
  or package installation step.

Read-only commands used:

```sh
# From the repository root
gh run view 34060935175 --repo Aquaticat/Monochromatic --json jobs,conclusion,headSha,url
gh run view 34060935175 --repo Aquaticat/Monochromatic --job 101561212817 --log
git show 4c815fb13f5e31fc481d75dde215e5ef274e4ff1:.github/workflows/cli-git-performance.yml
```

Relevant log output:

```text
shell: /usr/local/bin/node {0}
cli-git version 0.0.1 -> 0.0.1; benchmark=false
Set output 'should_run'
```

The matching [Ubuntu image manifest][image] lists Node.js `22.23.2` as installed software.
The executable path in the job log establishes PATH availability;
the fact that checkout is a JavaScript action would not establish that by itself.

### Working catalog

- Explicit `shell: node {0}` runs dependency-free JavaScript before project setup.
- The observed program reads a package manifest,
  compares a historical manifest using Git,
  and writes a workflow output.
- The output is consumed as a job output;
  the dependent benchmark job is skipped for this unchanged-version result.

### Failure catalog and limits

No failing hosted custom-shell run was generated during this read-only investigation.
No claim is made that a rewritten toml-edit gate has passed regression tests.
Issue 308 was the separate repository-owned fail-open defect,
not a limitation of the runner's custom-command support.

The result does not establish Node availability on arbitrary self-hosted runners,
container images,
or future hosted images.
It also does not establish support for every current TypeScript syntax feature in the bootstrap runtime.

## Verified alternative invocation

The invocation that existed until its deletion on 2026-10-13 (commit `724d70bfa7`) was line 37 of
`.github/workflows/cli-git-performance.yml`, readable with
`git show 4c815fb13f5e31fc481d75dde215e5ef274e4ff1:.github/workflows/cli-git-performance.yml`:

```yaml
# .github/workflows/cli-git-performance.yml
shell: node {0}
```

Its advantage is pre-install execution without shell decision syntax.
Its tradeoffs are reliance on a PATH-visible Node runtime and responsibility for explicit error propagation.
A reusable program also needs a defined runtime floor,
module mode,
and testable entrypoint.
The hosted example establishes feasibility,
not a ready-made replacement for the toml-edit algorithm.

## What does not establish a solution

- A successful workflow conclusion alone:
  inspect the particular step,
  its log,
  and the exact workflow revision to rule out a skipped step.
- A JavaScript action's internal Node runtime:
  it does not prove the custom command can resolve `node` from PATH.
- Renaming `shell` without translating the program:
  Bash syntax does not become JavaScript.
- Merely translating the program:
  swallowing a Git error still creates the same defect.
- Importing workspace dependencies before installation:
  the verified example uses Node built-ins,
  not the repository's installed dependency graph.

## Upstream filing decision

The `.out-of-scope/` Bash and shell search found only the unrelated Claude Code exemption.
No runner bug or missing feature was established,
so there is no upstream contribution to draft or duplicate report to file.

1.  **Upstream fault:**
    no;
    the runner provides the documented custom-command mechanism.
2.  **Upstream fixability:**
    no fix is required for this capability.
3.  **Supported use:**
    yes;
    custom command templates are documented and observed in a hosted run.
4.  **Contribution welcome:**
    not evaluated because no contribution is proposed.
5.  **Likely upstream action:**
    not applicable;
    the required mechanism already exists.
6.  **Fix prototype:**
    not applicable;
    no upstream source modification is needed.

Nothing to add upstream.
Any eventual issue 308 implementation belongs in this repository.

[issue 308]: https://github.com/Aquaticat/Monochromatic/issues/308
[syntax]: https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-syntax#custom-shell
[run]: https://github.com/Aquaticat/Monochromatic/actions/runs/34060935175/job/101561212817
[image]: https://github.com/actions/runner-images/blob/ubuntu24/20260831.293/images/ubuntu/Ubuntu2404-Readme.md
