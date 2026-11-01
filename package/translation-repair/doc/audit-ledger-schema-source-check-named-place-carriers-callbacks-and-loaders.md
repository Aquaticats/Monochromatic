# Audit ledger: the check over the schema sources, the named place, the carriers, the callbacks and the loaders

B953,
which continues B865 to B869 of
[Audit ledger: the check over the schema sources as a whitelist](audit-ledger-schema-source-check-as-a-whitelist.md)
and
[Audit ledger: B868 and B869](audit-ledger-schema-source-check-unreadable-code-and-surviving-mutants.md),
both at the limit of a file.

Part of [Audit ledger](audit-ledger.md),
which lists every file of this record.

## B953: the named place held the name of a function and not the function, and six reads of the sources were silent

Status:
fixed on 2026-10-25 (UTC),
found by re-check 19 of fix round 20 of the Claude route (no major finding,
6 minor findings and 9 nits).
The design that landed is open to the owner's veto,
and the ruling of the coordinator of 2026-10-25 on the named place stands:
the place must fail closed,
its key holds the module that declares the forwarded function as well as its name,
and every part is compared.

The defects and their fixes,
each with the commits that hold it (the red commit before each fix,
except the pins,
which are green because the code was right and the suite lacked the input):

- The named place `settle: settleEntry` of `corpus-pass.ts` held the bare name of the function.
  Red `b7b4aed3a487f190d0d3068dd6d817225ac2c1be`,
  fix `a7ab6da3c2e1023b9386b4633d6391c89b959183`.
  A second function of the same name,
  declared in another module and imported in its place,
  was a report the name stood for.
  Measured on the real sources with the import in `corpus-pass.ts` redirected to a new module that declares
  `settleEntry`:
  the check made one report and the test left none.
  A report of a forwarder read in a place that is no call now holds the key of the function (`file#name`,
  the key `keyOf` makes),
  the module that declares it and its name,
  so an import under another local name and a member of a namespace still give the key of the declaring module.
  A free name,
  a forwarder that no module declares,
  has no module,
  so its report holds the bare name,
  which no key equals and no name stands for.
  The place of the family's source holds the key of `corpus-pass/src/corpus-run/pass-entry-settle.ts#settleEntry`,
  and `isPlaceOf` compares it with the file,
  the shape and the position.
  Measured again with the same edit,
  the redirect to the new module reports 1 and leaves 1 (it left 0),
  and the 22 other variants of re-check 19 behave as before.
  Variants added in this round:
  the import redirected to a re-export of the declaring module,
  and the forwarder held as a member of a namespace re-export,
  leave the name on the place,
  since the key is read from what the chain resolves to;
  a module of the same name in another directory,
  and a free name of the same name,
  are reported and leave the name stale.
  The cases hold the new part:
  a report of a function of the same name that another module declares is not named,
  a free name is not named,
  and over imaginary files an import of the second module in place of the first,
  and an import of both with the second under a local name,
  leave the report of the second.
  The mutants that hold the bare name,
  the declaring file alone,
  or a key with no file for a free name are killed by 24,
  24 and 1 cases.
- Two mutants of re-check 19 on the comparison survived:
  a report that holds no function named by a place that gives one (M19-06),
  and the first place of the list taken in place of the one that stands for the report (M19-08).
  The cases are in the red commit of the named place and are green on its parent:
  a report that holds no function is not named by a name that gives one,
  and a report is matched with the name that stands for it and not with the first name of the list.
  Both mutants are killed by 1 case.
- The namespace of a module that declares forwarders and no builder was no carrier.
  Red `0708e65ed43c12d9e519de80b685ad8478630d7b`,
  fix `4bd6f1c9a5206a3136154fe00c5b9bb6fcb39177`.
  The check reads a file further only when a name in it can lead to a builder,
  a forwarder or a carrier.
  `ns.mk` was reported,
  because the member `mk` is itself a name the check knows,
  and `ns['mk']`,
  ``ns[`mk`]``,
  `ns[key]`,
  `Object.values(ns)`,
  `[ns]` and `{ ns }` were not,
  although the same forms are reported for a module that also carries a builder.
  `carries` in `standingAfter` now also holds for a namespace whose module declares a known forwarder (`declaresKnown`
  reads the keys of the forwarders).
  Seven cases failed on the red commit and two controls passed:
  a call of a member written as a name or as a string is read and its file listed,
  and the namespace of a module with no forwarder and no builder is accepted while a module of the same directory
  declares a forwarder.
  A namespace kept by an exported constant (`export const a = ns`) is followed into the file that reads the constant:
  `a.mk` held in a second file is reported,
  and `a.mk(...)` called there is listed.
  The type-marked form reports twice,
  the forwarder where it is held and the module where it is the object of the member,
  as the same form does for a module that carries a builder (an over-report that fails closed).
  A file that calls as a function a module whose export assignment wraps a forwarder in a call
  (`import s = require(...)`,
  `s(args)`) is reported as well,
  as the module read where the search does not look (`fdae57969e22c55342dab6e9ee7f7d4acf65d21d` holds it).
  The mutants that take no forwarder-only module for a carrier,
  every namespace for a carrier once a forwarder is known,
  and every module of the directory of a forwarder for a carrier are killed by 7,
  1 and 1 cases.
