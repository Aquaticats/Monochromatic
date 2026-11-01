# Claude subagent model restrictions

## Purpose and settled scope

Research recorded on 2026-10-15 for the request to prevent Claude Code from using Opus or Fable subagents.
The user chose account-wide configuration on this machine and ordinary-call enforcement.
Deliberate bypasses are outside scope.
The separately configured Fable advisor must remain unchanged.
The user selected ordinary Agent delegation only and requested Sonnet for the general-purpose agent.
Forks,
subagent skills,
and teammates are outside the selected scope.
The accepted general-purpose default and explicit model deny rules were installed on 2026-10-15.

The user corrected the ordering of the discussion:
enumerate supported mechanisms before asking which delegation paths to include.
The inventory distinguishes documented behavior from verified local behavior.
No model request was made for this research.

## Supported mechanisms

### Permission rules

`Agent(model:opus)` and `Agent(model:fable)` are supported deny rules.
Parameter rules compare literal inputs and support `*`,
so aliases and full IDs need appropriate coverage.
An omitted parameter never matches,
even with `Agent(model:*)`.
Therefore these rules alone cannot reject a prohibited model selected through a definition or inheritance.
`Agent(Explore)` selects an agent type,
not a model.
Denying `Agent` blocks that entire tool.
These are useful rejection primitives,
not an effective-model policy.
[Permission syntax][permissions]

### PreToolUse hooks

`PreToolUse` receives the Agent tool's inputs,
including its optional `model`.
A command hook can deny a call or modify `updatedInput`.
A custom guard can reject absent or unrecognized model inputs rather than trusting inheritance.
Resolving omitted models itself would require reproducing Claude Code's model selection rules.
The documented input is not a resolved-model attestation.
[Hook reference][hooks]

### Forced subagent model

`CLAUDE_CODE_SUBAGENT_MODEL` alone is a default.
Invocation models and agent definitions take precedence.
Adding `CLAUDE_CODE_SUBAGENT_MODEL_FORCE=1` forces the chosen model for ordinary subagents,
teammates,
and workflow agents,
ignoring definition models and removing per-spawn model selection.
This requires v2.1.257 or later.
Conversation forks and skills explicitly using `model: inherit` remain exceptions.
This is model substitution,
not rejection of a prohibited request.
[Subagent model controls][subagents]

### Fork controls

`Agent(fork)` denies Claude's conversation-fork agent type.
`CLAUDE_CODE_FORK_SUBAGENT=0` disables fork mode,
but a human can still invoke `/subtask` directly.
Forks inherit the main conversation's model.
Blocking ordinary Agent forks does not establish coverage of direct commands or forked skills.
[Subagent fork controls][subagents]

### Agent definitions

User agent files can select permitted models,
but project and CLI `--agents` definitions have higher priority.
Per-invocation model selection also wins without FORCE.
Definitions therefore configure defaults rather than enforce a universal family prohibition.
[Subagent configuration][subagents]

### Skill controls

`context: fork` skills start isolated subagents,
distinct from conversation forks.
Their `agent` field selects a definition;
omitting it selects `general-purpose`.
Skill frontmatter can select a model.
`Skill(name)` deny rules block selected skills,
while denying `Skill` blocks Claude's skill tool.
`disable-model-invocation: true` prevents automatic invocation but leaves direct human invocation available.
Direct slash commands and Claude's Skill calls are separate entry paths requiring separate validation.
An Agent-only guard must not be described as covering all skill launches without evidence.
[Skill controls][skills]

### Global model policies

`availableModels` controls main sessions,
subagents,
teammates,
skills,
and advisors.
Excluding the configured advisor disables it,
conflicting with the settled requirement.
Blocked subagent overrides fall back rather than fail.
The Default selection also needs `enforceAvailableModels` for complete allowlist coverage.
User/project allowlists merge;
a managed allowlist has stronger precedence.
`deniedModels` and `availableModelsMatch` are managed-only controls requiring v2.1.283 or later;
user settings ignore them with a warning.
These policies do not offer a documented subagent-only exception that preserves a prohibited-family advisor.
[Model policies][models]

### Lifecycle hooks and advisory instructions

`SubagentStart` cannot block creation;
it injects context.
Its documented input includes agent identity,
not a resolved model.
`PreModelSwitch` blocks user/client-requested session model changes,
but does not run for automatic fallback or restored models.
It is not a subagent launch veto.
`PostModelSwitch` runs after a change and cannot block it.
[Hook reference][hooks]

