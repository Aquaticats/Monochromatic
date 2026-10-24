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
which reaches no builder by the way the paths are resolved
(an argument and not a measurement).
The first run left 34 alive:
the cases that the survivors called for are `7bb024afce`,
`dfa39ec392`
and `95074ffdca`,
and the mutant G-B02 showed that a filter of the default through a star was dead code,
since resolving a name through a star refuses the default,
so the filter is gone and the case stays.

The test file takes 3.68 to 4.00 s before and 3.18 to 3.33 s after,
three runs each,
alternating,
on a host with other sessions running.
It is faster although it holds more cases,
because the builder search is memoized.
The runs are too few to say that the difference is more than the order of the spread.

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
