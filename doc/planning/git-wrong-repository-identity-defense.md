# Wrong-repository Git identity defense plan

## Status

Plan complete on 2026-07-09;
implementation has not started.

The failure has been diagnosed and future identity restored.
The architecture has incorporated independent advisor reviews.
This plan records the defense-in-depth work that remains.
No prevention code or remote protection change has been implemented yet.

Issue `#357` retirement amendment:
hk and Pkl are removed and must not be reintroduced by this plan.
Every hk adapter,
`HK=0` case,
Pkl snippet,
and config-hook integrity requirement in the original design is superseded.
Future implementation uses cli-git's pre-forward,
post-commit,
and Git-native manual-push lifecycle plus authoritative server rules.

## Goal

Make the July 9 failure stop at several independent boundaries before it can alter or publish repository history.

The failure chain was:

1. Pi 0.80.6 accepted an unsupported `cwd` property in a Bash tool call.
2. The Bash implementation ignored that property and ran the command from the session directory.
3. Fixture `git init` and `git config user.*` commands ran in the main Monochromatic repository.
4. Repository-local fixture identity overrode the intended global identity.
5. Git continued to create cryptographically signed commits under `fixture@example.invalid`.
6. GitHub classified those signatures as `no_user` because the commit identity was not associated with a user.
7. Classic branch protection required verified signatures but exempted administrators,
    so the owner could publish them.

The design must enforce these invariants:

- Unsupported Bash tool properties fail before shell execution.
- Reinitializing an existing repository fails before Git changes state.
- Automated Git invocations cannot write protected identity or signing configuration.
- A contaminated commit environment fails before commit.
- Any contaminated or unverifiable commit fails before push.
- GitHub rejects unverifiable commits even when the pusher administers the repository.
- Force pushing cannot replace protected history.

The design fixes future operations only.
It does not rewrite earlier commits.

### Threat model and meaning of prevention

The protected failure class is accidental or mechanically misdirected automation running as the repository owner:
ignored tool properties,
wrong working directories,
fixture configuration,
stale local identity,
skipped local hooks,
and ordinary administrator pushes.

The design does not claim to resist a malicious process with the owner's operating-system account
and GitHub credentials.
Such a process can rewrite user-owned trust files and deliberately change remote rules.
Repository administration remains an ultimate authority boundary.

Within this threat model,
"cannot recur" means no unsupported `cwd` call can execute,
no fixture identity can create or publish a new commit through normal tooling,
and bypassing one local layer still meets an independent local or remote rejection.

## Confirmed current state

### Pi execution boundary

Pi 0.80.6 validates known Bash properties but preserves unknown properties.
`dist/core/tools/bash.js` executes only the command and timeout values,
so the unsupported `cwd` property has no effect.

`package/pi-plugin/guardrail/src/index.ts` already owns the deterministic `tool_call` refusal seam.
Its Bash rule in `package/pi-plugin/guardrail/src/bash-guard.ts` currently detects only `bun test`.
The package is documented for global installation,
but `/home/user/.pi/agent/settings.json` does not currently install it.
Project `.pi/settings.json` intentionally has no packages.

`package/pi-plugin/auto-mode` is installed globally,
but it is not an invariant layer:
it reasons about command text and permits explicit bypass.

### Git command boundary

`package/git-policy/cli/src/index.ts` evaluates a pre-spawn rule pipeline before invoking real Git.
That is the deterministic seam for command-level repository safeguards.

`package/git-policy/cli/src/effective-target.ts` already replays global repository-selection options and inherited
`GIT_DIR` or `GIT_WORK_TREE` state through real Git.
The new rules must reuse that seam rather than independently guessing the target repository.

Current gaps include:

- `package/git-policy/cli/src/rules/require-root.ts` exempts `git init`.
- No rule rejects reinitialization of an existing worktree.
- No rule rejects writes to `user.name`,
   `user.email`,
   or signing configuration.
- No rule checks the effective author or committer identity before commit.
- The wrapper auto-pushes a successful `git commit` without first verifying the created commit artifact.

