# Audit ledger: tests read only what their package declares

The gate is to run the packages a change can reach through the declared package graph,
and a test that reads another package's files by path,
with no declaration to say so,
is a test that graph cannot see.
Round w402 (2026-11-01 UTC) makes the graph true for the tests,
and adds a scan that keeps it true.

Part of [Audit ledger](audit-ledger.md),
which lists every file of this record.
The rule and its guard are also in
[Preventing mistakes: tests that read other packages](mistake-prevention-tests-that-read-other-packages.md).

The owner's rulings that decide every item here (2026-11-01):
"Gate only the pkgs a round could possibly impact";
"tests that indirectly depend on a package - that's just poor design and the code and tests should be fixed";
"You don't need a temporary always run list.
Fix before testing,
common sens."

The commits are `8440155b9c`:
the scan,
red on the head it was written against;
`5fff95204a`:
the six fixes that turn it green;
and the docs commit that carries this file.
The tree was detached at `e9b966af55`.
Ids B1265 to B1272 are used in this file only.

## B1265: the defect, and the rule that replaces it

The selector reads impact from the package graph:
a package is impacted when its own files change,
or when a package it lists under `workspace:*` in its `package.json` is impacted.
A test that reads the files of a package it does not list passes in a gate that never ran it,
because the package it reads was changed and the package it sits in was not.

The rule this round sets:
no test (`*.unit.test.ts`,
`*.expensive.unit.test.ts`) and no test fixture (`*.test-fixture.ts`) of a package other than `source-scan` reads,
lists or runs git over the files of a package it does not declare.
It reads its own folder,
a folder it made,
the pinned corpus clone,
and the API of a package it declares through that package's name.
`source-scan` is the exception by purpose:
its input is the whole family,
and its scans run in every gate.

## B1266: the census

Searched over every `*.unit.test.ts`,
`*.expensive.unit.test.ts` and `*.test-fixture.ts` outside `source-scan` (`rg` from `package/translation-repair`):

- every literal naming `translation-repair` (most hits are imports of a package name or `mise run` task names);
- every `import.meta.dirname`,
  `import.meta.url`,
  `import.meta.filename`,
  `fileURLToPath`,
  `process.cwd` and `__dirname`,
  with the lines around each;
- every climb of two or more levels that does not end in `dist`,
  and every multi-line `'..'` argument of a `join`;
- every `import.meta.resolve`,
  `builtDirectoryOf` and `builtPipelineDigest` (the built output of a package reached by its name);
- every git verb (`log`,
  `show`,
  `diff`,
  `ls-files`,
  `rev-parse`,
  `rev-list`) and every helper named for git;
- the helpers the brief named (`familySources`,
  `FAMILY_ROOT`).

The offenders were six reads in five files:

