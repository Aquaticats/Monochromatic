# Where a commit's time goes on 0.6.1-alpha.11, measured against 0.6.1-alpha.10

What this answers:
whether the wrapper's 0.6.1-alpha.11 release, installed on this machine on 2026-11-11,
made a commit slower than 0.6.1-alpha.10,
and if not, where the extra seconds in the real repository come from.

The bar is the same as `doc/planning/cli-git-commit-latency-under-many-agents.md`:
a commit through the wrapper takes no more than 5 s longer than native Git,
and commits never fail.

## The question

The timing log of `/var/home/user/Monochromatic` reads, for landed commits:

- `0.6.1-alpha.10`, its last hour: median 2,645 ms, ninetieth percentile 14,391 ms.
- `0.6.1-alpha.11`, from its install: median 11,583 ms, ninetieth percentile 34,132 ms.

The two windows were not measured under the same load:
the release gate's container build ran throughout alpha.11's window,
the load average stood near 120, and `/proc/pressure/io` reported `full avg10` near 37.

(This record is being written as the measurement runs.)
