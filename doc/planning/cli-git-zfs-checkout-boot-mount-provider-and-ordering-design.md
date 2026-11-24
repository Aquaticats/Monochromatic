# Which unit mounts the checkout, and the ordering the boot gate needs

Design proposal by an agent,
revised a second time,
after a same-vendor re-read of the first revision,
**unapproved and non-final**.
It answers the two parts plan step 19 leaves open
(`doc/planning/cli-git-zfs-worktree-implementation-plan.md`, "Step 19", "Open items"),
folds the findings of that re-read,
and measures them on two throwaway guests.
It designs no wrapper code,
edits nothing under `doc/decision/` or `doc/handover/`,
and applies nothing to any machine.

Every claim in this document carries a label for what it rests on:

- a kept log under this round's `keep/logs/`, named in backticks;
- a journal line, quoted with its Unix timestamp and the boot it belongs to;
- a host read, kept under this round's `keep/`, with the command that printed it;
- a source reading, with the file and the lines;
- `[nothing]`, where a sentence is an inference, a recommendation or an open question.

This round's folder is `~/temp/agent/zfs-step-19-third-revision-2026-11-24/`.
"Round one" means `~/temp/agent/zfs-step-19-checkout-boot-mount-provider-and-ordering-2026-11-23/`,
whose guest logs hold the first round's arms.
"Round two" means `~/temp/agent/zfs-step-19-design-after-its-gate-review-2026-11-24/`,
whose logs this document still cites where this round did not re-measure a fact.

## What the re-read changed

The re-read
(`~/temp/agent/gate-review-zfs-step-19-revision-2026-11-24/keep/re-read-of-the-revised-step-19-design.md`,
a same-vendor second pass, not an independent review)
found one high defect repeated from the first review,
one high citation defect,
fourteen medium ones,
and the lows it named.
Every finding held when this round checked it against `main`, the kept logs and the sources;
none is answered with a rebuttal,
and each is named here with what became of it.

### The high findings

- H1, the choice V9 unit runs a mode no grammar holds.
  Held: `handler_mode_arguments.rs`'s `MODE_WORDS` holds five words
  (`boot-reconcile`, `start-probe`, `shutdown-unmount`, `holder-scan`, `adopt-worktree-parent`),
  and `rg mount-path-descendant` over `main` finds only the proposal.
  Folded in: choice V9's option A now says,
  in its own words and under "What each part waits on",
  that the root-modes design must add the mode
  `mount-path-descendant`, taking `--dataset <dataset>`,
  refusing a dataset whose mountpoint is not inside the verified checkout,
  and **idempotent on an already-mounted dataset**:
  exit 0 and a report, never the status one the `zfs` program returns for a second mount
  (source reading, `zfs-2.4.4` `cmd/zfs/zfs_main.c`, the `OP_MOUNT` branch:
  `explicit` and already mounted prints `filesystem already mounted` and returns 1).
  That idempotence is measured as needed, not optional:
  a provider restart re-runs every descendant unit, and the idempotent stand-in is what kept the
  restart arm green (kept log `restart-provider.log`).
  The option set also says what ships while the mode does not exist:
  option B, the boot unit's own pass, measured green this round (kept log `evidence-v9b.log`).
- H2, the start bound's evidence was cited to logs that do not hold it.
  Held, line by line: `evidence-bound.log` and `evidence-bound-late.log` of round two hold no timeout
  line and no sessions line, because they were read 7 s and 14 s into a boot whose bound fired at 20 s
  (kept file, in the re-read's `work/keep/` folder,
  `cited-timestamps-and-lines-against-kept-logs.log`).
  Folded in by re-running the arm with `systemd-user-sessions.service` in the journal filter and reading
  the file only after the bound fired, at uptime 37.11 s, plus a follow-up read at 37.56 s (kept logs
  `evidence-bound.log`, `evidence-bound-late.log`, boot `cb7b74f3-97bd-4919-950b-ec9b3c7c0a49`):

  ```text
  1795507884.711975 reconcile-git-worktree-clone-mounts.service: start operation timed out. Terminating.
  1795507884.713096 reconcile-git-worktree-clone-mounts.service: Main process exited, code=killed, status=15/TERM
  1795507884.728790 reconcile-git-worktree-clone-mounts.service: Failed with result 'timeout'.
  1795507884.732255 Starting systemd-user-sessions.service - Permit User Sessions.
  1795507884.751115 Finished systemd-user-sessions.service - Permit User Sessions.
  SWEEP stopped 2026-11-24T08:11:24.724Z uptime=26.57
  ```

  Every timestamp quoted in this revision is in the kept log it is cited from.
- M1, citations to the wrong log and missing host facts.
  Held: the four `1795503632.*` and `1795503633.152026` lines are in round two's
  `evidence-green-final.log`, not in `evidence-green-closing.log`, whose boot is `1795503850.*`.
  Folded in: the citations are corrected or replaced by this round's own boot lines,
  and every host read this document uses is in `keep/host-reads.log`
  (`findmnt`, the generator, the cache file, the Zed state, `systemctl cat` of the ZFS units,
  `zfs get`, `stat`, the LSM list and the AppArmor profiles).

### The medium findings

- M2, the "no key units" boot was mis-described, so this host's shape had no arm.
  Held: round two's `evidence-production.log` is a green boot with both key units enabled,
  and `checkout-not-the-dataset` appears in none of its logs.
  Folded in by running this host's shape: no key-loading unit for the checkout at all,
  and a checkout whose key is unavailable, so nothing mounts it.
  Measured (kept log `evidence-no-key-unit.log`, boot `e229df39-45ba-47c7-badd-9dd21dc2565b`):
  the provider finished `Result=success`, `ExecMainStatus=0`;
  the gate read the home underlay and refused;

  ```text
  SWEEP checkout-row=zfsstep19/home zfs
  SWEEP refused=checkout-not-the-dataset 2026-11-24T08:10:03.712Z
  ```

  and `systemd-user-sessions.service` started at `1795507804.250669`.
