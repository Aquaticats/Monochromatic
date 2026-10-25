# Audit ledger: the check over the schema sources, code the search cannot read and the mutants that survived

B868 and B869,
which continue B865 to B867 of
[Audit ledger: the check over the schema sources as a whitelist](audit-ledger-schema-source-check-as-a-whitelist.md),
which was at the limit of a file.

Part of [Audit ledger](audit-ledger.md),
which lists every file of this record.

## B868: code the search cannot read, and a specifier the check cannot resolve, were silent

Status:
fixed on 2026-10-24 (UTC),
found by the re-check of fix round 15 of the Claude route
(re-check 14:
inputs that build a loader or run text,
and imports through an alias outside the sources).
The design that landed is open to the owner's veto.

The defect:
the first total of B865 reads the strings that name a module that carries a builder.
A file can reach a builder without such a string:
`createRequire(...)`,
`module.require`,
an alias of `require`,
`eval`
and `Function`
read a path or a text the search never sees,
and an import specifier the check cannot resolve
(`#lane/contest`)
may name a module that carries one.

The fix
(the red commits are `8e890caef51b4b0478c20afe600f79738269cb51`
and `2a546f212419f56c9d3874fb66bf01ac562b8ace`,
one case failing each;
the checks are `b55aaf374e7751d60b4ca88e8b1f60eab8c174ab`
and `8120c64c84581b9ea92f878ac60dc499a7e000c3`):

- `code-the-search-cannot-read` reports `createRequire`,
  `eval`
  and `Function`
  by name,
  and `require` used otherwise than as the callee of a direct call
  (an alias,
  a member,
  an argument).
  A name in a type position is not code.
- `unresolved-module-specifier` reports a specifier that is no built-in module,
  no package the lockfile holds for a package of the family,
  and no file of the family's sources or of a package beside them.
  A type-only import or export is not read.
  A file with an unresolved specifier is read further,
  so the other places in it are still reported.
- The check reads `pnpm-lock.yaml` and the family's files at the top of its module,
  because the repo bans synchronous file reads and regular expressions in this code.
- Limits,
  held by no case and named here:
  the lockfile is read as the union of the dependencies of every package of the family,
  where one package's own dependencies would be exact;
  `.constructor` and `node:vm` were not reported when the check was written,
  and are from fix round 18 (follow-up below).
  `rg --word-regexp 'Function|eval|createRequire'` over the non-test sources of the family finds five hits,
  all prose,
  none code.

Follow-up of fix round 17:
a member of `globalThis`,
`window`,
`self` or `global` whose key is not a string literal is reported as
`code-the-search-cannot-read` where it is written
(red `c1e9a491f86eb70099c5967af54c7ad9a3ead4ba`,
fix `ebfb8c0c59204b0930d861bd9746650971f19930`;
input F06,
a key joined from two strings).
A name in brackets is the value of a variable,
so the earlier read of such a key as the name it is written with had let `globalThis[key]` through.
A specifier of a package of the family resolves only to a file of the sources:
the package,
its `/ts` subpath and `/ts/<file>`
(red `796bddda272f8ea2cbdad6fe930132a3e93c2576`,
fix `dd57a75de1a0006d9b5d9eadcd08ddf3d4afff9a`;
inputs G08 and G09,
a subpath that does not exist and a path outside the exports map).
The red commit's second file expected silence for two specifiers the imaginary sources could not resolve,
and the fix corrected the file by giving it a `provider/src/index.ts`;
the four failing places of the red commit are unchanged by that.
The stated limits stay,
pinned silent by a case:
the constructor of a function,
the constructor of the prototype of an async function,
and `node:vm`
(inputs F11,
F14 and F12).
The sentence that followed,
"They cannot be told from any other member by a read of the text",
was wrong:
the module is told by its specifier,
and the constructor by a member named `constructor` or a `Reflect.get` with that string as its key.
Fix round 18 reports them (follow-up below),
and the pins of this paragraph moved into a case that asks for the reports.

