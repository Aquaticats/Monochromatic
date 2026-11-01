# Domain documentation

How the engineering skills should consume this repo's domain documentation when exploring the codebase.

## Source remains authoritative for current behavior

This repo intentionally does **not** maintain `CONTEXT.md`,
 `CONTEXT-MAP.md`,
 or `doc/adr/` directories.
 Cached code summaries stale quickly and create false confidence.
 Working understandings, user-defined meanings, and accepted decisions are nevertheless recorded
 beside the relevant plan or package, with dated evidence and explicit unknowns.
 The [understanding practice](understanding.md) describes that record.
 Instead:

- **Read the relevant source code directly** on every probe.
   Use codebase search,
   `rg`,
   and file reads to build understanding from scratch.
- **Do not create** `CONTEXT.md`,
   `CONTEXT-MAP.md`,
   or `doc/adr/` (even if a skill's default instructions suggest it).
- **Do not flag** their absence or suggest creating them.

## Vocabulary and recorded meanings

When naming a domain concept in output (issue title,
 refactor proposal,
 hypothesis,
 test name),
 infer the project's terms from the code you just read,
 and check the task's recorded user-defined meanings and accepted decisions.
 If you're unsure which term the project prefers,
 search the codebase for both candidates and use whichever appears more often.

## Architectural decisions

Accepted decisions and their rationale belong in the repository's canonical documentation.
Code and git history supply additional evidence about what actually landed.
A proposal or historical experiment is not proof of user acceptance or current implementation.