Consequently instructions asking Claude to avoid particular models are not a rejection mechanism.
This follows from the distinction between context injection and supported blocking decisions.

### Fallback configuration

`fallbackModel` and `--fallback-model` chains apply to subagents too.
A permitted launch model therefore does not establish the model used after an availability failure.
The documented FORCE control does not claim to constrain that subsequent chain.
Inspect fallback configuration when validating any proposed restriction.
[Model fallback][models]

## Account-wide placement and local evidence

User settings live at `~/.claude/settings.json` and apply across projects.
Project/local files and command-line settings can override ordinary keys;
permission lists merge.
Managed settings are a stronger delivery tier,
not a new subagent-specific model policy.
[Settings precedence][settings]

The main agent's sanitized local probe found `model: sonnet`,
`advisorModel: fable`,
an empty deny list,
no `fallbackModel`,
and no `availableModels` or `deniedModels`.
Existing subagent environment settings set depth to 1 and concurrency to 5.
These observations describe configuration,
not a tested enforcement result.

## Remaining decisions and validation

Q3 is settled:
ordinary Agent delegation only.
The user requested Sonnet for the general-purpose agent to cover omitted-model inheritance.
The installed account-wide settings edit uses `CLAUDE_CODE_SUBAGENT_MODEL=sonnet`
and deny rules `Agent(model:*opus*)` and `Agent(model:*fable*)`.
This retains the built-in general-purpose instructions rather than replacing its definition.
The documented selection order gives the environment default priority over parent-model inheritance,
but gives agent definitions and explicit invocation models priority over that default.
The deny rules address explicit Opus/Fable selections.
Custom definitions selecting either prohibited family or `inherit` remain an unresolved path.
No complete prohibition across named agents has been demonstrated.

Automatic approval review initially blocked the launcher's cgroup PID write.
After the question widget failed to appear for the user,
the agent restated the exact approval question in chat and the user approved it.
The settings write completed successfully.
Its readback showed the Sonnet subagent default,
both deny rules,
and unchanged `advisorModel: fable`.
`claude doctor` on native version 2.1.300 exited successfully with no installation issues.
This is configuration and installation verification,
not a runtime proof of model selection or denial.

The user subsequently requested that resumes remain allowed,
then deferred removal of the repository's existing blanket resume guard.
They reported that the guard does not appear to work and requested README documentation only.
The observation and verification limits are recorded in the
[guardrail README](../../package/claude-code-plugin/guardrail/README.md#resume-blocking-status).
The guardrail implementation remains unchanged.

Validation must exercise accepted and rejected calls through the installed Claude Code host,
including omitted models,
aliases,
full IDs,
definitions,
and resumed agents.
Forks,
skills,
and teammates are excluded from this enforcement scope.
Direct user commands must be distinguished from model-invoked tools.
The advisor must remain configured and operational.
Use disposable configuration fixtures;
do not test policy mutations against shared real state.
Host verification must use human-supervised interactive sessions,
following the [Claude Code boundary](../decision/general-purpose-subagent-ban.md)
as it stood on 2026-06-12.
That supervision requirement was lifted on 2026-10-29,
so keep this requirement only if host verification needs it on its own merits.

Research continues without Codex subagents.
The user reiterated that Codex delegation is disabled;
`~/.codex/config.toml` still has `multi_agent = false`.
The [Codex decision](../decision/codex-context-and-subagent-settings.md) requires absent collaboration tools.
This session nevertheless exposed working tools and research agents were launched before that decision was checked.
Their work has finished;
no further delegation is authorized by the research workflow.

## Proposed guidance correction

Tighten the existing `QPM` rule in `AGENTS.md`,
rather than adding a duplicate rule:

> Before asking mechanism-dependent questions,
> enumerate supported controls and try dissolving the constraint;
> ask only choices the evidence leaves open.

This proposal addresses the premature scope question.
It has not been accepted or applied.

## Next action

The authorized account settings edit is installed outside repository version control.
Runtime verification of model selection and rejection remains outstanding.
Custom named-agent inheritance remains an unresolved limitation.
Do not remove the resume guard based on the unverified observation.

[permissions]: https://code.claude.com/docs/en/permissions#match-by-input-parameter
[hooks]: https://code.claude.com/docs/en/hooks
[subagents]: https://code.claude.com/docs/en/sub-agents
[skills]: https://code.claude.com/docs/en/skills#run-skills-in-a-subagent
[models]: https://code.claude.com/docs/en/model-config
[settings]: https://code.claude.com/docs/en/settings#settings-precedence
