# cli-git 0.1.6: an LFS clean filter fails on an unrelated asset when `.git/lfs` cannot create its temporary directory

Note 2026-11-04T22:23:58-05:00:
the program `cli-git-native` is renamed `monochromatic-git-policy-cli`,
the command word `git cli-git` becomes `git monochromatic-git-policy-cli`,
the settings file `cli-git.config.jsonc` becomes `monochromatic-git-policy-cli.config.jsonc`,
and the folders,
logs and service units that carry `cli-git` carry `monochromatic-git-policy-cli`
(the list is in `doc/planning/git-policy-cli-program-rename.md`).
The new names arrive with the first release after this date;
this repository switches its installed program and its settings file when it installs that release.
This record keeps the names of its time.

Note 2026-11-09T16:46:15-05:00:
these citations in this record name paths that no longer exist.
`package/pi-plugin/auto-mode/native/build-reuse.ts` left the tree on 2026-10-19 in `ff55ced353`.
`package/pi-plugin/auto-mode/native/build-reuse.platform.test.ts` left the tree on 2026-10-19 in `ff55ced353`.
`package/pi-plugin/auto-mode/native/` is not in the tree;
its files were deleted over several commits,
most of them on 2026-10-19 in `ff55ced353`.
`package/git-policy/cli/src/native/shadow_create.rs` is now
`package/git-policy/cli/src/native/shadow/shadow_create.rs`,
moved 2026-11-03 in `373765289c`.

Note 2026-11-10T15:05:41-05:00:
this record absorbed the second incident of `git-policy-wrapper-launcher-path.md`,
which held the symptom,
the root cause and the three-arm verification for the same failing commit.
That file keeps the launcher PATH incident and points here for the LFS one,
so one incident has one record with the required sections.

## Symptom

On 2026-10-18,
the configured MiMo RPC worker `pi-mimo-rpc-session-9Tl0ts` attempts a protected,
explicit-path commit of `package/pi-plugin/auto-mode/native/build-reuse.ts`
and `package/pi-plugin/auto-mode/native/build-reuse.platform.test.ts`.
The bounded Fedora 44 consumer has a disposable home,
2 GiB memory and two CPUs.
Its qualified wrapper is active.
The commit exits 128:

```text
Error cleaning Git LFS object: open <worktree>/.git/cli-git/shadow/<transaction>/lfs/tmp/2882421905: no such file or directory
error: packet write failed: Broken pipe
error: external filter 'git-lfs filter-process' failed
fatal: package/ide/app/design/screenshots/2026-10-05-annotations-as-built/caret-card-dark.png: clean filter 'lfs' failed
```

The named LFS asset is outside the requested commit paths.
The owned source changes remain staged.
The worker stops Git retries and preserves its complete original tool event and stderr.
Supervisor delivery of those exact paths through the qualified host wrapper succeeds as `9363d8998b`.

The paths this record names under `package/pi-plugin/auto-mode/native/` were removed on 2026-10-19
in `ff55ced353` (refactor(pi-plugin-auto-mode):
retire unapproved native Pi build);
no file in the working tree carries them now.

Anyone meets the same failure in any repository whose `.git/lfs` cannot create a temporary directory:
a pathspec-scoped or whole-tree commit exits 128 while Git runs the clean filter,
whether or not the asset in the message is in the commit's paths.

## Root cause

Git LFS creates its temporary files under `<git storage directory>/lfs/tmp`,
and Git LFS has no setting that moves only that directory;
`lfs.storage` moves the whole store.
Measured on this host with git-lfs 3.8.1 on 2026-11-10:

```text
$ git lfs env   # in a disposable repository
LocalMediaDir=<repo>/.git/lfs/objects
TempDir=<repo>/.git/lfs/tmp
```

The wrapper's shadow repository links its `lfs` entry to the real `.git/lfs`:

```rust
// package/git-policy/cli/src/native/shadow/shadow_links.rs:288
fn link_directory_entries(
    source_directory: &Path,
    shadow_directory: &Path,
    skip: SkipRule,
) -> std::io::Result<()> {
    for entry in shared_entries(source_directory, skip)? {
        let destination: PathBuf = shadow_directory.join(&entry.name);
        share_replacing_template(entry.source.as_path(), destination.as_path(), entry.directory)?;
    }
    return Ok(());
}
```

The shadow links every common-directory entry except the private ones
(`link_shadow_entries`,
same file,
line 368),
and the module doc names `lfs` among the entries that must be the real ones
(`package/git-policy/cli/src/native/shadow/shadow_links.rs:9`).
So the failing path is the shadow path in the message,
while the directory that must exist and be writable is the real one.

When `.git/lfs/tmp` is absent and `.git/lfs` cannot be written to,
Git LFS cannot create it and reports `no such file or directory` on the temporary file.
Measured on this host with git-lfs 3.8.1 and no wrapper in the command:

```text
$ rm -rf .git/lfs/tmp; chmod a-w .git/lfs; printf 'PNG1\n' | git lfs clean
Error cleaning Git LFS object: open <repo>/.git/lfs/tmp/909698913: no such file or directory
exit 2
```

The same command with `.git/lfs` left writable succeeds and creates `tmp` itself,
and with `tmp` present and writable inside a read-only `.git/lfs` it also succeeds.
The clean filter ran for the unrelated modified asset,
not for the named files.

## Verification

Version under test:
Git reported `2.56.0`,
the installed wrapper identified `0.1.6`,
and the git-lfs side was re-measured on 2026-11-10 with git-lfs 3.8.1.

