# bubblewrap 0.12.0: replaced `/tmp`, `/run`, and `/dev` hide projects below them, and `--clearenv` drops the `PWD` spelling of the working directory

The IDE's Language module runs every language server inside `/usr/bin/bwrap`
(`package/ide/app/src/language/confine.rs`).
Two path behaviors of bubblewrap shaped that launch,
and a third showed up in the mount audit.

## Symptom

### A project below a replaced directory is missing inside

With the adopted recipe
(`--ro-bind / / --dev /dev --proc /proc --tmpfs /run --bind STATE/tmp /tmp --bind STATE STATE`),
a project below `/tmp`,
 `/run` (for example `/run/media/<user>/...` or `/run/user/<uid>/...`),
 or `/dev/shm`
does not exist inside the sandbox:

```text
# measured with /usr/bin/bwrap 0.12.0, project /tmp/ide-bind-project.KA4py7
ls: cannot access '/tmp/ide-bind-project.KA4py7': No such file or directory
sh: line 1: /tmp/ide-bind-project.KA4py7/written: No such file or directory
```

A server started that way gets a root it cannot see and answers nothing,
 without an error naming the cause.

### The server starts in the canonical directory, not Helix's `PWD` spelling

When the application is started with `PWD` naming a symbolic link
(here `/tmp/ide-language-alias-P6D3Zw/project-link` pointing at the project),
Helix roots the server at that spelling:

```json
{"rootPath":"/tmp/ide-language-alias-P6D3Zw/project-link","rootUri":"file:///tmp/ide-language-alias-P6D3Zw/project-link"}
```

and spawns bubblewrap with that working directory,
but the server's `getcwd` inside is the canonical project path
(`/var/home/user/temp/agent/ide-language-confinement-3evKAq/scripted-project`).

### `binfmt_misc` is listed read-write inside

`/proc/self/mountinfo` inside the sandbox lists `/proc/sys/fs/binfmt_misc` as `rw`:

```text
3092 3091 0:56 / /proc/sys/fs/binfmt_misc rw,nosuid,nodev,noexec,relatime master:230 - binfmt_misc binfmt_misc rw
```

## Root cause

### Replaced directories are new mounts that cover the host's

`--tmpfs /run`,
`--bind STATE/tmp /tmp`,
and `--dev /dev` each mount something new on that path,
so everything the host had below it is covered,
including the project.
This is the documented meaning of those options,
not a defect.
A later mount covers an earlier one,
so binding the project again after the replacements makes it visible:
the measured run with `--ro-bind PROJECT PROJECT` last listed the project
and failed the write with `Read-only file system`.

### `--clearenv` runs while options are parsed, before the working directory is recorded

bubblewrap clears the environment as soon as it parses the option
(`bubblewrap.c` at tag `v0.12.0`,
 line 2476):

```c
// bubblewrap.c:2474 (v0.12.0)
      else if (strcmp (arg, "--clearenv") == 0)
        {
          xclearenv ();
        }
```

Options are parsed at line 2907 (`parse_args (&argc, (const char ***) &argv);`).
Much later it records the working directory with glibc's `get_current_dir_name`,
which returns `$PWD` only when that variable is set and names the same directory as `.`,
and `getcwd` otherwise:

```c
// bubblewrap.c:3247 (v0.12.0)
  old_cwd = get_current_dir_name ();
```

After the mounts it changes into that directory if it exists inside,
 and then sets `PWD` itself:

```c
// bubblewrap.c:3414 (v0.12.0)
  else if (chdir (old_cwd) == 0)
    {
      /* If the old cwd is mapped in the sandbox, go there */
      new_cwd = old_cwd;
    }
...
  xsetenv ("PWD", new_cwd, 1);
```

With `--clearenv`,
`PWD` is already gone at line 3247,
so `old_cwd` is the canonical `getcwd` path.
The server therefore runs in the canonical spelling of the same directory,
while its `initialize` root is Helix's spelling.
The same line 3427 explains why a server sees `PWD` although the allowlist does not contain it
(measured:
 `bwrap --unshare-user --ro-bind / / --dev /dev --proc /proc --clearenv -- /usr/bin/env` prints only `PWD=...`).

### `binfmt_misc` comes from the host's automount and stays unwritable

The host mounts `binfmt_misc` through a systemd automount below `/proc`;
`--ro-bind / /` copies host mounts recursively and they propagate as `master:` peers,
so the instance appears below the fresh `/proc` with the mount flag the host has.
Its files belong to the host's root,
 which is unmapped inside (`nfsnobody`),
so registration is refused:

```text
/usr/bin/sh: line 1: /proc/sys/fs/binfmt_misc/register: Permission denied
```

## Verification

