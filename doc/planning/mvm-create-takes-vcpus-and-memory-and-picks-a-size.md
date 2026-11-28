# mvm's create takes a number of vCPUs and an amount of memory, and a cloud backend picks the size

Status:
a proposal,
unapproved and non-final.
Written by the coordinating session on 2026-11-28T05:09:39-05:00.
No gate read has read it yet,
and nothing here is built.
Only the owner declares it accepted.

## Why this exists

The owner,
2026-11-28,
asked how a billed test of the OVHcloud backend may choose its machine:

> My impression is mvm should natively support passing how many vcpu and how many GBs of RAM
> and auto pick the most relevant size if the provider doesn't support all mix and match..

And a few minutes before,
after a run had booted a GPU driver image:

> Please don't pick GPU?

The first sentence says what `mvm` should take.
It does not say what "the most relevant size" is,
so that mechanism is one of the decisions listed here,
and nothing is built before the owner has answered them.

## What mvm does today

Read on `main` at `8200df4622`.

`mvm create` takes `--from`,
`--image`,
`--server-type` and `--location`
(`package/mvm/cli/src/index-parsers-cmds.ts`,
lines 59 to 116).
It takes no number of vCPUs and no amount of memory.

- The `libvirt` backend gives every guest the same two constants,
  `DEFAULT_VCPUS = 4` and `DEFAULT_MEMORY_MIB = 8_192`
  (`package/mvm/cli/src/config.ts`,
  lines 38 and 43,
  written into the domain at `package/mvm/cli/src/domain-xml.ts`,
  lines 247 and 251).
- The `hetzner` backend takes a server type by name,
  from `--server-type` or `MVM_HCLOUD_SERVER_TYPE`,
  and otherwise takes "the cheapest non-deprecated type offered in the target locations"
  (`package/mvm/cli/src/backend/hetzner/server-types.ts`,
  lines 2 to 7).
- The `ovh` backend takes a flavor by name or identifier,
  from `--server-type` or `MVM_OVH_FLAVOR`,
  and otherwise takes the smallest flavor in stock,
  by vCPUs and then by memory
  (`package/mvm/cli/src/backend/ovh/flavors.ts`).
  On the owner's project that was `d2-2`,
  with 1 vCPU and 2 GiB.

So one command gives a guest of 4 vCPUs and 8 GiB on this machine
and a guest of 1 vCPU and 2 GiB on OVHcloud,
and the only way to ask for more on a cloud is to know that provider's names.

A record that bears on guest size,
`doc/planning/mvm-guest-memory-held-for-a-rounds-life.md`,
is shelved by the owner until the crate split
("Shelved until the crate split (owner,
2026-11-26T00:20-05:00)").
It measured a 4 GiB guest on binaries built outside it.
This proposal does not reopen it;
an option for memory would be the way such a guest is asked for.

## What the providers offer

OVHcloud,
region `BHS5`,
read from the owner's project on 2026-11-28 between 04:48 and 04:50 (UTC-05:00) with read-only calls:
182 flavors in seven types.
The smallest Linux flavors in stock were
`d2-2` (1 vCPU,
2 GiB,
disk 25),
`c3-4` and `d2-4` (2 vCPUs,
4 GiB,
disk 50),
`b2-7` and `c2-7` (2 vCPUs,
7 GiB,
disk 50),
and `b3-8` (2 vCPUs,
8 GiB,
disk 50).
No size in stock had 2 vCPUs and 6 GiB,
which is the owner's "doesn't support all mix and match".

A flavor record carries `name`,
`type`,
`osType`,
`vcpus`,
`ram`,
`disk`,
`available`,
`quota`,
`capabilities` and `planCodes`.
None of them marks a GPU flavor:
the `capabilities` are the same four on every type.
The types were
`ovh.d2`,
`ovh.ssd.eg`,
`ovh.ssd.cpu`,
`ovh.ssd.ram`,
`ovh.iops`,
`ovh.raid-nvme.t1` and `ovh.baremetal`.
One image in the catalogue restricts itself to flavor families through its `flavorType`,
whose value was the series
`g1,g2,g3,t1,t2,i1,a100,l4,rtx5000,l40s`:
the leading parts of flavor names.
The record carries no price.

Hetzner's server type records were not read for this proposal.
What fields they carry
(cores,
memory,
a price,
a kind of processor)
is not known here and is named under "What is not known".

## How other tools take a size

Fetched on 2026-11-28.
The links are plain page links:
a link to each quoted sentence,
checked in a browser,
is owed before this is put to the owner.