- A value stored through the parameter of a function written as an argument of a call was not followed.
  Red `10ae17fcadb23d73dd4e7abf3996709d25763a7e`,
  fix `d6a572c5d25efe8a7f648e9724e7578f4de39155`.
  In `args.forEach((value) => { holder.a = value; })` the parameter `value` takes the elements of `args`,
  and no link bound it,
  so `holder` was no name made from the parameter and the caller of the function was not listed.
  `claude-schema-blind-callback-links.test-fixture.ts` binds the names that the parameters of each arrow function or
  function expression among the arguments of a call or of `new` bind (under type marks too) to what the callee and
  each argument that is no function written in place read.
  The call is read widely,
  as a call of a function that is not known is read everywhere else,
  and the body of a callback is not read for the call:
  a callback whose own parameter comes from elsewhere binds nothing even when its body reads the parameter of the
  function.
  The red commit holds ten cases that failed and two pins that passed;
  the fix adds four cases:
  a callback of `reduce` that stores its second parameter,
  a callback under a type assertion,
  a callback of a call of `new`,
  and a callback whose own parameter is stored while only its body reads the parameter.
  Two flows stay unread,
  each with a case or a reason:
  a callback declared apart and handed over by its name (`args.forEach(put)`),
  which is a flow between two functions,
  and a callback held in an array and called with the parameter (`hs[0](args)`).
  This record said a third flow stayed unread,
  a callback handed to a function that was made from the parameter and stored.
  Fix round 22 corrects it:
  the call binds the parameters of that callback,
  because the stored function is a name made from the parameter,
  and the caller is listed (N42 of re-check 20).
  The link can bind the parameters of a callback that the call does not make from the parameter of the function,
  and then it lists a caller that the search did not need,
  which fails closed.
  Two inputs of earlier rounds change from a report to a listing:
  A13 of re-check 17 (`list.map((a) => send(a))`) and C08 of re-check 19.
  The arrow was reported as a function that forwards with no declaration;
  now the function that holds it forwards,
  its caller is listed,
  and the design of B867 reports only the outermost of the functions that forward.
  An arrow that reads a parameter of its enclosing function directly was never reported,
  for the same reason.
  The mutants of the link are killed by 13,
  11,
  2,
  1,
  1,
  12,
  1,
  1 and 1 cases (no link,
  callee not read,
  other arguments not read,
  `new` left out,
  arrow functions only,
  function expressions only,
  first parameter only,
  type marks not seen through,
  the whole call read).
- `process.getBuiltinModule` and `process.binding` were told only as the callee of a direct call.
  Red `59abf4225852cc95c898e65012fb92efb7b82866`,
  fix `f77a19e5d6f884250dd50511c9f239bbb865993f`.
  Both can be called through `call`,
  `apply` or `bind` (`process.getBuiltinModule.call(process, 'vm')` returns `vm` on Node 22 and later),
  as the last operand of a comma,
  as an arm of a conditional,
  or through a constant that holds the function.
  `claude-schema-blind-loader-reads.test-fixture.ts` reports every read of either member (whether `process` is read
  off the global object or not,
  under any mark) that is not the callee of a call,
  under the marks that leave it the same value.
  A direct call stays as before:
  told by its text when the string names `vm` or `contextify`,
  accepted otherwise.
  The limit of B868 that a loader assigned to a constant and called by another name is not told is withdrawn:
  the read of the member is reported where it is held,
  and the pinned case changed from accepted to reported.
  Re-check 19 listed six silent inputs (`.call`,
  `.apply` and `.bind` of the loader,
  the comma,
  the conditional and `process.binding.call`),
  and all six are reported.
  A feature test (`typeof process.getBuiltinModule`) is reported,
  which fails closed.
  Limits that stay,
  each with a case:
  a pattern that takes `binding` out of `process`;
  `process` under another name (a constant,
  a default import or a namespace import of `node:process`);
  a name imported from `node:process`;
  `internalBinding` read by `call`;
  and a variable or a spread as the specifier.
  The real sources read neither member and give the same report as before.
  The mutants that report no such read,
  report a direct call too,
  do not see through type marks,
  take neither member for a loader,
  take the member of any object for one,
  or leave such a file unread are killed by 4,
  6,
  1,
  1,
  3,
  1 and 3 cases.
- The key readers of the constructor of a function were told only when written with a dot on a plain identifier.
  Red `928ce59c7eb745d8ccfcbc5bead46db203ad4653`,
  fix `a1c78ef41bc4494afeef31b5a7dbf60b18fb7990`.
  `Reflect['get'](f, 'constructor')`,
  ``Reflect[`get`](...)`` and `Object['getOwnPropertyDescriptor'](...)` are the same call as the dotted form,
  and so are `(Reflect as any).get(...)` and `globalThis.Reflect.get(...)`.
  The method is read by `memberNameOf` and the object by `builtinNameOf`,
  the helper of the storing calls,
  moved to `claude-schema-blind-global-values.test-fixture.ts`.
  Found while judging the 20 inputs that re-check 19 wrote and did not run:
  K01 and K03 were silent,
  and are reported now.
  The others that are silent are limits with a case each:
  `Object.getOwnPropertyDescriptors(f)`,
  which returns every member,
  and a pattern in an assignment that takes `constructor` (`({ constructor: c } = f)`).
  A function bound twice and called at once (`mk.bind(null).bind(null)(...)`) is reported although it is called,
  because the unwinding of `bind` handles one level:
  an over-report that fails closed.
