# Git 2.54.0 commit interruption lacks an exact pre-ref OID but exposes durable reflog attribution

## Symptom

A wrapper must prepare index recovery state before `git commit` can advance `HEAD`.
The landed commit OID cannot be recorded in that prepared state because Git computes it only after message cleanup,
hooks,
parent selection,
and optional signing.
If the wrapper exits after Git updates `HEAD` but before the wrapper records the OID,
matching only the intended tree and parents cannot distinguish the wrapper commit from another commit with the same
shape.

Git provides `GIT_REFLOG_ACTION` as durable ref-update provenance.
With `GIT_REFLOG_ACTION=cli-git:transaction:<nonce>`,
the resulting reflog subject begins with:

```txt
cli-git:transaction:<nonce>: <commit subject>
```

This identifies the Git invocation without predicting its commit OID.
It does not work when reflogs are unavailable,
and later ref movement replaces the latest reflog entry.
Recovery must fail closed in either case.

## Root cause

The source trace uses Git commit `f60db8d575adb79761d363e026fb49bddf330c73`.

`builtin/commit.c:1847` to `builtin/commit.c:1851` reads the action from the environment before parent and commit
construction:

```c
/* Determine parents */
reflog_msg = getenv("GIT_REFLOG_ACTION");
if (!current_head) {
```

`builtin/commit.c:1938` to `builtin/commit.c:1948` first creates the commit object
and only then passes its resulting OID
and the action into the ref update:

```c
if (commit_tree_extended(sb.buf, sb.len, &the_repository->index->cache_tree->oid,
                         parents, &oid, author_ident.buf, NULL,
                         sign_commit, extra)) {
        rollback_index_files();
        die(_("failed to write commit object"));
}

if (update_head_with_reflog(current_head, &oid, reflog_msg, &sb,
                            &err)) {
```

The ordering disproves an earlier assumption that a wrapper could journal the exact landed OID before invoking
`git commit` while preserving native message,
hook,
and signing behavior.

`sequencer.c:1268` to `sequencer.c:1272` constructs the reflog subject by appending a colon and commit subject to the
action:

```c
if (action) {
        strbuf_addstr(&sb, action);
        strbuf_addstr(&sb, ": ");
}
```

`sequencer.c:1281` to `sequencer.c:1288` records that subject in the same ref transaction that updates `HEAD`:

```c
if (!transaction ||
    ref_transaction_update(transaction, "HEAD", new_head,
                           old_head ? &old_head->object.oid : null_oid(the_hash_algo),
                           NULL, NULL, 0, sb.buf, err) ||
    ref_transaction_commit(transaction, err)) {
```

The Git manual documents the intended extension point at `Documentation/git.adoc:954` to
`Documentation/git.adoc:961`:

```adoc
`GIT_REFLOG_ACTION`::
        When a ref is updated, reflog entries are created to keep
        track of the reason why the ref was updated
```

## Verification

The behavior was reproduced with `/usr/bin/git` 2.54.0 on Linux.
The upstream source was cloned from `https://github.com/git/git.git` at
`f60db8d575adb79761d363e026fb49bddf330c73`.

Run this catalog in a disposable directory:

```sh
fixture="$(mktemp --directory)"
git -C "$fixture" init --quiet --initial-branch=main
git -C "$fixture" config user.email fixture@example.com
git -C "$fixture" config user.name Fixture
printf 'one\n' > "$fixture/file.txt"
git -C "$fixture" add file.txt
GIT_REFLOG_ACTION='cli-git:transaction:first' git -C "$fixture" commit --quiet -m initial
git -C "$fixture" reflog show --format='%H%x00%gs' --max-count=1 HEAD
printf 'two\n' >> "$fixture/file.txt"
git -C "$fixture" add file.txt
GIT_REFLOG_ACTION='cli-git:transaction:second' git -C "$fixture" commit --quiet -m second
git -C "$fixture" reflog show --format='%H%x00%gs' --max-count=1 HEAD
rm --recursive --force "$fixture"
```

Working catalog:

- An initial commit records `cli-git:transaction:first: initial`.
- A later commit records `cli-git:transaction:second: second`.
- `%H%x00%gs` provides an unambiguous OID and subject boundary.
- A private random nonce makes the action specific to one prepared wrapper transaction.

Fail-closed catalog:

- No reflog entry means automatic attribution is unavailable.
- A latest entry whose OID differs from current `HEAD` is conflicting movement.
- A latest entry whose subject lacks the prepared nonce is conflicting movement,
  even when its OID,
  tree,
  and parents match.
- A malformed subject or missing NUL separator is not attributable.

## Verified workarounds

Cli-git records a private random `GIT_REFLOG_ACTION` in its prepared journal,
passes the same value only to the real commit process,
and requires the latest `HEAD` reflog entry to contain both current OID and exact nonce-bearing prefix when no durable
post-ref marker exists.

Tradeoffs:

- Repositories with disabled or expired reflogs cannot recover this ambiguous phase automatically.
- Any later ref update intentionally converts recovery into a conflict,
  even if the ref later returns to the same OID.
- The nonce proves provenance against ordinary concurrent Git activity,
  not against a same-account adversary that can read and deliberately copy private journal data.

Once cli-git has durably written the exact landed-OID marker,
recovery uses that marker instead of reflog provenance.

## What does not work

- Predicting the landed OID before native `git commit` does not preserve message cleanup,
  hooks,
  signing,
and author/committer identity behavior.
- Matching only intended tree and ordered parents accepts a distinct commit with the same shape.
- Looking only at current `HEAD` accepts unrelated movement that returns to an expected OID.
- Treating a missing reflog as successful recovery silently guesses ownership.

## Upstream filing decision

No `.out-of-scope/` entry covers Git reflog behavior.
Open and closed GitHub issues and pull requests were searched for `GIT_REFLOG_ACTION commit reflog action`;
no duplicate appeared.
Git's `.github/CONTRIBUTING.md:3` to `.github/CONTRIBUTING.md:6`
directs bug reports and patches to the mailing list rather
than GitHub.

The six constraints do not justify filing:

1. **Upstream fault**:
   No.
   Git behaves as documented and supplies the required provenance mechanism.
2. **Upstream can fix**:
   Not applicable because no upstream defect remains.
3. **Supported use case**:
   Yes.
   `Documentation/git.adoc` explicitly supports scripted porcelain reflog actions.
4. **Contribution welcome**:
   Yes through Git's mailing-list process,
   but there is no defect to contribute.
5. **Likely fix**:
   Not applicable because no change is requested.
6. **Minimal prototype**:
   The consumer-side nonce and recovery validation are implemented in cli-git;
   an upstream patch would be unrelated to the diagnosed need.

Nothing should be filed upstream.
A new-issue draft would misrepresent documented behavior as a Git defect,
so there is no fileable draft or additive comment.