- Multipass takes the numbers themselves.
  Its `launch` help lists
  `-c, --cpus <cpus>`,
  "Number of CPUs to allocate.
  Minimum:
  1,
  default:
  1.",
  and `-m, --memory <memory>`,
  "Amount of memory to allocate.
  Positive integers,
  in bytes,
  or decimals,
  with K,
  M,
  G suffix.
  Minimum:
  512M,
  default:
  1G."
  (<https://documentation.ubuntu.com/multipass/en/latest/reference/command-line-interface/launch/>).
  It runs its own guests,
  so every mix exists.
- Amazon EC2 calls the cloud case attribute-based instance type selection:
  "you can specify the attributes that an instance must have,
  and Amazon EC2 will identify all the instance types with those attributes",
  with a minimum and a maximum for `VCpuCount` and `MemoryMiB`,
  and "price protection",
  which "prevents your EC2 Fleet or Spot Fleet from using instance types that you would consider too expensive even if they happen to fit the attributes that you specified"
  (<https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/ec2-fleet-attribute-based-instance-type-selection.html>).
- OpenStack's client,
  the family OVHcloud's Public Cloud belongs to,
  filters flavors by floors:
  `--min-disk <min-disk>`,
  "Filters the flavors by a minimum disk space,
  in GiB.",
  and `--min-ram <min-ram>`,
  "Filters the flavors by a minimum RAM,
  in MiB."
  (<https://docs.openstack.org/python-openstackclient/latest/cli/command-objects/flavor.html>).

All three read the asked numbers as what the machine must have at least.
The two cloud ones then leave several sizes that fit,
and EC2 adds a rule to keep an expensive one out.

## The decisions

They are separate:
any answer to one can be combined with any answer to another,
except where a line says otherwise.

### What the two numbers mean

For every option the asked numbers are `--vcpus 2 --memory 6` in `BHS5`.

- **At least.**
  The backend lists the provider's sizes in stock,
  keeps those with at least 2 vCPUs and at least 6 GiB,
  and takes the least of them:
  fewest vCPUs,
  then least memory,
  then the rule under "Which of two equal sizes".
  It stores nothing.
  Result here:
  `b2-7` or `c2-7`,
  both 2 vCPUs and 7 GiB.
  What it gives up:
  the guest can be larger than asked,
  and so cost more than the numbers suggest.
- **Exact,
  or refused.**
  The backend takes only a size with exactly 2 vCPUs and 6 GiB,
  and otherwise refuses,
  naming the nearest sizes that would fit.
  Result here:
  a refusal that names `b2-7` and `c2-7`.
  What it gives up:
  on a provider with fixed mixes most requests are refused,
  which is the case the owner's sentence asks `mvm` to handle itself.
- **Nearest.**
  The backend takes the size whose numbers are closest to the asked ones,
  larger or smaller.
  Result here:
  `b2-7` or `c2-7`,
  one GiB away,
  against `c3-4`,
  two away.
  Asked for 3 vCPUs and 4 GiB it can take `c3-4`,
  with 2 vCPUs,
  where no size with 3 or more is as close.
  What it gives up:
  the guest can be smaller than asked,
  so a build sized to its memory can fail in it.

Ranking:
at least > exact,
or refused > nearest.
At least over exact:
both give a guest the work fits in,
and only the first gives one whenever the provider has a larger size.
Exact over nearest:
a refusal costs a second command,
while a guest that is too small costs a failed run that looks like a defect of the work.

### Which sizes are candidates at all

This is where "Please don't pick GPU?"
is answered for machine sizes.
The image side is already built:
a shorthand such as `ubuntu` never stands for an image OVHcloud tags `application`,
which every GPU driver image carries
(`package/mvm/cli/src/backend/ovh/images.ts`).

- **The families an image reserves are never picked.**
  The backend reads the region's images,
  collects every family named in an image's `flavorType`
  (on the owner's project:
  `g1`,
  `g2`,
  `g3`,
  `t1`,
  `t2`,
  `i1`,
  `a100`,
  `l4`,
  `rtx5000`,
  `l40s`),
  and drops every flavor whose name opens with one of them,
  such as `t1-45`.
  It stores nothing and no name is written into the source.
  What it gives up:
  it holds only where the catalogue carries such an image,
  and it also drops `i1`,
  a storage family that is no GPU.
- **A list in the source.**
  The backend drops the flavors whose family is on a list kept in `mvm`.
  What it gives up:
  the list goes stale when a provider adds a family,
  and it must be taken from the provider's documents,
  which this proposal has not read.
- **No rule.**
  The least size that fits is taken,
  whatever its family.
  A GPU size is then picked only when nothing smaller fits and is in stock.
  What it gives up:
  the guarantee the owner asked for.

A size named with `--server-type` is taken as named under every option,
GPU or not.

Ranking:
the families an image reserves > a list in the source > no rule.
The first over the second:
both keep the reserved hardware out today,
and the first follows the provider's catalogue where the second waits for an edit.
The second over the third:
the third keeps nothing out.
The first two can both be built,
the list covering a region whose catalogue marks nothing.

### Which of two equal sizes

`b2-7` and `c2-7` both have 2 vCPUs and 7 GiB,
and the record carries no price.

- **By price.**
  The backend reads the provider's price for each size that fits and takes the cheapest.
  What it gives up:
  one more read per create,
  and OVHcloud's price call has not been read for this proposal,
  so whether it exists in a usable form is not known.
- **By disk,
  then by name.**
  The smaller disk wins,
  then the name that sorts first.
  What it gives up:
  the name order is arbitrary,
  so the choice between `b2-7` and `c2-7` means nothing.
- **Refuse and list them.**
  The backend prints the sizes that tie and asks for `--server-type`.
  What it gives up:
  the owner's "auto pick".

Ranking:
by price > by disk,
then by name > refuse and list them.
Price over disk and name:
both pick one,
and only the first picks for a reason a person would give.
Disk and name over a refusal:
a refusal hands the choice back.

### How the two options are written

- **`--vcpus <count>` and `--memory <GiB>`.**
  Two options that each take one whole number and may each be given once:
  `mvm create --vcpus 2 --memory 6 dev`.
  Memory is in GiB,
  the unit of the owner's sentence and of a flavor's `ram`.
  What it gives up:
  a guest under 1 GiB cannot be asked for.
- **`--vcpus <count>` and `--memory <size with a suffix>`.**
  Memory is written `6G` or `512M`,
  as Multipass writes it.
  What it gives up:
  a second spelling of the same amount to parse and to document,
  for sizes no cloud flavor read here has.

Ranking:
a whole number of GiB > a size with a suffix.
Both ask for every size a cloud offers;
the second adds guests under 1 GiB,
which only the `libvirt` backend could give and no use on record needs.

### What a create without the two options gets

- **Today's defaults,
  per backend.**
  `libvirt` keeps 4 vCPUs and 8 GiB,
  `ovh` the smallest size in stock,
  `hetzner` the cheapest type.
  What it gives up:
  the same command still gives guests of different sizes on different backends.
- **One default everywhere.**
  A create without the options asks for 4 vCPUs and 8 GiB on every backend,
  which on `BHS5` today is the least size with at least those numbers.
  What it gives up:
  the default cloud guest costs more than the smallest one,
  and a person who wants the smallest must say `--vcpus 1 --memory 1`.
- **No default on a cloud.**
  A cloud create without the options is refused.
  What it gives up:
  the shortest command stops working there.

Ranking:
one default everywhere > today's defaults > no default on a cloud.
One default over today's:
a guest arm written for this machine then runs unchanged on a cloud,
which is what the OVHcloud backend was asked for.
Today's over none:
the shortest command keeps working.
This one is the owner's to weigh,
because it trades money for sameness.

### What does not need a decision

- On `libvirt` the guest gets exactly the asked numbers,
  because it can.
- `--server-type` keeps meaning one size by its provider name.
  Given beside `--vcpus` or `--memory` it is refused,
  because the two can disagree.
- The MCP server's create tool gains the same two inputs.
- Disk size is not part of this:
  a cloud size fixes its disk,
  and `libvirt` has its own default of 20 GiB.

## What is not known

- What Hetzner's server type records carry,
  so whether the same rule can be built there from the same fields.
- Whether OVHcloud's API gives a price per flavor to a key with the rights the owner granted.
- Which sizes the wrapper's guest arms need on a cloud machine.
  They are written for the `libvirt` default of 8 GiB.
- Whether the family test keeps every GPU size out in every region:
  it was read in one region of one project.

## Questions for the owner

1.  What do the two numbers mean:
    at least,
    exact or refused,
    or nearest?
2.  Which sizes are never picked without being named:
    the families an image reserves,
    a list in the source,
    both,
    or none?
3.  Which of two sizes with the same numbers:
    the cheaper,
    the smaller disk and then the name,
    or a refusal that lists them?
4.  How is memory written:
    a whole number of GiB,
    or a size with a suffix?
5.  What does a create without the two options get:
    today's defaults per backend,
    4 vCPUs and 8 GiB everywhere,
    or a refusal on a cloud?
