# Claude CPU cap and thermal watch

> Moved 2026-10-29 by the generic package category elimination
> (`doc/planning/generic-category-elimination.md`);
> paths and names below this note are historical:
> `package/dev-script/claude-cpu-cap` is now `package/claude-cpu-cap/daemon`.

## Purpose

On 2026-10-13 the user asked why Claude sessions did not use all CPU threads on the 16 core, 32 thread Ryzen 9 7950X
host (62 GiB RAM, Bazzite, kernel 7.2.9).
The session found a stale limit, lifted it, capped Claude's own work at a chosen share, and added a watcher.
It then moved on to whether the machine runs too hot, which is still being measured.
This file is the running record.
The operating instructions live in `doc/runbook/claude-cpu-cap.md`.

## User decisions

- The whole account may use all 32 threads (a persistent `AllowedCPUs=0-31`).
- Claude tool commands, including gate jobs run through `gate-slot.ts`, share one cap.
  It was 24 threads, then 30 by user request ("still exploring the new CPU"), then 28 after the tone-down rule.
  The user raised it back to 30 at about 18:55 EDT because their fan curve is more aggressive (full speed from 61
  degrees instead of 65). The curve read back from CoolerControl still showed 65 at that
  moment; the user saved it a few minutes later and it now reads 100% at 61 degrees (30%/45%/60% at 40/50/60).
