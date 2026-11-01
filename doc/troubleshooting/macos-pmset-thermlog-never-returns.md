# macOS `pmset -g thermlog` never returns and prints nothing,
so a thermal probe in an agent command wedges the call;
`pmset -g therm` is the one-shot form

## Symptom

Probing an Apple Silicon machine's thermal state over `ssh` before a benchmark
run,
 with what reads like a status query,
 hangs until the caller's timeout:

```bash
ssh m1 'pmset -g thermlog'
# no output at all, and the command never exits
```

In the meow cache key hash vet's Apple M1 run this consumed a 120-second command
budget on the first machine probe and produced nothing to show for it.
 The
sibling query on the same line,
 `pmset -g therm`,
 answers immediately:

```bash
ssh m1 'pmset -g therm'
# Note: No thermal warning level has been recorded
# Note: No performance warning level has been recorded
# Note: No CPU power status has been recorded
```

The two differ by four characters and behave nothing alike.

## Root cause

`thermlog` is not a query.
 `man pmset` describes it as a log,
 not a snapshot,
and it is documented next to the other streaming form:

```text
-g thermlog shows a log of thermal notifications that affect CPU speed.
Not available on all platforms.
-g assertions displays a summary of power assertions. ...
-g assertionslog shows a log of assertion creations and releases.
```

The `...log` forms subscribe to a notification stream and print an entry when one
arrives.
 On a machine that is not thermally limited no notification ever
arrives,
 so the process produces zero bytes and stays alive indefinitely.
 There
is no "print the log so far and exit" behaviour to fall back on:
 the empty output
is not a truncated answer,
 it is the whole answer for a machine with nothing to
report.

`therm`,
 by contrast,
 reads the current warning levels and exits.
 Its three
`Note:` lines name the three things it reports when they have been recorded:
 a
thermal warning level,
 a performance warning level,
 and a CPU power status.
 The
form those take when they are present was not observed here,
 because the machine
never registered one;
 `CPU_Speed_Limit` is the string commonly cited for the CPU
power status,
 and that is recall from training rather than something this
investigation saw.

The "Not available on all platforms" line in the manual is a second trap:
 on a
platform where `thermlog` is unavailable the silence looks identical.

## Verification

- Machine:
   `MacBookAir10,1`,
   Apple M1,
   macOS 27.0 build `26A428`,
   Darwin 27.0.0,
   reached over `ssh`.

Bounded reproduction,
 which is the only safe way to run it:

```bash
timeout 20 ssh -o BatchMode=yes m1 'pmset -g thermlog' > out.log 2>&1
echo "exit=$?"     # exit=124  (timeout killed it)
wc -c < out.log    # 0         (nothing was printed in 20 seconds)
```

The one-shot form on the same machine,
 in the same session:

```bash
ssh -o BatchMode=yes m1 'pmset -g therm'
# Note: No thermal warning level has been recorded
# Note: No performance warning level has been recorded
# Note: No CPU power status has been recorded
# exit 0, immediate
```

Used as intended,
 `pmset -g therm` ran 100 times in that vet,
 before and after
each of 50 benchmark runs,
 always returning immediately,
 and `CPU_Speed_Limit`
appeared in none of them.

What this probe does and does not establish:
 it reports the warning levels macOS
has recorded,
 so an absence of `CPU_Speed_Limit` is evidence that macOS did not
register a limit.
 No positive control was run for it,
 because forcing a genuine
thermal limit on the owner's machine is not something to do on purpose,
 so a
campaign that wants to rule out throttling should also compare run order:
 later
runs being at least as fast as earlier ones is independent evidence.

## Verified workarounds

- Use `pmset -g therm` for a thermal snapshot.
   Tradeoff:
   it reports recorded
  warning levels rather than a temperature or a live frequency,
   so it answers
  "did macOS register a limit" and not "how hot is it".
