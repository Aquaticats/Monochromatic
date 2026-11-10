# Git 2.55.0 hands hooks a relative `GIT_WORK_TREE=.` and ignores command-line `core.worktree` for explicit git dirs

This document covers two related quirks found while driving `git commit` against a private admin dir
(a directory holding its own `HEAD` and a `commondir` file that points at the real `.git`)
together with a private index:

- Behavior B:
  with `--work-tree=<abs>` or `GIT_WORK_TREE=<abs>`,
  hooks inherit `GIT_WORK_TREE=.`,
  so a hook that changes into a subdirectory and runs Git gets the subdirectory as its toplevel.
- Behavior C:
  `core.worktree` supplied through `git -c`,
  `GIT_CONFIG_COUNT`,
  or the private admin dir's own `config` file is ignored,
  and `git -c extensions.worktreeConfig=true` does not enable `config.worktree`;
  only `extensions.worktreeConfig` in the real common config plus `<admin>/config.worktree` sets the work tree.

The hook-disabling quirks found in the same probe
(`hook.<event>.enabled=false` skipping only configured hooks,
and `core.hooksPath=/dev/null` skipping only hookdir hooks)
live in [`git-hook-disable-switches.md`](git-hook-disable-switches.md).

Both behaviors here are working as designed;
neither is filed upstream.
See the "Upstream filing decision" sections.

## Symptom

### Behavior B: hook in a subdirectory sees the wrong toplevel

No error is printed.
A `pre-commit` hook that runs `cd sub` and then `git rev-parse --show-toplevel`
prints `<repo>/sub` instead of `<repo>`,
`git rev-parse --show-prefix` prints an empty string instead of `sub/`,
and `git status` or `git ls-files` report every tracked path relative to `sub`
(so files outside `sub` look deleted and files inside `sub` look untracked).

Triggering invocations (all from `<repo>` or any subdirectory):

```sh
git --git-dir=<abs .git or admin dir> --work-tree=<abs repo> commit ...
GIT_DIR=<abs> GIT_WORK_TREE=<abs repo> git commit ...
```

The hook environment shows the rewrite:

```text
cwd=<repo>
GIT_DIR=<repo>/.git
GIT_WORK_TREE=.
[in sub] show-toplevel=<repo>/sub
[in sub] show-prefix=
[in sub] ls-files=a.txt sub/b.txt
```

`GIT_DIR=<abs>` with no work tree at all
(neither `--work-tree`,
 `GIT_WORK_TREE`,
 nor an honoured `core.worktree`)
produces the same subdirectory view,
but through a different,
 documented path:
Git regards the current directory as the work tree top.

### Behavior C: `core.worktree` from the wrong source is silently dropped

No error or warning is printed.
`git rev-parse --show-toplevel`,
run from `<repo>/sub` with `--git-dir=<private admin dir>`,
prints `<repo>/sub` (the current directory) for each of these:

```sh
git --git-dir=<adm> -c core.worktree=<repo> rev-parse --show-toplevel
GIT_DIR=<adm> GIT_CONFIG_COUNT=1 GIT_CONFIG_KEY_0=core.worktree GIT_CONFIG_VALUE_0=<repo> git rev-parse --show-toplevel
GIT_DIR=<adm> git rev-parse --show-toplevel                  # with core.worktree in <adm>/config
GIT_DIR=<adm> git -c extensions.worktreeConfig=true rev-parse --show-toplevel   # with <adm>/config.worktree
git --git-dir=<repo>/.git -c core.worktree=<repo> rev-parse --show-toplevel   # plain .git, no commondir
```

The last line shows the `-c` part is not specific to private admin dirs:
`core.worktree` from the command line is never consulted for work tree discovery.
When a hook later runs from the repository root the top-level command still succeeds,
so the symptom usually surfaces only in hooks or in commands started from a subdirectory,
exactly like Behavior B.

## Root cause

Source:
 Git `v2.55.0` (`e9019fcafe0040228b8631c30f97ae1adb61bcdc`),