- Tone-down rule from the user: if the system gets bad, lower the cap 2 threads at a time.
- IO pressure is non-actionable (the user's drives are slow) and is not watched.
- The temperature alert is a throttling alert (user choice, 2026-10-13): Tctl at or above 95 degrees Celsius and the
  average clock at least 300 MHz below the boost-off clock of 4490 MHz (first 100 MHz as chosen, raised to 300 the same
  evening at the user's request after 100 flapped). Plain 95 degrees no longer alerts, because
  Zen 4 regulates itself to 95 by design. The earlier plain limit of 95 was raised from 90 by the user.
- No deliberate stress tests: the user needs the machine to work. Temperatures are judged from real work.
- Boost and PBO stay off. The user disabled them (`/sys/devices/system/cpu/cpufreq/boost` is 0).
- Commit eagerly.
- The user allowed storing the CoolerControl token on disk. They planned to rotate it on 2026-10-14, then said on
  2026-10-18 that they are not rotating it and that sessions may keep using it (`~/.config/claude-cpu-cap/cc-token`).
  The watcher stopped reading it on 2026-10-18 (see the section on the board swap).
- The user's own fan changes no longer count as an excuse for an alert: they are confident that tinkering will not
  push the CPU to 95 degrees as before, so every alert is real and no alert is exempt (2026-10-13).
- The 4-hour stop-working check is pushed back to 4 hours after every weird event (user rule, 2026-10-13).
  Weird means any alert except IO pressure, a cap change, `WATCHER-ERROR`, an unresolved `FAN-READ-FAILED`
  (no longer emitted since 2026-10-18), or a `WATCHER-STOPPED` the session did not cause.

## Current state

- `user.slice` and `user-1000.slice` have `AllowedCPUs=0-31`, persisted in `/etc/systemd/system.control`
  (replacing the drop-in from 2026-02-22 that held `0-15` and left threads 16 to 31 idle).
- `claude-cpu-watch.service` (user unit, enabled, linger on) re-applies the quota to the `claude-code-bash` cgroup.
  Claude Code creates that cgroup anew in every boot and has no CPU limit setting, so a service has to do it.
  The thread count is read from `~/.config/claude-cpu-cap/threads` every 2 seconds
  (28 now, lowered from 30 at 20:15:53 after the 95.3 degree alert).
- The watcher alerts on CPU pressure 40, memory pressure 10, scheduler lag 250 ms and thermal throttling (300 MHz).
  The limit is `CPU_WATCH_LIMIT_THROTTLE_MHZ`, default 300 in
  `package/dev-script/claude-cpu-cap/src/cpu-watch.ts`. The user asked for a better value after
  100 MHz flapped. Among 125 samples at Tctl 95 or more (an 8 minute plateau at 133 W) the shortfall was median 83,
  p95 183, p99 221, max 255 MHz, and 0 percent reached 300, so 300 sits above normal regulation.
  The pressure limit of 40 was a first guess and is uncalibrated.
- It logs a sample every 10 seconds to `~/.local/share/claude-cpu-cap/samples/<date>.csv` (14 days kept):
  temperatures, package watts, per-chiplet watts, clock, pressure, load, Corsair fan RPMs and probe,
  motherboard readings, a burst flag (2-second samples for 90 seconds after a 25 watt step), and since 20:55 EDT the
  discrete GPU and integrated graphics temperature and watts.
- The fan profile "CPU Cooler (Silent)" in CoolerControl was replaced by this session with:
  30% at 40 degrees and below, 45% at 50, 60% at 60, 85% at 70, 100% at 75 and above, from Tctl, no smoothing.
  The previous version (which already had the user's own 75-degree to 100% edit) is saved at
  `~/temp/agent/cc-spec/silent-before.json`; restore it with `PUT /profiles`.
  The user then changed it themselves and will keep adjusting it, because they cannot tolerate much noise while the
  machine is quiet.
  Current curve: 30% at 40, 45% at 50, 60% at 60, 100% at 61 and above (saved by the user about 18:57 EDT).
  The fan profile belongs to the user now; this session does not edit it unless asked.
- Monitoring ended by the user at about 21:25 EDT on 2026-10-13 ("enough with the monitor and alert - we already
  determined our current setup is defective"): the session's Monitor and the 4-hour check were stopped and are not
  re-armed, and alerts are no longer acted on. Before that the check had been pushed back after each alert (the last
  at 21:14:53). The service `claude-cpu-watch.service` keeps running by design: it enforces the cap and logs samples,
  which the paste comparison needs. Its alert lines only go to the journal.
- The 20:11:41 CPU pressure alert (46.1, limit 40) came at cap 30 and 130 W. It was the first sustained load in the
  log: about 55 seconds at 125 to 136 W, load average up to 57 on 32 threads, pressure up to 67.8, and it cleared at
  17.2 within a minute. Tctl stayed below 95, so the cap was left at 30. Its numbers are in the evidence section.
- The 19:16:45 memory pressure alert (16.4, limit 10) came at cap 30 and 132 W; it cleared at 3.7 within about a minute.
  Tctl was 77.6 degrees and 41 GiB were available. The `claude-code-bash` cgroup held 2 GB of its 8 GB limit, and the
  host ran a 5 GB `qemu-system-x86` process. The cap was left at 30: one short blip is not a degraded system, and
  nothing shows the CPU cap caused it. A repeat or a longer episode is the trigger to lower it.
- The user raised the shared `claude-code-bash` memory limit from 8 GB to 32 GB at about 19:45 EDT.
  The live `memory.max` was written (reads 32000000000) and `CLAUDE_CODE_TOOL_MEMORY_LIMIT` in
  `~/.claude/settings.json` is now 32000000000 for sessions started later.
  `doc/troubleshooting/systemd-oomd-reaps-terminal-scope-with-the-agent-in-it.md` still shows 8000000000 as its
  example and says the limit must sit above the heaviest build's peak; it was not edited.
  Open check: a session started with the old environment might write 8 GB back; compare `memory.max` later.
- Other sessions were told about each change by cross-session message
  (the four peers got the memory limit change at about 19:46 EDT).
  `gate-slot.ts` (in `~/temp/agent`, outside the repo) no longer wraps commands in a `systemd-run` scope,
  so gate commands stay under the shared cap.

## Evidence so far

- Before the change, threads 0 to 15 were 15% to 98% busy and threads 16 to 31 were at 1% to 3%, with load near 30.
- One core does AES-XTS at about 4.9 GiB/s in memory (VAES AVX-512 driver), so encryption compute is not the limit.
  While writing 4 GiB to `/var/home`, the encryption workers sat near 0.3% CPU and the machine was 77% idle.
- That write ran at 88.5 MB/s; raw reads of the NVMe partition ran at 187 MB/s (one request in flight)
  and 325 MiB/s (8 parallel readers). Other sessions were doing IO at the time, so these are lower bounds.
- The NVMe is healthy (2% worn, 0 errors, 48 degrees).
  Its PCIe link runs at 8 GT/s x4 while it reports 16 GT/s as maximum.
  The cause of the low disk speed is not found. The volumes have no `no_read_workqueue` or `no_write_workqueue` flag.
- Hot episodes: 94.8 to 96.1 degrees at 135 W and 4483 MHz (boost off), with CPU pressure near 57.
  A later alert at 91.1 degrees had the fans at 2000 to 2600 RPM. Idle runs about 54 degrees at 60 W.
- Analysis of the sample log, 18:11 to 18:44 EDT on 2026-10-13 (336 rows, 173 of them burst rows, cap 28 throughout;
  scratch scripts `~/temp/agent/analyze-samples.ts`, `analyze-columns.ts`, `analyze-mapping.ts`, outside the repo):
  - No settled data above 80 W exists yet. The loads were three short bursts, and Tctl rose from about 65 to 94.5
    degrees in about 25 seconds as package power climbed from 105 to 149 W, then fell within 4 to 6 seconds
    once power dropped.
  - Time at or above Tctl 85 was 42 seconds, 90 was 16 seconds, 93 was 6 seconds, and 95 was 0 seconds in the window.
    Highest value 94.5 degrees at 149 W for one 2-second sample, with the first fan at about 2030 RPM,
    its highest observed speed, so the fans were already at their top at the spike.
  - Settled Tctl, 40 to 80 W: 55 degrees at 45 to 55 W, 58 at 60 to 70 W, 62 at 70 to 80 W.
    The quiet-load slope is 0.29 degrees per watt (130 rows).
    A straight-line extension to 149 W gives about 82 degrees, so the 94 degree spikes sit about 12 degrees above
    that line; this is an extrapolation, not a measurement, and fan speed also rises with load.
  - Verdict for this window: about 90 degrees was a short peak, not a settled value.
    The earlier alerts at 17:27 to 17:49 lasted at least 6 seconds each and are before the log starts,
    so whether a sustained 120 W or more settles below 95 is still unanswered.
  - The first chiplet is cores 0 to 7: in 251 of 253 rows with a clear gap, the chiplet with more watts was the one
    with the hotter CCD (Tccd1 vs Tccd2), consistent with that mapping.
    Chiplet 1 drew about 53 W against 48 W for chiplet 2 at the 149 W peak.
  - The Corsair probe stayed between 25.5 and 26.2 degrees across 42 to 149 W (correlation with watts -0.12).
    It does not follow heat from the cooler, so it measures something near room temperature.
    The cause is not known (probe not in the exhaust stream, loose contact, or a slow sensor).
    It cannot yet serve as the heatsink-exhaust reference the user wanted.
  - Motherboard readings: `board3_c` follows load (52 to 94 degrees, correlation 0.83 with watts), so it matches the
    CPU reading. `board5_c` moves a little with load (39 to 42). `board1_c`, `board2_c`, `board4_c`,
    `board6_c` move at most 2 degrees. `acpi_c` is constant at 16.8, so it is not informative.
  - Pressure stayed at or under 11.4 and the one-minute load at or under 20 in the window,
    so the 40 limit was not tested.
- First sustained load, 20:11:38 to 20:12:28 EDT on 2026-10-13 (24 rows, cap 30; script
  `~/temp/agent/analyze-episode.ts`): package power median 129.7 W (124 to 136), average clock 4490 MHz,
  Tctl median 88.8 degrees (86.5 to 92, never 95), Tccd1 86.6 and Tccd2 82.9, first fan at 2009 RPM (its top speed),
  Corsair probe 26.0 degrees. Tctl rose from 75.6 to 87.3 within 10 seconds of the load starting, then stayed flat
  at 88 to 91 for about 40 seconds, and fell to 64 within 20 seconds of the load ending.
  Reading: at 130 W with the fans at their top speed, about 89 degrees is a plateau over a 40 second window, not a
  short peak. This corrects the earlier verdict that about 90 was only a peak (that window had no sustained load).
  The 94 to 96 degree episodes happened at 135 to 150 W. A longer load could still creep up as the heatsink warms;
  one 55 second episode cannot rule that out. The quiet-load slope of 0.29 degrees per watt under-predicts this
  plateau (it predicted about 76 degrees at 130 W), so the slope is steeper at high power.
  Chiplet 1 drew 42.9 W against 38.8 W for chiplet 2 and ran 3.7 degrees hotter.
- Second sustained load, 20:15:21 EDT onward on 2026-10-13 (cap 30, then 28 from 20:15:53): Tctl pinned at
  95.3 to 95.4 degrees for more than a minute (flat to 0.1 degree), package power 134 to 140 W (down from 146 W at the
  start), average clock down from 4489 to about 4350 to 4430 MHz, first fan at 2009 to 2036 RPM,
  CPU pressure only 2 to 3,
  one-minute load 17 to 23. The chip was regulating itself to the 95 degree target.
  - TechPowerUp (Ryzen 9 7950X cooling requirements, fetched 2026-10-13) states that Zen 4 is designed to sit at
    95 degrees under load: it boosts clocks and voltage until 95, then regulates to stay at 95 by lowering clocks,
    and weaker cooling shows up as lower clocks, not higher temperature. AMD is quoted as saying the processor is
    designed to run at that temperature 24/7 without risk of damage.
    So a 95 degree reading under sustained load is expected behavior, and the 95 degree alert fires on it by design.
  - The cap was binding in the 20:11 episode (load average up to 57 on 32 threads) but not in this one
    (load 17 to 23, below the cap), so lowering the cap from 30 to 28 at 20:15 cannot have reduced this heat.
  - Best reference, same chip: Noctua's chart "Thermal performance AMD Ryzen 9 7950X (180W)" (image fetched and read
    2026-10-13 from the NH-D15 G2 versus NH-D15 performance data page; the metric is "delta of average core to ambient
    temperature"). The original NH-D15 (G1, the same heatsink as the D15S) reads 46.4 degrees with the AM5 offset
    mounting and 48.4 without it; the G2 versions read 44.1 to 44.8 with offset. That is about 0.26 degrees per watt.
    Here the average of the two CCD temperatures during the plateau is about 91.7 degrees (Tccd1 95, Tccd2 88.5),
    which is 65.7 over the 26 degree probe at 133 W, about 0.49 degrees per watt. At Noctua's ratio, 133 W would give
    about 35 over ambient, near 61 degrees. The measured gap is about 30 degrees, for the same chip, the same heatsink
    and a lower power. It is not a sensor error: the chip also lowers its clock at that temperature.
    Caveats: Noctua's ambient, fan speeds and paste are its own; the power scaling is linear; the probe is a loose
    ambient reference. Noctua's own fan-count and G1 versus G2 differences are 1 to 3 degrees each; no source read
    here puts any of these caveats near 30 degrees.
  - Reference numbers with their limits. GamersNexus (NH-D15 G2 review): the 2023 NH-D15 (the G1 design) ran
    53.3 degrees over ambient at about 198 W with 100% fans, on a Ryzen 9 3950X overclock (not Zen 4; ambient not
    stated). That is about 0.27 degrees per watt. Here: 95.4 minus the probe's 26.0 is 69.4 degrees over ambient at
    137 W, about 0.51 degrees per watt, roughly 1.9 times the reference. At the reference ratio, 137 W would sit near
    63 degrees. Noctua states the G2 is 2 to 3 degrees better than the original at 180 to 250 W, and GamersNexus
    measured about 1 degree, so the G1 versus G2 difference is small next to this gap.
  - TechPowerUp ran a single tower Noctua NH-U14S (rated 165 W) on the same chip: at full fan it held frequencies well
    above 5 GHz on all threads at 95 degrees, with a power chart that spans 100 to 225 W. This setup, with a bigger
    dual tower cooler, reaches 95 degrees at about 137 W and 4.4 GHz with boost off.
    Different platform, case, ambient and fan setup, so this is a rough comparison, not a controlled one.
  - Caveats on the 0.51 figure: a Zen 4 chip may be harder to cool than the 3950X for reasons not checked here
    (heat spreader thickness, die layout), and the probe is a loose ambient reference (see hardware facts).
    A gap this size is still more than the G1 versus G2 difference, so the contact or airflow path is the suspect.
- The same load continued: 20:16:30 to 20:25:00 EDT (59 rows, cap 28), Tctl median 95.4 (95.3 to 95.6), package power
  median 132.9 W (123 to 137), average clock median 4407 MHz (low 4289), fans at their top speeds. So the setup settles
  at about 95.4 degrees, 133 W and 4.3 to 4.4 GHz with boost off;
  this answers the earlier question about a settled value.
  The new throttle alert fired at 20:24:23 (158 MHz shortfall), cleared at 20:24:39, and fired again at 20:24:45
  (197 MHz); it flaps because the average clock jitters around the 100 MHz limit. The cap stayed at 28 (the load average
  was 21 to 45, mostly under 32 threads, and the chip regulates itself at 95 degrees).
- User's mounting answer (about 20:20 EDT): they believe all the checklist steps were done right and are considering
  buying the NH-D15 G2 in its AM5-suited version.
  Noctua sells no AM5-only SKU. The regular G2 includes the AM5 offset bars and, with the offset, matches or beats the
  LBC variant on AM5; LBC suits AM5 without the offset or for the lowest CCD and IOD temperatures;
  HBC is not recommended
  for AM5 (Noctua's G2 version FAQ and contact quality article, fetched 2026-10-13). The G2 gain over the G1 is
  2 to 3 degrees by Noctua and about 1 degree by GamersNexus, small next to the measured gap.
  The zero-cost next step is to lift the cooler and inspect the paste imprint and the base;
  no purchase decision is made.
- Other components as a cause (the user asked at about 20:50 EDT whether they could be the problem):
  - Case air: the probe on the heatsink top stayed at 26.0 degrees at 133 W, so intake and case air are not hot. The
    discrete GPU read 48 to 52 degrees at 28 to 32 W while idle (it was not under load at the check); the integrated
    graphics read 2.2 W and 53 degrees (a 16 W reading appeared once earlier). The GPU is not logged under CPU load yet.
  - Heat load: 133 W at 4.4 GHz is within what the chip is built for (TechPowerUp quotes a 170 W TDP and a 230 W
    limit), so the chip is not producing unusual heat. Package power minus the two chiplets (85.4 W) leaves about
    48 W for
    the IO die, SoC and fabric at the plateau, and about 30 to 35 W at idle; no reference was found for what is normal.
  - Not visible from here: no voltage sensor exists (`k10temp` and `zenergy` expose none), so CPU core and SoC voltage
    and load-line settings need the BIOS or a tool the user runs. The user first said memory overclocking was enabled,
    then corrected it to disabled (about 20:58 EDT), so EXPO is off. AMD capped the SoC voltage at 1.3 V in an AGESA
    update after burnout reports tied to EXPO on some boards, and a Hardware Busters test showed idle power spikes
    of 109 to 130 W on early BIOS versions (Notebookcheck report); with EXPO off that route to extra IO die heat is
    closed. The BIOS version was not read (the user declined that command).
  - Nothing measured says the motherboard is defective: the heat load is normal for the chip, and the motherboard
    does not carry heat from the chip to the cooler. A board fault would show as odd voltages, instability or odd
    sensor readings; none was observed or reported.
  - The probe on top of the heatsink stayed within 0.3 degrees of its idle value while the CPU sat at 95.
    If the heatsink
    were passing the heat to the air well, its top should warm, so this fits a bottleneck between the chip and the
    heatsink; the probe's contact is weak (its tape cannot bend), so this is a hint and not a measurement.
  - Four new sample columns log the GPU temperature and power and the integrated graphics, so the next load shows
    whether the GPU heats the case.
- Memory pressure, measured at 19:30 EDT (the user asked why tasks stall with so much memory in total):
  - The host is not short of memory: `MemAvailable` 45 GiB of 62 GiB, `user-1000.slice` peaked at 46 GB against a
    `memory.high` of 59.8 GB that it never reached (0 high events).
  - The `claude-code-bash` cgroup has a 8,000,000,000 byte limit (`CLAUDE_CODE_TOOL_MEMORY_LIMIT` in
    `~/.claude/settings.json` line 9) shared by every session's tool commands. Its peak is 8,000,282,624 bytes
    and `memory.events` counts 1,055,909 `max` events since boot, so it hits its own limit constantly.
    Inside a cgroup at its limit, each allocation reclaims in that cgroup, and the stalls count as memory pressure
    for the whole host. At 19:16 the cgroup read `some avg10` 8.03, the host 8.25.
  - Other cgroups at their limit: a podman container with a 2 GB limit (540 `max` events).
  - Not proven: at the instant of the 19:16 alert the cgroup held 1.96 GB, below its limit, and `memory.peak` is a
    lifetime figure. The `max` event counter was not captured around the alert.
  - A link to the CPU cap is plausible but unproven: more parallel jobs under the same 8 GB limit means more reclaim.
  - The earlier framing in this handover and in the conversation, "RAM is full and the kernel reclaims", was wrong.
    Since-boot counters (direct reclaim 98 million pages scanned, 12.2 million swapped to zram) fit cgroup-local
    reclaim, which direct reclaim counters include.
- Reference points found online (different benches): 78 degrees at 142 W on a 240 mm AIO (forum report),
  86.1 degrees at about 204 W on a water loop, 87 degrees at 200 W for an NH-D15 G2 on a 9950X.
  Neither fixes an expected value for this setup.

## Hardware facts

- CPU cooler: a Noctua NH-D15S, the asymmetric single-fan version of the NH-D15 (user correction at about 20:30 EDT
  on 2026-10-13, after first saying G2 and then G1). Noctua's FAQ says the heatsinks are identical and the two-fan
  NH-D15 is 1 to 3 degrees better. The user later said two fans are fitted, push and pull, so the full two-fan setup
  is what Noctua compares.
- Thermal paste: Arctic MX-4, applied as a cross (user, about 20:35 EDT), not Noctua NT-H1 which the user could not
  find. The amount is not known.
- Fans: 7 Noctua and 1 Lian Li, some chained, on a Corsair Commander Core XT (5 of 6 ports used; fan3 is empty).
  CoolerControl labels the controller "(broken)" but applies profiles successfully.
- The Corsair probe sits on top of the air cooler's heatsink. Its tape cannot be bent, so the tip cannot touch the
  fins (user, 2026-10-13). It therefore reads the top surface and the air around it, which matches the flat
  25.5 to 26.2 degree reading; it is not a heatsink-fin reading. It cannot separate contact from airflow problems.
  The user's view: it still gives a room or case temperature, which is useful.
  Treat it as the ambient reference (about 25.8 degrees in the first window) when comparing Tctl against watts.
- The `fan1` reading in `sensors` belongs to the GPU, not a CPU fan. An early claim based on it was wrong.
- Motherboard: Gigabyte B650 GAMING X AX rev 1.3. The six `gigabyte_wmi` readings are unlabeled and no source maps them.
  Early data: only the third one tracked CPU load. BIOS updated in July 2026 (user, about 21:00 EDT on 2026-10-13;
  the exact version was not read), memory overclocking disabled.

## Rejected or withdrawn

- A VM or loop-device test of the dm-crypt flags: the user pointed out it would not be bare metal.
- A sustained-load stress test: declined by the user.
- Boost and PBO: documented behavior says they push the chip to 95 degrees sooner for more clock, so they add heat.
- Switching to ZFS or CachyOS: not recommended yet, since nothing shows the filesystem is the limit.
  Another handover, `doc/handover/cachyos-zfs-vm-validation.md`, exists; it was not read here.

## Per-session limits (researched, nothing implemented)

The user asked on 2026-10-13 whether every session can have its own limits.
They pointed at `tweakcc` hoping to learn something, and said not to patch the binary; it was not patched.
`tweakcc unpack` of version 2.1.296 yields only a bytecode stub (no `claude-code-bash` in it),
so the findings come from reading the readable source strings inside the binary with `strings`.

- Claude Code names the tool cgroup with a constant, `claude-code-bash`, and creates it as a sibling of the cgroup
  that Claude itself runs in: `<parent of Claude's own cgroup>/claude-code-bash`, with `memory.max` set from
  `CLAUDE_CODE_TOOL_MEMORY_LIMIT`. If Claude's own cgroup path already contains a `claude-code-bash` segment,
  it logs "nested: already capped" and reuses it. The tool cgroup is not named per session.
- Every session started from a terminal surface lives in `app.slice/app-ghostty-surface-transient-N.scope`,
  so its parent is `app.slice` and all of them share `app.slice/claude-code-bash`.
  That is the whole reason it is shared.
- There is no CPU setting in Claude Code. `CLAUDE_CODE_TOOL_MEMORY_CGROUP_EXCLUDE` only picks which helper classes
  (mcp, lsp, hooks, plugin, tmux, helper, agent) stay outside the cgroup.
- Verified by a real run: `systemd-run --user --scope --slice=claude-probe.slice -- claude --print ...` put the
  session's Bash tool in `.../user@1000.service/claude.slice/claude-probe.slice/claude-code-bash`, a separate cgroup.
  (A dash in a slice name nests it: `claude-probe.slice` sits inside `claude.slice`.)
  The probe slices were stopped afterwards. Interactive sessions were not tested.
- Verified on a throwaway slice (stopped afterwards): `systemctl --user set-property claude-limittest.slice
  MemoryMax=2G CPUQuota=300%` made the slice's `memory.max` read 2147483648 and `cpu.max` read `300000 100000`,
  and the `cpu memory pids` controllers were available under `claude.slice`.
  Depth is not limited here (`cgroup.max.depth` and `cgroup.max.descendants` read `max`).
- So per-session limits need no patch: start each session in its own slice, and put `MemoryMax=` and `CPUQuota=` on that
  slice, which covers both the session and its tool cgroup. A drop-in directory `claude-.slice.d` in
  `~/.config/systemd/user/` would apply to every `claude-*.slice`, and limits on `claude.slice` would be a shared total.
- Not decided: the per-session numbers, whether to keep a shared total, and how sessions get launched
  (a `claude` wrapper in `PATH`, a shell function, or `spawn-claude`). Running sessions keep the shared cgroup until
  restarted, because Claude reads its own cgroup path once at startup.
- The CPU watcher writes `cpu.max` only into `app.slice/claude-code-bash`. Per-session slices would carry their
  limits in systemd properties, and the watcher would need a decision about how to tone those down.

## Replacement board research (2026-10-13, about 21:20 EDT, nothing bought or decided)

The user's warranty on the B650 GAMING X AX has expired, they see no reason to suspect the power supply yet and do not
want to buy a spare one now, and are open to buying a new board "correctly" for the current setup plus an RTX 3080 10G
for VFIO passthrough.

- Machine facts read from sysfs and `lspci`: host discrete GPU is a Radeon RX 7600 class card (Navi 33, an x8 card,
  power cap 162 W); the 7950X has integrated graphics (Raphael 610M) that can drive the host display; one NVMe drive
  with a Realtek RTS5772DL DRAM-less controller; Wi-Fi is a Realtek RTL8852CE and the wired LAN is switched off in the
  BIOS; 34 IOMMU groups with no IOMMU kernel arguments set; 2 DIMMs; NH-D15S cooler (AM5, offset bars).
- This board looks exposed to a reported VFIO problem: an AMD IOMMU patch thread (LKML, 2026-03-29 and 30, patch
  author Christos Longros, reply by Vasant Hegde of AMD) says some boards, naming Gigabyte B650 Gaming X AX V2 and
  X870E, ship IVRS tables whose unity-map ranges apply to nearly every IOMMU group, so VFIO is refused with "Firmware
  has requested this device have a 1:1 IOMMU mapping". AMD asked for an ACPI dump and had not reproduced it. On this
  board 28 of 34 groups carry a 'direct' reserved region (two ranges: 0x0 to 0xfff and 0x8af74000 to 0x8af75fff),
  the condition the thread describes. Whether passthrough actually fails here was not tested.
- Candidate boards, with what was verified: ASUS ProArt X870E-CREATOR WIFI (ASUS page): two PCIe 5.0 x16 SafeSlots
  (x16 single or x8/x8 dual), one PCIe 4.0 x16 slot (max x4), four M.2, dual USB4, 10 Gb and 2.5 Gb LAN, Wi-Fi 7,
  HDMI 2.1 and 16+2+2 power stages; a SHI listing shows MSRP 549.99 US dollars and 618 dollars at the time.
  ASRock X870E Taichi (manual snippet): two PCIe 5.0 x16 slots, x16 or x8/x8, with lane modes depending on the CPU;
  no price or VFIO report found. ASUS ProArt X670E-Creator: one PCPartPicker build reported easy IOMMU and VFIO setup;
  no price found. MSI MEG X870E ACE MAX: x16, x8 and x4 slots (not x8/x8). Gigabyte X870E Aorus Master: second slot
  shares lanes with M.2 (forum table, unverified).
- The IVRS risk is not Gigabyte-only. The same "1:1 IOMMU mapping" refusal appears in an LKML reply for a Gigabyte
  B850 AORUS ELITE WIFI7 (BIOS F11 and F7, RTX 5080) and in an Unraid forum report for an MSI X870E board. A v2 patch
  series (Christos Longros) applies cleanly on kernel 7.2.8 and fixed the refusal for the B850 reporter; whether it is
  merged was not checked. No report was found for ASUS or ASRock boards, but that search was one query only, so it
  proves nothing.
- Slot layout choice, separate from the board choice. Two boards below give two CPU x16 slots as x8/x8 (the RX 7600
  and the 3080 each at x8). Cheaper boards give the CPU x16 to one slot and hang a second x16 slot off the chipset at
  x4: the 3080 would take the CPU slot, and the RX 7600 would sit at Gen4 x4 on the chipset, or be dropped for the
  7950X integrated graphics. MSI MAG B850 TOMAHAWK MAX WIFI (spec text): PCI_E1 PCIe 5.0 x16 from the CPU, PCI_E3
  PCIe 4.0 x16 from the chipset at up to x4 (x2 if M.2_3 is used), PCI_E2 PCIe 3.0 x1; Wi-Fi 7 module pre-installed
  in an M.2 Key E slot; 5 Gb LAN; EZ Debug LED (four LEDs); reviews quote 209.99 to 239 US dollars.
- Diagnostics per board: ASUS ProArt X870E and B850 Creator list Q-LED (four LEDs for CPU, DRAM, VGA, boot device, no
  two-digit code; a text search of the saved X870E page found no Q-Code). ASRock X870E
  Taichi listings show a Dr. Debug LED (a two-digit display) and power and reset buttons (retail listing text, not the
  ASRock page). MSI Tomahawk: EZ Debug LED, four LEDs. Supermicro H13SAE-MF has IPMI but no Wi-Fi and a two
  double-width-card limit, so it is out. ASUS ProArt B650-Creator has x8/x8 but no onboard Wi-Fi (empty M.2 Key E).
- ASUS ProArt B850-CREATOR WIFI NEO (ASUS page): two PCIe 5.0 x16 slots as x16 or x8/x8 (Ryzen 7000/9000), a PCIe 4.0
  x16 from the chipset at x4 that shares lanes with M.2_3, 14+2+1 power stages, three M.2 slots, Realtek RTL8922AE
  Wi-Fi 7, dual 5 Gb Ethernet, Q-LED; price not found.
- ASUS ProArt X870E-CREATOR WIFI (ASUS page): x16, x8/x8 or x8/x4/x4 on the two CPU slots, one chipset x4 slot, Wi-Fi 7
  module that is either Qualcomm QCNCM865 or MediaTek MT7927 depending on the unit (Linux driver support for
  either was not checked); prices 471.90 EUR to 586 EUR (ASUS shop) and 379.90 CHF. A ROG forum thread (2026-08-23)
  reports eight 0x9F bugchecks in 39 days on this board with Windows 11 tied to the ASM4242 USB4 controller;
  Windows only, cause unconfirmed.
- Linux Wi-Fi: the RTL8922AE is in the kernel as rtw89_8922ae from 6.10 on (kernel database lists 6.10 to 7.2). The
  Fedora 44 kernel 7.2.5 works with it per a Fedora forum post. A Bazzite report says the OGC kernel 7.0 and newer
  lost it through mac80211 API changes; this host runs `7.2.9-ogc3.1.fc44`, an OGC kernel, so it must be tested
  there before trusting a Realtek Wi-Fi 7 board. Not verified for MT7927 and QCNCM865.
- User answers (about 21:50 EDT): four coarse status LEDs do not count as diagnostics ("just like my current board"),
  so a two-digit POST code display (or a BMC) is required. Either slot layout is acceptable (x8/x8, or the 3080 on
  the CPU slot with the RX 7600 on the chipset x4 slot). The RX 7600 drives four 4K 60 Hz displays. The 3080 will sit
  on a riser. The case is a Corsair 4000D Airflow (slot count and cooler height limit not verified here).
- With the two-digit display required, ProArt B850 Neo, ProArt X870E and MSI Tomahawk are out. The ASRock Taichi
  family remains: OCinside reviews of the B650E Taichi, B650E Taichi Lite and X670E Taichi Carrara name a 7-segment
  Dr. Debug display; the X870E Taichi shows it in retail listing text; for the X870E Taichi Lite a spec aggregator
  (MaxMyBuild) lists a POST code display (not ASRock's own page). Lite versus non-Lite (TheFPSReview, PCMag): same
  PCIe, M.2 and USB configuration and the same 24+2+1 power design; the Lite has a plainer look, no onboard RGB
  and no EZ Release design (graphics card release latch); PCMag priced the Lite at 399.99 US dollars.
  ASRock pages list the X870E Taichi and Taichi Lite with two CPU PCIe 5.0 x16 slots as x16 or x8/x8, a
  Key E socket for the Wi-Fi 7 module (chip not found), and 5 Gb LAN (Taichi: also 2.5 Gb). A Level1Techs post shows
  X870E Taichi Lite IOMMU groups on BIOS 3.20 (called fine, not independently read), and a GitHub guide passes a
  Radeon RX 7900 XTX through on a Taichi Lite with a 7950X under Fedora 41. Prices conflict: 479.99 US dollars
  (CGMagazine review) against about 329 (a Funky Kit article for the 10th Anniversary edition). Possible other leads
  with a code display, not researched: ASUS ROG Crosshair X670E Hero (Q-Code, from memory).
- The user confirmed (about 22:10 EDT) that the X870E Taichi Lite page shows a code display, and asked for boards
  outside the Taichi line. Findings:
  ASUS ROG Crosshair X870E Hero (ASUS pages): spec lists Q-Code and Q-LED, two CPU PCIe 5.0 x16 slots as x16 or
  x8/x8 (x8/x4/x4 with M.2_3), Wi-Fi 7 that is Qualcomm QCNCM865 or MediaTek MT7927 by unit, Intel 2.5 Gb plus
  Realtek 5 Gb LAN; 456.90 CHF, 568.90 EUR (list 762 EUR) in the ASUS shop. The Dark Hero also lists Q-Code and x8/x8
  but ships MediaTek MT7927 and 10 Gb LAN. ROG Crosshair X670E Hero: x16 or x8/x8, Wi-Fi 6E, Intel 2.5 Gb; Q-Code
  not seen in the fetched text; price not found.
  CORRECTION (about 22:40 EDT): an ASRock FAQ table (support FAQ 544) was first read as naming a Dr. Debug display
  for the X870E Nova WiFi, X870 Pro RS and X870 Steel Legend WiFi. Read again, its first row gives a Dr. Debug code
  only under the Nova; the Pro RS and Steel Legend columns describe LED behavior (CPU LED solid, DRAM LED blinking),
  so the Steel Legend most likely has only status LEDs and fails the code display requirement (confirm on the
  spec page). X870
  Steel Legend WiFi (ASRock page): PCIE1 PCIe 5.0 x16 from the CPU, PCIE2 PCIe 4.0 x16 at x4 from the chipset, Key E
  Wi-Fi socket; no x8/x8. X870E Nova WiFi: one PCIe 5.0 x16, one PCIe 3.0 x16, one x1; no x8/x8. A Reddit VFIO post
  reports passthrough success with good IOMMU groups on a Steel Legend. ASRock forum thread titles (not read) report
  POST freezes (A6, 0xd6, error 21) on Nova WiFi and idle or sleep freezing and POST trouble on Steel Legend.
  MSI MPG X870E Carbon WiFi: an Unraid thread (January 2026) reports the IVRS "1:1 IOMMU mapping" refusal on this
  exact board; its slot layout was not confirmed. Gigabyte X870E Aorus Master X3D: a CPU x16 plus a CPU x8 slot
  (x8/x8 once the second is used) and a chipset x4 slot, but Gigabyte boards have IVRS reports and its display was
  not confirmed. ASUS X870E reports found: a ProArt Reddit post says CPU slots sit in separate IOMMU groups, and a
  Proxmox post says an RTX 5090 link dropped to x8 after a function-level reset until cold boot (5090 only).
- Stability reports searched at the user's request (about 22:40 EDT); forum titles and snippets only, with no
  failure rates, so they cannot rank boards. ASRock Taichi Lite and Taichi: a Reddit title says BIOS 4.43 breaks
  XMP and EXPO profiles and lengthens boots; an Overclock.net user says changing a fan setting triggers a full memory
  retrain (code 15); an ASRock forum post reports a 128 GB two-DIMM kit not POSTing in A2 and B2; a Tom's Hardware
  post reports a Crucial T705 drive vanishing in the BIOS; several Reddit titles report dead 9800X3D, 9950X3D and
  7800X3D CPUs on ASRock AM5 boards. ASRock shipped AGESA 1.3.0.0a in BIOS 4.07 beta and 4.10 for no-boot behavior;
  Level1Techs reported that fixed instability on a 9600X (abit.ee summary); a Steel Legend 9950X death was reported
  after that BIOS (VideoCardz via PauseHardware), which Hardware Unboxed did not reproduce. ASUS Hero: freezes fixed by
  BIOS 2306 (ROG forum), a once-a-month hang on the DRAM LEDs (Overclock.net), EXPO profiles failing to boot, a code
  1A after BIOS 2102, code 62 after BIOS 1715, the MediaTek Wi-Fi adapter disappearing after sleep (July 2026), and a
  Reddit comment that ASUS acknowledged a CPU and board burn-up problem in January 2026 (not verified). Both vendors
  show memory training trouble, which matches the user's own EXPO errors.
- The user confirmed (about 22:50 EDT) that the ROG Crosshair X870E Hero shows a code display.
- Wi-Fi chips and Linux (searched about 22:55 EDT). X870E Taichi: MediaTek MT7925 (TechPowerUp; a Spanish review calls
  it RZ717, the module name); the Taichi Lite chip was not stated by any source found, probably the same module but
  unverified; the Lite wired LAN is a Realtek RTL8126, and a Reddit title reports a Lite user without working wired
  networking on Pop!_OS (cause not read). Hero: Qualcomm QCNCM865 or MediaTek MT7927 by unit (ASUS). Kernel
  database (cateee.net lkddb, CONFIG_MT7925E): `mt7925e` lists MT7925 (14c3:7925, RZ717 14c3:0717) from 6.7 and
  adds MT7927 (14c3:7927) and 14c3:6639 in 7.2; before that the MT7927 needed an out-of-tree DKMS package (an
  Ubuntu gist on a Hero), and the mainline patch series was posted in March 2026. Presence of an ID is not proof it
  works on `7.2.9-ogc3.1`. Qualcomm QCNCM865 (WCN7850) uses `ath12k`, in the kernel since 6.3, with Framework and
  Ubuntu reports of it working and one Ubuntu 24.04 report of it failing for lack of current firmware. A Linux review
  of a Gigabyte board with an MT7925 module found its Bluetooth not detected. Realtek RTL8922AE boards carry the OGC
  kernel regression report from the earlier note. Result: Wi-Fi does not separate the Taichi Lite from the Hero on
  paper; both need a test on the real kernel.
- Not verified: IOMMU group layouts of any candidate on the user's CPU and kernel, BIOS stability reports for 2026
  firmware, the 4000D's expansion slot count and cooler height limit against the NH-D15S, and the power supply.
- A new board does not prove the memory errors at EXPO 6000 are the board's: the CPU memory controller and the DIMMs
  are shared with the old setup. Validate EXPO with memtest86+ on the new board before concluding. The thermal gap is
  a separate problem; a new board does not change heatsink contact.

## Board recommendation (about 23:10 EDT, nothing bought)

The user asked for the final recommendation after the board research above. It rests on forum and retailer text,
not on a tested board; the checks in the last subsection come before any purchase.

### Requirements it answers

AM5 for the 7950X; the RX 7600 driving four 4K 60 Hz displays plus an RTX 3080 10G on a riser for VFIO; a two-digit
code display (four status LEDs were rejected); onboard Wi-Fi 5 or newer; 24/7 operation; a 850 W or larger power
supply; a Corsair 4000D Airflow case; either x8/x8 or the 3080 on the CPU slot with the RX 7600 on a chipset slot.
The user confirmed seeing a code display on the X870E Taichi Lite, the ROG Crosshair X870E Hero and the X870E Nova
WiFi.

### Ranking and reasons

1. ASRock X870E Taichi Lite. It is E-ATX (12.0 by 10.5 inches, TechPowerUp and Vortez), x8/x8, code display,
   Wi-Fi 7, 5 Gb LAN, about 399.99 US dollars at launch (PCMag). Corsair's own 4000D Airflow page (CC-9011200-WW,
   Tech Specs) lists "E-ATX (305mm x 277mm)", and 10.5 inches is 267 mm, so the board fits the stated size; KitGuru,
   one UK reseller and two spec aggregators list only ATX, so Corsair's page and the listings that copy it are the
   source used. Fit with the NH-D15S (160 mm tall, case limit 170 mm) and the board's mounting holes is unchecked.
2. ASUS ROG Crosshair X870E Hero. It is ATX (ASUS), x8/x8, Q-Code, Wi-Fi 7 (Qualcomm or MediaTek by unit), 456.90 CHF
   to 568.90 EUR in the ASUS shop. It is the only candidate with every requirement checked, including the fit.
3. ASRock X870E Nova WiFi. It is ATX (12.0 by 9.6 inches), with a Dr. Debug display (retailer spec) and Wi-Fi 7, 5 Gb
   LAN, MSRP 349 US dollars (Tech4Gamers, Tom's Hardware 349.99). Its only chipset slot of full length, PCIE3, runs
   at PCIe 3.0 x2 (ASRock page) and is disabled when M2_5 is used, so the RX 7600 would get a Gen3 x2 link.
4. ROG Crosshair X670E Hero: x8/x8 and Wi-Fi 6E, display not confirmed in text read, price not found, older platform.

Pair reasons: the Lite beats the Hero on price and VFIO evidence now that the case lists E-ATX; if its fit fails in
practice, the Hero is first. The Hero beats the Nova because the Nova gives the RX 7600 only a Gen3 x2 slot. The
Nova beats the X670E Hero because its display is confirmed and it is current. Out: ProArt B850 Neo, ProArt X870E and
MSI Tomahawk (status LEDs only), ASRock Steel Legend and Pro RS (status LEDs only, per the FAQ table), MSI Carbon (IVRS
report on that board), Gigabyte boards (IVRS reports).

### VFIO and 24/7 evidence (searched about 23:30 EDT, forum and guide text only)

Passthrough layout that matters: only the RTX 3080 (CPU slot, with its audio function) must sit in a clean IOMMU group;
the RX 7600 stays on the host, so its group does not matter. The AMD reset bug (a card failing to reset after a VM
stops) is described for the AMD card being passed through, which is not this layout.

- Taichi Lite: a Level1Techs member posted its IOMMU groups on BIOS 3.20 (list not read in full); a GitHub guide runs a
  Fedora 41 host with a 7950X on a Taichi Lite and passes an RX 7900 XTX through; an r/VFIO thread names the Taichi
  and Taichi Lite as the E-ATX x8/x8 choices. No IVRS report for ASRock found (one query only).
- Hero: no VFIO report for this board was found. ASUS X870E evidence comes from the ProArt: CPU slots in separate
  groups, but both Ethernet controllers, a USB controller and the SATA controller share one chipset group (ACS patch
  needed to split SATA); one ProArt owner runs an RTX 4090 and 5090 at x8/x8, another in the same thread says to avoid
  the board (reasons not read). A Proxmox post on an ASUS X870E says an RTX 5090 link fell to x8 after a function-level
  reset until a cold boot (5090 only; forcing Gen3 did not change it).
- Nova: no VFIO report found; its RX 7600 slot hangs off the chipset in either case.
- RTX 3080: Ampere cards appear in several working passthrough guides; nothing on a 3080 on these boards was found.
- Riser: no source read; a riser rated for PCIe 4.0 matters because the slot can run Gen5 and x8 lanes.
- ECC: ASRock lists DDR5 ECC and non-ECC unbuffered support; a Level1Techs thread summarizing a Hardware Unboxed test
  says ECC worked on Gigabyte and ASRock boards, an ASUS ROG board posted without working ECC, and an MSI board did
  not post. ECC DIMMs cost more and the user has memory errors at EXPO, so this is an option, not a requirement.
- 24/7: no uptime or failure-rate data was found for any candidate. ASUS ProArt pages carry a "24/7 Reliability"
  marketing line; the Hero and Taichi are consumer boards. A Level1Techs post planning a 24/7 server on a Taichi asks
  about 3 to 5 minute AM5 boots from memory training. USB4 controllers (ASM4242) appear in freeze and 0x9F reports on
  ProArt, TUF and MSI X870 boards, with one report of disabling USB4 in the BIOS curing freezes; which USB4 controller
  the Lite or Hero uses was not checked. Server-class boards were ruled out earlier by slots and Wi-Fi; the section
  "Wi-Fi card allowed" below revisits them.

### Checks before buying

- Done: Corsair's 4000D Airflow page lists E-ATX (305mm x 277mm). Still open: the Taichi Lite's mounting holes against
  the case standoffs, and cable reach for the 24+2 pin and CPU power on a 267 mm tall board.
- Read the chosen board's own specification page for the slot table and the code display; the retailer text used
  above can be wrong.
- Choose the BIOS from the release notes: AGESA 1.3.0.0a or later; ASRock 4.43 has a Reddit title about broken EXPO
  profiles and long boots.
- On the new board, boot a live USB of the OGC kernel and confirm Wi-Fi and Bluetooth appear before committing.
- Run memtest86+ at EXPO 6000 on the new board; it is the only way to tell a board fault from a CPU or DIMM fault.
- Check the IOMMU groups with the 3080 installed, and keep in mind the thermal gap is a separate problem.

### Risks left open

Dead-CPU reports exist for ASRock AM5 boards (mostly X3D, a few standard Ryzen), and an ASUS burn-up acknowledgment
is mentioned in one Reddit comment; neither vendor is clean. Memory training trouble appears on both vendors. No
IOMMU group list for the user's CPU and kernel was seen. The Taichi Lite's Wi-Fi chip is unconfirmed, which no longer
matters if the user buys a card (next section).

### Wi-Fi card allowed (user decision, about 23:45 EDT)

The user said buying a Wi-Fi card is acceptable, so onboard Wi-Fi is no longer a requirement.

- The ranking does not change. Onboard Wi-Fi was a tie-break and a risk (the Lite's chip unknown, the Hero's chip
  varying by unit), not what put the Lite, Hero and Nova ahead: the two-digit code display and the slot layout did.
- The risk shrinks to choosing a card with a mainline Linux driver and testing it on the OGC kernel (7.2.9) before
  relying on it. The onboard module stays on the board unused. No card model was researched this period; Intel
  iwlwifi cards are the usual Linux choice, which is unverified here.
- A card in the board's M.2 Key E 2230 slot keeps every PCIe slot free; a PCIe card would take a slot.
- Boards that return only because they lack Wi-Fi:
  - ASUS ProArt B650-Creator: x8/x8, empty Key E slot, 314 to 334 US dollars, code display unconfirmed, and one
    r/VFIO user warned against the ProArt line.
  - Supermicro H13SAE-MF (Supermicro and ServeTheHome pages): micro-ATX, B650, two PCIe 5.0 x16 slots (x16/x0 or
    x8/x8) plus a Gen4 x4 chipset slot, ASPEED BMC with IPMI, ECC UDIMM at 5200 officially, no EXPO, about 502 US
    dollars at one retailer. A code display is not in its specification. A Reddit thread reports Ryzen 9000 with two
    48 GB ECC DIMMs failing at 5600 on early BIOS. The CPU socket is rotated 90 degrees for rack airflow, which
    changes NH-D15S orientation in a tower case (unchecked). No VFIO report found (a Level1Techs thread asks for one).
  - ASRock Rack B650D4U: one PCIe 5.0 x16 and one x4 slot per retailer text, so no x8/x8 and an unverified second
    slot; display not checked.
- Both server boards fail the confirmed code display requirement as far as the pages read show, so none enters the
  ranking.

### Cheaper boards with a code display (searched about 00:10 EDT, user asked for lower budgets)

All three below pair a CPU x16 slot with a chipset or CPU x4 slot, which the user accepted for the 3080 and the
RX 7600. Prices are launch or historical figures; current stock and street price were not checked.

- ASUS ROG Strix X670E-E Gaming WiFi (ASUS spec page): ATX, "Q-Code" listed, two PCIe 5.0 x16 slots (x16 or x8/x4)
  plus a chipset PCIe 4.0 slot at x4, Wi-Fi 6E, one 2.5 Gb LAN, ECC UDIMM listed. MSRP 499.99 US dollars; one price
  tracker shows a low of 349.99 in February 2025; a used unit was listed at 250 euro on 2026-10-05; a refurbished unit
  at 726.84 US dollars on Newegg. The only cheaper board here whose own spec page names the code display.
- ASUS ROG Strix B650E-E Gaming WiFi (ASUS spec page): same slot layout and ATX size, Wi-Fi 6E, launch price about
  349.99 US dollars (Tom's Hardware). The spec page lists Q-LED but not Q-Code, while owner threads on Reddit and the
  ROG forum report Q-Code values (15, b8, C5), so the display is probably present and unconfirmed on the page. Those
  threads include a board and Ryzen 7900 that both failed, a single anecdote.
- ASRock B650E Taichi Lite and B650E Taichi: Dr. Debug 2-digit display (OCinside, a spec table), E-ATX 30.5 by 26.7 cm
  (Guru3D, fits the 4000D's listed 305 by 277 mm), PCIe 5.0 x16 plus a chipset PCIe 4.0 x16 at x4, Wi-Fi 6E, price
  279.99 to 299.99 US dollars at launch. ASRock's own page says both reached end of life and were discontinued in
  October 2024, so only used or leftover stock exists; one Reddit owner of an open-box Lite reported SATA and NIC
  faults.
- Not usable: MSI MPG X670E Carbon WiFi has a 2-digit debug LED but the earlier IVRS report on that board excludes it;
  MSI MEG X670E ACE has a post code display but is a flagship price; budget ASRock boards (Steel Legend, Pro RS) have
  status LEDs only.
- No IVRS or VFIO report for the two ASUS Strix boards was searched; the ASUS evidence so far is the ProArt line only.
- Ranking inside this tier: X670E-E, then B650E-E, then the B650E Taichi Lite. The X670E-E beats the B650E-E because
  its own spec page names the display. The B650E-E beats the Taichi Lite because the Lite is discontinued, so only
  used stock exists. Against the earlier list, both Strix boards give the RX 7600 a better slot than the Nova (CPU or
  chipset Gen4 x4 instead of Gen3 x2) at a similar launch price, but have less VFIO evidence.

#### VFIO and stability for the Strix boards (searched about 00:40 EDT, snippets only, no thread read in full)

- X670E-E, VFIO: an r/VFIO post with a 7950X on this board says the IOMMU groups "generally look pretty good", with
  the network, audio and USB controllers sharing one group, and that the poster returned the gear because DDR5 caused
  problems. The iommu.info database lists 47 groups with each USB controller separate; a Level1Techs member doubted it
  and no owner confirmed it. A commenter says ASUS ROG boards usually expose a PCIe ACS setting, other ASUS lines do
  not. No report of the 1:1 IOMMU mapping refusal was found for any ASUS Strix board.
- B650E-E, VFIO: one r/VFIO thread from January 2023 ("Considering returning my Rog Strix B650E-E") says the IOMMU
  groups looked good but lists several other issues (text not read); no group list was found.
- X670E-E, BIOS stability in 2026: an r/ASUSROG post is titled "Updating to BIOS 3804 on ASUS ROG STRIX X670E-E bricked
  my system with my 7950X (Code 00)", and a commenter's 7950X3D worked, so it may be specific to the plain 7950X (the
  user's CPU); an ROG forum post says EXPO broke after BIOS 3702 with BSODs and restarts; other ROG forum posts report
  hard freezes or Code 00 on Crosshair X670E boards with BIOS 3702 (AGESA 1.3.0.1). Older reports (2023) include random
  shutdowns on this board. ASUS X670E-F with BIOS 4003 freezes in Windows with fTPM on, Linux boots (a different board).
- These are anecdotes, not rates. They argue for choosing a BIOS from release notes and keeping the board's previous
  BIOS file for rollback, and for not updating to 3804 on a 7950X until a thread confirms it is safe.
- Neither board would fix a memory fault that comes from the CPU's memory controller or the DIMMs; the memtest86+ run
  at EXPO 6000 on the new board is still the discriminating test.

### Prices, stock and BIOS (searched about 01:10 EDT, aggregator and retailer snippets, region unknown)

The user's country is not recorded, so prices are listed by source and currency; none was confirmed at checkout.

- X870E Taichi Lite: Tech-America lists 356 US dollars; maxmybuild shows from 399.99 at one retailer; Pangoly best
  price 383.99; Box.co.uk lists 354.99 pounds (updated 2026-09-02, all-time low 309.99 pounds); two UK shops show it in
  stock; a Beach Audio US listing is out of stock; one Canadian private seller asks 480 Canadian dollars new.
- Crosshair X870E Hero: Best Buy 558 US dollars; Pangoly current 549 with a low of 512.99 on 2026-06-15; UK shops list
  479.99 to 546.99 pounds in stock; one US reseller lists 666.49.
- X870E Nova WiFi: UK shops 291.99 to 308.81 pounds in stock; Tech-America 282 US dollars; AVADirect 396.15 (ships in
  1 to 3 weeks); Pangoly low 259.99 in November 2025.
- ROG Strix X670E-E: no current new price found; a used unit was listed at 250 euro on 2026-10-05.
- Taichi Lite BIOS: the 3.x series ran from AGESA 1.2.0.0a to 1.2.0.3g (3.50); the 4.x series carries AGESA 1.3.0.0a
  (4.07 beta, then 4.10 stable on 2026-02-25), and an ASRock forum user mentions 4.41 and 4.43 as current. A Level1Techs
  test (via a news summary) says 1.3.0.0a fixed Ryzen instability on an X870E Taichi. A Reddit owner runs 3.25 with a
  9800X3D without problems; another post reports a new Taichi Lite stuck at code 00 after BIOS Flashback.
- Canada (the user said they are in Canada at about 01:30 EDT; Canadian dollars; Newegg.ca and Canada Computers
  snippets without dates, so each needs a live check):
  - Hero: Canada Computers page read live, 839.99, free shipping, available online (2) and in a few stores.
  - Taichi Lite: Newegg.ca listings at 429.99 (marked down from 579.99), 449.99, 459.99 and 569.99 to 579.99; a Canada
    Computers clearance listing at 349.88 whose product page now says "no longer available".
  - Nova: Newegg.ca listings at 339 and 369 (list price 499); a Canada Computers open-box listing at 284.89 (new
    299.88) whose page now says "no longer available".
  - Taichi Lite and Nova stock at Canada Computers looks gone; Newegg.ca marketplace sellers may be third parties.
- Decision: the Taichi Lite and the Hero are both in stock somewhere, so the cheaper Strix path saves little against the
  Lite in the UK (354.99 pounds for the Lite against 291.99 for the Nova) and has a worse BIOS record for the 7950X.

### Is there a de-facto VFIO board (asked about 02:00 EDT)

No board is the de-facto choice.
The nearest thing to a consensus is the Level1Techs thread "AM5 IOMMU VFIO - Best Motherboard" (started 2022-10-07,
posts mostly from 2023, so BIOS-dated), which read as follows in the first pages fetched and in search snippets:

- Its most-cited recommendation is the ASRock X670E Taichi (one poster, "IMHO", after comparing the group dumps seen).
  A later poster summed the thread up as the ASUS ProArt B650-Creator and the ASRock B650E Taichi (non-lite) having
  the best groups.
- ASUS ProArt X670E-Creator and ROG Crosshair X670E Extreme: networking, Thunderbolt and some SATA/USB controllers
  sit in one group; a Proxmox poster found dual-GPU passthrough impossible on the ProArt X670E and then bought the
  ProArt X870E.
- ASRock B650E PG Riptide: the GPU and the chipset devices sit in separate groups in the posted dump.
- Gigabyte: the Aorus Master appears in the group comparison, but the X870E and B650 Gaming X AX V2 reports are the
  "1:1 IOMMU mapping" refusals recorded in "VFIO and 24/7 evidence".
- X870E: a Reddit poster says a Level1Techs review found ASRock groups fine; the same Reddit poster would pick ASRock
  next time. No thread read here covers the X870E Taichi Lite or the Nova in detail.
- A "best motherboards for GPU passthrough" web page ranks the ROG Strix X870E-E first; it cites no group dumps
  and is not treated as evidence.

What this means for the plan: the 3080 sits in a CPU slot, whose root port is its own group on the posted dumps, so
the ranking does not hinge on a "best groups" board; the groups matter for chipset devices (USB, NIC) the user has not
said they need to pass through. This last point is inference from the dumps, not a measurement on the user's CPU.
The day-one checks under "Checks before buying" still decide it.

## Open questions

- The user suspects the motherboard (about 21:05 EDT): the Gigabyte B650 GAMING X AX rev 1.3 freezes in the BIOS
  settings screen from time to time, and its BIOS was updated in July. No search result described this exact symptom
  on this board (the closest were POST hangs and no-boot reports on other Gigabyte B650 boards). The kernel log of this
  boot shows no machine check, hardware error or PCIe error line; it has one `clocksource: Watchdog remote CPU 12 read
  timed out` at 17:01:32, which is not a board fault by itself. A firmware bug, the USB keyboard or mouse, or a hardware
  fault all fit a freezing setup screen, and none is tied to the heat gap yet. Proposed order: reseat with NT-H2 and
  compare first; if the gap stays, reset the BIOS to defaults (this also clears voltage and load-line settings and the
  user's boost and PBO choices), retest, and treat a board fault as the next suspect, with the freezes as supporting
  evidence. Nothing is decided.
  - More from the user (about 21:10 EDT): with the previous CPU too they had to disable many board settings for
    stability, and memory at EXPO 1 produces errors (EXPO is therefore off). The parts common to both CPUs are the
    board, the memory and the power supply, so instability across two CPUs points away from either CPU; this is a
    pattern, not proof. AMD's 7950X page rates memory at DDR5-5200 with two DIMMs (one or two ranks) and DDR5-3600
    with four DIMMs, so EXPO errors can be expected if the kit is above those speeds with that many DIMMs; the DIMM
    count and the EXPO speed are not known yet. A memory test at default settings (no EXPO) would separate faulty
    memory, board or power supply from an over-rated kit.
    Answers (about 21:12 EDT): the user ran memtest86+ at JEDEC defaults and the memory passed, and two DIMMs are
    fitted. So the memory is sound at defaults, and two DIMMs are within AMD's DDR5-5200 rating, so EXPO errors at
    a higher EXPO speed mean the kit is above the processor's rated speed on this board, not that the memory is bad;
    the EXPO speed was not asked. This does not yet separate a board fault from a kit or memory controller limit.
  - Further answers (about 21:20 EDT): the EXPO speed is 6000, above AMD's DDR5-5200 rating for two DIMMs. BIOS
    settings the user disabled for stability: spread spectrum control, global C-state control, ASPM, PCIe Gen4,
    PCIe Gen5,
    LAN, wake, and probably more. An SSD that works in the user's ThinkPad X13 Gen 4 does not work in this board.
    Checked in sysfs: the NVMe link is 8 GT/s x4 (Gen3) with a 16 GT/s maximum, so that slot's link is held at Gen3; the
    discrete GPU link is 16 GT/s x8 (its maximum); no PCIe error counters are set on any of them; the cpuidle driver is
    `none`, so deep CPU idle states are not in use, which fits the high idle power seen earlier (about 60 W and
    54 degrees
    at idle) but does not explain the load temperature gap. The pattern across two CPUs (BIOS freezes, memory errors
    above the rated speed, PCIe and idle-state workarounds, an SSD that is not accepted) points at the board or the
    power
    supply; the thermal gap stays a separate problem until a link to it is shown.
- The cooler is the NH-D15S (answered). Is it mounted correctly? The numbers in the evidence section say the heat
  path is far weaker than references; the model difference (1 to 3 degrees for one fan instead of two) does not
  explain it, and the paste is not the Noctua NT-H1 the checklist assumes. The user tightened the screws until they
  could not turn them with a normal screwdriver and little force, which matches Noctua's instruction (below).
  Noctua's NH-D15 manual and torque FAQ (fetched 2026-10-13) give this checklist, with the user's answers pending:
  - Tighten the heatsink screws 2 to 3 turns each in turn, then repeat, "gently until they stop"; maximum torque
    0.6 Nm for the spring-loaded screws and for the NM-ALS1 screws on the AMD mounting bars. The manual gives no force
    in newtons.
  - Remove the plastic protection cover from the heatsink base before mounting (the manual lists it before the
    heatsink is placed on the CPU).
  - On AM5 use the -7 mm offset hole position of the NM-AMB12 mounting bars "to ensure optimal contact quality";
    the standard position (0) is for compatibility problems.
  - NM-AMB12a ("NORTH") on the top side of the socket, NM-AMB12b ("SOUTH") on the bottom side, "CPU" arrows toward the
    CPU, plastic spacers on the stock AMD backplate, which stays in place.
  - Paste: 5 dots of NT-H1 (four of about 2 mm near the corners, one of 3 to 4 mm in the centre); too much lowers
    conductivity.
- How hard is the cooler-to-die path really? The 20:11 plateau gives 88.8 degrees at 130 W with 26 degree ambient,
  about 0.48 degrees per watt over ambient. Compare it with the 0.32 degrees per watt implied by the one online
  NH-D15 G2 report (87 degrees at 200 W, assuming a 23 degree ambient that the report does not state, so it is
  loose). A reseat or mounting check is the
  next physical step, and it is the user's call.
- What do the six motherboard readings measure? (The ambient reference is the Corsair probe; see the hardware facts.)
- The probe cannot touch the fins (answered above), so fin or exhaust temperature stays unmeasured.
  Contact versus airflow now has to be judged from Tctl against watts and fan speed.
- A higher fan floor was offered and not taken: the user is balancing noise themselves and edits the curve.
- Is 40 the right pressure limit? Judge from settled data.
- `ide7`, `ide8`, and `tr-gate` units run outside the cap (their own cgroups).
- The hold paragraph in `doc/handover/cli-git-rust-implementation.md` should point to this work.
  The session `monochromatic-18` was asked to update it.
- A reboot has not been tested against the persistent setup.
- The token is not being rotated (user, 2026-10-18) and still works for ad hoc CoolerControl API reads.
  The watcher does not read CoolerControl since 2026-10-18, so `cc-token` is only for manual use.

## Next actions

1. No monitor and no 4-hour check (ended by the user). Read alerts from the journal only when the user asks.
2. When a session has run sustained load, run `node package/dev-script/claude-cpu-cap/src/summarize-samples.ts 1`.
   It still needs a chiplet comparison and a (Tctl minus probe) per watt figure.
3. The service stays running; stop it only if the user asks.
4. Record every user decision and measurement in this file as the session continues.
5. After the user reapplies paste (decided about 20:45 EDT on 2026-10-13: Noctua NT-H2 3.5g AM5 Edition, ordered, not
   yet arrived): compare against the baseline below with the same kind of load of 100 W or more for at least 3 minutes.
   Baseline before the reapply (MX-4 cross, 2 fans, cap 28, 20:16 to 20:25 EDT): Tctl 95.4, average of the two CCD
   temperatures about 91.7, package power 133 W, clock 4407 MHz median, probe 26 degrees, so about 0.49 degrees per
   watt over ambient; Noctua's lab figure for the same heatsink and chip is about 0.26. Compare degrees per watt, not
   raw temperature, because the load will differ.
   `node package/dev-script/claude-cpu-cap/src/analyze-episode.ts <from> <to>` prints the medians.
   The NT-H2 AM5 Edition includes the NA-TPG1 paste guard and 3 cleaning wipes; Noctua's AM5 pattern is five small
   dots (four of about 2 mm near the corners, one of 3 to 4 mm in the centre) with no manual spreading, and the paste
   needs no cure time (NT-H2 AM5 Edition manual and product page, fetched 2026-10-13). The reseat also resets the
   other checks: base film removed, -7 mm offset, bars NORTH and SOUTH, screws alternately to the stop.
   Done on 2026-10-18 together with a board swap; see the section on the board swap.

### In-repo CPU cap package (2026-10-13, about 22:15 EDT)

The watcher, summarizer, episode analyzer and unit moved into `package/dev-script/claude-cpu-cap` at the user's
request ("Make (almost) everything live in the repo"); the running service starts from that checkout.
It was first copied unchanged, then rewritten (2026-10-13, about 22:40 EDT) into small modules with injectable paths,
unit tests on disposable fixtures (all passing), and zero lint errors and warnings; the live service runs the rewrite.
Event lines now carry the tagged logger prefix.
Host state stays outside the repo: the `threads` file and the sample log.

### Board swap, repaste and motherboard fans (2026-10-18)

What the user reported (about 00:30 to 00:40 EDT on 2026-10-18): the new paste (NT-H2) is on, the motherboard was
swapped, the top rear fan was removed, and the CPU cooler fan now sits much lower. The user fixed the fan curve
about ten minutes before 00:40 EDT, and was connecting every fan directly to the motherboard (the Corsair Commander
Core XT is bypassed). The user said it "seems to be running better now".
The new board is an MSI MAG B850 Tomahawk Max WiFi (`/sys/class/dmi/id/board_name`: MAG B850 TOMAHAWK MAX WIFI
(MS-7E62)). Its `nct6687` chip exposes eight fan inputs and the labels CPU Fan, Pump Fan and System Fan #1 to #6.
At 00:40 EDT the speeds were 1485, 0, 1541, 0, 2063, 0, 2027 and 2590 RPM; which physical fan sits on which header
is not recorded.

Evidence so far, and its limits: the last row with the Gigabyte board readings is 2026-10-17 17:36 UTC and the first
new-board row is 2026-10-18 03:15 UTC. Settled medians (previous 12 samples in the same 20 W bucket) at 120 to 140 W
were 95.4 degrees at 4480 MHz on the old board (1033 samples) and 82.1 degrees at 5045 MHz on the new one
(18 samples); at 180 to 200 W the new board held 90.1 degrees at 5183 MHz (57 samples); 100 to 120 W matched
(75.5 and 75.1) and 40 to 60 W moved from 51.9 to 49.3. The old 180 to 200 W row (44 samples, 70.8 degrees) is not
explained. The new rows span about 1.4 hours, all from before the fan curve fix, with no run-to-run spread
measured, and paste, board, fan removal and cooler height changed together, so no single change can be credited.
Repeat the comparison after a few hours of normal use, using rows after the fan curve fix.

Rerun at 18:07 EDT on 2026-10-18 (`node ~/temp/agent/before-after-swap.ts [fix-time-utc]`, a scratch script that
splits at the last row with `board1_c` filled and at the fix time taken as 04:30 UTC; `~/temp/agent/band-detail.ts
<lowest-watts-of-band>` prints per-chiplet medians for one band). The after-fix era spanned 17.6 hours
(04:30 to 22:07 UTC; 23309 rows, because burst mode writes a row every 2 seconds; the first version of this note
divided rows by 360 and overstated every span in this section) and had 885 settled samples. The stop condition
(at least 100 settled samples in both the 120 to 140 W and 180 to 200 W buckets) was met only for the first:
170 samples at 120 to 140 W and 4 at 180 to 200 W.
- 120 to 140 W settled: old board 95.4 degrees median (p90 96.0, 1033 samples, 4480 MHz); new board after the fix
  80.0 degrees (p90 85.8, max 87.8, 170 samples, 5057 MHz). Before the fix the new board read 82.1 degrees (18 samples).
- 40 to 60 W: 51.9 old, 51.1 new. 100 to 120 W: 75.5 old, 75.8 new (385 samples). Both match.
- 140 to 160 W: 82.4 degrees (33 samples); 160 to 180 W: 83.4 (23); 180 to 200 W: 90.0 (4 samples, too few to use).
- 80 to 100 W looks worse on the new board (81.9 against 60.3 settled) but the load differs: the band medians are
  28.2 W on the first chiplet, 13.5 W on the second and load 4.6 on the new board, against 23.7 W, 17.2 W and load 10.3
  on the old one. The hotter chiplet sets Tctl, so band medians of Tctl mix load shapes. This is read as a load shape
  effect, not shown to be a cooling change.
- The CPU fan ran at a median 1056 RPM across the rows at 120 to 140 W (system fans 5 and 6: 1479 and 1768), so the
  quiet curve leaves speed unused; raising it is the user's noise trade, not decided.
Caveats stand: paste, board, fan removal, cooler height and fan curve changed together; ambient temperature is not
logged; run-to-run spread was not measured. Still queued: the 180 to 200 W bucket needs at least 100 settled samples
(sustained load near 190 W for about 17 minutes). Next check: 2026-10-19 18:07 EDT by a session-only cron job; this
paragraph is the durable record.

Correction from the user (2026-10-18, after the 18:07 EDT rerun): the comparison is still not apples to apples,
because the fan curves were changed once more after the first fix. The log supports this. The CPU fan on the new board
ran a median 1486 to 1501 RPM at 70 to 90 degrees Tctl until about 01:00 to 02:00 EDT, then 960 to 1200 RPM for the
rest of 2026-10-18, so the "after the fan curve fix" era from 04:30 UTC mixes at least two curves; the exact times of
the user's changes are not recorded. On the old board the Corsair channels ran at their maximum while the CPU sat at
95 degrees: medians at 120 to 140 W were 1996, 2589, 0 (unused), 2034, 1504 and 1491 RPM on `fan1_rpm` to `fan6_rpm`
(3178 rows; which channels drive the cooler was not checked). So the fan settings were never matched between the boards,
and the old board had the faster fans. Read the 120 to 140 W result above as: the new board holds 80 degrees at fan
speeds well below the old maximum, not as a controlled comparison. A fair comparison needs equal watts and equal CPU
fan RPM, or a curve held fixed for the measurement window.

Curve eras found in the log (2026-10-18; `~/temp/agent/curve-eras.ts` prints the steps): the CPU fan ran a median
1465 to 1503 RPM at 75 to 85 degrees Tctl from the first fan data (00:46 EDT) until 02:00 EDT, 971 at 02:15 EDT and
mostly 955 to 1000 after that. It drifted up to about 1150 in the afternoon; smaller curve edits and load sitting higher
inside the band were not told apart. The main change is at about 02:10 EDT (06:10 UTC).
Fan speed cannot be the comparison key: the curve sets RPM from Tctl, so grouping by RPM groups by the outcome (in the
80 to 100 W bucket the higher RPM band was the hotter one, 82.5 against 72.6 degrees). This replaces the equal-RPM plan
the user chose (2026-10-18) as the way to compare; the comparison uses windows with one curve instead
(`~/temp/agent/before-after-swap.ts [start-utc] [end-utc]`).
Settled medians, 04:46 to 06:10 UTC (about 1490 RPM): 120 to 140 W 80.6 degrees (8 samples, too few), 140 to 160 W 81.4
(23), 160 to 180 W 82.5 (18), 180 to 200 W 90.0 (4). From 06:10 UTC (about 980 RPM and up, 16.6 hours): 120 to 140 W
80.0 (164), 140 to 160 W 84.9 (17), 160 to 180 W 85.0 (5).
Reading: at 120 to 140 W the new board sits near 80 degrees at either fan speed, so the gap to the old board's 95.4
degrees (fans at their maximum) is not a fan speed effect, because the new fans ran slower. At 140 to 160 W the slower
fan reads about 3.5 degrees warmer. The old Corsair channels were not mapped to the cooler (CoolerControl no longer
lists that device), and fan RPM is not airflow (a fan was removed and the cooler fan moved lower), so this does not
separate paste, board mounting and airflow.

Decision (user, 2026-10-18): switch the sample log to the motherboard fans. The watcher no longer reads CoolerControl;
the seven Corsair columns stay in the header as empty slots so older rows keep their positions, and eleven columns
were appended (eight `nct6687` fan speeds and its System, VRM MOS and PCH temperatures). Rejected: keeping the
Corsair columns and leaving them empty (still logs a failed-read warning, records no fan speed), and dropping fan
columns (loses the link between a temperature change and a fan speed change).

## Commits so far

- `6bb8cc9a7d`, `25909f463a`, `609b693cf5`, `bc561191bb`, `6c8d57d685`, `59c107b737`: the runbook, in order.
- `1aedd30d61`, `bdae3faedd`: this handover.
