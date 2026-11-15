# Secret values reaching a model provider from a Pi run

## What happened

On 2026-11-14 at 09:11:15 a Pi run this package started
was working out why `mise` could not decrypt `.env.local.json` under a disposable `HOME`.
It ran `mise env 2>&1 | head -3`.
The tool result held the real value of one credential that file declares,
and a tool result is sent to the model whole.

The rules file every brief names already forbids printing the environment
(`doc/rules-every-pi-brief-carries.md`).
That was not enough twice over:
the agent did not read `mise env` as printing the environment,
and nothing stood between the command's output and the model.

A coordinating session afterwards scanned 1,837 Pi sessions under `${HOME}/temp/agent/pi-runs`
for the real values of the 35 names `.env.local.json` declares,
by exact match.
It found real values of 30 names, in six sessions, each in a tool result.
The commands before those results:

- `mise env --json 2>/dev/null | head -40` (29 names in one result) and
  `mise env 2>&1 | head -3`.
- `tr '\0' '\n' < /proc/$pid/environ | grep -i "CARGO_TARGET\|MISE\|RUSTUP"`
  and `env | rg -i "CARGO_TARGET|MISE"`:
  a filter for one harmless word that also matched a token's name.
- `sed -n '40,70p' ~/.bashrc`:
  a file outside the repository that holds a token in clear.
- `cat ~/.pi/agent/models.json`:
  Pi's own provider file, which holds keys in clear.

## Every road from an armed value to a provider or a kept file

Two reviews found twenty-two roads between them, and the second found none of the first's again: each fix held and the list was
still not complete either time. So the table is written from the places a value could be written or sent, and every road is a row of
it, with the one point that covers the road and the control that shows it.

Where a row says "no hook", that is Pi's own limit, and the row also says what the run does about it.

