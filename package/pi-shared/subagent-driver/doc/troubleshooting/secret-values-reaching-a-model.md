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

Two reviews found twenty-two roads between them, and the second found none of the first's again: each fix held
and the list was still not complete either time. So it is written from the places a value could be written or
sent, and every road is one entry here, with the one point that covers it and the control that shows it.

Where an entry says "no hook", that is Pi's own limit, and the entry also says what the run does about it.

- **A tool result's `content`.** the `tool_result` hook, masked as data (`secret-value-masking-extension.ts`)
  Control: "masks a value a command printed inside a longer line"
- **A tool result's `details`.** the same hook, as data, and returned in the patch Control: "masks a tool result
  details through Pi own tool loop, the driver feed and the session file"
- **A tool result's `structuredContent`.** the same hook; a field that did not change is returned as it came, so
  Pi keeps it Control: "keeps the structured content a tool declared when only its text was masked"
- **Error text of a tool that threw.** the same hook, which sees the error result Control: the review's positive
  control, and "keeps … the reply whole"
- **A key of an object a tool returned.** the same hook: the mask is applied to the data, so keys are masked with
  values Control: "has its keys masked, because the mask is applied to the data the object serialises to"
- **A `toJSON`, a getter or a cycle in an object.** the same hook: the round trip hands back plain data, which
  carries no method and no cycle Control: "cannot reintroduce a value through a toJSON method" and "is withheld
  whole when its details cannot be masked"
- **A result the mask cannot write.** the same hook withholds the whole result, with explicit details, so Pi keeps
  no field of it Control: "is withheld whole when its details cannot be masked, and Pi keeps no field of it"
- **The first prompt, a steer, an inbox note.** the `message_end` hook, before Pi keeps the message Control:
  "masks the message Pi keeps, which is what its compaction request is built from"
- **An assistant message with its tool calls.** the same hook Control: the same control, and "masks the arguments
  of a tool call on the feed"
- **Any other message Pi keeps, custom messages included.** the same hook Control: "masks the message Pi keeps…"
- **The ordinary model request.** `context_with_system` Control: the review's positive control, and "masks a value
  a session already held, which is what a resumed run sends again"
- **Pi's compaction request.** no hook; its input is the session's messages, which `message_end` has masked, and a
  session a run resumes is masked before Pi starts Control: "masks the session a run resumes before Pi can read a
  value out of it"
- **Pi's branch-summary request.** the same: the session is the only input Control: the same control, by reading
  Pi's `branch-summarization.js:208`
- **The driver's feed: tool arguments, results, an inbox preview, an event's detail.** one boundary in
  `record-of-pi-run.ts`, which no caller can leave out Control: "masks the arguments of a tool call on the feed"
  and "keeps a value a tool printed out of every file the run writes"
- **The driver's summary: the last reply, a tool error, what Pi printed when a start failed.** one boundary in
  `write-summary.ts` Control: "keeps a value a tool printed out of every file the run writes"
- **Pi's standard error, in which an extension may print a value.** one stream matcher in `masking-a-stream.ts`,
  through `keep-standard-error.ts` Control: "holds back the beginning of a value that a cut would split, so the
  two writes mask it together"
- **Pi's file of a truncated tool output.** the `tool_result` hook, and only a file whose whole name is Pi's own
  pattern Control: "rewrites the file Pi saved a truncated output in" and "leaves Pi own output whole when a tool
  result names a file whose name is only its prefix"
- **The session file at rest: compaction entries, branch summaries, context edits, an extension's custom entry.**
  no hook before `appendCustomEntry`, `appendCompaction` or `appendContextEdit`; the file is masked whole once the
  run is over, and only while it is still the file that was read Control: "masks the session a run leaves behind,
  whatever wrote an entry of it" and "keeps what another command appended while the masked text was being
  prepared"
- **A session that still holds a value when the run is over.** nothing, and that is the answer: the run ends
  `session-unmasked` and the summary says where the value is Control: "does not end clean when the session a run
  leaves still holds a value"
- **The age identity, the key that opens every store.** an identity the machine declares and this code cannot read
  stops the run before Pi starts Control: "stops the run when an age identity the machine declares cannot be read"
