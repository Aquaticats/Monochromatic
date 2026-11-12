# A commit says the git wrapper's service did not start while systemd reports its socket unit active

Affected:
`monochromatic-git-policy-cli` on Linux with a systemd user manager,
when a worktree's commit service socket file leaves
`$XDG_RUNTIME_DIR/monochromatic-git-policy-cli/` while its socket unit stays loaded.
Measured on this host on 2026-11-11,
after the repository's disk filled and the worktree at
`.claude/worktrees/pi-kimi-findings-never-waits-on-btrfs-commits` was rebuilt.
The wrapper's side is
`package/git-policy/cli/src/native/service/service_instance.rs`,
`service_client_reach_service.rs` and
`package/git-policy/cli/src/native/landing/service_provision.rs`.

## Symptom

`git commit` in such a worktree ends verbatim with:

```text
Starting the commit service...
The commit service did not start, so this commit was not made. Its log shows why: `journalctl --user --unit 'monochromatic-git-policy-cli@*.service'`. Run the commit again after fixing that. Nothing in the repository changed.
```

Measured behaviour of that failure,
which makes it confusing:

- `systemctl --user list-units 'monochromatic-git-policy-cli@<instance>*' --all` reports the
  `.socket` and the `.service` both `active running`.
- `systemctl --user show '<instance>.socket' -p Listen` still prints
  `/run/user/1000/monochromatic-git-policy-cli/<instance>.sock (Stream)`,
  while `ls -la` of that exact path answers `No such file or directory`.
- `git monochromatic-git-policy-cli service status` says
  "The commit service of this worktree does not answer here."
- the service's own journal has no line about the commit at all,
  because nothing reached it.
- the wrapper told the truth:
the branch did not move,
  the merge or the index stayed exactly as it was,
  and the next attempt fails the same way however often it is repeated.

## Root cause, as far as this host proves it

The instance name is the FNV-1a hash of the worktree's real Git directory path
(`service_instance.rs`,
`instance_of`),
so one worktree keeps one socket file name and one unit instance.
Two events put this socket file and its unit out of step:

1. the wrapper's journal records that the service of that worktree stopped on
   2026-11-11T06:31:00 with `exiting reason=worktree-gone`,
   after reporting "the worktree of this commit service is gone
   (/var/mnt/encrypted/repository/Monochromatic/.git/worktrees/<name>),
   so it serves nothing,
   but its socket was not stopped (TimedOut)".
   The worktree's administrative directory was then rebuilt at 12:18,
   which is when the disk filled and the repository recovered.
2. the socket unit then stayed loaded and active,
   with a socket file that was no longer on the disk.
   What removed that file is not proven here:
   the directory still holds socket files dated 2026-11-09 and 2026-11-10,
   so no wholesale wipe of it happened,
   and the only removal this worktree's own history names is the
   `git monochromatic-git-policy-cli doctor` / `service stop-units-of-removed-worktrees`
   pair,
   which is exactly what runs against a worktree whose Git directory has gone.

From then on the wrapper's start request is a no-op:
`systemd` answers that the unit is already `active`,
so nothing re-creates the `ListenStream` file,
and the client's connect fails with `ENOENT`.
A failed connect is the client failure
`ClientFailure::DidNotStart`,
which is why the words are "did not start" while the unit reads `active`.

## Recovery, measured

Restart that worktree's own two units.
Only the instance's own units are touched,
and the file comes back:

```sh
# The instance name is the one in the unit names, e.g. from
# systemctl --user list-units 'monochromatic-git-policy-cli@*'
git monochromatic-git-policy-cli service status
systemctl --user restart \
  'monochromatic-git-policy-cli@<instance>.socket' \
  'monochromatic-git-policy-cli@<instance>.service'
ls -la "$XDG_RUNTIME_DIR/monochromatic-git-policy-cli/<instance>.sock"
git monochromatic-git-policy-cli service status
```

Measured on this host:

- before the restart,
  the `.sock` path answered `No such file or directory`;
- `systemctl --user restart <socket> <service>` printed nothing and exited 0,
  and the socket's journal then read
  `Closed ...` / `Stopping ...` / `Listening on monochromatic-git-policy-cli@<instance>.socket`;
- `ls` then found the socket file,
  mode `srw-------`,
  and
  `git monochromatic-git-policy-cli service status` answered
  "The commit service of this worktree is running and answers.";
- the next `git commit` landed in 9 seconds instead of failing,
  and the one after it in 18 seconds.

Nothing is lost by that restart:
the two attempts that failed had left the branch where it was,
and the staged merge resolutions were still in the index,
so the commit that had failed was repeated as it stood.

## What to check when this happens again

- `systemctl --user show '<instance>.socket' -p Listen` prints the path the wrapper dials;
  if `ls` of that path fails while the unit is `active`,
  this is the state this page is about.
- A unit that names a program an upgrade removed is a different failure with its own page
  (`cli-git-service-unit-names-a-removed-program-status-203-exec.md`);
  there the journal holds `status=203/EXEC`,
  and here it holds nothing about the commit at all.
- A rerelease of the wrapper asks a running service to finish its batch and exit
  (`README.md`,
  the `service status` entry),
  so restarting the units costs at most that hand-over.