- If a stream is genuinely wanted,
   bound it from the caller's side and then clean
  up on the machine:
  `timeout <n> ssh host 'pmset -g thermlog'`,
   followed by
  `ssh host 'pgrep -f thermlog'` and a `kill` of what it lists.
   The `timeout` bounds
  the caller;
   it does not stop the remote process
  ("What does not work").
   Tradeoff:
   the caller must pick `<n>`,
   the cleanup is a
  second round trip that is easy to skip,
   and on a healthy machine the result is
  always an empty file and exit 124,
   which is indistinguishable from the platform
  not supporting `thermlog` at all.
- For continuous thermal data,
   `powermetrics` is the tool that actually samples,
  but it requires root.
   Not used in the vet,
   because running `sudo` on the owner's
  machine for a measurement is a larger authorization than the measurement needed.

## What does not work

- `pmset -g thermlog | head -5` to bound it.
   `head` exits after five lines,
   but
  no lines ever arrive,
   so `head` waits and the pipeline waits with it;
   a
  `SIGPIPE` never fires because nothing is ever written.
   Measured:

  ```bash
  timeout 12 ssh -o BatchMode=yes m1 'pmset -g thermlog | head -5' > out.log 2>&1
  echo "exit=$?"     # exit=124
  wc -c < out.log    # 0
  ```

- Assuming the caller's `timeout` stops the remote process.
   It does not.
   After
  three bounded or interrupted calls,
   the machine still held every one of them:

  ```bash
  ssh m1 'ps -Ao pid,etime,args | grep "[p]mset"'
  # 3985  02:02:38  zsh -c pmset -g thermlog 2>&1 | head -5 && uptime && sysctl ...
  # 3986  02:02:38  pmset -g thermlog
  # 8511     21:57  pmset -g thermlog
  # 8795     00:38  zsh -c pmset -g thermlog | head -5
  # 8796     00:38  pmset -g thermlog
  ```

   The oldest had been running for two hours,
   started by the probe that first
  exposed the symptom.
   They are idle,
   so they cost almost nothing,
   but they
  accumulate one pair per attempt and they are the caller's litter on someone
  else's machine.
   Clean up explicitly:

  ```bash
  ssh m1 'pgrep -l -f thermlog'      # list, by PID
  ssh m1 'kill <pid> ...'            # never pkill -f from the calling shell
  ssh m1 'pgrep -f thermlog || echo "no thermlog processes remain"'
  ```
- Killing the local `ssh` by matching the remote command with `pkill -f`.
   The
  pattern `pmset -g thermlog` appears in the local shell's own command line,
   so
  `pkill -f 'pmset -g thermlog'` matches and kills the caller.
   This happened once
  in the vet:
   the call that ran the `pkill` terminated with exit 144 and no output,
  and so did the backgrounded call it was meant to clean up after.
   Bound the call
  with `timeout` instead,
   or kill by PID after looking it up.
- Reading the empty output as "no thermal events".
   It is consistent with that,
  and equally consistent with the platform not supporting `thermlog`,
   with the
  command having just started,
   and with the stream having nothing to say yet.
   The
  answer has to come from `pmset -g therm`,
   which distinguishes them.

## Upstream filing decision

Nothing to file,
 and nothing was filed or commented.
The 6-constraint check stops at the first constraint:

1. **Is it really upstream's fault?
   ** No.
    `pmset -g thermlog` is documented as a
   log rather than a query,
    the manual pairs it with `-g assertionslog`,
    which it
   describes the same way,
    and the one-shot form the caller wanted exists under a
   name four characters shorter.
    Apple's tool did what its manual says.
    The
   mistake was reaching for a plausible-looking subcommand without reading the
   page.
    Constraints 2 through 6 are not reached.

This is also not a repository issue:
 the durable value here is the rule that a
`...log` subcommand of a system tool is a stream,
 that an agent command must bound
anything it has not confirmed terminates,
 and that bounding the caller is not the
same as stopping the callee on a remote machine.