- **A shell start-up file's literal, quoted or hash-leading.** the shell's own quoting and comment rules, and a
  value the reader cannot give stops the run Control: "arms the literal a shell start-up file exports with a hash
  at its start, and stops the run on a value it cannot give"
- **A key whose value begins with `/`.** the mask believes the source that says a field is a credential, not the
  shape of the value Control: "arms an OAuth credential Pi own schema names, a shell value with a comment after
  it, and a key that begins with a slash"
- **OAuth `access` and `refresh`.** Pi's own credential schema for `auth.json` Control: the same control
- **A source that is there and cannot be read: a pipe, a folder, a file that is not JSON.** the ordinary-file
  read, and the stop before Pi starts Control: "stops the run when Pi own credential file is there and cannot be
  read" and "refuses a named pipe at a source path instead of waiting on it"
- **A named pipe where Pi's output file is named.** the same ordinary-file read, refusing it Control: "does not
  wait on a named pipe a tool result calls Pi own output"
- **A masked file whose write fails.** the replacement is a file of its own, renamed over the original only when
  whole Control: "leaves Pi own output whole when the masked text cannot be put in its place"
- **A marker that names a value, or a second pass that changes the text.** a marker names the variable only when
  its name holds no armed form, or says `#N` instead; a pass that leaves a form is refused Control: "never puts an
  armed value, or a form of one, inside a marker it writes" and "changes nothing when it runs over its own answer
  a second time"
- **A tool result that masks nothing, on a result of ordinary data.** the same hook, which hands back the same
  data Control: "answers the result as data when no value stands in it"

### The four classes a third review found, and what answers each

The third review found eight roads, five of them in classes the earlier rounds had called closed. The answer to a class is a shape,
not a row, so each of these is an entry above or a stated limit where a shape cannot close it.

- **A file replaced while another writer may append to it.** One decision, `FileWriterAnswer`, is what every replacement asks for:
  a caller that cannot say the file's writer is gone gets a refusal, not a try. The three call sites answer it explicitly: a session
  a run resumes is never replaced at all (it is masked into a copy that Pi resumes, and the original keeps what it holds, which is
  the stated limit), the session at rest only when the Pi this run started has been reaped, and Pi's output file only because Pi's
  own tool closed it before the result existed (`finishOutput` awaits `closeTempFile`). After the fourth review closed the fallback
  that rewrote a resumed session, every file this path replaces is one the run itself made, so the window reaches only a program
  nobody started on that path, as the limits below say
  (`a-file-replaced-only-when-its-writer-is-gone.unit.test.ts`).
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

### The four defects the fourth review found, and what answers each

The fourth review found four, in classes the earlier rounds had named. Two were a value armed wrongly or not at
all, one rewrote a file the run does not own, and one was the rename window of the third round's limit, shown to
reach further because of the first.

- **A resumed run rewrote the original session when Pi's own answers named no session file.** The at-rest
  rewrite takes a value only `sessionFileToMaskAtRest` hands out, and that function refuses the session the
  caller named by name: a run that made a copy has handed Pi the copy, so the file it was given belongs to
  another run. The masker itself refuses any object it did not hand out, so a fallback to the given path is not
  a value this code can build (`a-session-file-this-run-may-rewrite.unit.test.ts`, and the driver test `leaves
  the session it was given alone when Pi names no session file of its own`).
- **A value the store declares and the environment already holds was armed by neither pass.** The set of names
  is now what the store declares, never what one reader happened to return, and each declared name is asked of
  every place its value can come from. The reader this code asks keeps the environment's precedence, so a
  shadowed store value is read by asking that reader with the shadowing names absent: both values are armed
  when both exist and differ, because a tool can print either
  (`every-name-a-store-declares-is-armed-from-every-place.unit.test.ts` walks a declared name through the store
  only, the environment only, both the same, both different, and two names one from each place). The
  consequence is named in the limits below: the store is now always read.
