# Portability and core principles

## Portability and interoperability

Portable,
 interoperable,
 detachable.
No platform-specific solutions.

### Repository tool configuration

A cloned repository should not dictate what software on someone else's machine can do.
Root `.claude/` is ignored because it can contain machine-local permissions,
 hooks,
 Claude Code native subagents,
 skills,
and agent memory.
Commit Claude Code project configuration only when it is intentionally shared infrastructure.
Force-add reviewed shared config when needed,
 and keep permission-bearing files minimal.
Users maintain personal permission lists in their home directory (`~/.claude/settings.json`) or local project settings
(`.claude/settings.local.json`).

### Markdown features

Plain markdown readability with optional enhanced tooling.
