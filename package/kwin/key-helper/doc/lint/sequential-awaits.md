# `no-await-in-loop` suppressions in the key helper

## Rule and configuration

`package/config/oxlint/src/rule/correctness.ts` enables `eslint/no-await-in-loop` as a warning,
and this package's lint exits 1 on any warning.

The rule has no options,
and no allow-list,
so a loop that must wait for one step before the next carries one statement-scoped
`oxlint-disable-next-line` comment on the `await` line.

Turning the rule off for this package would loosen a lint rule,
which needs prior approval.

## Where the order is the design

`src/nvim.ts` `sendNvimInput` walks the Neovim listen sockets newest first,
and stops at the first one that accepts a connection.

Each candidate has to answer before the next is tried,
because the point of the walk is that a leftover socket file left by a Neovim that died refuses the connection,
while the live Neovim's socket accepts it.

Connecting to them together would deliver the key sequence twice whenever two Neovims are running,
and it would report a failure only after the live socket had already accepted.

The list is short,
one entry per running Neovim plus one per unclean exit,
and each attempt is a local Unix-socket connect,
so the serial cost is one connect per dead candidate.

`src/evdev.ts` `watchShiftDevices` carries three of them,
because the watcher is one pass at a time.

- `await discover()`:
   a scan must finish before the devices it found are started,
   or a device listed twice in one pass would be started twice.
- `await startMonitor(...)`:
   one monitor at a time,
   so a device that cannot be opened does not abort the others the same scan found.
- `await waitForRescan()`:
   the interval is the design.
   The daemon has no device-added notification,
   so it rescans on a five second cadence;
   scanning again sooner would only repeat sysfs reads over an unchanged device tree.

The same pattern,
with the same reason,
is recorded for the repository move tools,
in `package/repository-move/cli/doc/lint/sequential-awaits.md`.

## Where the wait is for something outside the process

Two test helpers read a fact that another process produces after the call that
started it has returned:
`src/fake-command-fixture.ts` `reportedUntil` reads the daemon's log until a
failure line appears,
and `src/launch.unit.test.ts` `recordedUntil` reads what a stand-in launcher
recorded until it appends the line it writes only after its last successful
output write.

Each read is of the whole fact so far,
so a later read supersedes an earlier one and reading them together would report
the state at one instant for every attempt instead of waiting for the change.
The interval between reads is the design:
both waits are bounded by one deadline,
and reading again sooner would only repeat an unchanged read.