cloned with `gh repo clone git/git -- --depth 1 --branch v2.55.0`.
`master` at `0f8e75ab` (2026-09-23) keeps both behaviors:
`setup_work_tree()` still exports `.` (`setup.c:505-510` there),
and a discovery refactor renames `check_repository_format_gently()` to
`read_and_verify_repository_format()` and moves `git_work_tree_cfg` into a `repo_discovery` struct,
but still takes `core.worktree` only from the on-disk repository format read
and still drops it when a `commondir` exists
(new comment:
"core.worktree" is supposed to be ignored when we have a commondir configured,
unless it comes from the per-worktree configuration).

### Behavior B: `setup_work_tree()` rewrites `GIT_WORK_TREE` to `.`

`--work-tree` is turned into the environment variable before setup,
`git.c:242-247`:

```c
// git.c (handle_options)
} else if (!strcmp(cmd, "--work-tree")) {
	...
	setenv(GIT_WORK_TREE_ENVIRONMENT, (*argv)[1], 1);
```

`git commit` is registered with `NEED_WORK_TREE` (`git.c:556`),
so `git.c:499-500` calls `setup_work_tree()`,
which changes into the work tree root and then overwrites the variable with `.`,
`setup.c:496-513`:

```c
// setup.c
void setup_work_tree(struct repository *repo)
{
	...
	work_tree = repo_get_work_tree(repo);
	if (!work_tree || chdir_notify(work_tree))
		die(_("this operation must be run in a work tree"));

	/*
	 * Make sure subsequent git processes find correct worktree
	 * if $GIT_WORK_TREE is set relative
	 */
	if (getenv(GIT_WORK_TREE_ENVIRONMENT))
		setenv(GIT_WORK_TREE_ENVIRONMENT, ".", 1);
}
```

Hooks inherit the process environment and current directory.
`run_commit_hook()` only adds `GIT_INDEX_FILE` and `GIT_EDITOR` (`commit.c:1994-2015`),
and `pick_next_hook()` leaves the child's directory at the caller's value,
`hook.c:609`:

```c
// hook.c (pick_next_hook)
cp->dir = hook_cb->options->dir;
```

For commit hooks `options->dir` is unset,
so the hook starts in the work tree root with `GIT_WORK_TREE=.`.
That pair is consistent;
it stops being consistent the moment the hook changes directory,
because `.` is then resolved against the new directory.

The `.` is a deliberate choice.
Commit `0ed748134748` ("setup_work_tree:
 adjust relative $GIT_WORK_TREE after moving cwd",
Nguyễn Thái Ngọc Duy,
 2010-12-27) says:

```text
Instead of making $GIT_WORK_TREE absolute too, we just say "." and let
subsequent git processes handle it.
```

The hook contract in `Documentation/githooks.adoc:24-35` promises the starting directory
("the root of the working tree in a non-bare repository")
and that exported variables such as `GIT_WORK_TREE` let Git commands run by the hook locate the repository;
it does not promise the variables survive a `cd`.

The `GIT_DIR`-only case is documented separately in `Documentation/config/core.adoc:336-339`:

```text
If --git-dir or GIT_DIR is specified but none of
--work-tree, GIT_WORK_TREE and core.worktree is specified,
the current working directory is regarded as the top level
of your working tree.
```

### Behavior C: work tree discovery reads `core.worktree` only from on-disk repository files

With an explicit git dir,
`setup_explicit_git_dir()` takes the work tree from `GIT_WORK_TREE`,
else from `git_work_tree_cfg`,
else from the current directory,
`setup.c:1141-1179`:

```c
// setup.c (setup_explicit_git_dir)
/* #3, #7, #11, #15, #19, #23, #27, #31 (see t1510) */
if (work_tree_env)
	set_git_work_tree(repo, work_tree_env);
...
else if (git_work_tree_cfg) { /* #6, #14 */
	if (is_absolute_path(git_work_tree_cfg))
		set_git_work_tree(repo, git_work_tree_cfg);
	...
}
...
else /* #2, #10 */
	set_git_work_tree(repo, ".");
```

`git_work_tree_cfg` is filled only by `check_repository_format_gently()`,
`setup.c:753-809`:

