# A `%` in a desktop entry's `Exec` value is a field code, even inside a quoted argument

An autostart entry written by `mise run //package/kwin/key-helper:install` names
the daemon by its checkout path.
The entry worked until the checkout path held a
`%`:
a `%f` there was expanded by the desktop-entry field-code rule,
so the entry
ran a different path than it named and the failure appeared only at login,
as an
autostart entry that silently did nothing.
Issue 3645 records the quoting half of
this value;
the field-code half was still open after the quoting fix landed in
`8ffe166f9a`.

## Symptom

Nothing reports the mismatch.
The entry parses,
`desktop-file-validate` accepts
it,
and the launcher runs whatever the expansion produced:

- Path `/scratch/pa%fth/bin` runs `/scratch/path/bin`:
  `%f` expanded to the file
  the entry was launched with,
  which for an autostart entry is none,
  so the two
  characters became empty.
- Path `/scratch/pa%qth/bin` makes the entry invalid:
  `%q` is not a defined field
  code,
  and the specification says such a command line "must not be processed".

## Root cause

The Desktop Entry Specification's "The Exec key" section states the rule and its
order of operations:

> Field codes consist of the percentage character ("%") followed by an alpha
> character.
> Literal percentage characters must be escaped as %%.

> Implementations must undo quoting before expanding field codes and before
> passing the argument to the executable program.

So the double-quoted argument this entry wraps the shell command in does not
protect a `%` inside it:
quoting is undone first,
and then every single `%` is a
field code.
The only literal form is the doubled one,
and the specification adds
that once expanded,
a replacement "should not be checked for field codes itself"
[desktop-entry-exec][].

`package/kwin/key-helper/src/install.ts` `desktopEntryQuote` now doubles each
percentage character at the same interpolation that escapes the quoting rule,
which is what makes a path holding `%f` run as written.

## Verification

GLib's validator and GLib's launcher were each asked,
on glib2 `2.90.1` under Fedora 45.
Scratch entries live outside this repository,
in
`${HOME}/temp/agent/pi-key-helper`.

- `desktop-file-validate` on an entry whose `Exec` holds `%f` inside the quoted
  argument:
  no output,
  exit 0,
  so the file is valid.
- The same validator on an entry holding `%q`:
  exit 1 with
  `contains an invalid field code "%q"`,
  which names the mechanism from the
  specification's side.
- GLib's own launcher,
  `Gio.Unix.DesktopAppInfo.launch` through pygobject
  `3.58.1`,
  run against each entry with a stand-in daemon that records its own
  path:
  - `Exec=/bin/sh -c "exec '/scratch/pa%fth/bin'"` ran
    `/scratch/path/bin` and failed with
    `/bin/sh: line 1: /scratch/path/bin: No such file or directory`.
  - `Exec=/bin/sh -c "exec '/scratch/pa%%fth/bin'"` ran the stand-in at
    `/scratch/pa%fth/bin`.
- `buildAutostartDesktopEntry` from this repository,
  given a checkout path
  holding a space,
  a single quote and `%f` (`c heck'%fout`),
  produced an entry
  whose `Exec` line is

  ```text
  Exec=/bin/sh -c "exec '/…/c heck'\\''%%fout/key-helper-service' 2>&1 | systemd-cat -t key-helper -p debug"
  ```

  and GLib's launcher ran the daemon at exactly
  `/…/c heck'%fout/key-helper-service`,
  with the shell's `'\''` form intact and
  the `systemd-cat` pipe connected.

The consumer in this repository is not GLib:
Plasma's autostart phase parses
`Exec` with KService,
which was not probed here.
The probe above is the
specification's reference implementation,
and the specification text quoted is
what makes the behavior a requirement rather than one library's reading.

The text fragment on the specification link was loaded in a headless browser
session and scrolled to the quoted section;
a fragment naming text the page does
not hold left the page at the top,
so the check tells the two apart.

## Verified workarounds

- Double every percentage character in the `Exec` value,
  after quoting,
  so the
  expansion yields one literal `%`.
  `desktopEntryQuote` in
  `package/kwin/key-helper/src/install.ts` does this.
  - Tradeoff:
    a reader of the generated entry sees `%%` where the path holds one
    `%`.
    The doubling belongs to the file,
    not to the path,
    and a tool that reads
    the entry without expanding field codes shows it literally.
- Avoid `%` in the path a desktop entry names.
  - Tradeoff:
    it is the checkout's path,
    not something the install chooses,
    and
    a user cannot be asked to rename a folder to keep an autostart entry working.

## What does not work

- Quoting the argument.
  The specification undoes quoting before field-code
  expansion,
  and the probe shows the same expansion inside a double-quoted
  argument as outside one.
- Relying on `desktop-file-validate`.
  It accepts a defined field code inside a
  quoted argument,
  so a valid entry can still run the wrong path.
- Escaping the `%` for the shell instead (`\%`,
  or a single-quoted word).
  Field
  codes are expanded before the shell sees the script,
  so a shell-level escape
  arrives too late and changes the shell's own reading of the word.

## Upstream filing decision

Nothing to file,
and the six constraints are walked here so a future session does
not repeat the audit.

1. Is it really upstream's fault?
   No. The specification states the expansion and
   the doubled literal,
   and GLib implements it.
2. Can upstream fix it?
   Not applicable:
   the behavior is specified,
   not broken.
3. Are they supporting this use case?
   Yes,
   `%%` is the documented way to write a
   literal percentage character in an `Exec` value.
4. Would the repo welcome our contribution?
   Not reached,
   since there is nothing
   to contribute.
   No issue,
   pull request,
   or comment is drafted in this
   repository for this finding.
5. Will they likely fix it?
   Not applicable.
6. Have we prototyped a minimal fix?
   The fix belongs on our side and is in
   `package/kwin/key-helper/src/install.ts`.

[desktop-entry-exec]:
  https://specifications.freedesktop.org/desktop-entry-spec/latest/exec-variables.html#:~:text=Field%20codes%20consist%20of%20the%20percentage%20character
