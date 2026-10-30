# amdgpu fdinfo on Linux 7.2.11 inflates per-process VRAM when summed per fd instead of per `drm-client-id`

## Symptom

Attributing GPU memory by summing `drm-resident-vram` from every `/proc/<pid>/fdinfo/*` line
reports per-process usage far above both the card counter and the buffers the processes hold.

Measured 2026-10-30 on an RX 7600 (`card1`,
8176 MiB VRAM,
KDE Wayland,
four outputs):

- Per-fd sum:
  `plasmashell` 2432 MiB,
  `ghostty` 1406 MiB,
  `coolercontrol` 1334 MiB,
  `equibop` 1169 MiB,
  `Xwayland` 920 MiB,
  `firefox-bin` 907 MiB;
  grand total 8788 MiB.
- Same moment,
  `cat /sys/class/drm/card1/device/mem_info_vram_used`:
  4320 MiB.
- One count per `drm-client-id`:
  `plasmashell` 808 MiB,
  `ghostty` 498 MiB,
  `coolercontrol` 445 MiB,
  `firefox-bin` 302 MiB,
  `equibop` 289 MiB,
  `Xwayland` 227 MiB.

Surface patterns:

- Fails:
  any aggregation that iterates `/proc/<pid>/fd/*` and sums the per-fd memory keys.
- Fails silently:
  unprivileged scans of a compositor that is non-dumpable.
  `ls /proc/6397/fd` as the owning user returns `Permission denied`,
  so `kwin_wayland`
  and the buffers it imports vanish from the report.
- Works:
  one count per `drm-pdev` + `drm-client-id`,
  cross-checked against `mem_info_vram_used`.

## Root cause

### Step 1: fdinfo memory keys describe one `drm_file`, not one file descriptor

The contract is documented in [drm-usage-stats-rst] (quote confirmed 2026-10-30 via
`agent-browser read`):

> Unique value relating to the open DRM file descriptor used to distinguish duplicated
> and shared file descriptors.
> ... Userspace should make sure to not double account any
> usage statistics by using the above described criteria in order to associate data to
> individual clients

amdgpu prints the counters per `drm_file` in
`drivers/gpu/drm/amd/amdgpu/amdgpu_fdinfo.c:58` ([pinned source]):

```c
void amdgpu_show_fdinfo(struct drm_printer *p, struct drm_file *file)
{
	...
		drm_print_memory_stats(p,
				       &stats[i].drm,
				       DRM_GEM_OBJECT_RESIDENT |
				       DRM_GEM_OBJECT_PURGEABLE,
				       pl_name[i]);
```

Every `dup(2)` of a DRM fd repeats the same `drm-client-id` with the same numbers,
so a per-fd sum multiplies one client by its fd count.

### Step 2: the inflated figures equal per-client totals times fd count

Inflation factors from the Symptom catalog,
deduped totals as denominator:
`gearlever` 125 MiB over 1 fd = 1.0x,
`plasmashell` 2432 MiB over 3 fds = 3.0x,
`firefox-bin` 907 MiB over 6 fds holding 2 clients = 3.0x,
`equibop` 1169 MiB and
`Xwayland` 920 MiB = about 4x.
The factor tracks fds repeating the dominant client,
not app behavior.

### Step 3: shared buffers count in every holder and the compositor imports nearly all of them

[drm-usage-stats-rst] defines the shared key:

> The total size of buffers that are shared with another file (i.e.,
> have more than one
> handle).
> The same requirement to avoid double-counting that applies to
> drm-total-\<region\> also applies here.

`kwin_wayland` reports 3620 MiB total of which 3446 MiB is `drm-shared-vram`:
its handle table is mostly imports of client window buffers,
which also appear in the
owning apps' totals.
The card arithmetic closes once:
exclusive sum 2018 MiB plus about
2302 MiB of unique shared buffers equals the 4320 MiB card counter,
while
`drm-purgeable-vram` across all clients is only about 55 MiB,
so almost nothing is
reclaimable cache.

### Step 4: the compositor is invisible to unprivileged scans

`kwin_wayland` runs non-dumpable (its `/proc/6397/fd` denies the owning user),
so user-level walks attribute nothing to it while its imports inflate every other
holder's shared column.
Root reads see it;
the harness below is run with `sudo` for
that reason.

### Superseded reading from the 2026-10-30 session

The first diagnosis blamed shared-buffer double counting alone and called
`plasmashell` a 2.4 GiB holder.
That is disproved by the fd-count factor match in
Step 2 and by the GEM cross-check:
deduped fdinfo totals match `amdgpu_gem_info`
per-pid sums exactly (`plasmashell` 808 = 808,
`ghostty` 498 = 498,
`coolercontrol` 445 vs 442).
A GEM-dump parser keyed to the most recently inserted
pid also misattributed `kwin_wayland`'s 120 objects to `systemd-logind` (2344 MiB)
and `Xwayland` (1502 MiB);
the corrected parse and an independent `awk` agree on
`kwin_wayland` 3618 MiB / 120 objects and `Xwayland` 227 MiB / 24 objects.

