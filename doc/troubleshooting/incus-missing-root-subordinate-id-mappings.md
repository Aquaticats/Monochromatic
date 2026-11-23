# Incus 7.6 rejects unprivileged containers when root has no subordinate UID/GID mappings

Status: incomplete and unapproved. The cause and the host configuration repair were recorded on
2026-11-23 during a broader, unfinished AppArmor rollout. This is not an upstream-filing artifact.

## Symptom

After a valid image alias was available, instance creation failed immediately:

```text
Error: Failed instance creation: Failed creating instance record: Failed initializing instance: System doesn't have a functional idmap setup
```

The daemon had already logged `Unable to parse system idmap` with `No map found for user` during
initialization. The failing test had a real `images:alpine/3.24` product and a valid local Incus
connection. No privileged-container setting enabled the operation, and no AppArmor AVC explained
this particular error.

Only `user:100000:65536` was present in both `/etc/subuid` and `/etc/subgid`. The daemon was running
as root, and those files had no mapping for root. `newuidmap` and `newgidmap` existed and carried
their expected set-user/group-ID capabilities.

## Root cause

Incus v7.6.0, commit `8f5e6c2235f6424ec59d3dc06f12e344a4a416e2`, reads `/etc/subuid` and
`/etc/subgid` in `shared/idmap/set_load.go`. `NewSetFromSystem` first looks up the supplied
username, falling back to that user's numeric UID (`shared/idmap/set_load.go:171-240`, [selected
source](https://github.com/lxc/incus/blob/8f5e6c2235f6424ec59d3dc06f12e344a4a416e2/shared/idmap/set_load.go#L171-L240)):

```go
entries, err := getFromShadow("/etc/subuid", username)
if err != nil {
        if err != ErrNoUserMap {
                return nil, err
        }

        // Try using the uid instead.
        u, err := user.Lookup(username)
        if err != nil {
                return nil, ErrNoUserMap
        }
        entries, err = getFromShadow("/etc/subuid", u.Uid)
        if err != nil {
                return nil, err
        }
}
```

The daemon initialization in `internal/server/sys/os.go:252-264` requests the map for `root` and
returns a nil map when that lookup fails ([selected
source](https://github.com/lxc/incus/blob/8f5e6c2235f6424ec59d3dc06f12e344a4a416e2/internal/server/sys/os.go#L252-L264)):

```go
idmapset, err := idmap.NewSetFromSystem("root")
if err != nil && !errors.Is(err, idmap.ErrSubidUnsupported) {
        logger.Error("Unable to parse system idmap", logger.Ctx{"err": err})
        return nil
}
```

The container driver's nil-map guard in `internal/server/instance/drivers/driver_lxc.go:539-544`
then emits the observed user-visible error ([selected
source](https://github.com/lxc/incus/blob/8f5e6c2235f6424ec59d3dc06f12e344a4a416e2/internal/server/instance/drivers/driver_lxc.go#L539-L544)):

```go
if d.state.OS.IdmapSet == nil {
        return nil, 0, errors.New("System doesn't have a functional idmap setup")
}
```

The host's only mapping was for `user`, not for root. A root mapping was therefore the missing
input. The eventual `Error: Failed instance creation` was not caused by the earlier piped-stdin
wait, and the nonexistent bare `alpine` alias was another separate input error.

## Verification

The installed package was `incus 7.6.0-1.1`, matching the v7.6.0 source tag cited here. The failing
configuration contained exactly:

```text
# /etc/subuid and /etc/subgid
user:100000:65536
```

The first test with a valid image alias returned `System doesn't have a functional idmap setup`.
Incus source documents the unprivileged-container requirement and the root mapping used in its
machine setup (`doc/installing.md:486-502`, [selected
guide](https://github.com/lxc/incus/blob/8f5e6c2235f6424ec59d3dc06f12e344a4a416e2/doc/installing.md#L486-L502)):

```text
root:1000000:1000000000
```

That range is also nonoverlapping with the existing `100000:65536` range. Incus documentation
explicitly says a partial `/etc/subuid`, `/etc/subgid`, `newuidmap`, or `newgidmap` setup requires
configuration rather than falling back to privileged containers.

The original files were backed up before replacement. The corrected files retained the existing user
mapping and added the same root range to both UID and GID mappings:

```text
user:100000:65536
root:1000000:1000000000
```

After restarting the empty Incus daemon, the same launch succeeded:

```sh
sudo incus launch images:alpine/3.24 apparmor-rollout-check \
  --config limits.cpu=1 --config limits.memory=256MiB < /dev/null
sudo incus info apparmor-rollout-check
```

`Status: RUNNING` was observed. The expanded configuration reported the expected host UID and GID
ranges beginning at `1000000`, with no privileged-container override. A shell inside the instance
wrote and read `/root/apparmor-workload-proof.txt` successfully. The host's stacked process label
ended in `:unconfined (enforce)`, demonstrating the outer enforcing Incus profile; the guest-side
`/proc/self/attr/current` value was simply `unconfined`.

A disposable test instance was then deleted with
`sudo incus delete --force apparmor-rollout-check < /dev/null`.

## Verified workaround

Back up both subordinate-ID files and check every existing row before editing. Preserve existing
mappings, then add `root:1000000:1000000000` to both files. In this host's measured configuration,
the resulting content and mode were:

```text
# /etc/subuid and /etc/subgid, mode 0644, root:root
user:100000:65536
root:1000000:1000000000
```

Use `newuidmap` and `newgidmap` normally. Restart Incus only when no user instance operation needs
the running daemon. This repair keeps the intended unprivileged-container boundary; it does not
select `security.privileged=true` or bypass user namespaces.

The large allocation reserves identifiers for Incus, including future isolated-container use. It
does not, by itself, assign every container a separate range or guarantee inter-container policy.
Changes to `security.idmap.isolated` and `security.idmap.size` are a separate configuration
decision. The disposable instance here used the default Incus mapping plus the observed outer
AppArmor policy.

## What does not work

- Mapping only the interactive user and assuming the root daemon will use that row. Incus explicitly
  looks up `root`; the observed successful user row did not satisfy this lookup.
- Continuing before daemon initialization has a valid ID map. The driver guard turns the nil map
  into a container-initialization failure. Restarting the empty daemon after fixing both files was
  the measured repair.
- Treating the bare `images:alpine` failure as proof of an idmap defect. That alias was not
  advertised; `images:alpine/3.24` was advertised and reached the separate idmap check.
- Claiming guest-side `unconfined` means the outer Incus policy is absent. The host-reported stacked
  label must be inspected as well.

## Upstream filing decision

Nothing should be filed for this host configuration defect. No issue, comment, or upstream patch is
proposed.

1.  The missing mapping belongs to local host initialization, not to AppArmor policy or the image
    server. The observed files match Incus's documented failure precondition.
2.  No upstream repair is needed for this configuration requirement. Adding the root allocation
    repaired the measured host failure without editing Incus.
3.  The use case is supported directly by the cited machine-setup documentation and the
    subordinate-ID reader. The source explicitly treats missing resources as requiring
    configuration.
4.  Direct filing also conflicts with Incus's contribution rules. `CONTRIBUTING.md:21-26` says
    `We do NOT allow direct filing of issues by LLMs` and requires a human to enter the issue form
    ([selected
    policy](https://github.com/lxc/incus/blob/8f5e6c2235f6424ec59d3dc06f12e344a4a416e2/CONTRIBUTING.md#L21-L31)).
    No such filing was requested.
5.  There is no established upstream defect for maintainers to fix. No external issue search is
    being treated as evidence that local configuration is optional.
6.  The local configuration repair was exercised through actual container creation, an inner
    workload, label inspection, and cleanup. It is not a proposed upstream-code prototype.

The `.out-of-scope/` filenames were inspected and searched for `incus`, `subuid`, `subgid`, and
`stdin`. None of these records matched this incident. The document does not turn that absence into a
recommendation to publish a machine-generated report or associate a person with an unwanted public
issue.
