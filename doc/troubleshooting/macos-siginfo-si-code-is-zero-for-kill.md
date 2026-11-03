# macOS delivers `si_code` 0 for `kill`, not the header's `SI_USER` (Darwin arm64, macOS 27.0)

## Symptom

The cli-git signal relay forwards a signal to its Git child only when `si_code` says a process sent it,
because a
terminal interrupt already reaches Git through the foreground process group.
On macOS the relay never forwarded `kill` or a signal from a forked child,
and three `signal_relay` tests failed
(`signal_during_start_is_relayed_after_the_child_starts`,
`process_signal_is_relayed_and_the_child_outcome_is_reported`,
`term_during_a_held_pre_commit_ends_git_and_reports_143`).
The code had used `SI_USER = 0x10001` and `SI_QUEUE = 0x10002`,
typed from memory,
and compiled without complaint.

## Root cause

The Darwin SDK header defines the constants,
but the kernel does not deliver them for `kill`.
The header on the probe host (`MacOSX.sdk/usr/include/sys/signal.h`,
lines 325 and 326):

```c
#define SI_USER         0x10001 /* [CX] signal from kill() */
#define SI_QUEUE        0x10002 /* [CX] signal from sigqueue() */
```

A handler installed with `SA_SIGINFO` read `si_code` 0 for `kill` from the process itself and from a forked child,
and
`si_pid` was set in both cases.
A Ctrl-C written to a pseudo-terminal master was recorded the same way in the original measurement (recorded 2026-10-31T23:52:16-04:00),
so `si_code` cannot tell a terminal sender from a process sender on this platform.
Only the first two cases were re-run for this write-up;
the pseudo-terminal case is the recorded measurement.

## Verification

Probe compiled and run on the Apple Silicon host `m1` on 2026-11-03 (`uname`:
Darwin arm64;
`sw_vers`:
27.0):

```c
/* file: si.c (compile with: cc -o si si.c) */
#include <signal.h>
#include <stdio.h>
#include <string.h>
#include <sys/wait.h>
#include <unistd.h>

static volatile sig_atomic_t seen_code = -1;
static volatile sig_atomic_t seen_pid = -1;

static void handler(int signo, siginfo_t *info, void *context) {
  (void)signo;
  (void)context;
  seen_code = info->si_code;
  seen_pid = info->si_pid;
}

int main(void) {
  struct sigaction action;
  memset(&action, 0, sizeof action);
  action.sa_sigaction = handler;
  action.sa_flags = SA_SIGINFO;
  sigaction(SIGUSR1, &action, NULL);
  kill(getpid(), SIGUSR1);
  printf("kill from self: si_code=%d si_pid_set=%d\n", (int)seen_code, seen_pid > 0);
  seen_code = -1;
  seen_pid = -1;
  pid_t child = fork();
  if (child == 0) {
    kill(getppid(), SIGUSR1);
    _exit(0);
  }
  waitpid(child, NULL, 0);
  printf("kill from forked child: si_code=%d si_pid_set=%d\n", (int)seen_code, seen_pid > 0);
  return 0;
}
```

```text
kill from self: si_code=0 si_pid_set=1
kill from forked child: si_code=0 si_pid_set=1
```

## Resolution in the wrapper

- On macOS every signal that reaches the wrapper counts as a process sender (`276e673fd9`).
  A missed relay orphans the Git child,
  while a second SIGINT,
  SIGTERM,
  SIGHUP or SIGQUIT reaching Git is harmless.
- Not taken:
  a `tcgetpgrp` foreground-process-group check,
  which cannot separate `kill -INT` from a terminal interrupt.

## What does not work

- Comparing `si_code` with the header's `SI_USER` or `SI_QUEUE` on macOS.
- Constants for a platform that cannot be run are not facts until a probe on that platform has read them.

## Upstream filing decision

Not filed.
The gap is between Apple's header and its kernel,
which this repository cannot change,
and the wrapper already works around it.