- `corpus-pass/src/claude-schema-sources.test-fixture.ts` (`familySources`,
  a `glob` and `readFile` from the family folder)
  and `corpus-pass/src/claude-schema-blind-specifiers.test-fixture.ts` (a `glob` of every package's `src`
  and `package.json`,
  and the repository's `pnpm-lock.yaml`),
  which the answer-schema search of `claude-schemas.unit.test.ts` and its 138 fixtures use.
- `coverage-census/src/family-build-entries.unit.test.ts` (`readdir` of the family folder).
- `typography/src/typography-prose-mask.unit.test.ts` (`readFile` of five source files in four other packages).
- `artifact-pool/src/corpus-run/artifact-eligible.unit.test.ts` (`git rev-list` and `git rev-parse` of the checkout the
  test runs in,
  for the root and the head that four cases hand `selectEligible`).

The brief's other known names are not offenders.
The `cache-account-*` tests and fixtures of `slice-cache` run git only in a repository the fixture builds
(`makeCacheAccountRepo`,
`-C repo.path`,
`packageDirectory: repo.path`),
and the `package/translation-repair/...` literals in them are files of that repository or a pathspec it is asked for.
The two `process.cwd()` cases of `cache-account-search-reach.unit.test.ts` build a relative path to a scratch folder and
read nothing else.
`coverage-census`,
`probe`,
`run` and `glossary` have no other read that leaves the package:
their `import.meta.dirname` reads end in their own `dist`,
and their `reask-role` and `child-environment` reads are files of their own folder.

Three classes stay,
and are decided in B1270.

## B1267: the guard, and its red evidence

`source-scan/src/package-reads.unit.test.ts` parses every test and fixture outside `source-scan` and finds six kinds of
read:

- `climb`:
  a path chain built from `import.meta.dirname`,
  `import.meta.filename`,
  `import.meta.url`,
  `__dirname`,
  `__filename` or `process.cwd()`
  (through `join`,
  `resolve`,
  `normalize`,
  `dirname`,
  `fileURLToPath`,
  `pathToFileURL`,
  `new URL`,
  `import.meta.resolve`,
  a template,
  a `+` or a constant holding one of these)
  whose `..` segments climb past the package folder,
  counted from the depth of the file's own folder.
  A constant holding the climb is reported where it is held and not at each place it is read.
- `import`:
  a static import,
  re-export,
  dynamic import or `require` of a relative specifier that climbs past the package folder.
- `literal-read`:
  a read of the file system (`readFile`,
  `readdir`,
  `stat`,
  `access`,
  `opendir`,
  `existsSync` and their synchronous and stream forms) whose path is written in literals from its start and either
  climbs or names `package/translation-repair`.
- `glob`:
  a `glob` whose pattern climbs past the package folder from its `cwd`.
- `git-checkout`:
  a git read (`log`,
  `show`,
  `diff`,
  `ls-files`,
  `ls-tree`,
  `rev-list`,
  `rev-parse`,
  `grep`,
  `cat-file`,
  `blame`,
  `status`,
  `describe`,
  `for-each-ref`,
  `merge-base`) whose `-C` argument or folder option is a path chain.
- `git-cwd`:
  a git read that names no folder.

The scan has 30 cases that must be found,
one for each form listed here and each way to write it
(a join,
a resolve,
a string with the climbs in it,
a template,
a plus,
a URL,
the folder of the file's own name,
the file's own name,
the CommonJS name,
the working directory,
a held constant,
a glob pattern,
an import,
a re-export,
a dynamic import,
a `require`,
a relative path read whole,
a listing,
the family folder read whole,
a template with no variable at its root,
and six git forms),
and 19 controls that must pass
(a file beside the test,
the package's own `dist` by a join and by a URL,
a sibling by a URL,
the working directory joined below it,
a scratch folder climbed and read,
the family's folder names inside a scratch folder,
the family folder as data a tool is handed,
three git reads of a fixture repository or a corpus clone,
a git command that writes,
a list of git command names that is no call,
a package imported by name,
a climb in package source that is no test,
and a climb and a git read in a test of `source-scan`).
The cases print their labels,
and a case that fails prints the labels it found different.

Red evidence,
on `e9b966af55` with only the scan added (`8440155b9c`;
the run's whole output is in the agent folder as `red-1.txt`):

```text
artifact-pool/corpus-run/artifact-eligible.unit.test.ts:91 git-checkout: [ '-C', import.meta.dirname, 'rev-list', '--max-parents=0', 'HEAD', ]
artifact-pool/corpus-run/artifact-eligible.unit.test.ts:107 git-checkout: [ '-C', import.meta.dirname, 'rev-parse', 'HEAD', ]
corpus-pass/claude-schema-blind-specifiers.test-fixture.ts:40 climb: join( import.meta.dirname, '..', '..', )
corpus-pass/claude-schema-sources.test-fixture.ts:219 climb: join( import.meta.dirname, '..', '..', )
coverage-census/family-build-entries.unit.test.ts:31 climb: join( import.meta.dirname, '..', '..', )
typography/typography-prose-mask.unit.test.ts:235 climb: new URL( `../../${directory}/src/${underSrc.join('/',)}`, import.meta.url, )
```

The fixture cases passed in the same run
(the scan finds each form and leaves each control alone),
and only the package-wide case was red.
After the six fixes of B1268 and B1269 the package-wide case and the four fixture cases pass.

What the scan leaves out is stated in its header:
a path built from a text another module returns,
a climb written in a variable that holds the `..` segment,
a git read built apart from its call that has no `-C` argument,
and a read made by a child process the test starts.

## B1268: the answer-schema search moves to `source-scan`

The search that keeps the list of answer-schema producers honest reads the source of every package.
It was a case of `corpus-pass/src/claude-schemas.unit.test.ts`,
with 138 fixtures beside it.
All of it now lives in `source-scan/src`,
and `corpus-pass` keeps the cases that hold each listed schema to what Anthropic accepts.

Moved with `git mv` (history kept,
the import of the scan fixtures made relative,
one line in 37 of them):
the 138 `claude-schema-*.test-fixture.ts` files and `claude-schema-drain.unit.test.ts`.

The test file was cut at its own boundaries.
From `corpus-pass/src/claude-schemas.unit.test.ts` to `source-scan/src/claude-schema-source-search.unit.test.ts`,
with 16 case blocks
(a block is one `it`,
or one `.map` that makes a case for each element of a fixture's list):
the list of files that declare an answer schema,
the shared-builder search,
the broken-file refusal,
the form cases of every fixture,
the shapes the search leaves unread in the family's source,
the places the test names,
the memo of the search,
the callers of a builder,
the shared builders of the family,
the forwarders of the family,
the wrapper that sends,
and the parameter named like a forwarder.
Staying in `corpus-pass`:
the named callers of a shared builder in the producer list,
the closed objects,
the rules through the Claude body,
the one bound dropped,
the six forms and the positive control,
and one new case.
The blocks add up:
22 before the move,
and after it 16 here and 6 in `corpus-pass`,
with the one new case making 7 there.

Two things changed besides the place.
Eight cases that read the family's source compared through `toEqual`,
which `scan-direct-comparisons.unit.test.ts` refuses in a scan,
so they now compare through `expectFindingsAsListed`,
`expectNoFindings` and `expectRecordAsListed`,
which fail with each difference on a line of its own and then make the same comparison.
Two cases needed the producer list,
which is a runtime value of `corpus-pass`:

- The files of the list are read by `producerFilesOf` (`claude-schema-producer-files.test-fixture.ts`)
  off the text of `claude-schema-producers.ts`,
  and the new case of `corpus-pass` holds that reading to `CLAUDE_SCHEMA_PRODUCERS` as the pass runs it.
  The two cannot drift,
  and the strength of the file-list case is the same.
- The list of shared builders moved to `claude-schema-shared-builders.test-fixture.ts`,
  and the case of `corpus-pass` that holds its named callers to the producer list imports it.

No dependency was added.
A scan of `source-scan` that imported the built entry of `corpus-pass` would need a new `devDependencies` line,
and so a regenerated `pnpm-lock.yaml`,
which this round may not run,
and a cycle with the `devDependencies` line `corpus-pass` already has on `source-scan`.
`corpus-pass` keeps that line,
since two of its cases import the two fixtures,
and `source-scan` has the same lines it had.

## B1269: three smaller moves and rewrites

- `coverage-census`:
  `READS THE BUILD ENTRIES OF EVERY PACKAGE OF THE FAMILY THAT HAS A LIST, not only the census's own`
  moved from `coverage-census/src/family-build-entries.unit.test.ts` to
  `source-scan/src/family-build-entries.unit.test.ts`,
  with its name.
  It listed every package's `src` folder and compared the folders with `FAMILY_ENTRY_LISTS`;
  the scan reads the keys of that map off `coverage-census/src/family-build-entries.ts` and the packages with a
  `build-entries.ts` off `readPackageSource`,
  compares them with the findings helper,
  keeps `expect(listing.length > 1)`,
  and has two fixture cases:
  one for the reading of the keys and of the packages,
  and one that a source with no map under the constant's name is an error,
  so a rename cannot turn the scan into a pass.
  The three merge cases stay in the census.
  The comment of `family-build-entries.ts` names the scan.
- `typography`:
  `IS THE ONE SCANNER OF PROSE: every rule that reads quotes, ellipses or guillemets takes its mask from proseMaskReading, and none keeps a scanner of its own for code or tags`
  moved from `typography/src/typography-prose-mask.unit.test.ts` to `source-scan/src/prose-mask-readers.unit.test.ts`,
  with its name and its five readers.
  It read five files by `new URL('../../<package>/src/...')`;
  the scan takes them from `readPackageSource` by the same keys,
  and a reader the family does not hold is a finding,
  where the read threw before.
  A fixture case holds both findings.
- `artifact-pool`:
  four cases of `corpus-run/artifact-eligible.unit.test.ts` asked git about the root commit and the head of the checkout
  the test runs in:
  `splits ONE generation by commit when a required commit is named`,
  `NAMES a generation the required commit excludes as stale, beside the one it keeps`,
  `WARNS that a required-commit pool is a post-baseline cohort when two generations both satisfy it`
  and `THROWS rather than returning an empty pool when a required commit excludes every settled entry`.
  They stay in the package,
  and `throwawayHistory` builds a repository of two empty commits in a scratch folder
  (`fixtureGit`,
  `scratchDirWith`) whose root and head they use and whose path they hand to `selectEligible`.
  The cases check what they checked,
  and a shallow checkout no longer changes their answer.
  Reading the checkout's history is not reading a package's files,
  but it ties the test to the checkout's depth.

## B1270: the decisions on what stays

- The tool lock of the repository.
  `typography/src/semantic-wrap-binary.unit.test.ts` holds the version the package pins to the one `toolchain.lock.json`
  records,
  by a climb of four levels to the repository root.
  The lock belongs to no package,
  so no declaration could name it.
  The scan lists the read with its reason,
  and fails if the file holds it more or fewer times than listed.
  The gate's selector will not run the case when only the lock changes.
  That is a suggestion for the owner to rule on,
  not a requirement:
  the selector could treat a change of a root file as a change of every package that lists it,
  or the case could move to a place that always runs.
- The repository's `pnpm-lock.yaml`,
  which `claude-schema-blind-specifiers.test-fixture.ts` reads to find the packages each family package lists.
  The fixture is now in `source-scan`,
  so the read is allowed.
- The built output of a package reached by its name
  (`import.meta.resolve('@monochromatic-dev/translation-repair-<name>')` with `builtDirectoryOf`,
  then a digest of that folder).
  Node resolves the name only for a package the importing package lists,
  so the read is covered by a declaration,
  and the scan does not count it.
  It is a read of a dependency's files by path in the sense of the brief,
  and the brief said to import the dependency's API.
  These cases hand the built folder to `digestPipeline`,
  which reads the folder it is given,
  and the API of a package does not return its own built folder.
  Left alone and named here.

## B1271: the cache versions

The rule is step 4 of `mistake-prevention.md` (M25,
M28).
No version moves.
Evidence:
`git diff --name-only e9b966af55 HEAD`,
read for every file that is neither a test nor a fixture,
lists two:
`source-scan/mise.toml` and `coverage-census/src/family-build-entries.ts`.
The task file holds no stage.
The census file changes two lines of a comment (the name of the scan that holds the map to the packages),
and `coverage-census` is listed as a dependency by no package:
`rg translation-repair-coverage-census package --glob package.json` lists only its own.
No sheet,
floor,
house rule or prose range a stage imports is touched,
and no `*-cache-version.ts` is,
so no key (the version,
the shape of the run and the texts a stage is given) can read a different value.
Every other change is a test or a fixture.
`slice-cache:cache-account-audit` was not run:
the round's brief forbids a built runner,
and the diff named here is the reading its output would be compared with.

## B1272: the slips of the round and what stays open

Slips:

- The commit message of `5fff95204a` says 137 fixtures moved,
  and 138 did (139 renames with the work-stack test);
  it says `claude-schemas.unit.test.ts` keeps "6 schema cases and the named-callers case",
  and it keeps 5 schema cases and the named-callers case,
  with one new case (7 blocks).
  The message of `8440155b9c` says 20 controls,
  and the scan has 19 (30 cases that must be found).
  The numbers in this file are the measured ones.
- The first commands of the round ran without the keyless wrapper the brief names for every command
  (reads of the tree with `rg`,
  `sed`,
  `ls` and `cat`,
  none of which reads a key),
  two evidence reads were cut with `head`,
  one command joined two searches with `;`,
  and one chained four `&&` steps.
  They were found and the wrapper used from then on;
  none changed a file.
- The first draft of the scan declared its absence as `undefined` in four places,
  which the repository's lint refuses;
  the final one marks absence with a value of the type.
- The one run of the whole source-scan suite found two scans red on the files of this round:
  `tsdoc-leading-articles` listed 24 `@returns` and `@param` descriptions that open with an article in four files,
  and `dead-test-declarations` listed `SharedBuilder`,
  a type exported and imported by no file.
  Both come from the files written in the round,
  which had been linted and tested alone and not run beside the other scans before the docs commit.
  The commit after the docs removes the articles and the export;
  the two scans and the three new ones pass when run alone.

Open:

- The lock read of B1270 (a decision for the owner).
- The built-output reads of B1270.
- `slice-cache:cache-account-audit` was not run (B1271).
- The hand-back lists the gates run.
