# Remove enforcement disclosures from agent instructions

## Proposed wording and scope 2026-11-03T20:20:35-05:00

The user flagged the enforcement clause in `AGENTS.md` rule `RT1` as unnecessary
and asked what to do.
Remove that clause and the equivalent clause in `RGT`,
preserving the required search behavior.
This is a proposal;
adoption and edits to the instruction files remain pending.

## Evidence

- [`AGENTS.md`](../../AGENTS.md),
  `RT1`,
  names the search wrappers and their rejection boundary.
- [`AGENTS.md`](../../AGENTS.md),
  `RGT`,
  names the wrapper rejecting the short replacement flag.
- [`doc/philosophy/agents.md`](../philosophy/agents.md),
  "Enforcement mechanisms:
  why hook existence is not in AGENTS.md",
  already explains why agent instructions should carry the underlying discipline
  without announcing the mechanism enforcing it.
  Its examples concern hooks;
  extend that explanation to command wrappers.
- [`file-enforcer.config.jsonc`](../../file-enforcer.config.jsonc),
  rule `claude-md`,
  generates `CLAUDE.md` from its preamble and `AGENTS.md`.
  Regenerate the derived file through its existing owner when adopting this proposal.

## Exact replacements

```markdown
<!-- AGENTS.md -->
RT1:
 `rg` for text search,
 not directory navigation;
 `rg --files` for globs.
Scope `--hidden`,
 recursive `grep`,
 and unbounded `find` to named paths other than repo root.

RGT:
 `rg` recurses by default;
 its `-r` means `--replace`,
 so grep-reflex `rg -rl`/`-ir` silently rewrites matches in output.
```

Using "named paths other than repo root" preserves the repository-root restriction
currently conveyed through the enforcement clause.
Deleting only the final clause would leave "named path",
which could be read as permitting the repository root.

## Preventing recurrence

Extend the existing philosophy section to state that this reasoning covers command wrappers
as well as hooks.
When reviewing agent instructions,
retain the required behavior and information needed to perform the task;
keep implementation identities and detection details in the relevant maintenance documentation.
Avoid adding an always-loaded rule announcing that enforcement details are being withheld.

The recommended scope includes the directly matching search rules and their generated copy.
It fixes the reported wording and the sibling occurrence without broadening into unrelated tooling changes.
A wider review of agent instructions would be separate work.
Deleting the entire search rule would discard required behavior and is rejected.

This change removes unnecessary prompt material.
It does not make the enforcement implementation secret or establish a security boundary.
No claim about a measured improvement in agent behavior is made.

## Adoption checks

After adoption,
inspect the instruction diff for preserved search restrictions,
regenerate `CLAUDE.md` through the existing file-enforcer task,
and confirm the removed disclosures are absent from both instruction files.
Render the changed Markdown and run the scoped Markdown lint task.