- `Object.assign` under a type mark or read off the global object stored in no object.
  Red `966aa797df445c6c2ce5b3ca760809a96c214fcc`,
  fix `e2294da9b07d375f6e4ba585118f444f68a7e045`.
  `namesStoredByCall` looked the storing statics up by the identifier of the callee object,
  and `(Object as any)`,
  `Object!` and `globalThis.Object` are no identifier.
  The path of the object is read now,
  and an object that is a member of another object,
  or of a global of another name,
  gives no name.
  Stay outside the list,
  each with a case:
  the storing function under another name (`const copy = Object.assign`,
  `const { assign } = Object`),
  `Reflect.defineProperty`,
  `Object.setPrototypeOf`,
  and a method of another object named `assign`.
  The mutants of the name (the identifier,
  any member of the global object,
  the last name of any path,
  the first name of any path) are killed by 3,
  2,
  2 and 2 cases.
- Four mutants and one claim of a commit message had no case (`5d788e1995bd69d785b4f4c543f2b1849c095ca8`,
  `5d893dce21fa26038fbaf0c46cf25305d43476a3`).
  The commit `d0e3217f526770d3d67910cd33bc7d9f5d2c9868` says that the superclass of a class and the default of a
  parameter count in a declaration inside a function,
  and no case held it.
  Three cases list the caller for `class Box extends (args as any) {}`,
  for a parameter that defaults to the parameter,
  and for a destructured default;
  the mutant that reads only the body of the declaration (M19-31) is killed by 3 cases.
  The mutant that takes the statics of `Object` for those of any object (M19-14) is killed by 4 cases,
  and the mutant that takes any string that starts with `context` for the binding that makes scripts (M19-25) by 1.
  The mutant that keeps the reports of a function once for each start offset (M19-11) mutates the key of the reports
  to the shape and the offset,
  which is not the code of the head any more;
  the row is retired with the code it mutated,
  and the key is the subject of the entries on the key in this list.
- The reports of a file are kept once for each node and shape,
  whatever they hold.
  Red `07eb7a6e0bae095921b87c337f01f9bab2689050`,
  fix `4ae811e77c4323b0e146252fcd84b98c7fd45a0a`.
  A report that holds no function was kept once for each line and shape,
  so a named place of such a shape could have hidden a second report on its line;
  no such place exists,
  and the whitelist case shows the change on a real shape:
  the line `registry.set(contestResponseFormat, contestResponseFormat,)` holds the builder twice in a position the
  search does not read and gives two reports now in place of one.
  The key was the shape and the start offset of the node,
  and `4ae811e77c4323b0e146252fcd84b98c7fd45a0a` says that a node has one offset,
  one shape,
  one position and one function in every input.
  A control on the mutants of the key refuted it:
  the name `Function` and `Function.prototype.constructor` start at one offset,
  so the key folded two nodes into one report,
  and the mutant that adds the position to the key differed on that input (K10 of re-check 19).
  Red `882bcc9d240efbea96fb4658818c8fad1083206e`,
  fix `d1e5e454776e70793a042bc22e6f736f1044081e`:
  the key is the shape,
  the type of the node and its start and end offsets.
  `(ns as any)[key]` gives two reports,
  as one expression may:
  the module as the object of the member and the member computed from a name.
  The mutants that key by line and shape,
  by shape and position,
  by offset alone,
  by line,
  shape and offset,
  and by shape and offset are killed by 7,
  45,
  2,
  1 and 1 cases.
  The mutants that add the position or the function to the new key survive the suite and are equivalent:
  the control is the table of the round.
- Y-E1g,
  the mutant that makes an export assignment of an expression that is no name export the name of nothing,
  was argued equivalent in rounds 18 to 20.
  The comparison on the head of this round found it differs on M09 of re-check 17 (`export = wrap(mk)` and a file that
  calls the module),
  where the head adds the report of the call of the module that the namespace as a carrier makes.
  `fdae57969e22c55342dab6e9ee7f7d4acf65d21d` holds both reports and the mutant is killed by 1 case.
- A condition that no sender parameter can equal is removed (`1ff3eae4cb039905d9ea290a890bd81e42c003e1`).
  `isSenderProperty` returned false for an empty key before it asked `callsSender`,
  which compares the key with parameters that are all non-empty names.
  The row X11,
  argued equivalent in rounds 17 to 20,
  goes with the code it mutated.

Reads that report more than once or more than the shape needs,
all of which fail closed,
and that no case removes:
a forwarder under a type mark that is held (`[(mk as any).bind(null)]`) gives two reports,
the element and the member object;
a member of a namespace under a type mark (`[(ns as any)['mkb']]`) gives two reports for one expression;
a function bound twice and called at once is reported although it is called;
a feature test of the loader (`typeof process.getBuiltinModule`) is reported;
a key of an object that a sender takes,
written as a template or as a computed string (`{ [`responseFormat`]: build(args) }`),
is reported while `{ "responseFormat": build(args) }` is read.
The inputs are n4 and n5 of re-check 19 (H14,
I04,
S01,
S02) and the multi-bind inputs of its file of 20.
A report that fails closed costs a case or a note in the file that holds it,
and a missed report costs a hidden send,
so none of them is removed.