```c
// setup.c
has_common = get_common_dir(&sb, gitdir);
strbuf_addstr(&sb, "/config");
read_repository_format(candidate, sb.buf);
...
if (candidate->worktree_config) {
	/*
	 * pick up core.bare and core.worktree from per-worktree
	 * config if present
	 */
	strbuf_addf(&sb, "%s/config.worktree", gitdir);
	git_config_from_file(read_worktree_config, sb.buf, candidate);
	strbuf_release(&sb);
	has_common = 0;
}

if (!has_common) {
	...
	if (candidate->work_tree) {
		free(git_work_tree_cfg);
		git_work_tree_cfg = xstrdup(candidate->work_tree);
	}
}
```

and `read_repository_format()` parses exactly one file,
`setup.c:859-863`:

```c
// setup.c
int read_repository_format(struct repository_format *format, const char *path)
{
	clear_repository_format(format);
	format->hash_algo = GIT_HASH_SHA1_LEGACY;
	git_config_from_file(check_repo_format, path, format);
```

Walking each source through that code:

- `-c core.worktree` and `GIT_CONFIG_COUNT`:
  command-line and environment config are applied by `git_config_from_parameters()`
  only in the full config sequence (`config.c:1601-1602`),
  which work tree discovery never calls.
  The value lands in the ordinary config set,
  where nothing reads `core.worktree` again
  (`rg 'core\.worktree'` over `*.c` finds the only discovery reader at `setup.c:595`).
- `<adm>/config`:
  because `<adm>/commondir` exists,
  `get_common_dir()` redirects the read to `<commondir>/config`,
  so the admin dir's own `config` is never opened.
  The runtime config sequence does the same (`config.c:1563-1565` reads `<commondir>/config`
  and `<gitdir>/config.worktree`,
   never `<gitdir>/config`).
- `core.worktree` in `<commondir>/config`:
  read,
  but `has_common` is still set,
  so it is dropped;
  `Documentation/git-worktree.adoc:328-331` documents that such values
  "will be applied to the main worktree only".
- `-c extensions.worktreeConfig=true`:
  `candidate->worktree_config` comes from `handle_extension_v0()` (`setup.c:629-630`)
  called while parsing the on-disk `<commondir>/config`,
  so a command-line value never sets it and `config.worktree` is not read.
- `extensions.worktreeConfig=true` in `<commondir>/config` plus `<adm>/config.worktree`:
  the `candidate->worktree_config` branch reads `config.worktree`,
  clears `has_common`,
  and `git_work_tree_cfg` is set.
  This is the one working layout.

`Documentation/git.adoc:77-79` says a `-c` value "will override values from configuration files",
and the `core.worktree` entry (`Documentation/config/core.adoc:327-339`) lists only
`GIT_WORK_TREE` and `--work-tree` as overrides without saying command-line config is ignored.
That is a wording gap,
not a behavior bug:
repository format and discovery keys have to come from the repository itself.

## Verification

Verified 2026-09-25 against Fedora `git-core-2.55.0-1.fc44.x86_64` (`/usr/bin/git`,
 `git version 2.55.0`),
source-traced at tag `v2.55.0` (`e9019fcafe0040228b8631c30f97ae1adb61bcdc`).
The `git` on this repo's `PATH` is a wrapper,
so each run pinned `/usr/bin/git` and put it first on the hooks' `PATH`,
with `GIT_CONFIG_GLOBAL=/dev/null` and `GIT_CONFIG_NOSYSTEM=1`,
inside a disposable `mktemp --directory "${HOME}/temp/agent/..."` directory that was removed afterwards.
The full driver was a Node TypeScript harness (cases B1 to B6 and C0 to C6 below);
the shell snippets reproduce the same results by hand.

### Behavior B harness

```sh
# Behavior B: run in an empty scratch directory with Git 2.55.0 first on PATH
export GIT_CONFIG_GLOBAL=/dev/null GIT_CONFIG_NOSYSTEM=1
git init --quiet --initial-branch=main repo && cd repo
git config user.name Probe && git config user.email probe@example.invalid
mkdir sub && echo b >sub/b.txt && echo a >a.txt
git add --all && git commit --quiet --message=base
printf '%s\n' '#!/bin/sh' \
  'echo "GIT_WORK_TREE=${GIT_WORK_TREE-<unset>}"' \
  'cd sub && echo "toplevel from sub: $(git rev-parse --show-toplevel)"' >.git/hooks/pre-commit
chmod +x .git/hooks/pre-commit
echo a2 >a.txt && git add a.txt
git commit --quiet --message=plain
echo a3 >a.txt && git add a.txt
git --git-dir="$PWD/.git" --work-tree="$PWD" commit --quiet --message=explicit
```

