# dsh-rewind-plugin adoption

## Decision

Adopt `dsh-rewind-plugin@0.17.1`
(npm package `dsh-rewind-plugin`,
repository `SiriLee/dsh-rewind`,
revision `6e627cc0fa4e9a0991db8398ca1967bc361e98f8`,
tag `v0.17.1`,
tarball `shasum 6fc7a017c7bd128e67f094e5bee24cf279736029`)
into the `web` DeepSeek Harness profile,
for in-window conversation rewind with optional workspace file restore.

## Authorizing request and date

- 2026-10-21:
  the workspace owner upgraded the harness to `dsh` 0.2.1-rc.2
  and authorized plugin setup
  ("I updated dsh myself.
  You config the plugin.").
- The prerequisite upgrade from `dsh` 0.2.1-alpha.4 to 0.2.1-rc.2 was
  performed by the owner,
  satisfying the vet report's named prerequisite.

## Governing records

- Vet report:
  [doc/audit/tech-dsh-rewind-vet-2026-10-20.md](../audit/tech-dsh-rewind-vet-2026-10-20.md)
  (score 83.3 of 100 in context B,
  hard gates and evidence records E1 to E7).
- Governing skill:
  `.agents/skills/choosing-technology/SKILL.md`,
  commit `37117e36397e30233062828581a5e38217da9458`,
  SHA-256 `552ac9955299b65a9f921a4e836b60a3fabc15d3477eeb8ec2bfb3f400647f9b`.
- Capability background:
  [doc/troubleshooting/dsh-web-sent-message-edit-unavailable.md](../troubleshooting/dsh-web-sent-message-edit-unavailable.md).

## Hard constraints and frozen weights

Hard constraints carried from the vet:

1.  Installs and activates on the runtime in service without a version
   exemption.
2.  Public inspectable source under an OSI license.
3.  No runtime network endpoints.
4.  No runtime credential or secret access.
5.  Session history never rewritten in place.
6.  Web profile support.

Frozen criteria weights were all 1
(target-runtime compatibility,
supply-chain provenance,
runtime safety,
feature fit,
human auditability,
maintenance responsiveness),
because the owner named no priorities.

## Ranking and rejected alternatives

Ranking in context B
(after the runtime upgrade):
  `dsh-rewind-plugin` 0.17.1
  >
  no plugin.

- Reason:
  the plugin supplies the requested edit-and-rerun capability with
  verified provenance and append-only history,
  while the baseline supplies none of it.

Alternatives recorded but not adopted:

- `Locietta/dsh-rewind-retry`:
  fork-based edit-prompt actions;
  leanest visible alternative,
  but never deep-validated,
  so it cannot outrank a validated candidate.
- `Anionex/dsh-turn-rewind`,
  `extracurricular-ai/dsh-filesnap`,
  `XSJUSTC/dsh-rewind`,
  `wyq09/dsh-rewind`:
  survive screening on metadata but were not deep-validated
  (`dsh-filesnap` additionally needs its Rust-core provenance gate).
- `L0251314/dsh-rewind`:
  desktop-client scoped,
  not the web profile.
- `dpskh/dsh-rewind`:
  fold-into-report compaction,
  not message rewind.
- `ddll8023/dsh-rewind`,
  `jiangr100/dsh-rewind`:
  stale derivative and prompt-skill replica.
- `PerryLink/dsh-checkpoint-rewind`,
  `skkaczsh/dsh-checkpoint-rewind`:
  git-managed workspace required.
- `Myoontyee/deepseek-harness` fork prototype:
  replaces the harness runtime instead of installing a plugin.

## Integration boundary

- Installation:
  plugin manager install of `dsh-rewind-plugin@0.17.1` into the `web`
  profile,
  which adds the bundle patch row `dsh-rewind-plugin` and a pnpm
  dependency in the profile directory.
- Configuration:
  the profile user patch layer
  `~/.dsh/profiles/web/cordis.patch.yml` carries the adopted policy
  values
  (`fileCheckpointingEnabled: true`,
  `autoCleanupEnabled: true`,
  `autoCleanupMaxAgeDays: 30`);
  store knobs stay at plugin defaults
  (dedup on,
  `maxFileBytes` 8 MiB,
  `maxAnchorGroups` 100,
  `autoSweepIntervalMs` 24 h).
- Effects on data:
  session logs gain appended `dsh-rewind` producer markers only,
  never in-place rewrites;
  before-backups of tool-written files land under
  `~/.dsh/rewind-snapshots`,
  bounded per session and swept after 30 days.

## Migration

None.
No incumbent rewind capability existed;
first-party queue edit,
steer,
and branch remain as they were.
Existing sessions are read as before.

## Exit and rollback

- Disable:
  plugin manager `set_plugin` with `enabled: false`,
  or `dsh plugin --profile web remove dsh-rewind-plugin`.
- Configuration rollback:
  delete the `dsh-rewind-plugin` entry from the profile patch layer.
- Data:
  `~/.dsh/rewind-snapshots` holds only backup copies and is safe to
  delete;
  deleting it forfeits file restore for older rewinds.
- Session logs keep their appended markers after removal,
  and the plugin's downgrade-safety contract keeps them replayable.

## Verification status at adoption

- Live host row active:
  plugin manager `list_plugins` shows
  `include:dsh-rewind-plugin`
  with `fiberPhase: active` and `patchId: dsh-rewind-plugin`.
- Composition and config:
  `dsh --profile web --dump-config` shows the row patched by the profile
  patch layer with the adopted values.
- Client half:
  confirmed by the workspace owner on 2026-10-21 after a page reload
  (rewind control on message hover and `/rewind` in the composer).
- Not exercised here:
  a live `/rewind` run against a disposable message.

## Revisit triggers

- A `dsh` release outside the plugin's engine ceiling
  (`<0.3.0-0`).
- Any `dsh-rewind-plugin` release past 0.17.x,
  given its harness-adaptation history
  (upstream issues 40,
  43,
  52).
- Any session-log replay failure or restore misfire
  (the destructive path).
- Snapshot store growth under `~/.dsh/rewind-snapshots` beyond
  expectation.
