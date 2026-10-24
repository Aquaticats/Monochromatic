# Audit ledger: the check over the schema sources as a whitelist

The check over the family's real sources that backs the header of the producer-list search,
rebuilt from a list of named shapes into a whitelist anchored at the module-load boundary.
It continues B864 of
[Audit ledger: survivors of the guard demonstrations and the tests that pin them](audit-ledger-guard-demonstration-survivors-and-their-pins.md),
which was at the limit of a file.

Part of [Audit ledger](audit-ledger.md),
which lists every file of this record.

## B865: the check over the real sources was a list of seven shapes, and a list grows by a shape at every re-check

Status:
fixed on 2026-10-24 (UTC),
found by the re-check of fix round 14 of the Claude route
(re-check 13:
its adversarial scenarios of builder references,
its eight mutants of the check itself
and its mutants of the nested bindings).
The design that landed is open to the owner's veto.

The defect:
the check of B864 named seven shapes the search leaves unread and reported those.
Re-check 13 wrote scenarios of references the seven did not name
and the check reported none of them:
a builder held by a class field,
a default parameter,
a getter or a method that returns the call,
an assignment,
a logical operand or a conditional;
a module path handed to `createRequire(...)`,
`require.resolve` or `import.meta.resolve`;
a namespace import handed to a call or stored in an object;
and a load held in a form the search does not bind.
Each of them reaches a builder,
or can,
without the search listing the file.
The seven shapes were a blacklist:
the header said what the search misses
and the check said where the header's list is found,
so every shape nobody had thought of passed both.
The rule B864 drew from its own defect
("a header that lists shapes as unread is paired with a check that names them")
gave that list the status of a guarantee.