### Cli-git lifecycle boundary

Cli-git now owns pre-forward,
post-commit,
and manual-push policy execution.
Its Git-native manual-push probe obtains one record per pushed ref containing local ref,
local object ID,
remote ref,
and remote object ID,
then validates remote state authoritatively before forwarding.
Identity and signature policy must extend these existing lifecycle seams rather than add another hook runner.

### Identity and server boundary

The effective identity is currently `Aquaticat <an@aquati.cat>`.
The effective signing configuration is global:

- `commit.gpgSign=true`
- `gpg.format=ssh`
- `user.signingKey=/home/user/.ssh/github_sign.pub`
- `gpg.ssh.allowedSignersFile=/home/user/.ssh/allowed_signers`

GitHub classic branch protection for `main` currently has:

- required signatures enabled;
- administrator enforcement disabled;
- force pushes enabled.

The separate `Copilot review` ruleset has no bypass actors,
but it enforces only Copilot review and does not repair the signature-protection gap.

A scan of commits reachable from local refs found these committer identities:

- 4,543 `Aquaticat <an@aquati.cat>` commits;
- 56 `Aquaticat (aider) <an@aquati.cat>` commits;
- 36 `Final newline fixture <fixture@example.invalid>` commits;
- 9 `GitHub <noreply@github.com>` commits.

The two intended human names share `an@aquati.cat`.
The server identity rule therefore needs email semantics,
while the local policy can enforce exact name-and-email pairs.
The inventory must be refreshed before rollout because branch creators and web-generated commits can change.

## Chosen architecture

Use a deep repository-safety module with thin adapters at Pi,
Git invocation,
Git hook,
and GitHub boundaries.

### Module interface

The implementation home is `package/git-policy/cli/src/repository-safety/`.
It exposes three concepts:

```ts
assertSafeGitInvocation({ args, cwd, env })
assertSafeCommitEnvironment({ cwd, env })
verifyCommits({ cwd, source })
```

`source` is a tagged union for explicit object IDs,
a successful wrapped commit,
or native pre-push input.
This keeps commit verification in one module while adapters own lifecycle-specific input.

The interface stays small while hiding:

- Git global-option and subcommand parsing;
- effective target-directory resolution;
- repository and worktree discovery;
- Git configuration scope and origin parsing;
- trusted committer lookup;
- allowed-signers parsing;
- push-input parsing;
- revision-range expansion and deduplication;
- cryptographic commit verification;
- diagnostic formatting.

Implement identity checks as cli-git policy and lifecycle modules bundled into the existing single `index.mjs` artifact.
Pre-forward checks cover commit-producing commands;
post-commit checks gate automatic push;
manual-push checks consume cli-git's Git-native authoritative push updates and candidate ranges.
All internal Git subprocesses reuse `package/git-policy/cli/src/resolve-git.ts` and a sanitized environment,
so verification cannot recurse into cli-git.

A linked-worktree
and clean-clone fixture must prove the built shadow `git` resolves from the active package installation
without falling back to a main-checkout artifact or another worktree.

### Trusted identity source

Do not hardcode `Aquaticat` in TypeScript source.
Provision exact local committer identities in a dedicated user policy file,
for example:

```text
~/.config/monochromatic/git-trusted-committers.json
```

The file is a regular,
non-symlink file owned by the current user with mode `0600`
under a directory that is not group-writable or world-writable.
The production path is not overridable by process environment.
Tests inject a fixture path through the TypeScript dependency seam,
not a production environment variable.

The initial policy contains the currently effective identity,
`Aquaticat <an@aquati.cat>`.
Any additional exact name-and-email pair is an explicit onboarding change.
Matching is byte-exact after parsing Git's canonical ident output;
email comparison for allowed-signers principals is case-sensitive because the trust file is explicit policy,
not mailbox normalization.

The configured SSH allowed-signers file remains the trust source for principal-to-key verification.
Require the same ownership,
regular-file,
non-symlink,
and mode checks as the committer policy.
The committer email must appear as a principal in that file,
and allowed-signers validity windows or namespace restrictions must remain effective through Git's verifier.
This supports key rotation and multiple trusted identities without source edits.

