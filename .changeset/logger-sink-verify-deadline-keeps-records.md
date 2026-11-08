---
"@monochromatic-dev/logger-core": minor
---

A sink whose verification runs past `verifyTimeoutMs` no longer loses its records.
The limit now bounds only how long startup waits:
the probe keeps running,
and its answer still decides the sink whenever it arrives,
so a backend that a busy host delayed attaches and receives the records logged meanwhile.
Only a verifier that rejects or throws,
or one that reports its backend unavailable,
retires a sink.
Verification reports now name the sink instead of only its startup index,
state whether records were lost,
and name the setting that changes the wait.

`Sink` gains an optional `name`,
used only in those reports.
Every shipped sink supplies one,
and a sink without it keeps working through the index fallback.
