# Markdown-route findings on `main`

## Audit scope at 2026-11-09T06:56:11-05:00

Audited revision:
`main` at `baf3d03fe668432f127e024275edeb3b9a668281`.
The audit branch started at that exact revision.
Later movement of `origin/main` and other agents' unmerged branches is not part of this audit.
Linux only.
No production code was changed.
Temporary controls are not committed.

Scope:
`package/git-policy/cli/src/native/markdown/`,
plus the candidate,
commit-convergence,
fix-installation and embedded-linter boundaries this route actually calls.
Unless a citation names another package,
source paths in findings are relative to `package/git-policy/cli/src/native/`.

The model was [Copy-route findings on `main`](cli-git-copy-route-findings-on-main.md).
The handover's `Open review findings` section was read at the audited revision.
It does not record the findings on this page.
The [commit-checks audit](cli-git-commit-checks-route-findings-on-main.md) recorded older direct-fix races and mode loss.
This revision already uses exchange-based installation and preserves modes and owners.
Those older findings are not repeated here.
The extended-attribute loss is a separate,
remaining loss in that shared installer.

## Purpose, callers and the actual linter boundary

The optional `markdown/autofix` policy checks Markdown and MDX candidates for `lfs-image-url`.
Its purpose is to replace relative image links to LFS-managed images with immutable LFS object URLs,
so readers see images rather than LFS pointers.
The policy defaults to `warn` when configured;
without an entry in repository policy configuration it is off
(`policy/policy_registry.rs:189` to `:195`).

`policy/policy_checks.rs:195` constructs the embedded linter,
and `:518` dispatches the policy to `check_markdown`.
`policy/policy_trigger.rs:130` enables the policy at every lifecycle trigger,
including pre-forward,
post-commit,
manual push,
direct check and direct fix.
Candidates and their Markdown bytes come from `ContentState`,
not from an independent worktree read by the linter
(`markdown/policy_markdown.rs:450` and `:468`).

The question's outside-program premise does not describe this revision.
`markdown/markdown_linter.rs:42` imports the linked library's `fix_source`.
Its `lint` method constructs an `EmbeddedRequest` and runs it on a guarded thread
(`:258` to `:291`).
There is no linter executable lookup,
child exit status,
stdout parser or JSON Lines parser at this boundary.
The status in `EmbeddedFailure` is a typed library result.
The generated linter configuration is also supplied in memory.

A changed linter result becomes a finding at every trigger,
but a correction is proposed only for `DirectFix`
(`markdown/policy_markdown.rs:401` to `:420`).
This matters to the interpretation of every finding:
wrong LFS context can produce a false commit-time warning or rejection,
but this route writes the corresponding wrong Markdown text only on direct `fix`.
It does not silently replace staged Markdown with unstaged prose during a commit.
The mismatch between that behavior and the printed promise of commit-time correction is a separate finding.

## Owner requirements and decisions touched

The audit brief supplies the owner's requirements verbatim:

- "Commits should never fail."
- "never discard user data"
- "nothing should ever wait forever or indefinitely"

Normal conditions include "a process killed at any instruction,
a closed terminal or broken pipe,
slow fsync,
unusual filesystems,
DNS or mirror trouble",
as quoted from `AGENTS.md` in the brief.

The matching decision search was `rg` over `doc/decision/` for Markdown,
LFS,
the Git wrapper,
configuration and concurrent commits.
Relevant recorded decisions:

- [Rewrite cli-git in Rust with shipped policies](../decision/cli-git-rust-rewrite.md),
  lines 3 to 9:
  "The user selected a Rust cli-git replacement,
  JSONC configuration through the repository's existing Rust package,
  forbidden-strings bundled into cli-git,
  shipped policies only,
  and removal of executable-configuration trust."
  It also says:
  "Repository configuration may disable or relax policies directly."
  Findings about exclusions concern the reading of that data configuration,
  not a proposal to load executable configuration again.
- The same decision,
  lines 12 to 17:
  "The same implementing agent owns the unified Rust/Markdown/MDX linter."
  "The user subsequently authorized simple handwritten configuration merging inside the linter for now".
  The LFS-context findings cross the linked linter boundary;
  their suggested repairs belong in that owner rather than a second wrapper implementation.