The process running local checks can still rewrite user-owned files.
These files are operational safety roots,
not adversarial security roots.
The independent GitHub ruleset supplies the remote email and signature boundary.

The author may differ from the committer for imported or cherry-picked work.
The policy therefore requires the committer,
not every historical author,
to match the trusted local identity.
Explicit `--author` metadata is permitted and tested.
GitHub's vigilant-mode documentation likewise distinguishes author from committer when evaluating signed commits.

## Prevention layers

### Layer A: fail closed on unsupported Bash input

Extend `package/pi-plugin/guardrail` so Bash tool-call input is shape-checked before any command policy.

Accepted top-level properties for the current Pi Bash tool are:

- `command`
- optional `timeout`

Any other property blocks the call.
A `cwd` rejection explains that Pi 0.80.6 does not support the property and requires either:

- `cd -- <absolute-target> && <command>`;
   or
- a command-native directory argument such as `git -C <absolute-target>`.

The strict local copy is intentionally fail-closed.
If a future Pi release adds a real Bash property,
the guardrail must be updated and verified before that property is accepted locally.

After building and testing the package,
install `package/pi-plugin/guardrail` in `/home/user/.pi/agent/settings.json` as documented in its README.
Keep project `.pi/settings.json` package-free.

### Layer B: reject dangerous Git invocations before spawn

Add non-bypassable rules to the existing `package/git-policy/cli` rule pipeline.

#### Existing-repository initialization

Reject `git init` when its effective target is:

- an existing repository or worktree root;
   or
- nested inside an existing worktree.

Intentional nested repositories are forbidden by this policy.
Use `git submodule add` for a tracked nested repository and `git worktree add` for another worktree.

Reuse `classifyEffectiveTarget` and extend init-specific target parsing to cover:

- chained global `-C` options;
- `--git-dir` and `--work-tree`;
- inherited `GIT_DIR` and `GIT_WORK_TREE`;
- the optional `git init <directory>` operand;
- `--bare` and `--separate-git-dir`;
- relative and absolute paths;
- linked worktrees;
- symlinks;
- nonexistent targets resolved through their nearest existing parent.

Allow initialization only outside an existing repository.
Fixture setup remains supported by creating a disposable directory first and invoking real Git there.

#### Protected configuration writes

Reject writes through the wrapper to these keys at every scope and file target:

- `user.name`
- `user.email`
- `user.signingKey`
- `commit.gpgSign`
- `gpg.format`
- `gpg.ssh.allowedSignersFile`
- `include.path`
- `includeIf.*.path`
- `core.hooksPath`
- `hook.*.command`
- `hook.*.event`

The invocation parser rejects before real Git:

- legacy `git config <key> <value>` mutation;
- `set`,
   `unset`,
   `rename-section`,
   and `remove-section` mutation;
- legacy `--add`,
   `--replace-all`,
   `--unset`,
   `--unset-all`,
   `--rename-section`,
   and `--remove-section` mutation;
- mutation through `--local`,
   `--worktree`,
   `--global`,
   `--system`,
   or `--file`;
- protected `-c key=value` global options;
- every wrapper subcommand that resolves to a Git alias;
- cli-git policy escape controls on guarded commit and push commands.

Native `--no-verify` remains allowed because it bypasses only retired/native Git hooks;
cli-git policies still execute.

The environment checker rejects before a commit or push:

- `GIT_CONFIG_PARAMETERS` containing a protected key;
- `GIT_CONFIG_COUNT` plus every indexed `GIT_CONFIG_KEY_<n>` and `GIT_CONFIG_VALUE_<n>` protected-key pair;
- `GIT_CONFIG_GLOBAL`,
   `GIT_CONFIG_SYSTEM`,
   or `GIT_CONFIG_NOSYSTEM` trust redirection;
- `GIT_COMMITTER_NAME`,
   `GIT_COMMITTER_EMAIL`,
   or `EMAIL` identity overrides;
- unexpected `core.hooksPath` or newly introduced config-based hook commands.