- M3, the proposed unit text was never booted as one unit.
  Held: the round two arms ran `After=home.mount` plus `After=<learned>`,
  no `TimeoutStartSec=`, an `ExecStopPost=` without its dash, and two of the green boots alone had
  `RequiresMountsFor=`; the guest's home dataset unit is `srv-zfs\x2dstep\x2d19\x2dfixture-home.mount`,
  so `After=home.mount` named a different unit there (round two's `requires-mounts-for-dependencies.log`).
  Folded in by booting the unit text as this document writes it, three ways:
  green (kept log `evidence-as-written.log`, boot `c42af216-c8fc-4946-ad1c-b1449aeb1743`),
  key removed (`evidence-key-removed.log`), and flag removed (`evidence-flag-removed.log`).
  The only substitution is the placeholders, and each arm's own state log prints the unit as systemd
  read it (`state-as-written.log`).
- M4, the group word is a two-word pair.
  Held: the root-modes design's start rule is the two words `monochromatic-git-policy-cli zfs-root`
  followed by a mode word (its choice 1, option A, and `ROOT_GROUP_WORD`),
  and `handler_mode_arguments.rs` reads `zfs-root` as an unknown mode word and refuses with status 2.
  Folded in: the placeholder is now `<root-words>`, defined as that pair,
  and the document says the root-modes design owns the constant.
- M5, the start-bound claims contradicted the proposal's own arm and the manual.
  Held: `systemd.service(5)` at v262 says the start timeout "is also applied to service reloads",
  and `service.c` arms the same `s->timeout_start_usec` per phase.
  Folded in by measuring all three parts on systemd 262.2, the host's own version
  (kept log `systemd-262-arms.log`):
  a 4 s bound with a 6 s `ExecStartPre=` died at 4033 ms;
  a 4 s bound with a 3 s pre and a 3 s start **succeeded at 6037 ms**, so the bound is per command;
  a 3 s bound with a 6 s `ExecReload=` failed the reload at 3102 ms while the unit stayed `Result=success`;
  and a 3 s bound whose stop pass slept 5 s reached `Failed with result 'timeout'` 5.01 s after the
  timeout, with the dependent unit starting only then.
  "What a hung start costs" now states the worst case as an arithmetic of those three numbers,
  and the unit text gains a `TimeoutStopSec=` line, whose bound this round measured on the host's own
  user manager (kept file `keep/host-user-manager-stopsec.log`).
- M6, the `ExecStartPre=` deferral is stale.
  Held: the probe's deadline is built: `PROBE_DEADLINE` is 350 ms with a 231 ms margin
  (source reading, `package/git-policy/cli/src/native/start_probe/probe_deadline.rs`,
  the two constants and the doc comment that cites the 2026-11-21 measurement).
  Folded in: the line no longer waits, and the unit is written with it.
- M7, the learned-unit "second shape" result was confounded.
  Held: round two's `evidence-learned-unit-required.log` ran with both drop-ins ON,
  while the "only" state ran with both OFF.
  Folded in by running the control the finding asked for, **the provider required and the learned
  unit required, both drop-ins absent**, at a boot on systemd 262.2, plus the same requirement
  without the provider, at a boot on 258 and on 262.2:

  - provider required, learned unit required, no drop-ins (kept log
    `boot-arm-learned-with-provider.log`, boot `8e07e08b-ee3f-44ca-9827-b999a57c365e`):
    the provider finished `Result=success` and made its mount, the learned unit was `ActiveState=active`,
    and the requiring unit was `LoadState=loaded`, `ActiveState=inactive`, `Result=success` with no
    journal line at all;
  - the learned unit alone, both drop-ins off, on 258 (kept log `evidence-learned-unit-only.log`,
    boot `867cba26-983a-4291-933b-31fcd17243bb`): the same silence, `inactive`, `Result=success`,
    no sweep;
  - the learned unit alone on 262.2 (kept log `boot-arm-learned-unit.log`, boot
    `941589a1-33ab-4e02-a73a-f5bbc16764bb`): the same.

  So the rule stands as "order, never require", and round two's "unreliable rather than uniformly
  fatal" is withdrawn: the boot transaction drops the job in every shape measured, and the shape that
  ran green was started by the propagation service's own request, the drop-in round two had left on.
- M8, a provider restart reaches every consumer and descendant, and the choices omitted it.
  Held and measured on both systemd versions.
  On 258, with the gate, the per-dataset unit, two account consumers and the propagation service in
  the journal filter, `systemctl restart zfs-mount.service` gave every one of them a new invocation
  (kept log `restart-provider.log`), and the stop pass reported `stopped-slot=unmounted`.
  On 262.2 the same chain of four units all changed invocation (kept log `systemd-262-arms.log`,
  arm F).
  Folded in: the cost is in V2, V6 and V9, and V9's descendant call is required to be idempotent.
- M9, V9's reasons rested on things that do not exist.
  Held: plan line 2569 names no unit per descendant; the one-program record holds no rule that
  root's mounts come from the one program; and the round two green boot's account consumer finished
  at `1795503850.892713`, before the per-dataset unit at `1795503850.921889`, with no descendant in
  its `SESSION` line.
  Folded in: those sentences are gone, and the benefit V9's option A claims is measured instead:
  a consumer that requires the per-dataset unit sees the mount (kept log `evidence-as-written.log`,
  `SESSION-DESCENDANT mounted=zfsstep19/nosnap/vm-builder-output`), and the same consumer on a boot
  without that unit stays `inactive (dead)` with `Result=success` and no journal line
  (kept log `evidence-v9b.log`).
- M10, the generator's key unit is misdescribed, and V1's example is a hybrid.
  Held: the generator's key unit carries `DefaultDependencies=no`, `Wants=` and
  `After=zfs-import.target`, `StandardOutput=null`, an `ExecStop` that unloads the key, and no
  `[Install]` section; the `Before=zfs-mount.service` belongs to the generated mount unit
  (source reading, `zfs-2.4.4` `etc/systemd/system-generators/zfs-mount-generator.c`, the
  `keyloadunit` block and the mount unit's header).
  Folded in: the key unit's shape is corrected, V1's example is labelled for the machine it is,
  and V1's ranking no longer rests on that example.
- M11, the ZED hook against this host's AppArmor.
  Held as a reading: the profile text grants exec of `/usr/lib/zfs-linux/zed.d/*.sh` only,
  and read access to `/etc/zfs/zed.d/{,*}`, so a symlink to a file under
  `/usr/libexec/monochromatic-git-policy-cli/` has no exec grant.
  The loaded mode cannot be confirmed without root.
  Folded into a new section, "AppArmor on this host: what the hook and the sweep need",
  which states the reading, its limit, and what the hook waits on.
- M12, V8's option set and ranking.
  Held: option A promised a value it does not have, B's cost line called an owner decision
  unsourced, and no option dropped the sessions ordering while relying on V6's per-service
  requirements.
  Folded in: V8 now says what ships, adds that option, and ranks by the worst-case hold it leaves,
  which "What a hung start costs" measures.
- M13, V2's reason is stronger than the records.
  Held: the lost-event design states restart propagation as a fact and names its price (`F17`,
  "at the price of the stop pass"), and the propagation design names the same price (`F39`,
  "unmounts every clone"); neither requires that the boot unit be on `Requires=`.
  Folded in: V2 presents A and B as the owner's choice, with A's measured costs
  (every consumer and descendant restarts; attached clones are unmounted) and B's measured
  alternative.
- M14, examples and a lost fact.
  Held: three of the A examples showed no unit line and no command, and the old caveat that
  `Requires=zfs-mount.service` pulls the unit in without making its condition pass had been dropped.
  Folded in: the caveat is back where V1 and V2 discuss the pull-in, and each A example now shows
  the unit lines, the command and what it printed.

### The low findings

- L1: the argument is in `zed_exec.c:168` (`execle(path, prog, NULL, env)`), and `zed_event.c` only
  builds the environment. Corrected, and quoted that way in "The ZED hook".
- L2: the route-owned example said `/usr/sbin/zfs` while this host installs `/usr/bin/zfs`; the
  example now says `/usr/bin/zfs` and names the guest path it ran under.
- L3: the two transcription logs and the missing `mvm` create and destroy records. This round keeps
  `mvm-create-guest1.log`, `mvm-create-guest2.log`, `mvm-destroy-guest1.log` and
  `mvm-destroy-guest2.log` beside the arms, and "Measurements and logs" says which round two logs
  were transcriptions rather than captured output.
- L4: "newest systemd reachable" needed the qualifier. Corrected and measured: the built-in registry's
  newest is Fedora 43 with systemd 258, and `resolveImage`'s custom-template fallback reaches a
  Rawhide image with systemd **262.2-1.fc46**, which this round ran.
- L5: the stop-order claim cited "the earlier round" with no path. It now names round one's
  `keep/guest-logs/journals-per-boot.log`, with the line numbers, and this document says the shutdown
  arm was not re-run this round.
- L6: the elided `NESTED` quote and the unreported earlier attempt. This revision quotes no
  contiguous-looking text that is not contiguous, and reports each arm's own log in full.
- L7: style. Section names replace position pointers, the bare dates are gone, and the old V4 note
  about step 20's snapshot coverage is restored.
- L8: the claims the re-read checked and found sound are unchanged.

## What it ran, and how to read this document

Three throwaway guests over two images, created and destroyed through `mvm --backend libvirt`, one at a
time,
each destroyed through `mvm` by the name its own `mvm create` printed
(kept logs `mvm-create-guest1.log`, `mvm-destroy-guest1.log`, `mvm-create-guest2.log`,
`mvm-destroy-guest2.log`, `mvm-create-guest2b.log`, `mvm-destroy-guest2b.log`).

- **Guest 1**, `zfs-step-19-third-2026-11-24`: Fedora 43, systemd `258.11-1.fc43`,
  kernel `6.17.1-300.fc43`, OpenZFS `2.4.4-1` built from the release tarball pinned at sha256
  `2a3c70d55a37cc71618a95a60e81ad66530201eb118d37741dc92efcf848c8b1`
  (kept logs `prepare.log`, `fixture.log`, `units-install.log`).
  Every ZFS arm ran here.
- **Guest 2**, `zfs-step-19-262-2026-11-24`: Fedora Rawhide, systemd **`262.2-1.fc46`**,
  kernel `7.4.0-0.rc3.261119g4405c18c95d2.29.fc46.x86_64`, no ZFS.
  This is the same systemd release the host runs (`262.2-1-arch`, host read in
  `keep/host-reads.log`), so the facts that could move with a version were measured at the host's
  version: the start bound's phases, the reload, the stop pass, `RequiresMountsFor=`,
  `Upholds=` and `BindsTo=`, restart propagation, and the learned-unit trap at a boot
  (kept logs `systemd-262-arms.log`, `upholds-arm.log`, `escaping-arm.log`,
  `boot-arm-learned-unit.log`).
  The image comes from `resolveImage`'s custom-template fallback, not the built-in registry
  (source reading, `package/mvm/cli/src/registry.ts`, the `customTemplatePath` branch).
  A second guest of the same image, `zfs-step-19-262b-2026-11-24`, ran the re-read's M7 control
  (kept log `boot-arm-learned-with-provider.log`).

**SELinux ran enforcing in every guest**, and every boot's audit trail was read, not assumed.
Guest 1's agent ran in `unconfined_service_t`, which is `mvm`'s own drop-in for a confining policy
(`doc/troubleshooting/mvm-fedora-guest-confined-by-selinux.md`, and the source reading of
`cloud-init-init-systems.ts`, `guestAgentUnconfinedDropInEntry`), so no arm needed permissive mode.
`ausearch --message AVC,USER_AVC --start boot` reports `<no matches>` in every arm's log
(kept logs `evidence-*.log`, `boot-arm-learned-unit.log`).
Guest 2's custom template does not get that drop-in, so its agent started confined and refused
`mvm push`, writes outside `/run`, and `systemctl daemon-reload`;
the drop-in was written from the guest's own serial console once, and the guest then ran unconfined
with SELinux enforcing (kept log `mvm-create-guest2.log` and `keep/state-so-far.md`).

Three limits, so no result is read wider than it is:

- The programs are stand-ins. The gate, the probe, the stop pass, the per-dataset call and the account
  consumers perform the reads this design names, because the wrapper's root modes are not built.
  Every result is about the units, their ordering edges, the failures they carry and systemd's and
  ZFS's own behaviour, never about the wrapper's handler.
- The checkout is its own encryption root on guest 1, so one arm can make its mount fail by moving
  one key file. This host's checkout inherits the home dataset's key, which is why this round also
  ran the no-key-unit arm, M2.
- No arm made the ZFS mount generator write a mount unit. That branch of the provider rule remains a
  source reading plus the host reads.

## The answer in brief

- The provider is `zfs-mount.service` on a machine of this host's shape: this host's generator writes
  no unit for any dataset here, and `zfs mount -a` is what mounts them
  (host reads, `keep/host-reads.log`).
- The provider's exit status says nothing about the checkout. With the checkout's key unavailable
  and no key-loading unit, the provider finishes `Result=success` and `ExecMainStatus=0` while the
  checkout is not mounted at all (kept log `evidence-no-key-unit.log`).
- What fails closed on a missing checkout key is the key-loading unit,
  `zfs-load-key@<escaped encryption root>.service`, required by the boot unit where one exists.
  Measured: the key unit exits `255/EXCEPTION`, and the gate, both account consumers and the
  per-dataset unit all fail with `dependency` (kept log `evidence-key-removed.log`, boot
  `1010ab84-5d06-4ede-8f5d-fd9880c6b700`).
- Where no key unit exists, the gate's own refusal is the signal:
  `SWEEP refused=checkout-not-the-dataset`, exit status two, and the same fail-closed job results
  (kept log `evidence-no-key-unit.log`).
- Never require a mount unit systemd learned from the mount table.
  Measured at a boot on 258 and on 262.2, and again with the provider also required and both drop-ins
  absent: the requiring unit's start job leaves no journal line at all and the unit stays `inactive`
  with `Result=success` (kept logs `evidence-learned-unit-only.log`, `boot-arm-learned-unit.log`,
  `boot-arm-learned-with-provider.log`).
- Order with `RequiresMountsFor=<checkout path>` rather than by hand. One line put `After=` on the
  root mount and the home dataset's mount unit, and `Requires=` only on the fragmented root mount
  (kept logs `state-as-written.log`, `systemd-262-arms.log` arm G).
- The gate's ordering before `systemd-user-sessions.service` is real, and its cost is three bounds,
  not one: a oneshot starts with `TimeoutStartUSec=infinity`, a start's bound is per phase, a reload
  shares it, and the stop pass runs inside the start job's failure path, so a dependent waits for it
  (kept logs `start-bound.log`, `systemd-262-arms.log`, `evidence-bound.log`).
- The datasets under the checkout path must carry `canmount=noauto`, and something after the
  verified checkout must mount them. With the flag absent and the checkout failing, the provider
  created the mountpoint folders inside the exposed home underlay and mounted the dataset there
  (kept log `evidence-hazard-nested-auto.log`, boot `ce693fde-d03d-4583-a357-78db392ea64e`);
  with `canmount=noauto` nothing was mounted and no folder appeared
  (kept log `evidence-hazard-v4.log`).
- The boot unit text as this document writes it boots green, key-removed and flag-removed
  (kept logs `evidence-as-written.log`, `evidence-key-removed.log`, `evidence-flag-removed.log`).
- A consumer that requires the per-dataset unit sees that dataset mounted; one that names a unit the
  boot does not install gets nothing at all, silently (kept logs `evidence-as-written.log`,
  `evidence-v9b.log`).
- The ZED hook is the root-owned wrapper file under a ZED-legal name, started with no argument, and
  on this host it waits on an AppArmor grant that does not exist yet
  (profile reading, `keep/apparmor-zed-reading.log`).

## The provider on this host, and how the reads decide it

Read-only, on this host, 2026-11-24; the whole output is in `keep/host-reads.log`.

- The checkout path has no mount of its own; `/home` is the pool's home dataset, mounted `shared`:

  ```text
  TARGET SOURCE                  FSTYPE PROPAGATION
  /home  zpcachyos/ROOT/cos/home zfs    shared
  ```

- The ZFS mount generator is installed, `/etc/zfs/zfs-list.cache` does not exist, and
  `/run/systemd/generator` holds no ZFS mount unit.
  `zfs-mount-generator(8)` says the generator reads only that cache file.
- `zfs-zed.service` is `disabled` and `inactive`, so nothing keeps a cache file either.
- What mounts the datasets is the pair the OpenZFS package installs:
  `zfs-import-cache.service` runs `zpool import -c /etc/zfs/zpool.cache -aN $ZPOOL_IMPORT_OPTS`,
  and `zfs-mount.service` runs `/usr/bin/zfs mount -a`, with `ConditionPathIsDirectory=/sys/module/zfs`,
  `Before=local-fs.target` and `WantedBy=zfs.target`.
  `zfs.target`, `zfs-import.target`, `zfs-import-cache.service` and `zfs-mount.service` are all
  enabled, and `/etc/default/zfs` leaves `ZPOOL_IMPORT_OPTS` empty.
- `/usr/lib/systemd/system/zfs-load-key.service` is a symlink to `/dev/null` owned by
  `zfs-utils 2.4.4-3`, there is no `zfs-load-key@.service` file, and
  `systemctl is-enabled zfs-load-key.service` answers `masked`.
  That unit is the package's mask, not a key loader, and never was.
- The host's own key path is a person's: `zpcachyos` has `keyformat=passphrase` and
  `keylocation=prompt` with `keystatus available`, and the home dataset inherits it.
- `home.mount` is learned from the mount table (`FragmentPath=` empty,
  `SourcePath=/proc/self/mountinfo`), and the checkout's own unit name
  (`systemd-escape --suffix=mount --path /home/user/Monochromatic`) is `not-found`.

The path the covered entry sits in, read the same way:

```text
/home                      root:root     0755
/home/user                 user:user     0700
/home/user/Monochromatic   user:user     0755
```

## What decides the provider on another machine

The rule, in the order the facts decide it:

1.  The provider is the generated mount unit `<escaped checkout path>.mount`
    when the distribution installs the ZFS mount generator and
    `/etc/zfs/zfs-list.cache/<pool>` exists and lists the checkout dataset.
    That unit has a fragment, so `Requires=` and `After=` both work on it,
    and systemd mounts the checkout as that unit's own job.
    Its ordering is the generator's: `After=zfs-import.target` and the key-loading unit,
    `Before=zfs-mount.service`.
2.  Otherwise the provider is the unit that runs `zfs mount -a` at boot: `zfs-mount.service` in every
    OpenZFS packaging this route's records name.
    It mounts the checkout as a side effect of one command over every dataset, so the route names
    the actor and not a mount unit, because a mount unit appears only after the actor ran.
3.  Where neither is enabled, nothing mounts the checkout at boot, and nothing loads the checkout's
    key unless the pool's import does.
    The route refuses at boot and names what is missing; `doctor` reports which of the three states
    the machine is in, and whether a key-loading unit exists.

Two consequences a reader must carry:

- A `Requires=` on the unit that runs `zfs mount -a` is a **pull-in and a dependency, not a proof of
  a mount**. The unit carries `ConditionPathIsDirectory=/sys/module/zfs`, so where the module is not
  loaded the start is a condition-skip that counts as success and mounts nothing
  (source reading of the unit text in `keep/host-reads.log`, `systemctl cat zfs-mount.service`).
- A mount unit systemd learns from the mount table cannot be required, only ordered.

## The provider's status is not the checkout's status

This section replaces the earlier failure control, which read a provider exit status one as a
checkout failure. It is not one, and the recommended configuration never produces it.

Measured, this host's shape (kept log `evidence-no-key-unit.log`): the checkout's dataset has
`canmount=on`, its key is unavailable, and no key-loading unit exists for it, so nothing loads the
key and `zfs mount -a` skips the dataset. The provider's own result:

```text
Id=zfs-mount.service
Result=success
ExecMainStatus=0
```

What the boot then shows:

```text
SWEEP start 2026-11-24T08:10:03.699Z mode=boot-reconcile uptime=6.74
SWEEP checkout-row=zfsstep19/home zfs
SWEEP refused=checkout-not-the-dataset 2026-11-24T08:10:03.712Z
1795507804.250669 Starting systemd-user-sessions.service - Permit User Sessions.
```

Three things are true at once, and the design says all three:

- the provider can finish green with the checkout absent, because `zfs mount -a` treats an
  unavailable key as a dataset to skip
  (source reading, `cmd/zfs/zfs_main.c`, `share_mount_one`'s key branch: `if (!explicit) return 0;`,
  reached through `share_mount_one_cb` with `B_FALSE`);
- the checkout-specific failure arrives through the key unit where one exists, and through the gate's
  own refusal where none does;
- nothing was written into the covered entry: the home underlay kept its inventory
  (same log, "the home underlay's inventory").

Measured the other way, with a key unit present and its key file away
(kept log `evidence-key-removed.log`, boot `1010ab84-5d06-4ede-8f5d-fd9880c6b700`):

```text
1795507758.973672 zfs-load-key@zfsstep19-home-monochromatic\x2dgit\x2dcheckout.service: Main process exited, code=exited, status=255/EXCEPTION
1795507758.976797 Dependency failed for reconcile-git-worktree-clone-mounts.service - Reconcile the managed worktree clone mounts.
1795507758.983254 reconcile-git-worktree-clone-mounts.service: Job reconcile-git-worktree-clone-mounts.service/start failed with result 'dependency'.
1795507760.676330 Starting systemd-user-sessions.service - Permit User Sessions.
```

The provider in that same boot finished `Result=success`, `ExecMainStatus=0`, which is the whole
point of the section.

## The key-loading unit, and what happens where there is none

The unit to name is the generator's own: `zfs-load-key@<escaped encryption root>.service`,
written by `zfs-mount-generator` beside the mount unit it serves
(source reading, `zfs-2.4.4` `etc/systemd/system-generators/zfs-mount-generator.c`, the block that
builds `keyloadunit` with `systemd_escape(p_encroot, "zfs-load-key@", ".service")`).
Its whole shape, as the generator writes it:

```ini
[Unit]
Description=Load ZFS key for <dataset>
SourcePath=/etc/zfs/zfs-list.cache/<pool>
Documentation=man:zfs-mount-generator(8)
DefaultDependencies=no
Wants=zfs-import.target
After=zfs-import.target

[Service]
Type=oneshot
RemainAfterExit=yes
StandardOutput=null
StandardError=null
ExecStart=/bin/sh -euc '[ "$(<zfs> get -H -o value keystatus "<dataset>")" = "unavailable" ] || exit 0; exec <zfs> load-key "<dataset>"'
ExecStop=/bin/sh -euc '[ "$(<zfs> get -H -o value keystatus "<dataset>")" = "available" ] || exit 0; exec <zfs> unload-key "<dataset>"'
```

It has no `[Install]` section, and the `Before=zfs-mount.service` line belongs to the **mount** unit
the generator writes beside it, not to the key unit.
This is a correction of the earlier text, which gave the key unit that line.

Two consequences:

- Where such a unit exists, the boot unit requires and orders after it:
  `Requires=zfs-load-key@<escaped encryption root>.service` and the same in `After=`, with the name
  taken from the checkout dataset's own encryption root where it has one and from the home dataset's
  where the checkout inherits.
  Measured: with the key file away, the key unit failed and the gate's start job failed with
  `dependency` before its `ExecStart` ran (kept log `evidence-key-removed.log`).
- Where none exists, nothing loads a key at boot except the pool's import.
  That is this host's state, and the measured consequence is the gate's refusal
  `checkout-not-the-dataset` (kept log `evidence-no-key-unit.log`).
  `doctor` reports that no key-loading unit exists and that the checkout's key must be loaded before
  the provider runs.

The guest is the second shape, so it wrote two key units in the generator's shape and names, one for
the pool's encryption root and one for the checkout's
(`zfs-load-key@zfsstep19.service`,
`zfs-load-key@zfsstep19-home-monochromatic\x2dgit\x2dcheckout.service`; kept log `units-install.log`),
and the failure arm moved only the checkout's key file.

## The units, whole

The files this design installs are the boot unit, one unit per dataset whose mountpoint lies inside
the checkout path, the propagation service, the provider's drop-in, the propagation drop-in, and the
ZED hook.
One line's value waits on a measurement this round does not have, `TimeoutStartSec=`, and choice V8
settles it and says what ships meanwhile.

Two placeholders stand in the unit text, and each is owned by another design:

- `<root-file>` is the root-owned regular file the modes are installed as, which the root-modes
  design's choice 3 fixes: one file under the ZED hook's own name (its option A), one file named for
  the modes (option B), one file named for the wrapper (option C), or a copy or hard link per name
  (option D).
- `<root-words>` is the **two words** the units spell before the mode word:

  ```text
  monochromatic-git-policy-cli zfs-root
  ```

  That pair is the root-modes design's start rule (its choice 1, option A, and its constant
  `ROOT_GROUP_WORD`), and the design, not this one, owns the constant.
  A single group word is not enough: `handler_mode_arguments.rs` reads one word as the mode word, so
  `<root-file> zfs-root boot-reconcile` is refused with status 2 as an unknown mode, and on a file
  whose name rule does not read the words, the same line would run the wrapper as Git with root.

Every unit line in this document holds under any answer to those choices:
the root-modes design drops the words when the start is the hook's own name and reads them otherwise,
and this design spells both the file and the words.

### The boot and shutdown unit

```ini
# /etc/systemd/system/reconcile-git-worktree-clone-mounts.service
[Unit]
Description=Reconcile the managed worktree clone mounts
Requires=zfs-load-key@<escaped encryption root>.service
After=zfs-load-key@<escaped encryption root>.service
Requires=zfs-mount.service
After=zfs-mount.service zfs-import.target
RequiresMountsFor=<checkout path>
Before=systemd-user-sessions.service
Before=umount.target
Conflicts=umount.target

[Service]
Type=oneshot
RemainAfterExit=yes
TimeoutStartSec=<the measured ceiling of one reconcile>
TimeoutStopSec=<the measured ceiling of one stop pass>
ExecStartPre=-<root-file> <root-words> start-probe
ExecStart=<root-file> <root-words> boot-reconcile
ExecReload=<root-file> <root-words> boot-reconcile
ExecStopPost=-<root-file> <root-words> shutdown-unmount

[Install]
WantedBy=multi-user.target
```

This text, with only the placeholders filled, is what booted green, key-removed and flag-removed
this round (kept logs `evidence-as-written.log`, `evidence-key-removed.log`,
`evidence-flag-removed.log`; the unit as systemd read it is printed in `state-as-written.log`),
with two substitutions this design makes for the stand-ins:
the stand-in program name stands where `<root-file> <root-words>` stands, and its argument is the
mode word.

### Why each line

- `Requires=zfs-load-key@<escaped encryption root>.service` and its `After=`.
  The prerequisite whose failure is the checkout's, on a machine that has one.
  Measured: with the key file away, the key unit failed and this unit's start job failed with
  `dependency` before its `ExecStart` ran, and so did both account consumers and the per-dataset
  unit (kept log `evidence-key-removed.log`, journal lines `1795507758.973672` to
  `1795507758.983254`).
- `Requires=zfs-mount.service` and `After=zfs-mount.service zfs-import.target`.
  What the requirement buys is a pull-in, a dependency failure and restart propagation.
  The pull-in is weaker than it reads: the provider's
  `ConditionPathIsDirectory=/sys/module/zfs` turns a start without the module into a condition skip
  that succeeds and mounts nothing (source reading of the unit text, `keep/host-reads.log`).
  Restart propagation is load-bearing at runtime: measured on 258 and on 262.2, a provider restart
  takes this unit, every per-dataset unit and every consumer that requires them through their stop
  pass and starts them again (kept logs `restart-provider.log`, `systemd-262-arms.log` arm F).
  At boot the ordering edge adds nothing: the same unit with no edge to the provider still started
  after it and passed (round two's `evidence-no-provider-edge.log`,
  `Finished zfs-mount.service` at `1795503075.003525`, `Starting reconcile-…` at
  `1795503075.501653`).
  Replace the name with the generated mount unit where the generator writes one; keep the
  `zfs-import.target` edge either way.
- `RequiresMountsFor=<checkout path>`.
  This replaces the two hand-written ordering lines.
  Measured on both systemd versions: one line put `After=` on `-.mount` and on the home dataset's
  learned mount unit, and `Requires=` only on `-.mount`, the one of them with a fragment
  (kept logs `state-as-written.log`, `systemd-262-arms.log` arm G).
  At the moment the unit loads, the checkout's own learned unit may not exist yet, so that edge
  orders the stop half rather than the first start; what orders the first start is the provider's
  requirement and `zfs-import.target`.
- `Before=systemd-user-sessions.service`.
  What makes the checkout ready before an account session, and the line whose cost "What a hung start
  costs" measures. Measured in the green boot: the gate finished at `1795507717.347527` and the
  sessions unit started at `1795507718.233646`, after the per-dataset unit at `1795507717.422` too
  (kept log `evidence-as-written.log`).
- `Before=umount.target` with `Conflicts=umount.target`.
  The stop half. Measured at runtime on 258 through the provider restart, which took the units down in
  reverse order and ran the stop pass (kept log `restart-provider.log`:
  `Stopping git-account-session-standin.service` at `1795507731.282283`,
  `Stopping reconcile-…` at `1795507731.284386`,
  `SWEEP stopped … stopped-slot=unmounted`).
  A shutdown arm was not run this round; round one's shutdown order on systemd 255 is in its
  `keep/guest-logs/journals-per-boot.log`, lines 83 to 90, and this round's
  "What the runs do not show" says so.
- `TimeoutStartSec=<the measured ceiling of one reconcile>` and
  `TimeoutStopSec=<the measured ceiling of one stop pass>`.
  New in this revision, and their values are not yet known: no run has measured one reconcile or one
  stop pass over this host's storage.
  What the pair buys is measured at provisional values: `TimeoutStartSec=20` on a hung start produced
  `start operation timed out. Terminating.`, `Failed with result 'timeout'`, the stop pass's own
  durable line, and sessions starting after the failure (kept logs `evidence-bound.log`,
  `evidence-bound-late.log`), and a `TimeoutStopSec=` of two seconds killed a five second
  `ExecStopPost=` on the host's own manager (kept file `keep/host-user-manager-stopsec.log`).
  Choice V8 ranks the alternatives and says what ships while the values are missing.
- `ExecStartPre=-<root-file> <root-words> start-probe`.
  The start-probe design's placement, adopted here because plan step 19 installs the unit: one root
  process per boot, in the service manager's own child, before the reconcile.
  The probe's own deadline is built and bounds the probe: `PROBE_DEADLINE` is 350 ms with a 231 ms
  margin over the slowest measured worker phase
  (source reading, `package/git-policy/cli/src/native/start_probe/probe_deadline.rs`, the two
  constants and the measurement its doc comment cites). So the earlier deferral is gone and the line
  ships with the unit.
  The dash prefix is load-bearing and measured: a failing `ExecStartPre=` under the prefix left the
  unit `active` and ran its `ExecStart`, the same line without the prefix failed the unit and skipped
  it, and a timeout inside `ExecStartPre=` sent the unit to its stop action whatever the prefix said
  (kept log `start-bound.log`, three units, each read back).
  The unit's bound is a second, larger bound over the same phase, not the probe's deadline.
- `ExecStart=<root-file> <root-words> boot-reconcile`.
  The pass itself: it verifies the checkout's mount identity, the covered entry's flag and the private
  propagation, then attaches what the intent asks for.
- `ExecReload=<root-file> <root-words> boot-reconcile`.
  The line the lost-event record accepts and the propagation design's printed lever needs.
  Measured in round two: `systemctl reload-or-restart` on the active unit ran this command and no stop
  pass (round two's `reload-gate.log`, `Reloading reconcile-…` at `1795503559.285343`, `Reloaded` at
  `1795503559.315252`).
  A reload re-runs the reconcile on an active checkout, so the mode must be idempotent; the stand-in
  was made so after a first run showed a second bind of the same slot.
- `ExecStopPost=-<root-file> <root-words> shutdown-unmount`.
  The stop pass, and the dash prefix is this design's answer to the round one review's M1.
  The built `shutdown-unmount` always refuses
  (`package/git-policy/cli/src/native/worktree_clone_mount_handler/shutdown_unmount_mode.rs`,
  `run` returning `AWAITING`), so without the prefix every stop fails the unit for a mode whose step
  has not landed.
  With the prefix the refusal is logged and the stop stays clean, and the line loses the prefix when
  the unmount half lands.
- `WantedBy=multi-user.target` with default dependencies.
  This is what makes the unit part of the boot at all, and it carries the implicit `After=basic.target`.

### What a stop does today

Two source readings of `main`:

- `shutdown-unmount` refuses, so the stop line carries `-` until that mode lands.
- `boot-reconcile` exits **zero** when individual clones fail: `boot_reconcile_mode.rs` publishes a
  readiness record with `SweepState::Failed` and still returns `Ok`
  (`boot_reconcile_mode.rs:158` to `:164`), and only a refused prerequisite returns `Err`.
  So a consumer's `Requires=` on this unit carries a boot refusal and not a per-clone sweep failure.
  A clone's failure reaches a consumer through the readiness record, not through the unit's result.

### The units for the datasets under the checkout path

One unit per dataset whose mountpoint lies inside the checkout path, generated by setup and reported
by `doctor`:

```ini
# /etc/systemd/system/git-worktree-nested-<slot-or-name>-mount.service
[Unit]
Description=Mount a dataset whose mountpoint lies inside the verified checkout
Requires=reconcile-git-worktree-clone-mounts.service
After=reconcile-git-worktree-clone-mounts.service
Before=systemd-user-sessions.service

[Service]
Type=oneshot
RemainAfterExit=yes
ExecStart=<root-file> <root-words> mount-path-descendant --dataset <dataset>

[Install]
WantedBy=multi-user.target
```

`mount-path-descendant` does not exist yet.
The root-modes design owns the mode list (`handler_mode_arguments.rs`'s `MODE_WORDS`), and this
design asks it for one more mode, with these properties:

- the word is `mount-path-descendant` and it takes `--dataset <dataset>`, the shape
  `holder-scan` already uses for its own dataset option;
- it refuses a dataset whose mountpoint is not inside the checkout path it verifies first, with the
  same refusal the pass gives (`checkout-not-the-dataset`), and it says which dataset it refused;
- it is **idempotent on an already-mounted dataset**: exit 0 and print what it found.
  Measured as needed: a provider restart re-runs every descendant unit (kept log
  `restart-provider.log`), and the program it would call returns status one for a second explicit
  mount of a mounted dataset (source reading, `zfs_main.c`, the `OP_MOUNT` branch's
  `filesystem already mounted` and `return (1)`).

For this host that is one unit, for `zpcachyos/nosnap/vm-builder-output`.
The measured stand-in ran the same call the mode would make, an explicit `zfs mount <dataset>`, after
checking that the checkout's own mount row is the checkout dataset, and it is idempotent
(kept log `evidence-as-written.log`):

```text
SWEEP checkout-row=zfsstep19/home/monochromatic-git-checkout zfs
NESTED start 2026-11-24T08:08:37.384Z
NESTED mounted=zfsstep19/nosnap/vm-builder-output 2026-11-24T08:08:37.418Z
SESSION-DESCENDANT mounted=zfsstep19/nosnap/vm-builder-output
```

The explicit form matters: `zfs mount -a` skips a `noauto` dataset and skips an unavailable key
silently, while the explicit form refuses with a named message (source reading,
`cmd/zfs/zfs_main.c`, `share_mount_one`'s `canmount` table and key branch, both gated on `explicit`;
measured for the key case in "The route-owned alternative").
`Before=systemd-user-sessions.service` is what closes the window the re-read's M2 named: without
it round one's `noauto` boot finished the descendant unit after the sessions unit
(round one's `keep/guest-logs/journals-per-boot.log`, `Finished systemd-user-sessions.service` at
`1795486959.129715`, `Finished git-worktree-nested-output-mount.service` at `1795486959.180427`);
with it, the closing boot's order is gate, then descendant, then sessions
(kept logs `evidence-as-written.log`).

Choice V9 says who mounts these datasets and why one unit per descendant is ranked first, and says
what ships while the mode does not exist.

### The provider drop-in, from the lost-event record

```ini
# /etc/systemd/system/zfs-mount.service.d/reconcile-git-worktree-clone-mounts-after-checkout.conf
[Service]
ExecStartPost=-/usr/bin/systemctl --no-block --no-ask-password reload-or-restart reconcile-git-worktree-clone-mounts.service
```

Measured in round two on systemd 258, with the gate's requirement written as `Wants=` so the restart
would not move it: after `systemctl restart zfs-mount.service` the gate stayed active and then
`Reloading reconcile-…` and `Reloaded` appeared (round two's `wants-control.log`, journal lines
`1795503693.546036` and `1795503693.566525`), and the `-` prefix kept the provider `active` with
`Result=success`.
With the gate on `Requires=`, the same restart already restarted the gate and that boot's journal
holds no separate reload (round two's `dropin.log`), which is the whole of that result.
The install is the record's, and the record's reason stands: the request is what reconciles after a
post-key-load checkout mount, including a provider started by something other than the boot.

### The propagation service and its drop-in, from the propagation record

```ini
# /etc/systemd/system/checkout-propagation.service
[Unit]
Description=Ask the boot unit to re-verify the checkout's private propagation
BindsTo=<escaped checkout path>.mount
After=<escaped checkout path>.mount

[Service]
Type=oneshot
RemainAfterExit=yes
ExecStart=/usr/bin/systemctl --no-block --no-ask-password reload-or-restart reconcile-git-worktree-clone-mounts.service
```

```ini
# /etc/systemd/system/<escaped checkout path>.mount.d/10-upholds-propagation.conf
[Unit]
Upholds=checkout-propagation.service
```

Measured on systemd 258 and again on 262.2:

- The drop-in is read for a mount unit systemd learned: `systemctl show` reports the `Upholds=` and
  names the drop-in in `DropInPaths`, and the unit's `FragmentPath` is empty with
  `SourcePath=/proc/self/mountinfo` (kept logs `dependencies-read.log`, `upholds-arm.log`).
- `Upholds=` starts the service when the mount unit activates, and `BindsTo=` stops it when the mount
  unit deactivates: on 262.2 the service started at `1795508243.805597` and was stopped at
  `1795508245.851061`, while the consumer the service's request had started stayed active
  (kept log `upholds-arm.log`).
- The service starts before the gate on a boot where both are wanted: in the green boot the service
  started at `1795507717.233498` and the gate at `1795507717.255023`, because the learned unit's
  activation submits the upheld service immediately (kept log `evidence-as-written.log`).

The escaped name must be spelled as `systemctl` prints it, backslashes and all; doubling the
backslashes names a unit that does not exist (measured, kept log `escaping-arm.log`).
`systemd-analyze verify` reports one line for this pair, and it is an artifact of verify, which has no
live manager to find a learned unit:

```text
checkout-propagation.service: Failed to create checkout-propagation.service/start:
Unit <escaped checkout path>.mount not found.
```
So `doctor`'s list of known lines must carry it rather than treat it as a defect, and a reader must
not conclude from it that the dependency is broken: the same spelling resolved once the unit was
loaded (kept logs `escaping-arm.log`, `upholds-arm.log`).

Neither drop-in is installed without the boot unit's `ExecReload=`, which is the propagation record's
own condition: without that line the same request is a restart, which runs the stop pass and unmounts
every attached clone.

## The ZED hook, and what it waits on

The hook is **the root-owned regular file under a ZED-legal name**, and nothing else.

```text
/usr/libexec/monochromatic-git-policy-cli/<the installed file>   regular, 0755, root
/etc/zfs/zed.d/history_event-<hook-name>                          symlink to the file
```

- ZED starts a zedlet by its file name, with no arguments: `execle(path, prog, NULL, env)`
  (source reading, `zfs-2.4.4` `cmd/zed/zed_exec.c:168`), with the environment built in
  `zed_event.c`'s block that fills it before the exec.
- A zedlet's name must carry the zevent class as a prefix followed by a non-alphabetic character
  (`zed.8`); `history_event-…` satisfies that.
- No argument is the attaching sweep: `handler_mode_arguments.rs`'s `parse` returns
  `HandlerMode::AttachingSweep` for an empty argument list.
  There is no `sweep` word in the grammar, and this design writes none.
- No script is installed. The earlier text's `exec <root-mode-entry> <sweep>` file is deleted with its
  justification: "every zedlet here is a `.sh` file" described the packaged zedlets, not a
  requirement of ZED's, and the one-program record retires an installed program of the route's own.

What each part waits on in the root-modes design, by whose choice:

- the file's own name, and whether the file is named for the hook or for the modes: choice 3;
- the hook name's exact spelling, which the record fixes as
  `history_event-mount_git_worktree_clone_datasets` today
  (`handler_install_paths.rs`, `ZEDLET_INSTALL_PATH`) unless that design renames it;
- the two words in every unit line: choice 2, whose constant this design spells;
- whether a root start under another name runs the wrapper: choice 4's placement;
- the words `--version` and the refusals print: choice 5, which only `doctor`'s reading depends on;
- an AppArmor grant, which no design owns yet and which the next section reads;
- the new mode `mount-path-descendant`, which choice V9's option A needs.

## AppArmor on this host: what the hook and the sweep need

This is a **reading of profile text**, not a measurement of enforcement: the loaded-profile list,
`/sys/kernel/security/apparmor/profiles`, is readable by root only, so the profile's mode cannot be
confirmed from an unprivileged session. The whole output is in `keep/apparmor-zed-reading.log`.

What the reading says:

- The host's LSM list is `capability,landlock,lockdown,yama,apparmor,bpf`, and
  `/sys/module/apparmor/parameters/enabled` is `Y`.
- `/etc/apparmor.d/zed` exists, has no `flags=` line, is not in `/etc/apparmor.d/disable/`, and
  `include if exists <local/zed>` has no local file beside it, so the shipped text is what would
  apply if the profile is loaded.
- That text grants exec of exactly two classes of file: the program itself
  (`/usr/bin/zed mr`) and packaged zedlets (`/{usr/,}{local/,}lib/zfs-linux/zed.d/*.sh rix`).
  It grants **read** of `/etc/zfs/zed.d/{,*}` and no exec of anything under it or under
  `/usr/libexec/monochromatic-git-policy-cli/`.
- Whatever a confined ZED starts inherits the `zed` profile. The profile grants no `mount` or `umount`
  rule at all, no write rule for `/run` or for the account's paths, and the `zfs` and `zpool`
  programs only with `rpx`.

So the hook and the pass it starts need, on this host, one of:

- an AppArmor rule set added to the `zed` profile (exec of the installed file, `mount` and `umount`,
  writes where the pass writes, access to the paths it verifies); or
- a profile transition, `px` on a dedicated profile for the installed file, which is the cleaner
  shape for a program that mounts and writes; or
- a decision that the `zed` profile is not loaded on this machine, which only root can confirm.

This design does not choose between those; it lists the grant under what the hook waits on, and
`doctor` should report whether the profile is loaded and whether it can exec the installed file.
Two facts are worth keeping beside the reading:

- The host's own `zfs` and `mount-zfs` profiles do allow `mount fstype=zfs` and `umount fstype=zfs`
  for the `zfs` program (`keep/apparmor-zed-reading.log`), which is how a person's `zfs mount` works.
  That does not help the hook: the hook's own process is not the `zfs` program.
- The hazard arm's mountpoint creation by `zfs mount -a` would also be a profile question here, so
  the hazard's shape on this host is not settled by the guest, where no AppArmor runs.

The root-modes design lists the same point as open ("no profile exists yet"); this section is the
reading that item asks for.

## The covered entry, and the ancestors the earlier text got wrong

Measured as the account, with the checkout unmounted and the entry immutable
(round one's `keep/guest-logs/guest-ancestor.log`, unchanged since and re-read for this text):

```text
mv ~/Monochromatic ~/Monochromatic.moved          -> Operation not permitted     (the flag)
rmdir ~/Monochromatic                              -> Operation not permitted     (the flag)
touch ~/Monochromatic/planted                       -> Operation not permitted     (the flag)
mv <home>/account <home>/account.moved              -> Permission denied           (the parent, root-owned)
mv ~/projects ~/projects.moved                      -> succeeds                    (the parent, account-owned)
   then ~/projects/Monochromatic/planted exists     (planted by root in the arm, not by the account)
zfs mount zfsstep19/home/monochromatic-git-checkout -> Insufficient privileges    (the account)
```

The condition the design needs:

> the covered entry is protected while every directory **above the entry's parent**, up to the home
> dataset's root, is one the account cannot write.

The earlier text's wording, "every directory between it and the home dataset's root is one the
account cannot write", fails at this host's own path, because `/home/user` is the account's directory
(mode `0700`, owner `user:user`, read from the host), and the earlier text then claimed the condition
"holds" there. What holds at `/home/user/Monochromatic` is the corrected condition: `/home` is
root's, the entry is the account's, and the account cannot rename `/home/user` because that needs
write on `/home`. At a deeper path the account owns, such as `~/projects/Monochromatic`, the account
renames `~/projects` and the flag on the entry protects nothing, which the arm shows.

## The failure controls

Five arms this round, each with its own kept log, all on systemd 258 with SELinux enforcing.

- Green, the unit as written: passes. Kept log `evidence-as-written.log`, boot
  `c42af216-c8fc-4946-ad1c-b1449aeb1743`.
  The probe ran, the gate read the checkout row, the flag and the private propagation, attached the
  slot, the per-dataset unit mounted its dataset, the account consumer saw the checkout and the slot,
  the second account consumer saw the dataset, and sessions started last.
  The order, from the journal: gate `1795507717.255023` to `1795507717.347527`, per-dataset unit
  `1795507717.363772` to `1795507717.419298`, its consumer `1795507717.425546` to `1795507717.445132`,
  sessions `1795507718.233646`.
- The checkout's key file away: the key unit fails and the gate's job fails closed.
  Kept log `evidence-key-removed.log`.
- The covered entry's flag removed: refuses and fails closed. Kept log `evidence-flag-removed.log`,
  boot `e36e69bf-8441-46b0-9972-7d7409090919`:
  `SWEEP covered=----------------------`, `SWEEP refused=covered-entry-not-immutable`,
  `ExecMainStatus=2`, and both consumers' and the per-dataset unit's jobs failed with `dependency`.
- No key-loading unit for the checkout and its key unavailable: the provider is green and the gate
  refuses with `checkout-not-the-dataset`. Kept log `evidence-no-key-unit.log`, quoted in
  "The provider's status is not the checkout's status".
- The account's login is not gated by the gate's success, only delayed by its end: in each of the
  three failing arms, `Starting systemd-user-sessions.service` follows the failed job
  (kept logs `evidence-key-removed.log`, `evidence-flag-removed.log`, `evidence-no-key-unit.log`).
  That is why the design puts `Requires=` on the services of the account that need the checkout,
  which is choice V6.

Two hazard arms, re-run this round with SELinux enforcing:

- With the flag absent, the checkout's mount failing and the dataset under the path at
  `canmount=on`, the provider created the mountpoint folders inside the exposed home underlay and
  mounted the output dataset there: the underlay's inventory holds
  `d /srv/zfs-step-19-fixture/home/account/Monochromatic/package/vm-builder/cli/output` and the mount
  table shows the dataset mounted on it, while the checkout itself is absent
  (kept log `evidence-hazard-nested-auto.log`).
- The same with `canmount=noauto`: nothing mounted, and the underlay's inventory is exactly the three
  directories the arm started with, with no `package` folder (kept log `evidence-hazard-v4.log`).

## What a hung start costs

The costs are four numbers, measured on systemd 262.2, the host's own version
(kept log `systemd-262-arms.log`, and one host read in `keep/host-user-manager-stopsec.log`).

The default: a oneshot has no start timeout.
`systemctl show` for a unit with no `TimeoutStartSec=` reports `TimeoutStartUSec=infinity`
(arm A), so a hung reconcile ordered before the sessions unit holds every login with no limit.

The bound is **per phase**, not for the start as a whole:

```text
arm B1: TimeoutStartSec=4, ExecStartPre=-/bin/sleep 6   -> start-exit=1, elapsed-ms=4033,  Result=timeout
arm B2: TimeoutStartSec=4, ExecStartPre=-/bin/sleep 3, ExecStart=/bin/sleep 3
                                                       -> start-exit=0, elapsed-ms=6037,  Result=success
```

So the ceiling covers each of the start's phases separately: a pass that spends its pre-phase running
the probe and its main phase running the reconcile can take twice the line's value.

The same bound covers a **reload**, and a reload that overruns does not fail the unit:

```text
arm C: TimeoutStartSec=3, ExecStart=/bin/true, RemainAfterExit=yes, ExecReload=/bin/sleep 6
reload-exit=1, elapsed-ms=3102, TimeoutStartUSec=3s, Result=success
```

(systemd.service(5) at v262: "the start timeout is also applied to service reloads, regardless of
whether implemented through `ExecReload=` or via the reload logic enabled via `Type=notify-reload`";
and `service.c` arms the same `s->timeout_start_usec` per phase).

The **stop pass extends the hold**, because it runs inside the start job's failure path and the
sessions unit is ordered after the job, not after the kill:

```text
arm D: TimeoutStartSec=3, ExecStart=/bin/sleep 10, ExecStopPost=/bin/sleep 5, dependent After= it
1795508155.691091 Starting arm-d.service...
1795508158.748077 arm-d.service: start operation timed out. Terminating.
1795508158.749128 arm-d.service: Main process exited, code=killed, status=15/TERM
1795508163.759644 arm-d.service: Failed with result 'timeout'.
1795508163.761897 Starting arm-d-dependent.service...
```

That is 3.06 s to the kill and 5.01 s more before the unit reached `failed` and the dependent started.
The stop pass is itself bounded by `TimeoutStopSec=` (default 90 s): on the host's own user manager a
two second `TimeoutStopSec=` killed a five second `ExecStopPost=` after 2.07 s, with
`State 'stop-post' timed out. Terminating.` and `Control process exited, code=killed, status=15/TERM`
(kept file `keep/host-user-manager-stopsec.log`).

The bound at work in a boot, read after it fired (kept logs `evidence-bound.log`,
`evidence-bound-late.log`, `TimeoutStartUSec=20s`, `Result=timeout`):

```text
1795507884.711975 reconcile-git-worktree-clone-mounts.service: start operation timed out. Terminating.
1795507884.713096 reconcile-git-worktree-clone-mounts.service: Main process exited, code=killed, status=15/TERM
1795507884.728790 reconcile-git-worktree-clone-mounts.service: Failed with result 'timeout'.
1795507884.732255 Starting systemd-user-sessions.service - Permit User Sessions.
1795507884.751115 Finished systemd-user-sessions.service - Permit User Sessions.
SWEEP stopped 2026-11-24T08:11:24.724Z uptime=26.57
```

The gate's start was at `2026-11-24T08:11:03.409Z` (uptime 6.39) and sessions opened at
`08:11:24.732Z`, so the login hold was 21.3 s for a 20 s bound and a stop pass that took about a
second.

So the worst case a reader should carry is:

```text
TimeoutStartSec (the pre phase, if it overruns) + TimeoutStartSec (the main phase)
  + the SIGTERM wait + TimeoutStopSec (the stop pass, default 90 s)
```

For a shipped 20 s bound with the default stop timeout that is up to about 130 s of held login, which
is why the unit text now carries `TimeoutStopSec=` as well as `TimeoutStartSec=`, and why choice V8
ranks the options by the hold they leave.

The prefix's limit, measured separately (kept log `start-bound.log`):

- a failing `ExecStartPre=` under `-`: unit `active`, `ExecStart` ran;
- the same line without `-`: unit `failed`, `ExecStart` did not run;
- a **timeout** in `ExecStartPre=` under `-`: `start-pre operation timed out`, `Result=timeout`, and
  the unit's `ExecStopPost=` ran.

The consequence for the design is that the probe must enforce its own deadline, which it does, and
must not rely on the prefix to survive a timeout.

## Never require a mount unit systemd learned

Measured three times on three system managers, in the shape the earlier rounds used.

**systemd 258, at a boot, the learned unit alone required.** The gate's start job never appeared in
the boot's journal, the unit was left `inactive` with `Result=success`, the sweep never ran, and
sessions opened (kept log `evidence-learned-unit-only.log`, boot
`867cba26-983a-4291-933b-31fcd17243bb`, with the provider's drop-in and the propagation drop-in both
off).

**systemd 262.2, at a boot, the learned unit alone required.** A provider unit made a bind mount at
boot, so the learned unit did not exist when the boot transaction was built; the requiring unit's
result was `LoadState=loaded`, `ActiveState=inactive`, `Result=success`, and the boot's journal holds
no line for its start at all (kept log `boot-arm-learned-unit.log`, boot
`941589a1-33ab-4e02-a73a-f5bbc16764bb`).

**systemd 262.2, at a boot, the provider required beside the learned unit, both drop-ins absent.**
The provider finished `Result=success` and made its mount, the learned mount unit was
`ActiveState=active`, and the unit requiring both was `LoadState=loaded`, `ActiveState=inactive`,
`Result=success`, with no journal line for its start (kept log `boot-arm-learned-with-provider.log`,
boot `8e07e08b-ee3f-44ca-9827-b999a57c365e`).
That is exactly the control the re-read asked for, and it settles the confound: requiring the
provider does not make the boot transaction start a unit that requires the learned mount unit.

**Round two's "second shape", now explained.** There the learned unit was required beside a required
provider **with both drop-ins on**, and the gate did run. Its journal puts the gate's first start ten
milliseconds after `Finished checkout-propagation.service`, while the provider's own request had
failed with `Unit <escaped checkout path>.mount not found` (round two's
`evidence-learned-unit-required.log`). With the control's measured silence, the start that ran was
the propagation service's request, the one component the control removes.

So the rule the design states is:
**order with `After=` or with `RequiresMountsFor=`,
and never write `Requires=` on a unit systemd learned from the mount table.**
The same rule covers a unit a boot does not install at all: a consumer that requires the per-dataset
unit on a boot without it stayed `inactive (dead)` with `Result=success` and no journal line
(kept log `evidence-v9b.log`).

The source says why the failure is quiet: a dependency job that cannot be added for `-ENOENT` is
logged at debug level (systemd v262 `src/core/transaction.c`, the block that logs
`Cannot add dependency job, ignoring` through `log_unit_full_errno` with `LOG_DEBUG`), and a learned
mount unit cannot be started anyway: its own start answers `No mount parameters to operate on`
(round one's `keep/guest-logs/journals-per-boot.log`, `No mount parameters to operate on` at
`1795486467.319399`, then `Failed with result 'resources'` at `1795486467.319417`).

The corrected V1 reason follows from it: the lost-event record rejected a mount unit **as a
trigger**, not as a provider (`doc/decision/cli-git-zfs-lost-event-triggers-hook-nudge-and-cost-cap.md`,
"What was not chosen"), and the account-writable-path exposure applies to `zfs mount -a` just as much
as to a route-owned unit.

## The route-owned alternative, measured

Choice V1's second option is a unit the route owns, which mounts the checkout explicitly, with the
checkout dataset at `canmount=noauto` so the provider's scan skips it. Round two ran it three ways,
and its whole text is in that round's `guest-scripts/03-units.sh`.
The program path is this host's, `/usr/bin/zfs`; the guest that ran it used its own
`/opt/openzfs-2.4.4/sbin/zfs`, which is the same call.

- Green: it mounts the checkout. Kept log (round two) `evidence-route-owned.log`: the provider
  finished at `1795503146.401532` with the checkout skipped, the route unit started at
  `1795503146.404749`, finished at `1795503146.434234`, and the gate verified afterwards.
- With the key unit required and the key file away: a dependency failure, like the provider shape
  (round two's `evidence-route-owned-key-removed.log`).
- With no key requirement and the key file away: a status that names the dataset and the reason
  (round two's `evidence-route-owned-bare.log`):

  ```text
  zfs[838]: cannot mount 'zfsstep19/home/monochromatic-git-checkout': encryption key not loaded
  mount-monochromatic-checkout.service: Main process exited, code=exited, status=1/FAILURE
  mount-monochromatic-checkout.service: Failed with result 'exit-code'
  ```

That third result is the option's only measured advantage, and it exists exactly when the design does
**not** put the key unit's requirement on the route unit. Choice V1 weighs it.

## What the runs show, one arm per log

Every log names the boot's id, uptime and versions at its head.
Guest 1's logs are this round's, on systemd 258; guest 2's are on systemd 262.2.

- `prepare.log`: Fedora 43, systemd `258.11-1.fc43`, kernel `6.17.1-300.fc43`, OpenZFS `2.4.4-1`,
  the pinned tarball's sha256, and the source install's unit list with its
  `zfs-load-key.service -> /dev/null`.
- `fixture.log`: the pool, the datasets, the cache file the import unit reads, the immutable entry,
  and `zfs mount -a` mounting the checkout **and** the dataset under it.
- `units-install.log`: every unit and drop-in as systemd reads them, and the two escaped names.
- `evidence-as-written.log`: the green boot of the unit text as written, with the order gate,
  per-dataset unit, its consumer, the account consumer, sessions.
- `evidence-key-removed.log`: the key unit's `255/EXCEPTION` and every dependent's `dependency`
  failure, with the provider green.
- `evidence-flag-removed.log`: `covered-entry-not-immutable`, exit status two, dependents failed.
- `evidence-no-key-unit.log`: this host's shape, the gate's `checkout-not-the-dataset` refusal with
  the provider green.
- `evidence-bound.log`, `evidence-bound-late.log`: the hung start, the 20 s bound firing, the stop
  pass, sessions after the failure, read at uptime 37.56 s.
- `evidence-learned-unit-only.log`: the learned-unit trap at a boot on 258, both drop-ins off.
- `evidence-v9b.log`: choice V9's option B, the pass mounting the dataset in its own process, and the
  consumer that names a per-dataset unit getting nothing.
- `evidence-hazard-nested-auto.log`, `evidence-hazard-v4.log`: the hazard and its exclusion, re-run
  with SELinux enforcing.
- `restart-provider.log`: the provider restart that moved the gate, the per-dataset unit and both
  consumers, with the stop pass's `stopped-slot=unmounted`.
- `systemd-262-arms.log`: the four bound facts, `RequiresMountsFor=`'s expansion, `Upholds=` and
  `BindsTo=` on a learned unit, and the four-unit restart chain, all on 262.2.
- `upholds-arm.log`: `Upholds=` and `BindsTo=` on a learned mount unit, with `FragmentPath=` empty and
  `SourcePath=/proc/self/mountinfo`.
- `escaping-arm.log`: how a unit file must spell a name that carries escapes.
- `boot-arm-learned-unit.log`: the learned-unit trap at a boot on 262.2.
- `boot-arm-learned-with-provider.log`: the same with the provider also required, both drop-ins absent.
- `mvm-create-guest1.log`, `mvm-destroy-guest1.log`, `mvm-create-guest2.log`,
  `mvm-destroy-guest2.log`: the two guests' own records.

## What the runs do not show

- Not the product handler. Every program is a stand-in; the wrapper's root modes, the readiness
  record, the start probe's own deadline and the unmount half never ran.
- Not this host's kernel or pool for the ZFS arms. Guest 1 runs kernel `6.17.1-300.fc43`, OpenZFS
  `2.4.4-1` and a file-backed pool, and no host dataset was mounted, unmounted, imported, exported or
  changed.
- Not a generated mount unit. No arm made the ZFS mount generator write a unit, because that needs a
  root-written cache file on a machine whose ZED tracks the pool; the branch of the rule that names
  `<escaped checkout path>.mount` as the provider is a source reading plus the host reads, not a run.
- Not the generator's key units as generated: guest 1's two key units were written by hand to the
  generator's shape and names.
- Not ZED. No arm ran `zfs-zed.service`, and the hook is a text this design proposes.
- Not AppArmor. Guest 1 runs Fedora's SELinux, not this host's AppArmor, so the hook's own profile
  question is a reading here, not a run.
- Not this host's key path. Guest 1's checkout is an encryption root of its own with a file key; this
  host's is a passphrase prompted at import with no key unit at all, which the no-key-unit arm models
  but does not reproduce in its prompt.
- Not a failed import, a delayed import, or a delayed key load.
  Plan step 19's acceptance text asks for the delayed cases too, and they remain unmeasured.
- Not a busy shutdown. The stop pass ran with one slot attached and nothing holding it; a busy clone,
  a failed stop and an interrupted boot are step 9's arms. The stand-in's stop pass does not force or
  detach, and reports a busy target, which is the design's requirement.
- Not a shutdown arm this round: the reverse stop order was seen at runtime through the provider
  restart, and the boot-time shutdown order is round one's on systemd 255.
- Not the same kernel as this host on guest 2 either: guest 2 runs systemd 262.2, but no ZFS and no
  kernel-side ZFS paths.
- Not the `mount-path-descendant` mode: it does not exist, and the stand-in that stands in for it is
  idempotent by construction, which is what the design requires of the real one.

## What else on this host touches the checkout's boot

`zfs-space-guard.service` is `After=zfs-mount.service` and its timer runs every five minutes,
deleting old `/home` snapshots when the pool is over 85 percent full (host read in
`keep/host-reads.log`).
It has no edge to the boot unit and this design gives it none; what matters for a reader is that a
service ordered after the provider can run while the checkout is absent, which is another reason a
provider's own success is not a readiness signal.

## The choices this design puts to the owner

Each is separable, each shows a concrete example with the unit lines, the command and what it
printed, and each is ranked by what it delivers for this route's boot, not by effort and not by what
an incumbent does.
After each ranking, "What the review changed" names the earlier ranking it replaces and why.

### V1. Which unit is the provider

#### Option A: the unit that performs the mount

`Requires=zfs-mount.service` and `After=zfs-mount.service zfs-import.target` on this host, or
`Requires=` and `After=` the generated `<escaped checkout path>.mount` where the generator writes one.

```text
$ systemctl --no-ask-password show --property=Id --property=Result --property=ExecMainStatus zfs-mount.service
Id=zfs-mount.service
Result=success
ExecMainStatus=0
```

with the checkout absent, and the same boot's gate refusing (kept log `evidence-no-key-unit.log`,
boot `e229df39-45ba-47c7-badd-9dd21dc2565b`):

```text
SWEEP checkout-row=zfsstep19/home zfs
SWEEP refused=checkout-not-the-dataset 2026-11-24T08:10:03.712Z
```

What it delivers: one actor mounts each dataset once, so there is no second mount of the checkout
path and no property beyond V4's; the accepted lost-event and propagation designs name this unit; and
on a machine with a key-loading unit, the checkout-specific failure is carried by that unit's
requirement (measured, kept log `evidence-key-removed.log`).

What it costs: the provider's own status says nothing about the checkout, so a reader must know which
unit to read, and on a machine with no key-loading unit the only signal is the gate's own refusal,
which names the checkout path but not the dataset.

#### Option B: a route-owned unit that mounts the checkout explicitly

```ini
# /etc/systemd/system/mount-monochromatic-checkout.service
[Unit]
Description=Mount the checkout dataset explicitly
DefaultDependencies=no
Requires=zfs-load-key@<escaped encryption root>.service
After=zfs-load-key@<escaped encryption root>.service
After=zfs-import.target zfs-mount.service
Before=systemd-user-sessions.service
Before=umount.target
Conflicts=umount.target

[Service]
Type=oneshot
RemainAfterExit=yes
ExecStart=/usr/bin/zfs mount <checkout dataset>
ExecStop=/usr/bin/zfs unmount <checkout dataset>

[Install]
WantedBy=multi-user.target
```

with the checkout dataset at `canmount=noauto`, and the variant without the key requirement printed
this when the key was away (round two's `evidence-route-owned-bare.log`):

```text
$ systemctl --no-ask-password show --property=ExecMainStatus mount-monochromatic-checkout.service
ExecMainStatus=1
zfs[838]: cannot mount 'zfsstep19/home/monochromatic-git-checkout': encryption key not loaded
```

What it delivers: a unit whose status and journal entry are the checkout's, and, where the design does
not also require the key unit, a message that names the dataset and the reason.

What it costs: a `canmount=noauto` on the checkout dataset, which is an owner's property change; a
missing property makes the provider mount the checkout and this unit fail with
`filesystem already mounted` (source reading, `zfs_main.c`, the `OP_MOUNT` branch), a red boot for a
state `doctor` could have reported; its `ExecStop=` unmounts the checkout independently of the
provider's stop, which the design's stop ordering does not want; and with the key requirement in
place its failure is the same dependency failure as option A's
(round two's `evidence-route-owned-key-removed.log`).

#### Option C: keep the conditional

A unit whose `Requires=` names whichever provider the machine has.

What it delivers: nothing to settle here.

What it costs: a conditional in a unit file is not a thing systemd does; a reader cannot audit which
prerequisite a given boot had; and step 19's own open item asks this design to settle it.

#### Ranking of V1

Ranking: A > B > C.

A over B: B's whole measured advantage, a status that names the checkout dataset, appears only in the
variant that drops the key unit's requirement, and it costs a property change on the checkout dataset
plus a second mount actor whose own stop unmounts the checkout; A reaches the same fail-closed boot
through the key unit where one exists and through the gate's refusal where none does, both measured.

B over C: B gives one auditable prerequisite with a status of its own, while C leaves the reader to
guess which unit a machine had.

What the review changed: the ranking stands and the A-over-B reason is rewritten again.
Round two's reason ("the key unit's requirement already buys B's checkout-specific failure") did not
account for this host, which has no key unit; the reason now names this host's shape and its measured
signal, and B's status advantage is stated as real but tied to the variant without the key
requirement.

### V2. What the boot unit requires of the provider

#### Option A: `Requires=` plus `After=`

The text this document writes today.

```text
$ systemctl restart zfs-mount.service
$ systemctl --no-ask-password show --property=Id --property=InvocationID \
    reconcile-git-worktree-clone-mounts.service git-worktree-nested-output-mount.service \
    git-account-session-standin.service git-vm-builder-standin.service
Id=reconcile-git-worktree-clone-mounts.service
InvocationID=a845ba3c94244579b023b19ff4d42bb7          (before the restart: c48b243f1a6040a6b8966b9df592b22e)
Id=git-worktree-nested-output-mount.service
InvocationID=c416a0899b184575962dac06b8f85f01          (before: 419f427fa4d641a380c645b86e799a7f)
Id=git-account-session-standin.service
InvocationID=01623935d40241018456f53f3927d3c9          (before: e5a00fd9ea1f46ed9e3f4932d29e59ed)
Id=git-vm-builder-standin.service
InvocationID=a1ec513011d944b68f5baca9c69398b0          (before: ecafad6210754a4fa4f3bbc7806f28e9)
```

(kept log `restart-provider.log`; the same result on systemd 262.2 in `systemd-262-arms.log` arm F),
and the stop pass ran inside it:

```text
SWEEP stopped 2026-11-24T08:08:51.298Z uptime=21.72
SWEEP stopped-slot=unmounted
```

What it delivers: a dependency failure when the provider fails, the pull-in on a machine where
nothing wanted the provider, and restart propagation, which the propagation design's sentence about
"the one job-driven case the unit covers today" describes.

What it costs: a provider restart restarts every consumer and every per-dataset unit, and the stop
pass unmounts every attached clone before the pass re-attaches them, all measured in
`restart-provider.log`; and the re-run the propagation design wants is also what the provider drop-in
already provides, measured, so
the propagation is a second path to the same re-run at the price of the disruption.

#### Option B: `Wants=` plus `After=`

The same lines with `Wants=` in place of `Requires=`. Measured in round two with the drop-in
installed: after `systemctl restart zfs-mount.service` the gate stayed active and the drop-in's
request ran its reload (round two's `wants-control.log`):

```text
1795503693.546036 Reloading reconcile-git-worktree-clone-mounts.service - Reconcile the managed worktree clone mounts...
1795503693.566525 Reloaded reconcile-git-worktree-clone-mounts.service - Reconcile the managed worktree clone mounts.
```

What it delivers: the same re-run after a provider restart, through the drop-in, with no consumer or
descendant restart and no clone unmounted; the pull-in is kept; and a provider failure leaves the gate
to run and refuse with its own reason instead of failing with `dependency`.

What it costs: no restart propagation, so the two accepted designs' sentences describe a mechanism the
installed unit no longer has; and a provider failure no longer fails the boot unit as a dependency,
which a machine that wants that failure to name the provider would miss.

#### Option C: `After=` only

The boot-ordering control of round two, which started after the provider and passed with no edge to it
at all (round two's `evidence-no-provider-edge.log`: `Finished zfs-mount.service` at
`1795503075.003525`, `Starting reconcile-…` at `1795503075.501653`).

What it delivers: the least coupling.

What it costs: nothing pulls the provider in, nothing re-runs the gate after a provider restart, and
the pull-in caveat in "Why each line" applies: with the module not loaded, the provider is a condition skip.

#### Ranking of V2

Ranking: B > A > C.

B over A: both deliver the re-run after a provider restart, one through propagation and one through the
installed drop-in; B delivers it without unmounting every attached clone or restarting every consumer
and descendant, which is measured to be A's price, and B still keeps the pull-in and still fails closed
because the gate refuses on its own.

A over C: A delivers a re-run and a pull-in, and C delivers neither.

What the review changed: the ranking is reversed.
Round two ranked A first on the ground that "two accepted records rely on restart propagation".
They do not: the lost-event design states the propagation as a fact and names its price
(`F17`, "at the price of the stop pass"), and the propagation design states it as "the one job-driven
case the unit covers today" (its "What becomes of `Type=oneshot`" section).
The measured price is larger than either said, because it reaches every consumer and every
descendant, and the drop-in both records install already delivers the re-run.
Adopting B means those two sentences describe a state the design no longer installs; that is an
amendment for their owners, and it is why this design puts A beside B rather than deleting A.

### V3. How the mount units are named in the ordering

#### Option A: `RequiresMountsFor=<checkout path>`

```text
$ systemctl --no-ask-password show --property=Requires --property=After reconcile-git-worktree-clone-mounts.service
Requires=system.slice zfs-mount.service sysinit.target -.mount "zfs-load-key@zfsstep19-home-monochromatic\\x2dgit\\x2dcheckout.service"
After="zfs-load-key@zfsstep19-home-monochromatic\\x2dgit\\x2dcheckout.service" zfs-mount.service -.mount "srv-zfs\\x2dstep\\x2d19\\x2dfixture-home.mount" zfs-import.target systemd-journald.socket basic.target sysinit.target system.slice
```

(kept log `state-as-written.log`), and the same line on the host's own systemd release, over a mount
learned from the mount table (kept log `systemd-262-arms.log` arm G):

```text
Requires=sysinit.target -.mount system.slice
After=systemd-journald.socket basic.target "srv-step19\\x2d262-bind\\x2dtarget.mount" -.mount system.slice sysinit.target
```

What it delivers: every prefix mount unit derived from the path; both provider states covered without
a line per machine; `Requires=` only on units that have a fragment, which is the safe half of the rule
in "Never require a mount unit systemd learned"; one line instead of two.

What it costs: the derived dependency set is a manager report rather than a line a reader can check in
the unit file; a machine whose path prefix differs has a different set; and the checkout's own learned
unit enters the `After=` only once it exists, so the line orders the stop half rather than the first
start.

#### Option B: the two hand-written ordering lines

`After=home.mount` and `After=<escaped checkout path>.mount`.

What it delivers: every name visible in the file, and no expansion to reason about.

What it costs: `home.mount` is not the home dataset's unit name on a machine whose home dataset is
mounted at another path (guest 1's is `srv-zfs\x2dstep\x2d19\x2dfixture-home.mount`), so the line
silently orders nothing; it misses the prefix's other mounts; and the learned unit's name must be
computed per machine.

#### Option C: no mount-unit edge and a prose statement

What it delivers: nothing to keep in step.

What it costs: the stop half loses its only statement, and `systemd-analyze verify` cannot check the
intent.

#### Ranking of V3

Ranking: A > B > C.

A over B: A covers both provider states and every prefix mount with one line, and it cannot put a
`Requires=` on a learned unit, while B's `home.mount` name is measured to be wrong on a machine whose
home dataset sits elsewhere.

B over C: B states the stop half and C states nothing.

What the review changed: nothing in the ranking; the example is new and now shows the measured
expansion on both systemd releases.

### V4. `canmount=noauto` on every dataset under the checkout path

#### Option A: exclude them and mount them after the gate

Measured with the checkout's mount failing and the dataset under the path at `canmount=on`
(hazard arm): the provider mounted it into the exposed underlay and created the folder
(kept log `evidence-hazard-nested-auto.log`):

```text
d /srv/zfs-step-19-fixture/home/account/Monochromatic/package/vm-builder/cli/output
zfsstep19/nosnap/vm-builder-output  .../Monochromatic/package/vm-builder/cli/output  zfs  shared
```

and with `canmount=noauto` (kept log `evidence-hazard-v4.log`):

```text
== the home underlay's inventory
d /srv/zfs-step-19-fixture/home
d /srv/zfs-step-19-fixture/home/account
d /srv/zfs-step-19-fixture/home/account/Monochromatic
zfsstep19/nosnap/vm-builder-output  .../Monochromatic/package/vm-builder/cli/output  noauto  no  available
```

What it delivers: the provider cannot mount a descendant before the checkout is verified; the gated
unit's explicit `zfs mount` refuses with a named message rather than skipping; and it is the only way
the design's "gate every path descendant" sentence can hold.

What it costs: a property change on existing datasets, which is the owner's act; a dataset under the
path that setup misses is mounted by the provider again, so `doctor` must read and report it; and a
dataset under the path is no longer mounted by `zfs mount -a` alone, so anything that expected that
must name it.

Step 20's snapshot coverage and the setup runbook are the two other places this property has to be
written down, and neither is changed by this design.

#### Option B: let the provider mount them and have the gate report them

What it delivers: no property and no extra units.

What it costs: the hazard arm, an account-visible mount of `vm-builder-output` created by the provider
under an exposed home underlay with no checkout mounted over it, measured; and the design's
gated-descendant claim becomes a report rather than a gate.

#### Ranking of V4

Ranking: A > B.

A over B: the hazard arm is the difference, and it is the failure the design exists to refuse.

What the review changed: nothing in the ranking; the hazard arm was re-run with SELinux enforcing and
the same result.

### V5. How the propagation window is closed

#### Option A: the gate's own first act makes the checkout private

`mount --make-rprivate <checkout path>`, then the propagation read reports `private`
(kept log `evidence-as-written.log`: `SWEEP propagation=private`).

What it delivers: one step, in the same pass that verifies, measured in every green boot.

What it costs: a descendant mounted by the provider before the step is a child of a shared parent for
that window, which V4 removes.

#### Option B: order each descendant after the gate's propagation step

What it delivers: the window closed by ordering rather than by a step.

What it costs: the provider cannot be ordered, so only its own scan can be prevented, which is V4.

#### Ranking of V5

Ranking: A > B.

A over B: B has no mechanism the provider obeys, and A is measured and already the design's.

What the review changed: nothing.

### V6. What carries the checkout's readiness to the account

#### Option A: the account's services require the boot unit, named one by one

```ini
# /etc/systemd/system/git-vm-builder-standin.service
[Unit]
Description=A vm-builder service of the account that needs the dataset under the checkout
Requires=git-worktree-nested-output-mount.service
After=git-worktree-nested-output-mount.service
Before=systemd-user-sessions.service

[Service]
Type=oneshot
RemainAfterExit=yes
ExecStart=<the service's own command>
```

and the measured journal of a green boot, where the consumer that requires the per-dataset unit ran
after it and saw the mount (kept log `evidence-as-written.log`,
`SESSION-DESCENDANT mounted=zfsstep19/nosnap/vm-builder-output`):

```text
1795507717.419298 Finished git-worktree-nested-output-mount.service - Mount a dataset whose mountpoint lies inside the verified checkout.
1795507717.425546 Starting git-vm-builder-standin.service - A vm-builder service of the account that needs the dataset under the checkout.
1795507717.445132 Finished git-vm-builder-standin.service - A vm-builder service of the account that needs the dataset under the checkout.
```

What it delivers: a real prerequisite, visible in the journal per service, and independent of login.

What it costs: it protects nothing about login; `boot-reconcile` exits zero when clones fail, so a
clone failure does not reach a consumer through the requirement; the service list is still to be
named; and with V2's option A every consumer restarts when the provider restarts, while with V2's
option B it does not.

#### Option B: gate the account's user manager

A drop-in on `user@<uid>.service` with `Requires=` and `After=` on the boot unit.

What it delivers: everything the account starts is behind the gate.

What it costs: no design here for its ordering against `systemd-user-sessions.service`, or for a
refused boot that would leave the account entirely without a session; and a failed gate would remove
far more than the checkout.

#### Option C: rely on login ordering alone

No consumer requirement; sessions start after the gate's job by the gate's own `Before=`.

What it delivers: nothing to name.

What it costs: measured, sessions start after a failed gate
(kept logs `evidence-key-removed.log`, `evidence-flag-removed.log`, `evidence-no-key-unit.log`,
`evidence-bound.log`), so a session begins with no checkout and no signal.

#### Ranking of V6

Ranking: A > B > C.

A over B: A fails only what needs the checkout and leaves the session usable; B's blast radius is the
whole session and has no design for the ordering it would need.

B over C: C carries nothing, while B at least refuses something.

What the review changed: the ranking stands; the review's measured point remains (a failed gate does
not hold sessions), and the option's cost line now names what a provider restart does to a consumer.

### V7. What `doctor` and setup report

#### Option A: the whole set

This design's answer, in plain words:

- which of the three provider states the machine is in, and the unit that performs the mount;
- whether a key-loading unit exists, and its name;
- every dataset under the checkout path whose `canmount` is not `noauto`;
- the boot unit's start bound, stop bound and their source;
- the provider drop-in and the boot unit's `ExecReload=`, with the note that a copy systemd has not
  reloaded is not seen;
- the propagation service and its drop-in, installed only beside a boot unit that can reload, with the
  escaped name spelled as `systemctl` prints it;
- the hook's name being one ZED's scan accepts, and whether AppArmor's `zed` profile (if loaded) can
  exec the installed file and mount;
- the installed file's owner, mode, ancestors, `--version` and digest;
- whether the mode choice V9's option A needs exists in this build's grammar.

What it delivers: every state a boot depends on is one command away, and each finding prints the
command that repairs it.

What it costs: a long report, and every line must be kept true as the design changes.

#### Option B: the provider state only

One line, `provider: zfs-mount.service`.

What it delivers: a short report.

What it costs: it reads green on a machine with the wrong `canmount`, no `ExecReload=`, a hook name ZED
ignores, or a profile that cannot exec the hook.

#### Ranking of V7

Ranking: A > B.

A over B: the states that make a boot fail are exactly the ones B leaves unread, and this round added
two more of them: the AppArmor grant and the missing mode.

What the review changed: A stands, and its list grew by the AppArmor reading, the stop bound, the
escaped-name spelling and the missing mode.

### V8. What bounds a start, and how long a login can be held

#### Option A: a sourced `TimeoutStartSec=` and `TimeoutStopSec=`, with the ordering before sessions kept

```ini
[Service]
Type=oneshot
RemainAfterExit=yes
TimeoutStartSec=<the measured ceiling of one reconcile>
TimeoutStopSec=<the measured ceiling of one stop pass>
```

What it delivers: the gate's protection is kept and its cost is a stated arithmetic, and a hung pass is
a failed unit rather than a stuck boot.

What it costs: no measurement of one reconcile or one stop pass exists yet, so both values are step
22's or the owner's to supply; a bound that is too tight kills a reconcile mid-attach and then runs the
stop pass; and the worst case is two start bounds plus a stop bound, which is why the stop bound is
part of the option rather than an afterthought.

#### Option B: provisional fixed bounds by the owner's decision

`TimeoutStartSec=5min` and `TimeoutStopSec=1min`, recorded as the owner's ceilings rather than as
measurements.

What it delivers: a bounded hold now, with two numbers to revisit, and the journal records why they
are the values they are.

What it costs: unsourced numbers; the project's rule for shipped limits allows them only with an owner
decision beside them, which is what this option records.

#### Option C: no bound and the ordering line withheld until bounds exist

What it delivers: no unsourced number and no login hold at all.

What it costs: while the bound is missing, nothing makes the checkout ready before a session, which is
the unit's purpose.

#### Option D: no bounds and the ordering kept

The unit with no `TimeoutStartSec=`, whose manager default is `infinity`
(kept log `systemd-262-arms.log` arm A, and round two's `start-bound.log`).

What it delivers: no number to defend, and no reconcile killed mid-attach.

What it costs: a hung reconcile holds every login with no limit, and the stop pass's own default is
90 s on top; a boot nobody can log in to, cleared only by a person.

#### Option E: the ordering line dropped, and the consumers carry the readiness

No `Before=systemd-user-sessions.service`; every consumer that needs the checkout requires the boot
unit (V6's option A), and no login waits.

What it delivers: no login hold at all, whatever the pass does; the readiness reaches exactly the
services that name it.

What it costs: a session can start with the checkout absent, measured as the shape of every failing
arm, so an interactive account sees no checkout until the pass ends; and the boot unit's own
`Before=umount.target` ordering is the only thing left stating the intent.

#### Ranking of V8

Ranking: A > B > E > C > D.

A over B: the same protection with evidence instead of a promise, which is what the project's rule
asks for beside a shipped limit.

B over E: both bound what can go wrong, and B keeps the login protection the unit exists for while E
gives it up.

E over C: E has no unbounded hold and keeps per-consumer readiness; C has neither.

C over D: D ships an unbounded login hold together with the ordering line, the one combination a boot
unit ordered before the sessions must not have.

What the review changed: the ranking is rebuilt around the measured arithmetic.
Round two ranked A first with no value to ship; this revision says B is what ships while the values
are missing, adds E as the option the review asked for, and states the worst case as two start bounds
plus a stop bound, measured on the host's own systemd release.

### V9. Who mounts a dataset under the checkout path

#### Option A: one unit per dataset, after the gate and before sessions, running the handler's mode

```ini
# /etc/systemd/system/git-worktree-nested-output-mount.service
[Unit]
Description=Mount a dataset whose mountpoint lies inside the verified checkout
Requires=reconcile-git-worktree-clone-mounts.service
After=reconcile-git-worktree-clone-mounts.service
Before=systemd-user-sessions.service

[Service]
Type=oneshot
RemainAfterExit=yes
ExecStart=<root-file> <root-words> mount-path-descendant --dataset <dataset>

[Install]
WantedBy=multi-user.target
```

What it delivers, measured (kept log `evidence-as-written.log`): the mount after the verified checkout
and before sessions, its own journal entry, and a consumer that requires exactly it and sees it:

```text
1795507717.363772 Starting git-worktree-nested-output-mount.service ...
1795507717.419298 Finished git-worktree-nested-output-mount.service ...
1795507717.425546 Starting git-vm-builder-standin.service ...
SESSION-DESCENDANT mounted=zfsstep19/nosnap/vm-builder-output
```

What it costs: the mode `mount-path-descendant` does not exist, so this option ships only once the
root-modes design adds it, idempotent on an already-mounted dataset; one unit per dataset, generated
by setup and reported by `doctor`; and each new dataset under the path needs its unit.

#### Option B: the boot unit's own pass mounts them, one explicit `zfs mount` each

The pass mounts each dataset after it verifies the checkout, in the same process, with no per-dataset
unit. Measured this round (kept log `evidence-v9b.log`, boot
`7f58994e-ac75-4789-a678-d16cebe76fe2`):

```text
SWEEP start 2026-11-24T08:10:27.493Z mode=boot-reconcile uptime=6.53
SWEEP checkout-row=zfsstep19/home/monochromatic-git-checkout zfs
SWEEP attached=registered 2026-11-24T08:10:27.526Z
SWEEP descendant-mount start 2026-11-24T08:10:27.536Z
SWEEP descendant-mount exit=0 2026-11-24T08:10:27.551Z
SWEEP descendant-mounted=zfsstep19/nosnap/vm-builder-output 2026-11-24T08:10:27.555Z
```

What it delivers: one unit, one ordering edge, and the mount in the same process as the verification
that authorizes it, with no new mode; a consumer that requires the boot unit runs after the pass, so
every descendant is mounted by the time it starts, which the same boot's journal shows:

```text
1795507827.468523 Starting reconcile-git-worktree-clone-mounts.service ...
1795507827.558124 Finished reconcile-git-worktree-clone-mounts.service ...
1795507827.564004 Starting git-account-session-standin.service ...
1795507827.924776 Starting systemd-user-sessions.service ...
```

What it costs: a consumer cannot require one descendant's readiness, only the whole pass's; and a
consumer that names a per-dataset unit anyway gets nothing, silently, measured in the same boot
(`git-vm-builder-standin.service`, `ActiveState=inactive`, `Result=success`, no journal line); and
the pass's start bound then has to cover every descendant's mount.

#### Option C: one unit per dataset running `zfs mount <dataset>` directly

What it delivers: the shortest unit text.

What it costs: no verification handshake, so a unit that starts early mounts a descendant under an
unverified checkout, and the design has two mounters instead of one.

#### Ranking of V9

Ranking: A > B > C.

A over B: A is the only option where a consumer can require the exact mount it reads, with a
per-dataset entry and failure, which is what a `vm-builder` service needs; B's measured shape leaves
such a consumer with nothing at all, silently.

B over C: B keeps the verification that authorizes the mount; C does not.

What ships while the mode is missing: B.
The mode is one word and one option on the root-modes design's list
(`mount-path-descendant --dataset <dataset>`, idempotent on an already-mounted dataset), and until it
exists the per-dataset unit cannot be installed.

What the review changed: the option set is unchanged and every reason is replaced.
The earlier text justified A with the plan's sentence about path descendants and with a rule that root's
mounts come from the one program; neither exists as stated, and the benefit now rests on the two
measured consumer behaviours.

## What each part waits on

Named so a reader can see which piece is blocked, and on whom.

- The installed file's own name, and whether it is named for the hook or for the modes: the root-modes
  design's choice 3.
- The hook's exact spelling: that design's choice 1, whose constant is
  `history_event-mount_git_worktree_clone_datasets` today.
- The two words `<root-words>` in every unit line: that design's choice 2, whose constant is
  `monochromatic-git-policy-cli zfs-root`.
- Whether a root start under another name runs the wrapper: that design's choice 4.
- The mode `mount-path-descendant`, with `--dataset <dataset>` and idempotence on an already-mounted
  dataset: that design's mode list, and step 19's build.
- An AppArmor grant for the hook's file and for the pass it starts: no design owns it yet; this
  document's AppArmor section reads the profile as it stands.
- `TimeoutStartSec=` and `TimeoutStopSec=` values: a measurement of one reconcile and one stop pass,
  or the owner's ceilings, which is choice V8.
- Whether consumers restart on a provider restart: choice V2.
- Whether the per-dataset unit or the pass mounts a descendant: choice V9, and `mount-path-descendant`.
- V4's property change on existing datasets: the owner's act, and step 20's snapshot coverage and the
  setup runbook must state it too.

## Edits this design suggests

Suggestions only; this round edits neither record.

To plan step 19, "Open items", which reads:

> Two parts the design leaves open block its acceptance arms:
> the checkout provider stays conditional in "One-time move and checkout boot protection", which names "the
> protected `home-user-Monochromatic.mount` if using a generated mount unit, or `zfs-mount.service` where it mounts
> the checkout", and "Checkout boot mount protection" says to prove the failure control and production ordering
> before enabling the gate.

the suggestion is to replace that sentence with the fixed provider, the key unit and a pointer:

> The checkout provider is fixed on a machine of this host's shape as `zfs-mount.service`, whose `zfs mount -a`
> mounts the checkout because the ZFS mount generator writes no unit here; where the generator's cache file exists
> and names the checkout dataset, the provider is the generated `<escaped checkout path>.mount` instead.
> The checkout's key-loading unit is the generator's `zfs-load-key@<escaped encryption root>.service` where one
> exists, and the provider's status is not a readiness signal for the checkout.
> The failure control, the production ordering, the hazard arm and the route-owned alternative are measured in
> `doc/planning/cli-git-zfs-checkout-boot-mount-provider-and-ordering-design.md`, which also fixes the units'
> whole text, their start and stop bounds, the key and provider drop-ins, and the exclusion every dataset mounted
> under the checkout path needs.

To plan step 19, "What it builds", the sentence "The gate that orders every path descendant after a verified
checkout mount" needs the condition this design measured:

> The gate that orders every path descendant after a verified checkout mount, where each such dataset carries
> `canmount=noauto` so that the automatic mount cannot mount it first, and where either a unit per descendant or
> the gate's own pass mounts it after the verification and before `systemd-user-sessions.service`.

To the design's "Boot and shutdown unit", the two bullets that name the provider and the legacy alternative need
the same substitution, and the paragraph beginning "The decided provider must require the actual home and
key-loading providers" needs five additions:

- the learned mount unit is ordered with `RequiresMountsFor=` and never required, with the measured reason;
- the provider is named by the unit that performs the mount, not by the mount unit systemd learns afterwards,
  and its `ConditionPathIsDirectory=` makes a `Requires=` a pull-in rather than a proof of a mount;
- the key unit is the generator's instance unit, named for the encryption root the checkout's dataset needs, and
  it carries no `Before=zfs-mount.service` of its own;
- a machine with neither provider refuses at boot and says so, and a machine with no key-loading unit says that
  the key must be loaded before the provider runs;
- the unit carries a stop bound beside its start bound, because the stop pass runs inside the start's failure
  path.

To the design's "One-time move and checkout boot protection", the paragraph that begins "The account can replace
`~/Monochromatic` while it is unmounted, regardless of the entry's owner" needs the ancestor condition:

> The account can replace an ancestor of the checkout path while the checkout is unmounted, wherever it can write
> that ancestor's parent; the immutable covered entry refuses the entry's own rename, removal and inner writes, so
> the protection holds only while the account cannot write any directory from the entry's parent up to the home
> dataset's root. At `/home/user/Monochromatic` that holds: `/home` is root's, and `/home/user` is the account's
> but its own parent is not. At a deeper account-owned path such as `~/projects/Monochromatic` it does not hold, and
> the entry belongs at a path where the account cannot write those directories.

and the paragraph ending "The probe did not replace an ancestor of the protected entry or exercise unattended boot."
can say what has now been run:

> Item 8's two gaps are measured in the guests of
> `doc/planning/cli-git-zfs-checkout-boot-mount-provider-and-ordering-design.md`:
> an unattended boot with the checkout's mount made to fail wrote nothing to the covered entry and created no
> mountpoint folder there while the exclusion was in place, the same boot without the exclusion created the folders
> in the home underlay and mounted the output dataset into them, and an account-owned ancestor was renamed with a
> file planted at the entry's path while a root-owned ancestor refused it.

To the design's "Checkout boot mount protection" decision, whose last sentence reads "Failed ordinary mount and
unattended boot remain unmeasured.", the measurement now exists and the sentence becomes a pointer to this design's
"The failure controls".

To the design's `ExecStart`, `ExecStopPost` and hook text, the program name follows the one-program decision
(`doc/decision/cli-git-zfs-one-program-only-the-root-modes-join-the-wrapper.md`); this design writes them against
`<root-file>` and `<root-words>` and says so.

To the lost-event design's "The hook, in one form a sibling can cite", one amendment if choice V2's option B is
taken: its sentence that the boot unit's `Requires=` on the provider makes a provider restart re-run the pass
describes a unit this design would no longer install, and the re-run then rests on the same design's drop-in,
which the record already installs.

To the propagation design's "What becomes of `Type=oneshot` with `RemainAfterExit=yes`", the same amendment and
the same reason.

To the root-modes design, one addition to its mode list: `mount-path-descendant --dataset <dataset>`, idempotent on
an already-mounted dataset, and the reason choice V9's option A needs it.

## Facts and what each rests on

- F1. The checkout path has no mount of its own; `/home` is the home dataset, mounted `shared`.
  Basis: host read, `keep/host-reads.log`.
- F2. The ZFS mount generator is installed, `/etc/zfs/zfs-list.cache` is absent, and
  `/run/systemd/generator` holds no ZFS mount unit. Basis: host read.
- F3. `zfs-zed.service` is disabled and inactive. Basis: host read.
- F4. `zfs-mount.service` runs `/usr/bin/zfs mount -a` with `ConditionPathIsDirectory=/sys/module/zfs`,
  `Before=local-fs.target` and `WantedBy=zfs.target`; `zfs-import-cache.service` runs
  `zpool import -c /etc/zfs/zpool.cache -aN $ZPOOL_IMPORT_OPTS`; `ZPOOL_IMPORT_OPTS` is empty; the four
  ZFS units are enabled. Basis: host read.
- F5. `/usr/lib/systemd/system/zfs-load-key.service` is a `/dev/null` symlink owned by
  `zfs-utils 2.4.4-3`, there is no `zfs-load-key@.service` file, and the unit is `masked`. Basis: host read.
- F6. The host's pool key is a passphrase prompted at import, its keystatus is available, and the home dataset
  inherits it. Basis: host read.
- F7. `home.mount` is learned from the mount table; the checkout's own unit name is `not-found`. Basis: host read.
- F8. `/home` is root's `0755`, `/home/user` is the account's `0700`, the checkout path is the account's `0755`.
  Basis: host read.
- F9. `zfs-space-guard.service` is `After=zfs-mount.service` with a five-minute timer. Basis: host read.
- F10. This host runs systemd `262.2-1-arch`. Basis: host read.
- F11. Guest 1 ran systemd `258.11-1.fc43` on kernel `6.17.1-300.fc43`, OpenZFS `2.4.4-1`, SELinux enforcing,
  and its agent ran `unconfined_service_t`. Basis: kept logs `prepare.log`, `evidence-as-written.log`.
- F12. Guest 2 ran systemd `262.2-1.fc46` on kernel `7.4.0-0.rc3...fc46`, SELinux enforcing, its agent
  unconfined after the drop-in was written from the serial console. Basis: kept logs
  `mvm-create-guest2.log`, `boot-arm-learned-unit.log`, and `keep/state-so-far.md`.
- F13. The unit text as this design writes it boots green, with the order gate, per-dataset unit, its consumer,
  the account consumer and sessions. Basis: kept log `evidence-as-written.log`.
- F14. The checkout's key unit failing fails the gate, both consumers and the per-dataset unit with `dependency`,
  and sessions start after. Basis: kept log `evidence-key-removed.log`, journal lines `1795507758.973672` to
  `1795507760.676330`.
- F15. With the checkout unmountable, the provider finishes `Result=success`, `ExecMainStatus=0`, both with a
  failing key unit and with no key unit at all. Basis: kept logs `evidence-key-removed.log`,
  `evidence-no-key-unit.log`.
- F16. With no key-loading unit, the gate refuses `checkout-not-the-dataset` with status two.
  Basis: kept log `evidence-no-key-unit.log`.
- F17. With the flag removed, the gate refuses `covered-entry-not-immutable` with status two and its dependents'
  jobs fail with `dependency`. Basis: kept log `evidence-flag-removed.log`.
- F18. With the flag absent, the checkout's mount failing and the descendant at `canmount=on`, the provider
  creates the mountpoint folder in the home underlay and mounts the descendant there.
  Basis: kept log `evidence-hazard-nested-auto.log`.
- F19. The same with `canmount=noauto` mounts nothing and creates nothing. Basis: kept log `evidence-hazard-v4.log`.
- F20. A 20 s start bound kills a hung pass, records `timeout`, runs the stop pass, and sessions start after the
  failure. Basis: kept logs `evidence-bound.log`, `evidence-bound-late.log`.
- F21. A oneshot's default start timeout is `infinity`. Basis: kept log `systemd-262-arms.log` arm A, and round
  two's `start-bound.log`.
- F22. The start bound is per phase: a 4 s bound killed a 6 s pre-phase at 4033 ms, and a 4 s bound let a 3 s pre
  plus a 3 s start finish at 6037 ms. Basis: kept log `systemd-262-arms.log` arms B1 and B2.
- F23. A reload shares the start bound: a 3 s bound failed a 6 s `ExecReload=` at 3102 ms while the unit stayed
  `Result=success`. Basis: kept log `systemd-262-arms.log` arm C, and `systemd.service(5)`'s
  `TimeoutStartSec=` sentence at v262 (F41).
- F24. The stop pass extends the hold: after a 3 s bound fired, a 5 s `ExecStopPost=` delayed the unit's failure
  and its dependent's start by 5.01 s. Basis: kept log `systemd-262-arms.log` arm D.
- F25. `TimeoutStopSec=` bounds the stop pass: a 2 s value killed a 5 s `ExecStopPost=` after 2.07 s.
  Basis: kept file `keep/host-user-manager-stopsec.log`.
- F26. `RequiresMountsFor=` puts `After=` on every prefix mount unit and `Requires=` only on the fragmented one,
  on 258 and on 262.2. Basis: kept logs `state-as-written.log`, `systemd-262-arms.log` arm G.
- F27. A provider restart restarts the gate, the per-dataset unit and both consumers, and the stop pass unmounts
  the attached slot. Basis: kept log `restart-provider.log`.
- F28. The same propagation reaches a four-unit chain on 262.2. Basis: kept log `systemd-262-arms.log` arm F.
- F29. `Requires=` on the learned mount unit alone, with both drop-ins off, leaves the gate's boot job unrun on
  258: no journal line, `inactive`, `Result=success`. Basis: kept log `evidence-learned-unit-only.log`.
- F30. The same on 262.2 at a boot, with the learned unit created during the transaction: the requiring unit is
  `inactive` with `Result=success` and no journal line. Basis: kept log `boot-arm-learned-unit.log`.
- F30b. Requiring the provider beside the learned unit, with both drop-ins absent, still leaves the
  requiring unit unrun at a boot: the provider ran, the learned unit was active, and the requiring unit was
  `inactive` with `Result=success` and no journal line. Basis: kept log
  `boot-arm-learned-with-provider.log`.
- F31. A consumer that requires a unit the boot does not install gets nothing: `inactive`, `Result=success`, no
  journal line. Basis: kept log `evidence-v9b.log`.
- F32. The pass mounting a descendant in its own process works: `descendant-mount exit=0`, then
  `descendant-mounted=zfsstep19/nosnap/vm-builder-output`. Basis: kept log `evidence-v9b.log`.
- F33. A consumer requiring the per-dataset unit runs after it and sees the mount. Basis: kept log
  `evidence-as-written.log`.
- F34. `Upholds=` and `BindsTo=` work on a mount unit learned from the mount table: the service starts with the
  mount unit's activation and stops with its deactivation. Basis: kept logs `upholds-arm.log`,
  `dependencies-read.log`, `propagation-cycle.log`.
- F35. An escaped unit name must be spelled as `systemctl` prints it; doubling the backslashes names a unit that
  does not exist, and `systemd-analyze verify`'s "not found" line is a verify-time artifact for a learned unit.
  Basis: kept log `escaping-arm.log`.
- F36. ZED starts a zedlet with no arguments and passes its own environment: `execle(path, prog, NULL, env)`.
  Basis: source reading, `zfs-2.4.4` `cmd/zed/zed_exec.c:168` and `zed_event.c`'s environment block.
- F37. `MODE_WORDS` holds five words and none of them names a folder to mount or a dataset to change; no argument
  is the attaching sweep; a single group word is refused as an unknown mode.
  Basis: source reading, `handler_mode_arguments.rs`, `MODE_WORDS` and `parse`.
- F38. An explicit `zfs mount` of an already-mounted dataset prints `filesystem already mounted` and returns 1,
  while `-a` returns 0; `-a` skips a `noauto` dataset and an unavailable key, while the explicit form refuses.
  Basis: source reading, `zfs-2.4.4` `cmd/zfs/zfs_main.c`, `share_mount_one`.
- F39. The generator's key unit carries `DefaultDependencies=no`, `Wants=`/`After=zfs-import.target`,
  `StandardOutput=null`, an `ExecStop` that unloads the key and no `[Install]`; `Before=zfs-mount.service` is on
  the generated mount unit. Basis: source reading, `zfs-mount-generator.c`, the `keyloadunit` block.
- F40. The start probe's deadline is 350 ms with a 231 ms margin over the slowest measured worker phase.
  Basis: source reading, `probe_deadline.rs`, `PROBE_DEADLINE` and `PROBE_DEADLINE_MARGIN`.
- F41. The start timeout also applies to reloads, and `TimeoutStopSec=` bounds the stop phase.
  Basis: source readings, `systemd.service(5)` at v262 (the `TimeoutStartSec=` and `TimeoutStopSec=` entries) and
  `service.c`'s per-phase `service_arm_timer(s, ..., s->timeout_start_usec)`.
- F42. `boot-reconcile` exits zero when clones fail and only a refusal returns an error; `shutdown-unmount` always
  refuses. Basis: source readings, `boot_reconcile_mode.rs:158` to `:164`, `shutdown_unmount_mode.rs`'s `run`.
- F43. The root-modes design's start rule is the two words `monochromatic-git-policy-cli zfs-root` followed by a
  mode word, on any file; under the hook's own name, no argument is the sweep.
  Basis: source reading, `doc/planning/cli-git-zfs-root-modes-in-the-one-wrapper-executable-design.md`,
  its choice 1 option A and `ROOT_GROUP_WORD`.
- F44. Plan line 2569 names no unit per path descendant. Basis: source reading,
  `doc/planning/cli-git-zfs-worktree-implementation-plan.md`.
- F45. The one-program record holds no rule that root's mounts come from the one program; it fixes one executable
  and leaves the modes' reach to a design. Basis: source reading,
  `doc/decision/cli-git-zfs-one-program-only-the-root-modes-join-the-wrapper.md`.
- F46. The lost-event design states the provider restart's propagation as a fact and names its price, and the
  propagation design calls it "the one job-driven case the unit covers today".
  Basis: source readings, `doc/planning/cli-git-zfs-lost-last-event-triggers-design.md`, `F17` and the section
  around it; `doc/planning/cli-git-zfs-private-propagation-after-replacement-mount-design.md`, "What becomes of
  `Type=oneshot` with `RemainAfterExit=yes`" and `F39`.
- F47. The `zed` AppArmor profile grants exec of packaged zedlets only, read of `/etc/zfs/zed.d`, no mount rule,
  and can be amended only from `/etc/apparmor.d/local/zed`, which does not exist.
  Basis: profile reading, `keep/apparmor-zed-reading.log`.
- F48. The built-in `mvm` registry's newest systemd is Fedora 43's 258, and a custom template reaches the
  Rawhide image with systemd 262.2. Basis: source reading, `package/mvm/cli/src/registry.ts`, and kept logs
  `mvm-create-guest2.log`, `systemd-262-arms.log`.

## Sources read

On this host, read-only, 2026-11-24; the whole output is in `keep/host-reads.log`:
`systemctl --version`, `systemctl cat` for `zfs-import-cache.service`, `zfs-mount.service` and
`zfs-space-guard.service`, `zfs-load-key.service`'s symlink and owner through `pacman -Qo`,
`systemctl --no-ask-password is-enabled` and `is-active` for the ZFS units,
`systemctl --no-ask-password show -p Id -p LoadState -p FragmentPath -p SourcePath` for `home.mount`
and the checkout's own unit name, `findmnt` for the checkout path and `/home`,
`zfs get -o name,property,value encryption,keyformat,keylocation,keystatus` for the pool and its home
dataset, `zfs get -H -o property,value canmount`, `grep` of `/etc/default/zfs` for `ZPOOL_IMPORT_OPTS`,
`systemd-escape --suffix=mount --path` for the checkout path, `stat --format` for `/home`, `/home/user`
and the checkout path, `/sys/kernel/security/lsm`, and the AppArmor files
(`/etc/apparmor.d/zed`, `zfs`, `mount-zfs`, the `disable` folder and `local`).

In guest 1, from the sources that shipped with the arms: OpenZFS `zfs-2.4.4`
`etc/systemd/system/zfs-mount.service.in`, `etc/systemd/system-generators/zfs-mount-generator.c`,
`cmd/zfs/zfs_main.c` and `man/man8/zed.8`.
In the re-read's kept copy
(`~/temp/agent/gate-review-zfs-step-19-revision-2026-11-24/work/src/`): systemd v258 and v262
`src/core/{job,mount,service,transaction,unit}.c` and the `systemd.{service,exec,unit}` manuals, and
`zfs-2.4.4` `cmd/zed/{zed_conf,zed_event,zed_exec}.c`.

## Measurements and logs

This round's folder: `~/temp/agent/zfs-step-19-third-revision-2026-11-24/`.

- `keep/guest-scripts/`, every guest step as its own script, so each arm is reproducible:
  `01-prepare.sh`, `02-fixture.sh`, `10-units.sh`, `11-state.sh`, `12-evidence.sh`, `13-reboot.sh`,
  `14-restart-provider.sh`, `20-systemd-262-arms.sh`, `21-boot-arm.sh`, `22-boot-evidence.sh`,
  `23-escaping-arm.sh`, `24-upholds-arm.sh`.
- `keep/logs/`, every step's whole output, named for its arm.
- `keep/host-reads.log`, the host reads with their exact output.
- `keep/apparmor-zed-reading.log`, the AppArmor reading and its limits.
- `keep/host-user-manager-stopsec.log`, the one host user-manager arm, capped with `MemoryMax`,
  `CPUQuota`, `TasksMax`, `IOWeight=1` and `RuntimeMaxSec`, and removed afterwards.
- `keep/state-so-far.md`, the round's running state.
- Two round two logs are transcriptions written after that guest was destroyed, not captured output:
  `guest-load-key-unit.log` and `reload-gate-after.log`, and round two's `prepare.log` lacks the
  tarball's sha256 line.

The guests were created and destroyed through `mvm` at the times in `mvm-create-guest1.log`
(2026-11-24T08:02:35Z), `mvm-destroy-guest1.log` (08:13:06Z), `mvm-create-guest2.log` (08:13:10Z),
`mvm-destroy-guest2.log` (08:17:56Z), `mvm-create-guest2b.log` (08:29:10Z) and
`mvm-destroy-guest2b.log` (08:30:17Z).
`mvm list` no longer reports either; the only guest it reports is another session's.

## Revisions

The first revision was written after the first gate review;
the second after a same-vendor re-read of that revision.
What the re-read changed is "What the re-read changed", finding by finding.
This revision replaces every citation the re-read showed wrong,
boots the unit text as written, measures the bound's three parts on the host's own systemd release,
adds the missing learned-unit control and this host's no-key-unit arm,
names the AppArmor reading the hook waits on,
adds the mode `mount-path-descendant` as a waiting item instead of assuming it,
and reverses choice V2's ranking on the measured cost of restart propagation.

No cross-vendor review of this revision has been received, so none is folded in, and the document
stays unapproved and non-final.