The mutant table of the round has 417 rows on `a1c78ef41bc4494afeef31b5a7dbf60b18fb7990`,
and 418 rows after the rows that mutate the key were split,
on `fdae57969e22c55342dab6e9ee7f7d4acf65d21d`
(the first full run had 417 rows:
406 killed by a case,
one killed by not ending and 10 survived;
this record gave 418 for the first head,
which was the count of the later table).
The breakdown that follows is that of the table of 418 rows:
354 of the 356 rows of fix round 20 (X11 goes with its code and S20-06 is replaced by the rows on the key;
six rows are at a new anchor),
24 of the 32 mutants of re-check 19 of its own that still apply (M19-09,
M19-10 and M19-11 are retired,
and the five that survived are among the new rows),
and 40 for the branches and comparisons this round added.
The rows that mutate the key of the reports and the row of the export assignment were run again on
`fdae57969e22c55342dab6e9ee7f7d4acf65d21d`,
because the key and the cases changed after the table.
The merged table reads:
409 rows killed by a case,
one killed by not ending (N-R3e:
no exit within 300 s,
as in rounds 18 to 20),
and 8 survived.
The whole table was then run again,
in one capped unit,
on the code of `fdae57969e22c55342dab6e9ee7f7d4acf65d21d` (the documents that follow it change no code),
and every row has the verdict it has in the merged table.
The live rows of fix round 20 (S20-16,
S20-17,
S20-18,
S20-31,
S20-40 and S20-42) stay killed.
The eight survivors are equivalents.
X3,
X3b,
N-R5w,
N-R5x,
N-R3f and Y-F5r are the six argued in rounds 17 and 18,
and each was run again with its control on a copy of the head and on a copy with the mutant:
the output is the same
(N-R5w differs in the count of loader entries on the control of the loads and in nothing else:
the files found and the reports are the same),
and the mutant that changes the same control gives a different output.
R21-n1-key-by-node-shape-and-position and R21-n1-key-by-node-shape-and-held are the two new ones:
the control of two reports on one line and the real sources give the same output,
none of the 1112 adversarial inputs differs,
and the mutant that keys by line and shape,
which is killed by 7 cases,
differs on 31 of them.
The runs were sequential in a user unit capped at 8 GiB of memory and 800% of the CPU.

The adversarial inputs were run on the head of this round and on the four heads it is compared with.
The files hold 1112 inputs:
the 434 rows of the 430 inputs of re-check 17 and before,
the 260 of re-check 18 and its 7 probes,
the 46 of fix round 20,
the 224 of re-check 19 and its 20 that were written and not run,
and 121 of this round (a namespace of forwarders,
the parameters of a callback,
the loaders,
the storing calls).
Silent at `b6540001c6c69b843a69f4e7ba9cf13e0b37ae26`,
`6b7244fbb8d243387b3a9ed6ccbecae3bac420fe`,
`a50897193205a09db7f3a3d3af7ed6db9815492c`,
`e65098851b57c0feea83c86ee5cf06778d4b863f` and the head:
the 434 rows 176,
64,
20,
20 and 21;
the 260 of re-check 18 126,
104,
43,
32 and 32;
its 7 probes 6,
0,
0,
0 and 0;
the 46 of fix round 20 29,
26,
26,
1 and 1;
the 224 of re-check 19 114,
105,
85,
36 and 10;
its 20 not run 9,
4,
4,
4 and 2;
the 121 of this round 78,
78,
73,
65 and 1.
No input throws.
One input is silent at the head and was not at `e65098851b57c0feea83c86ee5cf06778d4b863f`:
A13 of re-check 17,
whose arrow is not reported any more because the function that holds it lists its caller (the entry on the parameters
of a callback).
X3-09 of re-check 18,
whose expectation was wrong,
is silent at `a50897193205a09db7f3a3d3af7ed6db9815492c` and the older heads of that comparison,
as in fix round 20.
The 67 silent inputs of the head are judged:
the 21 of the 434 rows are the 16 inputs re-check 17 judged wrong,
RC21,
RK37 and RK40,
G20 (a stated limit),
and A13;
the 32 of re-check 18 are the ones recorded in fix round 20;
P-38 is the alias of an object,
a stated limit;
Q22,
Q35 and Q36 are calls outside the list of the storing calls,
with Q38 pinned;
M01 to M07 are the expectations of re-check 19 that it judged wrong;
R07 is a default import of `node:module` with no use,
which loads nothing;
K06 and K12 are the limits of the key readers,
pinned;
C22 is `process` under another name,
a limit,
pinned.

The 1691 real sources give one report,
the named place of B867,
which now holds the key of `settleEntry` and is named,
and 15 forwarders with the same callers as before the round;
the merge adds two sources and no forwarder.

