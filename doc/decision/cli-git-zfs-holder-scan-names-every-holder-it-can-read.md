# cli-git ZFS holder scan: it names every holder it can read, and the wrapper says so

## Status

Answered in the question tool on 2026-11-23,
recorded 2026-11-23T17:06:00-05:00.
Nothing of it is built.
The three questions came from the integration pass of plan step 9's holder scan,
on `feat/git-policy-cli-zfs-step-9-holder-scan-across-mount-namespaces` at `1b574285f3`
(`~/temp/agent/integration-pass-zfs-step-9-holder-scan-across-mount-namespaces-2026-11-22/keep/review.md`,
"For the owner").
(scratch,
deleted on 2026-11-27;
what this record uses from it is stated here).
Each answer is given as the option's label and its own description,
as the project was shown them.

## How the person is told when the scan finds a holder

Asked:
before a removal destroys a worktree's ZFS clone,
root checks whether any other mount namespace still holds it.
When the scan finds a holder and the removal stops,
how is the person told?
The example shown:
the person removes worktree `one`;
a service started after the clone was attached has its own copy of the mount;
the wrapper unmounts its own copy fine,
then,
just before destroying the clone's first snapshot,
the scan names the service's namespace.
As built,
nothing tells the person:
the command times out after 30 s.

The project chose "A:
wrapper names holders (Recommended)":
"Before each destroy,
the wrapper asks root for a fresh scan and prints e.g.
"removal stopped:
namespace mnt:[4026532403] still holds one at .claude/worktrees/one (task 1234);
end it or leave it,
then run the removal again".
Pros:
the scan stays a read,
the refusal stays the wrapper's,
always fresh,
no receipt format change.
Cons:
a new file kind and reader (step 17's work);
one more round trip to root per destroy."

Not chosen:
"B:
unmount answer carries them",
one scan right after the unmount whose answer the unmount's receipt carries;
and "C:
kernel's 'dataset is busy'",
which carries nothing and leaves the scan to `doctor`.

## What "exclude new managed attaches" means for the scan

Asked:
the design says the scan,
"before reporting removal-ready unmount",
"excludes new managed attaches with the standing unmount intent".
The round read that as a rule inside the scan that sets aside any mount of the clone at the worktree's path in any namespace,
which is the pass's high finding:
with the clone held in four namespaces the scan said clear and `zfs destroy` said busy.
The staging design reads it the other way:
the intent stops new attaches,
and the removal refuses if one is present.

The project chose "A:
no set-aside (Recommended)":
"The scan names the unshare task's namespace every time;
it only answers "who holds the dataset".
Pros:
narrowest fix,
nothing hidden.
Cons:
if a managed attach were in progress at scan time it would be named and the removal would refuse
(the sweep's refusal under the intent should make that impossible)."

Not chosen:
"B:
set aside reader's own only";
and "C:
as built".

## Whether the scan reads every thread and every root

Asked:
the pass found holders the scan misses silently:
a thread with its own mount namespace,
a process whose main thread has ended,
and a namespace whose lowest-numbered task is chrooted away from the clone.
Catching them means reading every thread and picking,
per namespace,
a reader whose root is the namespace's root.
The cost shown:
about 7 microseconds per extra task was measured,
so roughly a quarter second at 40,000 threads
(extrapolated,
not measured).

The project chose "A:
read threads and roots (Recommended)":
"Build both;
a namespace with no task at its own root is reported as an incomplete read.
Pros:
stops missing those holders.
Cons:
more reads on machines with many threads;
more code."

Not chosen:
"B:
report what it can't read";
and "C:
document the limits only".

## What follows

The coordinator's reading,
not the project's words.
The holder scan's fix round takes the set-aside out of the scan,
reads every thread's mount namespace and picks a reader at each namespace's root,
and gives each scan its own nonce,
which the wrapper's fresh scan before each destroy needs.
The carrier that brings root's finding to the wrapper,
and the wrapper's refusal that names the holders,
are plan steps 16 and 17's to build.
