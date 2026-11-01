# Preventing mistakes: inputs, defaults and values read from text

Rules for what the code takes in:
defaults,
fields and numbers carried as text,
command lines,
the package's own log lines,
and what providers send.

Part of [Preventing this package's mistakes](mistake-prevention.md),
which lists every file of this record.

## Defaults that stand in for an input

What happened:
the final polish held the slice's disputed wordings and never passed them,
so a polish landing on one shipped past the three checks that had refused it;
the chunk's checker check left the refiners out,
and the bench read at the checker stage was read for quorum alone;
two writers' sheets displayed candidates without their break counts;
reference attestation decided on one voice;
and a chunk scan swallowed a parse error with `void error` (ledger B29).
A page no grammar could read was read as an empty page,
and the translate floor passed the candidate against the original alone (ledger B36).
Each default read as "none",
and the code compiled and ran.
Three exported names were each declared twice,
one pair with one signature for two comment grammars.
The archive revision's footnote floor passed every revision of a block the archive did not carry,
a branch its comment kept for fixtures a caller composed,
though production hands it an exact slice and no test reached it (T8's eighteenth batch).
The attribution reader read every missing or malformed field but one as empty,
though every generation that settled an artifact writes each of them,
and credited a proposer its own chunk had not heard (ledger B106).
Two readers made an empty string stand for an input and then compared it as a key:
a directory id with no Latin letter read as empty,
as did any alias in another script,
so the id stood on an alias that was not it (ledger B131);
and a link destination the trim cut to nothing compared equal to every other one it emptied,
so a page that dropped the parent directory and kept the current one dropped nothing (ledger B139).

A textless confirmation recorded the deterministic reader's count as `0`
where that reader never ran (ledger B158).
A reading that failed reaches the verdict as a failure,
never as an empty value.

The destination reader cut every destination where prose would end an address,
so two explicit links differing only past a full-width comma read as one (ledger B173).
The trace of a dropped destination searched each slice for the address as a substring,
and found a shorter address inside a longer one and a full stop in every sentence (ledger B174).
The site root,
shed of its trailing slash,
keyed as the empty destination (ledger B175).
A rule written for one kind of input is applied to that kind alone,
told apart by what the parse says,
and a key never maps a value that names something onto the key of nothing.

A percentile no rank belongs to was answered with zero (ledger B272);
a routed call that left its loop with no attempt left said every provider refused it,
under a comment calling that end unreachable (ledger B275);
seed detection skipped an issue naming a slice the preparation lacks (ledger B276);
and a score command given a named file that was absent scored without it,
under the note for a file nobody named (ledger B285).
Absence is an answer only for an input nobody named,
and a comment calling a loop's end unreachable is checked against every `continue` in the loop.

An agent's regression count looked the incumbent's count up by position with `?? 0`,
so a missing count would have read as zero (ledger B323,
M137).
A value that belongs to a key is carried with that key,
and a read that cannot miss throws if it does.
The archive review looked a block's source section up with `?? ''`,
and so did the alignment lookup behind the map,
though an empty section is itself an answer (a section only the archive has);
a miss checked every quote against nothing and dropped each reply that cited the original (ledger B717).
Where an empty value is a legitimate answer,
a miss is not it:
the lookup throws a named error carrying the key and what the map holds.

A commit probe that threw in the process,
its working directory removed before git ran,
replaced the failure of the read it was asked about with its own (ledger B374).
A failure caught while another is being handled is carried with it,
as an `AggregateError` of both.

The rule:
a call states every input that decides what a floor refuses,
what a sheet shows or whether a text may ship,
even where the default happens to equal it;
a call that leaves one out says why at the call or in the guard's named list.
A gate flag never defaults to the permissive answer without every caller stating it.
A reading that failed reaches the verdict as a failure,
never as an empty value a check then passes.
No branch is kept for a fixture production cannot build:
an input that breaks the caller's contract is refused with a named class,
and the fixture changes.
A reader of stored records refuses a field every generation writes when it is absent,
naming the path,
and keeps an absent reading only for a field some generation never wrote,
saying which.
A refusal added to such a reader is run over every stored record at the builds before and after it,
and each record whose reading moves is accounted for.
A check re-read mid-run applies every rule the first read applied.
A caught error is logged,
rethrown,
or returned as data carrying it;
a parse whose failure is ordinary goes through `parseModelJson`,
never a catch that drops the error.
Each exported function has a name no other file exports,
chosen for its role and its boundary.
Text is sliced by index,
never grown a character at a time.
A trim or reading that can come out empty says what the empty result means,
and an empty reading is never compared as a key:
one key never stands for two inputs a reader would follow to two places.

What enforces it:
`directory-id-name.unit.test.ts` and `dropped-destinations.unit.test.ts` hold inputs whose readings are empty;
`floor-inputs-stated.unit.test.ts` fails on a production call leaving a floor input or gate flag out,
unless the call is named with its reason or sits in a named measurement file,
and on a named call or file that no longer leaves anything out;
it cannot see a key nested in a named type or a parameter typed by a named alias.
`exported-function-names.unit.test.ts` fails on a name two source files export.
`caught-errors-kept.unit.test.ts` fails on a catch clause that binds nothing,
discards its error with `void`,
or neither logs,
rethrows nor names it;
a clause that names the error only to test its class passes,
so review still reads what a clause returns.
`text-accumulators.unit.test.ts` fails on a `let` begun as text and grown inside a loop,
unless it is named as reading no text.

## Fields carried as strings

What happened:
footnote mentions were keyed as `role convention identifier` strings,
and four readers split the keys back into parts three different ways,
two with a fallback for a part the key always has (ledger B35).
A draw looked artifact file names (`whiskers.json`) up in a map keyed by entry id (`whiskers`),
both plain strings,
so no lookup matched and every real sample manifest said no kept entry recorded a pipeline digest;
the unit fixture keyed its map by file name too,
agreeing with the defect rather than with the map's builder (ledger B63).
The lone-tag mask recorded each tag's offsets and returned only its text,
so the container deficit searched the slice for that text,
found a whole element of the same name beside the container,
and counted that element's blocks as the container's (ledger B67).
The shortfall budget named each passage by its position written as text,
and its callers read the admitted names back with `.map(Number)` (ledger B73).

The rule:
a producer hands its readers fields;
a key string is built where keys are compared,
and nothing reads a part of one back.
A reader that needs a part of a key reads the field it was built from.
A finder hands its readers the offsets it found,
and a reader never searches the text again for what was found:
the search finds the first or last copy,
not the one the finder meant.
Two strings that name one thing differently are two types
(`ArtifactFileName` against an entry id),
and a lookup is fed the key its builder used.
A number handed through the package's own functions stays a number,
never written as text for a helper to hand back.
A fixture standing in for a lookup another module builds is paired with one case that drives that module.

What enforces it:
habit and review;
a fallback on a part a value always has shows in the coverage census as a stretch no test can reach.
The artifact name's template literal type (`` `${string}.json` ``) refuses a bare entry id where a file name is expected.

## Numbers read back from text

What happened:
the slice-cost reader took a count as whatever `Number` made of the field's text,
so an empty field read as 0,
`0x1F` as 31,
`1e3` as 1000,
a leading sign as part of the count,
and a digit run past the largest exact integer as a neighbouring number,
though its writer only ever writes the digits of an integer (ledger B71).
It also checked each field in one loop and read it again through helpers that threw when the loop had not,
throws no line could reach.
The same reading ran through the package (ledger B73):
counts in its own log lines,
sheets and declarations,
the dials and flags an operator types,
the prices and lengths providers list,
and the stamp on every log line,
which `Date.parse` read without its zone as local time
and,
cut off part way,
as NaN that one reader kept as a call's end and another as a sample's time.
A probe counting the family first missed readers handed on as values (`.map(Number)`) and `Date.parse`
(ledger M90),
and a call about what providers write was first made from the package's fixtures,
not their stored listings (ledger M91).

An offset was read out of a parse refusal's message,
which quotes a short file back whole,
so a file's own words gave the offset (ledger B274);
and a request rate written with a fraction was read as a rate,
and made the pace read its list of starts between two entries (ledger B292).
Text a message may quote is never searched for a phrase the message uses as structure,
and a count of places is a whole number of one or more,
or is refused where it is read.
The same held for the ledger:
the lead's tooling lists a ledger passage as a choice awaiting the owner by a marker phrase,
and two passages of a process entry that quoted the phrase were listed as such choices (ledger M141).
A record that quotes a marker phrase its tooling matches on is reworded,
or the match is anchored where the marker stands.

The rule:
a reader of a number in text accepts only the spelling its writer writes,
through a shared rule or a round trip through the writer:

- a count is ASCII digits a double holds exactly (`isWholeNumberText`),
  leading zeros read as written,
  and every refusal cites `WHOLE_NUMBER_RULE`;
- an amount an operator types is a plain decimal (`isDecimalText`),
  then finite;
- a number a service writes as text takes JSON's spelling (`isUnsignedNumberText`),
  then finite and not under zero;
- a log stamp is what `toISOString` writes (`isIsoStampText`),
  and a line whose stamp is not is left out and counted,
  never placed at a time nobody wrote;
- a value the package wrote with `String` is read back only when `String` spells it again.

Each field is read once,
into its value or its refusal,
and the record is built from those readings.
A number an operator types is read by the same rules,
a blank variable reads as unset,
a minus sign before digits is answered as a count under zero,
and the refusal says what leaving the value off does.
A number passed between the package's own functions stays a number:
it is never written as text only to be read back.
Before counting a family,
list every way the language reaches the behaviour
(a call,
a construction,
a value handed on,
a coercion);
before deciding what a writer writes,
measure its stored output with a control.

What enforces it:
`number-reads.unit.test.ts`,
which `source-scans` runs,
fails on a call of `Number`,
`parseInt`,
`parseFloat`,
`BigInt`,
`Number.parseInt`,
`Number.parseFloat` or `Date.parse`,
a `new Date` given an argument,
a unary plus on anything but a literal,
or any of those readers handed on as a value,
until the read is named with the rule that holds it;
`whole-number-text.unit.test.ts` and `iso-stamp-text.unit.test.ts` hold the shared rules,
and `iso-stamp-text.test-fixture.ts` lists the stamp spellings every stamp reader is shown refusing.
Out of the scan's reach:
a reader reached through `globalThis` or a renamed binding,
and arithmetic on text.

## Command lines

What happened:
every runner found its own flags with `indexOf` or `includes` on the exact token,
so `--cap=0`,
a mistyped `--olny`,
a second `--only`,
or any argument to a runner that reads none
read as not written,
and the runner ran what nobody asked for:
every pending corpus entry,
every subject the settled audit could buy,
or the seated roster in place of the models named (ledger B75).
A flag written last or before the next flag read as unwritten,
a plain-object table looked up by a typed name found `Object.prototype`'s keys,
the mise task descriptions,
written by hand,
had drifted from what five runners read,
and three refusals did not show what was typed,
one of them not even which part of the line was wrong.
An `--only` naming an entry a runner could not reach was dropped without a word,
so the runner ran over nothing,
or over the rest when the stray stood beside real entries (ledger B76).
The line a pass prints for an `--only` said every entry it names runs,
where a named entry that has finished does not (ledger B376):
an operator line is read against every state of what it names.

The rule:
a runner declares what it reads in `corpus-run/command-lines.ts`
(valued flags,
repeatable flags,
switches and positions),
hands `process.argv` and its own name to `reportingRefusals`,
and reads only the line that call hands it;
a flag's value goes through `corpus-run/command-flags.ts`.
The reader refuses,
all at once and ending with the usage line,
anything the declaration does not name or a value it cannot use,
and every refusal names the part that is wrong and quotes what was typed.
A table keyed by typed text is a `Map`,
and a list the command line names,
such as the card providers,
is written once and read by both the declaration and the reader.
A task description,
and every invocation in the living docs,
names only flags the runner reads.
Ids a line names are held to what the runner can reach through `askedAmong`,
which refuses the line,
naming every stray,
before anything is spent.

What enforces it:
`command-lines.unit.test.ts` fails on a runner the build makes that the table does not declare,
a malformed declaration,
a task description out of step with its runner's flags,
or a living doc invoking a runner with a flag it does not read;
`command-line-reads.unit.test.ts`,
which `source-scans` runs,
fails on a read of `process.argv`,
an import of `parseArgs`,
or a dynamic import of `node:process` or `node:util` anywhere but a runner's own hand-off and the one reader;
`command-line.unit.test.ts` holds the reader's readings and refusals,
and `command-flags.unit.test.ts` the refusal of ids a runner cannot reach.
Habit holds a new runner that narrows its walk by id to `askedAmong`.
Out of the scan's reach:
`process` reached through `globalThis` or handed on whole,
and a property name built at run time.

## Readers of the package's own log lines

What happened:
`run-timing-report` reads back the round and completion lines a run logs.
Its reader knew one of the two round lines `runGatherRound` writes,
so a round whose quorum never stood left every figure the report printed;
a round line short of its fields was refused for a missing unit,
a completion line without its outcome read as an empty outcome,
and calls spanning no time divided by a zero span (ledger B82).

Two report commands counted the closing newline of a log as a line,
and totalled a log named twice (ledger B278).
A log's lines are what its newlines end,
and a reader that totals refuses a log it is handed twice.

The rule:
a reader of a line the package writes starts from the writer:
list every shape the writer can emit,
read each into its own kind rather than optional fields,
and refuse,
naming the field,
any line that carries the marker but none of those shapes.
Fixtures copy the writer's template for each shape,
and a figure folded from the readings counts every kind
or says why one is left out.
A division whose divisor the input can make zero is refused or guarded,
never printed as a number.

The same gap ran the other way when a writer changed:
B109 made the stream line write "1 content char",
and `cap-census-read.ts` found the content count by the text " content chars",
so a one-character stream would have read as no completed stream;
its fixtures copied the old wording and could not show it.

A writer's wording change therefore checks every reader of that line first:
search the package for the changed field's text,
and give each reader the old and the new spelling,
since run logs written before the change stay on disk.
Reader fixtures are built by calling the writer,
not by copying its template.

Three readers settled a field written twice by order,
two keeping the later value and one the first,
though each writer writes a field once,
so two records run together read as one clean record (ledger B209).
A reader says what a repeated field means,
with its module's own value for a record that will not read.

What enforces it:
`run-timing.unit.test.ts` holds a fixture line for each round shape and the completion line,
and a case for each refusal.
`cap-census.unit.test.ts` builds its stream lines through `reportStreamProgress`,
so a change to that line's wording reaches the cap-census reader's cases;
the run-timing fixtures still copy the writer's template,
so a new shape in `stage-round.ts` or `reportStreamProgress` reaches that reader untested
(ledger B82 records the follow-up).

## Provider fields read by their documentation

What happened:
the Anthropic reader reported `input_tokens` as the prompt,
as the name suggests,
though Anthropic's documentation counts in it only the tokens after the last cache breakpoint
and gives the prompt as that plus the tokens read from and written to the cache,
so a cached call read as a short one (ledger B88).
The same reader read a `text` or `partial_json` that was not a string as empty,
and passed over a delta frame with no delta,
so an answer came back with a piece missing and nothing said (ledger B89).
The scanner reading the same stream for the stream guards passed those frames over uncounted,
and block starts it could not read too,
so the tally its contract says should stay at zero
read zero for a body the reader then refused (ledger B93).
The reader also reported a count the stream never sent as 0
once the other count arrived (ledger B94).

The rule:
a field on a provider's wire is read as the provider's current documentation defines it,
fetched while writing the reader,
and a reader mapping one provider's fields onto another's shape
(Anthropic's usage onto the OpenAI-shaped `prompt_tokens`)
states each mapping beside the documentation it follows.
A gateway that serves one provider's format for another is read by the format's documentation,
and a capture that cannot be checked against it says so where it is kept.
A field the documentation makes part of a frame,
and that carries the answer,
is refused when absent or of the wrong type,
as a frame that does not parse is refused;
an empty string stands only for an empty fragment the provider sent.
A frame of a kind the reader does not read is passed over,
since the protocol may add kinds.
Where two readers walk one wire,
each counts or refuses a frame it reads and cannot,
judged by what that reader reads the frame for,
so neither reports a clean stream the other refused.
A count the provider never sent is absent,
never 0,
unless its documentation says an absent field means none,
as the cache fields do.

What enforces it:
habit and review;
`anthropic-completion.unit.test.ts` holds a stream that reads and writes the cache,
a delta frame with no delta,
a text that is a number,
a tool fragment with no `partial_json`,
and the deltas still passed over;
`anthropic-delta-scan.unit.test.ts` holds the frames the scanner counts as unreadable
beside the ones it reads or passes over.

A published per-request figure is the same kind of input (P19).
The NeuralWatt meter held each call at the energy matrix's mean for its prompt-size bucket,
doubled on a prefill argument nothing had measured;
the first run's settled calls read GLM-5.3-Flash at more than three times the matrix
and traced the spread to completion tokens,
which a prompt bucket cannot see.
A provider's published mean that sizes a hold is a placeholder until the pipeline's own settled calls replace it,
and the replacement reads what drives the spread before choosing any multiplier.
The replacement is drawn from the rotation of entries the pipeline runs and not from one run:
the means of one run of one entry (`zheermao19`) read 2.6 times low in the main bucket
once three entries were pooled (ledger B808).
`neuralwatt-estimate.unit.test.ts` pins each figure with the run it came from.

An owner's lift of a ban is applied at the scope the owner's words state (ledger B807).
"I'm lifting the ban on passing params,
but only for NeuralWatt" was built for the one call kind that had failed that day,
the picture reading,
and the same model ran to its completion cap at the provider's default effort on 11 of 288 text calls over the next four passes,
three runs in a row on the letter block alone.
The rule:
a lift is built for the scope its words name,
and the kind of call that prompted it is recorded as the evidence for the lift,
never as its edge;
a narrower build says in the ledger entry which part of the lift it leaves unused and why.
A parameter is sent only for a model whose supported values the repo records from the provider's documentation,
and a model with no recorded list gets none,
so the lift never becomes a guess at a provider's reply.
The level is chosen from the distribution of the calls that did not cap,
read over every pass log that holds the model,
with the capped calls counted beside them,
and the entry says that no measurement exists at the chosen level until a keyed run provides one.
`neuralwatt-text-effort.unit.test.ts` pins the value on the text calls of the model that lists it,
none on the model that does not,
and that only the NeuralWatt side of a card can name one.

## Two layers reading one refusal

What happened:
the budget layer held a refuser whose meter read wet for no time
when every other provider was dry,
so the calls kept going to it,
and the router read the same refusal as a reason to call that provider dry,
found no hold to wait out,
and ended the call as if no provider had budget,
166 times on Uekawakuyuurei while Synthetic's meter read wet (ledger B124).
Each layer's own test held its half;
no test ran a refusal through both.
The same shape sat inside one module:
the introduced-defect screen decides which side of a prober's claim it anchors on,
and two later readers,
the quote compared with the accepted issues and the grading sheet's direction,
decided again by whether the raw omitted text was empty,
so a claim padded with spaces was counted as introduced damage and shown to a grader as dropped wording
(ledger B128).
The blankness census found the shape three more times:
a rendering-audit claim's side decided blank by two readers,
a reply's emptiness by two transport readers,
and an archive block revision's by a stage that never asked and a floor that asked another way
(ledger B189,
B191,
B192).
And a refusal the provider states in a stream's error event was retried,
where the same refusal as an HTTP status is returned at once (ledger B202):
the ladder now reads both as one refusal.

Seven refusals about a value the operator set or a file the operator named were authored,
marked sentences,
and the commands' boundary printed each as a fault of the command,
with frames and the fault's exit code (ledger B221):
the thrower read its refusal as the operator's mistake and the printer as a bug.
A refusal's class says which of the two it is,
and a case drives each such class through the printer.

Five of the classes B221 left became stated refusals,
and the sixth stays a fault with its reason said (ledger B254).
The runners that no test had run as built held the same shape many times over:
an unreadable or missing log,
a missing `artifacts` directory,
a sheet and a manifest of different lengths,
a listing that is not JSON,
an empty corpus,
a roster too narrow,
each with a message telling the operator what to do,
printed as a fault with frames (ledger B278,
B284,
B290,
B296,
B303,
B307);
and a census run where no build stands raised its listing error uncaught,
printed as a fault at exit 5 (ledger B311).
Two settings that can refuse were read in module constants,
before the boundary that prints refusals ran,
and exited 1 under a dump of source (ledger B302);
the corpus pin was read that way too,
at the load of `run-config.ts` (ledger B381),
in 34 of the 43 built bundles,
until each entry read it inside its boundary on 2026-10-10 (ledger B394).
The slice overlap was read inside the boundary but for each entry,
so an unreadable value refused the pass after the runs lock was taken and the first attempt recorded (ledger B400).
The artifact pool's contradictory pair of variables was a plain `Error`,
printed as a fault at exit 5 (ledger B380);
and a fidelity reference request refused for what the operator typed printed under frames at exit 5,
its class a fault where a reference the clone holds fails its hashes (ledger B382).
A refusal that is always the operator's is a stated refusal by class,
never caught and wrapped where it happens to be printed;
a class that is a fault elsewhere is restated at the one site where it is the operator's;
and a setting that can refuse is read inside the boundary,
once,
before the command claims a directory or records anything (ledger B400).
A refusal that arrives in a 200 stream's error chunk with a 4xx code is returned unretried,
as the same refusal over plain HTTP is (ledger B251).

The rule:
where one event is read by two layers,
each layer's reading is stated beside the other's,
and a test drives the event through the real code of both,
stubbing only the wire and the meters,
for every reading the first layer can give it
(here:
held for the backoff,
held for a named wait,
held for no time,
read dry until paid).
A layer that folds in an earlier layer's evidence reads what that layer decided,
such as the hold it set,
or the side a screen's verdict names,
not the raw event the decision was made from.
A reader that can only be handed what the decision admitted
throws `unreachable:` for anything else,
rather than deciding it a second way.

What enforces it:
`provider-router.unit.test.ts` holds "ASKS A WET REFUSER AGAIN",
which runs the refusal through the real budget layer and the router;
`budget-hold-wait.unit.test.ts` holds the reading for a refuser with no hold;
`introduced-defect-screen.unit.test.ts` holds the padded claim beside the same claim with an empty side,
and `probe-verify-sheet.unit.test.ts` holds
"NAMES a claim the screen corroborated on its evidence as added"
and the refusal of a claim the screen did not corroborate.
`corpus-run/cli-refusal.unit.test.ts` holds a case for each of the seven classes of ledger B221,
asserting the printed line and the exit code;
each moved runner's suite "<command> as built" asserts the printed line and the exit code of its refusals,
from a child process;
`corpus-run/corpus-pin-entries.unit.test.ts` holds each command other than `corpus-pass` that reads the corpus pin
at exit 6 with its stated line alone,
and one that never reads it running past an unreadable commit (ledger B394);
no scan finds an operator's mistake still thrown as a class the printer reads as a fault.

## Rulings held as prose

What happened:
the owner ruled on 2026-10-20 to leave Bedrock off rather than issue a new key (ledger B760),
and the ruling stood in a handover only.
Every run afterwards held the expired key,
asked Bedrock once,
and was refused with HTTP 401 before it held the provider out.

The rule:
an owner ruling that changes what a run does is data the run reads
(`PROVIDERS_LEFT_OFF` in `run/src/corpus-run/providers-off.ts`),
with the reason beside it,
and the launch prints it.
Code that serves the ruling reuses an existing path
(here the unset key),
and a brief that says a mechanism exists is checked against the source before it is relied on.

What enforces it:
`run-client-bedrock-off.unit.test.ts` sets the key and asserts the transport sees no Bedrock exchange;
`providers-off.unit.test.ts` pins the list;
the required-providers gate refuses an off provider with the reason "left off".
A second ruling of this kind is outside what these can see until it is added to the list.
