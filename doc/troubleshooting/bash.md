# Bash redirection ordering: `2>&1 > file` splits streams instead of merging them

## Symptom

Capturing combined stdout+stderr to a file with `cmd 2>&1 > file; cat file`
produces output where stderr and stdout lines appear in an order that
no longer reflects the execution sequence:

- Lines that the program clearly emitted earlier appear after lines
  emitted later.
- Some lines are missing from the file entirely.
- When the outer process captures both terminal and file (the shape of
  Claude Code's Bash tool),
   the two streams interleave nondeterministically.

Discovered while verifying that `mise run //pkg:build` completed before
`bun test` started in the `buildAndTest` task.
 Build-completion lines
appeared *after* the test results in the captured output,
 suggesting
concurrent execution;
 in reality the commands ran sequentially and the
ordering misperception came from how bash applied the redirects.

## Root cause

Bash processes redirections left-to-right.
 Each redirect mutates the
file-descriptor table at the point where it appears,
 and subsequent
redirects see the post-mutation table:

```bash
cmd 2>&1 > file
```

1.  `2>&1`:
     duplicate fd 1 (currently the terminal) onto fd 2.
     Stderr
    now points to the **terminal**.
2.  `> file`:
     redirect fd 1 to `file`.
     Stdout now points to **file**.

Final state:
 stderr goes to the terminal,
 stdout goes to the file.
 `cat
file` shows only stdout.
 The terminal separately showed stderr at the
time the command ran.
 The two recombine only if an outer process
captures both,
 and the interleaving order is then a function of OS-level
scheduling,
 not source order.

Bash's manual documents this behaviour explicitly (`man bash`,
 section
"REDIRECTION");
 the trap is that the colloquial reading of `2>&1 > file`
("redirect stderr to stdout,
 then send everything to file") inverts the
actual order of operations.

## Verification

Reproduce the split:

```bash
{
  echo 'on stdout';
  echo 'on stderr' >&2;
} 2>&1 > /tmp/out
cat /tmp/out
# Prints: "on stdout"
# Terminal also shows: "on stderr"  (printed before the cat, in real time)
```

Now with the correct ordering:

```bash
{
  echo 'on stdout';
  echo 'on stderr' >&2;
} > /tmp/out 2>&1
cat /tmp/out
# Prints: "on stdout" and "on stderr" in source order.
```

## Verified workaround

Place the file redirect **before** the fd duplication:

```bash
# Both stdout and stderr go to the file, preserving order
cmd > file 2>&1; cat file

# Equivalently: subshell redirect captures both
(cmd) > file 2>&1; cat file
```

With `> file 2>&1`:

1.  `> file`:
     redirect fd 1 to `file`.
2.  `2>&1`:
     duplicate fd 1 (now `file`) onto fd 2.
     Stderr also points to
    **file**.

Tradeoff:
 none for the redirect ordering itself.
 When the goal is to
**see** combined output rather than save it,
 drop the file and use
`cmd 2>&1` alone,
 which pipes both streams to the same destination
without any file-handle reordering subtlety:

```bash
cmd 2>&1
```

This is what worked for verifying the `buildAndTest` task's execution
order in the original incident.

## What does not work

- `cmd &>file` (bash-specific shorthand):
   equivalent to `> file 2>&1`,
  so it works in bash.
   Avoid in POSIX-portable scripts:
   `&>` is a
  bashism,
   not portable to dash or other strict-POSIX shells.
   The
  longhand `> file 2>&1` is both correct and portable.
- Capturing with `>(tee file)` process substitution:
   also works but
  introduces a child process and a write order that depends on tee's
  buffering.
   For the diagnostic case where order matters,
   the simple
  `> file 2>&1` is preferable.

## Why we do not file this upstream

The behaviour is a documented feature of bash,
 not a defect.

1.  **Is it really upstream's fault?
    ** No. `man bash` documents the
    left-to-right redirection order explicitly.
2.  **Can upstream fix it?
    ** Nothing to fix.
     Any change to the order
    would break decades of working scripts.
3.  **Are they supporting this use case?
    ** Both forms (`2>&1 > file` and
    `> file 2>&1`) are supported and produce well-defined behaviour;
    they just differ.
4.  **Will they likely fix it?
    ** N/A.
5.  **Have we prototyped a minimal fix?
    ** N/A.

Decision:
 keep this doc as an internal reference;
 do not file upstream.