| Road | The one point that covers it | The control |
| --- | --- | --- |
| A tool result's `content` | the `tool_result` hook, masked as data (`secret-value-masking-extension.ts`) | "masks a value a command printed inside a longer line" |
| A tool result's `details` | the same hook, as data, and returned in the patch | "masks a tool result details through Pi own tool loop, the driver feed and the session file" |
| A tool result's `structuredContent` | the same hook; a field that did not change is returned as it came, so Pi keeps it | "keeps the structured content a tool declared when only its text was masked" |
| Error text of a tool that threw | the same hook, which sees the error result | the review's positive control, and "keeps … the reply whole" |
| A key of an object a tool returned | the same hook: the mask is applied to the data, so keys are masked with values | "has its keys masked, because the mask is applied to the data the object serialises to" |
| A `toJSON`, a getter or a cycle in an object | the same hook: the round trip hands back plain data, which carries no method and no cycle | "cannot reintroduce a value through a toJSON method" and "is withheld whole when its details cannot be masked" |
| A result the mask cannot write | the same hook withholds the whole result, with explicit details, so Pi keeps no field of it | "is withheld whole when its details cannot be masked, and Pi keeps no field of it" |
| The first prompt, a steer, an inbox note | the `message_end` hook, before Pi keeps the message | "masks the message Pi keeps, which is what its compaction request is built from" |
| An assistant message with its tool calls | the same hook | the same control, and "masks the arguments of a tool call on the feed" |
| Any other message Pi keeps, custom messages included | the same hook | "masks the message Pi keeps…" |
| The ordinary model request | `context_with_system` | the review's positive control, and "masks a value a session already held, which is what a resumed run sends again" |
| Pi's compaction request | no hook; its input is the session's messages, which `message_end` has masked, and a session a run resumes is masked before Pi starts | "masks the session a run resumes before Pi can read a value out of it" |
| Pi's branch-summary request | the same: the session is the only input | the same control, by reading Pi's `branch-summarization.js:208` |
| The driver's feed: tool arguments, results, an inbox preview, an event's detail | one boundary in `record-of-pi-run.ts`, which no caller can leave out | "masks the arguments of a tool call on the feed" and "keeps a value a tool printed out of every file the run writes" |
| The driver's summary: the last reply, a tool error, what Pi printed when a start failed | one boundary in `write-summary.ts` | "keeps a value a tool printed out of every file the run writes" |
| Pi's standard error, in which an extension may print a value | one stream matcher in `masking-a-stream.ts`, through `keep-standard-error.ts` | "holds back the beginning of a value that a cut would split, so the two writes mask it together" |
| Pi's file of a truncated tool output | the `tool_result` hook, and only a file whose whole name is Pi's own pattern | "rewrites the file Pi saved a truncated output in" and "leaves Pi own output whole when a tool result names a file whose name is only its prefix" |
| The session file at rest: compaction entries, branch summaries, context edits, an extension's custom entry | no hook before `appendCustomEntry`, `appendCompaction` or `appendContextEdit`; the file is masked whole once the run is over, and only while it is still the file that was read | "masks the session a run leaves behind, whatever wrote an entry of it" and "keeps what another command appended while the masked text was being prepared" |
| A session that still holds a value when the run is over | nothing, and that is the answer: the run ends `session-unmasked` and the summary says where the value is | "does not end clean when the session a run leaves still holds a value" |
| The age identity, the key that opens every store | an identity the machine declares and this code cannot read stops the run before Pi starts | "stops the run when an age identity the machine declares cannot be read" |
| A shell start-up file's literal, quoted or hash-leading | the shell's own quoting and comment rules, and a value the reader cannot give stops the run | "arms the literal a shell start-up file exports with a hash at its start, and stops the run on a value it cannot give" |
| A key whose value begins with `/` | the mask believes the source that says a field is a credential, not the shape of the value | "arms an OAuth credential Pi own schema names, a shell value with a comment after it, and a key that begins with a slash" |
| OAuth `access` and `refresh` | Pi's own credential schema for `auth.json` | the same control |
| A source that is there and cannot be read: a pipe, a folder, a file that is not JSON | the ordinary-file read, and the stop before Pi starts | "stops the run when Pi own credential file is there and cannot be read" and "refuses a named pipe at a source path instead of waiting on it" |
| A named pipe where Pi's output file is named | the same ordinary-file read, refusing it | "does not wait on a named pipe a tool result calls Pi own output" |
| A masked file whose write fails | the replacement is a file of its own, renamed over the original only when whole | "leaves Pi own output whole when the masked text cannot be put in its place" |
| A marker that names a value, or a second pass that changes the text | a marker names the variable only when its name holds no armed form, or says `#N` instead; a pass that leaves a form is refused | "never puts an armed value, or a form of one, inside a marker it writes" and "changes nothing when it runs over its own answer a second time" |
| A tool result that masks nothing, on a result of ordinary data | the same hook, which hands back the same data | "answers the result as data when no value stands in it" |

### The four classes a third review found, and what answers each

The third review found eight roads, five of them in classes the earlier rounds had called closed. The answer to a class is a shape,
not a row, so each of these is a row of the table above and a stated limit where a shape cannot close it.

- **A file replaced while another writer may append to it.** One decision, `FileWriterAnswer`, is what every replacement asks for:
  a caller that cannot say the file's writer is gone gets a refusal, not a try. The three call sites answer it explicitly: a session
  a run resumes is never replaced at all (it is masked into a copy that Pi resumes, and the original keeps what it holds, which is
  the stated limit), the session at rest is replaced only when Pi's leaving was confirmed, and Pi's output file only because Pi's own
  tool closed it before the result existed (`finishOutput` awaits `closeTempFile`). What no shape closes is a process outside this
  package appending in the window between the last read and the rename; that window is the stated limit, and the identity check
  reads the file again rather than pretending the window is closed (`a-file-is-replaced-only-when-its-writer-is-gone.unit.test.ts`).
- **"Could not be read" answered as "not there".** One function decides it, `presenceOfPath` and `presenceFromFailure`: only the
  operating system's own "no such file", with every parent askable, is absent; a denied file, a denied parent, a wrong type, a pipe,
  a name too long, a link that loops and a missing answer are all could-not. Every source and every kept place asks it, and a
  could-not stops the run or leaves an ending that is not clean
  (`every-source-answers-absent-could-not-or-values.unit.test.ts` walks each place through absent, a folder, a denied file, a denied
  folder above it, a pipe, and readable).
