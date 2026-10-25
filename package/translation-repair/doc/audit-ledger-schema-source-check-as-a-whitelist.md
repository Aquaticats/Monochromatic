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
  (the 1579 files of that round,
  1620 on the head of fix round 18,
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
  (1579 files then,
  1620 on the head of fix round 18,
  none reported,
  2.0 s cold and 1.0 s with the builders found).

Mutants:
The mutant table of the round holds 292 rows.
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
and the entry titled "B866:
the mutant G-C07a was called equivalent by an argument,
and it is not"
holds the correction:
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
  The bodies of `dfa39ec39274531438ffcb7fabc74b8c520a33b0` and `edf6323eb3f9be2c197c33cfde140a2e527a7189`
  are not amended,
  and `git show --no-patch --format=%B` prints them.
  The first says the mutant
  "walks on from a name that stands for neither a builder nor a carrier and is bound,
  which can reach no builder".
  The second says a probe fixture of
  "a nested object path,
  a static class member,
  a namespace of a module that carries no builder
  and a member of a declared function"
  gives "byte for byte the same places with and without the mutant".
  What both leave out is the input that mattered:
  a member of a namespace whose module is outside the sources,
  stored in an object,
  handed to a call
  or held in an array.

Follow-up of fix round 17
(the prevention rule of this entry was applied to every mutant that stayed alive):
the table of the round has 142 rows.
They are the 63 of round 16
(five re-anchored,
because the code they mutate moved:
D05 into `claude-schema-blind-reads.test-fixture.ts`
and S01 to S04 into the shared part of the sender check),
the 13 of re-check 15
(X1 to X11,
X3b
and X12;
six re-anchored:
X2,
X3,
X3b,
X4,
X7
and X11)
and 66 for the branches the round added.
No row failed to apply,
and none failed without naming a case.
The first full run,
on `a3a9fe5a8275020d9eb37933247d294cd1f18940`,
held 144 rows and left 23 alive.
Their dispositions:

- 14 got cases that pass against that code and fail under the mutant
  (`c0ab8ffe573abbbb7a21be8d6f667758b51ce0d2`);
  among them are
  a name bound by a `switch` case,
  a class expression and a parameter property;
  a `var` of a nested function and a `let` of a block,
  which do not bind the sender's name;
  an object given to `.bind` in the `this` slot;
  a template literal in the tag position of a tagged template;
  two modules that export everything of each other;
  an `export *` that carries no default;
  a member of a namespace computed from a name;
  a name a function binds again;
  a value that leaves by a function of another file;
  and a function that builds from its parameter and calls no sender.
- Two were conditions the syntax tree implies,
  and `a642a09a9d45195eaa36ed37aa705ab2b084ea63` drops them.
- One lets a builder or a wrapper be a forwarder as well.
  The file listing does not show it,
  because the callers of a wrapper are listed already,
  so `1d3bd37002b264b9b272afc76f17f1a78dd27aef` adds a case and a test over the list of forwarders.
- Six are argued equivalent with a control,
  and the final run,
  on `1d3bd37002b264b9b272afc76f17f1a78dd27aef`,
  left exactly these six alive out of 142.

The argued six:

- X3 and X3b remove the test for a computed member of the callee.
  A callee that computes a member from a name resolves to no function,
  so the test is redundant,
  as re-check 15 found.
  The controls are `(gatherThroughCache as any)[k](...)` and a namespace import called as `g[k](...)`:
  head and mutant give the same report for each.
  The first positive controls chosen,
  a callee that is no chain of names and a mutant that accepts any parameter,
  gave the same output on head and mutant too,
  which showed they could not differ,
  and they were replaced.
  The mutant that drops the test and also drops the computed member from the path changes the first control,
  and the mutant that reads the computed member as the sender's name changes the second.
  The test stays in the code.
- X11 removes the test that a key is not empty.
  A key that is no name gives an empty string,
  and no sender's parameter is empty.
  The control is a sender called with `{ 0: ... }`,
  `{ '': ... }` and `{ responseFormat: ... }`:
  head and mutant give the same two reports.
  The positive control drops the test and makes every parameter match,
  and then the numeric key is accepted.
- The prefilter of the nodes that load a module changes only how many entries the index of loaders holds
  (2 on the head and 7 on the mutant for the control,
  the duplicates of an `await` around a load),
  and the found files and the reports are identical.
  The 1579 real sources hold no load of a file of the family
  (1620 on the head of fix round 18,
  none loaded by `import()` or `require`).
  The positive control,
  which stops following loads,
  removes both reports.
- The memo of the context of a file changes only time:
  the control gives the same files and reports,
  and three alternating runs took 4.57 to 4.68 s on the head and 4.60 to 4.73 s on the mutant,
  inside the spread of 0.11 and 0.13 s.
  The memo could go.
- The prefilter of the files that refer to a sender changes only time:
  the control and the list of the forwarders over the real sources are identical
  (three then),
  and the positive control,
  which drops the callers from the listing,
  changes the control.

Follow-up of fix round 18
(re-check 16 ran 15 mutants on `b6540001c6c69b843a69f4e7ba9cf13e0b37ae26`:
six alive that an input can tell,
two that none can,
and seven killed):

- `a0ac36bf6ca9e0acec34ca85981d2f752c47c453` pins the six and the import-equals declaration of `vm`.
  Each case passes on the code as it is and is killed by its mutant in the table of the round.
- `86a6fa40b6cd1bffb3f71d914f25256eee201b23` removes the two that none can tell:
  a filter of readers with no node (the step of a node outside the tree takes the value nowhere)
  and a test that a statement is an export before its variable declaration is read
  (the parser puts a variable declaration under `declaration` of an export declaration only).
- `a1ecf74b681036e17a5444b724c5724e8b003ea7` removes four more guards that no input reaches.
- `7a2cecb0cebdf2920622b45bc8aa19e25717d9e9` and `d87291ce4045b28194d076f6457d435455f6966b`
  pin the survivors of the first run of the round's own table that an input can tell (39 of 246).
  The last mutant,
  a fallback of any unexported name to `export =`,
  was alive on `4d2b5d9cf8939edf8c322657b4c9c7e409d3a7aa` and is killed on `d87291ce4045b28194d076f6457d435455f6966b`.

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
which the body of that commit names:
a result handed to a call that is no sender,
a result handed to `new`,
the sender the family names,
a result stored in a member,
a field,
`this` or a computed member,
and a result held by a constant whose reference hands it on;
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
- The 26 inputs of re-check 14 that were silent are all reported:
  a specifier outside the sources
  (M09,
  N13),
  `eval`,
  `createRequire`,
  `module.require`
  and an aliased `require`
  (N01 to N04),
  a builder's result handed to a call,
  stored in a member or a field,
  pushed,
  boxed in a `Map`,
  parsed back from JSON
  or put in a template
  (M29 to M36,
  M40,
  N18 to N20,
  P16
  and Z1 to Z3),
  and a member of a namespace of an outside module stored,
  handed to a call
  or held
  (O01 to O04).
  The 1579 real sources report none
  (1620 on the head of fix round 18,
  none reported).

Follow-up of fix round 17
(re-check 15 found 20 inputs that stayed silent besides three stated limits;
the red commit comes before each fix,
and every fix makes the failing places of its red commit pass):

- A builder's result in a substitution of a tagged template is reported
  (red `e5ed26e767b0b82c5f76100c75db90c9fe940bad`,
  fix `47f78cd3ab3c21f849d728fd83435bfceca98023`;
  inputs B02,
  E22,
  E23,
  H10,
  A37 and C15).
- A call is no call of a sender when a scope around it binds the sender's name
  (red `6fc75948cd92f6c0cc6d443b7db64c127eb9efbd`,
  fix `8c1eaa05e8c5200f8bfeb43d6a3cb5b5184f7956`;
  inputs A17 to A20:
  a parameter,
  a local function,
  a nested function and a catch binding).
- A builder's result is accepted only at the argument its sender reads
  (red `494f5e59f6e064371fd2bfcd41d49cff96d0029c`,
  fix `270602318f0a3e2e41d9fa5a291da090d2a61e8a`;
  inputs A09 and H01:
  the second argument,
  and the `this` slot of `.call`),
  with `A10` and `A12` as controls that stay accepted.
- A constant a file exports is followed into the files that read it
  (red `8be03d18d8b3913913037b6bfaed219835277ac0` and `ae150f8c6e23e9076b40c9bbdfbf34cf4d294311`,
  nine places in all;
  fix `0d917b63617491de2409d3edaa8b851b18856e24`).
  The senders are three now:
  `assertSchemasSendable` of `pass-schema-check.ts` joined the two of round 16,
  because the real shape of `CONSOLIDATE_GATE_RESPONSE_FORMAT` ends in the call that checks the list of producers.
  The walk follows names an import gives,
  a re-export by name and by star,
  the member of a namespace import and `import()` or `require` of the file.
  A namespace read as a whole,
  a namespace export and a load of the file are reported at once,
  and a cycle of stars ends through a set of the pairs of a file and an export that were followed.
- A function that hands a sender a builder call made from its own parameters,
  a forwarder,
  gets its callers listed
  (red `9b4b423db2c33a73d04571dd55b5c593d73a02ec`,
  fix `a3a9fe5a8275020d9eb37933247d294cd1f18940`).
  The ruling is the lead's and is open to the owner's veto.
  The reading is wide:
  a name made from a parameter through the function's constants counts,
  and the sender call and the builder call need not be linked.
  Measured over the 1579 real sources,
  there were three forwarders in that change
  (fix round 18 reads them until no more are found,
  and the follow-up of that round holds the figure):
  `askConsistencyBench`,
  `askCarryChoice`
  and `askRunOff`,
  and their callers are `pass-consistency-bench-decide.ts` and `pass-consistency-carry-choose.ts`.
  The narrow reading,
  which asks for a parameter by its own name in the builder call,
  finds `askRunOff` only and lists `pass-consistency-bench-decide.ts` alone;
  the wide reading adds `pass-consistency-carry-choose.ts` and no other file.
  The wide reading is the one that lists,
  and the cost is the over-listing of a function that sends one schema
  and builds another.
  The next hop was not followed in that change.
  The measure of the chain of callers that pass their own parameters on found two functions at each of hops two to six,
  and the chain was not empty at hop six,
  so following it was thought to list most of the consistency pass without naming a file that writes a schema.
  The commit body added that a seventh hop was left unmeasured.
  Fix round 18 follows the chain to its end,
  and the measure it made refutes that reason (follow-up of fix round 18 in this entry).
- The inputs E01 and E02 are listed,
  not reported:
  the caller that supplies the arguments joins the list of files,
  and the sender's answer is not a schema.

Follow-up of fix round 18
(red commit before each fix):

- The forwarders are the fixpoint of the reading
  (red `2fbd1157198c54850eb53c184e7e2410b236ba33`,
  fix `f3d581b26e9c6dccaae864720f06a21bf6afe86f`).
  A function that passes its parameter to a forwarder is one,
  with no limit on the chain.
  Over the 1620 real sources there are 15 forwarders called from 9 files of the consistency pass,
  and the check reports none of the 1620.
  The reason B867 gave for stopping at the next hop did not bound the chain:
  it holds 15 forwarders,
  added over eight rounds (follow-up of fix round 19 in this entry).
  The listing adds files that pass a schema's arguments on and write none,
  the over-listing that was predicted and the ruling accepts.
- A forwarder written as an arrow,
  a function expression or a class method
  is reported where it is written as `forwarder-in-undeclared-function`,
  since no declaration names it for a caller to be listed by
  (red `eb67d3c766a6a82cc1801ae90fdf60dcb5651e66`,
  fix `781fab9e753ae3243a0cf8016da0a8a1a0f95cae`).
- A namespace is a scope for the sender's name and an import-equals declaration binds its name
  (red `b88b918456de77a3e55d28f16b71a77bc41229ee`,
  fix `4d82f4d8f7518f43835db77d429014ea7e8d4481`).
  The require form binds the module as a whole,
  so the walk across files follows a constant read through it
  (red `c62ec094f6c04a88f56825190ccb7d4100ead4cf`,
  fix `3cfa3997012f02de46cdb4874fef843ce3b8966f`).
- That fix stopped listing the caller of a forwarder exported with `export = mk`
  (red `e33e81212ab259bbd9575966208d044e76036115`,
  fix `9da0e1386f19031d1eb505e20f7bb0764ad49dcd`).
  The declaration had bound no name,
  so the call read as a free name that matched by accident,
  and with the binding it read as a namespace.
  `export = name` is now an export named `export=` of the module,
  and a whole-module import of a module that has it binds its value.
  A second red commit,
  `230854fb77a64f42a43ec59d619de18d7d4ba08c`,
  showed the default import and the namespace import of such a module still unlisted,
  and a top-level `const Boolean = makeFilter()` read as the global function;
  `4d2b5d9cf8939edf8c322657b4c9c7e409d3a7aa` fixes both.
- A builder's value handed to one of `every filter find flatMap forEach map some reduce`
  goes on into the parameters of the callback
  (red `d4902d22ab2aa8c719de7a8bdee6fa772348eb9e`,
  fix `cdf4516f463ff966890f7b73016a4e92fa87bc83`).
  A written function or a declared name is read,
  `Boolean` or `String` or `Number` that no scope binds keeps nothing,
  and any other callback is reported as code the search cannot read.
  Stated limit:
  another method that takes a callback ends the walk silently
  (a sort callback and `findLast` stay silent).
- A name that holds the sender and is assigned again is no call of the sender
  (red `b76fce0c79e090d8773758138c8f3bfe5df18d30`,
  fix `daafeec74b532c907b5c7a049c5cf7687c87e0eb`).

Follow-up of fix round 19
(red commit before each fix):

- A forwarder is reported wherever it is read except as a callee
  (red `d3e62082d4d9a11d115eedb94f19de41291394cf`,
  fix `8039afa5afa902895438f338688bd82332801c5a`,
  clean-up `773a6235186481f022efae6790bfc13bcf2e8b45`).
  A wrapper held by an object,
  an array,
  an argument or a comma is reported where it sits,
  and a forwarder held there was not,
  so a file that took the function from the object could call it with the arguments of a schema
  and no file was listed.
  A forwarder is now read as the callee of a call,
  of `new` or of a tagged template,
  through `call`,
  `apply` and `bind`,
  as the whole value of an export assignment
  and through the members `name` and `length`
  (the logger tag of the repo is `fn.name`).
  Any other place reports `forwarder-outside-a-read-position` with the node and the field.
  Eleven cases hold it,
  and ten of them failed on the red commit.
- The 1620 real sources give one report of the new shape:
  `corpus-pass/src/corpus-run/corpus-pass.ts` hands `settleEntry` to the pass as its `settle` dependency,
  and `corpus-pass-run.ts` calls `input.settle(...)` with a corpus pair and the settings of the run.
  The place is named in the test by file,
  shape and position,
  with the reason,
  in `claude-schema-named-places.test-fixture.ts`,
  and a second case fails when a name outlives its place.
  This is a design choice that is open to the owner's veto:
  the alternatives are to report the place and change the source so that no forwarder is handed over as a value,
  or to read the callers of `input.settle` as callers of `settleEntry`.
- A parameter property and a target under a type mark bind their names
  (red `9b11871b4291139f4eae5e265c262d63023a83ad`,
  fix `d54c1c4da72109841db003ad6f7c08fdb850af60`).
  `constructor(private args)` binds `args` as a parameter,
  and `(g as any) = other`,
  `(g!) = other`,
  `(g satisfies any) = other`
  and `(<any>g) = other` assign `g`,
  so a sender held by `g` that is assigned that way is no sender.
  The compound forms `g ||= other` and `g ??= other` already assigned,
  and two cases pin them.
- The names made from a function's parameters follow more places
  (red `074bc98b8a5757bd819b74d853e68910a75a0317`,
  fix `3eec535445dc2b74d0e43b0efe545e0cba9e46be`):
  a constant,
  an assignment,
  the head of a `for...in` or `for...of` loop over a value,
  the parameter of a `catch` clause for a value that the `try` block throws itself,
  and the `arguments` object,
  and a chain is read at its head whether or not a member is computed
  (`list[0]`,
  `rec[key]`).
  Two flows are not followed,
  each with the reason the source shows.
  A name that reaches the builder through a field of `this`
  (`this.args = args` read as `this.args`)
  is a read of `this` whose value the walk does not model,
  and the check cannot tell it from any other field.
  A value thrown by a function that the `try` block calls is thrown where the walk reads no function body.
  Both are limits.
  A case holds the first,
  and a second case holds that a value no name of the parameters reaches lists nothing.
  No case holds the second.
- A callback written as a name is read as the function the file declares
  only when no scope around it binds the name again
  (red `7c71cccda8146b66e556c9e541551428445ff68b`,
  fix `512d81f6008e58d15254de0bd41593b3215b929d`):
  a parameter,
  a destructured parameter,
  a constant of the function and a `catch` clause now make it a callback the search cannot read.
- A qualified name that an import-equals declaration reads is a reference
  (red `7e987c281d6d1fa6ed41046c52842198c82e5d92`,
  fix `49cc0602d3d9d05d4017a1b64871bffa03f8f24c`).
  `import U = A.FORMAT` read a member of a namespace through a node that is no member expression,
  so a constant that the module exports and the declaration reads was never followed.
  Its first name is a reference now,
  and the constant is reported at the declaration
  whether `A` comes from `import * as A` or from `import A = require(path)`,
  and whether the declaration is exported.
  `import B = A` was reported already.
- A function written as an argument of a method that is none of the eight is reported
  (red `c1f5aeca8da411c03883fd08a2dbc85da6925389`,
  fix `fe3967005565e46731910dc3b29236d8aafc6e73`)
  as a callback that cannot be read:
  `sort`,
  `findLast`,
  `findIndex`,
  `reduceRight`,
  `toSorted`
  and a method of an object that holds the value.
  A call with no function written as an argument still ends the walk without a report.
  The case that returns a builder's call chained with `.then` has one report more.
- The rounds of the search for forwarders are bounded
  (`c03b9f77c843fff3dd2ff1ca5b244120fafa53f4`):
  a round adds only functions no round added,
  so no more rounds than the family declares functions can add one,
  and a round that adds one after that throws with a message.
  Fix round 18 saw the mutant that drops the test for a function already found as a hang
  and counted it killed by not ending.
- The rounds were measured on a copy of the head with a line that prints what each round adds,
  over the 1620 real sources:
  eight rounds add 3,
  2,
  2,
  2,
  2,
  2,
  1
  and 1 forwarders,
  and the ninth adds none.
  The chain is eight hops long and holds 15 forwarders.
  Fix round 18 wrote "15" for the length of the chain,
  and 15 is the count.

Prevention rule:
a walk that follows a value ends in a destination the check lists,
never in a call it does not understand.
