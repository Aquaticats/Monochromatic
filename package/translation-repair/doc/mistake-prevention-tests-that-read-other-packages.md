# Preventing mistakes: tests that read other packages

Rules for what a test or a test fixture may read:
its own package's folder,
a folder it made,
and the API of a package it declares.

Part of [Preventing this package's mistakes](mistake-prevention.md),
which lists every file of this record.
The record of the round that made the rule is
[Audit ledger: tests read only what their package declares](audit-ledger-tests-read-only-what-their-package-declares.md).

## Tests that read the files of a package they do not declare

What happened:
the gate runs the packages a change can reach through the declared package graph
(`workspace:*` in `package.json`),
and six reads in five tests and fixtures of the family reached a package they did not list.
A fixture listed every package's `src` and read the repository's `pnpm-lock.yaml`;
a test read five source files of four other packages;
a test listed the family folder;
and four cases asked git for the root and the head of the checkout the test ran in.
Each passed in a gate that never ran it,
because the package it read changed and the package it sat in did not (ledger B1265).

The rule:
a test or a fixture of a package other than `source-scan` reads,
lists or runs git over
the files of its own package,
a folder it made,
the pinned corpus clone,
or a package it declares,
and a package it declares is reached through its API by name.
A test of a tool that reads files or history builds a throwaway folder or repository and reads that.
A check whose subject is the whole family's source goes into `source-scan`,
whose input is the whole family and whose whole `test:unit` runs in every gate:
the gate selects the package whatever changed and runs every test file of it,
not only the files the `source-scans` task lists (ledger B1310).
The gate takes no always-run list for the tests that read by path:
the owner ruled it out ("You don't need a temporary always run list.
Fix before testing").

What to do when the scan fails:
read the finding's kind.
A `climb`,
`import`,
`literal-read` or `glob` is a path that leaves the package:
use the folder the test made,
import the API,
or move the check to `source-scan`.
A `git-checkout` or a `git-cwd` is a git read of the checkout the test runs in:
build a repository in a scratch folder (`scratchDirWith` and `fixtureGit`) and read that.
A read of a file that belongs to no package (the repository's tool lock)
is listed in the scan with its reason,
its count and the expression that names the file,
so a rewrite of the read to reach another file is a finding too,
and the owner decides whether it stays.
A change of that file alone sends the gate to every package,
so the case runs (ledger B1311).

What enforces it:
`source-scan/src/package-reads.unit.test.ts`,
among the source scans,
parses every test and fixture of every other package.
It fails a path built from the file's own place or the working directory whose `..` segments climb past the package
folder,
a relative import past it,
a read of the file system written whole in literals that climbs or names `package/translation-repair`,
a `glob` whose pattern climbs,
and a git read whose folder is the checkout or is not named,
naming the file,
the line,
the kind and the expression.
It has 82 fixture cases that must be found and 24 controls that must pass
(a folder the test made,
a corpus clone,
a package imported by name,
a fixture repository,
a glob that stays inside the package,
a git command that only spells a read verb).
Each call,
git command and folder option the scan holds in a list has a case of its own
that reaches the finding through that name alone,
and a case fails for a name added to a list with none.
Each of 44 listed names,
7 further detectors and 30 branches of its reader was switched off in a scratch copy,
one at a time (ledger B1313);
a case failed every time but twice,
and the two branches that survived got a case that fails without them.
What was not switched off one at a time is the arithmetic of levels,
the loop that finds held constants and the exemptions of the scan's own package,
which the cases for the forms exercise.
Its header states what it does not read:
a path built from text another module returns,
a climb held in a variable,
a git read whose arguments are built apart from its call,
and a file read by a child process the test starts.