Author-only variables remain permitted because author and committer are different domains.
Read-only `get` forms and `--blob` remain allowed.

Read operations remain allowed.
Direct edits to `.git/config` or an included file are not command-interceptable;
the commit-environment check catches their resulting scope,
origin,
and effective identity.

Removal of repository-local protected values is allowed only through a dedicated repair operation that reports exactly
which origin and scope it will remove.
These rules have no routine bypass flag.
Changing trusted identity is an explicit administrative operation performed with the real Git binary after inspection,
not an automated wrapper operation.

### Layer C: validate the environment before commit

Add `assertSafeCommitEnvironment` to cli-git's pre-forward lifecycle before commit-producing commands.
It checks repository metadata before real Git runs.

The check evaluates `git config --show-origin --show-scope` with includes enabled and rejects when:

- any protected identity or signing key originates at local or worktree scope,
  including values loaded through `include.path` or `includeIf`;
- a protected process-local `-c` or `GIT_CONFIG_*` override is active;
- `git var GIT_COMMITTER_IDENT` does not exactly match a trusted local committer;
- the committer email is not a principal in the configured allowed-signers file;
- `commit.gpgSign` is not true;
- `gpg.format` is not `ssh`;
- `user.signingKey` or `gpg.ssh.allowedSignersFile` is absent or unreadable;
- either local trust file fails ownership,
  type,
  symlink,
  parent-directory,
  or mode validation.

It reports each offending key with the scope and origin returned by Git.
It does not edit configuration automatically.

Add the same preflight directly to the wrapper before commit-producing commands:
`commit`,
`merge`,
`cherry-pick`,
`revert`,
`rebase`,
`am`,
`pull`,
and their sequencer continuations.
Reject wrapper aliases instead of guessing whether alias expansion creates a commit.

Some sequencer and plumbing paths do not run `pre-commit`.
`commit-tree` can create an unattached commit object without updating `HEAD`.
The wrapper preflight improves locality,
but cli-git's manual-push lifecycle and server enforcement remain the correctness boundaries for every publishable ref
update.

### Layer D: verify created and outgoing commit artifacts

After a wrapped `git commit` succeeds,
verify `HEAD` before the existing auto-push path runs.
A failed verification leaves the commit local and returns a nonzero result.
Amend uses the same check.
Commands that do not auto-push rely on the broader pre-push boundary.

Reuse cli-git's implemented Git-native manual-push probe.
It performs a private native dry run with pre-push records,
validates destination state through `git ls-remote --refs`,
loads object content through bounded `git cat-file --batch`,
and forwards only after the policy result settles.
Disposable pushes must cover byte-exact update records,
destination races,
and all outgoing range classes through the built shadow `git`.

The verifier acquires an isolated object graph before calculating ranges:

1. Create a temporary bare clone with `git clone --bare --shared` from the active worktree.
   The clone sees every local object through a read-only alternates file and does not mutate the source repository.
2. Fetch live remote heads and tags from the hook-provided URL into only
   `refs/prepush-remote/heads/*` and `refs/prepush-remote/tags/*` in the temporary clone.
3. Abort on authentication,
   advertisement,
   fetch,
   or object-connectivity failure.
4. Remove the temporary clone through structured cleanup after verification.

For each native pre-push line inside that complete disposable graph:

- skip a deletion when the local object ID is all zeroes;
- require every nonzero local and remote object to exist;
- peel tag objects to commits when possible;
- for an existing remote ref,
  compute the local commit minus the exact advertised remote object;
- for a new remote ref,
  compute the local commit minus every fetched `refs/prepush-remote/*` tip;
- deduplicate commit object IDs across updates;
- require the committer identity to match the trusted local allowlist;
- run `git verify-commit` for every selected commit;
- require the verified SSH principal to match the commit's committer email.

Do not use remote-tracking refs as authority because they can be stale.
The remote fetch mutates only the temporary bare clone,
never local refs,
configuration,
index,
or object storage.
Existing bad commits already reachable from the remote are subtracted and do not require history rewriting.
An empty remote has no exclusions,
so publishing a complete history to it verifies every reachable commit and may require a separate migration decision.
Any missing object or ambiguous parse aborts the push.

