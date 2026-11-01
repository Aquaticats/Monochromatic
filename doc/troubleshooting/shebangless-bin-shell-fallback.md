# Shebangless executable output run from bash 5.3.9 falls to /bin/sh, where ImageMagick 7.1.2-31 import hangs or errors

## Symptom

Executing a built CLI or script file directly (`./dist/final/node/cli.mjs`,
`${CLAUDE_PLUGIN_ROOT}/bundle/node/index.mjs`,
 a symlink pointing at the file) when its first line is not `#!`
produces one of three failure modes instead of a Node program:

1. The file's first shell-parseable line names `import` and no display is reachable
   (`import` is ImageMagick's screenshot command on `PATH`):

   ```text
   import: unable to open X server `' @ error/import.c/ImportImageCommand/350.
   ```

2. The same file with a display reachable hangs.
   Three hangs were observed in one session,
    each past a 60 second tool timeout,
   and the ImageMagick child survived the wrapper's own `timeout 8` and kept holding the output pipe.

3. A JavaScript file whose lines are not shell words reports a shell parse error:

   ```text
   ./plain.mjs: line 1: syntax error near unexpected token `"ok"'
   ./plain.mjs: line 1: `console.log("ok");'
   ```

Execution that never goes through a shell reports `Exec format error (os error 8)` instead
(for example `timeout ./cli.mjs`,
 where `timeout` calls `execve` directly).

Not affected:
 `node_modules/.bin/<command>` under pnpm,
whose wrapper ends in `exec node "$basedir/../<pkg>/<target>" "$@"`,
so the interpreter is explicit and the target's first line is never consulted.
A file without the executable bit fails with `Permission denied` before any of the modes above.

## Root cause

1. The kernel's `execve` reads the first bytes of the file,
   finds no `#!` magic and no recognized binary format,
   and fails with `ENOEXEC`.
   The `Exec format error (os error 8)` probe in "Verification" is that failure surface.
2. The invoking shell catches `ENOEXEC` and re-runs the file as a shell script in a new shell instance.
   `man bash` (GNU bash 5.3.9,
    "COMMAND EXECUTION") states:

   ```text
   If this execution fails because the file is not in executable format, and the file is not a
   directory, it is assumed to be a shell script, a file containing shell commands, and the
   shell creates a new instance of itself to execute it.
   ```

   In `execute_cmd.c` (bash development snapshot fetched 2026-10-04 from
   [cgit](https://cgit.git.savannah.gnu.org/cgit/bash.git/plain/execute_cmd.c)),
   `shell_execve` calls `execve` and only handles the non-`ENOEXEC` errors:

   ```c
   // execute_cmd.c:6126
   shell_execve (char *command, char **args, char **env)
   {
     ...
   // execute_cmd.c:6133
     execve (command, args, env);
     i = errno;			/* error from execve() */
   // execute_cmd.c:6140
     if (i != ENOEXEC)
       {
   ```

   and after the binary-file check at `execute_cmd.c:6228` commits to shell execution
   (`execute_cmd.c:6236`):

   ```c
   /* We have committed to attempting to execute the contents of this file
      as shell commands.  */
   ```

3. The bundled JavaScript is then parsed as shell.
   A line like `import { x } from "./y.mjs";` is a simple command whose command word is `import`,
   and `PATH` lookup finds ImageMagick's `/usr/bin/import`.
4. ImageMagick `import` is an interactive screenshot command:
   with a display it waits for region selection (hang),
   without one it fails with the X-server error quoted in "Symptom".

Wrong earlier reading,
 kept so it is not re-derived:
 the issue #139 brief says Unix bin wrappers
"fall through to `/bin/sh`",
 which reads as a kernel fallback.
The kernel does no such thing;
 `execve` returns `ENOEXEC` and only a shell's command execution path
re-runs the file.
Kernel `binfmt_script` dispatches `#!` lines only.

## Verification

Versions under test:

- GNU bash 5.3.9(1)-release (x86_64-redhat-linux-gnu),
   from `bash --version`.
- ImageMagick 7.1.2-31 Q16-HDRI x86_64 24606,
   from `import -version`.
- Node.js v26.10.0,
   the interpreter the fixed build emits in its preamble.
- bash `execute_cmd.c` development snapshot,
   fetched 2026-10-04 from
  [cgit](https://cgit.git.savannah.gnu.org/cgit/bash.git/plain/execute_cmd.c);
  the host's 5.3.9 documents the same behavior in `man bash`,
  quoted in "Root cause".

Harness (each `cd` target is a fresh `mktemp -d` directory):

```bash
# doc/troubleshooting/shebangless-bin-shell-fallback.md probe harness
printf 'import { x } from "./y.mjs";\n' > withimport.mjs
printf 'console.log("ok");\n' > plain.mjs
printf '#!/usr/bin/env node\nconsole.log("ok");\n' > shebanged.mjs
chmod +x withimport.mjs plain.mjs shebanged.mjs
env -u DISPLAY -u WAYLAND_DISPLAY bash -c './withimport.mjs'
env -u DISPLAY -u WAYLAND_DISPLAY bash -c './plain.mjs'
./shebanged.mjs
```

Patterns that work cleanly:

1. `#!/usr/bin/env node` at byte 0,
    file executable:
    prints `ok`.
2. Interpreter given explicitly (`node plain.mjs`):
    prints `ok`.
3. `node_modules/.bin/<command>` under pnpm:
    wrapper calls `exec node <target>`,
    no shebang consulted.
4. File without the executable bit:
    `Permission denied`,
    no shell parse attempted.

Patterns that fail,
 by variant:

1. ImageMagick error variant:
    shebangless executable whose first parsed command word is `import`,
   no display:
    ``import: unable to open X server `' @ error/import.c/ImportImageCommand/350.``.
2. Hang variant:
    the same file with a display reachable,
   three observed hangs past 60 seconds,
   surviving the wrapper's `timeout 8` through the grandchild ImageMagick process.
3. Shell parse error variant:
    shebangless JavaScript with no runnable first word,
   run directly or through a symlink (`ln -s plain.mjs linked; ./linked`):
   `syntax error near unexpected token`,
    identical for the symlink.
4. Exec boundary variant:
    `timeout ./cli.mjs` reports
   `timeout: failed to run command './cli.mjs': Exec format error (os error 8)`.

## Verified workarounds

1. Build-side injection,
    landed in this repository:
   `package/config/rolldown/src/bin-shebang.ts` prepends `#!/usr/bin/env node` at byte 0 to
   `bin`-targeted entry chunks that lack a shebang,
   wired through `nodeConfig` and `perEntryNodeConfig` in `package/config/rolldown/src/index.node.ts`
   (commit `b7a23c71c`,
    issue #139).
   Tradeoffs:
    covers only builds going through the shared Node preset;
   a `bin` target that no emitted chunk matches is ignored,
    so typo'd `bin` paths stay silent;
   a shebang already present is treated as user error and left untouched.
2. Shebang at byte 0 of the source entry,
    kept through the bundler.
   All 37 built-output `bin` targets in this repository carry `#!/usr/bin/env node` this way,
   measured 2026-10-04.
   Tradeoffs:
    discipline only,
    nothing enforces it outside the preset;
   direct `.ts` `bin` entries (`package/watch-restart/cli` `src/cli.ts`,
   `package/claude-code-plugin/source` `src/cli/spawn-claude.ts`) need it in the source itself
   because no build rewrites them.
3. Explicit interpreter at the call site (`node plain.mjs` runs the shebangless file,
    probe prints `ok`).
   Tradeoffs:
    every caller must know the interpreter,
   and symlink dispatch (`ln -s plain.mjs linked; ./linked`) still parses the target as shell,
   measured identical to direct execution.

## What does not work

- `chmod +x` alone:
   the executable bit only enables the exec path,
  and the file still has no interpreter (exec boundary variant in "Verification").
- Expecting the kernel to run the file with `/bin/sh`:
   it returns `ENOEXEC` instead,
  see the wrong-earlier-reading paragraph in "Root cause".
- A repository task scan as the only enforcement:
   it reports exact package and `bin` paths
  after the fact while the emitted artifact stays unfixed;
  recorded as rejected in the issue #139 findings.
- Validating or rewriting shebangs that already exist in the output:
  the human rejected this as out of contract on 2026-10-04 (user error,
   undefined behavior),
  so the injection stays one directional.
- Relying on `node_modules/.bin` wrappers to hide a shebangless target:
  pnpm's wrapper happens to call `node` explicitly,
  but direct execution and symlink dispatch do not,
  and Claude Code plugin hooks execute `${CLAUDE_PLUGIN_ROOT}/bundle/node/index.mjs` directly.

## Upstream filing decision

Nothing to file.
The failure is the composition of three documented,
 by-design behaviors,
so the six-constraint check fails at the first constraint.

1. Is it really upstream's fault?
   No.
   Kernel `ENOEXEC`,
    the shell's documented script fallback,
    and ImageMagick's `import` name
   each behave as designed;
    no component has a bug.
2. Can upstream fix it?
   Not without changing documented semantics
   (a shell would have to stop re-running `ENOEXEC` files,
   or ImageMagick would have to rename `import`).
3. Are they supporting this use case?
   The shell fallback exists precisely to run interpreter-less scripts,
   which is the documented feature `man bash` describes.
4. Would the repo welcome our contribution?
   Not applicable:
    constraints one through three fail,
   so there is no upstream change to propose.
5. Will they likely fix it?
   No signal sought,
    since nothing is broken upstream.
6. Have we prototyped a minimal fix compatible with their architecture?
   Not applicable for the same reason;
   the fix that solves the cause lives at our artifact boundary
   (`package/config/rolldown/src/bin-shebang.ts`).

The earlier plan to rename `import` in our own sources
to dodge the ImageMagick collision is out of scope per issue #139
and does not address the general case.
