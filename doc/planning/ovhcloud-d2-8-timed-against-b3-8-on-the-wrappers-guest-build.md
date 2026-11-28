# An OVHcloud shared-core size timed against a size with cores of its own, on the wrapper's guest build

This page is a measurement, unapproved and non-final.
It is one day's, 2026-11-28, and one region's, `BHS5`, the project's only Canadian region.
It answers a question
`doc/planning/mvm-create-takes-vcpus-and-memory-and-picks-a-size.md` left to a run rather than to a person
(its fifth decision, "May the cheapest size be a Discovery one?"):
what a guest round's own work costs in minutes on OVHcloud's cheapest shared-resource size
against a size whose cores are its own.
The owner declares completion;
this page records what was run, what was read, and what the numbers rest on,
and it names what it does not settle.

## The two sizes

Both sizes were made from the catalogue's `Ubuntu 24.04` image
through the same guest driver, with no release code and no driver code changed.

- `d2-8`, the Discovery family, which OVHcloud's own documentation calls "Entry-level instances with shared resources":
  4 vCPUs, 8 GB of memory, 50 GB of disk, whole CA$0.06392 an hour.
- `b3-8`, a general-purpose family whose vCPUs are its own:
  2 vCPUs, 8 GB of memory, 50 GB of disk, whole CA$0.09799 an hour.

The pair differs in two things at once, and the reader should hold both:
the cores are shared on `d2-8` and the machine's own on `b3-8`,
and `d2-8` publishes twice the vCPUs.
So this page measures the two sizes as OVHcloud sells them
and it does not measure sharing alone:
a ratio it reports is the ratio of these two machines,
never the cost of sharing with the vCPU count held equal.
What keeps the build step a comparison of one core against one core is the driver's own build line,
which passes `--jobs 1`.

A guest round is told to name `b3-16` today,
4 vCPUs and 16 GB for whole CA$0.19135 an hour,
which is 2.99 times the `d2-8` hour.

## What was timed

The workload is what a guest round pays for on one driver's own path,
from an empty machine to a finished arm,
and it is the lock driver's own path
(`package/git-policy/cli/e2e/README.md`, "Guest work that runs longer than the agent's safe length",
which lists `e2e/managed-git-lock-guest.ts` and its bounds):

1. the recipe's `create`, which makes the machine and the guest's `/mnt/shared`;
2. the recipe's `prepare`, which masks the image's unattended upgrades, installs the build packages,
   builds OpenZFS 2.4.4 against the guest's own kernel headers, loads the module,
   and installs Rust 1.100.0 and `cargo-nextest` 0.9.152;
3. the driver's own build of the crate's test binary,
   `cargo build --release --test native_binary --locked --features test-only-switches --jobs 1`,
   behind a 6 GiB swap file the driver adds;
4. the driver's arm,
   one `cargo nextest run` of the managed-lock guest test.

The driver's `run` step carries the last two together with the push of the checkout's snapshot
and the making of the guest's file-backed pool;
this page reports the step's whole wall time
and, inside it, the two figures the guest's own programs print:
`cargo`'s `Finished ... in` line and `nextest`'s `Summary [ ... ]` line.

## How each round was made

Three rounds.
In each round one machine of each size was asked for at the same moment,
so whatever the provider's own load was at that hour fell on both,
and each machine was destroyed when its arm had ended.
Six machines in all; two alive at a time.

Every step ran as its own `mise` task of this worktree,
with `MVM_BACKEND=ovh` and `MVM_OVH_FLAVOR` naming the arm's size,
so the driver took its backend from the one place guest drivers read it
(`bin/mvm-guest-backend.ts`) and the five OVHcloud names came from this worktree's encrypted file.
No key, address, machine identifier or project name is written on this page or into the records beside it.

Each step's wall time is this host's clock at the step's own start and end
(`date --iso-8601=seconds` and the millisecond clock around the step),
and the guest's own lines are the second reading of the same work.

## What was recorded from the guest

- `nproc`, `MemTotal` and the guest's `PRETTY_NAME` and `uname`, read after `prepare` and before the build;
- the processor's `steal` field of `/proc/stat`, read by a sampler inside the guest every 5 s across the whole `run`
  step, and the `cpu` line read once before the build and once after it;
- `MemAvailable` from `/proc/meminfo` in the same samples, whose lowest figure inside the build's window is what this
  page reports;
- the birth and modification times of the guest's own `/root/lock-build.log`,
  which is the build's window on the guest's clock,
  because `cargo` creates that file when it starts and its last write is the build's end;
- `cargo`'s own `Finished` line and `nextest`'s own `Summary` line.

The sampler is one `sh` loop that appends `date`'s answer, the `cpu` line, and four `/proc/meminfo` fields to a file
under the guest's `/mnt/shared`, once every 5 s;
it costs the guest three file reads and one `sleep` per interval,
and the identical script ran on both machines of every round.
It was started through `mvm exec` on the machine the driver had made, never by a driver step,
because the driver records none of these facts:
no release code and no driver code was changed for this measurement.

## The rounds

The rounds are recorded in the folder beside this page.
Each round's files are named for the step they hold,
one file per arm per step for the step records,
and one file per arm for the guest's own lines.

## Results

Filled from the records beside this page in the commit that adds them.

## What this does not settle

- Whether a shared-resource size is the one a guest round should name.
  That is the proposal's fifth decision and the coordinating session's to record.
- What a shared vCPU costs with the vCPU count held equal:
  no size whose only difference from `d2-8` is shared against own cores was timed.
- Any size but these two, and any image but `Ubuntu 24.04`.
- Any hour but these rounds' own.