This covers normal pushes,
new branches,
multiple refspecs,
merge commits,
and non-fast-forward attempts.
Annotated-tag behavior is tested separately:
commit objects reachable from a tag are checked,
and tag-signature policy remains out of scope unless the remote fixture proves
that a combined commit-and-tag policy is needed.
Symbolic refs,
all-zero object IDs,
malformed input,
and missing advertised objects are explicit test paths.

The wrapper's direct post-commit check improves error locality.
The manual-push lifecycle covers wrapper-mediated pushes,
including merge commits and batches of existing commits.
Absolute real-Git paths,
GUI clients that embed Git,
and libgit2 bypass the PATH shadow;
the remote fixture must prove the server rejects the same bad updates.

### Layer E: make the server authoritative

Create one active branch ruleset with:

- target `branch`;
- `~ALL` branch inclusion and no exclusions;
- no bypass actors;
- `required_signatures`;
- `committer_email_pattern` containing an anchored regular expression for approved committer emails;
- `non_fast_forward`.

The initial server email set is derived from the measured repository workflows:
`an@aquati.cat` for local commits and `noreply@github.com` for existing GitHub-generated commits.
Refresh the inventory and verify current web,
automation,
release,
and topic-branch creators before activation.
Adding another committer email is an explicit server-policy change,
not something a local commit can self-authorize.

The temporary-branch contract sent to the repository-rules API is equivalent to:

```json
{
  "name": "Verified approved committers fixture",
  "target": "branch",
  "enforcement": "active",
  "bypass_actors": [],
  "conditions": {
    "ref_name": {
      "include": ["refs/heads/git-safety-fixture"],
      "exclude": []
    }
  },
  "rules": [
    { "type": "required_signatures" },
    {
      "type": "committer_email_pattern",
      "parameters": {
        "name": "Approved committer emails",
        "negate": false,
        "operator": "regex",
        "pattern": "^(an@aquati\\.cat|noreply@github\\.com)$"
      }
    },
    { "type": "non_fast_forward" }
  ]
}
```

The fixture is a mandatory contract test,
not an assumption:
it proves anchored-regex behavior,
composition with `required_signatures`,
owner behavior with an empty bypass list,
and the GitHub-generated identity path.
The final full-policy payload changes only the name and the include condition to `~ALL`.

If GitHub rejects the email-rule payload or observed behavior differs:

- do not claim server-side approved-email enforcement;
- activate the `~ALL` ruleset with only `required_signatures` and `non_fast_forward`,
  still with no bypass actors;
- retain exact local identity enforcement in cli-git;
- retain administrator enforcement in classic `main` protection;
- record the failed request,
  response,
  and residual risk that a GitHub-verified but locally unapproved identity could pass the remote signature rule.

This fallback still blocks the diagnosed incident:
`fixture@example.invalid` produced GitHub reason `no_user`,
which is not a verified signature.
It does not broaden the guarantee to a different GitHub-recognized identity.

GitHub's ruleset API documents `~ALL` as the all-branches condition,
`required_signatures` as requiring verified commits,
`committer_email_pattern` as a commit metadata rule,
and `non_fast_forward` as preventing force pushes.
After the full-policy fixture passes,
the ruleset closes ordinary administrator bypass,
rejects a valid signature carrying an unapproved committer email,
and protects topic-branch publication as well as default-branch history.
Under the fallback,
only signature and non-fast-forward guarantees are server-authoritative.

The server can enforce an email but has no committer-display-name rule.
Local checks remain responsible for exact `Name <email>` identity.
An administrator can deliberately alter repository rules,
which is outside the accidental-automation threat model and must remain stated as residual authority.

Keep classic `main` protection active during rollout.
After the all-branch ruleset passes remote fixtures,
enable administrator enforcement and disable force pushes in classic protection too.
The duplicate `main` controls are intentional defense in depth
and preserve the existing deletion and status-check settings.

