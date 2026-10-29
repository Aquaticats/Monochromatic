# git 2.56.0: an SSH-signed commit writes its signing buffer to `TMPDIR` or `/tmp`, so a full `/tmp` fails the commit

Observed on 2026-10-29 with `commit.gpgsign` on and `gpg.format = ssh`:
`git commit` died with `failed writing ssh signing key buffer ... Disk quota exceeded` while the repository disk had hundreds of gigabytes free.
Git picks the buffer's directory from `TMPDIR` (default `/tmp`) and nothing else,
so setting `TMPDIR` to a directory on a disk with room fixes the commit.

## Symptom

The commit,
made through the repository's `cli-git` wrapper,
printed:

```text
error: failed writing ssh signing key buffer to '/tmp/.git_signing_buffer_tmpNt9hBt': Disk quota exceeded
fatal: failed to write commit object
```

The wrapper exited with code 128 and no commit object was written.
At that time `/tmp` was a 16 GiB tmpfs with a per-user quota:

```text
tmpfs                   16G   13G  3.6G  78% /tmp
Disk quotas for user user (uid 1000):
     Filesystem   space   quota   limit
          tmpfs  12706M  13108M  13108M
```

Hundreds of `native-binary-*` directories (about 95 MB each) and `monochromatic-lint-stub-cargo-*` directories (about 5 MB each),
left behind by other sessions' test runs,
held most of it.
The repository disk (`/var/home`) had 784 GB free,
so the failure had nothing to do with the repository's own storage.

## Root cause

Git writes the buffer it is about to sign into a temporary file before it runs `ssh-keygen -Y sign`.
`gpg-interface.c:1092` creates that file,
and `gpg-interface.c:1098-1100` writes it and reports a failed write with the observed message:

```c
	buffer_file = mks_tempfile_t(".git_signing_buffer_tmpXXXXXX");
	if (!buffer_file) {
		error_errno(_("could not create temporary file"));
		goto out;
	}

	if (write_in_full(buffer_file->fd, buffer->buf, buffer->len) < 0 ||
	    close_tempfile_gently(buffer_file) < 0) {
		error_errno(_("failed writing ssh signing key buffer to '%s'"),
			    buffer_file->filename.buf);
		goto out;
	}
```

`mks_tempfile_t` takes no directory argument.
`tempfile.c:188-194` shows where the file lands:

```c
	const char *tmpdir;

	tmpdir = getenv("TMPDIR");
	if (!tmpdir)
		tmpdir = "/tmp";

	strbuf_addf(&tempfile->filename, "%s/%s", tmpdir, filename_template);
```

So the directory is `TMPDIR` when set and `/tmp` otherwise.
The message text in the symptom matches `gpg-interface.c:1100` exactly,
which places the failure in the write (`write_in_full`) and not in the creation of the file.
The create step has its own message (`could not create temporary file`,
line 1094),
seen in the harness below.

Git's behavior here is correct:
a full `/tmp` makes any write to it fail.
The surprise is that signing a commit depends on `/tmp` having room at all,
on a machine where the repository lives elsewhere.

Source under citation:
`git/git` shallow clone of the default branch at commit `8d1b215c01512b9658d4525c54ff9b9245f9a7e8` ("The 10th batch",
2026-10-28).
The binary that produced the observation and runs the harness is the installed `git version 2.56.0`;
this doc did not compare the cited commit against the 2.56.0 tag,
so a line number may differ by a few lines in that tag.

## Verification

Version under test:
`/usr/bin/git`,
`git version 2.56.0`.

The harness uses a throwaway repository and a throwaway SSH key under `${HOME}/temp/agent/`,
no repository state of this project:

```bash
# Throwaway repository and key.
root="$(mktemp --directory "${HOME}/temp/agent/signing-harness-XXXXXXXX")"
ssh-keygen -q -t ed25519 -N '' -f "${root}/key"
git -C "${root}" init --quiet repo
git -C "${root}/repo" config user.name Harness
git -C "${root}/repo" config user.email harness@example.invalid
git -C "${root}/repo" config gpg.format ssh
git -C "${root}/repo" config user.signingkey "${root}/key.pub"
git -C "${root}/repo" config commit.gpgsign true
echo a > "${root}/repo/a.txt"
git -C "${root}/repo" add a.txt

# Fails: git tries to create the buffer under TMPDIR.
TMPDIR=/nonexistent-dir git -C "${root}/repo" commit --message x

# Works: a writable TMPDIR.
mkdir "${root}/scratch"
TMPDIR="${root}/scratch" git -C "${root}/repo" commit --message x
```

