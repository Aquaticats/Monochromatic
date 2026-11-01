# cli-git staging in a fresh temporary worktree requires trusted policy and scanner inputs

## Symptom

On 2026-09-13,
a scoped `git add` in the owned runtime-closure prototype worktree exited `2`:

```text
cli-git configuration is not trusted; run `git cli-git trust` after reviewing it.
```

The engine event code was `config-untrusted`.
After native trust,
staging reached a separate failure:

```text
Forbidden-strings scanner exited with infrastructure status 2.
```

That engine event was `plugin-threw`,
policy `security/forbidden-strings`,
trigger `pre-forward`.
The scanner identified itself as `forbidden-strings 0.3.0`.

These failures are distinct from the previously investigated ignored-state worktree copier.
Neither staging attempt forwarded its write after its respective policy failure.

## Root cause

The new worktree has a new canonical policy path.
`package/git-policy/cli/src/trust/trust-service.ts:259`
refuses absent trust with the observed diagnostic:

```ts
// package/git-policy/cli/src/trust/trust-service.ts
'config-untrusted',
'cli-git configuration is not trusted; run `git cli-git trust` after reviewing it.',
```

The policy bytes were compared with the development worktree and matched.
Trust still needed to be granted through the tool's native reviewed-policy flow.
`package/git-policy/cli/src/trust/account-root.ts:35`
derives its production registry from the operating-system account,
not repository environment overrides:

```ts
// package/git-policy/cli/src/trust/account-root.ts
await realpath(userInfo().homedir);
```

The scanner failure concerned a missing explicit rules file,
not an unsupported `--builtin-rules` flag.
The actual scanner help lists that flag.
Root `mise.toml:1239` supplies a worktree-relative generated rules input:

```toml
# mise.toml
FORBIDDEN_STRINGS_RULES = "{{config_root}}/.cache/forbidden-strings.rules.txt"
```

Fresh worktree creation deliberately did not copy ignored state.
The development rules file existed and was 10905 bytes;
the corresponding temporary-worktree path was absent.
`package/forbidden-strings/cli/src/lib.rs:601`
classifies an environment-selected rules file as explicit:

```rust
// package/forbidden-strings/cli/src/lib.rs
let explicit_rules_source =
    cli.rules_path.is_some() || env::var("FORBIDDEN_STRINGS_RULES").is_ok();
```

An explicit missing rules input is not the scanner's allowed implicit-baseline-only case.
Direct invocation reproduced its deciding error:

```text
forbidden-strings: read rules .../.cache/forbidden-strings.rules.txt: No such file or directory (os error 2)
```

## Verification

Evidence is retained in the agent scratch directory:

- `runtime-closure-config-trust-20260913.out`
- `runtime-closure-prototype-commit-20260913.out`
- Process logs for `proc_3f96` and `proc_45c7` retain the separate rejected staging attempts.

The native trust disclosure records the exact temporary configuration path,
1423 source bytes and a 40728-byte bundle.
It also discloses recursive authority under that temporary root
and the bundled policy package's exclusion from automatic invalidation.
This is not a sandbox or an authorization to trust unrelated temporary directories.

Working cases:

- The unchanged reviewed policy succeeds through `git cli-git trust --yes`.
- The scanner accepts its existing `--builtin-rules` option.
- Scoped staging and commit succeed after supplying the existing scanner executable and generated rules input.

Refused cases:

- Untrusted temporary policy:
  `config-untrusted`,
  exit `2`.
- Missing explicit scanner rules path:
  scanner exit `2`,
  surfaced as `plugin-threw`.

The prototype commit is `cb3101583` on a detached worktree.
cli-git explicitly reported that it skipped auto-push because HEAD was detached.
No production build-policy change was made by that commit.

## Verified remedies

After comparing and reviewing the exact policy,
use native trust consent:

```sh
# In the reviewed owned temporary worktree
 git cli-git trust --yes
```

Tradeoff:
this grants full-account execution authority to that policy snapshot,
including its disclosed recursive intent.
Revoke that temporary root's trust before removing the worktree.
Do not substitute an environment override or disable the policy.

For this prototype,
explicit symlinks supply only the existing development scanner executable and generated rules file:

- `package/forbidden-strings/cli/target/release/forbidden-strings`
- `.cache/forbidden-strings.rules.txt`

Tradeoff:
these remain development-owned inputs rather than immutable snapshots.
They support guarded local staging,
not acquisition-runtime provenance.
The owned links must be included in the later cleanup inventory.
No ignored-state tree was copied,
no rules were weakened and no native Git write bypass was used.

## Container-backed cleanup observation

Native `git worktree remove --force` later reports:

```text
error: failed to delete '.../translation-repair-runtime-closure-20260913': Permission denied
```

In this attempt the Git registration was already absent afterward,
while the ignored directory remainder was still present.
Inspection finds empty bind-mount target directories,
plus their one-child parents,
owned by host UID/GID `524288` with mode `1755`.
The caller owns their surrounding temporary workspace.
Git file inventories had not described these empty directories.

The measured `podman unshare` identity is UID `0` for the caller,
and UID/GID `1:1` for those placeholders.
The inspected Podman source documentation,
`docs/source/markdown/podman-unshare.1.md:10`
in `/var/home/user/temp/agent/podman-v5.8.4-20260903`,
explains that mapping:

```text
namespace is configured so that the invoking user's UID and primary GID appear
to be UID 0 and GID 0, respectively.
```

The repair checks each exact path as a real directory,
verifies its owner,
and requires empty contents or the sole expected `data` child.
It records device/inode identity before changing ownership.
`podman unshare chown --no-dereference 0:0 <exact checked paths>`
then restores caller ownership without recursive traversal.
After-state checks confirm unchanged device/inode identities and the caller's UID/GID.
Neither the real corpus nor linked dependency ownership is changed.

The partial-deletion recheck also accounts for Git representing nested fixture repositories by directory entries:
the recorded `artifact-generation-shallow-.../` entry covers its `.git/shallow` file.
Remaining files are checked against the original audit,
with only such recorded directory entries allowing descendants.
Empty `.local/state/cli-git/trust/v1/transactions` scaffolding carries no additional file content.
The owned unregistered remainder is removed,
and native Git removes the still-registered second worktree.

`sealed-runtime-proof-20260913` retains ownership-before/after records,
the partial-removal recheck and final removal evidence.
Both directories and registrations are absent at `2026-09-13T08:34:22.034Z`.
Trust was revoked and independently reported untrusted before removal.

Proposed `AGENTS.md` clarification,
not applied:
retain `GCL`'s existing cleanup-review scope and require inspection of all directory owners,
including empty container mount targets,
in addition to ignored root artifacts.

## What does not work

- Trusting the original path does not automatically trust every detached worktree copy.
- Native trust does not create missing generated scanner inputs.
- Retrying an unchanged missing explicit rules path reproduces scanner exit `2`.
- Rebuilding or changing scanner flag handling is not justified by this incident;
  the installed binary already supports the requested flag.

## Upstream filing decision

1.  Upstream fault:
    no.
    The worktree lacked reviewed trust and generated local inputs.
2.  Fixability:
    existing native trust and explicit input provisioning resolve the observed failures.
3.  Supported use:
    cli-git's management documentation and the scanner's actual help describe these operations.
4.  Contribution policy:
    not reached;
    no upstream modification or communication is proposed.
5.  Maintainer intent:
    not inferred from these expected refusals.
6.  Prototype:
    the scoped staging/commit invocation succeeded without changing either tool or its policies.

Nothing to file upstream.
Local trust revocation and audited temporary-worktree cleanup are complete.
The repair did not weaken the checks.