Do not add a bypass actor for identity or signature enforcement.
This is the final boundary for clients that skip hooks,
invoke a different Git binary,
or pass `--no-verify`.

Investigate a companion all-tag ruleset in the remote fixture.
Keep it only if GitHub demonstrably checks commit signatures reachable from tag updates
without breaking signed release-tag workflows.
Do not claim tag publication is protected until that behavior is verified with a real temporary tag.

## Observability and repair

Every rejection states:

- violated invariant;
- repository root;
- offending command,
  configuration origin,
  or commit object ID;
- safe next action.

Do not log private-key contents or full environment values.
Public signing-key fingerprints and allowed-signers paths are safe to report.

Add a read-only diagnostic command to the cli-git package that prints:

- resolved repository root;
- effective and trusted committer identities;
- protected Git configuration scopes and origins;
- signing mode and public-key fingerprint;
- local verification result for `HEAD`;
- current GitHub protection state when `gh` is available.

The diagnostic command does not repair anything.
A separate narrowly scoped repair command may remove forbidden local or worktree values after showing the exact keys.
It must never rewrite commits.

## Disposable regression matrix

All Git mutations run in temporary repositories or worktrees.
No test writes the main repository's `.git/config`,
refs,
index,
or remote configuration.

### Pi extension fixtures

Use Pi's real extension event path with a deterministic fake model response that requests a Bash tool call.
Verify:

- `{"command":"touch marker","cwd":"/tmp/target"}` is blocked;
- the marker is absent in both session and target directories;
- valid `command` plus `timeout` input reaches the Bash tool;
- an arbitrary future-looking property is blocked;
- the refusal names the unsupported property and directory-safe alternatives.

Run the built-extension load verification in addition to unit tests.

### Git invocation fixtures

Create disposable repositories for:

- bare `git init` while current directory is an existing worktree;
- `git init .` in the root;
- `git init child` under a worktree;
- chained `git -C` targeting a worktree;
- `--git-dir`,
   `--work-tree`,
   `GIT_DIR`,
   and `GIT_WORK_TREE` targeting another repository;
- `--bare` and `--separate-git-dir` under an existing worktree;
- linked-worktree and symlinked targets;
- initialization in a new external temporary directory;
- protected identity writes using legacy and current config syntax;
- `--worktree`,
   `--file`,
   includes,
   `includeIf`,
   protected `-c`,
   and indexed `GIT_CONFIG_*` injection;
- config `--add`,
   `--replace-all`,
   `--unset-all`,
   section rename,
   and section removal variants;
- aliases that expand to protected config writes or commit-producing commands;
- protected hook-path or config-based-hook mutation;
- wrapper policy-escape rejection while native `--no-verify` still runs cli-git policies;
- read-only protected config queries;
- explicit cleanup of contaminated local values.

For every rejection,
hash repository config and refs before and after and require byte equality.

### Commit-environment fixtures

Use isolated `HOME`,
Git configuration,
trusted-committer files,
and SSH keys.
Cover:

- trusted identity with no local override;
- multiple explicitly trusted local identities;
- local name override;
- local email override;
- worktree-scope override;
- included local or worktree identity;
- protected `-c`,
   legacy and indexed `GIT_CONFIG_*`,
   and committer-environment overrides;
- unexpected newly introduced config-based hook commands;
- allowed author-only environment and `--author` metadata;
- malformed display names and exact matching;
- fixture email with a cryptographically valid signature;
- missing signing mode;
- wrong signing key;
- missing,
  symlinked,
  wrongly owned,
  or overly permissive trust files;
- untrusted principal;
- allowed-signers validity windows and key rotation;
- distinct trusted author and committer;
- merge,
  cherry-pick,
  revert,
  rebase,
  `am`,
  `pull`,
  `commit-tree`,
  and sequencer continuation paths.

A rejected pre-commit fixture must leave `HEAD`,
index,
worktree,
and configuration byte-identical.

### Pre-push fixtures

Push to disposable bare remotes through cli-git's Git-native manual-push probe.
Require byte-equal proof that private native pre-push records reach the verifier.
Cover:

- one valid signed commit;
- one valid signature under an untrusted committer;
- one unsigned commit;
- one bad signature;
- multiple outgoing commits with one bad middle commit;
- existing remote object absent from the source clone but acquired into the temporary graph;
- new branch against live remote refs;
- stale remote-tracking refs ignored;
- prior bad ancestors already reachable from the remote excluded without rewriting;
- empty remote requiring verification of every reachable commit;
- remote authentication,
  fetch,
  and object-connectivity failures;
- multiple refs;
- merge commit;
- symbolic and annotated tag inputs;
- malformed and truncated input;
- deletion;
- non-fast-forward update;
- absolute real-Git and embedded-client bypasses reaching server rejection.

Rejected pushes must leave every remote ref unchanged.
Accepted pushes must update only the requested refs.
Every case hashes source refs,
configuration,
index,
and object directories before and after to prove the remote snapshot touched only its temporary bare clone.
The temporary clone must be removed after both success and failure.

### GitHub protection fixture

First export the current classic-protection and ruleset JSON for rollback.
Then create the proposed branch ruleset with a temporary branch-name condition.
Verify:

- an owner push containing an unverified commit is rejected;
- an owner push containing a valid signature and unapproved committer email is rejected;
- an owner force push is rejected;
- an owner push containing an approved committer email and valid signature succeeds;
- a Contents API commit with no custom identity demonstrates the actual GitHub-generated committer path;
- current automation and release commit paths either satisfy the rule or receive an explicit policy decision.

Record the fixture request JSON,
response JSON,
rule-suite results,
and observed remote diagnostics in this plan before creating the `~ALL` ruleset.

Run a separate temporary tag-ruleset experiment:

- push an unsigned tag pointing to a verified commit;
- push a signed tag pointing to a verified commit;
- attempt to introduce an unverified commit only through a tag.

The experiment determines whether a companion tag rule protects commit publication or only tag-object policy.
Remove temporary refs and rulesets after API state and remote behavior are recorded.
Do not test rejection by pushing a bad commit to `main`.

## Rollout sequence

1. Land strict Bash input validation and Pi extension tests.
2. Build the guardrail and install it in global Pi settings.
3. Start a fresh Pi session and verify a synthetic unsupported-`cwd` tool call is blocked before execution.
4. Land cli-git invocation rules and disposable repository tests.
5. Provision the mode-`0600` trusted-committer policy and verify its provenance checks.
6. Land the shared commit-environment checker inside cli-git's pre-forward lifecycle.
7. Prove the built shadow `git` resolves fail-closed from a disposable linked worktree.
8. Land direct post-commit verification and Git-native manual-push verification.
9. Run the full disposable commit and push matrix,
   including deliberate PATH-shadow bypasses.
10. Run package lint,
    type checking,
    unit tests,
    build,
    and end-user wrapper verification through mise tasks.
11. Verify a normal signed checkpoint commit remains local until the post-commit verifier succeeds.
12. Export current remote-protection JSON and inventory current branch creators and committers.
13. Test the proposed all-branch ruleset and tag behavior with temporary refs.
14. Activate either the verified full-policy all-branch ruleset or its documented signature-only fallback,
    with no bypass actors.
15. Enable administrator enforcement and disable force pushes in classic `main` protection.
16. Query GitHub rulesets and classic protection through the API and verify the exact final fields.
17. Confirm subsequent pushed commits report GitHub verification reason `valid` and approved committer email.
18. Update this plan's status and measured results.

Commit each repository change at the earliest scoped checkpoint.
Stage only explicit task paths so concurrent changes are never swept into a commit.

## Rollback

Rollback is per layer,
not all-or-nothing.
The active all-branch signature and non-fast-forward ruleset with no bypass actors
is the minimum safe publication state after rollout.
Approved-email enforcement remains part of that minimum only when its empirical contract passes.
Never disable signature enforcement merely to repair a local false positive.

- Pi guardrail false positive:
  keep rejecting `cwd`,
  narrow only the newly legitimate property rule,
  and restart Pi.
  Do not remove the guardrail package entirely.