`origin/main` at `495bf2f39e5b7ebbeacec33b5cdfb239a2649c5a` is merged (`d0565720a3eaa85173d1b6ada7ef7927b56dd5b4`).
It brings the removal of the weight floors in other packages of the family and in `package/translation-repair`
documents,
and a note of the ide package.
No file conflicts,
and the two index files that both sides edit merged without a marker;
the ids stay in order,
B900 to B947 of `origin/main` after B864 to B869 and the other earlier ids,
and the entries B832 and B899 that `origin/main` repeats are still repeated.
`origin/main` has taken B948 to B952 since,
for the weight floors removed in rounds 10 to 12,
so this entry,
which was numbered B948 until fix round 22,
is B953.
B954 and every higher id are reserved for other branches.
On the merged head corpus-pass `build`,
`lint:types` and `lint:oxlint` (0 warnings,
0 errors) pass,
`test:unit` has 220 markers of pass and none of fail,
and in source-scan `test:unit` has 68 markers of pass and `source-scans` has 53,
with none of fail in any.
On the tree of the last code commit,
`fdae57969e22c55342dab6e9ee7f7d4acf65d21d`,
with this entry and the notes in the other files,
the same gates give the same counts:
corpus-pass `build`,
`lint:types` and `lint:oxlint` (0 warnings,
0 errors) pass,
`test:unit` has 220 markers of pass and none of fail,
source-scan `test:unit` has 68 and `source-scans` has 53,
and the Markdown linter finds nothing in the five files this round touched.

Records corrected in fix round 21:

- The report of fix round 20 counts the 430 inputs silent 173 times at `b6540001c6c69b843a69f4e7ba9cf13e0b37ae26`.
  The files hold 434 rows,
  and 176 of them are silent there;
  the count left out three of the four controls of `adv17-h`.
  The sentences of B869 that gave 173 carry the correction in place.
- B869 said "the ledger called the last one equivalent" for the mutant that asks every member for a callback,
  which points by position.
  It names the mutant now (Y-F5v).
- Three commit messages of fix round 20 promise a statement of the ledger that a later commit holds:
  `a3aef2b33efa77395fe4ec47c6ccb13a0a5ba7c3` ("the two limits of the reading that the ledger will state"),
  `482aadc10d64bbd530dfcbcbd4f500686b244a87` ("a limit that the ledger will state")
  and `fbab5f01f19115dda54083b90b36a825467a6786` ("The ledger will state why the name stays a name").
  The statements are in the follow-up of fix round 20 in B867 and B868.
- `1e62055f00cb064a2763c0b452363c678af23e98` said `origin/main` had not moved after the merge,
  and `e65098851b57c0feea83c86ee5cf06778d4b863f` corrects it.
  `3b47f21768905ddea009bc02d60d07ea41c2d526` and `c20e309953903a7962ed6637651dd6c839a1ef91` point by position
  (recorded in fix round 20).
- The index lines of B864 to B867 in the list of entries by id passed 120 columns,
  and B865 to B867 are reference links now,
  as B868 and B869 were.
- Commit messages of this round,
  as the scan of the round finds them:
  the subjects of `966aa797df445c6c2ce5b3ca760809a96c214fcc`,
  `0708e65ed43c12d9e519de80b685ad8478630d7b`,
  `10ae17fcadb23d73dd4e7abf3996709d25763a7e`,
  `d6a572c5d25efe8a7f648e9724e7578f4de39155`,
  `f77a19e5d6f884250dd50511c9f239bbb865993f`,
  `928ce59c7eb745d8ccfcbc5bead46db203ad4653`,
  `a1c78ef41bc4494afeef31b5a7dbf60b18fb7990`
  and `fdae57969e22c55342dab6e9ee7f7d4acf65d21d` are longer than 118 columns,
  which no subject of fix round 20 was;
  twelve bodies hold lines longer than 120 columns (`5d788e1995bd69d785b4f4c543f2b1849c095ca8`,
  `a7ab6da3c2e1023b9386b4633d6391c89b959183`,
  `4bd6f1c9a5206a3136154fe00c5b9bb6fcb39177`,
  `d6a572c5d25efe8a7f648e9724e7578f4de39155`,
  `59abf4225852cc95c898e65012fb92efb7b82866`,
  `f77a19e5d6f884250dd50511c9f239bbb865993f`,
  `07eb7a6e0bae095921b87c337f01f9bab2689050`,
  `4ae811e77c4323b0e146252fcd84b98c7fd45a0a`,
  `1ff3eae4cb039905d9ea290a890bd81e42c003e1`,
  `5d893dce21fa26038fbaf0c46cf25305d43476a3`,
  `928ce59c7eb745d8ccfcbc5bead46db203ad4653`
  and `a1c78ef41bc4494afeef31b5a7dbf60b18fb7990`);
  `d6a572c5d25efe8a7f648e9724e7578f4de39155` writes "the following numbers" and
  `59abf4225852cc95c898e65012fb92efb7b82866` writes "the rule below",
  which point by position;
  `f77a19e5d6f884250dd50511c9f239bbb865993f` and `d0565720a3eaa85173d1b6ada7ef7927b56dd5b4` promise a later statement
  (the ledger commit of this round,
  and the gates of the merged tree);
  `4ae811e77c4323b0e146252fcd84b98c7fd45a0a` says a node has one offset in every input,
  which `d1e5e454776e70793a042bc22e6f736f1044081e` corrects.
  Nothing is posted on these commits.

