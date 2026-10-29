# Handover: em-dash sweep (issue #55)

> Moved 2026-10-29 by the generic package category elimination
> (`doc/planning/generic-category-elimination.md`);
> paths and names below this note are historical:
> `package/cli/forbidden-strings` is now `package/forbidden-strings/cli`.

## Status (2026-05-14, complete)

Issue:
 [#55](https://github.com/Aquaticat/Monochromatic/issues/55).
Tracks `docs:` sweep of em-dash and en-dash prose violations against the
`AGENTS.md` punctuation rule.

What's done:

- Unicode em-dash (`—`) in `.md` files:
   0 outside intentional content
  (commit `54c17ce0`,
   33 files).
- Unicode em-dash (`—`) in `.ts` files:
   0 outside intentional content
  (commit `f2b15a56`,
   28 files).
- Unicode en-dash (`–`) in `.md` and `.ts` files:
   0 outside intentional
  content (same two commits).
- ASCII `--` em-dash substitute in `.md` files (commit `7c13761b`,
   104
  files).
   Remaining `--` is CLI args inside code blocks,
   inline backticks,
  or CLI invocations,
   preserved by heuristic.
- ASCII `--` em-dash substitute in `.ts` files (commit `369b73e7`,
   248
  files).
   Conservative scope:
   only lines starting with `//` or `*`
  comment markers,
   skipping disable directives.
- ASCII single-dash (`-`) em-dash substitute in `.md` files (commit
  `78e055b9`,
   20 files,
   1242 instances replaced).
   Conservative scope:
  only when preceded by a close marker (backtick,
   `)`,
   `]`,
   `}`,
   `**`,
  `~~`);
   plus a focused regex for `**Status**: <value> - <description>`
  Status patterns;
   plus date-range pass converting `Month DD - Month DD`
  to `Month DD to Month DD`.
- ASCII single-dash (`-`) em-dash substitute in `.ts` files (commit
  `4978ae4f`,
   2 files,
   16 instances replaced).
   Conservative scope:
   only
  comment lines (`//` or `*`) preceded by a close marker,
   skipping
  TSDoc tag lines (`@param`,
   `@returns`,
   `@throws`,
   `@example`,
   etc.)
  because TSDoc convention uses `-` as separator,
   and skipping
  ```fences within TSDoc comments to avoid rewriting code examples.
  ```

What's left:

- A `forbidden-strings` rule to prevent regressions.
   Per user instruction
  during this session,
   this is filed as a separate GitHub issue rather
  than completed here.

Issue #55 closed with a comment noting the completed work and pointing
to the follow-up forbidden-strings issue.

## Sweep scripts (not retained)

The five one-shot sweep scripts written for this work are not kept around
after the sweep completes.
 The strategy and heuristics are documented in
the "Replacement heuristics" section below;
 that's the durable artifact.
If future content additions need a re-sweep,
 reconstruct the script from
the heuristics or write a more general `forbidden-strings` rule (tracked
as a separate follow-up issue) that prevents regressions at write time.

## Excluded files (intentional content)

The sweep scripts skip these files entirely:

- `AUDIT.em-dash.md` -- the audit itself,
   contains self-references.
- `PLANNING.forbidden-strings-em-dash.md` -- forbidden-strings investigation.
- `package/cli/forbidden-strings/README.md` -- forbidden-strings docs.
- `AGENTS.md` -- the rule statement (in backticks).
- `GLM_LIMITATIONS.md` -- documents model violations as examples.
- `TODO.claude-code-words.md` -- intentional dictionary-style definitions.
- `TODO.forbidden-strings.md` -- forbidden-strings infrastructure.
- `package/module/hyperscript/src/css/index.unit.test.ts` -- en-dash as
  CSS counter-style test data.

## Replacement heuristics (for reference)

Em-dash (`—`) was replaced by:

- `:` when preceded by code/bold/strikethrough/paren/bracket/brace close
  (definition / elaboration pattern).
- `;` otherwise (linked-clause / fallback pattern).
- `($middle)` when paired em-dashes bracket a short aside on the same
  line.
- Coordinate-vertex listings `(a,b) — (c,d)` became `(a,b), (c,d)`.

En-dash (`–`) was replaced by:

- `to` for numeric ranges (`14–22` -> `14 to 22`) and letter ranges
  (`A–E` -> `A to E`).
- `,` for any remaining en-dash (paired-aside fallback).

ASCII `--` was replaced by the same code-marker-aware heuristic.
 CLI
invocations and disable directives were preserved.

ASCII single-dash `-` (markdown) was replaced by:

- `:` when preceded by a close marker (backtick,
   `)`,
   `]`,
   `}`,
   `**`,
  `~~`) at line content position (skipping list-bullet position so
  `- item` bullets stay as bullets).
- `;` for the focused `**Label**: <value> - <Uppercase>` Status pattern
  (e.g. `**Status**: High Priority - Developer experience` becomes
  `**Status**: High Priority; Developer experience`).
- `to` for month-name date ranges (`August 16 - August 30, 2025` and
  `2024 - 2025`).
- Skipped for math expressions (`mean - 2*stddev`),
   list bullets,
   fenced
  code blocks,
   and inline backtick spans.

ASCII single-dash `-` (TypeScript) was replaced by the same close-marker
heuristic with three additional exclusions:
 TSDoc tag lines (`@param`,
`@returns`,
 `@throws`,
 `@example`,
 etc.) because TSDoc convention uses
`-` as the name/description separator,
 ``` fences within TSDoc
comments (`@example` continuation lines that hold code),
 and arrow
notation (`->`).

## Quality trade-offs

The mechanical sweep accepts some stylistic imperfection in exchange for
scope coverage:

- Inside parentheses,
   `—` sometimes maps to `;` where a comma would
  read better (e.g. `(2026-05-12, ninth handover; visual-fix iteration)`
  reads slightly off vs `(... ninth handover, visual-fix iteration)`).
- Headings like `# Done -- Implementation plan` become `# Done;
  Implementation plan` where `:` would be more idiomatic.
- Paired em-dashes around a phrase ending in a comma can leave a missing
  comma after the closing paren (`solids (spheres for leaf, octahedra
  for non-leaf) round/diamond-silhouetted` -- a follow-up pass could
  insert the missing comma).

These are listed for completeness;
 none affect content correctness.
Re-run the scripts or hand-edit individual cases if cleanup is needed
later.
