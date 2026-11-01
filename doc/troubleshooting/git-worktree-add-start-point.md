# Git 2.55.0 `worktree add` forms can select a commit other than invoking `HEAD`

> Scratch-path note:
> `/tmp/agent` paths in this document are historical.
> Use `~/temp/agent` for current work.

## Symptom

### Git 2.56.0 branch-option recurrence

On 2026-10-18,
creating the Depot graph research worktree with `git worktree add --branch` failed before creation.
Git `2.56.0` returned exit `129` and the exact diagnostic:

```text
error: unknown option `branch'
```

Fresh `git worktree add --help` lists `-b` and `-B` without a long branch-option spelling.
The corrected `-b` form created the isolated worktree at the explicit frozen commit.
A parser-only invocation without a destination reproduced the diagnostic without creating another worktree.
Use the documented short option here;
do not derive a long option from the separate `git branch` command's name.

### Start-point selection

A caller wants to copy ignored files from an existing worktree into every worktree created by
`git worktree add`.
The intuitive rule is that a fresh worktree starts from the invoking worktree's branch,
so both worktrees have the same tracked paths and a copied ignored path cannot collide with a tracked destination path.

That rule is correct for this form:

```console
git worktree add -b <new-branch> <path>
```

It is not a property of every accepted `worktree add` form.
These commands can select another commit,
an existing branch,
a detached commit,
a remote-tracking branch,
or an unborn branch:

```console
git worktree add <path> <commit-ish>
git worktree add -b <new-branch> <path> <commit-ish>
git worktree add --detach <path> <commit-ish>
git worktree add --guess-remote <path>
git worktree add --orphan -b <new-branch> <path>
```

Even the no-start-point `-b` form can encounter an ignored destination entry.
Git runs the shared `post-checkout` hook in the new worktree after checkout,
and that hook can create the same ignored path before a wrapper begins copying.

The rejected reading was too broad:
a target branch may track a source-ignored path only when the accepted command selects a different tree.
For the intended no-start-point `-b` form,
the source and destination tracked trees begin at the same commit.
Only hook output,
a concurrent writer,
or another post-checkout mutation can create a differing destination entry in that form.

## Root cause

The source trace uses Git tag `v2.55.0`,
commit `e9019fcafe0040228b8631c30f97ae1adb61bcdc`.

### Git chooses `HEAD` only when no commit-ish argument exists

`builtin/worktree.c:859` maps the optional positional commit-ish to `branch`.
An omitted value becomes `HEAD`,
while an explicit value is retained:

```c
path = prefix_filename(prefix, av[0]);
branch = ac < 2 ? "HEAD" : av[1];
used_new_branch_options = new_branch || new_branch_force;
```

For `-b` and `-B`,
Git then creates or resets the requested branch at that selected value.
`builtin/worktree.c:932-950` invokes the internal branch command and replaces the checkout target with the new branch:

```c
} else if (new_branch) {
	struct child_process cp = CHILD_PROCESS_INIT;
	cp.git_cmd = 1;
	strvec_push(&cp.args, "branch");
	if (new_branch_force)
		strvec_push(&cp.args, "--force");
	if (opts.quiet)
		strvec_push(&cp.args, "--quiet");
	strvec_push(&cp.args, new_branch);
	strvec_push(&cp.args, branch);
	if (opt_track)
		strvec_push(&cp.args, opt_track);
	if (run_command(&cp))
		return -1;
	branch = new_branch;
```

The corresponding contract in `Documentation/git-worktree.adoc:195-201` is explicit:

```text
With `add`, create a new branch named _<new-branch>_ starting at
_<commit-ish>_, and check out _<new-branch>_ into the new worktree.
If _<commit-ish>_ is omitted, it defaults to `HEAD`.
```

Therefore `git worktree add -b fresh <path>` starts at invoking `HEAD`,
but `git worktree add -b fresh <path> old` starts at `old`.

### The convenience form can choose an existing or remote branch

Without `-b`,
`-B`,
or an explicit commit-ish,
Git derives a branch name from the destination basename.
`builtin/worktree.c:766-785` returns an existing branch or,
when remote guessing is enabled,
a unique tracking branch:

```c
static char *dwim_branch(const char *path, char **new_branch)
{
	int n;
	int branch_exists;
	const char *s = worktree_basename(path, &n);
	char *branchname = xstrndup(s, n);
	struct strbuf ref = STRBUF_INIT;

	branch_exists = !check_branch_ref(&ref, branchname) &&
			refs_ref_exists(get_main_ref_store(the_repository),
					ref.buf);
	strbuf_release(&ref);
	if (branch_exists)
		return branchname;

	*new_branch = branchname;
	if (guess_remote) {
		struct object_id oid;
		char *remote = unique_tracking_name(*new_branch, &oid, NULL);
		return remote;
	}
	return NULL;
}
```

`builtin/worktree.c:887-895` applies that derivation when the command has only a path:

```c
} else if (ac < 2) {
	/* DWIM: Guess branch name from path. */
	char *s = dwim_branch(path, &new_branch_to_free);
	if (s)
		branch = branch_to_free = s;
	new_branch = new_branch_to_free;
```

`Documentation/git-worktree.adoc:91-98` documents the same split:
a missing basename branch is created from `HEAD`,
but an existing basename branch is checked out instead.
`Documentation/git-worktree.adoc:215-225` documents that `--guess-remote` or `worktree.guessRemote` can base the new
branch on a matching remote-tracking branch.

### Checkout hooks run before wrapper post-processing

After creating and checking out the linked worktree,
Git deliberately clears its cleanup marker before running `post-checkout`.
A hook failure leaves the new worktree in place.
`builtin/worktree.c:592-622` shows the ordering and sets the hook working directory to the new path:

```c
if (opts->checkout &&
    (ret = checkout_worktree(opts, &child_env)))
	goto done;

is_junk = 0;
FREE_AND_NULL(junk_work_tree);
FREE_AND_NULL(junk_git_dir);

done:
/* ... */
if (!ret && opts->checkout && !opts->orphan) {
	struct run_hooks_opt opt = RUN_HOOKS_OPT_INIT_FORCE_SERIAL;

	strvec_pushl(&opt.env, "GIT_DIR", "GIT_WORK_TREE", NULL);
	strvec_pushl(&opt.args,
		     oid_to_hex(null_oid(the_hash_algo)),
		     oid_to_hex(&commit->object.oid),
		     "1",
		     NULL);
	opt.dir = path;

	ret = run_hooks_opt(the_repository, "post-checkout", &opt);
}
```

A cli-git copy phase that runs after real Git returns therefore observes hook-created ignored files.

### Git aliases can hide worktree creation from literal argv classification

Git receives the alias name as its initial command and expands it internally.
`git.c:368-455` looks up the first argument,
splits a non-shell alias,
and replaces that argument with the expansion:

```c
alias_command = args->v[0];
alias_string = alias_lookup(alias_command);
/* ... */
count = split_cmdline(alias_string, &new_argv);
/* ... */
/* Replace the alias with the new arguments. */
strvec_splice(args, 0, 1, new_argv, count);
```

A wrapper that checks only whether its raw subcommand is literally `worktree` cannot detect every worktree created by
real Git.
Shell aliases are broader still because Git executes their command through a shell.
Outcome-based detection must compare the effective repository's registered worktrees before and after forwarding.

## Verification

The installed executable reported:

```console
$ /usr/bin/git --version
git version 2.55.0
```

All state-changing probes ran in disposable repositories under `/tmp/agent`.

### Start-point catalog

The harness created two commits,
kept branch `old` at the first commit,
and invoked each worktree form from the second commit.
The relevant command sequence was:

```console
/usr/bin/git -C "$repo" worktree add --quiet -b fresh "$fixture/fresh"
/usr/bin/git -C "$repo" worktree add --quiet "$fixture/existing" old
/usr/bin/git -C "$repo" worktree add --quiet -b based-old "$fixture/based-old" old
/usr/bin/git -C "$repo" worktree add --quiet --detach "$fixture/detached" "$first"
```

Observed object IDs were:

```text
source=440ce058aba55201dbdf5f866eeb9351990c7003
fresh-no-start=440ce058aba55201dbdf5f866eeb9351990c7003
existing-branch=ec53718411d934b68a01871682b0373ad725e625
new-at-explicit-old=ec53718411d934b68a01871682b0373ad725e625
detached-explicit-old=ec53718411d934b68a01871682b0373ad725e625
```

Forms verified to start at invoking `HEAD`:

- `git worktree add -b fresh <path>` with no commit-ish;
- `git worktree add --detach <path>` with no commit-ish.

Forms verified to select another commit:

- `git worktree add <path> old`;
- `git worktree add -b based-old <path> old`;
- `git worktree add --detach <path> <old-oid>`.

Documented forms that do not promise invoking `HEAD`:

- a path-only invocation whose basename is an existing branch;
- a path-only invocation with `--guess-remote` or `worktree.guessRemote`;
- `--orphan`,
   which creates an unborn branch.

### Hook-created collision catalog

A separate fixture committed `.gitignore` with `ignored.txt`,
placed `source-local` in the source worktree's ignored file,
and installed this shared `post-checkout` hook:

```sh
#!/bin/sh
printf 'hook-local\n' > ignored.txt
```

After `git worktree add --quiet -b fresh <path>`,
the destination contained:

```text
destination ignored.txt: hook-local
```

The destination and source still had the same tracked commit.
The differing ignored entry came from the hook,
not from target-branch history.

### Alias-created worktree catalog

A fixture configured this ordinary Git alias:

```console
/usr/bin/git -C "$repo" config alias.newwt 'worktree add -b aliased'
```

The raw invocation contained no literal `worktree` token:

```console
/usr/bin/git -C "$repo" newwt "$wt"
```

Git created the linked worktree and `symbolic-ref --short HEAD` printed:

```text
aliased
```

## Verified workarounds

### Restrict guaranteed same-tree copying to no-start-point branch creation

A wrapper can recognize `git worktree add -b <new-branch> <path>` with no commit-ish,
record invoking `HEAD` before forwarding,
and compare it with the destination `HEAD` after Git succeeds.

```console
source_oid=$(/usr/bin/git rev-parse HEAD)
/usr/bin/git worktree add -b topic ../topic
created_oid=$(/usr/bin/git -C ../topic rev-parse HEAD)
test "$source_oid" = "$created_oid"
```

Tradeoff:
this guarantee excludes explicit start points,
existing-branch worktrees,
remote guessing,
and orphan worktrees.

### Query the created worktree instead of inferring its tree

For every successful `worktree add`,
a wrapper can resolve the created path and ask real Git for its actual `HEAD` and tracked paths before copying.
This supports explicit start points and existing branches without pretending they match the source.

Tradeoff:
the copy algorithm needs a collision policy when source and destination tracked trees differ.
An orphan worktree has no commit OID,
so it needs an explicit absence state rather than a failed assumption.

### Treat post-checkout output as existing destination state

Run copying only after real Git and its `post-checkout` hook return.
Accept byte-identical entries,
but do not overwrite a differing entry without an explicit settled policy.

Tradeoff:
a collision can be reported only after Git has created the worktree,
because the hook does not run until checkout is complete.
Git itself intentionally retains that worktree when the hook fails.

## What does not work

### Assume every `worktree add` starts at invoking `HEAD`

The explicit commit-ish,
existing-branch,
remote-guessing,
detached,
and orphan forms disprove this rule.
It is safe only after classifying the accepted argv shape.

### Dismiss all collision handling for the no-start-point `-b` form

Matching tracked trees remove the target-history collision,
but the verified `post-checkout` fixture still creates a differing ignored destination file.
A concurrent process can do the same between checkout and copying.

### Trigger only on a literal `worktree add` argv sequence

The verified `newwt` alias creates the same worktree without exposing `worktree add` to a wrapper's initial argument
parser.
Literal classification misses aliases and any other forwarded command that creates a linked worktree as a side effect.

### Copy before Git creates the worktree

Git normally expects to own worktree creation and checkout.
Pre-populating the path can make native creation fail,
and it cannot account for files produced by checkout or `post-checkout`.

### Overwrite destination entries unconditionally

This can erase hook output or dirty a worktree whose selected commit differs from the source.
It also removes the evidence needed to diagnose which producer created the collision.

## Upstream filing artifact

### Upstream filing decision

The `.out-of-scope/` inventory contains no Git or generic version-control exemption.
Searches of open and closed GitHub issues and pull requests in `git/git` for
`worktree add HEAD start point` returned no matching thread.
Git development primarily uses its mailing list,
but no report is warranted because the observed behavior matches the documented contract.

The filing constraints resolve as follows:

1. **Is it really upstream's fault?
   ** No.
   Git documents each start-point form and its `post-checkout` ordering is deliberate.
2. **Can upstream fix it?
   ** Not applicable as a defect.
   Changing all forms to start at invoking `HEAD` would remove documented selection features.
3. **Are they supporting this use case?
   ** Yes.
   `Documentation/git-worktree.adoc` specifies branch creation,
   explicit commit-ish selection,
   remote guessing,
   detached worktrees,
   and orphan worktrees.
4. **Would the repository welcome our contribution?
   ** No for an AI-produced filing or patch.
   `Documentation/SubmittingPatches:500-524` permits careful AI guidance but says content that looks AI-generated will
   be rejected.
   The project otherwise accepts mailing-list patches under its contribution process.
5. **Will they likely fix it?
   ** No fix is requested.
   The behavior is intentional and no matching tracker item was found.
6. **Have we prototyped a compatible minimal fix?
   ** Not applicable.
   There is no upstream defect to patch;
   the consumer wrapper must classify command forms and post-checkout state.

Nothing should be filed upstream.
There is no defect report or additive comment to draft.
