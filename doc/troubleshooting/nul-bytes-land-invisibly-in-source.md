# NUL bytes land invisibly in source, and the usual searches do not find them

FOUND 2026-08-17 in `@monochromatic-dev/module-translation-repair`,
while adding the repeat readings for `#115`.

## What happened

An `Edit` would not apply.
The old text was copied straight out of a `Read` of the same file,
character for character,
and the tool reported no match.

`od` on the line explained it:

```text
0000016   o   w   .   r   u   n   S   e   t   }  \0   $   {   r   o   w
```

The source read `` `${row.runSet} ${row.entryId}` ``.
The byte between the two interpolations was NUL,
not a space.

Six of them were in that package,
across two files,
and one had been committed days earlier
in the key builder for the `#107` relocation rule.

## Nothing was broken, which is the problem

A NUL is a legal string character
and a genuinely good separator for a composite key,
better than a space,
because it cannot occur inside a run set name or an entry id.
Both sides of every comparison built their keys the same way,
so every pairing was correct
and every test passed.

The hazard is that it does not survive being retyped.
The next hand to write the same key with an ordinary space
produces a second builder that agrees with the first about nothing.
That has already happened once in this package,
between a NUL and a space,
and the symptom was a resumption path that silently rebought everything
while every fixture that spelled its own key passed.

## The searches that find nothing

This is the `QRY` failure mode in its purest form.
Both of these report a clean file:

```sh
rg --count-matches --perl-regexp '\x00' ./src
grep --recursive --files-with-matches --perl-regexp '\x00' ./src
```

`rg` treats a file containing NUL as binary
and prints `binary file matches (found "\0" byte around offset N)`
INSTEAD OF the matching lines,
so a pipeline that greps `rg` output for line numbers sees nothing
while `rg` itself has already said the file is full of them.
That one line is the whole warning,
and it is easy to lose in a long lint or search transcript.

These do work:

```sh
# count NUL in one file
tr --delete --complement '\000' < FILE | wc --bytes

# show them in context
od --address-radix=d --format=c FILE | rg '\\0'

# show them per line, as ^@
sed --quiet 'Np' FILE | cat --show-all
```

## Where they come from

An agent writing the file.
The Bash tool refuses a command containing a control character
(`command contains control characters that would be hidden in the approval dialog`),
which is how the mechanism was confirmed:
the same intended space came out as NUL in a Bash command,
in a Write payload,
and in a commit message body.
`Write` and `Edit` have no such guard,
so anything written through them lands as-is.

Nothing in the repo catches it either:
`tsc` is happy,
`oxlint` is happy,
the bundler is happy,
and a `git diff` shows a space-looking gap.

## What to do about it

Do not write a separator as a bare literal between interpolations.
Name it,
write it as an ESCAPE,
and share the one constant:

```ts
// package/translation-repair/probe/src/corpus-run/rendering-audit-settled-row.ts
export const SLOT_SEPARATOR = '\u0000';
```

An escape is visible,
it is greppable,
and it says the separator was chosen rather than typed.
Sharing one constant removes the second builder that could disagree with it,
and building each key through a single named function removes the rest.

When an `Edit` will not apply to text you just read,
suspect the bytes before suspecting the tool.
Run `od` on the line.

## How widespread it was: measured, not guessed

Every tracked text file in the repository was scanned after the fix,
6870 of them across `.ts`,
`.tsx`,
`.js`,
`.mjs`,
`.rs`,
`.md`,
`.toml`,
`.json`,
`.yaml` and `.yml`,
reading each one and counting `String.fromCharCode(0)`:

```text
scanned 6870 tracked text files
no NUL bytes anywhere
```

So this was confined to the two files it was found in
and is now gone.
Run that scan before assuming a new one is isolated,
because the count is the only way to tell an accident from a habit,
and it costs one pass over the tree.

## Fixing a file that already has them

Replace them from a script that never has to contain one,
using `String.fromCharCode(0)`:

```js
const NUL = String.fromCharCode(0,);
const after = before.split(`'${NUL}'`,).join(`'\\u0000'`,);
```

Then re-check with `tr --delete --complement`,
not with `rg`.
