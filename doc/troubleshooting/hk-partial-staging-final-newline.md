# hk 1.50.0 pre-commit fixing duplicates LF at a partially staged EOF tail

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

> Scratch-path note:
> `/tmp/agent` paths in this document are historical.
> Use `~/temp/agent` for current work.

## Symptom

With `fix = true` and `stash = "git"`,
hk can preserve the committed blob but alter the partially staged worktree.
The triggering byte states are:

```text
index before hook:     staged
worktree before hook:  staged\nunstaged\n
fixer output:          staged\n
```

Here `index before hook` has no final LF.
After hk runs:

```text
index after hook:      staged\n
worktree after hook:   staged\n\nunstaged\n
```

The unstaged text survives,
but hk inserts an unintended blank line at the boundary.
This violates exact partial-staging preservation.

Before retirement,
the repository kept hk pre-commit newline checks read-only and exposed rewriting only through the explicit `fix` hook
with `--no-stage`.
Issue `#357` removed hk and Pkl after cli-git parity passed.

## Cli-git migration resolution

The core cli-git `final-newline` policy now avoids hk's stash-tail merge entirely.
For commit correction,
cli-git copies the real index,
projects the exact selected candidate state,
applies single-path Git patches to that private index,
and commits through the private index.
The worktree is not rewritten,
so a partially staged tail remains byte-for-byte identical.
After success,
cli-git reconciles the real index to the committed result without folding the unstaged tail into either state.

Direct `git cli-git fix` uses another private index for whole-policy convergence.
After convergence it revalidates selected worktree bytes against the initial snapshot,
installs sibling-file replacements atomically,
and verifies that the real index bytes are unchanged.
`git cli-git check` and manual push remain read-only.

The packed acceptance fixture in
`package/git-policy/cli/src/trust/fixtures/built-final-newline-consumer.ts`
exercises direct check,
direct fix,
commit correction,
and rejected manual push through the shipped shadow `git` executable.
The transaction fixture in
`package/git-policy/cli/src/trust/fixtures/built-autofix-transaction-consumer.ts`
retains the exact partial-staging regression coverage.
Independent final-newline workflow run `29171565809` passed typed Node orchestration and direct policy checking.
Issues `#356` and `#357` completed release readiness and removed hk/Pkl infrastructure.
This document remains as durable root-cause and exact-byte regression evidence;
it no longer prescribes repository setup.

`package/git-policy/cli/src/trust/fixtures/built-final-newline-consumer.ts` is not in the working
tree and no commit in this repository carries it.
`package/git-policy/cli/src/trust/fixtures/built-autofix-transaction-consumer.ts` is not in the
working tree and no commit in this repository carries it.

## Root cause

The source trace uses `jdx/hk` tag `v1.50.0`,
commit `e9cc0984a1a8d639519e76ab6e329effcce42144`.

Both documented stash names reach the same behavior.
`pkl/Config.pkl:197-200` states:

```pkl
/// - `"git"`: Use `git stash` to stash unstaged changes before running fix steps.
/// - `"patch-file"`: Alias of `git` behavior for now.
/// - `"none"`: Do not stash unstaged changes before running fix steps.
/// - `true` (boolean): Alias of `"git"`.
```

Before running a fixer,
the hook captures the staged index and stashes all unstaged changes.
`src/hook.rs:1069-1074` contains:

```rust
{
    let mut r = repo.lock().await;
    r.capture_index(&files_vec)?;
    // Stash ALL unstaged changes in the repository (not only files under consideration)
    // so that unrelated worktree changes do not affect or get affected by fixers.
    r.stash_unstaged(&file_progress, stash_method, &git_status)?;
}
```

During restoration,
hk reads three versions:
the index before the hook,
the worktree before the hook,
and the fixer result.
`src/git.rs:1180-1201` contains:

```rust
let work_pre = if let Some(map) = &self.saved_worktree {
    map.get(&path).cloned()
} else {
    None
}
.or_else(|| work_bytes.and_then(|b| String::from_utf8(b).ok()));
let base_pre =
    git_read_raw(["cat-file", "-p", &format!("{}^1:{}", &stash_ref, path_str)])
        .ok();
let index_pre =
    git_read_raw(["cat-file", "-p", &format!("{}^2:{}", &stash_ref, path_str)])
        .ok();
let fixer = fixer_map
    .get(&path)
    .and_then(|(_, oid)| git_read_raw(["cat-file", "-p", oid]).ok());
```