- [cli-git concurrent commits](../decision/cli-git-concurrent-commits.md),
  `Parallel preparation`:
  "Explicit-path commits capture selected worktree bytes at invocation;
  index commits capture the real index at invocation."
  The staged-versus-worktree controls check this boundary.
- The handover's `Owner decisions in force`,
  `Answers after the restart (2026-11-04)`,
  records the owner's choice for non-Git tools:
  "Silence plus a ceiling (Recommended)".
  It names the Markdown linter among those tools.
  The subsequent choice is "5x (Recommended)",
  five times the longest measured run,
  rounded up to whole minutes.
  `service/wait_ceilings.rs:484` supplies the resulting four-minute `LINT_CEILING`.
  The blocked-notice finding is a wait outside the guard's enforcement of that ceiling.

`SPEC.md:5365` explicitly documents that a fix replacement does not carry extended attributes or its access-control list.
That is a documented implementation limit,
not an unmentioned regression.
No owner exception to "never discard user data" for those attributes was found in the matching decisions or handover.
The saved-conflict-copy pruning exception in the handover concerns a different store.

## Findings ranked by risk

The order prioritizes permanent user-data loss,
then a command with no enforced return bound,
then exclusions that silently skip protection or stop commands,
then incorrect LFS rewrites,
and finally a false corrective instruction.
The blocked-notice case has more preconditions than the brace mismatch,
but ranks ahead of it because its bad result has no return bound;
the brace compiler at least returns a named error.

### A successful Markdown fix deletes extended-attribute data

Status:
**reproduced through the built CLI**.
Risk:
persistent loss of data stored outside the file's ordinary byte stream.
This is the shared direct-fix installer reached by Markdown,
not a Markdown parser defect.

Locations:

- `policy/direct_fix_install_writes.rs:108` to `:128` creates the replacement,
  writes ordinary bytes and restores only mode and owner.
- `policy/direct_fix_install_judging.rs:82` to `:87` describes a file using ordinary bytes,
  mode and owner.
- `policy/direct_fix_install.rs:310` to `:319` removes the displaced original when that comparison succeeds.
- `SPEC.md:5365` acknowledges the attributes the replacement does not carry.

Sequence:

1.  In a disposable repository,
    configure Markdown autofix,
    an LFS endpoint and `*.png filter=lfs`.
    Stage `README.md` containing `![image](image.png)`.
2.  Keep `README.md` at mode `0600` and put the only copy of a draft in its `user.draft` attribute.
3.  Run the built wrapper as `git monochromatic-git-policy-cli fix -- README.md`.
4.  The command installs the rewritten Markdown,
    removes the displaced inode and returns a successful fix summary.
    The attribute is absent afterward.

Output from `probe.ts`:

```text
AUDIT xattr before: "only-copy-of-draft"
AUDIT fix: status=0, fix-summary, passes=1, changedPaths=["README.md"]
AUDIT xattr after: status=1, user.draft: No such attribute
AUDIT mode after: 600
AUDIT index unchanged: true
```

The summary is abbreviated from the emitted JSON record;
`getfattr`'s absolute fixture pathname is omitted.
The value was successfully stored and read before the fix,
so absence is not a filesystem-support guess.
No index or commit holds that attribute value,
and the successful fix left no recovery notice for it.

Wrong result:
a text correction silently discards separately stored user data.
An access-control-list loss could also change who can access a file,
but that consequence was not reproduced.

Suggested fix,
not implemented:
carry extended attributes and ACLs with the replacement and include them in ownership-of-state checks,
or refuse the correction without deleting the attributed original when preservation cannot be proved.
Keep this responsibility in the shared installer.

### A blocked progress notice defeats the linter's ceiling

Status:
**reproduced through the existing notice-writer seam**.
Risk:
the person's command can wait without a bound outside the loop that is supposed to end the wait.

Locations:

- `markdown/markdown_linter_guard.rs:201` to `:204` prints a due notice before checking the ceiling at `:219`.
- `push/wait_progress.rs:363` to `:366` performs synchronous `writeln!` on the notice sink.
- `markdown/markdown_linter.rs:146` supplies the wrapper's standard error as the production sink.
- `push/wait_progress.rs:62` supplies the production notice grace of one second.

Sequence:

1.  Supply a notice sink whose write is blocked,
    the state a full still-open stderr pipe can have while its reader stops draining it.
2.  Start `run_guarded` with a completed job,
    an immediately due notice and a `50 ms` ceiling.
3.  Wait for the seam to confirm that the caller entered the notice write.
    Keep that write blocked for `500 ms`.
4.  The guard returns nothing during those ten ceiling intervals.
    Release the write;
    only then does the guard notice the elapsed ceiling and return.

The control bounds every channel wait,
releases the sink before asserting,
and leaves no stalled job or notice thread.
Output:

```text
AUDIT guard with blocked notice after ten ceilings: Err(Timeout)
AUDIT guard after notice release: Err(Ceiling(50ms))
a notice write must not defeat the guard ceiling
```

Wrong result:
even a completed linter cannot be returned to its caller while an advisory write blocks.
The production write has no corresponding release deadline.
In production the lint must first last long enough for a notice to be due,
and stderr must be a captured or piped stream whose still-open reader has stopped draining it.
The seam's zero grace reaches that same write deterministically,
without manufacturing a slow lint or depending on scheduling a pipe-filling race.
The seam demonstrates the enforcement gap;
an actual full operating-system stderr pipe was not used in this reproduction.
A closed pipe returning an error is a different case and is ignored as intended.
This also affects other callers of the shared notice writer,
but the sequence here uses the Markdown guard itself.

Suggested fix,
not implemented:
make advisory output nonblocking or separately bounded,
without making the guard wait for a notice writer before checking its deadlines or receiving a finished result.
Do not add a second unbounded writer queue.

### Gitignore braces become alternate-group syntax, excluding the wrong file or failing the policy

Status:
**reproduced against native Git 2.56.0 and the built CLI**.
Risk:
a valid exclusion is not honored,
an unrelated file is silently unchecked,
or a warn-level policy fails with exit `2`.

Locations:

- `markdown/markdown_exclude.rs:78` to `:96` passes each claimed gitignore pattern to the default matcher builder.
- `markdown/policy_markdown.rs:463` to `:467` turns a compile failure into `policy-incomplete`.
- `policy/policy_engine.rs:414` to `:430` turns that outcome into a failed pass regardless of warning severity.

Sequence:

1.  Give native Git a `.gitignore` containing the literal pattern `notes{old,new}.md`.
    Ask `git check-ignore --no-index --quiet -- <name>` for both that literal filename and `notesold.md`.
2.  Remove the temporary `.gitignore`,
    so it cannot hide candidates from the policy.
    Configure the same pattern as `markdown/autofix.exclude`.
3.  Check each explicit Markdown file with the built CLI.
    Each contains a link that the no-exclusion positive controls detect as needing a rewrite.
4.  Git ignores the literal brace name,
    but the policy checks it.
    Git does not ignore `notesold.md`,
    but the policy silently excludes it.
5.  Repeat with the valid literal Git pattern `a{b.md`.
    Git ignores the matching literal filename;
    the policy refuses the pattern as an unclosed alternate group and exits `2` even at severity `warn`.

Output,
projected from `differential.ts`'s JSON records:

```text
pattern=notes{old,new}.md name=notes{old,new}.md gitIgnored=true  policyStatus=0 rewrite=true
pattern=notes{old,new}.md name=notesold.md       gitIgnored=false policyStatus=0 rewrite=false
pattern=a{b.md name=a{b.md gitIgnored=true policyStatus=2
error parsing glob 'a{b.md': unclosed alternate group; missing '}' (maybe escape '{' with '[{]'?)
```

The no-warning result for `notesold.md` is not a broken generator:
`keep.md` re-included by negation and `doc/README.md` outside an anchored exclusion both produce rewrite findings.
Of the nine pattern/path comparisons,
three disagree with native Git,
all in the brace cases.
The complete finite shape list is in the verification record.

