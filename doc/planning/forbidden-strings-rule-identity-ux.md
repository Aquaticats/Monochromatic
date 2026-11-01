# Forbidden-strings rule-identity UX

Status:
RESOLVED 2026-07-20 by maintainer instruction
("migrating the rules so every rule can have a sanitized name that
forbidden-string rule match outputs can refer to";
local rules hide even their subject area).
Outcome:
findings render named rules as `rule=<name>` (tail-format section names;
baseline names are betterleaks ids embedded at build time as a name
sidecar),
unnamed legacy rules keep the offset numeric fallback,
cross-set name collisions fail the load closed,
and local-appendix rules take opaque sequential `local-NNN` names.
The namespacing (`builtin:`/`local:`) and hash options below were not
adopted;
names alone kill the index drift.
The analysis stays for the record.

Context:
the forbidden-strings engine migration
(`doc/planning/forbidden-strings-engine-migration.md`)
shipped 0.2.0 with redacted findings `PATH:LINE rule=N`.
This doc captures the follow-up question,
"how does one know which rule triggered?",
and the full design space for answering it,
so the reasoning survives a context compaction.

## The problem

A finding is `PATH:LINE rule=N`,
where `N` is the compiled index of the matched rule,
and the rule text is never printed.
That is the redaction invariant:
a sensitive rule's pattern must never reach a shared log such as CI output.
There is no lookup affordance in the CLI,
so a blocked developer cannot tell what `rule=N` is
without opening the rule file and counting non-comment lines.

Worse,
`N` is not stable.
Rules load into a single index space:
the runtime rules (the local and shared appendix files) take indices `0..k`,
and the embedded builtin baseline is offset above them at `k..`.
So the same baseline rule has a different `N`
depending on how many local rules loaded.
Measured example:
an AWS access-key baseline rule is `rule=20` against the embedded baseline alone,
but `rule=58` in the commit-gate configuration,
because the roughly 38 loaded local rules push every baseline index up by 38
(`20 + 38 = 58`).
The number is not even writable down across environments.

## What is actually sensitive

The redaction invariant only constrains the sensitive local rules:
codenames,
 customer or partner identifiers,
and politically charged literals in the gitignored appendix.
The 259 builtin baseline rules are public:
they are generated into the checked-in
`package/forbidden-strings/cli/data/builtin-rules.txt`.
So there is real room to make baseline findings self-describing
without disclosing anything,
while keeping local findings opaque.

## Key constraint: the baseline cannot identify itself at runtime

The binary embeds only the compiled baseline automaton
(the `to_bytes` blob loaded through `from_bytes`),
and the port stripped every betterleaks description.
So given a baseline match at index `i`,
the running binary has no name,
 no description,
and (unless the engine's serialization retains the source strings,
which is worth confirming) no pattern text to report.
That information was discarded at build time.

Local rules are the exception:
the scanner reads their text at scan time,
 so it still holds them.

Consequence:
any scheme that resolves,
 names,
 or hashes a baseline rule
needs an identity artifact embedded at build time,
 aligned to the compiled set.
The `build.rs` step already parses `builtin-rules.txt`,
so emitting an aligned sidecar
(source patterns,
 per-rule hashes,
 or names)
next to the `.bin` blob is a small addition.
Local-rule resolution needs no embed.

## The hash idea, assessed

A hash of the rule text is a legitimate stable identifier
and has one edge an index lacks:
it is stable across baseline version changes,
since the same rule text hashes the same regardless of position.
That would matter if a suppression or allowlist mechanism keyed by rule id
is ever added (none exists today).

Two caveats temper it.
First,
 a plaintext hash of a low-entropy sensitive local rule is a membership oracle:
anyone who reads a CI log can hash a guess
and confirm whether that term is on the deny-list.
Closing that needs a keyed hash
(an HMAC under a per-repo secret the developer and CI hold but outsiders do not),
which adds a key to manage.
Second,
 a baseline hash still has to be precomputed and embedded,
so it does not dodge the identity constraint above.

Because the baseline is public and its names are public,
embedding a name (`aws-access-key`) buys more than a hash for the same embed,
and for a sensitive local rule a per-file index leaks nothing
and is developer-resolvable.
So the hash's niche here is narrow:
reach for it only if cross-version-stable ids are wanted.

## Options

### Option A: source-tagged ids plus a lookup command

Findings become `rule=builtin:N` and `rule=local:N`.
Each `N` is an index within its own source,
so it stops drifting when the other source's count changes,
and the prefix names the file to consult.
A new `explain <id>` subcommand prints the rule
from the same sources the scan loaded:
baseline freely,
local rules behind an explicit `--include-local`
so default `explain` output stays CI-safe.

Pros:
fixes both opacity and the cross-environment drift;
baseline ids become directly lookupable.
Cons:
one lockstep git-policy parser update (as in #388),
plus the new subcommand and the baseline identity embed.

### Option B: lookup command only

Keep `rule=N` unchanged;
add `explain N` reading the same loaded sources.
Pros:
no format change,
 covers both sources.
Cons:
the number stays opaque until the command runs,
and it still drifts across environments.

### Option C: category names on baseline findings

Baseline findings gain a safe label,
`rule=147 (aws-access-key)`.
Pros:
best at-a-glance UX for the common credential catch,
zero pattern disclosure.
Cons:
needs a names sidecar from the betterleaks porter,
covers only the baseline,
and is still a format and parser change.

### Option D: document the manual lookup

No code change;
document that a developer maps `rule=N`
by counting non-comment lines and subtracting the local-rule offset.
Pros:
zero work,
 redaction stays pristine.
Cons:
the gap and the drift both remain.

## Recommendation

Combine the stable namespacing of Option A with the naming of Option C.

- Namespace findings as `rule=builtin:N` / `rule=local:N`.
  This alone kills the drift and disambiguates the source.
  It is the piece the git-policy scanner-output parser must learn,
  one more lockstep update.
- Embed baseline names at build time
  so a `builtin:N` finding can render `(aws-access-key)`
  and be resolved straight from an installed binary.
  The betterleaks porter that generates the baseline has the descriptions
  and can emit an aligned names file.
- Keep `local:N` a bare per-file index:
  zero disclosure,
  resolved from the developer's own short appendix,
  or through `explain local:N --include-local` locally.
- `explain` is then thin:
  baseline reads the embedded name,
  local reads the loaded file behind the flag.

Hashing stays available for cross-version-stable ids but is not the lead:
names dominate it for the public baseline,
and an index dominates it for the sensitive local rules.

## Open forks awaiting the maintainer

- Namespaced ids yes or no.
- For the baseline:
  names,
   hashes,
   or full source in the embedded sidecar.
- For local rules:
  a plain per-file index,
   or a keyed hash.

## Relationship to the word-boundary fix and the 0.2.1 release

The short-literal word-boundary change (commit `296c5169c`) is committed but dormant:
the local gate and CI still run 0.2.0.
Activating it in CI needs a 0.2.1 release,
because CI downloads the version-matched release binary.
Whatever rule-identity option lands should be cut in the same 0.2.1 release
so there is one publish,
 not two.
That release will also turn the index.html full-tree finding green,
since that finding is the same short-literal base64 collision
the word-boundary gating resolves.
