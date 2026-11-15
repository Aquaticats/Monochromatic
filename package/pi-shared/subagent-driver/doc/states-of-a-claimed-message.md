# States of a claimed message

A message put into a run's inbox is delivered once or named as not taken:
never lost, never doubled.
Five reviews each found the next place in the life of a claim,
so this file writes that life down once,
and the code in `src/claims-of-an-inbox.ts` and `src/claims-of-a-gone-driver.ts` follows it.

## Rules that hold at every step

- **A step a second caller can be making too is written so that either one finishing it, or both, leaves one result.**
  The link from a claim to a kept name is recognised as already made,
  because the two names are one file and the filesystem says so,
  so a second pass finishes the removal and makes no second copy.
  A folder that is gone when it is read is the other pass's work done,
  not a failure.
- **"The owner is gone" is a question about whether that process can still act.**
  A process the kernel reports as dead and not yet reaped cannot.
- **Two names are one file only when their device and inode are equal exactly**, read as 64-bit values.
  A JavaScript number holds only integers up to 2^53, and two inode numbers one apart above that are one number:
  a comparison that rounds them takes one message for another's publication and removes it.
  Every file-identity comparison in the driver reads both numbers as bigints.

## The states

A message a driver has taken moves through these states.
Every state is readable on disk by a person who knows the names here.

### In the inbox

`<run folder>/inbox/<name>`.
No driver holds it.

### Claimed

`<run folder>/inbox/.claims/<pid>-<start>/<a fresh identifier>/<name>`.
The owner folder is named for the claiming driver,
the identifier folder is created for this one claim and can hold nothing else,
and the message keeps the name it was published under.
A claim says that delivery had begun and never whether Pi took the message.

### Published

The claimed message and a kept name are one file, and on a filesystem that refuses hard links the kept name is a copy of it:
`<run folder>/inbox/.failed-<name>[.<n>]` when Pi refused it while it rested or never answered,
`<run folder>/inbox/.orphaned-<name>[.<n>]` when the driver that claimed it died before it knew whether Pi took it.
The `<n>` counts up over names already taken, so publication never replaces an earlier kept message.

### Kept

Only the kept name is left.
The claim and the folders that held it are gone.

### Delivered

No name of it is in the inbox.
Pi took it into a turn, and the owner removed the claim.

### Set aside under its own name

A message the filesystem cannot hold under a kept prefix (a name near the length limit, or an entry that is a folder)
stays claimed, under its own name, and the run's summary names it in `inboxNotTaken`.
This is the state a message ends in when it cannot be kept.

## The steps between them

### 1. Claim: in the inbox to claimed

The driver makes `.claims/<pid>-<start>`, then `<a fresh identifier>`, then renames `<name>` into it.
The rename is one call, and its destination is a folder this step created and nothing else names,
so it never lands on another claim, including a second publication of the same name.

A kill leaves one of two things:
before the rename, an empty claim folder and the message still in the inbox;
after it, the claimed state.

The next driver to read the inbox delivers a message still in the inbox, or claims it itself.
A recovery pass removes the empty folder of an owner it proves gone, and says nothing about a message it never held.

### 2. Publish: claimed to published

The driver links the claim to the next free kept name,
or, on a filesystem that refuses hard links, makes a copy created exclusively under that name.

A kill between the link and the removal below leaves both names: one file, or a copy where hard links are refused.

The next recovery pass reads the claim and the kept name and finds them one file, so it knows the publication is already
made: it removes the claim and makes no second copy. Where hard links are refused, the kept name is a copy and cannot be told
from the claim by which file it is; there a pass that finds the claim gone knows the publication was made, and a kept name that
holds another file is taken as another message's.

### 3. Remove the claim: published to kept

The driver removes the claim's own name, then the folder that claim lived in, its owner's folder, and `.claims` when each is
empty.

A kill after the unlink leaves the kept name and an empty folder;
a kill after the folder removal leaves only the kept name.

The next recovery pass takes empty folders away.
A folder gone when it is read is the other pass's work done.

### 4. Deliver: claimed to delivered

