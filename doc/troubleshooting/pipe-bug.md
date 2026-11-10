# Claude Code sandbox pipe breakage investigation

Status:
 **root cause identified**;
 [upstream issue filed](https://github.com/anthropics/claude-code/issues/31968)

## Summary

All pipes at the top level of Claude Code's bash sandbox are broken.
`echo hello | cat` produces no output.
 `echo hello | wc -l` produces no output.
Every piped command silently fails.

Appending `; true` (or any trailing command) after the pipe fixes it:
`echo hello | cat; true` works correctly.

## Root cause

The bug is a shell quoting error in Claude Code's eval chain.

### The eval chain

Claude Code wraps user commands in a nested eval:

```bash
eval "source $SNAPSHOT && shopt -u extglob || true && eval \"$USER_CMD\" \< /dev/null && pwd -P >| /tmp/cwd-XXX"
```

The `\<` before `/dev/null` is the bug.
 Backslash-quoting `<` inside a double-quoted
string makes it a **literal character argument** to eval,
 not a redirect operator
on the eval command.

### What happens step by step

1.  The outer eval receives arguments:
     `source $SNAPSHOT && ... && eval "$USER_CMD"`,
     `<`,
     `/dev/null`,
     `&&`,
     `pwd -P`,
     `>|`,
     `/tmp/cwd-XXX`
2.  The `<` and `/dev/null` are literal string arguments,
     not a redirect on the outer eval
3.  The inner `eval` builtin receives three arguments:
     `$USER_CMD`,
     `<`,
     `/dev/null`
4.  `eval` joins its arguments with spaces:
     `$USER_CMD < /dev/null`
5.  When `$USER_CMD` is `echo hello | cat`,
     this becomes:
     `echo hello | cat < /dev/null`
6.  `< /dev/null` is parsed as a redirect on `cat` (the last simple command in the pipeline)

In `execute_disk_command()` (execute_cmd.
c:
5770),
 the child process for `cat`:

1.  `do_piping()` at line 5870 correctly sets fd 0 to the pipe read end via `dup2(pipe_in, 0)`
2.  `do_redirections()` at line 5884 applies the `< /dev/null` redirect,
     overwriting fd 0 with `/dev/null`

The pipe data from `echo hello` arrives at the pipe write end,
 but `cat` reads from `/dev/null` and gets EOF immediately.

### Why `; true` fixes it

With `; true` appended,
 the user command is `echo hello | cat; true`.
The inner eval joins to:
 `echo hello | cat; true < /dev/null`.
Now `< /dev/null` redirects `true` (not `cat`),
 so the pipe remains intact.

### Why it doesn't reproduce outside the sandbox

The correct eval chain should use `<` (a redirect operator on the eval command),
not `\<` (a literal argument).
 With unescaped `<`:

```bash
# CORRECT: < is a redirect on eval itself, stdin of eval becomes /dev/null
# The inner eval never sees < or /dev/null as arguments
bash -c 'eval "echo hello | cat" < /dev/null'
# Output: hello

# BROKEN: \< makes < a literal argument passed through to eval
bash -c 'eval "echo hello | cat" \< /dev/null'
# No output
```

Outside Claude Code,
 reproduction attempts used `< /dev/null` (unescaped),
which correctly redirects eval's stdin rather than passing `<` as an argument.
The bug only manifests with the escaped `\<` form.

### Verification with set -x

```bash
# BROKEN: \< passes < as literal arg
$ bash -c 'set -x; eval "echo hello | cat" \< /dev/null'
+ eval 'echo hello | cat' '<' /dev/null    # <-- three args to eval!
++ echo hello
++ cat
# no output

# WORKING: < is a redirect on eval
$ bash -c 'set -x; eval "echo hello | cat" < /dev/null'
+ eval 'echo hello | cat'                  # <-- one arg, stdin redirected
++ echo hello
++ cat
hello
```

### Strace confirmation

Strace of the broken case inside the sandbox showed the exact sequence:

```text
# cat child process:
dup2(3, 0) = 0         # do_piping: fd 0 = pipe read end
close(3) = 0
openat("/dev/null", O_RDONLY) = 3   # command-level redirect from eval'd string
dup2(3, 0) = 0         # OVERWRITES fd 0 with /dev/null
close(3) = 0
execve("/usr/bin/cat")  # cat reads from /dev/null, gets EOF
```

Strace of the working case (`; true`):

```bash
# cat child process:
dup2(3, 0) = 0         # do_piping: fd 0 = pipe read end
close(3) = 0
execve("/usr/bin/cat")  # cat reads from pipe, gets "hello\n"
```

## Source code evidence (Claude Code v2.1.71)

Extracted JS via `tweakcc unpack` from the compiled binary at
`~/.local/share/claude/versions/2.1.71`.

### `oAD`: wraps user command with `/dev/null` redirect (the broken path)

```javascript
// Minified names: H = command, $ = shouldRedirect, ID = shellQuote
function oAD(H, $ = true,) {
  if (zyA(H,) || N61(H,)) { // heredoc or multiline string
    let L = `'${H.replace(/'/g, `'"'"'`,)}'`;
    if (zyA(H,))
      return L;
    return $ ? `${L} < /dev/null` : L; // ← correct: unquoted <
  }
  if ($)
    return ID([H, '<', '/dev/null',],); // ← BUG: < is shell-quoted as literal
  return ID([H,],);
}
```

`ID` is the shell-quoting function.
 `ID(["echo hello | cat", "<", "/dev/null"])`
produces `'echo hello | cat' '<' '/dev/null'`:
 three quoted tokens.
The `<` becomes a literal argument `'<'`,
 not a redirect operator.

### `HLD`: pipe-aware redirect insertion (the correct path, but not used in sandbox)

```javascript
function HLD(H,) {
  // parse command tokens, find first pipe operator
  let D = S61(L,); // index of first |
  if (D <= 0)
    return odH(H,); // fallback
  // Insert < /dev/null BEFORE the pipe, not after
  let f = [...eAD(L, 0, D,), '< /dev/null', ...eAD(L, D, L.length,),];
  return ID([f.join(' ',),],);
}
```

This correctly places `< /dev/null` on the left side of the pipe.
But it is **gated on `!q.useSandbox`**:

### Call site; the sandbox gate

```javascript
Y = sAD(f,); // replace NUL with /dev/null
O = aAD(Y,); // check if redirect is needed
X = oAD(Y, O,); // default: broken shell-quoting
if (!q.useSandbox && Y.includes('|',) && O)
  X = HLD(Y,); // correct pipe handling, ONLY outside sandbox