- cli-git false positive:
  revert the specific invocation rule while retaining identity,
  manual-push,
  and server verification.
- GitHub policy blocks an intended workflow:
  add a measured approved email or repair that workflow's signing;
  restore old protection only from exported JSON after confirming another server rule still blocks unverified commits.
  If email enforcement is withdrawn,
  record the transition to the documented fallback and its residual identity risk.

The implementation records current classic-protection JSON,
ruleset JSON,
and resulting ruleset history before mutation.
A rollback must name its authorizer,
actor,
time,
reason,
minimum safeguard that remains active,
and restoration verification.
Earlier commits are never rewritten as rollback.

## Rejected shortcuts and ranking

1. Full defense in depth,
   as designed in this plan.
   Pros:
   the failure stops before execution,
   before Git mutation,
   before commit,
   before push,
   and at the server;
   each layer covers bypass of the preceding layer.
   Cons:
   more implementation and fixture work,
   plus explicit identity onboarding.
2. Git and server enforcement without Pi validation.
   Pros:
   protects history and requires less Pi-specific code.
   Cons:
   unsupported tool input can still mutate unrelated working state before Git notices.
3. Pi guardrail or prompt rules alone.
   Pros:
   fastest initial change and clearest immediate diagnostic.
   Cons:
   extensions can be absent or bypassed,
   and no independent boundary prevents publication.

Ranking:
full defense in depth is preferred over Git-only because it prevents the wrong-directory mutation itself;
Git-only is preferred over Pi-only because hooks and server protection still cover alternate clients and publication.

## Acceptance criteria

The work is complete only when:

- unsupported Bash properties cannot reach shell execution;
- the guardrail is active in a fresh real Pi session;
- reinitialization and protected config writes leave disposable repository state byte-identical;
- local trust files pass ownership,
  type,
  symlink,
  parent-directory,
  and mode checks;
- contaminated identity fails before a normal commit is created;
- a created bad commit cannot reach the wrapper's auto-push path;
- Git-native manual-push records reach the verifier byte-for-byte;
- manual push rejects every fixture containing an untrusted identity,
  unverifiable commit,
  missing object,
  or malformed update;
- deliberate absolute-real-Git and embedded-client bypasses are rejected by the server;
- valid signed commits still commit and push through user-facing commands and linked worktrees;
- every branch requires verified signatures and fast-forward updates with no bypass actors;
- every branch also requires approved committer emails when the empirical GitHub contract passes,
  otherwise the documented signature-only fallback and its narrower guarantee are recorded;
- classic `main` protection also enforces signatures for administrators and forbids force pushes;
- an owner cannot publish an unverified fixture commit to a temporary protected branch;
- an owner also cannot publish an unapproved but valid identity when the empirical email-rule contract passes;
- current web and automation workflows have measured compatibility decisions;
- tag-publication behavior is tested and either protected by a verified tag ruleset
  or recorded as a residual limitation;
- package linting and type checks report zero errors or warnings;
- every exported policy branch has a passing test;
- documentation matches installed configuration and observed behavior;
- remote-protection rollback material and ruleset history are recorded;
- earlier commits remain unchanged.

## References

- Incident diagnosis:
  `doc/troubleshooting/pi-bash-ignored-cwd-git-signature-verification.md`
- Pi guardrail:
  `package/pi-plugin/guardrail/README.md`
- cli-git entry point:
  `package/git-policy/cli/src/index.ts`
- cli-git root rule:
  `package/git-policy/cli/src/rules/require-root.ts`
- Cli-git manual-push lifecycle:
  `package/git-policy/cli/src/policy-engine/manual-push-lifecycle.ts`
- Cli-git post-commit lifecycle:
  `package/git-policy/cli/src/policy-engine/post-commit-lifecycle.ts`
- GitHub commit signature verification:
  <https://docs.github.com/en/authentication/managing-commit-signature-verification/about-commit-signature-verification>
- GitHub repository rules API:
  <https://docs.github.com/en/rest/repos/rules>
