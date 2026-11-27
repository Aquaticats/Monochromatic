# systemd-tmpfiles 262.2 cannot traverse configured home cleanup trees through shallow AppArmor rules

## Status

A narrow `/home/user/temp` rule allowed the real dry-run to traverse nested directories with zero
permission errors in its collected partial output. The owner then stopped the long-running dry-run,
so no complete-traversal claim is made. A separate `.cargo/build` directory-read probe now passes.
`temp.conf` has migrated to user tmpfiles, the system copy is removed, and create-only user
dry-runs pass. The 812-profile cache rebuild passed. No non-dry cleanup was run.

## Symptom

This read-only command reported permission failures for several directories:

```sh
sudo systemd-tmpfiles --clean --dry-run /etc/tmpfiles.d/temp.conf
```

Representative output was:

```text
Opening directory "/home/user/temp/agent" failed, ignoring: Permission denied
Opening directory "/home/user/temp/fe-run-2026-11-26" failed, ignoring: Permission denied
Opening directory "/home/user/temp/browser-chooser" failed, ignoring: Permission denied
Opening directory "/home/user/temp/zio-results" failed, ignoring: Permission denied
Opening directory "/home/user/temp/ab" failed, ignoring: Permission denied
Opening directory "/home/user/temp/fe-home-2026-11-26" failed, ignoring: Permission denied
```

The audit showed two `open` denials for each of those six directories, all with
`profile="systemd-tmpfiles"`, `fsuid=0`, and `ouid=1000`.

The configured rule is:

```text
e /home/user/temp - - - 24h -
```

The same output listed many `Would remove` candidates. Verification must retain `--dry-run`; a real
clean is outside this repair's authorization.

A later user-instance failure had the same directory-open boundary but a different root:

```text
Failed to open directory /home/user/.cargo/build: Permission denied
```

Its configuration is `e %h/.cargo/build - - - 24h -`. The real
`systemd-tmpfiles-clean.service` failed with status `73/CANTCREAT` at 2026-11-26T18:31:45-05:00.
The clean timer remained enabled, but its previous failure was preserved rather than hidden with
`reset-failed`.

## Root cause

systemd 262.2's `clean_item_instance()` calls `opendir_and_stat()` before cleaning an item in
`src/tmpfiles/tmpfiles.c:3834-3855`:

```c
r = opendir_and_stat(instance, &d, &sx, &mountpoint);
if (r <= 0)
        return r;

return clean_item_instance_from_dir(c, i, instance, d, &sx, mountpoint);
```

