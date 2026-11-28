# The figures the proposal derives from the survey

This folder holds the scripts that recompute every figure
`doc/planning/monochromatic-lint-a-line-break-at-no-semantic-boundary-is-or-is-not-a-finding.md`
prints that the survey does not count,
with each script's whole output beside it,
so that a reader can check a number instead of trusting it.

The survey's own rows are in `../output.txt`.
Nothing here changes a count:
each script reads `../output.txt` or a copy of a file at the survey commit,
prints sums and quotients,
and writes nothing.

## What each file is

- `figures-for-each-kinds-answer.py` and its `.output.txt`:
  each kind's count in each predicate,
  each predicate's findings and ordinary share under two answers to the kinds question,
  the coverage and clear figures the predicate ranking reads,
  the emphasis bound,
  and each scope's count in each predicate's unit.
  Run:
  `python3 -I figures-for-each-kinds-answer.py ../output.txt`.
- `the-rule-files-prose-walked.py` and its `.output.txt`:
  the breaks inside paragraphs of `AGENTS.md` at the survey commit,
  and the class of each break at no boundary.
  It reads a copy of that file at `<root>/AGENTS.md`,
  made with `git show bc5b4ca2ba:AGENTS.md > <root>/AGENTS.md`.
- `the-examples-checked-against-their-files.py` and its `.output.txt`:
  for every break the proposal quotes,
  the file's own lines,
  the class of the next line's first token,
  the clause width,
  the forced flag,
  and the word counts.
  It prints the quoted text as the file holds it,
  which is the text to paste into the proposal.
- `forced-clauses-a-rewrap-cannot-clear.py` and its `.output.txt`:
  the clauses of two or more lines in `doc/audit`'s Markdown files at the survey commit,
  how many are forced,
  how many of those hold three words or fewer,
  and the class of the break a rewrap would have to leave.
  It reads copies under `<root>/doc-audit`.
- `a-finding-as-an-author-meets-it.txt`:
  a real run of the installed binary over two fixtures,
  the finding it prints,
  and the text `--fix` writes.
- `the-sweeps-and-the-hold-in-the-records.txt`:
  the read-only commands that name the sweeps of this rule's fix,
  their dates and which of them are ancestors of the survey commit,
  the hold's line in `doc/planning/repository-linter-stays-at-zero.md`,
  and the date the records give for the words.
- `the-sweeps-added-to-the-list-with-dates-and-diff-sizes.txt`:
  the commits the read above names for this rule's fix
  and the proposal's list of sweeps did not have,
  each with its own date,
  its subject and its file count against its first parent,
  with the commits the same search returns that are not sweeps.

## The copies a reader needs

Each script that reads repository files expects copies made with `git show`,
so that a rerun reads the survey commit and not a later branch:

```sh
git show bc5b4ca2ba:AGENTS.md > <root>/AGENTS.md
git show bc5b4ca2ba:doc/philosophy/agents.md > <root>/doc/philosophy/agents.md
git show bc5b4ca2ba:.agents/skills/css/SKILL.md > <root>/.agents/skills/css/SKILL.md
```

and so on for every path the script names.
`forced-clauses-a-rewrap-cannot-clear.py` needs each of the `doc/audit` Markdown files
at `<root>/doc-audit` under its own basename.

## What these figures rest on

A real invocation for the linter capture and the rule count,
and the survey's own rows for everything else.
The walkers are line walkers and not the linter:
they read no syntax tree,
their paragraph model is their own,
and the proposal says so where it uses them.