## Verification

### Version under test

- Linux `7.2.11-ogc4.1.fc44.x86_64`,
  amdgpu,
  Navi 33 (RX 7600),
  Mesa on KDE Wayland.
- Cited source pinned at commit `a9547379cb381e2a036e3261a6becc17973ac470`.

### Harness

Deduped per-process VRAM (needs root so non-dumpable processes are visible):

```ts
// fdinfo-vram-harness.ts: one count per drm client (pdev + client-id), then per-pid rollup.
import { readdirSync, readFileSync } from 'node:fs'

const units: Record<string, number> = { B: 1, KiB: 1024, MiB: 1024 ** 2, GiB: 1024 ** 3 }
const seen = new Set<string>()
const perPid = new Map<number, { comm: string, total: number, shared: number }>()

for (const pidText of readdirSync('/proc').filter((p) => /^\d+$/.test(p))) {
  let fds: string[]
  try {
    fds = readdirSync(`/proc/${pidText}/fd`)
  } catch {
    continue
  }
  let comm = '?'
  try {
    comm = readFileSync(`/proc/${pidText}/comm`, 'utf8').trim()
  } catch {}
  for (const fd of fds) {
    let text: string
    try {
      text = readFileSync(`/proc/${pidText}/fdinfo/${fd}`, 'utf8')
    } catch {
      continue
    }
    const pdev = text.match(/drm-pdev:\s*(\S+)/)?.[1]
    const clientId = text.match(/drm-client-id:\s*(\S+)/)?.[1]
    if (!pdev || !clientId) continue
    const key = `${pdev}:${clientId}`
    if (seen.has(key)) continue
    seen.add(key)
    const pick = (name: string): number => {
      const m = text.match(new RegExp(`${name}:\\s*(\\d+)\\s*(B|KiB|MiB|GiB)?`))
      return m ? Number(m[1]) * units[m[2] ?? 'B'] : 0
    }
    const row = perPid.get(Number(pidText)) ?? { comm, total: 0, shared: 0 }
    row.total += pick('drm-total-vram')
    row.shared += pick('drm-shared-vram')
    perPid.set(Number(pidText), row)
  }
}

const rows = [...perPid.entries()].map(([pid, r]) => ({ pid, ...r, exclusive: r.total - r.shared }))
  .sort((a, b) => b.exclusive - a.exclusive)
for (const r of rows) {
  const mib = (bytes: number): string => (bytes / 1024 ** 2).toFixed(0)
  console.log(`${mib(r.exclusive)} exclusive ${mib(r.total)} total ${mib(r.shared)} shared MiB  ${r.pid}  ${r.comm}`)
}
```

Run:

```bash
sudo node fdinfo-vram-harness.ts
```

GEM cross-check (also needs root):

```bash
sudo cat /sys/kernel/debug/dri/1/amdgpu_gem_info | awk '
  /^pid/ {pid=$2} /byte VRAM/ {sum[pid]+=$2; n[pid]++}
  END {for (p in sum) printf "%d MiB in %d objects  pid %s\n", sum[p]/1048576, n[p], p}'
```

### Patterns that work cleanly

- The deduped harness above,
  run twice back to back,
  returns identical rows:
  `plasmashell` 470 exclusive / 811 total,
  `ghostty` 364 / 488,
  `coolercontrol` 351 / 445,
  `firefox-bin` 238 / 302,
  `equibop` 196 / 289,
  `kwin_wayland` 176 / 3626.
- Those totals match the `awk` GEM sums per pid (`kwin_wayland` 3618 MiB / 120 objects,
  `plasmashell` 808 / 113,
  `ghostty` 498 / 44).
- Card arithmetic closes:
  2018 MiB exclusive + about 2302 MiB unique shared = 4320 MiB
  `mem_info_vram_used`.
- A root GEM read attributes nothing to `systemd-logind` once blocks are keyed to their
  own header pid,
  so the "logind holds 2.3 GiB" reading disappears with the parse fix.

### Patterns that fail (dup-fd inflation)

- The per-fd sum catalog in the Symptom section,
  up to 4x overstatement per app and
  8788 MiB against a 4320 MiB card.
- Any unprivileged attribution that reports a full picture:
  `kwin_wayland` is missing
  entirely (`/proc/6397/fd`:
  `Permission denied`).

## Verified workarounds

### Deduplicate by `drm-pdev` + `drm-client-id` before summing

The harness above.
Tradeoff:
shared buffers still count in every holder's total,
so
holder sums still exceed card usage;
use `total - shared` for exclusive holdings and
accept that the unique shared size is not exported by the interface.

