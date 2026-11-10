# Git: the index of a working tree is not `<repo>/.git/index`

## Symptom

A tool that reads tracked paths by opening `<scanned directory>/.git/index`
reports nothing for that repository:

- In a linked worktree,
  where `.git` is a regular file holding one `gitdir:` line.
- In a submodule checkout,
  where the same file holds a relative `gitdir:` line.
- From a subdirectory of either,
  where no `.git` exists at all.

`forbidden-strings --all` had exactly that shape
 (`package/forbidden-strings/cli/src/walk.rs`,
issue 1232).
It fell back to its walker-only output,
 which honours `.gitignore`,
 so the force-added,
tracked files the union existed to recover were never
 scanned and the run exited clean.

## How Git decides

The rules below are read from `setup.c` in `git/git` at tag `v2.56.0`
 (`read_gitfile_raw`,
`read_gitfile_gently`,
`is_git_directory`,
 `get_common_dir_noenv`,
`repo_discovery_find_dir`)
 and then checked against the installed `git version 2.56.0`.

### Finding the Git directory

From the working directory upward,
per directory:

1.  A `.git` **file**:
    read it and stop if it names a valid Git directory.
2.  A `.git` **directory**:
    use it when it is a Git directory,
    else keep
    walking upward.
3.  A directory that is itself a bare Git directory:
    Git stops there too.

### Reading a `.git` file

```c
if (st.st_size > max_file_size)            /* max_file_size = 1 << 20 */
    error_code = READ_GITFILE_ERR_TOO_LARGE;
if (!starts_with(buf, "gitdir: "))
    error_code = READ_GITFILE_ERR_INVALID_FORMAT;
while (buf[len - 1] == '\n' || buf[len - 1] == '\r')
    len--;
if (len < 9)
    error_code = READ_GITFILE_ERR_NO_PATH;
strbuf_add(contents, buf+8, len-8);
```

Measured consequences:

- Only `\n` and `\r` are stripped,
  and every trailing one of them:
  `gitdir: <path>` with no newline,
  with CRLF,
  with a bare `\r`,
  and with
  three newlines all resolve,
  while trailing spaces and tabs stay in the path and make it invalid.
- A newline *inside* the path survives,
  because only trailing line breaks are terminators.
- `gitdir:` without the following space is `invalid gitfile format`.
- Exactly `1 MiB` is attempted;
  only a strictly larger file is refused.
- A non-UTF-8 path blob is not rejected by Git,
  which works in bytes.

### Resolving and validating

- A relative `gitdir:` is joined to the directory holding the `.git` file by
  string concatenation,
  with no component normalization:
  the kernel resolves `..`,
  including when `..` follows a symlink.
- The named directory is then validated as a Git directory:
  `HEAD` must be a symbolic ref into `refs/` or an object id,
  and `objects` and `refs` must exist beside the shared state.
  A `.git` file naming any other directory is
  `gitfile does not point to a valid repository`,
  which Git treats as fatal rather than as "keep walking".
- The accepted Git directory is then realpath'd (`strbuf_realpath`).

### The common directory

`<gitdir>/commondir`,
when present,
 names the directory holding the shared state,
 with every trailing `\n` and `\r` stripped and a relative entry resolved
 against `<gitdir>`.
A linked worktree's `config` and its objects and refs live there,
 while its own directory holds only per-worktree state and the index.

Measured for `git worktree add`:

```text
<main>/.git/worktrees/<name>/.git   ->  gitdir: <main>/.git/worktrees/<name>
<main>/.git/worktrees/<name>/HEAD   ->  ref: refs/heads/<branch>
<main>/.git/worktrees/<name>/commondir  ->  ../..
<main>/.git/worktrees/<name>/index  ->  this worktree's index
<main>/.git/config                  ->  the object format, shared by all worktrees
```

A submodule's `.git` file holds a relative line instead,
 for example `gitdir: ../../.git/modules/vendor/sub`.

### Which file is the index

`<gitdir>/index` unless `GIT_INDEX_FILE` names a different file,
 which Git reads relative to the current directory.
Git sets that variable itself for some commands and hooks,
 so a reader that ignores it can read a different index than the command in
 progress uses.

## Verification

Reproduced with throwaway fixtures under a scratch directory,
 using the installed Git binary:

```text
gitdir: <abs>            no trailing newline  -> rev-parse --git-dir succeeds
gitdir: <abs>\r\n        CRLF                 -> succeeds
gitdir: <abs>\r          bare CR              -> succeeds
gitdir: <abs>\n\n\n      three newlines       -> succeeds
gitdir: <abs>   \n       trailing spaces      -> fatal: gitfile does not point
gitdir: <abs>\t\n        trailing tab         -> fatal: gitfile does not point
gitdir:<abs>             no space after colon -> fatal: invalid gitfile format
gitdir: <plain dir>      not a repository     -> fatal: gitfile does not point
<dir>/.git empty dir     stray .git directory -> stepped over, repo above used
<dir>/.git malformed     no gitdir: prefix    -> fatal: invalid gitfile format
```

The scanning side of the fix lives in
 `package/forbidden-strings/cli/src/git_working_tree.rs`,
 and its Git-parity tests assert these shapes against the installed `git`
 binary rather than against the reader's own expectations
 (`package/forbidden-strings/cli/src/git_working_tree_tests.rs`).

## What the scanner does when it cannot use the index

The walk still runs,
 so every file the walker sees is scanned,
 and one line on stderr names the file that could not be used and why:

```text
forbidden-strings: the tracked-file union is unavailable: <path>: <cause>; tracked files that .gitignore hides are not covered by this scan
```

That line does not change the exit status,
 so a scan whose visible files are clean still exits `0`
 while saying its coverage is incomplete;
 treat the line as a failed gate.
A directory with no `.git` entry above it prints no such line,
 because nothing there claims a working tree.

## Boundaries this reader does not cross

`GIT_DIR`,
`GIT_WORK_TREE`,
and `core.worktree` name a different working tree
 root,
 `GIT_CEILING_DIRECTORIES` and the mount-point stop bound discovery,
 and Git's ownership checks reject a repository owned by another user.
Honouring those needs Git's configuration and ownership machinery,
 which the scanner does not link,
 so the scanned directory stays the working tree root.
`GIT_INDEX_FILE` *is* honoured,
 because it names the very file the reader opens.