Wrong result:
data documented as gitignore syntax crosses into a richer glob grammar.
A literal character is interpreted as syntax,
not merely printed incorrectly.
The direct-check failure is reproduced;
the same failed-pass consequence on a wrapped commit is read from the shared engine,
not claimed as an end-to-end service-backed commit reproduction.
The nonblocking-engine-failure comment at `policy/policy_engine.rs:375` concerns `PolicyOutcome::Skipped`:
that arm continues at `:412`.
This route returns `PolicyOutcome::Failed`,
whose arm sets `StageEnd::Failed` at `:430`;
`pass_exit_code` returns the engine-failure exit code at `:499`.
`landing/service_prepare.rs:331` to `:334` ends the prepared commit on that nonzero code.

Suggested fix,
not implemented:
configure or adapt the existing matcher to Git's actual pattern grammar,
including literal braces,
and keep a differential control against `git check-ignore`.
Correct the linked linter's matching of the duplicated exclude option in the same owning change.
Do not teach the configuration reader a separate pattern language.

### A linked worktree inherits an enclosing checkout's LFS rules

Status:
**reproduced through both the embedded-linter seam and a real linked-worktree CLI fix**.
Risk:
Markdown is rewritten to an endpoint and repository-relative object path belonging to another checkout.

Locations:

- `markdown/markdown_linter.rs:269` to `:286` passes the correct worktree top level,
  but no LFS-context boundary.
- `package/linter/monochromatic-lint/src/markdown_lfs_context.rs:165` to `:178` searches all ancestors for `.lfsconfig`.
- The same file at `:201` to `:211` reads `.gitattributes` from whichever directory that search chose.

Sequence:

1.  Create a parent Git repository and commit only `base.txt`.
    Leave an LFS endpoint and `*.png filter=lfs` untracked in that parent checkout.
2.  Use native `git worktree add --detach nested HEAD` to make a real linked worktree beneath the parent.
    The linked worktree contains neither `.lfsconfig` nor `.gitattributes`.
3.  In `nested`,
    configure the optional policy and create `image.png` plus `README.md` with `![image](image.png)`.
4.  Native `git check-attr filter -- image.png` in `nested` reports `unspecified`.
5.  Run the built CLI's direct fix inside `nested`.
    It searches beyond `nested`,
    selects the parent's endpoint and attributes,
    and successfully replaces the relative image link with a URL whose path includes `nested/image.png`.

Output from the real linked-worktree reproduction:

```text
AUDIT linked worktree has lfs config: false
AUDIT linked worktree Git attributes: image.png: filter: unspecified
AUDIT linked worktree fix: status=0, fix-summary, passes=1, changedPaths=["README.md"]
```

The resulting Markdown is:

```markdown
<!-- README.md in the disposable linked worktree -->
![image](https://lfs.example.invalid/org/repo/a4b9deeaf28ef402473e06fb5a1ef4cbb951f3e9a20f505565579b40b49ccc3e/nested/image.png)
```

The unit seam independently failed with:

```text
assertion `left == right` failed: a different checkout's endpoint must not govern this candidate
```

Wrong result:
the supplied worktree root does not delimit the configuration used to rewrite its text.
The parent checkout can change the result without changing any candidate in the linked worktree.
A nested linter configuration file is not the cause:
this is LFS-context discovery,
not discovery of `monochromatic-lint.config.jsonc`.

Suggested fix,
not implemented:
let the embedded linter consume an explicitly bounded repository context from the caller,
and stop LFS discovery at that worktree root.
Fix the linter's context owner rather than duplicating its endpoint parser in the wrapper.

### A later non-LFS filter assignment is ignored when choosing image rewrites

Status:
**reproduced against native Git 2.56.0 through the embedded-linter seam**.
Risk:
`fix` can replace an ordinary image link with an LFS object URL for an image Git does not manage with LFS.

Locations:

- `markdown/markdown_linter.rs:278` to `:285` delegates the policy's candidate to the embedded engine.
- `package/linter/monochromatic-lint/src/markdown_lfs_patterns.rs:177` to `:195` recognizes `filter=lfs`,
  `-filter` and `!filter`,
  but omits an ordinary overriding `filter=other` assignment.

Sequence:

1.  Create a repository with an endpoint,
    an image and Markdown linking that image.
2.  Write the root `.gitattributes` with `*.png filter=lfs`,
    followed by `*.png filter=other`.
3.  Ask native Git for the image's effective filter.
    It returns `other`.
4.  Give the actual embedded linter the same repository root and candidate Markdown.
    It returns an LFS URL rewrite anyway.

Output:

```text
AUDIT Git attribute: pkg/asset/shot.png: filter: other
AUDIT filter override fixed: ![shot](https://lfs.example.com/org/repo/de7030234493a8bea844dbe1d8676e68a2c1a4b014c721f0425a22b6df66faec/pkg/asset/shot.png)
assertion `left == right` failed: Git says the image is not LFS-tracked
```

The existing `the_linked_linter_meets_the_contract` control passed in the same focused run.
It verifies that the ordinary `filter=lfs` case really rewrites,
so the override comparison is not vacuous.

Wrong result:
the linter's LFS classification disagrees with Git's effective attribute assignment.
At `warn` this is a false finding during a commit;
at `error` it can reject the commit;
a direct fix proposes the wrong text.
No LFS server request or missing remote object was measured,
so this finding does not claim an observed HTTP failure.

Suggested fix,
not implemented:
resolve effective filter values with Git-compatible assignment semantics in the linked linter,
including replacement by another value,
and compare representative attribute shapes against `git check-attr`.

### The commit promises a Markdown correction that its pass never proposes

Status:
**reproduced at the convergence seam with native Git**.
Wrapped-commit wiring was read from code only.
Risk:
a person trusts the printed instruction and commits the original image link instead of the promised correction.

Locations:

- `policy/event_hints.rs:69` says "the commit applies the linter's corrections itself".
- `markdown/policy_markdown.rs:401` permits a correction only when the trigger is `DirectFix`.
- `commit/commit_convergence.rs:260` enables patches for its pre-forward pass,
  but the Markdown policy does not consult that capability.
- `README.md:2268` describes running the fix engine over staged Markdown inside the commit.
- The repository's `monochromatic-git-policy-cli.config.jsonc:6` to `:8` says image links are rewritten
  "inside the commit transaction".

Sequence:

1.  Stage Markdown with a relative link to an LFS image.
2.  `check` reports `markdown-autofix` and prints the hint promising correction in the commit.
3.  Commit the staged Markdown without running direct `fix` first.
4.  The pre-forward Markdown pass reports the rewrite but proposes no correction,
    because it is not `DirectFix`.
    Its candidate retains the original relative link.

The control ran the real patch-enabled `converge_commit_policies` over a disposable private index,
with different staged and working-file text,
then let native Git commit exactly that private index.
Output:

```text
AUDIT Markdown commit convergence: Settled { changed_passes: 0 }; paths=[]
pre-forward finding: markdown-autofix, severity=Warn, fix_available=false
AUDIT Markdown committed blob: "staged prose\n![image](image.png)\n"
AUDIT Markdown worktree: "unstaged prose\n"
the advertised commit-time Markdown correction was not staged
```

The finding line is projected from the printed typed event.
This exercises the real commit policy-pass implementation,
not the complete wrapped-commit invocation,
service-backed landing or automatic publication.
The production wiring was read separately:
`commit/commit_command.rs:207` to `:216` constructs a `PreForward` request and passes it to `service_commit`;
`landing/service_prepare.rs:319` to `:329` runs that request through either a read-only pass or convergence.
The reproduction specifically chooses the patch-enabled arm,
so a deliberately read-only explicit-path selection cannot explain the missing correction.

Wrong result:
the instruction tells the person an operation will happen automatically,
while the implementing policy explicitly excludes that operation at the commit trigger.
This is not evidence that unstaged prose enters a commit:
the original staged bytes are precisely the bytes left unchanged.