Source: [systemd 262.2 `tmpfiles.c`, clean directory open](https://github.com/systemd/systemd/blob/63511d7283d7d0e2a2fc28075481afaf1a7be8f5/src/tmpfiles/tmpfiles.c#L3834-L3855).

`opendir_and_stat()` calls `opendir_nomod(path)` and returns the error when the directory cannot be
opened, in the same file at lines 582-620:

```c
d = opendir_nomod(path);
if (!d) {
        bool ignore = IN_SET(errno, ENOENT, ENOTDIR);
        r = log_full_errno(ignore ? LOG_DEBUG : LOG_ERR,
                           errno, "Failed to open directory %s: %m", path);
        if (!ignore)
                return r;
```

Source: [systemd 262.2 `tmpfiles.c`, directory error path](https://github.com/systemd/systemd/blob/63511d7283d7d0e2a2fc28075481afaf1a7be8f5/src/tmpfiles/tmpfiles.c#L582-L620).

The stock `systemd-tmpfiles` profile grants broad write access to selected top-level locations and
several shallow home patterns:

```apparmor
@{HOME}/ rw,
@{HOME}/* rw,
@{HOME}/*/ rw,
@{HOME}/*/* rw,
```

Those patterns do not recursively authorize directories such as `/home/user/temp/agent/` and the
nested trees that cleanup must traverse. The user `.cargo/build` cleanup has the same `opendir()`
boundary. Each configured cleanup root is a local administrative choice, so the narrow repairs
belong in `/etc/apparmor.d/local/systemd-tmpfiles`.

## Verification

Installed systemd is `262.2-1`. Tag `v262.2` resolves to commit
`63511d7283d7d0e2a2fc28075481afaf1a7be8f5`, whose source was inspected directly.

### Patterns that fail

Before the rule, the real dry-run produced twelve directory-open AVCs across the six named
directories. It nevertheless continued and printed `Would remove` entries for other paths. That
continuation is not proof of complete cleanup coverage: unreadable directories were ignored.

### Patterns that work

After adding and loading the local rule, the same dry-run traversed `/home/user/temp/agent` and many
nested directories. Its log contains detailed `Would remove` and `Would restore` entries beneath
those formerly unreadable paths.

The initial shell window ended after 60 seconds, but the dry-run remained active as PID 580910. It
was not duplicated. The owner later asked to stop it rather than wait for completion. SIGTERM was
sent to that exact PID, the bounded wait was stopped, and the PID was absent afterward.

The collected partial log contained no `Permission denied` or `Opening directory ... failed` record
and had 189803 `Would remove` lines. This proves recursive access during the observed portion, not
that every directory was eventually traversed. No full-log or complete-cleanup claim is made.

For the cargo incident, `aa-exec --profile=systemd-tmpfiles -- ls -ld /home/user/.cargo/build`
succeeded before the rule, but only proved a metadata `stat`. The matching red probe was
`aa-exec --profile=systemd-tmpfiles -- ls -A /home/user/.cargo/build`, which failed to open the
directory. After adding the cargo rule, that same directory read passed without running cleanup.

The user migration installed `e %h/temp - - - 24h -` as
`/home/user/.config/user-tmpfiles.d/temp.conf`. The `%h` specifier means the invoking user's home,
matching the existing cargo configuration and the installed `tmpfiles.d` manual. Both explicit
configuration files and the complete user configuration passed `systemd-tmpfiles --user --create
--dry-run`. Only then was `/etc/tmpfiles.d/temp.conf` removed, preventing duplicate system and user
cleaners for the same tree. The next user clean timer remained scheduled about 23 hours later; no
actual service start or failure reset was performed.

`verified-selected-systemd-tmpfiles-temp-recursion-2026-11-26T18-15-32-0500.json` records the
successful cache verification for the earlier temp-only rule.
`verified-selected-user-tmpfiles-migration-2026-11-26T18-37-50-0500.json` records all 812 expected
artifacts for the cargo and migration state, exact live feature bytes, selection, and reload.
A subsequent parser invocation reported `Cache hit` for the selected `systemd-tmpfiles` artifact.
No new tmpfiles denial appeared after the cargo rule.

## Verified workarounds

### Permit recursion only beneath the configured cleanup root

Create `/etc/apparmor.d/local/systemd-tmpfiles` with:

```apparmor
# The configured temp.conf cleanup recursively traverses only this designated tree.
/home/user/temp/{,**} rw,

# The user cargo-build.conf cleanup traverses only its own build tree.
owner @{HOME}/.cargo/build/{,**} rw,
```

Reload the profile and rerun only the dry-run:

```sh
sudo apparmor_parser --replace --write-cache /etc/apparmor.d/systemd-tmpfiles
sudo systemd-tmpfiles --clean --dry-run /etc/tmpfiles.d/temp.conf
```

The temp-tree rule has no `owner` qualifier because the original system process ran with `fsuid=0`
while the files were owned by UID 1000. The cargo rule uses `owner` because its measured user
process and files both belonged to UID 1000. It grants read and write access throughout the explicitly configured cleanup
tree, including deletion and replacement operations needed by tmpfiles. It does not expand access to
other home directories or change the cleanup policy itself.

The tradeoff is that root's confined tmpfiles process can modify anything beneath
`/home/user/temp`, subject to the existing profile and the supplied tmpfiles configuration. This is
necessary for complete traversal and cleanup of that designated tree. It makes a non-dry
`--clean` operation consequential, so only the dry-run was used for verification.

The pre-change state, an absent local file, is recorded privately at
`/home/user/temp/agent/apparmor-full-system-cachyos-2026-11-17/systemd-tmpfiles-local.before-temp-recursion`
(scratch,
deleted on 2026-11-27;
what this record uses from it is stated here).
The temp-only policy and system configuration before the user migration are retained beside it as
`systemd-tmpfiles-local.before-cargo-and-user-migration` and
`systemd-tmpfiles-temp-clean-2026-11-26/temp.conf.system-before-user-migration`.

### Migrate the temp rule to user tmpfiles

Install `/home/user/.config/user-tmpfiles.d/temp.conf` with:

```text
e %h/temp - - - 24h -
```

Validate with `systemd-tmpfiles --user --create --dry-run` before removing the old
`/etc/tmpfiles.d/temp.conf`. This preserves the original path and age through `%h` while assigning
the work to the enabled user clean timer. Do not start the real service merely to clear its old
failure; the next scheduled run will provide the actual result.

## What does not work

- Treating a zero exit or continued `Would remove` output as proof: the original command ignored
  unreadable directories, so its coverage was incomplete despite continuing.
- Adding an `owner` rule for the original root cleanup: that audited process used `fsuid=0` and the
  files had `ouid=1000`. The separate user cargo probe did have matching user ownership.
- Using `ls -ld` as the cargo acceptance test: it only stats the directory and can succeed while
  `ls -A` and tmpfiles' `opendir()` remain denied.
- Granting recursive access to all of `/home/user`: the configured cleanup root is only
  `/home/user/temp`.
- Rerunning the same dry-run concurrently after a shell timeout: the original process remained
  active and had to be awaited instead.
- Running `--clean` without `--dry-run` as a test: the output demonstrates real deletion candidates,
  and no such mutation is needed to verify AppArmor read and traversal permissions.

## Upstream filing decision

1.  **Is it really upstream's fault?** No new upstream systemd defect is demonstrated. systemd
    performs the expected recursive traversal. The local AppArmor policy did not authorize the
    locally configured cleanup root.
2.  **Can upstream fix it?** systemd needs no code change for this denial. The generic `apparmor.d`
    profile could choose different home-directory defaults, but the exact configured path is local.
3.  **Are they supporting this use case?** systemd explicitly supports cleanup of configured home
    paths. The local profile must match the administrator's chosen cleanup root.
4.  **Would the repo welcome our contribution?** Not evaluated because no external report is
    recommended. No contribution-policy claim is made.
5.  **Will they likely fix it?** Not predicted. A machine-local `temp.conf` path does not justify a
    new upstream systemd request.
6.  **Have we prototyped a compatible fix?** The local rule enabled real recursive dry-run traversal
    without broadening other home directories or performing a clean.

No upstream issue or additive comment is recommended. This record does not authorize or validate a
non-dry cleanup, a different cleanup root, or deletion of any listed candidate.