- **A continued line armed a text Bash never exported.** This one is beyond the line the owner drew for the masking,
  and it is kept because it is built and controlled, not because a way through had to be closed: a text Bash never
  exported is not a credential, so arming it masked ordinary words and lost nothing. A backslash and a line break
  are one joined word: the
  file is read as the lines the shell reads, each joined line keeping the number of the physical line it began
  on, and a value that ends with a backslash and no line after it is a line the reader refuses rather than one it
  arms with the backslash in place. The control is made from every character Bash treats specially at each place
  in a word and at the end of a word before a line break, each row held against real Bash
  (`a-shell-value-for-every-special-character-is-what-bash-exports.unit.test.ts`): the reader's value equals
  Bash's, or the reader refuses and the run stops naming the file and the line.
- **An append after the last identity check was discarded by the rename.** This is the third round's stated
  limit, and the first defect above is what made it reach further than the limit said. Every file this package
  replaces through the identity-checked path is now a file the run itself made: the session copy it wrote for
  Pi, or the session its own Pi created, whose writer is the Pi this run reaped (`piIsGone` is the supervisor's
  own `stopped` and the connection's `hasExited`), and Pi's own output file of a tool result, whose writer is
  the tool that returned before the result existed. No file another run owns is replaced at all, so what stands
  is one window, exact in the limits below: a program nobody started on a path this run owns, appending between
  the last identity read and the rename.

### How the names and the files were looked for

Two searches answer where a *set* of names or of files could be taken from what a reader returned rather than
from what was declared. A set of names: every `new Set(`, every `Object.keys(` and every `.map(` over a
reader's answer, each read for whether the names came from a declaration. A set of files: every `writeFile`,
`appendFile` and `rename` in the package, each read for whose file it is and who could hold the path. What that
found: the store's declared names (fixed above), the names of Pi's own credential fields (from Pi's schema,
which is a declaration), the names a start-up file declares (a declaration in the file itself), and the session
file Pi names (one path, now decided in one place). The writes that are not value places are named in the
limits below.

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
- Pi's output files for a *greater* output than it shows are masked only when Pi names one that is in the operating
  system's temporary folder and whose name is exactly `pi-output-` and sixteen hexadecimal digits, with an optional
  extension (`src/pi-output-file-of-a-tool-result.ts`). A file named any other way is left as Pi wrote it.
- **A tool result whose `details` cannot be read.** The masking reads `details` to mask it. When reading it throws, the
  result is withheld, and withholding it also has to read `details` to find the full-output file a peer may have named.
  Both reads fail, so Pi keeps the tool's own result: the model sees the original content, a value in it included.
- **A result with a cycle that names a full-output file.** A result that cannot be serialised whole is withheld, which
  is right, but the withheld text only points at the full-output file: that file keeps the value Pi wrote into it.
- **A full-output file in a folder that cannot be written.** The masked replacement is written beside the file and
  renamed over it. When the folder refuses the new file, the result the model sees is masked and the file Pi wrote
  keeps the value; the run records the failure on its feed.
- **A value split across two parts of one tool result.** Each part is masked on its own, and a value whose text is
  divided between two parts matches nothing in either; reading the two parts together gives the value back.
- **The rename window, exactly.** A file this package masks at rest is read, checked again, and renamed over.
  Between that last check and the rename, a program this package did not start, holding the path of a file
  this run made, can append to it, and what it appended is replaced with the snapshot that was read. No check
  before the rename closes that window, and the fourth review showed that adding one is not a fix: the window
  only moves. What bounds it is ownership, and after the fourth review that bound holds: the only files this
  path replaces are the session file this run's own Pi wrote or the copy this run made for it, both written only
  by a Pi this run reaped, and Pi's own output file of a tool result, written only by the tool that finished
  before the result existed. A session file Pi names that this run cannot show to be its own is not rewritten at
  all: a run that made a copy rewrites only that copy, and a run that made none rewrites only a file that did not
  exist before it started, so no file of another run is written. The window therefore reaches only a file this
  run made. The run's summary, its checked-runs file and its Pi start check are written beside and renamed the
  same way, and hold counts and names rather than entries or values, so a lost concurrent write of one of them
  loses a count and no entry.
- **A store that is there and is not what the store reader reads.** Because the store is now read even when the
  environment supplies every name it declares, a `.env.local.json` that is not the sops envelope the reader
  expects stops the run instead of being skipped. That is the same rule as a store that cannot be read, and it
  is stated here because a plaintext store used to be invisible when the environment was full.
- **A shell line the reader does not join.** A backslash at the end of a value with no line after it, a computed
  value, a concatenation of a quoted and an unquoted part, and a control operator all stop the run unless the
  driver's own environment holds the name, because the joined word a shell would export is not one this reader
  can be sure of.


## The stated limits, in plain words

Read this before deciding to rotate a credential: it is what the masking does and does not catch, without the machinery.

What it catches: a value the checkout's store declares, printed as it is, is taken out of a tool result, a request Pi
builds, the run's feed, its summary, its kept standard error, and the files the run writes, its own session among
them, except for the places named further down. The values are read once at the start of a run, from the store,
Pi's own credential files, the shell start-up files, the age identity and the driver's own environment, and nothing of
them is written down anywhere.

The three limits the owner accepted in so many words:

- A value inside an encoded container: base64 of a whole file or object, hex, gzip, or any other re-encoding. The
  masking matches the text of a value, not its meaning.
- A base64 spelling is masked as base64 writes it on one line; base64 wrapped at 76 columns is a different text and
  is not masked.
- A value shorter than eight characters is not masked at all, because such a string appears in ordinary text by
  chance.

Further limits, each one a way through or a way a run stops:

- A program this package did not start, writing into a run's own folder in the moment between the last identity read
  and the rename that puts a masked file in place, can lose what it appended. No session file of another run is
  ever rewritten, and the only files this path replaces are ones the run itself made, so no session of another run
  and no kept file can lose an entry from it.
- A `.env.local.json` that is there and is not the sops envelope the store reader reads stops the run, even when the
  driver's environment supplies every name the store declares.
- A start-up file line this reader does not read exactly, such as a value the shell computes, a concatenation of a
  quoted and an unquoted part, or a control operator, stops the run unless the driver's own environment holds the
  name.
- Pi's crash log is written by Pi outside every hook and is not masked.
- Pi's own messages for a `!` command's bash execution are not masked; a run over the protocol this package uses
  makes no `!` command.
- A credential put on Pi's command line by a free-form `--api-key` word is readable by every process of the account
  and is not masked.
- A session a run resumes keeps whatever it holds: the run reads it, masks a copy, and Pi resumes the copy. The
  original is never rewritten, so a value an earlier run left in it stays there on disk.
- A value that is in neither the store, Pi's files, the start-up files, the identity nor the driver's environment is
  not known to this code and is not masked.
- A tool result whose `details` cannot be read keeps the tool's own content, a value in it included, because the
  withheld result cannot be built either. What a person would see is the tool's own output, unmasked.
- A tool result that cannot be serialised whole is withheld, and the full-output file it names still holds the value.
  What a person would see is a withheld result and a file beside Pi's session folder holding the printed value.
- A full-output file in a folder that refuses a new file stays as Pi wrote it while the result the model sees is
  masked. What a person would see is the masked result and the value still in that file.
- A value split across two parts of one tool result is not masked, because each part is masked on its own. What a
  person would see is two halves that read as the value when put together.
- **A value changed while a run is going.** The values are read once, when a run starts, and a going run has no way to
  learn a new one. A value written into a file all runs share (`~/.pi/agent/auth.json`, `models.json`, a shell start-up
  file) or into a store a working tree just merged goes through that run unmasked until it ends. Rotate while no run is
  going, or accept that the runs which started before the change carry the old value armed and the new one plain.

Nothing on this page is a promise about a way through that is not on it. Masking is one boundary, and the review
that follows this package keeps finding roads; the owner's decision is to keep the masking narrow, accept the
risk, and rotate.

## Fix

`README.md` under "What is masked before it reaches the model". The values are read once in the driver,
travel on a descriptor the driver opens for Pi, and are replaced in each tool result and again in the whole
transcript by a marker that names where the value came from.