Expected output (the hook prints to stderr):

```text
GIT_WORK_TREE=<unset>
toplevel from sub: <scratch>/repo
GIT_WORK_TREE=.
toplevel from sub: <scratch>/repo/sub
```

### Behavior C harness

```sh
# Behavior C: run in an empty scratch directory with Git 2.55.0 first on PATH
export GIT_CONFIG_GLOBAL=/dev/null GIT_CONFIG_NOSYSTEM=1
git init --quiet --initial-branch=main repo
git -C repo -c user.name=Probe -c user.email=probe@example.invalid commit --quiet --allow-empty --message=base
mkdir repo/sub priv-admin
R="$PWD/repo" A="$PWD/priv-admin"
git -C "$R" rev-parse HEAD >"$A/HEAD"
echo "$R/.git" >"$A/commondir"
cd "$R/sub"
git --git-dir="$A" --work-tree="$R" rev-parse --show-toplevel
git --git-dir="$A" -c core.worktree="$R" rev-parse --show-toplevel
printf '[core]\n\tworktree = %s\n' "$R" >"$A/config.worktree"
git --git-dir="$A" -c extensions.worktreeConfig=true rev-parse --show-toplevel
git -C "$R" config extensions.worktreeConfig true
git --git-dir="$A" rev-parse --show-toplevel
```

Expected output:

```text
<scratch>/repo
<scratch>/repo/sub
<scratch>/repo/sub
<scratch>/repo
```

### Invocations that give hooks a usable work tree

- B1 plain `git commit` with discovery:
  no `GIT_DIR` or `GIT_WORK_TREE` exported;
  the hook sees `show-toplevel=<repo>` and `show-prefix=sub/` from `sub`.
- B5 `--git-dir` plus `--work-tree`,
  with the hook pinning `GIT_WORK_TREE` to an absolute path before any `cd`
  (workaround 1 below):
  `GIT_WORK_TREE=<repo>`,
  `show-toplevel=<repo>`,
  `show-prefix=sub/`.
- B6 and C5 `GIT_DIR=<adm>` with `<adm>/config.worktree` and
  `extensions.worktreeConfig=true` in the common config
  (workaround 2 below):
  no `GIT_WORK_TREE` exported;
  `show-toplevel=<repo>` from `sub`,
  also when the outer command starts in `<repo>/sub`.
- C0 `--git-dir=<adm> --work-tree=<repo>`:
  the outer command resolves `<repo>` (its hooks still see `.`).

### Invocations that break subdirectory Git calls in hooks

Relative `GIT_WORK_TREE=.` (Behavior B):

- B2 `git --git-dir=<abs> --work-tree=<abs> commit`
- B3 `GIT_DIR=<abs> GIT_WORK_TREE=<abs> git commit`
- The earlier probe also saw it when the outer command starts in `<repo>/sub`
  (`GIT_PREFIX=sub/`,
   hook still started in `<repo>` with `GIT_WORK_TREE=.`).

Work tree silently falls back to the current directory (Behavior C,
 plus the documented `GIT_DIR`-only case):

- B4 `GIT_DIR=<abs>` only (documented fallback)
- C1 `--git-dir=<adm> -c core.worktree=<abs>`
- C2 `GIT_DIR=<adm>` with `GIT_CONFIG_COUNT` setting `core.worktree`
- C3 `GIT_DIR=<adm>` with `core.worktree` in `<adm>/config`
- C4 `GIT_DIR=<adm> -c extensions.worktreeConfig=true` with `<adm>/config.worktree`
- C6 `--git-dir=<repo>/.git -c core.worktree=<abs>` (no `commondir`;
   shows `-c` is ignored everywhere)

When the outer command starts in `<repo>/sub` under C1,
the hook itself starts in `<repo>/sub`,
and a hook that runs `cd sub` fails with `cd: sub: No such file or directory`,
aborting the commit.

