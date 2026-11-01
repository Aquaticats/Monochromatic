# Bash 5.3.9 builtin `kill --signal` fails with an invalid signal specification

## Symptom

Bash's builtin rejects separate arguments `kill --signal TERM <pid>` with:

```text
# Observed Bash builtin diagnostic
kill: -signal: invalid signal specification
```

The original incident targeted an owned search process.
Changing the invocation to `kill -s TERM <pid>` terminated that process,
which was reaped with status `143`.
The independent reproduction in this note uses only signal `0` against each fresh Bash process itself.
No process-terminating signal is needed to reproduce the parser error.

This is an invocation mistake:
the installed builtin's `help kill` lists `-s`,
`-n`,
`-l` and `-L`,
not `--signal`.
The separate [Bash redirection note](bash.md) concerns a different incident.

## Root cause

The source is GNU Bash upstream commit `637f5c8696a6adc9b4519f1cd74aa78492266b7f`,
whose `patchlevel.h` declares patch level `9`.
The read-only shallow clone was obtained from [GNU's upstream Git service][source].
The inspected parser and diagnostic files are byte-identical in the `bash-5.3` release tree
and current upstream patch level `20`;
this is not a removed-option finding.
The Fedora downstream build was not reconstructed.

`builtins/common.h:26` defines an option as exactly a hyphen and one option character:

```c
/* GNU Bash builtins/common.h:26 */
#define ISOPTION(s, c) (s[0] == '-' && s[1] == c && !s[2])
```

`builtins/kill.def:114` recognizes `-s` and `-n` explicitly.
Its zero-signal branch also makes the reproduction non-delivering:

```c
/* GNU Bash builtins/kill.def:114 to 124 */
else if (ISOPTION (word, 's') || ISOPTION (word, 'n'))
  {
    list = list->next;
    if (list)
      {
        sigspec = list->word->word;
use_sigspec:
        if (sigspec[0] == '0' && sigspec[1] == '\0')
          sig = 0;
        else
          sig = decode_signal (sigspec, dflags);
```

`--signal` is neither a recognized short option nor the exact `--` end-of-options token.
The fallback at `builtins/kill.def:157` removes only the first hyphen,
so it tries to decode `-signal` as the signal name:

```c
/* GNU Bash builtins/kill.def:157 to 163 */
else if (*word == '-' && saw_signal == 0)
  {
    sigspec = word + 1;
    sig = decode_signal (sigspec, dflags);
    saw_signal++;
    list = list->next;
  }
```

`trap.c:236` defines `decode_signal`.
It accepts valid numeric or named signals and ends with `return (NO_SIG)` for unmatched input.

```c
/* GNU Bash trap.c:241 to 242 */
if (valid_number (string, &sig))
  return ((sig >= 0 && sig < NSIG) ? (int)sig : NO_SIG);
```

```c
/* GNU Bash trap.c:293, after the named-signal matching loop */
return (NO_SIG);
```

`builtins/kill.def:172` rejects that sentinel before the PID-delivery loop:

```c
/* GNU Bash builtins/kill.def:172 to 176 */
if (sig == NO_SIG)
  {
    sh_invalidsig (sigspec);
    return (EXECUTION_FAILURE);
  }
```

`builtins/common.c:228` emits the exact diagnostic:

```c
/* GNU Bash builtins/common.c:228 to 231 */
sh_invalidsig (const char *s)
{
  builtin_error (_("%s: invalid signal specification"), s);
}
```

The failure concerns option parsing,
not permissions,
the chosen PID,
or unsupported `TERM`.
The independently tested `--signal 0` form fails through the same path.

## Verification

The actual executable is `/usr/bin/bash`,
version `5.3.9(1)-release`,
from Fedora package `bash-5.3.9-3.fc44`.
Its SHA-256 is `379c1bfc53d08815975c647ad7da5d19d8e8a8566ce8097c50231c8b116e5e7b`.
The [verification receipt](../audit/bash-builtin-kill-signal-option-2026-10-18.evidence.json)
retains the source identities,
version/help output,
separate argv values and exact control results.

A runnable non-delivering check is:

```bash
# Check permission/existence for this Bash process; signal 0 does not terminate it.
bash --noprofile --norc -c 'builtin kill -s 0 -- "$BASHPID"'
```

The working catalog,
each with its own fresh Bash process and explicit builtin selection,
is:

- `builtin kill -s 0 -- "$BASHPID"`:
  status `0`.
- `builtin kill -n 0 -- "$BASHPID"`:
  status `0`.
- `builtin kill -0 -- "$BASHPID"`:
  status `0`.

The failing catalog is:

- `builtin kill --signal 0 -- "$BASHPID"`:
  status `1`,
  `kill: -signal: invalid signal specification`.
- `builtin kill --signal=0 -- "$BASHPID"`:
  status `1`,
  `kill: -signal=0: invalid signal specification`.

Each successful process prints a PID matching the spawned child PID,
and each completes without a terminating signal.
The harness clears startup-file injection through `BASH_ENV=/dev/null`
and uses a minimal child environment.
These controls qualify the installed parser and signal-zero check,
not delivery of a terminating signal or every external `kill` implementation.

## Verified workaround

Select the builtin and use its documented `-s` option with a separate signal argument.
The harmless verified form is:

```bash
# Verified builtin grammar; substitute a different signal only for an authorized target.
builtin kill -s 0 -- "$BASHPID"
```

The tradeoff is intentional:
this selects Bash's builtin grammar and job-control behavior,
not an external executable's additional options.
Signal `0` checks the target without delivering a signal;
it is not a replacement for actual termination when termination is required.
The original owned-process cleanup used the same documented grammar with `TERM`.

## What does not work

- Spelling the option `--signal=0` instead of separating `--signal` and `0` still reaches invalid-signal parsing.
- Adding `--` after the unsupported long option does not repair the preceding invalid signal specification.
- Reading an external utility's manual as the builtin's option contract does not establish builtin support.
  This diagnosis does not prescribe an untested external binary as a replacement.

## Upstream filing decision

The repository's `.out-of-scope/` names were checked;
there is no Bash-specific entry.
No upstream filing is authorized or needed for this invocation correction.

1.  Upstream fault:
    no.
    Installed help and the inspected parser agree on the supported spelling.
2.  Upstream fixability:
    no repair is required.
    Adding a new alias would be a feature request,
    not a correction to this observed incident.
3.  Supported use case:
    `-s` is explicitly supported and passes;
    `--signal` is not listed as supported builtin syntax.
4.  Contribution route:
    upstream `README:55` directs real bug reports to `bug-bash@gnu.org` through `bashbug`.
    [GNU's project page][bash] welcomes development discussion.
    No filing candidate reaches this gate,
    and no claim about a general AI-contribution policy is made.
5.  Maintainer intent and duplicate scope:
    an indexed search of the GNU bug-bash archive for this option produced no hit.
    The broader search found [util-linux issue 991][comparable],
    whose complete thread distinguishes shell builtins from the external utility
    and reproduces Bash's corresponding `-verbose` diagnostic.
    It is comparable precedent,
    not a Bash bug or proof that every archive was searched completely.
6.  Prototype:
    not applicable because the upstream-fault gate fails.
    No upstream source change was made.

## Upstream filing artifact

Nothing to add upstream.
Keep this invocation correction and its non-delivering controls in the repository;
do not file an issue or comment for it.

[source]: https://git.savannah.gnu.org/git/bash.git
[bash]: https://www.gnu.org/software/bash/
[comparable]: https://github.com/util-linux/util-linux/issues/991