### Cross-check with the root GEM dump and the card counter

The `awk` line above plus `mem_info_vram_used`.
Tradeoff:
needs root,
and per-pid handle
tables include imported buffers,
so per-pid sums double count shared buffers across
holders;
agreement with deduped fdinfo totals is the validation,
not the raw sum.

### Attribute the compositor with root reads

`sudo` fdinfo walks see `kwin_wayland`;
unprivileged ones cannot.
Tradeoff:
root access,
and nothing else in the report changes.

## What does not work

- Per-fd summing of `drm-resident-vram` or `drm-total-vram` (failing catalog).
- `/sys/kernel/debug/dri/1/gem_names` for de-duplicating shared buffers:
  it prints only
  a header row here (`name size handles refcount`) because no legacy GEM flinks are used.
- Unprivileged per-process attribution on a session whose compositor is non-dumpable.
- A GEM-dump parser that keys object lines to the most recently inserted pid instead of
  the current block header;
  it moved `kwin_wayland`'s 120 objects onto `systemd-logind`
  (2344 MiB) and `Xwayland` (1502 MiB).

## Measured inventory, 2026-10-30

Why the corrected holders hold what they hold (buffer sizes from `amdgpu_gem_info`,
`2160x3840x4 = 31.6 MiB` per full-screen RGBA layer on the 4K portrait outputs):

- `kwin_wayland` 3618 MiB / 120 objects:
  imports of visible window buffers plus its own
  render targets;
  3446 MiB is shared with the apps that own the windows.
- `plasmashell` 808 MiB / 113:
  wallpaper and panels across four outputs (one 64 MiB
  surface,
  five 41 MiB layers,
  many small widget textures).
- `ghostty` 498 MiB / 44:
  fourteen 31 MiB screen-sized render buffers plus atlas buffers.
  A throwaway 20x20 window measured 191 MiB including one 31 MiB buffer,
  so ghostty
  allocates screen-resolution targets regardless of window size.
- `coolercontrol` 445 MiB / 31:
  Qt 6 with `libQt6WebEngineWidgets` (embedded Chromium)
  and Qt Quick;
  one 142 MiB texture (about 6100x6100 RGBA) plus six 31 MiB surfaces.
- `firefox-bin` 302 MiB / 82:
  two 32 MiB surfaces plus many 2-8 MiB layer textures.
- `equibop` 289 MiB / 59 (Electron):
  three 31 MiB surfaces plus textures.
- `Xwayland` 227 MiB / 24:
  X11 client surfaces at full scale (two 74 MiB buffers behind
  the JetBrains Toolbox window).
- `music-player` 133 MiB / 15,
  `gearlever` 123 MiB / 14,
  `xwaylandvideobridge` 38 MiB / 8.
- Purgeable (reclaimable) across all clients:
  about 55 MiB.

## Upstream filing decision

### Duplicate and fix-in-progress search, 2026-10-30

- `gh search issues 'amdgpu fdinfo double count drm-client-id VRAM'` and
  `gh search issues 'per-process VRAM usage duplicated file descriptors'`:
  zero hits.
- `gh search prs 'amdgpu fdinfo memory stats'`:
  zero hits;
  no one is fixing anything here.
- Prior art found by web search:
  the [rework series] reworked amdgpu stat tracking and
  clarified `drm-usage-stats.rst` in November 2024.
  No tracker report of this consumer
  misreading exists.

### The 6-constraint check

1. Is it upstream's fault?
   No. The kernel documents the per-client contract and the
   non-double-accounting requirement in [drm-usage-stats-rst];
   the failure is in
   consumers that aggregate per fd.
2. Can upstream fix it?
   Not applicable:
   nothing in the interface is broken.
   An aggregate
   key or client-name enforcement would be an extension,
   not a fix.
3. Are they supporting this use case?
   Yes:
   the doc states the output exists to enable
   "top(1) like userspace monitoring tools".
4. Would the repo welcome our contribution?
   Not evaluated:
   nothing is being filed.
5. Will they likely fix it?
   Not evaluated:
   nothing is being filed.
6. Minimal fix prototype?
   None:
   the corrected aggregation lives in consumer code
   (the harness above),
   which is where the bug is.

Outcome:
nothing to add upstream.
The durable artifact is this doc.

### Out-of-scope check

`.out-of-scope/` was listed 2026-10-30 (`bun-install.md` through
`typescript-project-references.md`);
no entry matches kernel GPU memory accounting.

### Risk to the poster

Nothing is filed.
If a future session drafts anything upstream,
posting puts the
poster's account in front of maintainers and trust or blocklist systems can follow a
person beyond one repository;
filing stays the poster's decision,
and this doc's
findings are reproducible without it.
