# research-agent

Provides a Claude Code native `research` subagent with anti-hallucination guardrails
for investigating external projects,
 GitHub discussions,
 and technical questions.

## Motivation

The built-in Claude Code native Explore subagent uses Haiku,
 which confidently fabricates content
when source data fails to load (e.g. JavaScript-rendered GitHub pages returning error states).
This agent replaces Explore for external research tasks by:

- Inheriting the parent session's model instead of defaulting to Haiku
- Requiring `gh api` for GitHub content instead of web fetch
- Mandating explicit source failure reporting instead of gap-filling
- Structuring output with verified/unverified distinction

## When it triggers

The agent activates for questions about external project decisions,
 GitHub discussions,
library capabilities,
 and other research where factual accuracy matters.
It does **not** replace Explore for local codebase searches:
use Grep/Glob directly for those.

## Anti-hallucination rules

The agent's system prompt enforces:

1. Never fabricate quotes or attribute claims to unread sources
2. Report source failures explicitly with what was actually returned
3. Mark claims as "verified from [source]" or "inferred (no direct source)"
4. Use `gh api` for all GitHub content (issues,
    PRs,
    discussions)
5. Structure output with sources consulted and gaps identified