Output recorded on 2026-10-29:

```text
TMPDIR=/nonexistent-dir -> exit 128
error: could not create temporary file: No such file or directory
fatal: failed to write commit object
TMPDIR=<writable dir> -> exit 0
[master (root-commit) dc3a376] x
```

What this proves:
the buffer location follows `TMPDIR`,
and a commit succeeds whenever that directory is writable.

What it does not prove:
the `Disk quota exceeded` variant itself.
The harness does not fill a disk,
so the exact write failure at line 1100 was matched by message text only.
A reproduction would need a size-limited tmpfs for `TMPDIR`;
creating one was not attempted in this session.

### Patterns that work

- `TMPDIR=<directory on a disk with room>` before `git commit`.
  The session made nine commits through the `cli-git` wrapper with `TMPDIR=${HOME}/temp/agent/gittmp` after the failure,
  and every one succeeded;
  `/tmp` was not measured again during those commits,
  so they show the workaround works,
  not that `/tmp` was still full.
  The wrapper passes `TMPDIR` through to git.

### Patterns that fail

- Any signed commit while `TMPDIR` (or `/tmp`) cannot take the write,
  with either the create or the write message above.

Not run:
a commit with `commit.gpgsign` off.
By the code,
only the SSH signing path (`gpg-interface.c:1055`,
the function that contains line 1092) creates this buffer,
so an unsigned commit should not reach it,
but no unsigned commit was tried.

### Open observation

The quota listing quoted in the symptom was taken a few minutes after the failure and showed 12706M used against a 13108M limit,
about 400 MB below it.
A signing buffer is a few hundred bytes,
so the usage was probably at the limit at the moment of the write while other sessions kept writing,
but that was not captured;
this doc does not claim it.

## Verified workarounds

Set `TMPDIR` for the commit command to a directory outside the full filesystem:

```bash
mkdir --parents "${HOME}/temp/agent/gittmp"
chmod 700 "${HOME}/temp/agent/gittmp"
TMPDIR="${HOME}/temp/agent/gittmp" git commit --message "..." -- path/to/file
```

Tradeoffs:

- `TMPDIR` also applies to every program git starts for that command (hooks,
  `ssh-keygen`,
  `git-lfs`),
  so they write there too,
  which is usually what is wanted but can surprise a hook that assumes `/tmp`.
- The directory must exist and be writable by the committing user;
  git does not create it,
  and a missing directory fails with the create message.
- It does not free `/tmp`;
  other programs that write there still fail.

The cleanup that removes the cause is deleting the leftover test directories,
which belong to other sessions and was not done here.

## What does not work

- Looking in repository or user configuration for a signing-buffer directory:
  there is none.
  The call at `gpg-interface.c:1092` takes only a filename template,
  and `tempfile.c:188-194` reads only `TMPDIR`.
- Reading the failure as a repository disk problem:
  the repository filesystem had 784 GB free,
  and the error names the `/tmp` path.

## Upstream filing decision

Walk of the six constraints,
with the default policy of not filing:

1. Is it really upstream's fault?
   No. Git writes to the directory `TMPDIR` names and reports the failed write accurately.
   The cause is a full user-quota tmpfs on this machine.
2. Can upstream fix it?
   Not applicable,
   because constraint 1 fails.
3. Are they supporting this use case?
   Not applicable.
4. Would the repository welcome our contribution?
   Not checked:
   with constraint 1 failing there is nothing to contribute.
5. Will they likely fix it?
   Not applicable.
6. Have we prototyped a minimal fix?
   Not applicable.

Decision:
nothing to file.
`.out-of-scope/` lists no entry for git or temporary-file handling (checked against the directory listing on 2026-10-29),
so no exemption applies.
No duplicate search was run because nothing is drafted.