## Verified workarounds

### Workaround 1: pin `GIT_WORK_TREE` to an absolute path at the top of each hook

```sh
# .git/hooks/<event>, first lines after the shebang
if [ -n "${GIT_WORK_TREE+set}" ]; then
  GIT_WORK_TREE=$(cd -- "$GIT_WORK_TREE" && pwd -P) && export GIT_WORK_TREE
fi
```

Verified as case B5.
Tradeoffs:
it needs an edit in every hook,
or a `core.hooksPath` shim directory whose scripts pin the variable and then `exec` the real hook
(the earlier probe verified that shape as case F11,
and a dispatcher variant that also routes configured hooks through `git hook run`);
`pwd -P` resolves symlinks,
so paths the hook prints may differ from the caller's spelling (`/home` versus `/var/home` here).
A hook the caller does not control keeps the relative value.

### Workaround 2: give the private admin dir its work tree through `config.worktree`

```sh
# once per repository (changes the repository config for every Git on the machine)
git -C <repo> config extensions.worktreeConfig true
# per private admin dir
printf '[core]\n\tworktree = %s\n' <abs repo> ><adm>/config.worktree
GIT_DIR=<adm> GIT_INDEX_FILE=<private index> git commit ...
```

Verified as cases B6 and C5,
starting the commit from `<repo>/sub`.
Because no `--work-tree` or `GIT_WORK_TREE` is passed,
`setup_work_tree()` leaves the environment alone and hooks rediscover the work tree from `GIT_DIR`.
Tradeoffs:
`extensions.worktreeConfig` is a repository format extension,
so "Older Git versions will refuse to access repositories with this extension"
(`Documentation/git-worktree.adoc:342-343`);
`Documentation/config/extensions.adoc:129-138` requires moving any `core.worktree`
or `core.bare=true` from the common config into the main worktree's `config.worktree`;
and the flag changes what `git config --worktree` writes for every worktree.
The earlier probe confirmed `core.repositoryformatversion` stayed `0`
and the main worktree still resolved correctly after setting it.

### Workaround 3: avoid `cd` in hooks

Resolve paths from the hook's starting directory instead of changing into a subdirectory
(for example `git ls-files -- sub/` from the root).
Tradeoff:
only possible for hooks the caller writes;
third-party hook frameworks that `cd` internally still break.

## What does not work

- `git -c core.worktree=<abs>` or `GIT_CONFIG_COUNT`/`GIT_CONFIG_PARAMETERS` carrying `core.worktree`:
  never read by discovery (cases C1,
   C2,
   C6).
- `core.worktree` in `<adm>/config`:
  the file is never opened once `commondir` exists (case C3).
- `git -c extensions.worktreeConfig=true`:
  extensions come only from the on-disk common config (case C4).
- Passing an absolute `--work-tree` or `GIT_WORK_TREE`:
  `setup_work_tree()` overwrites it with `.` before hooks run (cases B2,
   B3).
- Relying on `git hook run` after the commit lands:
  it runs with a clean environment but against the real repository state,
  not the private index or admin dir `HEAD`,
  so it answers a different question (earlier probe,
   point 3).

## Upstream filing artifact

### Upstream filing decision

`.out-of-scope/` has no entry for Git
(checked `bun-install.md`,
`cargo-workspace.md`,
`claude-code-upstream-bugs.md`,
`codex-harness.md`,
`jsr.md`,
`lightningcss.md`,
`low-impact-typescript-formatting.md`,
`module-es-monolith.md`,
`pi-gpt55-long-context.md`,
`terminal-title-fork-parity-tests.md`,
`typescript-project-references.md`),
so the constraint check applies.
Git takes bug reports on `git@vger.kernel.org`,
not GitHub issues (`.github/CONTRIBUTING.md`,
 `README.md:32-36`).
Duplicate search:
`gh search prs --repo git/git` and `--repo gitgitgadget/git` for hook and worktree terms,
and web searches of the list archives for "git -c core.worktree" and the `setup_work_tree` rewrite,
found no report of either behavior;
`lore.kernel.org` itself rejected automated access (Anubis challenge),
so the archive search went through indexing mirrors (`ratatoskr.run`,
 `public-inbox.org`).

