# The one `no-await-in-loop` suppression in the key helper

## Rule and configuration

`package/config/oxlint/src/rule/correctness.ts` enables `eslint/no-await-in-loop`
as a warning,
and this package's lint exits 1 on any warning.
The rule has no options and no allow-list,
so a loop that must wait for one step before the next carries one
statement-scoped `oxlint-disable-next-line` comment on the `await` line.
Turning the rule off for this package would loosen a lint rule,
which needs prior approval.

## Where the order is the design

`src/nvim.ts` `sendNvimInput` walks the Neovim listen sockets newest first and
stops at the first one that accepts a connection.
Each candidate has to answer before the next is tried,
because the point of the walk is that a leftover socket file left by a Neovim
that died refuses the connection while the live Neovim's socket accepts it.
Connecting to them together would deliver the key sequence twice whenever two
Neovims are running,
and it would report a failure only after the live socket had already accepted.
The list is short (one entry per running Neovim plus one per unclean exit) and
each attempt is a local Unix-socket connect, so the serial cost is one connect
per dead candidate.

The same pattern, with the same reason, is recorded for the repository move
tools in `package/repository-move/cli/doc/lint/sequential-awaits.md`.
