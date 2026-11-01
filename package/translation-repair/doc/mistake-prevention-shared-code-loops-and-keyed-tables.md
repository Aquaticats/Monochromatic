# Preventing mistakes: shared code, loops and keyed tables

Rules for the shape of the code:
one copy of shared code,
loops that do not copy what they built,
and tables keyed by text.

Part of [Preventing this package's mistakes](mistake-prevention.md),
which lists every file of this record.

## Copies of shared code

What happened:
helpers were copied rather than imported (three private Han tests and code-point counters,
one with a guard that can never fail),
and two heading readers disagreed about HTML headings (M44,
X20).

Audit area six then found the family package-wide (ledger B1 to B15):
31 groups of function bodies kept in two to
six files,
among them two readings of one event stream,
five span-rewrite appliers under three contracts,
and a benchmark grader whose re-carved slices ran one behind the run's (B10).
Declaring the shared type for one merge duplicated a type another module already exported (M47).
A tuning constant was declared in two files from their first commit,
one copy "mirrored from" the other,
and only the undocumented copy was read,
so retuning the documented one changed nothing (ledger B31).
The rule for what an artifacts directory holds (regular files named `*.json`) was written into two of the directory's readers,
and four others took any name ending `.json`,
so a directory or a symlink named like an artifact became a settled entry;
five more filtered the one lister's names by the suffix again,
four sliced the suffix off by hand,
and ten files spelled the directory's name (ledger B64).
The same lapse sat in the run's other directories:
the ledger report read the atomic writer's temporary files as contests,
a stray file in the published tree raised ENOTDIR out of the verifier,
and the slice cache stopped an entry on a directory named like a slice;
meanwhile B64's own listing result was a copy of one `directory-listing.ts` already had (ledger B65).
Two readers moved into modules of their own for the italic-title pass shipped tested only through their callers,
since the rule asking for their own tests stood under "Text by code point",
where a reader of structure is not looked for (M88).
Two readers of a front matter `alias` disagreed on a list:
one joined its items,
the other read it as another schema and skipped the identity rule (ledger B97).
The package's check that parsed JSON is a record admitted arrays,
and eighteen checks in thirteen files had retyped its test inline rather than calling it,
so the fix to the shared guard reached none of them;
the census of the guard's callers listed none of the copies either,
since it searched by the guard's name (ledger B92).
Test files kept their own copies of three helpers that shared fixtures already held,
under thirteen names,
while each fixture's header said new cases import it;
the duplicate-body scan reads production source alone,
so no copy could fail it (ledger B103).
Hoisting those copies again left lint findings only the moved code met,
orphaned four type imports in their callers,
kept two type assertions the literals never needed,
and left two test helpers nothing called (ledger B115).
The runner tests then held near copies the duplicate-body scan cannot see,
since it groups only bodies that match once comments and spacing are aside:
a census by syntax shape found 111 groups,
41 of them across runner files;
four helpers copied across runner tests moved into fixtures,
and the rest were left with a reason each (ledger B385).
A group left apart says whether its reason comes from a diff or from names alone.
Two briefs of one round touched the same entry files,
one naming `RUN_OUTSIDE_READS` as its item and the other `corpus-pass.ts` as its file,
and neither said which agent owned which statements;
both agents removed the constant and built an `outsideReadsFrom` block in its place,
and the lead found the duplicate at the merge (ledger M144).

The rule:
before writing a helper,
declaring a type or copying a constant,
`rg` the package for its name and for its shape
(for example `rg 'function \w*(codePoint|Han|heading)' src`,
or the field names together) and import what is there.
A copy that must stay separate (an artifact version's frozen rule) says so where it stands,
naming the live copy and the check that compares them.
Code that claims to rebuild what the pipeline built (slices,
sheets,
verdicts) calls the pipeline's own function,
or measures its agreement over the corpus and records the result.
A rule about what a directory holds lives in one module every reader lists through
(`corpus-run/artifact-file-name.ts` for artifacts),
and a new reader of that directory starts from that module;
a name spelled in two files becomes a constant one of them owns
(`runs-layout.ts` for the slice-cache and prompt-payload directories).
Every reader lists through `corpus-run/directory-listing.ts`,
naming the kind of entry its writer makes
(files or directories,
never links),
then filters by the name its writer gives a record,
so a write still under its temporary name is never read.
A helper moved into a module of its own,
or given a second caller,
gets a unit test file of its own in the same change,
with the edges no caller reaches among its cases.
A new reader of a field another reader already reads starts from that reader's shapes,
and where the two must differ,
each says how and why.
A check a shared module owns is called,
never retyped,
and a census of a shared check searches for its shape as well as its name,
in every form the shape is written in (both `===` and `!==`).
A test helper is looked for among the `.test-fixture.ts` files,
by its body's shape as well as its name,
before it is written into a test file.
A fixture's header that says copies remain names where they stand,
and the change that writes that header moves them or queues them in the ledger.
Code moved into a `.test-fixture.ts` file meets the production lint rules there,
not the test-file override,
so the move is linted,
and a patch an agent wrote passes lint,
the source scans and the TSDoc attachment probe before any review of it is trusted
(ledger B111).
A helper that writes a process global moves into a fixture by stubbing through the case's own sandbox,
or,
where it assigns the global directly,
only where the global-writes scan can follow it to its cases:
imported by name,
never through a namespace import or a re-export (ledger B111,
B120).
A caller that passed a local helper to `map` passes a named wrapper once the helper is imported,
since the callback-arity rule reads the parameters of a function declared in the same file only.
A move that leaves a caller without a use for an import removes that import in the same change,
and an assertion a patch keeps because its author could not compile is tried without the assertion
(ledger B115).
A patch an agent cut is merged three-way against the commit the agent read,
recorded when it starts;
a diff it takes against the files as they stand when it finishes cannot show a change it undid,
and every file a commit touched since that start is checked line by line for undone work (ledger B117).
An agent's new case is a claim that the behaviour it pins is right:
before it merges,
each expectation is read against what the code should do,
and a case pinning a defect is held back while the defect is fixed or queued
(ledger B121,
and T8's `stream` cluster,
whose case pinned a frame reader passing a non-object frame on).
Agents working at once each write only the test files of their own modules,
so no two patches touch one file,
and their rules file carries every lint and scan rule a merge has had to fix
(`rules-t8.md` in the session's scratch).
When two briefs touch one production entry file,
the briefs name which agent owns each statement of it,
and the lead merges each patch's red commit before its fix (ledger M144).
A writer moved into a fixture is followed by the global-writes scan to the files importing it by name (ledger B120),
so the env writers it once kept copied have moved into fixtures,
and the list of global-writer copies is gone (ledger B385).
A test's expectation is never the module's own body restated:
it states the result,
or a relation between values the module exports,
so a change to the formula can fail it (ledger B116).
A string a case offers and then looks for is one named constant,
so the two cannot drift apart (ledger B116).
A tool version written in source is the same copy of a fact the lock holds:
`PINNED_LINT_BINARY_VERSION` in `semantic-wrap-binary-constants.ts` must equal the version of `monochromatic-lint`
in `toolchain.lock.json`,
and it follows each release in the change that moves the lock (ledger B700).
The rule holds for any binary a package pins:
the pin follows `toolchain.lock.json`,
and a lock move updates the pin in the same round,
which the move to 0.4.0 did not do until the owner ruled on it (ledger B1015).
The lock was `mise.lock` until the repository moved its tools to MPM on 2026-10-28.
A lock moved by a commit of its own leaves the constant behind,
so after any change to `toolchain.lock.json` the unit tests of typography and page-check run with the real program.
A gate tree finds the real program in its own `.cache/toolchain-bin`,
which `mise run toolchain:sync` makes after the tree's install;
without it the name falls through to whatever else the PATH holds.
The order of that setup is in "Gate trees and their tools" of
`mistake-prevention-tasks-builds-and-bulk-output.md` (ledger B980).

What enforces it:
`package/translation-repair/source-scan/src/duplicate-bodies.unit.test.ts` (ledger B19) fails on any function body of 80
or more characters,
comments and the spacing of code aside,
literals compared as written,
kept in two places in the package's source,
its tests or its test fixtures (ledger B116),
in one file or two,
except the frozen copies it lists with their reasons,
and fails when a listed copy no longer stands.
A copy that must stay separate is added to that list in the same change that makes it,
with the reason and the check that compares it.
The shared helpers (`code-points.ts`,
`han-only-text.ts`,
`page-headings.ts`,
`index-pair-list.ts`,
`sse-data-line.ts`,
`corpus-run/span-rewrites.ts`) say so in their headers.
`package/translation-repair/source-scan/src/directory-listing-scan.unit.test.ts` (ledger B65),
among the source scans,
fails on any directory-listing import outside `directory-listing.ts` and the walkers it names with why.
`package/translation-repair/source-scan/src/record-checks.unit.test.ts` (ledger B92),
among the source scans,
fails on any `typeof` compared with `object` in the package's source outside `isJsonRecord`
and the checks it names with why.
`package/translation-repair/source-scan/src/own-unit-tests.unit.test.ts` (ledger B102),
among the source scans,
fails on a production module with neither a unit test of its own nor a line in its allowlist.
`package/translation-repair/source-scan/src/unused-imports.unit.test.ts` (ledger B32),
among the source scans,
fails on an import a move left unread.
Habit holds the owner of each statement two briefs share,
and the lead's review at the merge (ledger M144).
The agreement case in `semantic-wrap-binary.unit.test.ts` (ledger B700),
"HOLDS THE PINNED VERSION TO THE ONE toolchain.lock.json RECORDS",
reads the tool's entry in `toolchain.lock.json` and fails when the constant differs.
It read `mise.lock` until that file was deleted on 2026-10-28,
and failed with a missing file at the first gate after the merge of main that deleted it.
It runs with typography's tests only,
so a commit that changes `toolchain.lock.json` alone is not checked by it until someone runs them;
this audit found no task of the package that runs them on a lock change,
and none was added.

## Loops that copy what they built

What happened:
folds over a page's lines,
a slice list,
a gap list or a list of losses returned what they had built so far spread into a new list,
or cut and rejoined,
at every step,
so each step cost as much as every step before it (ledger B70).
One of them was the stub-marker scan,
whose summary called it one linear pass;
it copied every line kept so far at each line of the page.
Text edited in several places was spliced one edit at a time,
last first,
at offsets into the original,
copying the whole text per edit,
and where one edit's rewrite changed its length and another's range held it,
the second cut in the wrong place:
a title quoted inside a quoted title lost the mark after it.
Nothing read the shape:
the text accumulator scan sees only a `let` begun as text and grown in a loop.
The guard written then read folds and reassigned bindings,
and five loops that grouped or queued by key set a map entry to a copy of what it held at every repeat,
among them the cap census's queue of a label's streams (ledger B74).
A scan for the next web address searched for each of two schemes apart from its cursor,
so a scheme the text lacked was read for to the text's end at every address,
and the scan cost the square of the text while its summary called it one linear pass;
a reader of the same two schemes beside it had searched for their shared stem once (ledger B137).

The rule:
a fold or loop over its input appends to what it builds,
and removes in place where a step removes;
it never returns a copy of the list or text built so far,
nor sets a map entry to a copy of what the entry held:
it pushes onto the entry's list,
or groups with `Map.groupBy`.
A record of fixed fields may be copied per step,
since it costs the same each time.
Several edits into one text are placed by offsets into that text
and written in one pass with `spliceDisjointEdits`,
which refuses edits that share a unit or a start;
a caller whose edits can nest decides which it keeps before it writes,
and names only the edits it wrote.
A fold whose each step reads what the step before wrote,
or that walks a fixed table,
may rebuild,
and says so where the guard names it.
A summary that calls a pass linear is read against the pass.
A loop that looks for the next of several needles from its cursor looks for all of them in one search,
or for what they share,
never for each apart to the text's end at every step;
where the package already has such a search,
the loop calls it (`nextSchemeStart` in `scheme-start-scan.ts`).

What enforces it:
`fold-copies.unit.test.ts` fails on a `reduce` or `reduceRight` callback that copies its accumulator
(a spread into a list,
a copying method,
a `Map` or `Set` built from it,
or `+` where the fold starts from text)
and on a loop that reassigns a list or text as a copy of itself
or sets a map entry to a copy of what the map's `get` read there,
unless the fold is named with why its copies are bounded or are its meaning,
and on a named fold that no longer copies;
`disjoint-splice.unit.test.ts` holds the one-pass writer to the old splice's text and its refusals.
A search that reads the text again at every step is not a copy,
so the scan cannot see it;
review reads a scan's searches against its cursor,
and `scheme-start-scan.unit.test.ts` holds the one scheme search's answers.

## Tables keyed by text

What happened:
plain objects served as tables looked up by text the package did not write:
a provider's stream delta type,
a corpus asset's extension,
a stored issue status and repair disposition,
and corpus entry ids in the attempt counts.
An object answers `constructor`,
`toString` and the other names it inherits with the prototype's values,
and one written through `__proto__` sets its prototype and drops the entry,
so a delta typed `constructor` was routed with `Object.prototype` as its channel,
an asset named `tabby.constructor` read as usable,
a claim with that status was filed under the `Object` function as its arm,
and a count for `__proto__` was never written (ledger B77).
A lookup took the first node carrying an id,
on the unstated assumption that ids are unique,
and a document repeating an id had a sound span checked against the wrong node
and rejected as stale (ledger B129).
A census of every first-match lookup in the production source found eight more:
a voice's second rendering of one title passed over (ledger B203),
an href keyed to the archive's last text for it (ledger B204),
a claim's proposers taken from the last chunk naming it (ledger B205),
a stored page assembly and a provider's listing read by their first row under a key (ledger B206,
B208),
trimmed texts keyed by slice beside two index lists that were checked (ledger B207),
three log readers settling a repeated field by order (ledger B209),
and an attribute restored from the first of two a tag names (ledger B210).
A heading cut at its last colon wrote a name holding a colon twice (ledger B198).
The census's search pattern missed maps built from typed pairs and maps filled by `set` in a loop,
where two of the eight stood.
Three sites that census described and left were fixed next:
a title linked twice in one slice,
whose second link was never located (ledger B228);
a refrain,
compared only at the first place its wording stands (ledger B229);
and an archive block found again by its text,
so its revision was checked at the first place the text stands (ledger B230).
A caller that cut a piece out of a text passes where the piece stands,
and the callee does not find it again by its text.

The other sites that census described and left were closed after those,
each a reader that answered for one of two elements under one key:
a roster seating one model twice,
a confirmation fitting two rounds,
a tally table,
a region's slice,
a ledger row for an arm,
a subject on two rows,
a sheet position named twice and a prober answering a region twice (ledger B238 to B245),
with a card dropped by a repeated served id (ledger B246).
A second census,
over map writes,
loops that return at a first match,
sets that drop repeats and keys of parsed objects,
found a page name the archive renders two ways given by its first rendering (ledger B263)
and one voice counted twice for naming a pair twice (ledger B264).
A report's merge key that left out two of four providers' states merged readings that differed there (ledger B281).
A reader says what a repeated key means,
by a refusal,
a count or a finding,
and a key holds every field that tells two records apart.

The rule:
a table looked up by text is a `ReadonlyMap`,
and a record filled by a key is a `Map` until it is handed on whole,
as an object from `Object.fromEntries` at its function's return or its file's write.
A table over a closed set of names keeps the names in its type
(a literal-keyed object,
checked with `satisfies` rather than annotated as a record of strings,
or a `Record` over the set's own union),
so the compiler refuses a lookup by unchecked text;
an exported table needs an annotation under `isolatedDeclarations`,
which widens its keys,
so an exported table looked up by text is a map.
A lookup that takes the first element matching a key
(`find`,
`indexOf`,
or a map built from pairs where a later pair replaces an earlier one)
names what makes the key unique:
the function that builds the collection,
or the check upstream that refuses a repeat.
Where neither does,
because the collection comes from a model's reply,
a file,
or a caller the signature lets build anything,
the lookup gathers every match and says what two mean,
in the refusal form its module already uses.
A cut at a delimiter is such a lookup too,
and a census of them reads maps built from typed pairs and maps filled by `set` in a loop
beside the calls.

What enforces it:
`text-keyed-tables.unit.test.ts`,
which `source-scans` runs,
fails on a declaration typed with string keys
(`Record<string, …>`,
that inside `Readonly`,
a string index signature,
or a package alias of one,
by annotation or by a cast)
whose value is an object literal at a module's top or `{}` anywhere,
or that is written through a computed key;
the red cases in each site's test file hold the inherited names.
`validate-issue.unit.test.ts` and `critic-wire.unit.test.ts` hold a document repeating an id,
and each site of ledger B203 to B210,
of B228 to B230 and of B238 to B246 holds its repeat in its own test file;
`blind-map-writes.unit.test.ts`,
which `source-scans` runs,
fails a two-argument `set` whose enclosing functions never read the same key of the same map,
outside the writes it allows with what makes each key unique (ledger B265);
first-match loops,
sets that drop a differing repeat and doubled keys in parsed JSON are held by habit and review.
Out of the scan's reach,
and read once by a typed census recorded in ledger B77:
a table built by `Object.fromEntries` or handed back from a function and then read by text,
a class field,
a parameter destructured from an object an alias types,
a key typed by a template literal,
and a cast of text to a table's literal keys.