#### Behavior B

1.  **Is it really upstream's fault?**
    No.
    The rewrite to `.` is deliberate (`0ed748134748`),
    correct at the directory where Git starts the hook,
    and `Documentation/githooks.adoc:24-35` defines that starting directory.
    The breakage needs the hook to change directory while keeping a relative variable.
2.  **Can upstream fix it?**
    Yes:
    `setup.c:511-512` could export the absolute work tree instead of `.`.
3.  **Are they supporting this use case?**
    Partly.
    Explicit `--git-dir`/`--work-tree` is supported (`t/t1501-work-tree.sh`),
    but no doc or test covers a hook changing directory under it.
4.  **Would the repo welcome our contribution?**
    Conditionally:
    `Documentation/SubmittingPatches:499-525` rejects content that "looks AI generated"
    or that senders "don't understand or cannot explain",
    so any report must be written and explained by the human sender.
5.  **Will they likely fix it?**
    Unclear;
    the 2010 commit chose `.` over an absolute path on purpose and gave no reason against the absolute form.
6.  **Have we prototyped a minimal fix compatible with their architecture?**
    No.
    The auto-prototype step is not triggered because constraint 1 fails.

Decision:
do not file.

#### Behavior C

1.  **Is it really upstream's fault?**
    No for the behavior:
    work tree discovery and repository format extensions have to come from the repository's own files,
    and the linked-worktree rules are documented in `Documentation/git-worktree.adoc:326-359`.
    The only upstream-side gap is wording:
    neither `Documentation/git.adoc:77-81` nor `Documentation/config/core.adoc:327-339`
    says command-line config cannot set `core.worktree`.
2.  **Can upstream fix it?**
    The wording,
     yes;
    honouring `-c core.worktree` would need discovery to parse command-line config earlier,
    which is a design change.
3.  **Are they supporting this use case?**
    No:
    a hand-built admin dir with a `commondir` file is an internal layout,
    not a documented interface.
4.  **Would the repo welcome our contribution?**
    Conditionally,
     as for Behavior B.
5.  **Will they likely fix it?**
    A one-sentence doc note is plausible,
    but it does not change anything for us.
6.  **Have we prototyped a minimal fix compatible with their architecture?**
    No;
    not triggered because constraint 1 fails for the behavior and constraint 3 fails for the layout.

Decision:
do not file.
Nothing to add to any existing thread.

### Draft (do not file as-is)

Kept as an auditable record for Behavior B only,
in case upstream signal changes (for example a hook framework report on the list).
Git reports go to `git@vger.kernel.org` as plain-text email;
per `Documentation/SubmittingPatches:499-525` a human must rewrite this in their own words before sending.

~~~md
Subject: hooks inherit GIT_WORK_TREE=. when git is run with --work-tree

When a command that needs a work tree is run as

    git --git-dir=/abs/repo/.git --work-tree=/abs/repo commit

setup_work_tree() (setup.c:511-512 in v2.55.0) rewrites GIT_WORK_TREE
to "." after chdir'ing to the work tree root. Hooks start in that root,
so the pair is consistent at hook start, but a hook that does

    cd sub && git rev-parse --show-toplevel

gets "/abs/repo/sub", an empty --show-prefix, and a status in which
every path is relative to "sub". Without --work-tree (plain discovery)
the same hook works, because GIT_WORK_TREE is not exported at all.

0ed748134748 (setup_work_tree: adjust relative $GIT_WORK_TREE after
moving cwd, 2010-12-27) chose "." over an absolute path. Would exporting
the absolute work tree instead be acceptable? repo_get_work_tree()
already holds it at that point.

Reproduction (v2.55.0):

    git init repo && cd repo && mkdir sub && echo b >sub/b.txt
    git add . && git commit -m base
    printf '#!/bin/sh\ncd sub && git rev-parse --show-toplevel\n' \
      >.git/hooks/pre-commit && chmod +x .git/hooks/pre-commit
    echo c >sub/c.txt && git add sub/c.txt
    git --git-dir="$PWD/.git" --work-tree="$PWD" commit -m explicit
    # prints .../repo/sub; plain "git commit" prints .../repo
~~~