Follow-up of fix round 18:

- The global object written as a whole value is reported
  (red `17196a93cdca9867dce76985550554e2cfdac94b`
  and fix `7b29be99a4a0ef248c51e49f2ac61c6be380ca9f`)
  when it is held in a constant or an array,
  handed to a call,
  chosen by a conditional,
  used as a key,
  or written under `void`.
  The object of a member and the operand of `typeof` are not reported.
- `vm` and `node:vm` in an import,
  an export from,
  a `require`,
  an `import()` or an import-equals declaration,
  a member named `constructor` written with a dot or a string literal in brackets,
  and `Reflect.get` with the string `constructor` as its key
  are reported as code made from text
  (red `181bc5a6fac956d6d65d1dcc5f84224f3aa463b6`
  and fix `13987f2e08130e382ba257a0a95de6a5259ddfda`).
  The limits that remain are measured silent:
  a key that is no literal,
  a pattern that takes `constructor` out of a value,
  and a `Reflect.get` with a key that is no literal.

Prevention rule:
a whitelist that anchors at the module-load boundary also lists the ways to build a loader,
and a specifier it cannot resolve is a place it does not understand.

## B869: four mutants survived, and the cases the new code called for

Status:
fixed on 2026-10-24 (UTC),
found by the re-check of fix round 15 of the Claude route
(re-check 14:
four mutants of round 15's check that every case passed)
and by the mutants of this round.
The design that landed is open to the owner's veto.

The defect:
`ChainExpression` and `TSTypeAssertion` as marks that leave a value as it is,
the member branch of the test for a followed load,
and the cycle guard of the walk of stars that lead back
(W1,
W3,
W4
and W5)
changed nothing in any case.

The fix:

- `eb1ec46210917d955bba16c8c6fb741c51ec9640` adds four cases,
  one for each mutant,
  which pass with the check as it is and fail under the mutant that names them.
  The cycle case is two modules that export everything of each other beside a module that carries a builder.
- `89fac8545b16a26a733cf413e51513fdf3afb4d6` adds cases for the branches of the new reports that no case reached,
  and `3d221c603626cb9ababbc9d8ee57633bc745c81d` adds a `new` of the sender.
- `b7d048af7b6f947e141982aa54390a7ec4a19359` drops three conditions of the walk of destinations
  (an arrow holder that is not the body,
  a default whose right side is not the value,
  a holder that is a call but takes the object other than as an argument).
  A probe that throws in each branch,
  run over the suite and the 1579 real sources,
  reached none of them,
  and a control branch that must be reached was reached by one case.
  The structure of the tree gives the same.
- The mutant table of the round has 63 rows.
  62 were killed on the first run on `81c62eae1e3e6a773e81f60d5ca30fb6bdcb5997`;
  S06 survived,
  and the survivor was first thought equivalent.
  It is not:
  a `new` of the sender has a callee and is no call,
  and the case of `3d221c603626cb9ababbc9d8ee57633bc745c81d` kills it.
  A second run of the same driver,
  before the cases of `89fac8545b` and the removal of the dead conditions,
  had left seven alive,
  of which four got cases and three were dead conditions of the new code.
- The test file took 3.37,
  3.34
  and 3.32 s on `956328c19d`,
  the merge this round started from,
  and 3.52,
  4.67
  and 4.77 s on `81c62eae1e`,
  three runs each,
  alternating,
  on a host with other sessions running.
  The head is slower by 0.15 to 1.45 s,
  and the spread of the head is 1.25 s against 0.05 s.
  The cause is not measured:
  the head holds more cases,
  the reads of the lockfile and the family's files,
  and the new totals,
  and no run held one of them apart from the rest.

Follow-up of fix round 17:

- The recheck of `b7d048af7b6f947e141982aa54390a7ec4a19359`.
  Its body argued that an object that is itself the callee of a call cannot be reached,
  because the call is no call of a sender.
  The place is reachable:
  the check reports the object as the value the function returns
  (`builder-result-returned-outside-a-wrapper`,
  "FunctionDeclaration returns it").
  The removed condition changes nothing,
  because the check of the callee rejects a callee that is no chain of names,
  and `2d02c768a937f7f1aa93781881dfae8889ef1415` holds the case.
  The same commit pins the function a report names when the value leaves by more than one:
  the first in the order of the source,
  so a function declaration before an arrow is named by the function and an arrow before it by the arrow.
- The mutant table has 142 rows on `1d3bd37002b264b9b272afc76f17f1a78dd27aef`:
  the 63 of round 16,
  the 13 of re-check 15
  and 66 new ones for the branches this round added.
  The run on `a642a09a9d45195eaa36ed37aa705ab2b084ea63` had left seven alive,
  the six of B866 and one that `1d3bd37002b264b9b272afc76f17f1a78dd27aef` kills,
  and the run on the final head left the six.
  Each of the six has controls in B866.
  The runs were sequential,
  one mutant at a time,
  in a user unit capped at 6 GiB of memory and 200% of the CPU.
- The schema suite took 3.77 to 4.13 s on `b3eda4996e706fcc81fc131e24240beb858527e6`,
  the base of the round,
  and 4.39 to 4.80 s on the head,
  three alternating runs each.
  A second set rotated four revisions three times:
  the revision before the cross-file constants,
  `2d02c768a937f7f1aa93781881dfae8889ef1415`,
  took 3.79 to 4.02 s;
  the constants fix,
  `0d917b63617491de2409d3edaa8b851b18856e24`,
  took 4.16 to 8.14 s;
  the forwarders fix,
  `a3a9fe5a8275020d9eb37933247d294cd1f18940`,
  took 4.16 to 5.79 s;
  and the head took 4.31 to 4.46 s.
  The spreads of 3.98 and 1.63 s are other sessions on the host,
  and 8.14 s is an outlier.
  The minimum rises by 0.37 s at the constants fix and by 0.15 s after it,
  so most of the 0.5 to 0.6 s the head costs came with the walk across files,
  and the cases the round added are part of that cost.
  The cause was not held apart from the cases.

Follow-up of fix round 18:

- The mutant table has 246 rows on `d87291ce4045b28194d076f6457d435455f6966b`:
  the 142 of round 17 and the 7 of re-check 16 that still apply,
  7 that pin the others and the import-equals declaration of `vm`,
  and 90 new ones.
  235 were killed by a failing case.
  1 was killed by not ending (N-R3e:
  no exit within 300 s).
  10 survived:
  the six of B866 and four of this round.
  Each is argued equivalent with a control that gives the same output on the head and on the mutant
  and a mutant that changes the same control.
  The four:
  a node that is no function never forwards.
  A spread callback has the same empty name either way.
  A value used as the key of a computed member names no method.
  A non-name `export =` exports nothing.
  The runs were sequential
  in a user unit capped at 6 GiB of memory and 200% of the CPU.
- The schema suite took 5.27,
  4.97 and 5.62 s on `b6540001c6c69b843a69f4e7ba9cf13e0b37ae26`
  and 6.58,
  14.46 and 6.78 s on `d87291ce4045b28194d076f6457d435455f6966b`
  (three runs each and alternating at a load average of 27 on 28 threads).
  The spread is 0.65 s on the base and 7.88 s on the head.
  The minimum rises by 1.61 s and the cause is not measured.
- The 1620 real sources gave 0 reports and 0 files loaded by `import()` or `require`.
  Of 225 adversarial inputs 45 were silent at the base of the round and 3 after it
  (RC21,
  RK37 and RK40,
  whose expectation re-check 16 judged wrong).
  None is worse.

Records corrected:
the body of `4527b378c1` says its header "is rewritten in the next commit",
and the header was rewritten in `35cea3a28b`,
two commits later;
the adversarial table of round 15 labels its row B19 "mjs path import (non-ts specifier)",
and the input is `import('node:fs')`,
which reaches no builder,
but not for the reason its name gives.
The commit is `4527b378c16de148a3c77968b405e6ca7edf8441`,
whose body ends
"The header of claude-schema-sources.test-fixture.ts still describes round 14's check
and is rewritten in the next commit",
and `git log` over that file shows the rewrite in `35cea3a28b`.
The row B19 reaches no builder because `node:fs` is a built-in module,
not because a non-ts specifier is not followed:
no scenario of round 15 exercised a non-ts specifier,
and the cases M09 and N13 of round 16
(the aliases `#lane/contest` and `#lane/wire`)
are the first that do.
B864's sentence "2.0 s with them" is "2.0 s without them"
in `audit-ledger-guard-demonstration-survivors-and-their-pins.md`.

Cache versions:
none moved.
The change is to test fixtures and to the files of the check.
The search over the commits of this fix prints nothing:

```sh
# package/translation-repair, over edf6323eb3..HEAD
git log --extended-regexp -G '_CACHE_VERSION = [0-9]+' edf6323eb3..HEAD -- package/translation-repair
```

The same search over `c265e8df95~1..c265e8df95` prints one line,
so the search can print.

Cache versions in fix round 17:
none moved.
The search over `b3eda4996e..HEAD` for `package/translation-repair` prints nothing,
although the range spans the merge of `origin/main` with the commits of the `ide` package,
and the same search without the path prints nothing either.
The same search over `c265e8df95~1..c265e8df95` prints the commit that moved `SLICE_CACHE_VERSION` from 43 to 44,
so the search can print.

Records corrected in fix round 18:

- `a3a9fe5a8275020d9eb37933247d294cd1f18940` says a seventh hop was left unmeasured
  and that following the chain would list most of the consistency pass.
  The chain ends at 15 forwarders called from 9 files,
  and its "1579 real sources" are 1620 on the head of fix round 18.
- `a642a09a9d45195eaa36ed37aa705ab2b084ea63` says the suite and both lint tasks give the same result as before.
  The suite passes on it (measured on a copy of the commit);
  the lint tasks were not run again.
- `cdf4516f463ff966890f7b73016a4e92fa87bc83` says two expectations change and lists three changes:
  two expectations and an input added to the accepting case.
- `13987f2e08130e382ba257a0a95de6a5259ddfda` says the import-equals declaration of `vm` is reported,
  and no case held that until `a0ac36bf6ca9e0acec34ca85981d2f752c47c453`.
- `a0ac36bf6ca9e0acec34ca85981d2f752c47c453` and `86a6fa40b6cd1bffb3f71d914f25256eee201b23` say `test:unit` ran.
  The schema suite was run directly with `node`,
  and `test:unit` ran on the final head only.
- `e33e81212ab259bbd9575966208d044e76036115` and `9da0e1386f19031d1eb505e20f7bb0764ad49dcd` name a review label
  that the repository does not hold:
  the input is RW26 of the adversarial table of re-check 15.

Cache versions in fix round 18:
none moved.
The search over `b6540001c6c69b843a69f4e7ba9cf13e0b37ae26..HEAD` for `package/translation-repair` prints nothing:

```sh
# package/translation-repair, over b6540001c6..HEAD
git log --extended-regexp -G '_CACHE_VERSION = [0-9]+' b6540001c6c69b843a69f4e7ba9cf13e0b37ae26..HEAD -- package/translation-repair
```

The same search over `c265e8df95~1..c265e8df95` prints `c265e8df9525f4513b8229482d3e8d932ea630df`,
so the search can print.

Recurrence:
`mistake-prevention-guards-searches-and-claims.md`,
"Guards that cannot fail".
