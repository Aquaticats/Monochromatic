# Give home recursive snapshots, name the exclusions and read the guard's scope

## What this proves

Plan step 20 of the ZFS route
(`doc/planning/cli-git-zfs-worktree-implementation-plan.md`,
"Step 20:
Sanoid coverage,
the exclusions and the guard's report")
gives the home dataset recursive Sanoid coverage,
names each excluded dataset in the one order Sanoid reads an exclusion out of
effect,
marks every exclusion for the other snapshotting tools,
and reports what the space guard can actually prune.

Status:
TODO | DONE

This page is the owner's act.
No step of the route writes `sanoid.conf`,
sets `com.sun:auto-snapshot`,
or changes a property on an existing dataset here.
`doctor` prints each of these lines and runs none of them.

## Why the order and the words matter

Sanoid reads its sections in file order and copies a recursive section's
settings onto every child dataset without checking for a child section of its
own
(`/usr/bin/sanoid`,
the `Config::IniFiles` tie and the recursion pass).
So an exclusion section written before its recursive ancestor is snapshotted
anyway,
and `recursive = zfs` takes one `zfs snapshot -r` that no child can be left out
of.
Both were measured on Sanoid 2.3.0 and reproduced on the 2.2.0 the guest had
(`doc/planning/cli-git-zfs-clones-mounted-under-the-checkout.md`,
"Sanoid's section order,
reproduced").

A value is read as Sanoid reads it:
both ends are trimmed,
the first of a repeated key wins,
and a trailing `# comment` stays part of the value,
so `autosnap = no # keep` is a non-empty string and the dataset is
snapshotted.

## Steps

1.  Read the sections the route generates,
    which are the ones below,
    and paste them into `/etc/sanoid/sanoid.conf`
    after every section they depend on.

    ```ini
    # /etc/sanoid/sanoid.conf (the sections this route decides)
    # Use yes, not zfs. Every exclusion stands after the section of each dataset above it.

    [zpcachyos/ROOT/cos/home]
      use_template = home
      recursive = yes

    [zpcachyos/ROOT/cos/varcache]
      autosnap = no

    [zpcachyos/nosnap]
      autosnap = no

    [zpcachyos/nosnap/cargo-build]
      autosnap = no

    [zpcachyos/nosnap/cargo-registry]
      autosnap = no

    [zpcachyos/nosnap/npm]
      autosnap = no

    [zpcachyos/nosnap/user-cache]
      autosnap = no

    [zpcachyos/nosnap/vm-builder-output]
      autosnap = no

    [zpcachyos/nosnap/libvirt-images]
      autosnap = no

    [zpcachyos/nosnap/git-worktree-probe]
      autosnap = no
    ```

    The probe dataset is the ninth exclusion
    (`doc/decision/cli-git-zfs-start-probe.md`,
    "Amendments made with this record").
    It stands outside the home subtree today,
    and it is named so a later recursive ancestor cannot silently include it.

2.  Set the marker on every exclusion,
    one command per dataset:

    ```sh
    zfs set com.sun:auto-snapshot=false zpcachyos/ROOT/cos/varcache
    zfs set com.sun:auto-snapshot=false zpcachyos/nosnap
    zfs set com.sun:auto-snapshot=false zpcachyos/nosnap/cargo-build
    zfs set com.sun:auto-snapshot=false zpcachyos/nosnap/cargo-registry
    zfs set com.sun:auto-snapshot=false zpcachyos/nosnap/npm
    zfs set com.sun:auto-snapshot=false zpcachyos/nosnap/user-cache
    zfs set com.sun:auto-snapshot=false zpcachyos/nosnap/vm-builder-output
    zfs set com.sun:auto-snapshot=false zpcachyos/nosnap/libvirt-images
    zfs set com.sun:auto-snapshot=false zpcachyos/nosnap/git-worktree-probe
    ```

    The marker does not implement Sanoid's exclusion,
    which reads only the sections above,
    so both are set.

3.  Run `git monochromatic-git-policy-cli doctor` and read the `sanoid`,
    `sanoid-coverage` and `space-guard-scope` lines.
    Each finding names one fact and the concrete correction:
    an INI section to add,
    a value to change,
    a section to move below its ancestor,
    or the `zfs set` above.
    The check answers unknown rather than passing when it cannot read a
    configuration,
    a section's `path` key,
    or the store's place in the tree.

4.  Ask Sanoid to prove the coverage before trusting it.
    A run in simulate mode prints what it would snapshot and changes nothing:

    ```sh
    sanoid --readonly --take-snapshots --verbose
    ```

    Every dataset under `zpcachyos/ROOT/cos/home` appears except the nine
    exclusions.

## What the space guard does and does not free

`zfs-space-guard` prunes `<dataset>@autosnap_*` of its one configured dataset
and does not descend
(`/usr/local/sbin/zfs-space-guard`,
the configured dataset and the destroy line).
The project kept that scope
(`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
"Space-pressure pruning" under "Project decisions"),
so `doctor` reports what it cannot free:

- the hourlies of every dataset under the home dataset,
  which includes the checkout's own dataset and every clone;
- the `git-worktree-clone-origin-` snapshots a living clone pins,
  whose blocks stay for as long as the clone lives;
- every snapshot whose name does not begin `autosnap_`.

A guard that descends,
or one configured for another dataset,
is a finding rather than a pass,
because the first destroys a living worktree's history before its expiry and
the second never prunes the hourlies that fill the pool.

## What this page does not cover

- The retention template's own body.
  The route keeps `use_template = home` and the project's policy for
  checkout underlays,
  clone snapshots and pinned origins is measured in plan step 22.
- The values a run of Sanoid 2.3.0 with a zero hourly retention prunes.
  The guest arm in `package/git-policy/cli/e2e/sanoid-coverage-guest.ts`
  measures them,
  and this page ships no number from it.