- **The driver reads shell text with a parser of its own.** The reader now refuses every line it cannot read exactly (one quoted or
  unquoted word, nothing after it but a comment) and takes the value from the environment the driver was started with, by name, when
  it cannot; without that, the run does not start and the message names the file and the line. A second answer to the class is
  stated: a value only the shell computes and only the file holds is one this code cannot arm, and the run stops rather than guess
  (`a-shell-value-is-what-the-shell-exports-or-a-stop.unit.test.ts` holds every spelling against real `bash`).
- **A second way in that the driver does not look at.** The free-form Pi words are read with Pi's own `parseArgs`, and every spelling
  that names or resumes a session is refused with its reason, because the driver masks only the session its own `--session` names
  (`free-form-words-that-resume-a-session-are-refused.unit.test.ts`). The other free-form words were listed: `--system-prompt` and
  `--append-system-prompt` reach the request and are masked by `context_with_system`; `--extension` and `--mcp` reach tool results,
  which `tool_result` masks, and stderr, which the stream matcher masks; `--skill` and `--prompt-template` reach the prompt, which
  `context_with_system` masks; `--no-session` leaves the session at rest with nothing to mask, which the run records; `--api-key` puts
  a credential on Pi's command line, which any process of the account can read, and is named here rather than refused.

Two single roads of the same review are rows above: the spelling a credential file writes a value in (`\uXXXX`) is armed beside the
decoded value, and the kept standard error's destination failure is reported through the copy's own error listener instead of
ending the driver as an unhandled stream error.

### How the rows were looked for

The list was made from Pi's installed source, not from the hooks this package knows about. Two searches, each over Pi's own
`dist/`:

- **Every place Pi builds a request.** `grep -rn 'streamFn(\|serializeConversation('` over `core/`, `core/compaction/` and
  `modes/rpc/` found the ordinary request (`core/sdk.js:271` through the model runtime), the two compaction paths
  (`core/compaction/compaction.js:486` and its conversation text at `:524`, `:731`) and branch summarization
  (`core/compaction/branch-summarization.js:208`). Those are the only request builders: every other module passes messages to one
  of them.
- **Every place Pi writes.** `grep -rn 'appendFileSync(\|writeFileSync('` found the session's JSONL (`core/session-manager.js:368`
  and `:813`), Pi's own credential store and package manifests (`core/auth-storage.js`, `core/package-manager.js`, which write Pi's
  configuration and never a run's text), Pi's crash log (`core/crash-log.js:26`, which records error messages and is named under
  the limits below) and the output files (`dist/utils/output-files.js`). The entry kinds that reach the session were then listed
  from `appendMessage`, `appendCustomEntry`, `appendCompaction`, `appendContextEdit` and the bash-execution appends at
  `core/agent-session.js:3304` and `:3332`.

A fifth class was looked for the same way and not found: the two searches above cover every place Pi builds a request and every place
it writes, the driver's own writes are one boundary each, and the two reviews' twenty-two roads were then walked against the class
shapes above. What remains outside them is named, not missed: a process outside this package writing a file the driver replaces
while it replaces it, a value only a shell can compute, and the three masking limits below.

Two rows come out of that second search and are named rather than covered: Pi's **crash log** (a crash record holds Pi's own error
text, and the driver stops Pi rather than crashing it) and the **bash-execution messages** of a `!` command, which RPC mode never
makes.

## What the masking does not cover

- A container the value was encoded inside: base64 of a whole file, hex, gzip, base64 wrapped at column 76,
  a substitution cipher. A decoder in front of every matcher would be needed, and this repository's
  leaks were the value itself.
- A value shorter than eight characters, because such a string appears in ordinary text by chance.
  Every value the store declares today is at least ten characters
  (`keep/measure-store-value-lengths.ts` in the report's scratch folder).
- A steer message's preview on the run's feed, which is a driver-written line
  (`src/watch-pi-run.ts`), is not masked: the message itself reaches the model masked.
- Pi's output files for a *greater* output than it shows are rewritten by the extension only when the
  details name a file Pi named `pi-output-*`.

## Fix

`README.md` under "What is masked before it reaches the model". The values are read once in the driver,
travel on a descriptor the driver opens for Pi, and are replaced in each tool result and again in the whole
transcript by a marker that names where the value came from.