The owner sends the claimed message to Pi and Pi takes it into a turn; the owner then removes the claim as in step 3.
Until Pi takes it, the message stays claimed and the claim is what says so.

A kill after Pi's turn began and before the claim is removed leaves the claim.
A later pass keeps the message as `.orphaned-<name>` and names it:
a claim does not record that Pi took it,
and sending it again would be a second delivery of one message.

A kill after the claim is removed leaves nothing of it in the inbox.

### 5. Hold and send again: claimed to claimed

Pi refuses a message because it has begun a turn.
That is no answer about the message, so the owner keeps the claim and sends the same claim again once Pi rests,
before any newer message, within the run's deadline.
No second claim is made, and no keep happens while the owner is still going to send it.

## The states of a publication

A publication is kept under `.failed-<name>[.<n>]` or `.orphaned-<name>[.<n>]`,
and a name in the inbox is a publication of a claim when it is that claim's own file,
which is to say the same device and inode, exactly.
The rows below are the kinds of publication and the states one can be in, with what a later driver does in each;
each row names the control that holds it.

- **A link, before the link.** The message is claimed and no kept name is its file.
  A later driver keeps it, under whichever kind its own reason names.
  Control: `keeps a message whose driver is gone under its own name, and names it once in the summary`.
- **A link, linked and not yet unlinked.** The claim and a kept name are one file, under either kind.
  A later driver finds every kept name that is this file, under either kind and whichever number,
  settles on the first of them, and removes the claim: no second name is made.
  Controls: `a kill between publishing a kept message and removing its claim publishes it once` (one kind),
  and `a publication a killed pass made under the other kind of kept name is finished, not made twice` (both kinds).
- **A link, unlinked.** Only the kept name is left.
  A later driver keeps the message and has nothing of the claim to do;
  empty folders of the claim are taken away.
  Control: `takes the empty folders a keep left and keeps the kept message, naming nothing`.
- **A copy, before the copy.** The message is claimed, on a filesystem that refuses hard links.
  A later driver publishes a copy under a kept name of its own.
  Control: `a keep on a filesystem without hard links publishes a claimed message by a copy of its own`.
- **A copy, copied and not yet unlinked.** The claim and a copy of it both stand, and the copy is a file of its own.
  A later driver cannot prove the copy is this claim's file, so it copies again:
  the message then stands under two kept names, each holding the whole of it, and nothing is lost.
  This is the one state where two names are accepted, and it exists only where hard links are refused.
  Control: `a copy a killed pass left under the other kind of kept name cannot be told from this message,
  so it is published again`.
- **A copy, unlinked.** Only the kept name is left, holding a copy.
  A later driver has nothing of the claim to do, and takes the empty folders away.
  Control: the same one as a link that is unlinked.
- **No publication.** A message that cannot be kept as a file (a folder, or a name with no room for the kept prefix)
  stays claimed, under its own name, and the run's summary names it.
  A later driver of a live owner leaves it, and a recovery of a gone owner names it and keeps nothing.
  Controls: `an inbox entry that cannot be read stays under its claim, and the summary names it` (drive),
  and `a message whose name is too long to keep stays under its claim, and the summary names it` (drive).

## Which later pass moves a message on, and how it knows it may

Every driver reads the inbox's claims folder on every look, and once more before it is put away.
For each owner folder it asks whether the process the folder names is gone:

- its process entry has gone: gone;
- its process entry names another start time: the number now belongs to something else, so gone;
- its process entry reports the process dead (a zombie the kernel has not reaped, or a dead one): gone;
- its process entry cannot be read, or names no process: not proved, so the claim stays.

A claim whose owner is not proved gone is left where it is: that driver may still deliver it.
For an owner proved gone, the pass keeps every message it finds:
it reads the owner's folder, each claim folder, and each message,
publishing every claimed message through the same keep every other keep uses,
which recognises a publication an earlier pass already made.
Each kept message is named in the run's summary and on the feed,
and the empty folders are taken away.

Two passes that meet leave the same result as one:
the reads and removals tolerate a folder that has gone,
the keep tolerates a claim that has gone,
and the kept name is never made twice.
