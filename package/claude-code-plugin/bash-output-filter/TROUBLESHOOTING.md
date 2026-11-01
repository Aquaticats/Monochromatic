# Troubleshooting

## Sandbox eval chain constraints

The Claude Code sandbox evaluates Bash tool commands through an internal eval chain
that imposes undocumented constraints on command structure.
These constraints prevent several standard shell constructs from working as expected
when the hook rewrites the command.

The hook appends a filter pipe (`2>&1 | node filter.mjs`) to the original command
and must propagate the original command's exit code through the pipeline.
Six approaches were tried before finding one that works reliably.

### Constraint 1: `{ }` compound commands are broken

The sandbox treats `{` as a command name rather than a bash reserved word.

**Wrapper:
** `{ cmd 2>&1; echo ___BOF_EC:$?; } | node filter; (exit $?)`

**Error:
**

```text
bash: command not found: {
___BOF_EC:
bash: command not found: }
/bin/bash: line 1: exit: : numeric argument required
```

**Cause:
**
the sandbox appears to split `;`-separated segments and evaluate them independently.
`{` alone is not a valid command:
 it must be parsed as part of `{ ...; }` compound syntax,
which requires the shell to see the entire construct as one unit.

### Constraint 2: shell state does not persist across `;` boundaries

Variables and `$PIPESTATUS` set after `;` may not reference the preceding pipeline.

**Wrapper:
** `cmd 2>&1 | node filter; _bof=$PIPESTATUS; (exit "$_bof")`

**Error (intermittent):
**

```text
/bin/bash: line 1: exit: of: numeric argument required
```

**Cause:
**
when the sandbox evaluates `;`-separated segments in separate bash contexts,
`$PIPESTATUS` in the second segment does not reference the pipeline in the first.
The variable `_bof` receives a non-numeric value from a different evaluation context,
and `exit "$_bof"` fails.

Removing the intermediate variable (`(exit $PIPESTATUS)` directly) has the same
root cause:
 `$PIPESTATUS` is still evaluated across a `;` boundary.

### Constraint 3: `bash -c` adds a quoting layer that corrupts special characters

Wrapping the command in `bash -o pipefail -c '...'` avoids `;` and `{ }`,
but introduces a second shell parsing pass that transforms characters inside
double-quoted strings.

**Wrapper:
** `bash -o pipefail -c 'node -e "if (!body) { ... }" 2>&1 | node filter'`

**Error:
**

```text
9 | if (\!body) { console.log('no body'); process.exit(1); }
         ^
error: Unexpected escape sequence
    at [eval]:9:5
```

**Cause:
**
the `!` inside the double-quoted `node -e` argument gets escaped to `\!`
somewhere in the eval chain between the sandbox and the inner `bash -c`.
In standard non-interactive bash,
 `!` inside double quotes is literal,
and single-quote escaping (`'\''`) preserves all characters verbatim.
The sandbox's eval chain does not follow standard bash quoting semantics
for nested shell invocations.

### Constraint 4: `< /dev/null` append overwrites filter pipe stdin

The sandbox's eval chain appends `< /dev/null` to the last simple command
in the command string.
If the filter is the last command,
 `< /dev/null` overrides the pipe
that feeds filtered output from the left side of the pipeline.

**Wrapper:
** `set -o pipefail && cmd 2>&1 | node filter`

**Symptom:
**
commands exit 141 (128 + SIGPIPE signal 13) and produce no output.

**Cause:
**
the sandbox transforms the wrapper into
`set -o pipefail && cmd 2>&1 | node filter < /dev/null`.
Bash applies `< /dev/null` to the filter,
 replacing its pipe stdin with `/dev/null`.
The filter reads nothing,
 outputs nothing,
 and the left side of the pipe
receives SIGPIPE when it tries to write.
With `pipefail`,
 the pipeline surfaces the SIGPIPE exit code (141)
instead of the filter's exit code (0).

### Constraint 5: `$?` expands to empty in suffix positions

Shell variable `$?` expands to an empty string in certain positions in the
command string,
 rather than to the expected numeric exit code.

**Wrapper:
** `set -o pipefail && cmd 2>&1 | node filter || (exit $?)`

**Error:
**

```text
/bin/bash: line 1: exit: : numeric argument required
```

**Cause:
**
the sandbox's eval chain either strips `$?` during preprocessing
or evaluates the `|| (exit $?)` suffix in a context where `$?` is unset.
The empty expansion causes `exit ""` which is not a valid numeric argument.
This affects `$?`,
 `$PIPESTATUS`,
 and likely all shell variable expansions
in suffix positions after `||`.

## Working approach: `set -o pipefail && ... && true`

**Wrapper:
** `set -o pipefail && cmd 2>&1 | node filter && true`

This avoids all five constraints:

- **No `{ }`**:
   no compound command grouping
- **No `;`**:
   `&&` chains without segment boundaries
- **No `bash -c`**:
   no extra quoting layer;
   the original command runs in the same
  shell context with identical quoting and expansion semantics
- **`< /dev/null` absorbed by `true`**:
   bash parses
  `cmd | filter && true < /dev/null` with the redirect on `true`,
  not on `filter`,
   preserving the filter's pipe stdin
- **No shell variables**:
   `&& true` uses no `$?` or `$PIPESTATUS`;
  exit code propagation relies entirely on `&&` short-circuit semantics

### Exit code propagation

`set -o pipefail` makes the pipeline return the exit code of the last (rightmost)
command that exits non-zero,
 rather than the rightmost (the filter,
 which always exits 0).
The `&&` operator ensures the pipeline only runs after `pipefail` is enabled,
and since `set -o pipefail` always succeeds,
 the pipeline always executes.

The `&& true` suffix propagates the pipeline's exit code through `&&` semantics:

- **Pipeline succeeds (exit 0):
  ** `&& true` runs,
   `true` exits 0,
   overall exit code is 0
- **Pipeline fails (exit N):
  ** `&&` is not taken,
   `true` does not run,
  overall exit code is the pipeline's exit code (N)

No shell variables are read,
 expanded,
 or assigned.

### Graceful degradation

If the sandbox splits `&&` the same way it splits `;` (not observed,
 but possible):

- `set -o pipefail` runs in its own context and its effect is lost
- `cmd 2>&1 | node filter && true` runs without `pipefail`;
   exit code is the filter's (0)
- Output filtering still works;
   only exit code propagation is lost
- No crash,
   no garbled output,
   no confusing error messages

### Limitation: chained commands

For commands with `&&` or `||` chains (e.g. `git pull && git push`),
the filter pipe binds to the last command due to operator precedence:

```text
set -o pipefail && git pull && git push 2>&1 | node filter && true
```

Parsed as:
 `set -o pipefail && git pull && (git push 2>&1 | node filter) && true`

Only `git push`'s output is filtered.
This is the same behavior as the original approach and all subsequent attempts:
filtering the full chain would require grouping constructs (`{ }` or `()`),
which the sandbox does not support.