The fix
(the cases for the nested bindings and the members of a builder are `fb6c4347f4`;
the red cases for the check are `d6b488f4af`,
seven of them failing against the check of B864,
and the check is `4527b378c1`;
the commits after it are `56823304f1`
(a control per read position,
the import-equals case),
`35cea3a28b`
(one list of the kinds that hold no value,
and the header),
`fbbb1b92e6`
(a call method,
a type-only use,
a template path,
a namespaced carrier,
the memo contract),
`7bb024afce` and `dfa39ec392` (the cases that the first runs of the round's mutants called for),
and `95074ffdca`
(a load taken apart by an array pattern,
and a dead filter removed)):

- The check is a whitelist anchored at the module-load boundary.
  It does not list shapes that are unread:
  it lists what the search reads,
  and reports every other place that holds a builder,
  a module that carries one,
  or a string that names such a module.
  It makes two totals.
  The first total reads every string that names a module that carries a builder
  (a module that declares one,
  exports one by its own names,
  exports everything of a module that carries one,
  exports one as a namespace,
  or exports an object with builder members,
  closed until no module is added):
  such a string must be the source of an import or of an export from,
  the one literal path of an `import()` or a `require`,
  or sit inside a type.
  Any other is `module-path-outside-a-load`.
  A load whose path is not one literal is `computed-load-path`,
  and a literal load that the search does not follow is `load-in-unfollowed-form`.
  The second total reads every chain of names that stands for a builder,
  or for a module or object that carries one,
  and the node that holds it:
  a place the search reads is silent,
  any other place is `builder-outside-a-read-position` or `module-binding-outside-a-read-position`
  with the kind of node and the field,
  a result that leaves a function the search does not list as a builder is `builder-result-returned-outside-a-wrapper`,
  a place in a namespace,
  an enum,
  a parameter property,
  an export assignment or an import-equals declaration is `builder-in-typescript-value-construct`,
  and a function that holds a format and that no declared function holds is `format-in-undeclared-function`.
  Eight shapes in all,
  where B864 had seven.
- The header says what the check reads and does not list what it leaves out.
  The sentence "The scan over-reads" of the header stays,
  and the new text closes with the statement that the check is a net:
  it reports every place it does not understand,
  whether or not a producer hides there.
- `format-in-undeclared-function` skips four files,
  the ones the unit test names as writing the format without building a stage's answer schema
  (`NOT_STAGE_SCHEMAS`:
  `bench/src/corpus-run/model-health-probe.ts`,
  `provider/src/anthropic-tool.ts`,
  `provider/src/chat-contract.ts`
  and `provider/src/claude-body.ts`).
  The header names them,
  and a case holds that a file named is skipped and the same file unnamed is reported.
  The other seven shapes are checked in those files too.
- Limits,
  named in the header and held by cases:
  a call that takes the value of a builder's call as an argument ends the walk,
  since the value is taken as the place the schema is sent;
  `.name`,
  `.length` and `.toString` read off a builder are reported,
  as are `require.resolve` and `import.meta.resolve` of a literal path;
  and a class method that returns a builder's call is reported
  where B864 had it silent.
- `buildersIn` runs once for each family the suite reads.
  An instrumented copy counted 280 calls and 131 searches over one run of the suite,
  of which 7 calls and 2 searches were for the real sources
  (the 1579 files,
  and the 1583 of the case that adds four fixture files).
  The answer is memoized by the modules' numbers and the files named as no stage schema,
  and a case holds that a file named,
  an unnamed file
  and an edited file give three answers.
- The cases:
  one scratch fixture file per class of adversarial scenario and per read position
  (`claude-schema-whitelist-*.test-fixture.ts`),
  each stating the files the search lists and the places the check reports;
  a control for each read position that proves the search reads it by itself,
  so the silence of the check there is the search's;
  and the real sources,
  which still hold none
  (1579 files,
  none reported,
  2.0 s cold and 1.0 s with the builders found).

Mutants:
`mutants.txt` of the round holds 292 rows.
They are the 39 of re-check 12,
the 69 of fix round 13,
30 of round 14 that still apply,
143 of the check
(re-check 13's eight aimed at the new code,
and one or more for each position and each branch of the two totals),
and re-check 13's 11 of the nested bindings.
286 fail at least one named case.
Six do not:
S02,
S02b and S14h are the rows of B864 that change nothing on any input or only on a type named as a builder,
G-X03 changes time only,
G-F15 reads names at the top of a file,
whose range holds no node,
and G-C07a walks on from a name that stands for neither a builder nor a carrier,
which reaches no builder:
a probe of a nested object path,
a static class member,
a namespace of a module that carries no builder
and a member of a declared function gave the same places with and without the mutant,
and a path that reaches a builder through a bound name makes that name a carrier.
The claim about G-C07a is wrong,
and B866 below holds the correction:
the survivors this entry could call equivalent are five.
The first run left 34 alive:
the cases that the survivors called for are `7bb024afce`,
`dfa39ec392`
and `95074ffdca`,
and the mutant G-B02 showed that a filter of the default through a star was dead code,
since resolving a name through a star refuses the default,
so the filter is gone and the case stays.

The test file took 3.68 to 4.00 s before and 3.18 to 3.33 s after,
three runs each,
alternating,
on a host with other sessions running.
The runs are too few to say that the difference is more than the order of the spread,
and no run held the memoization of the builder search apart from the rest of the change,
so no cause is claimed for it.
An earlier version of this entry said the file was faster because the search is memoized:
that was a cause without a control,
and B869 measures the file again.

Prevention rule
(replacing the one of B864):
a check that backs a header over the real sources is a whitelist
anchored at a boundary the language fixes
(a module load:
the strings that name a module,
and the chains of names that start at one),
not a list of the shapes found so far;
it is silent where the search reads and reports everywhere else.
It has one scratch fixture per class of reference and one control per read position,
a red commit that fails against the check it replaces,
and mutants for every position and every branch of its totals.

Cache versions:
none moved.
The change is to test fixtures.
The search for a moved cache version over the commits of this fix prints nothing:

```sh
# package/translation-repair, over 6cc75dfbc9..HEAD
git log --extended-regexp -G '_CACHE_VERSION = [0-9]+' 6cc75dfbc9..HEAD -- package/translation-repair
```

The same search over `c265e8df95~1..c265e8df95` prints one line,
so the search can print.

Recurrence:
`mistake-prevention-guards-searches-and-claims.md`,
"Guards that cannot fail".

## B866: the mutant G-C07a was called equivalent by an argument, and it is not

Status:
fixed on 2026-10-24 (UTC),
found by the re-check of fix round 15 of the Claude route
(re-check 14:
its probe of a namespace member whose head is a module outside the sources).
The design that landed is open to the owner's veto.

The defect:
B865 says the mutant G-C07a
(the walk goes on from a name that is bound and stands for neither a builder nor a carrier)
reaches no builder and counts it among the six survivors that change nothing.
The body of `dfa39ec392` called it equivalent before any probe,
and `edf6323eb3` then wrote a probe of four inputs and said the probe settles it.
The probe held no input of the kind that matters:
a member of a namespace whose module is outside the sources,
stored in an object,
handed to a call,
or held in an array.
The check walked on from such a member only when its head was free,
so the member was silent in all three places,
and the mutant that removes the guard changed that.
Round 15 therefore left five survivors,
not six,
and the sixth was a gap of the check.

The fix
(the cases are `cd6233ddfd2ab53828199f2095167a24264cf373`,
which fails one case against round 15's check,
and the check is `ac1344a4353fa95d469adddce3f3555fde1073ad`,
which removes the `isFree &&` guard of the chain walk):

- A namespace member of an outside head stored in an object,
  handed to a call
  and held in an array are reported as `builder-outside-a-read-position`.
  The same shape with a free head and with a named import from the outside module are controls,
  and the member called in place is read by the search.
- The mutant is now the other way about:
  `G-C07a-guard-restored` puts the guard back and is killed by one case.
- The survivors of round 15 that stay are five:
  S02,
  S02b,
  S14h,
  G-F15
  and G-X03.
  `fix16/ledger-notes.md` holds the bodies of `dfa39ec392` and `edf6323eb3`,
  neither of which is amended.

Prevention rule:
a mutant is equivalent when a probe that holds an input of every class the mutant can reach gives the same places,
and the probe lists those classes first;
a probe of four inputs chosen by the argument it is meant to test settles nothing.

## B867: a builder's result that goes where nothing answers for it was silent: a third total

Status:
fixed on 2026-10-24 (UTC),
found by the re-check of fix round 15 of the Claude route
(re-check 14:
26 inputs the check reported nothing for,
and the senders that mutant G-F06b flagged).
The design that landed is open to the owner's veto.

The defect:
the walk of a builder call's value ended without a report when the value was handed to a call,
put in `new`,
stored in a member,
a field
or `this`,
held in an array or a map,
spread,
pushed,
wrapped in `Object.freeze`,
`Promise.resolve`
or `JSON.parse`,
or held by a constant that hands it on.
A function that freezes,
copies or boxes the schema its caller asked for is neither a wrapper nor reported,
and the file that supplies the arguments is not listed as a producer.
The two totals of B865 see every load and every reference to a binding from a load,
so this was a place they do not look.

The fix
(the red commit is `2f09e42d70e6f3ceca2b8e500c6663412db07e48`,
five cases failing,
named in `red1-failing.txt` of the round;
the check is `39535a82bf9c84960388bfd426e47a80b3a7f679`;
a top-level constant and the windowed rounds are `c2e4b9217dfa36ca243deb882219a255105a2ab3`
and `bd53c8d8fe3e9ea62cc7362d124c7263f1caa280`):

- The third total reads the places where a builder call's value goes.
  A value that reaches a `return`,
  an arrow body or a `yield` of a function the search lists as a wrapper,
  a constant whose every reference meets the same rule,
  or a property of an object that is the argument of a sender,
  is silent.
  Marks that leave the value as it is
  (parentheses,
  assertions,
  a non-null mark,
  an optional chain)
  are read through.
  Every other destination is `builder-result-outside-a-destination`,
  with the kind of node and the field.
- A sender is a function of a module and one parameter of it,
  where a stage hands the schema it makes to the bench or to a window of rounds.
  The family has two,
  both named by module,
  by function and by plain key of the object argument:
  `gatherThroughCache` of `corpus-pass/src/corpus-run/pass-consistency-gather.ts`
  and `runWindowedRounds` of `stage/src/stage-windowed-rounds.ts`,
  each with the parameter `responseFormat`.
  The callee has to resolve to a function of that module and name,
  so a function of the same name elsewhere,
  a computed key,
  another parameter
  and a nested one are reported,
  and so is a `new` of the sender.
- The walk follows a top-level constant,
  with the program as its scope,
  where it had stopped.
- The 26 inputs of re-check 14 that were silent are all reported
  (the list is in `fix16/adversarial.txt`);
  the 1579 real sources report none.

Prevention rule:
a walk that follows a value ends in a destination the check lists,
never in a call it does not understand.

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
  `.constructor` and `node:vm` are not reported.
  `rg --word-regexp 'Function|eval|createRequire'` over the non-test sources of the family finds five hits,
  all prose,
  none code.

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

Records corrected:
the body of `4527b378c1` says its header "is rewritten in the next commit",
and the header was rewritten in `35cea3a28b`,
two commits later;
the adversarial table of round 15 labels its row B19 "mjs path import (non-ts specifier)",
and the input is `import('node:fs')`,
which reaches no builder,
but not for the reason its name gives.
`fix16/ledger-notes.md` holds both,
and B864's sentence "2.0 s with them" is "2.0 s without them".

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

Recurrence:
`mistake-prevention-guards-searches-and-claims.md`,
"Guards that cannot fail".