```

Inside the sandbox (`q.useSandbox === true`),
 `HLD` is never called.
The `oAD` path is always used,
 which shell-quotes `<` as a literal argument.

### `odH`: fallback for complex commands (works correctly)

```javascript
function odH(H,) {
  return ID([H,],) + ' < /dev/null'; // concatenates unquoted < /dev/null
}
```

Used for commands with backticks,
 `$()`,
 or control-flow keywords.
This path works because `< /dev/null` is appended as raw text outside `ID()`.

### Final assembly

```javascript
j.push(`eval ${X}`,);
j.push(`pwd -P >| ${z}`,);
let G = j.join(' && ',);
```

Producing for the broken case:

```text
source '/snapshot' && shopt ... && eval 'echo hello | cat' '<' '/dev/null' && pwd -P >| /tmp/cwd
```

The inner `eval` receives args `echo hello | cat`,
 `<`,
 `/dev/null`,
joins them into `echo hello | cat < /dev/null`,
 and `< /dev/null` becomes
a redirect on `cat` (the rightmost simple command in the pipeline).

## Fix

Two bugs to fix:

1.  **`oAD` shell-quotes `<` as a literal argument** instead of emitting it as
    a redirect operator.
     The heredoc/multiline branch already does this correctly
    (`${L} < /dev/null` with unquoted `<`).
     The normal branch should do the same:
    `return ID([H]) + " < /dev/null"` (like `odH` does).

2.  **`HLD` is gated on `!q.useSandbox`**,
     so the pipe-aware redirect insertion
    never runs inside the sandbox.
     Remove the `!q.useSandbox` condition,
     or fix
    `oAD` so that the redirect doesn't interfere with pipes regardless.

The purpose of `< /dev/null` is to prevent user commands from reading Claude Code's
IPC socket (which is the shell's stdin).
 Redirecting eval's stdin achieves this correctly;
the redirected stdin propagates to all child commands without interfering with pipes.

## Related: stdin-reading hang with `&&`/`;` chains

The same `\<` bug causes a different symptom when the user command contains `&&` or `;`
but **no pipes**.
 Tools like `rg` and `fd` auto-detect whether stdin is a TTY:

- TTY stdin:
   search current directory (normal interactive behavior)
- Non-TTY stdin:
   read input from stdin (filter mode)

Claude Code's stdin is a non-TTY socket that never sends EOF.
The `< /dev/null` redirect is meant to give commands immediate EOF,
but due to the `\<` bug it becomes a string argument to `eval`,
which appends it to the last simple command in the chain.

### Example

User command:
 `rg -l 'pattern' --type ts && cat results.txt`

After eval joins its arguments:

```bash
rg -l 'pattern' --type ts && cat results.txt < /dev/null
```

`< /dev/null` redirects `cat`'s stdin (harmless),
 but `rg` gets the original
non-TTY socket as stdin.
 Without an explicit path argument,
 `rg` enters
stdin-reading mode and blocks forever.

### Why bare `rg` (no chain) doesn't hang

Without `&&` or `;`,
 eval produces `rg -l 'pattern' --type ts < /dev/null`.
The redirect applies directly to `rg`,
 giving it immediate EOF.
`rg` reads nothing from stdin and exits with code 2 ("no files were searched"):
fast,
 but wrong results (it searched stdin instead of the filesystem).

### Workaround

Always pass an explicit search path (`.` or absolute) to `rg`/`fd`:

```bash
# HANGS (no path, rg falls back to stdin in non-TTY sandbox)
rg -l 'pattern' --type ts && echo done