Suggested fix,
not implemented:
reconcile the policy with its documented commit-time contract,
using the existing patch capability when correction is authorized,
and keep the hint,
repository configuration comment and consumer docs truthful about the resulting behavior.
A change to the intended contract needs an owner decision,
not an audit-time implementation.

## Verification record

Scratch consumers and fixtures stay inside this worktree under
`package/git-policy/cli/target/markdown-route-audit/`.
The native-test fixture root is
`package/git-policy/cli/target/markdown-audit-fixtures/`.
Neither is committed.
Copies of the temporary test sidecars,
with the audit controls included,
are kept there as `markdown-linter-tests-with-audit-controls.rs`,
`markdown-linter-guard-tests-with-audit-control.rs` and `commit-convergence-tests-with-audit-control.rs`.
The tracked sidecars were returned to their exact starting contents before report linting.
No installed wrapper or running installed service was used for the disposable-repository probes.
The TypeScript consumer harnesses resolve plain `git` with the fixture's `PATH=/usr/bin:/bin`,
without real home,
system Git configuration or global Git configuration.
Rust convergence controls use the crate's existing native-Git fixture helpers.

Commands and results:

- `mise run //package/git-policy/cli:native:build`:
  passed.
  This built the executable from the audited production sources.
- `mise run //package/git-policy/cli:native:test:host -- --success-output=final --failure-output=final 'markdown_linter::tests::audit_'`:
  the ancestor-context and attribute-override controls both failed at their expected assertions.
- `mise run //package/git-policy/cli:native:test:host -- --no-fail-fast --success-output=final --failure-output=final --filter-expr 'test(markdown::) | test(policy::direct_fix_install::tests::)'`:
  69 tests ran,
  66 passed and the three temporary defect controls failed.
  Every incumbent Markdown and selected installer test passed.
- `mise run //package/git-policy/cli:native:test:host -- --success-output=final --failure-output=final audit_markdown_rewrites_never_reach_the_commit_index`:
  the temporary commit-convergence control failed at its expected missing-correction assertion.
  Native Git committed the unchanged staged Markdown,
  not the different working-file prose.
- `node package/git-policy/cli/target/markdown-route-audit/probe.ts`:
  passed as a measurement harness,
  reproducing attribute loss and checking the ordinary file,
  index and committed blob independently.
- `node package/git-policy/cli/target/markdown-route-audit/differential.ts`:
  passed as a measurement harness.
  Native Git was `2.56.0`.
  Three of nine exclusion-pattern comparisons disagreed,
  and the real linked-worktree fix used its parent's LFS context.
- `mise run //package/git-policy/cli:lint`:
  passed after all temporary tracked test edits were removed.
  The audit made no production change to get this result.
- `mise run //:format:monochromatic-lint -- doc/planning/cli-git-markdown-route-findings-on-main.md`:
  passed.
- `mise run //:lint:monochromatic-lint -- doc/planning/cli-git-markdown-route-findings-on-main.md`:
  passed.
- `node package/git-policy/cli/target/markdown-route-audit/render-report.ts`:
  passed.
  Pandoc rendered the report as GitHub-flavored Markdown to HTML;
  its parsed rendering contained the expected evidence labels and code blocks,
  with no italic spans,
  tables or blockquotes.

Cross-vendor reviews by DeepSeek and Kimi checked the evidence and its limits.
The report distinguishes a blocked-writer seam from an actual full stderr pipe,
a convergence-seam result from a complete wrapped commit,
and a documented metadata omission from an owner-approved exception.
Those review clarifications did not authorize implementation changes.

The finite exclusion inputs were wildcard matching,
negation,
a negated child of an excluded directory,
root anchoring,
unclosed literal braces,
closed literal braces,
the unintended alternate-group match,
an escaped leading exclamation mark and an escaped space.
Each fixture removes the oracle's `.gitignore` before asking the wrapper,
so native Git's ignore handling cannot hide a policy candidate and fake agreement.

