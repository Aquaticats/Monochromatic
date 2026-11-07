# Load-bearing code and approved languages

## Status and requested scope

The user requested a documentation location and wording,
not a workflow migration.
The clarified requirement covers every unapproved language,
including Bash,
PowerShell,
Python,
`sh`,
and fish.
The motivating example was chained Bash code in GitHub Actions YAML.
The initial Bash/PowerShell-only proposal was too narrow.

This document records the corrected requirement and proposed wording.
It does not install new rules or authorize unrelated workflow changes.

## Evidence

- [`AGENTS.md`](../../AGENTS.md),
  `SCR`,
  prohibits Bash/PowerShell scripts but does not explicitly address other unapproved languages
  or executable code embedded in configuration.
- `1CB` limits chains in Bash tool calls.
  It is not a repository-wide boundary for persistent automation.
- `CM2` already permits declarative mise task sequencing.
  A new language rule must not accidentally prohibit that configuration.
- [Cargo publication workflow](../../.github/workflows/cargo-publish.yml),
  `Detect version bump`,
  uses shell and `awk` to read versions,
  chained tests to determine available history,
  and shell branches to decide whether to publish.
- [Forbidden-strings workflow](../../.github/workflows/forbidden-strings.yml)
  implements release fallback and changed-file selection in shell.
- [Logger fuzz workflow](../../.github/workflows/logger-fuzz.yml),
  `Scope to logger changes`,
  implements change-based execution decisions in shell.

These examples establish the embedded-code gap,
not an exhaustive inventory or evidence that the workflows currently fail.

## Recommended location

Keep the authoritative rules in `AGENTS.md`,
under `Cross-runtime and scripts`.
Add a general approved-language rule and an executable-command boundary with fresh shortcodes.
Remove the superseded language-specific prohibition from `SCR`,
retaining its TypeScript/mise execution guidance and filename restriction.
Clarify `1CB` as an ad hoc tool-call limit,
not an exemption for persistent automation.

Assign shortcodes only when applying the accepted wording,
checking existing codes and the local forbidden-strings appendix.
Do not duplicate the policy in a GitHub-only instructions file.

## Proposed rule text

### Implementation language

Repo-authored logic uses approved languages only,
even inside CI YAML,
hooks,
task definitions,
or generated snippets.
Unapproved languages may only provide direct invocation glue.

### Executable command boundary

Command count grants no exemption.
Parsing,
transformations,
branching,
retries,
safety checks,
and command orchestration belong in approved code,
not shell strings or interpreter arguments.

### Declarative configuration boundary

Native declarative workflow/task wiring is allowed;
custom algorithms belong in approved source.
Keep TypeScript as default for repository automation.

## Meaning and limits

Direct invocation glue means launching an existing approved task or tool with arguments,
including ordinary quoted environment values.
It does not include maintaining an executable program inside an argument,
heredoc,
command substitution,
or pipeline.
An approved-language wrapper that sends project-specific code to an unapproved interpreter
still violates the boundary.
Splitting shell logic into shorter commands or separate steps does not change its language.

Native configuration can connect jobs,
steps,
inputs,
and tool-produced outputs.
It must not become an alternative location for custom parsing or publication algorithms.
The executable-command boundary does not prohibit native declarative task sequencing.

The rule targets source semantics maintained by this repository,
including project-authored programs emitted by templates or generators.
It does not newly constrain the internal implementation languages of third-party tools
or ordinary compiler output from approved source.

The word "approved" refers to an explicit approval applicable to the work,
not the presence of a language in existing files or installed tools.
The [iOS stack decision](../decision/ios-iphone-x-music-player-kopia-stack.md)
records Kotlin,
TypeScript,
and Rust for that scoped comparison.
It is not evidence of a universal repository allowlist.
The present proposal does not silently promote that scoped list.

## Review and next action

An independent advisor review identified ambiguous direct-invocation,
declarative-sequencing,
generated-output,
and third-party boundaries in the draft.
The proposed wording and explanations distinguish those cases.

Present the revised wording for acceptance.
Before applying it,
resolve any missing language approval within the intended scope.
A workflow migration or broader inventory remains separate work.
No workflow files were changed or executed for this proposal.