# WORKS (explicit path, rg never checks stdin)
rg -l 'pattern' --type ts . && echo done
```

## Discarded hypotheses

These were investigated but turned out to be irrelevant:

- `async_redirect_stdin()`:
   Opens `/dev/null` and dup2's to fd 0 for async commands.
  The strace pattern matched,
   but the actual cause was `do_redirections()` applying the
  command-level `< /dev/null` redirect,
   not async stdin handling.

- Redirect propagation from eval builtin:
   Hypothesized that eval's redirects leak
  into pipeline children.
   In reality,
   the redirect was ON the pipeline command itself
  (from eval joining its arguments),
   not propagated from eval's redirect list.

- ONESHOT fork optimization:
   `should_suppress_fork()` / `can_optimize_connection()`
  in evalstring.
  c.
   Irrelevant:
   `eval` passes `SEVAL_NOOPTIMIZE`,
   and `can_optimize_connection`
  only handles `&&`/`||`/`;`,
   not `|`.

- `set -m` (job control):
   Investigated extensively.
   Not the cause.

- `set -t` (onecmd):
   Investigated extensively.
   Not the cause.

- `setsid` / `--new-session`:
   Red herring.
   `setsid` without `-w` breaks all output.

- Socket as stdin:
   Thought this was the missing reproduction ingredient.
  Actually,
   the missing ingredient was the `\<` escaping,
   which reproduction
  attempts outside the sandbox did not use.

- bwrap flags:
   `--unshare-pid`,
   `--unshare-net`,
   `--new-session` are all irrelevant.

## Evidence appendix

### fd inspection inside the sandbox

Ran `ls -la /proc/self/fd/0` inside the right side of a pipe:

- **Broken** (`echo hello | ls -la /proc/self/fd/0`):
  fd 0 points to `/dev/null`
- **Working** (`echo hello | ls -la /proc/self/fd/0; true`):
  fd 0 points to `pipe:[491709]`

### Pipe tests inside sandbox

All of these produce **no output** (broken):

```bash
echo hello | cat
echo -e "a\nb\nc" | wc -l           # returns 0
echo test | /usr/bin/cat
```

All of these **work** (with trailing command):

```bash
echo hello | cat; true               # returns "hello"
echo hello | cat; echo done          # returns "hello\ndone"
```

### Strace log files

- `/tmp/claude-1000/strace-broken.log`:
   broken case showing pipe overwrite
- `/tmp/claude-1000/strace-detailed.log`:
   working case for comparison
- `/tmp/claude-1000/strace-broken2.log`:
   broken case with fcntl tracing (no fcntl in child,
   confirming no redirect save/undo)
- `/tmp/claude-1000/strace-full.raw`:
   full strace confirming parent never opens /dev/null

### Bash source files

- `~/temp/bash-src/execute_cmd.c`:
   `do_piping()`,
   `execute_disk_command()`,
   `do_redirections()`
- `~/temp/bash-src/builtins/evalstring.c`:
   `parse_and_execute()`,
   fork optimization
- `~/temp/bash-src/builtins/eval.def`:
   eval builtin passes `SEVAL_NOOPTIMIZE`
- `~/temp/bash-src/redir.c`:
   `do_redirections()`,
   `do_redirection_internal()`

### LD_PRELOAD trace files

- `/tmp/claude-1000/trace_open*.c`:
   progressively more comprehensive open/openat hooks
- Only caught parent's `open()`,
   never child's (glibc's internal `open()` bypasses PLT)
