# Task-slice plan revision and the `RCI` gap

Purpose:
 record the two corrections the user gave on 2026-10-29,
 what they re-derived in the migration plan,
 and one `AGENTS.md` edit those corrections expose.

What changed:
 [`doc/planning/task-slice-pants-migration.md`](../planning/task-slice-pants-migration.md)
 was rewritten in place.
Its previous framing
 ("hop 1 is written for cheap reversal
 and cheap second migration")
 and its previous target-generation home
 ("owned by `monorepo-task`,
 `RCI`:
 extend the package that already owns task orchestration")
 are both gone.

## The two corrections

1.  "We don't care about how much work hop 2 would take."
    Hop 2's cost is not a design input.
    The `meow`-shaped command stub ranked first
    only because it saved a hop-2 token change,
    so that ranking is void.
    The command surface is re-ranked
    on the target workload
    and the `meow` stub now ranks last.

2.  "`task-run` (task CLI) is an extremely ugly hack we did
    because Mise was bugging out."
    `@monochromatic-dev/monorepo-task-cli` and its 21 `task.ts` mirrors
    are a stopgap,
    not an owner to extend.
    The plan now retires them with Mise
    and moves the generator
    to `file-enforcer.config.ts:691` `generateMiseToml`,
    which already emits task config.

## What to inspect

- [`doc/planning/task-slice-pants-migration.md`](../planning/task-slice-pants-migration.md),
  section "Corrections applied 2026-10-29",
  then "Target generation" and "Command surface".
- `package/monorepo-task/cli/package.json`
  for the `task-run` bin and its `task.ts` mirrors.

## The `AGENTS.md` gap

Correction 2 is a failure mode
 the rules did not catch:
 `RCI` says "extend one that already owns it",
 and `monorepo-task` *did* already run the task graph,
 so the rule pointed at a stopgap.
Nothing in `AGENTS.md` distinguishes
 a deliberate owner
 from a workaround written around a misbehaving tool.

Per `GAP`,
 the fix is to tighten `RCI`.

Options.
Each appends to `RCI`,
which today reads:
"Before proposing a new owner for a responsibility,
inspect existing repo-owned generators and managers;
extend one that already owns it."

1.  Append the test sentence
    `A stopgap for a misbehaving tool is not an owner.`
    Full rule 191 characters after whitespace normalization,
    31 words.
    Pro:
     the added sentence is a standalone test
     the reader can apply to any candidate.
    Con:
     one more sentence to carry.

2.  Append the phrase
    `as a deliberate owner, not a stopgap`
    to the closing clause.
    Full rule 178 characters,
    28 words.
    Pro:
     shortest change,
     keeps one sentence.
    Con:
     the phrase depends on parsing the whole clause
     and names no test.

3.  New code beside `RCI`.
    Pro:
     leaves `RCI` untouched.
    Con:
     `NCD` wants new codes rare and distinct;
     this is the same responsibility `RCI` governs,
     so a second code splits one rule in two.

Ranking:
 1 > 2 > 3,
 because option 1 gives a standalone test
 the reader can apply to any candidate
 (`SLF`),
 option 2 states the distinction
 without a way to check it,
 and option 3 spends a new code
 on a responsibility `RCI` already owns.

Both 1 and 2 fit `RLM`
 (under 50 words and 200 characters).

## How to respond

```text
1        # apply option 1
2        # apply option 2
3        # leave AGENTS.md alone
```

No `AGENTS.md` or `CLAUDE.md` change is made until you answer.