The pure-tail special case then concatenates the fixer result and the complete tail.
`src/git.rs:1245-1271` contains:

```rust
if let (Some(f), Some(w), Some(i)) =
    (fixer.as_deref(), work_pre.as_deref(), index_pre.as_deref())
{
    // Try strict prefix first
    let mut tail_opt = w.strip_prefix(i);
    if tail_opt.is_none() && i.ends_with('\n') {
        tail_opt = w
            .strip_prefix(&i[..i.len().saturating_sub(1)])
            .filter(|tail| tail.is_empty());
    }
    if let Some(tail) = tail_opt {
        let mut combined = f.to_string();
        if !tail.is_empty() {
            combined.push_str(tail);
        }
        merged = combined;
    }
}
```

For the triggering fixture:

- `i` is `"staged"`.
- `w` is `"staged\nunstaged\n"`.
- `f` is `"staged\n"`.
- `w.strip_prefix(i)` returns `"\nunstaged\n"`.
- Concatenation produces `"staged\n\nunstaged\n"`.

The three-way merge is not the cause in this case.
The later pure-tail override replaces its result with the duplicated separator.

## Verification

### Release behavior

The reproduction used:

```text
hk 1.50.0
git 2.54.0
```

A disposable repository installed the real hk hook and used a byte-exact Node assertion.
The essential setup was:

```sh
printf 'base\n' > partial.txt
git add partial.txt
git commit partial.txt --message 'test: add base'
printf 'staged' > partial.txt
git add partial.txt
printf 'staged\nunstaged\n' > partial.txt
git commit
```

The pre-commit fixer wrote `staged\n` to the index and
`staged\n\nunstaged\n` to the worktree.

### Patterns that work cleanly

- No unstaged change:
  the fixer result reaches both index and worktree.
- A staged file already ending in LF with a tail appended after that LF:
  the tail does not include a separator LF already supplied by the fixer.
- Read-only checking with `stash = "git"`:
  a rejected commit restores both staged and worktree bytes exactly.

### Pattern that fails

All of these conditions are required:

- Fix mode is active.
- Staged content has no final LF.
- The fixer adds final LF.
- Unstaged content appends a line at that EOF boundary.

### Upstream-compatible prototype

A fresh clone at tag `v1.50.0` was created under a private `/tmp/agent/` directory.
Its origin,
tag,
and commit were verified before editing.
The prototype strips one leading LF from the tail only when the fixer already ends in LF.
A two-hunk patch and regression test are stored in
[hk-partial-staging-final-newline.patch](hk-partial-staging-final-newline.patch).

The binary was built in a disposable container with two CPUs and 2 GiB memory:

```sh
PROTOTYPE=/tmp/agent/hk-newline-tail-prototype-8fF1KLdu
podman run --memory=2g --cpus=2 --rm \
  --volume "$PROTOTYPE:/work:Z" \
  --workdir /work rust:1.88-bookworm cargo build --release
```

The pre-patch 1.50.0 binary reproduced the extra blank line.
The patched binary produced these exact bytes:

```text
index:     staged\n
worktree:  staged\nunstaged\n
```

The targeted upstream Bats regression also passed:

```text
1..1
ok 1 fix avoids duplicate separator when fixer adds missing final newline
```

## Historical verified workarounds

### Keep hk pre-commit read-only

Before retirement,
the repository kept `stash = "git"` so hk checks observed staged bytes,
but removed `fix = true` from `pre-commit`.
A failed commit restores the partial worktree exactly.
The historical explicit command was:

```sh
hk fix --all --step final-newline --no-stage
```

Unlike pre-commit,
the `fix` hook does not enable stashing.
It normalizes the full worktree file instead of isolating and merging a staged prefix;
`--no-stage` leaves the index blob untouched.
An exact-byte fixture staged `staged`,
left `staged\nunstaged` in the worktree,
and produced `staged\nunstaged\n` without changing the staged blob.
This path therefore does not enter the faulty stash merge.

