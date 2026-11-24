# Step 19's boot mount provider and ordering: the owner's answers

Status: accepted by the owner on 2026-11-24T08:20:00-05:00, through the question tool.
The design these answers settle is
`doc/planning/cli-git-zfs-checkout-boot-mount-provider-and-ordering-design.md`,
fourth revision merged as `ba7ade182a`.
The choices were put to the owner after a same-vendor read of that revision
(`~/temp/agent/gate-review-zfs-step-19-fourth-revision-2026-11-24/keep/read-of-the-fourth-step-19-revision.md`),
whose corrections were written into the questions.
Every behaviour cited below was measured in throwaway Fedora guests,
whose logs the design names.

## The answers

### V1, V3, V5 and V7, accepted as the design ranks them

The owner was asked to pick each standing answer to accept,
and picked all four:

- "V1: zfs-mount.service is the provider".
  The boot gate requires and starts after `zfs-mount.service`,
  the unit that runs `zfs mount -a` on this machine.
- "V3: RequiresMountsFor=<checkout>".
  The gate carries `RequiresMountsFor=/home/user/Monochromatic`.
- "V5: the gate makes the checkout private first".
  The gate's first act makes the checkout's mount private.
- "V7: doctor prints the whole boot setup".
  `doctor` prints the provider,
  whether a key unit exists and matches the unit's key lines,
  every dataset under the checkout not set `canmount=noauto`,
  the time bounds,
  the drop-ins,
  whether ZED is enabled,
  what an unprivileged AppArmor read shows,
  and the installed file's identity.

### V2, what the gate requires of the provider

Answer:
"Wants= + After= (Recommended)".
On a provider restart the gate stays up and its drop-in re-runs it,
so no attached worktree clone is unmounted.
What the gate does when the provider fails is still unmeasured,
and the accepted lost-event and propagation records,
which describe `Requires=`,
are amended by this answer.

### V4 and V9, nested datasets under the checkout

Answer:
"noauto, zfs-mount unit now, mode later (Recommended)".
On this machine the nested dataset is `zpcachyos/nosnap/vm-builder-output`,
mounted at `/home/user/Monochromatic/package/vm-builder/cli/output`.
It is set to `canmount=noauto`,
which is the owner's act on an existing dataset,
and mounted by a unit that runs `/usr/bin/zfs mount zpcachyos/nosnap/vm-builder-output`
ordered after the gate.
That unit is replaced by the wrapper's own `mount-path-descendant` mode once the mode exists.

### V6, what makes the account's services wait

Answer:
"Each service requires the gate (Recommended)".
Any service that reads the checkout carries `Requires=` and `After=` on the gate,
or on one nested dataset's unit.

### V8, split in two

Whether logins wait for the gate.
Answer:
"Logins proceed (Recommended)".
The "before login" ordering (`Before=systemd-user-sessions.service`) is dropped
from the gate,
the nested-dataset units
and the consumers.

Where the time bounds come from.
Answer:
"5 min start, 1 min stop, then measured (Recommended)".
The units ship `TimeoutStartSec=5min` and `TimeoutStopSec=1min` as the owner's decision,
replaced by measured ceilings when plan step 22 measures the workload.

### The ZED hook's AppArmor permissions

ZED is disabled on this machine,
so nothing waits on this answer until ZED is enabled.
The owner was reminded of the earlier answer,
"No, please do rules for everything instead of a dedicated profile that agents would have to remember to use",
and told that a `px` profile ZED enters by itself asks nothing of agents.
Answer:
"Rules in local/zed".
The hook's needs (exec of the wrapper,
`dac_read_search`,
and the mount rules) are added to `/etc/apparmor.d/local/zed`,
with every ZED zedlet inheriting them,
which the owner accepted as the cost.

## State

The design's fifth revision ran before these answers;
the round that writes them into the design is the next step.
