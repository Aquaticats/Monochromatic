# Git 2.56.0 reads a caller's pathspec mode into a tool's own generated pathspecs, and `git ls-tree` refuses `GIT_ICASE_PATHSPECS=1` outright

## Symptom

A program that generates its own pathspecs and hands them to Git selects paths the batch never named
when the caller's shell sets a pathspec mode.
The git wrapper in `package/git-policy/cli` met three distinct failures,
all from one cause.

A service started from a shell holding `GIT_ICASE_PATHSPECS=1` and `GIT_LITERAL_PATHSPECS=1` fails every
landing with:

```text
fatal: :(top,literal)a.txt: pathspec magic not supported by this command: 'icase'
```

read by the wrapper as
`the tree the landing of batch 1 replaced cannot be read (Unavailable)`.

A service started from a shell holding only `GIT_ICASE_PATHSPECS=1` silently replaces an index entry
the batch never selected:
`git reset -- <the wrapper's own :(top,literal)a.txt>`
resets the staged entry of `A.txt` too,
and `git rm --cached -r --force -- <the same spec>`
removes `A.txt` from the commit's private index,
so the commit records that path as a deletion.
Neither command says anything.

The same variable alone also makes any `:(top,literal)` listing name both `a.txt` and `A.txt`.

## Symptom: names come back relative to the directory Git runs in

A program that writes `:(top,literal)<path>` pathspecs and then looks its results up by that path
misses entries when the caller's own `-C` moved Git below the top.
`git ls-files` reports every name relative to the directory Git runs in unless it is given
`--full-name`,
while `ls-tree`,
`diff-tree` and `diff-index` report repository-relative names from anywhere.
The mismatch is silent:
a lookup for `sub/f` finds nothing where the listing said `f`,
which the repair in `package/git-policy/cli` read as "the index holds no entry there" and answered
by resetting the commit's own entry to `HEAD`,
so the person's deliberately staged contents were left out of the commit.

## Root cause

Git adds four *global* pathspec magics from the environment and from global options,
and a global magic lands on every pathspec element the program passes.

`pathspec.c:296-320` at tag `v2.56.0`:

```c
static inline int get_icase_global(void)
{
	static int icase = -1;

	if (icase < 0)
		icase = git_env_bool(GIT_ICASE_PATHSPECS_ENVIRONMENT, 0);

	return icase;
}

static int get_global_magic(int element_magic)
{
	int global_magic = 0;

	if (get_literal_global())
		global_magic |= PATHSPEC_LITERAL;

	/* --glob-pathspec is overridden by :(literal) */
	if (get_glob_global() && !(element_magic & PATHSPEC_LITERAL))
		global_magic |= PATHSPEC_GLOB;

	if (get_glob_global() && get_noglob_global())
		die(_("global 'glob' and 'noglob' pathspec settings are incompatible"));

	if (get_icase_global())
		global_magic |= PATHSPEC_ICASE;

	if ((global_magic & PATHSPEC_LITERAL) &&
	    (global_magic & ~PATHSPEC_LITERAL))
		die(_("global 'literal' pathspec setting is incompatible "
		      "with all other global pathspec settings"));

	/* --noglob-pathspec adds :(literal) _unless_ :(glob) is specified */
	if (get_noglob_global() && !(element_magic & PATHSPEC_GLOB))
		global_magic |= PATHSPEC_LITERAL;

	return global_magic;
}
```

`pathspec.c:468` puts that global magic on the element,
so a `:(top,literal)` spec is not literal on its
own:

```c
		magic |= get_global_magic(element_magic);
```

`:(literal)` overrides the *glob* global (`pathspec.c:307`) and never touches `PATHSPEC_ICASE`,
which is why the reset and the prune above match case-insensitively.

The refusal is a separate step of the same function pair:
`pathspec.c:657-659` compares each element's magic with the mask the command passes,
and `pathspec.c:581-593` dies naming the magic that is left.

```c
		if (item[i].magic & magic_mask)
			unsupported_magic(entry, item[i].magic & magic_mask);
```

`builtin/ls-tree.c:420-422` allows `FROMTOP` and `LITERAL` only:

```c
	parse_pathspec(&options.pathspec, PATHSPEC_ALL_MAGIC &
		       ~(PATHSPEC_FROMTOP | PATHSPEC_LITERAL),
		       PATHSPEC_PREFER_CWD,
```

so the inherited `PATHSPEC_ICASE` is unsupported there,
while `ls-files` accepts it.

An earlier reading of this repository's review called the reset the whole defect.
That was incomplete:
the review's own sequence sets both variables,
and with both set `git ls-tree` refuses every pathspec before any reset runs.
The silent replacement needs `GIT_ICASE_PATHSPECS=1` alone.

## Verification

Git 2.56.0,
`/usr/bin/git --version` reports `git version 2.56.0` on this host on 2026-11-09.
The fixture is a repository holding committed `a.txt` and `A.txt`,
with a later commit changing `a.txt`
only;
every command below runs in it.
A second fixture holds `sub/f`,
and the name-base runs below happen inside `sub` with the spec `:(top,literal)sub/f`,
as a caller's `-C sub` makes them.

Patterns that work cleanly:

- `GIT_ICASE_PATHSPECS=1 git ls-files -s -z -- ':(top,literal)a.txt'`
  lists both `A.txt` and `a.txt` and exits 0.
