# An agent run of `vim` rewrites the account's `~/.viminfo`

A review round's `vim` runs on 2026-11-27,
all of them headless,
rewrote `~/.viminfo` in the account's home at 2026-11-27T18:09:40-05:00.
The round had run `vim` to see which column it shows for a line the linter was
refusing,
and found the file rewritten afterwards.

Status:
**unapproved and non-final**.
Written by the coordinating session from that round's report,
and from Vim's own installed documentation for the mechanism.

## Why an editor run reaches the account's home

Vim's viminfo file is `$HOME/.viminfo` on Unix,
and the `'viminfo'` option is non-empty by default,
so the file is read when Vim starts and written when it exits.
The file holds what a person's own editing left behind:
the marks,
the search history,
the registers and the yanked text,
and the last files edited.
An agent's run therefore does not fail because it changes nothing;
it replaces that record with its own session's.

None of it needs a window:
a run started headless writes the same file on the same exit.

Basis for the mechanism,
read from this machine's installed Vim 9.2:
`/usr/share/vim/vim92/doc/starting.txt`,
section "VIMINFO FILE NAME",
which names `$HOME/.viminfo` as the default and `-i NONE` as the way to read
and write none;
and `/usr/share/vim/vim92/doc/options.txt` at `'viminfo'`,
which states that a non-empty value is read upon startup and written when
exiting Vim.

## What to do instead

No brief for a round sends an agent into an editor,
and this record is the reason one of them says so.
A round that has to ask Vim a question about a file,
such as which column it shows,

- passes `-i NONE`,
  which reads and writes no viminfo file at all,
  or
- runs Vim with a fixture `HOME` of its own,
  which moves the file into that fixture rather than leaving the account's
  alone.

What cannot be undone by either is a run that already replaced the file:
the account's next `vim` session starts from whatever the agent's session left,
and only the person whose editing it was can say what is missing.

## What this record does not say

Whether the same round's `vim` runs touched any other file in the account's
home was not checked,
and no `vim` was run to reproduce this,
because a round runs no editor.
The time is the one the round saw the file change,
not a time this session measured.
