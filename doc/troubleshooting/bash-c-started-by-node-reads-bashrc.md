# Bash 5.3 `bash -c` started by Node with piped output reads `~/.bashrc`, so a test that fakes `PATH` runs the real commands

## Symptom

A Node test starts `bash -c <script>` with `child_process` and a `PATH` that lists a folder of stand-in programs first.
The script runs the real `fuser`,
`umount` and `mountpoint` instead of the stand-ins,
and prints lines that only the user's shell startup can print
(`Error: $HOME must be set to run brew.` from the Homebrew line of `~/.bashrc`).
`echo $PATH` inside the script shows a longer `PATH` than the one Node passed.

Measured 2026-11-07 on this host while testing the 860 mount unit's new `ExecStop`
(`package/repository-move/cli/src/guided-runner.unit.test.ts`):
a run with the `PATH` of stand-ins printed the real holders of `/var/mnt/encrypted`
and `umount: /var/mnt/encrypted: must be superuser to unmount.`
That is the real `umount` on the real mount.
It failed only because the test ran as an ordinary user;
as root it would have unmounted the drive.

## Root cause

`bash(1)`,
section INVOCATION,
read with `man bash` on bash 5.3.15 (`GNU bash, version 5.3.15(1)-release (x86_64-redhat-linux-gnu)`):

```text
Bash attempts to determine when it is being run with its standard input connected to a network connection,
as when executed by the historical and rarely-seen remote shell daemon, usually rshd, or the secure shell daemon sshd.
If bash determines it is being run non-interactively in this fashion, it reads and executes commands from ~/.bashrc,
if that file exists and is readable.
...
The --norc option inhibits this behavior
```

Node gives a child with `stdio: 'pipe'` a socket pair,
not an anonymous pipe,
so standard input is a socket and bash takes it for a remote shell.
The Unix socket is the trigger,
not the use of Node:
the same `bash -c` with standard input ignored,
or fed from a shell pipe,
did not read `~/.bashrc` (below).
I read the manual page only;
the bash source that tests for the socket was not read.

## Verification

Disposable home with a `.bashrc` that exports a marker,
bash 5.3.15,
Node 26.13.0:

```js
// run from a folder under ~/temp/agent; HOME names a folder whose .bashrc holds `export FROM_BASHRC=yes`
const { spawnSync } = require('node:child_process');
const env = { PATH: '/usr/bin:/bin', HOME: home };
const marker = ['-c', 'echo ${FROM_BASHRC:-unset}'];
spawnSync('bash', marker, { env, encoding: 'utf8' }).stdout;                                       // 'yes'
spawnSync('bash', ['--noprofile', '--norc', ...marker], { env, encoding: 'utf8' }).stdout;         // 'unset'
spawnSync('bash', marker, { env, stdio: ['ignore', 'pipe', 'inherit'], encoding: 'utf8' }).stdout; // 'unset'
```

Output of the three,
in order:
`yes`,
`unset`,
`unset`.
From a shell,
`echo | HOME=<home> PATH=/usr/bin:/bin bash -c 'echo ${FROM_BASHRC:-unset}'` printed `unset`.

## Verified workarounds

- Start the shell with `bash --noprofile --norc -c <script>`.
  The tradeoff:
  nothing the user's startup defines (functions,
  aliases,
  `PATH` additions) exists in the script,
  which is what a hermetic test wants and what a script that needs the user's tools does not.
- Give the child `stdio: ['ignore', 'pipe', 'pipe']`:
  standard input is then not a socket.
  The tradeoff:
  the script cannot read standard input.
- A test that fakes commands must also point the script at a scratch mount point and scratch device names,
  not only a scratch `PATH`:
  when a stand-in is not found the real command runs on the real name.
  `package/repository-move/cli/src/guided-runner.unit.test.ts` does both and keeps the real unit's names out of the run.

## What does not work

- Setting `PATH` in the child environment alone:
  `~/.bashrc` replaces it after bash starts (shown above).

## Upstream filing decision

Nothing to file.
Constraint 1 fails:
the behavior is documented in the bash manual with the option that turns it off,
so it is not a defect.
No tracker search,
prior-art search or trust-system search was run,
because no filing is drafted.