Harness:
disposable repositories with an LFS-tracked PNG modified in the worktree and an explicit ordinary-file commit
pathspec,
using the installed wrapper and,
for comparison,
raw Git.

- A writable `.git/lfs`,
  with or without `tmp`:
  the wrapper commits only the named file.
  A Fedora 44 container limited to 2 GiB and 2 CPUs,
  with a disposable `HOME` and no network,
  behaves the same.
- `.git/lfs` read-only and `tmp` absent:
  the wrapper fails with the message in the Symptom section,
  and raw Git fails with the same message naming `.git/lfs/tmp`,
  so the wrapper does not cause it.
- `.git/lfs` read-only and `tmp` present and writable:
  the wrapper commits only the named file.

The git-lfs probe of the same mechanism,
without any wrapper in the command,
gave these three arms:

- `tmp` absent and `.git/lfs` writable:
  `git lfs clean` exits 0 and leaves `.git/lfs/tmp` behind.
- `tmp` absent and `.git/lfs` read-only:
  exits 2 with `Error cleaning Git LFS object: open <repo>/.git/lfs/tmp/<number>: no such file or directory`.
- `tmp` present and writable inside a read-only `.git/lfs`:
  exits 0.

The consumer's mount permissions were not inspected,
so that the consumer's `.git/lfs` was read-only is consistent with the evidence but not confirmed.

The retained receipt of the incident is the worker event,
`call_f94fb711ff224917b3b6ec0b` in
`${HOME}/temp/agent/pi-mimo-rpc-session-9Tl0ts/host-events.jsonl`.
The receipt named `git-wrapper-lfs-environment-probe.json` in
`${HOME}/temp/agent/pi-mimo-preparation-failure-20261018/` belongs to the missing-helper probe,
which stays in `git-policy-wrapper-launcher-path.md`.
The git-lfs arms above ran in a disposable repository nothing keeps.

## Verified workarounds

Leave `.git/lfs` writable,
or create `.git/lfs/tmp` with write access before the child starts.
Both are launch-environment requirements,
and they cover exactly the failing arm:
a read-only `.git/lfs` with a writable `tmp` commits only the named file.

Tradeoffs:
the directory is shared by every process the child runs,
so the launcher grants write access to the LFS store of the checkout it is running in;
and the requirement has to be restated by every environment that runs a commit,
because nothing in the wrapper can detect it before Git starts.

Launch acceptance should run a scoped commit of an ordinary file in a repository that tracks LFS files.

## What does not work

### Reading the observation as an unidentified boundary failure

The earlier reading of this incident,
that the observation establishes an environment-sensitive boundary failure without identifying why the LFS
temporary path is unavailable,
is superseded by the probes above:
the missing or unwritable `.git/lfs/tmp` is the cause,
and raw Git fails with the same message when no wrapper runs.

### A wrapper-specific LFS defect

Rejected because raw Git fails with the identical message naming `.git/lfs/tmp`.
Attributing the failure to the wrapper would send the next investigator into the shadow code for a directory
Git LFS itself has to create.

### Redirecting the store with `lfs.storage`

Rejected because that setting moves the whole LFS store,
so the objects of an owned asset would be discarded when the shadow is removed.

### Having the wrapper create `.git/lfs/tmp`

Rejected because the wrapper runs with the child's permissions:
it can create the directory only where the child could,
so a store the child cannot write stays unwritable.

## Upstream filing artifact

Nothing to file and nothing to add.
The behaviour is Git LFS's documented one:
its temporary files live under `<git directory>/lfs/tmp`,
and no setting moves only that directory.

[Issue 1770](https://github.com/Aquaticats/Monochromatic/issues/1770) records the failed real consumer,
successful host delivery and required disposable native-Git/wrapper/filter controls.
Searches across all issue states for `LFS shadow`,
`filter-process` and `unable to write new index file` find no matching incident before filing.

No wrapper repair is claimed.
No Git,
Git LFS,
Pi,
Node or Linux defect is attributed from this observation.
The separate source worker's `unable to write new index file` failure
and the historical `bad line length character: Not ` failure remain separate incidents until their boundaries match.
This also differs from the missing-wrapper PATH incident in
[issue 1768](https://github.com/Aquaticats/Monochromatic/issues/1768)
and from `git-policy-auto-push-reconciliation.md`.

### Upstream filing decision

1. **Is it really upstream's fault?**
   No.
   Git LFS places temporary files under `<git directory>/lfs/tmp`,
   which its own `git lfs env` reports as `TempDir`,
   and it offers no setting for that directory alone.
   The failure needs a store it cannot write,
   which our launch environment supplies.
2. **Can upstream fix it?**
   Not applicable as a defect.
   A setting that moves only the temporary directory would be a feature upstream does not offer,
   and we do not need it:
   a writable store or a pre-created `tmp` already solves the failure at our boundary.
3. **Are they supporting this use case?**
   Yes,
   and the case is ordinary:
   every clean writes a temporary file into the store,
   so write access to the store is what the tool asks of any caller.
4. **Would the repo welcome our contribution?**
   Not applicable.
   No filing is made and no contribution is offered.
5. **Will they likely fix it?**
   Not applicable.
   Nothing is broken upstream,
   and this record asserts no leaning either way.
6. **Have we prototyped a minimal fix compatible with their architecture?**
   Not applicable upstream.
   The minimal fix is at our boundary and is prototyped in the Verification section:
   a writable store,
   or a writable `tmp` created before the child starts,
   in a disposable repository with an LFS-tracked asset.

The default policy is not to file,
and this record files nothing.