- Version:
  `bubblewrap 0.12.0` (`/usr/bin/bwrap --version`),
  source read at the `v0.12.0` tag of `containers/bubblewrap`;
  kernel `7.2.7-ogc1.1.fc44.x86_64`.
- Harness for the hidden project:
  the recipe ran with and without `--ro-bind PROJECT PROJECT`
  for projects below `/tmp`,
   `/run/user/1000`,
   and `/dev/shm`,
  and once with private state below `/tmp`;
  one case:

```sh
# one case of the measurement; PROJECT below /tmp, STATE in a private scratch directory
/usr/bin/bwrap --die-with-parent --new-session --unshare-user --unshare-pid --unshare-ipc --unshare-uts \
  --unshare-cgroup --unshare-net --ro-bind / / --dev /dev --proc /proc --tmpfs /run \
  --bind "$STATE/tmp" /tmp --bind "$STATE" "$STATE" --ro-bind "$PROJECT" "$PROJECT" \
  --clearenv --setenv PATH /usr/bin \
  -- /usr/bin/sh -c 'ls "$1"; echo x > "$1/written"; echo x > "$2/state-write"' sh "$PROJECT" "$STATE"
```

- Works:
  - project below `/tmp`,
     `/run/user/1000`,
     or `/dev/shm` with the project bound again last:
    listed,
     write refused with `Read-only file system`,
     state write succeeds;
  - private state below `/tmp` (bound after `--bind STATE/tmp /tmp`):
    state write lands in the host directory;
  - the full application path:
    `mise run //package/ide/app:inspect:language-confinement`
    runs rust-analyzer and the TypeScript 7 server on projects below `/tmp` and `$XDG_RUNTIME_DIR`
    with build-script,
     proc-macro,
     launcher,
     and escape probes.
- Fails:
  - the same projects without the second bind:
    `No such file or directory` for the listing and the write;
  - expecting the server's working directory to be Helix's `PWD` spelling:
    it is the canonical path whenever `--clearenv` is given.

## Verified workarounds

- Bind the project again after the replacements,
  at its canonical path and at Helix's spelling when that spelling lies below `/tmp`,
   `/run`,
   or `/dev`
  (`package/ide/app/src/language/confine/project.rs`,
   `project_binds`).
  Tradeoff:
  one more mount per server even for projects elsewhere;
  that canonical bind is also the single place a later write mode changes (`PROJECT_MOUNT`).
- Resolve the private state root through symbolic links before checking and binding it
  (`confine.rs`,
   `private_state`).
  Tradeoff:
  the state directory is named by its real location,
  so moving the link target moves the state.
- Accept the canonical working directory.
  Tradeoff:
  a server that compares its working directory textually with its root would see two spellings;
  neither measured server does,
  and the root it receives exists inside because of the alias bind.

## What does not work

- Binding the project at every spelling:
  a destination that is a symbolic link on the host makes bubblewrap follow it
  (upstream issue [#390][],
   "Bind mounting over symlinks attempts to create their targets and bind mount over them"),
  so spellings outside the replaced directories are left to resolve through the read-only root.
- Refusing projects below `/tmp` and `/run`:
  implemented first,
  then rejected on 2026-10-05 because projects there must work.
  Only `/proc` stays refused:
  `mkdir /proc/x` fails (`No such file or directory`),
   so no project can live there.
- Passing `PWD` with `--setenv` to restore the spelling:
  the spelling depends on the server's root,
  which Helix chooses per document after the launch arguments are built.

## Upstream filing decision

- `.out-of-scope/` has no bubblewrap entry.
- Duplicate search:
  `gh search issues --repo containers/bubblewrap "PWD"`,
   `"cwd symlink"`,
   `"get_current_dir_name"`,
   and `gh search prs ... "cwd"`
  found nothing about `--clearenv` and the working directory.
- Constraint 1,
   upstream's fault:
  no.
  `--clearenv` is documented as "Unset all environment variables",
  the fallback to `getcwd` names the same directory,
  and nothing fails;
  the replaced directories and the `binfmt_misc` flag are documented or kernel behavior.
- Constraints 2 to 6 were not pursued,
   since constraint 1 fails.
- Decision:
  nothing filed.
  The draft stays as a record.

~~~md
<!-- do not file as-is: constraint 1 fails -->
Title: --clearenv makes the sandbox start in getcwd's spelling instead of $PWD's

bubblewrap 0.12.0 applies --clearenv while parsing options (bubblewrap.c:2476), before
get_current_dir_name records the working directory (bubblewrap.c:3247), so a $PWD spelling
through a symbolic link is lost and the sandbox starts in the canonical path.
Recording old_cwd before option parsing would keep the spelling.
~~~

[#390]: https://github.com/containers/bubblewrap/issues/390