Tradeoff:
contributors must inspect and stage corrected files before retrying the commit.
That friction is preferred over silent worktree mutation.

### Separate the EOF-tail edit

Commit or stage the missing final LF before appending an unstaged tail,
or stage the tail with the same commit.

Tradeoff:
this constrains normal partial-staging workflow and depends on contributors remembering the edge case.
It is not suitable as repository enforcement.

## What does not work

### Keep pre-commit auto-fix and document the blank line

Documentation does not preserve worktree bytes.
The mutation is silent and can become part of a later commit.

### Change `stash` to `"patch-file"`

At hk 1.50.0,
`patch-file` is an alias of the Git stash behavior and reaches the same merge code.

### Use `stash = "none"` for pre-commit auto-fix

The fixer then runs against worktree content rather than an isolated staged snapshot.
A pre-commit auto-fixer can rewrite unstaged content,
which defeats staged-only commit semantics.
The explicit no-stage fix intentionally has different semantics:
it normalizes the worktree and leaves the index untouched.

### Assert only the worktree prefix and suffix

Upstream's existing `fix_preserves_unstaged_newline.bats` checks that the worktree starts with `formatted\n` and
ends with the unstaged tail.
Those assertions allow an extra blank line in the middle.
The prototype adds an exact-byte assertion.

## Upstream filing decision

No matching exemption exists under this repository's `.out-of-scope/` directory.

GitHub API searches completed for `partial staged newline blank line` and `stash fixer duplicate newline` before the
account search limit was reached.
A fallback web search found the directly related merged
[jdx/hk pull request #310](https://github.com/jdx/hk/pull/310),
which introduced the current tail-append logic and promises that the worktree preserves the full unstaged tail.
Pull request [#931](https://github.com/jdx/hk/pull/931) is related merge work but addresses lost fixer tail deletions,
not duplicate separator LF.
No second issue should be opened.

The six filing constraints resolve as follows:

1.  **Is it really upstream's fault?
    ** Yes.
    Pull request #310 explicitly promises preservation of full worktree content,
    while the current pure-tail concatenation adds one byte.
2.  **Can upstream fix it?
    ** Yes.
    The verified change is local to the pure-tail override and its regression test.
3.  **Are they supporting this use case?
    ** Yes.
    Stash-based fixer preservation has dedicated source logic and Bats coverage.
4.  **Would the repo welcome our contribution?
    ** Yes.
    The contribution guide asks contributors to discuss non-obvious changes,
    run CI,
    and address automated review.
    No AI-assistance ban was found;
    the related upstream pull requests themselves disclose AI assistance.
5.  **Will they likely fix it?
    ** Yes.
    The maintainer merged #310 and #931 for adjacent exact-byte stash bugs.
6.  **Have we prototyped a minimal fix compatible with their architecture?
    ** Yes.
    The saved patch changes the existing pure-tail branch,
    adds an exact-byte Bats case,
    builds successfully,
    fails with the release binary,
    and passes with the patched binary.

Nothing was posted upstream.
Because #310 is the existing thread,
the filing artifact is an additive comment rather than a new issue.

~~~md
hk 1.50.0 still has one exact-byte tail edge after #310.

Reproduction:

1. Stage `staged` with no final LF.
2. Leave `staged\nunstaged\n` in the worktree.
3. Run a stash-based fixer whose output is `staged\n`.

The index is correct (`staged\n`), but the restored worktree is
`staged\n\nunstaged\n`.

The current pure-tail branch computes `tail = w.strip_prefix(i)`, which is
`\nunstaged\n`, then appends that whole tail to fixer output that already ends in LF.
The existing prefix/suffix assertions in `fix_preserves_unstaged_newline.bats` do not catch the interior extra LF.

I verified a minimal patch at v1.50.0 that strips one leading tail LF only when fixer output already ends in LF,
plus an exact-byte Bats regression.
The targeted test passes, and an actual pre-commit fixture preserves:

```text
index:     staged\n
worktree:  staged\nunstaged\n
```

Patch and reproduction details are recorded in
`doc/troubleshooting/hk-partial-staging-final-newline.md` and its adjacent `.patch` file in the Monochromatic repo.
This comment was prepared with AI assistance;
a human should review the patch line by line before proposing it upstream.
~~~