Build and native-test commands used individual scopes with CPU quota `600%`,
`MemoryMax=12G`,
`TasksMax=512` and `IOWeight=10`,
with a `600 s` outside deadline.
Consumer harnesses used CPU quota `200%`,
`MemoryMax=2G`,
`TasksMax=128` and disk weight `10`,
with outside deadlines of `120 s` or `180 s` and a `30 s` deadline per subprocess.
The first focused-test launcher was stopped before its results were used,
then relaunched with the fixture root explicitly inside the worktree.
No mutation campaign or full crate suite was run.

## What could not be determined

- The real-world frequency of any finding.
  These are deterministic shapes,
  not a production-incident census.
- The ACL or security-label consequence of replacement.
  Only loss of a real `user.*` extended attribute was exercised.
- A full operating-system stderr pipe blocking a service-backed commit.
  The deadline bypass was reproduced through the existing synchronous writer seam;
  the production pipe case is its code-backed application.
- The outcome of an abort,
  stack overflow or out-of-memory termination inside the linked linter.
  The source and `SPEC.md:145` explicitly acknowledge that such failures terminate the wrapper.
  This audit did not manufacture memory pressure or synthetic host load to force them.
- A hard memory bound during linting.
  The `64 MiB` fixed-source and `4 MiB` finding limits are checked after the result is allocated
  (`markdown/markdown_linter_output.rs:229` to `:240`),
  not enforced while the embedded engine is running.
- Exhaustive Git attribute semantics,
  arbitrary repositories,
  network filesystems,
  power-loss durability or every possible ancestor-symlink interleaving.
  The attribute differential establishes the named override defect,
  not full attribute-reader equivalence.
- Windows and macOS behavior,
  which the owner's pause excludes.

## What was checked and found sound

- Markdown candidate bytes do not come from a second read of the Markdown working file.
  The index-versus-worktree consumer kept the real index byte-for-byte unchanged through direct fix.
  Native Git then committed the original staged Markdown,
  while the working file held the rewritten link.
  There was no unstaged prose substitution in that comparison.
- The selected installer controls passed for a write after preparation,
  a permission change,
  a selected symlink,
  an index change,
  a save beside rollback,
  open-descriptor writes caught before final cleanup,
  interrupted-fix journals and journal paths outside staging reach.
  These results cover the named seams,
  not every possible race or late descriptor write.
- Existing mode controls passed for private files,
  executable and special mode bits,
  owner and group preservation.
  The CLI consumer independently kept `0600`.
  This does not cancel the extended-attribute finding.
- Deleted files,
  symlink and submodule candidates,
  unsupported extensions,
  excluded paths and non-UTF-8 Markdown contents are skipped by the policy before linting
  (`markdown/policy_markdown.rs:259` to `:266`,
  `:463` to `:475`).
  The mixed-candidate control passed,
  including exact non-UTF-8 pathname bytes and executable Markdown.
- The generated JSONC configuration uses the repository emitter,
  not string concatenation of an exclusion into JSON syntax
  (`markdown/markdown_linter_config.rs:101` to `:117`).
  Its quote,
  backslash and newline controls passed.
  The embedded entry supplies an explicit configuration through `ConfigStore::from_source`,
  so a nested or user linter configuration is not discovered
  (`package/linter/monochromatic-lint/src/run_embedded.rs:190` to `:205`).
  LFS-context discovery remains the distinct finding on this page.
- Candidate names reach the embedded engine as paths,
  not shell commands or executable arguments.
  The exact-byte pathname control passed for spaces,
  a leading dash and a non-UTF-8 byte.
  Findings remain typed until the wrapper's JSON renderer escapes them.
  The syntax-boundary disagreement actually reproduced was the brace grammar in exclusions.
- The incumbent guard controls passed for finished results,
  panics,
  quiet abandonment,
  progress,
  the overall ceiling and refusal to start another linter after abandonment.
  This establishes the guard with a returning notice sink,
  not the blocked-sink case.
- The result-contract controls passed for unknown failure statuses,
  invalid finding codes and severities,
  missing positions,
  oversized result text and findings,
  and an empty replacement for a nonempty source.
  Explanation truncation is by characters,
  not a UTF-8-breaking byte slice
  (`markdown/markdown_linter_output.rs:177` to `:183`).