Cache versions in fix round 21:
none of the commits of this branch moved one.
The search over `e65098851b57c0feea83c86ee5cf06778d4b863f..93f03e1ce29f93e60329c0e7471292526df17475` prints 28 commits
of `origin/main` (the removal of the weight floors moved `CONSOLIDATE_CACHE_VERSION`,
`TRANSLATE_SLICE_CACHE_VERSION`,
`SLICE_CACHE_VERSION`,
`REFINE_CACHE_VERSION`,
`PAGE_TITLE_CACHE_VERSION` and `CONSISTENCY_CACHE_VERSION` in its own commits),
and the same search along the first parents prints the merge only.
Fix round 22 corrects the sentence of this record that said
the files that hold the versions were equal to those of `origin/main` at the head.
The sentence compared against a ref that moves,
and `origin/main` moved after the round
(re-check 20 found `consistency-cache-version.ts` and `pairing-cache-version.ts`
different from main's later commits).
The pathspec of the command also named the files that carry `cache-version` in their name,
while more files than those hold a version.
The comparison that holds is made against a full SHA,
over every file that holds a version
(the cache versions of fix round 22 give it).

The same search over `c265e8df95~1..c265e8df95` prints `c265e8df9525f4513b8229482d3e8d932ea630df`,
so the search can print.

Follow-up of fix round 22:
re-check 20 of fix round 21 found that the merge of `origin/main` made two entries head `B948`,
that forwarder-only modules,
object-literal forwarders,
a function called where it is written,
and storing calls
written another way were read less than their builder twins,
and the nits of that re-check.
The commits of the round are on top of `93f03e1ce29f93e60329c0e7471292526df17475`.
Every red commit comes before its fix,
except where this record says otherwise.

- The id `B948` headed two entries,
  this one and the one `origin/main` took for the removal of the weight floors.
  `5e321f8d45162a6f89ed1f4c16db7040fc55ab49` adds the scan
  `package/translation-repair/source-scan/src/ledger-ids-unique.unit.test.ts`.
  It fails when one `B<n>` or `M<n>` id heads two entries across the ledger files,
  and when a line of the list of entries by id names an id that no ledger file defines.
  An entry is a heading of the second level that starts with the id and a colon;
  a heading that continues an entry (`B899, round 8`,
  `B832 follow-ups`) and a deeper heading are not entries,
  and fenced lines are not read.
  The scan was red on the merged tree (`B948 heads 2 entries`) and is green after
  `ddea1de08b4fd2e39c8e67f77ad350f5ec4829bf`,
  which renumbers this entry to B953 in its heading,
  its opening line,
  the line of the list by id and its label
  `[b953-ledger]`,
  and in the mentions of the other ledger files and of `audit-ledger.md`.
  Positive controls over the real docs show that the scan finds a duplicate it is given and an index line it is given.
  The scan reads the tree it is run on,
  so an id that `origin/main` takes after a merge is found at the next merge.
- A module that only forwards was not read as a module that carries a builder
  (`9c9c4b9fa8b69b6c40cecf09a18b390b4340bf30` red,
  `bd5341110942f51ae24e5e26df39a0acf3a7a01d` red,
  `70f50d778d6b63d565002995871aa0f2a7130c74` fix).
  The modules that carry and the objects that carry were derived from the builders and the wrappers only,
  so a forwarder reached through a barrel,
  `export *`,
  a named re-export or `import()` was read as no module at all,
  and `declaresKnown` stood in for it with a test that any module declaring a known function was a carrier.
  The two sets are derived from the builders and the forwarders now,
  and `declaresKnown` is removed.
  Every row has a twin whose function is a wrapper of the builder,
  and the twin and the row give the same reports:
  10 rows through a barrel and 10 through `import()`.
  Cases of earlier rounds change their expectation to the report of the twin
  (the path of an import-equals,
  a default import member,
  an export assignment).
- Object-literal forwarders (`4673f00deea551b49fa91cb932a96234eae0f26d`,
  cases only).
  The two sets derived from the builders and the forwarders close the routes in which an object that holds a forwarder
  is read by value
  (E05,
  E06,
  Z12 and Z13 of re-check 20 are reported),
  and a declared function that lists its callers is kept (A01).
  Sixteen cases pin the rest.
  A property or a method of an object literal whose forwarder nothing reads (A04,
  A05,
  D03 to D08,
  O09 to O11 of fix round 22)
  and a declared function that nobody calls (A13) give no report,
  as their wrapper twins give none.
  Round 20 reported them because a module that declares a known function was a carrier,
  which also reported every such module that nothing reads.
  This is a narrowing against round 20 that the cases state:
  a forwarder with no reader reaches no send.
  The cases were written after the fix of the modules that carry,
  not before it;
  the 8 that fail do so on the export of `bd5341110942f51ae24e5e26df39a0acf3a7a01d`.
- A function called where it is written binds its parameters to the arguments of the call
  (`858e3225f58ab2f084e852f2470848520c1be8e0` red,
  `1a3ca06f81d2deb3f16143d7d50076c85a46f31a` fix).
  `(function (v) { holder.a = v; })(args)` and the form under type marks and parentheses are read:
  11 cases,
  of which 10 failed.
  Nine cases pin the limits of how a callback reaches a call:
  a function handed to a stored function is listed (N42),
  and a callback held in an array and called,
  handed over by a member,
  a conditional,
  a factory,
  a spread,
  a method written in place and handed over by `call`,
  an array argument of `Reflect.apply` or a class field
  is not (N41,
  N43 to N49).
  The sentence of this record that said a callback handed to a stored function stays unread is corrected in place.
  `3955484497f6c114a8accb0d70c6da5fdde83426` pins the form with `new`,
  and `fded1213c5103f663c8d0f8411eaa8b1756fcd50` and `42ed7af8540cb4af89bd1a7af0a6fb408934c913` remove two tests
  that changed no answer (the marks taken off a callee that is read through marks again,
  and a guard on a called
  function that binds no name).
- A storing method and a key reader are read however the call is written
  (`8c0560684a1e027c164b2cd8a6608c61dcf41672` red,
  15 cases failed,
  `39234557d729ff84d73a8ddbb7410f4ba96abb97` fix).
  `claude-schema-blind-static-calls.test-fixture.ts` reads a method of a built-in object called directly,
  through `call`,
  through `apply`,
  through `Reflect.apply`,
  with a leading spread of an array written in place,
  with a comma before the callee,
  or through `globalThis.globalThis.Object`.
  `Reflect.set` stores like `Object.assign`.
  What stays unread,
  each with a case:
  a method bound and then called (`Object.assign.bind(...)`),
  arguments spread from a name,
  and arguments of `Reflect.apply` held in a name.
  `Reflect.defineProperty` and `Object.setPrototypeOf` stay outside the list of storing functions,
  as the record of fix round 21 says.
  The key readers `Reflect.get` and `Object.getOwnPropertyDescriptor` are read in the same forms,
  with the same three limits.
- `43393e0f126505dd7f56934610ec92c94fc35cd3` wraps the six lines of the schema files that passed 120 columns,
  and the check of columns is run before every commit from this round.
- The mutants of the loader argument and of the marker,
  and the keys:
  `9775502039a5b5aad4b4129e9e01c5121356b950`
  adds the cases that kill them.
  RC20-08 is killed by 3 cases (inputs adv19-d:T34,
  adv21-c:C11,
  adv21-c:C34 and rc20-c:P36),
  and X3 and X3b by 1 case each,
  the case of a marker name (input I21).
  A case of a suffix path (input I01) pins what RC20-13 mutated,
  and the row is retired because the code it mutated is gone with the change of the carriers.
- The key of a named place is typed (`5e1ed66adb6993f1846d27f7f35aaea11ba1e998` red,
  `245e637fe343afd439c3be1d46432d2d4a56cfcc` fix).
  `keyOf` gives a `FunctionKey`,
  and `NamedPlace.held` has that type,
  so a bare name is an error of `lint:types`.
  The red of a change of a type is the type check and no failing case.
- Over-reports that fail closed are pinned (`1a06043c55f295f46783b93cb125417f4795cd52`):
  an assignment to `process.getBuiltinModule`,
  a `delete` of it,
  its `name`,
  `process.binding.call(process, "fs")` and a member named `getBuiltinModule` of any object named `process`.
  Each is reported and none is removed,
  because a reading that cannot tell a read that calls from one that does
  costs a case,
  and a missed report costs a hidden send.
- The mutants of the key of the reports (RC20-17,
  RC20-18 and RC20-19) survived because no source makes two
  reports of one shape that differ in one part only.
  A probe in a copy of the tree logs each pair that one of the three keys would merge and the real key keeps apart:
  none over the adversarial inputs and the real sources,
  with a positive control of 62 pairs counted in one file.
  `dda01ff5039a2926a13e9929cabc203d976d15f5` exports `spotsOf` and feeds it five written nodes;
  the three mutants are killed by 1,
  1 and 2 cases.
- Equivalent survivors,
  judged again on this code.
  RC20-06 (a callback that binds no name still gives a link) gives the same output as the head on the control
  with a callback that binds a name and one that binds none,
  and the mutant that drops the links of callbacks changes the same control.
  RC20-23 (a missing function equals an empty one) differs only for a report that holds an empty text,
  and no report holds one:
  a probe that logs the function of every report finds none over the adversarial inputs and the real sources,
  with the reports that hold a function as positive control.
- `origin/main` is merged three times in this round after the merge of fix round 21:
  at `654f87b6b2a4f14b264743e3a037d8ca97f026ee` (`917715e94c27a5a7f7b9f02f5646277fd9c7c343`,
  which brought B948 to B952
  and the one conflict,
  in the list by id,
  resolved by keeping both lines with the lines of main first),
  at `c8726940ee0c86051d3f1a6d827b789784288185` (`0f4d68af3d9666522333208b81f7759b1c41772d`),
  and at `ba037a0deab2f20cc26ad529ff71075e68ed87c4` (`71a36d117b68760224f4c1606e73d59bed0270ce`).
  The last two bring commits of the ide,
  nested-wayland-session and git-policy packages
  and no file of `package/translation-repair`.

The mutant table of fix round 22 has 457 rows on `42ed7af8540cb4af89bd1a7af0a6fb408934c913`:
416 of the 418 rows of fix round 21 (13 at a new anchor;
the row of the dot-written key reader and the row of the
module that declares a known function in any file are retired with their code),
21 of the 25 rows of re-check 20 (RC20-15 at a new anchor;
RC20-13,
RC20-14,
RC20-20 and RC20-22 are retired,
the last two replaced by the rows on the same edit at the new anchors),
and 20 for the code and the cases of this round.
The whole table ran once,
in one capped unit of 8 GiB and 800% of the CPU:
449 rows killed by a case,
one killed by not ending (N-R3e:
no exit within 300 s,
as in rounds 18 to 21),
and 7 survived.
X3 and X3b are killed now,
and so are the two rows that key by node,
shape and position.
The seven survivors are equivalents:
N-R5w,
N-R5x,
N-R3f and Y-F5r are the four argued in rounds 17 and 18
(each gives the same output as the head on its controls,
N-R5w differing in the count of loader entries only,
and each control is changed by a mutant that must change it),
R21-n1-key-by-node-shape-and-held gives the same output on its controls and on all 1492 adversarial inputs,
and RC20-06 and RC20-23 are argued in the item on equivalent survivors.

The adversarial inputs were run on the head of this round and on the five heads it is compared with.
The files hold 1492 inputs:
the 1112 of fix round 21,
the 339 of re-check 20 and its 10 of hidden routes,
and 31 of this round (object forwarders,
a call written another way,
an awaited import kept by a constant).
Silent at the head:
96 inputs;
at `93f03e1ce29f93e60329c0e7471292526df17475`:
145 of the same inputs.
None throws,
and none is silent at the head that was not silent at that head of fix round 21.
Against `e65098851b57c0feea83c86ee5cf06778d4b863f`,
12 inputs are silent at the head that were not:
A13 of re-check 17,
A04,
A05 and A13 of re-check 20,
D03 to D06 and D08,
and O09 to O11,
which are the forwarders that nothing reads and the declared function that nobody calls.
They are judged in the item on the object-literal forwarders.
The 1691 real sources give one report,
the named place of B867,
and the same 15 forwarders with the same callers as before the round.

What it does not read,
stated on 2026-10-29 (UTC):
the scan of the Claude route does not read these JavaScript forms.
- Variants of a function called where it is written,
  beyond the function expression
  and the forms under type marks and parentheses that the item on such a function pins.
- A method bound and then called (`Object.assign.bind(...)`),
  and arguments spread from a name.
  Both are told in the item on the storing methods.
- `Reflect.defineProperty` and `Object.setPrototypeOf`,
  which stay outside the list of storing functions.

Reading these forms is a later item.
The cases of this record that name the last four pin them as unread,
and the first has no case in this record that names its variants.

Cache versions in fix round 22:
none of the commits of this branch moved one.
The search over `93f03e1ce29f93e60329c0e7471292526df17475..HEAD` prints nothing
(the merges of `origin/main` bring only ide commits and commits that moved no version),
and the same search over `c265e8df95~1..c265e8df95` prints `c265e8df9525f4513b8229482d3e8d932ea630df`.
The files that hold a version are equal to those of `origin/main` at
`ba037a0deab2f20cc26ad529ff71075e68ed87c4`:

```sh
# package/translation-repair, over the files that hold a version
git grep --files-with-matches --extended-regexp '_CACHE_VERSION = [0-9]+' HEAD -- package/translation-repair
git diff ba037a0deab2f20cc26ad529ff71075e68ed87c4 HEAD -- <the files that the first command prints>
```

Records corrected in fix round 22:

- 417 rows on the first head and 418 on the second:
  corrected at the mutant table of fix round 21.
- B948 is B953 in every place of this record,
  the list by id and the other files (the commit
  `ddea1de08b4fd2e39c8e67f77ad350f5ec4829bf`).
- Short commit ids and the anchors `..HEAD` and `origin/main HEAD` of the cache-version paragraph of fix round 21
  are full ids now,
  and the false sentence is corrected there.
- The merge `917715e94c27a5a7f7b9f02f5646277fd9c7c343` has body lines of about 124 columns,
  and names "the next commits" as a promise;
  the scope of `5e321f8d45162a6f89ed1f4c16db7040fc55ab49` is `source-scan` where the package is
  `translation-repair-source-scan`.
  `4673f00deea551b49fa91cb932a96234eae0f26d` was written after its fix.
  Nothing is posted on these commits.

Prevention rule:
a name that stands for one place of the source holds everything that tells the place from another of the same name,
and a case compares each part with an input that differs in that part only.
A module that declares what the check follows is read as a module that carries a builder,
a function written inside a call is read as a function that takes its parameters from the call,
and an argument of equivalence that rests on "in every input" is run against the inputs before it is written.
After a merge of `origin/main`,
the ids of the ledger are scanned for one that heads two entries before anything else is committed,
and a record that cites a ref that moves (`HEAD`,
`origin/main`) cites a full SHA instead.

Recurrence:
`mistake-prevention-guards-searches-and-claims.md`,
"Guards that cannot fail".