- `GIT_LITERAL_PATHSPECS=1 GIT_ICASE_PATHSPECS=1 git reset --no-refresh <commit> -- ':(top,literal)a.txt'`
  exits 128 with `fatal: global 'literal' pathspec setting is incompatible with all other global pathspec settings`.
- `GIT_GLOB_PATHSPECS=1 GIT_NOGLOB_PATHSPECS=1 git ls-files -- a.txt`
  exits 128 with `fatal: global 'glob' and 'noglob' pathspec settings are incompatible`,
  and adding `--noglob-pathspecs` leaves it fatal,
  because the flag sets the same variable.

Names as each command reports them,
run inside `sub` with the spec `:(top,literal)sub/f`:

- `git ls-files -s -z` names `f`.
- `git ls-files -s -z --full-name` names `sub/f`.
- `git ls-tree -z --full-tree <commit>` names `sub/f`.
- `git diff-tree -r --name-only -z <old> <new>` names `sub/f`.
- `git diff-index --cached --name-only -z <commit>` names `sub/f`.

Patterns that fail:

- `GIT_ICASE_PATHSPECS=1 git ls-tree -z --full-tree HEAD -- ':(top,literal)a.txt'`
  exits 128 with `fatal: :(top,literal)a.txt: pathspec magic not supported by this command: 'icase'`.
- `GIT_ICASE_PATHSPECS=1 git ls-files -- ':(top,literal)a.txt'` lists `A.txt` and `a.txt`.
- `GIT_ICASE_PATHSPECS=1 git rm --cached -r --force -- ':(top,literal)a.txt'`
  removes both entries from the index it is given.

The consumer-side harness is the wrapper's own tests,
which reproduce each failure before the fix and pass
after it:

- `mise run //package/git-policy/cli:native:test:host -- case_twin`
- `mise run //package/git-policy/cli:native:test:host -- a_repair_removes_no_staged_entry_of_a_case_twin the_entries_of_a_path`

## Verified workarounds

A mode reaches a generated pathspec twice,
so both halves have to go:
as one of the four variables in the child's environment,
and as a global option the request forwards before its own words.

The wrapper declares the request's pathspecs its own once,
with `GitRequest::own_pathspecs`,
and that does both:
it appends the four names `conflict_side_modes::MODE_VARIABLES` already holds to the request's `unset` list,
and it marks the request,
so `GitRequest::full_arguments` takes the caller's global options through
`global_arguments::without_pathspec_mode_options`,
which drops `conflict_side_modes::MODE_OPTIONS`
and keeps the value a separated-value option consumes,
so `-C --icase-pathspecs` still changes into that directory.

Every start of a request goes through `full_arguments`:
the started command and the in-process reader both take their argument list from it,
so no route chooses between the two halves itself.

A listing whose names are looked up as repository-relative paths asks for them:
`ls-files --full-name`.
A listing that runs through `git_metadata`'s runner instead,
which has no `GitRequest`,
takes `run_metadata_git_without`,
so the named variables are removed before the start and the in-process readers stay out of the way,
because a reader would judge the caller's mode.

Tradeoffs:
a `:(top,literal)` spec in a mode-free environment is the path itself and nothing else,
so a tool that wanted the caller's mode for its own spec cannot have both;
and the caller's own command still keeps its mode,
because only the tool's internal starts are changed.

Global options alone are `git.c:282-300` setting the variables,
so dropping the option and removing the variable leave the same state,
and no `--no-...` flag has to be written after the forwarded prefix.

## What does not work

- `--no-icase-pathspecs` is an unknown global:
  `git --no-icase-pathspecs --version` prints `unknown option: --no-icase-pathspecs` and the usage line.
  `git.c:282-300` has a negating form for the literal variable only.
- Adding `--noglob-pathspecs` to defeat `GIT_GLOB_PATHSPECS=1` is fatal,
  because the flag sets the
  variable the same value the caller's variable sets.
- Relying on `:(top,literal)` per-element magic to keep matching case-sensitive:
  the element magic is the
  union of the spec's own magic and the global magic,
  so literal never removes icase.
- Setting a variable to `0` works on 2.56.0 measured for all four,
  but leaves a value in the child's
  environment that a future Git release may read differently;
  removal is what the wrapper does.

## Upstream filing decision

Nothing filed.
The six constraints walked:

1. Upstream's fault:
   partly.
   Git documents the mode variables and the two-magic refusals,
   and `ls-tree`'s
   mask is a deliberate per-command capability list.
   Only the *lack of a negating option for the icase
   variable* reads as a gap.
2. Upstream could fix it:
   yes,
   a `--no-icase-pathspecs` setting the variable to `0` beside `git.c:286-289`.
3. Supporting this use case:
   not shown.
   The variables are described as settings for the user's own
   pathspecs,
   not for programs that generate them.
4. Welcome a contribution:
   not checked against `Documentation/SubmittingPatches` and the list's policy for
   this run.
5. Likely to fix:
   no signal either way;
   the tracker was not searched for this run.
6. Prototyped fix:
   none.
   The consumer-side fix above removes the need for the wrapper.

Constraints 3 to 6 are unverified,
so the default policy of filing nothing applies.
No link in this doc was loaded in `agent-browser`;
each cites a file at tag `v2.56.0` read with
`gh api 'repos/git/git/contents/<path>?ref=v2.56.0'`.
