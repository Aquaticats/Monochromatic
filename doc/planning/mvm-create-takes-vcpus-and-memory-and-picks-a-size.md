# mvm's create takes a number of vCPUs and an amount of memory, and a cloud backend picks the size

Status:
a proposal for the owner,
unapproved and non-final.
Three same-vendor gate reads have been made of this file's drafts,
each a Claude model reading a draft a Claude session wrote,
so none is an independent review:
the first reported five High,
eight Medium and four Low findings,
the second,
delivered 2026-11-28,
reported three High,
twelve Medium and five Low,
and the third,
delivered 2026-11-28 and read at commit `60dc28feff`,
reported three High,
ten Medium and four Low.
What the owner has already decided,
what is built today,
and what is open to veto are in "The first screen".
Only the owner declares this accepted.

## The first screen

### What is asked, and which questions are still open

Seven questions are in this file.
Each has its own decision in "The decisions",
and each is stated there without leaning on another.

The numbers are the decisions' own numbers.

1.  **Does a bare number mean at least or exactly?**
    The record's chosen text says a bare number is a floor,
    and the same record restates that pick as open to the owner's veto and not answered.
    So the floor stands,
    as built,
    and the veto stays open.
2.  **Refuse when nothing fits or take the nearest?**
    Open.
    Under a floor this bites only above the largest size in stock,
    which the price page puts at 160 vCPUs and 1,024 GB.
3.  **How is the size and its price seen before money is spent?**
    Half answered.
    The owner chose that the create prints the size and its price before anything is created,
    and the CLI does that on OVHcloud for a sized create only;
    a create with no numbers prints the flavour and its numbers with no price.
    Still open:
    the same line on Hetzner,
    the size in an MCP caller's answer,
    whether the printed price is the whole hour (issue 6666),
    a list command,
    and a preview flag.
4.  **Which size among those that fit?**
    The record's chosen text says the cheapest by OVHcloud's public hourly prices,
    and that is built;
    the same record restates the pick as open to the owner's veto.
    The price the build reads and orders by is the instance row,
    and the whole-hour figures in this file add the public IPv4 line and the Gen 3 local storage line
    that a machine also bills (issue 6666).
5.  **May the cheapest size be a Discovery (`d2`) size?**
    Open,
    and not put as a call on risk.
    No build has run on a shared-vCPU size,
    so nothing is measured on either side;
    a round is timing one build on a `d2-8` and one on a `b3-8`.
    What the owner answers is the money:
    CA$0.06392 against CA$0.09799 an hour on the same ask.
6.  **Which unit does a typed `--memory 8` carry?**
    Taken here as a strict floor in gibibytes,
    and open to the owner's veto.
    It is a spend decision rather than a typing convention.
    Two billed runs measured the catalogue's number:
    a size listed with N holds N × 1,000 MiB,
    so the size called 8 carries 8,000 MiB,
    2.34 percent under 8 GiB.
    The size called 8 is `d2-8` at CA$0.06392 on the workload's ask;
    the strict floor takes the next size up,
    `b3-16`,
    at CA$0.19135.
    The build compares the number straight,
    which is the other option.
7.  **What does a create without the numbers get?**
    Open,
    and today it gets the backend's own default,
    which on OVHcloud is a 1-vCPU,
    2,000 MiB machine that cannot hold the measured build.

### What is recommended, and what it costs

The recommended set:
1 floors,
2 refuse and name the nearest,
3 the create's own line now and a list command after it,
4 the cheapest,
5 keep Discovery sizes as candidates,
6 a strict floor in gibibytes,
which this file takes and the owner may veto,
7 one default everywhere,
at 4 vCPUs and 8 GiB.

For the owner's own ask,
`--vcpus 2 --memory 6`,
the recommended set ends on `d2-8`:
4 vCPUs of which 2 are more than asked,
8,000 MiB of memory of which 2,000 are more than asked,
50 GB of disk,
in `BHS5`,
at **CA$0.06392 per hour whole**
(CA$0.0596 instance row,
CA$0.00432 public IPv4).
With Discovery left out of the candidates the same ask ends on `b3-8` at CA$0.09799 per hour whole,
which is CA$0.03407 more per hour,
or 53.3 percent.

For the workload's own numbers,
`--vcpus 4 --memory 8`,
the answer is one machine per rule of Decision 6,
because 8 GiB is 8,192 MiB and a size listed with 8 holds 8,000 MiB:

- the strict floor this file recommends takes `b3-16` at **CA$0.19135 per hour whole**
  (CA$0.1637 row,
  CA$0.00432 IPv4,
  100 GB of local storage at CA$0.0002333 per GB-hour),
  9.10 times the machine a numberless create makes;
- the provider's label,
  which is what a cloud backend does today,
  takes `d2-8` at **CA$0.06392 per hour whole**,
  3.04 times the same machine.

A create that names neither number makes what the backend makes today:
`d2-2` at CA$0.02102 per hour whole,
whose guest cannot hold the measured build.
The owner's own ask is `d2-8` at CA$0.06392 under either rule,
because 6 needs a 7,000 MiB size or more either way.

### What rests on a measurement or a line read, and what is inferred

Measured,
or read from a producer's own text:

- The catalogue read of 2026-11-28 between 04:48 and 04:50 (UTC-05:00):
  182 flavours in `BHS5`,
  their `type`,
  `ram`,
  `disk`,
  `osType`,
  `quota` and `planCodes`,
  and the ten smallest in stock.
  That raw answer is in no file in this repository,
  and the scratch that held it expires;
  every count and every per-flavour number in this file is that read's own statement.
- The Canada price page,
  fetched 2026-11-28T07:52:43-05:00:
  44 priced VM names in five groups,
  the public IPv4 line,
  the local storage lines,
  and the page's own family texts.
- OVHcloud's blog of 2026-08-25,
  fetched 2026-11-28T07:58:15-05:00:
  which lines became separate on 2026-10-01,
  the local disk each Gen 3 size carries,
  and the percentage each size's bill rose by.
- OVHcloud's key concepts page,
  fetched 2026-11-28T07:52:42-05:00:
  the seven family names and what each column says,
  including Discovery's.
- OVHcloud's guide to the OpenStack client,
  fetched 2026-11-28T07:58:15-05:00:
  its captured `openstack flavor list` prints `4000` in the RAM column of `d2-4`.
- Source at `f1d6eccf93`,
  read in this worktree:
  what each command does with a number,
  the family rule,
  the price read,
  the clone's size,
  the one-shot `run` handing both numbers to its create (merge `95f65ed945`),
  and the wording the built help and README carry.
- The workload figures in "The workload on record",
  which are `/proc/meminfo` readings from guests on this machine.
- The two billed runs of 2026-11-28 and what a size's memory number comes to inside the guest,
  recorded in `doc/decision/mvm-gains-an-ovhcloud-backend-billed-by-the-second.md`
  at merges `95f65ed945` and `b6d8e0020c`.
- The first arm of a wrapper driver on OVHcloud,
  recorded in the same file at merge `034df18341`:
  create 75 s,
  prepare 7 minutes 5 seconds,
  the crate's build 9 minutes 40 seconds,
  the arm 2 minutes 3 seconds,
  destroy 35 s.
- The five other tools' command references,
  each fetched again 2026-11-28T08:00:39-05:00.

Inferred,
and marked as such where it appears:

- the guest of a cloud size other than `d2-4`:
  that size is measured,
  the rest follow its rule,
  and the split that turns a domain into a guest is fitted to two measured guests;
- that OVHcloud's own catalogue prices an IPv4 addon and a local storage addon under the plan codes the price page shows (the price page sells them;
  the catalogue's addon list was not read for them);
- that shared vCPUs slow this repository's build (no build has run on a shared-vCPU size);
- what an OVHcloud clone of a larger source does (predicted from source,
  never run);
- any Hetzner price (no Hetzner API call was made);
- what a guest arm costs on the sizes a create picks,
  since the one that has run named `b3-16`.

### What the owner should hear: the picks that stand with the veto open

The record's sections and this file's revision set several behaviours that no question above asks about.
Each one is the project's own pick until the owner vetoes it,
and a yes to a question above does not decide it:

- The family list the sized walk keeps (`b2`,
  `b3`,
  `c2`,
  `c3`,
  `d2`,
  `r2`,
  `r3`).
  It stores seven family names in `flavor-families.ts` and refuses every other family,
  which is what keeps a GPU or storage family out;
  it gives up an OVHcloud family added later until the list is edited,
  and it guards only the sized walk,
  so a numberless create and a clone are not yet guarded.
- A size named with `--server-type`,
  `MVM_OVH_FLAVOR` or `MVM_HCLOUD_SERVER_TYPE` is refused beside either number.
  It stores nothing;
  it ends a create that naming the flavour alone would have run,
  and the alternative is that the command line wins and the create logs the variable it ignored.
- Either number beside `--from` is refused,
  with a reason that is not yet true:
  a clone does not keep its source's size today.
  It gives up clone-then-enlarge,
  which nothing on record has run.
- The typed memory is compared straight with the catalogue's number,
  which is Decision 6's second option.
  It gives up 2.34 percent of the number asked on every power of two,
  and it is the only thing a veto of Decision 6's first option would keep.
- One price read per create,
  before the region loop.
  It gives up a per-region price,
  and it reads the instance row only,
  so the printed figure is low by 0.7 to 25.9 percent of the machine's hour (issue 6666).
- Hetzner takes the smallest numbers that fit,
  and no question here asks it.
  It gives up the cheapest-by-price order the owner chose for OVHcloud,
  and it can end on an Arm type,
  because the sized walk has no architecture filter.
- The record's chosen text gives up the `-flex` twins as sizes with no published price.
  OVHcloud's own blog prices them lower than their plain sizes,
  the Canadian page lists none,
  and whether the Canadian catalogue prices them is not yet known.
- Every create line calls the catalogue's number "GiB" when it holds 1,000 MiB per unit.
  That is a defect against the measured unit,
  filed as such,
  and no decision waits on it.

### What a yes covers, and what it leaves undecided

- A yes to floors covers the meaning of a bare number.
  It leaves the fallback above the largest size,
  the choice among fits,
  the typed unit,
  and whether a suffix spells the other reading.
- A yes to "refuse and name the nearest" covers what a create does when nothing fits,
  and changes a message the build already prints in another shape.
  It leaves what counts as nearest if the measure changes,
  and whether the refusal prints whole prices.
- A yes to the cheapest covers the order among fitting sizes.
  It leaves whether Discovery sizes are candidates,
  whether the printed figure is the whole hour,
  which order Hetzner takes,
  and which last tie-break a real flavour list gives.
- A yes to keeping Discovery covers whether a shared-resource size may be picked.
  It leaves whether the printed line says so,
  and what the round now timing a build on one reports.
- A yes to a strict floor in gibibytes covers what a typed memory asks for.
  It changes the cloud comparison,
  which is the built one today,
  and leaves the suffix grammar,
  what the printed size names,
  and whether Hetzner's documented GB is read the same way.
- A yes to one default covers what a numberless create does.
  It leaves the number itself,
  what a numberless `run` does,
  and whether the guest rounds' own commands start passing the numbers.
- A yes to the create's line covers that the pick and its price are printed before the create.
  It leaves a list command,
  a preview flag,
  and the MCP answer's text.
- A yes to any of the picks above also sweeps in the picks that stand with the veto open,
  which the section before this one lists.

### What is built meanwhile, and how it differs from this file

The record `doc/decision/mvm-gains-an-ovhcloud-backend-billed-by-the-second.md` says the build of `--vcpus` and `--memory` started on 2026-11-28 "as a one-off,
with the session's picks marked for veto",
and that this file "no longer stands in front of that build".
Merge `b7f9f7ed2c` (2026-11-28T07:45:04-05:00) carries it:
`mvm create` and `mvm run` take `--vcpus` and `--memory` on all three backends,
the libvirt backend allocates exactly,
OVHcloud reads one price per create and takes the cheapest size that fits,
and Hetzner takes the smallest shared-vCPU type that fits.
Merge `95f65ed945` (2026-11-28T08:23:52-05:00) then made the one-shot `run` hand both numbers
to its create on every backend
and closed issue 6664.
Three differences from this file remain,
and each is written into the decision it touches:
the memory number is compared straight with the catalogue's number on both cloud backends (Decision 6's second option),
the family rule guards only the sized walk ("The rule that keeps GPU flavours out"),
and the create's line prints on the sized walk only (Decision 3).
"What is built today" states each backend's rule,
and where the build departs from this file each departure is either written into the decision it touches or put as a question the owner may answer against.

## What the owner decided on 2026-11-28

The record's section "The size taken among those that fit,
2026-11-28" holds the owner's answer to which size is taken,
chosen as "Cheapest that fits (Recommended)":

> at create time mvm reads OVHcloud's public hourly prices,
> keeps the sizes whose vCPUs and memory are both at least the ask,
> and takes the lowest-priced one,
> printing the size and its price before anything is created;

and it names the example the owner chose with it:

> example:
> `--vcpus 2 --memory 6` ends on `d2-8` at CA$0.0596 per hour;

The figure in that sentence is the instance row.
The whole hour of the same machine is CA$0.06392,
and "What an hour of a machine costs" gives the two lines the record's figure leaves out.
The record also says what the answer gives up:

> it gives up the sizes with no published price (the `-flex` twins),
> which are never picked;

The same section restates two of the session's own picks,
each marked open to the owner's veto and neither answered:

> the numbers mean at least;
> and no GPU size is ever taken,
> built as a list of the five general families,
> so that a GPU family OVHcloud adds later cannot be picked.

That second pick holds today for a create that carries `--vcpus` or `--memory`,
which is the one the owner's sentence was about;
a create with neither number and an OVHcloud clone reach a different function with no family test,
and "The rule that keeps GPU flavours out" states what each path does and that a round is closing the gap.

The same record's section "A standing yes for billed runs,
2026-11-28" holds the bounds every billed run works inside:
machines named `mvm-*` in the one project,
in Canada,
hourly billing and never monthly,
each deleted by the run that made it,
at most six alive at once,
and a listing and cleanup at the end of a round.

This file takes those as settled and does not ask them again.
It asks the owner only about what those sentences left open,
and about the facts the build had to settle a way.

## What is built today

Read at `f1d6eccf93` in this worktree,
with `package/mvm/cli/src` and `package/mcp/mvm/src` as they stand there,
and merge `95f65ed945` is where the one-shot `run` started handing both numbers on.
Where a path is given,
the sentence was checked against that file.

### `mvm create NAME --vcpus N --memory M`

Both options exist on all three backends (`package/mvm/cli/src/index-parsers-cmds.ts`),
each works on its own,
and neither carries a default when the other is absent.
The built help text now says what each backend does with them:
a libvirt guest gets exactly as many,
Hetzner takes the smallest server type with at least as many,
and OVHcloud the cheapest size that fits,
or its smallest numbers when no hourly price can be read.
Issue 6664 recorded the older wording and is closed.

- **libvirt**:
  the guest gets exactly the numbers asked,
  as a domain of `N` vCPUs and `M` × 1024 MiB (`package/mvm/cli/src/create.ts`,
  `package/mvm/cli/src/domain-xml.ts`).
  A memory ask above the three quarters cap on `mvm-libvirt-session.service` is **not**
  refused:
  the domain is accepted and the guest starts,
  and the README says so where a person meets it.
- **OVHcloud**:
  among in-stock flavours of a kept family,
  the ones whose `vcpus` and `ram` are both at least the ask,
  priced by the public order catalogue,
  cheapest first,
  and the first that boots a matching image is created (`package/mvm/cli/src/backend/ovh/provisioning.ts`,
  `package/mvm/cli/src/backend/ovh/flavors.ts`).
  The price is read **once per create**,
  before the region loop,
  and a create that cannot read it falls back to the numbers alone and says so (`package/mvm/cli/src/backend/ovh/read-price-choice.ts`).
  A size the catalogue publishes no hourly price for is never a candidate.
  The line naming the size,
  its numbers and its hourly price is logged on the sized walk only (`provisioning.ts`,
  `chooseFlavorAndImage`);
  a create with neither number logs the flavour and its numbers with no price,
  and reads the prices first anyway.
  The typed number is compared with `ram` as it stands (`flavor.ram >= request.memoryGiB`),
  with no conversion,
  which is Decision 6's second option.
- **Hetzner**:
  among non-deprecated types offered in a requested location and sold with `cpu_type` `shared`,
  the ones whose `cores` and `memory` are both at least the ask,
  and the **smallest**
  wins,
  by cores,
  then memory,
  then disk,
  then name (`package/mvm/cli/src/backend/hetzner/server-types.ts`).
  Five things about that rule,
  as the code and Hetzner's own documentation state them:
  it has no architecture filter,
  because the file's own opening says "Any architecture is eligible",
  so a numbered ask can end on an Arm type,
  and a type whose name sorts first wins a tie on cores,
  memory and disk;
  Hetzner documents `memory` as "Memory a Server of this type will have in GB",
  compared straight with the typed number,
  so Decision 6's unit question is OVHcloud's and this one is unread;
  the `deprecation` field the build reads for "not deprecated" is itself marked deprecated by Hetzner,
  which points at `locations[].deprecation`,
  and the build reads neither that nor `locations[].available`;
  the walk reads no price at all,
  and only the numberless cheapest-type walk reads `price_hourly.gross`;
  and Hetzner's prices exclude public IP addresses,
  a Primary IPv4 costs EUR 0.50 a month,
  and the system creates one for a server with a public address,
  so the gap issue 6666 records on OVHcloud's line would recur on any Hetzner price printed.

Two numbers asked for one machine,
with one source in the environment,
is refused rather than resolved:
a `--server-type`,
`MVM_OVH_FLAVOR` or `MVM_HCLOUD_SERVER_TYPE` beside either number ends in a refusal naming both (`package/mvm/cli/src/backend/size-request.ts`).
So the command line does not win over the environment in the built code;
the pair is refused.

### `mvm run -- <command>`

The numbers exist on the command and on the backend interface (`package/mvm/cli/src/cli.ts`,
`package/mvm/cli/src/backend/types.ts`),
and every backend hands them to its create since merge `95f65ed945`:
`ovhRun` and `hetznerRun` take `vcpus` and `memoryGiB` and pass them into `ephemeralRun`
(`package/mvm/cli/src/backend/ovh/exec.ts`,
`package/mvm/cli/src/backend/hetzner/exec.ts`).
A floor beside `--from` is refused by `ephemeralRun` itself,
before any machine is made.
Issue 6664 held the opposite and is closed;
its evidence is what the built code now prevents:
`mvm --backend ovh run --vcpus 4 --memory 8 -- <command>` made a machine of one core and 2 GiB,
with no refusal and no warning.
The MCP `run_in_vm` takes the same two arguments and reaches the same code.

### `mvm create --from SOURCE`

A clone beside either number is refused:

> --from clones an existing machine and keeps its size,
> so --vcpus and --memory cannot be honoured with it

That reason is not yet true of the clone itself.
The libvirt clone writes a fresh domain with the default 4 vCPUs and 8,192 MiB,
the OVHcloud clone resolves the region's smallest in-stock flavour (`package/mvm/cli/src/backend/ovh/lifecycle.ts`,
`ovhClone`),
and the Hetzner clone takes the cheapest type of the source's architecture.
So a clone is not the source's size on any of the three today,
and nobody has run a clone of a larger source onto a smaller size.

### `mvm list`

All three backends now print the size beside each machine:
libvirt from the size mvm wrote into the machine's own metadata,
OVHcloud from the flavour record the instance names,
and Hetzner from the server type.
A libvirt guest made before mvm recorded sizes shows no size at all.

### `mvm --backend ovh update`

Still reports each region's smallest in-stock flavour,
its image count and the instance quota,
and prints the flavour's memory number as GiB.
Nothing changed that has to change:
the smallest in stock is no longer what a create takes,
so this command answers a different question than a create does.

### The departures from this file's own choices, and what happens to each

1.  **The family rule reads the name and leaves `i1` out.**
    The build keeps a flavour whose family token,
    the part of its name before the first hyphen,
    is one of `b2`,
    `b3`,
    `c2`,
    `c3`,
    `d2`,
    `r2`,
    `r3` (`package/mvm/cli/src/backend/ovh/flavor-families.ts`).
    This file takes that list,
    so the storage family is not a candidate.
    Taken into "The rule that keeps GPU flavours out":
    the storage family is left out,
    with the measurement that keeping it changes no pick,
    and the list guards the sized walk only,
    so a numberless create or a clone can still take a family it excludes.
2.  **A clone refuses either number.**
    Taken into "What each command does",
    with the reason the refusal states,
    which is a claim about the clone that has to become true or change.
3.  **`--memory` is compared with the catalogue's number directly.**
    Built at `flavors.ts` (`flavor.ram >= request.memoryGiB`) and at `server-types.ts` (`type.memory >= request.memoryGiB`).
    That is Decision 6's second option,
    which the build keeps on both cloud backends;
    the first option converts once on a cloud,
    and libvirt already multiplies by 1,024.
    So a yes to the first option changes the cloud comparison and a yes to the second changes nothing.
4.  **A flavour named in the environment is refused beside either number.**
    Built at `package/mvm/cli/src/backend/size-request.ts`:
    the two sources are refused rather than resolved,
    so `MVM_OVH_FLAVOR` beside `--vcpus` ends a create that naming the flavour
    alone would have run.
    Taken into
    "What is built today",
    and marked open to the owner's veto here:
    the alternative is that the command line wins and the create logs the
    variable it ignored.
5.  **One price read for each create.**
    Taken into Decision 4 and into "What an hour of a machine costs",
    with what the read does not include,
    which is filed as issue 6666.

## What an hour of a machine costs

OVHcloud's price page,
fetched 2026-11-28T07:52:43-05:00,
sells a machine's hour in three lines,
and only one of them is the line the catalogue prices.

The first line is the instance row,
the figure beside a flavour's name.
The second is the public address,
and the page says under each of its three VM tables:

> Instances do not include an IPv4 address by default:
> a public IPv4 address is available as an option,
> including when it is not attached to an instance.
> You can view the prices in the Control Panel when you place your order.

Its own line is `Public IP IPv4 (/32)` at CA$0.00432 per hour (CA$3.1536 per month).
`mvm` reaches every instance on a public IPv4 (`package/mvm/cli/src/backend/ovh/instances.ts`),
so every machine `mvm` makes carries this line.
The footnote above sits under the price page's three VM tables and not under the Discovery or Storage Optimized ones,
and OVHcloud's first article on the 2026-10-01 model change states the line for every range,
in French:
"La facturation est à la seconde,
sur toutes les gammes de VM Public Cloud".
So the line is the file's reading of the page for `d2` and `i1` as well,
and the cited article is where it comes from.

The third is local storage,
and only the Gen 3 families sell it apart from the instance.
The page's own section reads:

> Ephemeral local storage,
> designed for hot,
> temporary data directly on your instances.

and its rows price it by size,
for example:

> Local Storage NVMe 50 GB Gen 3 (B3,
> C3,
> R3) 99.99% CA$0.011665 ~CA$8.51545

That is CA$0.0002333 per GB-hour,
and the four packages are the per-GB rate times 50,
100,
200 and 400 GB exactly.
OVHcloud's blog of 2026-08-25,
fetched 2026-11-28T07:58:15-05:00,
says which machines carry the line:

> Local storage and public IPv4 addresses,
> which were previously included in the price of Generation 3 instances (B3,
> C3,
> R3),
> are now two separate billing lines.

and

> Local storage remains included on all previous ranges (B2,
> C2,
> R2 and earlier).

It also gives each Gen 3 size's local disk:

> Example:
> a B3-64 comes with 400 GB,
> no matter what your actual use is.

and its table states,
size by size,
the disk and the percentage the bill rose:
`b3-8 50 GB +19.5 %`,
`b3-16 100 GB +16.9 %`,
`c3-4 50 GB +21.9 %`,
`c3-8 100 GB +18.9 %`,
`r3-16 50 GB +15.1 %`,
`r3-32 100 GB +13.1 %`.
The blog's own rates are in euros and this document's are in Canadian dollars,
so the percentages are what it is quoted for.

So the whole hour of an OVHcloud machine in `BHS5` is:

```text
whole = instance row + CA$0.00432
      + (Gen 3 only) local disk in GB × CA$0.0002333
```

Two checks that this sum is the page's own arithmetic.
First,
every published percentage comes out within a tenth of a point:
the sum gives `b3-8` +19.5 percent against the page's +19.5,
`b3-16` +16.9 against +16.9,
`c3-4` +21.8 against +21.9,
`c3-8` +18.9 against +18.9,
`r3-16` +15.1 against +15.1,
and `r3-32` +13.0 against +13.1.
Second,
the page says the increase is a fixed amount per size:

> The increase is identical in euros for all instances with the same storage capacity:
> storage is capped at 400 GB,
> while the IP address will cost a fixed amount.

### What the built price read counts, and what it gets wrong

`package/mvm/cli/src/backend/ovh/catalog-prices.ts` reads the public order catalogue once and builds a map from plan code to hourly price.
Its own doc comment states the join:

> an addon's `planCode` is the flavor's own `planCodes.hourly`,
> such as `d2-2.consumption`

and the currency and tax basis:

> `locale` is `{"currencyCode":"CAD","subsidiary":"CA","taxRate":0}`,
> so the currency and tax rate a price is stated in come from the answer itself

The type it reads carries `currencyCode` alone,
so the currency is named from the answer and a tax rate is not read:
the printed line names a currency and says nothing about tax.
Decision 3's mechanism says which two a completed line would name.

The create then asks that map for one plan code per flavour (`flavors.ts`,
`hourlyPriceFor`),
which is the instance row and nothing else.
The IPv4 line and the local storage line are other products with other plan codes,
and no code path looks them up.
So the built order compares instance rows and prints an instance row.

What that gets wrong:

- **Every printed figure is low**
  by 0.7 to 25.9 percent of the machine's hour.
  `d2-2` is the worst,
  at CA$0.02102 whole against CA$0.0167 printed;
  `b3-8` CA$0.09799 against CA$0.082;
  `b3-16` CA$0.19135 against CA$0.1637;
  `c3-8` CA$0.17375 against CA$0.1461;
  `d2-8` CA$0.06392 against CA$0.0596;
  `b2-7` CA$0.11782 against CA$0.1135.
  The line the owner agreed to see before anything is created is therefore low on every create.
- **The order among sizes can be wrong**
  though no ask in the measured grid is.
  Over 2048 asks,
  1 to 16 vCPUs by 1 to 128 GB,
  adding the IPv4 line changes no pick;
  adding each Gen 3 size's own local disk changes no pick either.
  Forcing every Gen 3 size to the 400 GB line,
  which is an upper bound above what most sizes carry,
  changes 30 of the 2048 (for example 1 vCPU and 9 to 15 GB,
  where `r3-16` at 50 GB becomes dearer than `r2-15`).
  So the choice survives at the disks the sizes actually carry and is not proved at every disk a future size might carry.
- **The row understates the Gen 3 families most**
  in relative terms,
  on the smallest sizes.
  `d2-2` is 25.9 percent above its row and `c3-4` 21.8 percent,
  where `c2-15` is 1.4 percent above and `i1-45` 0.7 percent.
  A ratio built from rows therefore overstates how much more a Gen 3 size costs.

That the read counts only the instance row is a defect of the built code against the decision the owner made:
the chosen text says the create prints "the size and its price",
and the price a person is charged is the three lines together.
It is filed as issue 6666,
whose text gives the two plan codes the page shows and the asks the measured grid moved.

### What a run costs, and whether there is a ceiling

A guest round that builds this repository's crate takes minutes:
the record's guest builds took between 6 and 14 minutes each,
and a round's whole life there sums to 1,238 seconds,
or about 20.6 minutes,
counting create,
prepare,
the crate's build,
the arm and destroy.
For a 6-minute build alone,
a 14-minute one,
and a whole round of 20.6 minutes:
`b3-16` costs CA$0.01914,
CA$0.04465 and CA$0.06580 whole;
`d2-8` CA$0.00639,
CA$0.01491 and CA$0.02198;
`d2-2` CA$0.00210,
CA$0.00490 and CA$0.00723.
Which of the three a round gets is Decision 7's question,
and today a round that names no size gets `d2-2`.

There is no spend ceiling in the code,
and no option that sets one.
The only bound is the standing yes's six machines alive at once,
which is per hour:
six `d2-2` is CA$0.12612,
six `d2-8` CA$0.38352,
and six `b3-16` CA$1.14810.
An agent that loops creates has nothing else stopping it,
which is the state this file leaves open rather than decides.

## The prices, whole

Every figure in this list is the whole hour in Canadian dollars,
excluding tax,
for `BHS5`,
read from the page on 2026-11-28 and summed by the rule in "What an hour of a machine costs".
Each states the instance row,
then the whole.

- `d2-2`:
  1 vCPU,
  2 GB,
  25 GB disk.
  Row CA$0.0167,
  whole **CA$0.02102**,
  which is 25.9 percent above the row.
- `d2-4`:
  2 vCPUs,
  4 GB,
  50 GB disk.
  Row CA$0.033,
  whole **CA$0.03732**.
- `c3-4`:
  2 vCPUs,
  4 GB,
  50 GB local storage.
  Row CA$0.0732,
  whole **CA$0.08919**.
- `d2-8`:
  4 vCPUs,
  8 GB,
  50 GB disk.
  Row CA$0.0596,
  whole **CA$0.06392**.
- `b2-7`:
  2 vCPUs,
  7 GB,
  50 GB SSD.
  Row CA$0.1135,
  whole **CA$0.11782**.
- `c2-7`:
  2 vCPUs,
  7 GB,
  50 GB SSD.
  Row CA$0.1629,
  whole **CA$0.16722**.
- `b3-8`:
  2 vCPUs,
  8 GB,
  50 GB local storage.
  Row CA$0.082,
  whole **CA$0.09799**,
  19.5 percent above the row.
- `r3-16`:
  2 vCPUs,
  16 GB,
  50 GB local storage.
  Row CA$0.1061,
  whole **CA$0.12209**.
- `b2-15`:
  4 vCPUs,
  15 GB,
  100 GB SSD.
  Row CA$0.2148,
  whole **CA$0.21912**.
- `b3-16`:
  4 vCPUs,
  16 GB,
  100 GB local storage.
  Row CA$0.1637,
  whole **CA$0.19135**,
  16.9 percent above the row.
- `c3-8`:
  4 vCPUs,
  8 GB,
  100 GB local storage.
  Row CA$0.1461,
  whole **CA$0.17375**,
  18.9 percent above the row.
- `c2-15`:
  4 vCPUs,
  15 GB,
  100 GB SSD.
  Row CA$0.3162,
  whole **CA$0.32052**.
- `r3-32`:
  4 vCPUs,
  32 GB,
  100 GB local storage.
  Row CA$0.2119,
  whole **CA$0.23955**.
- `r2-30`:
  2 vCPUs,
  30 GB,
  50 GB SSD.
  Row CA$0.1882,
  whole **CA$0.19252**.
- `r2-60`:
  4 vCPUs,
  60 GB,
  100 GB SSD.
  Row CA$0.3661,
  whole **CA$0.37042**.
- `i1-45`:
  8 vCPUs,
  45 GB,
  50 GB SSD and 1.9 TiB NVMe.
  Row CA$0.648,
  whole **CA$0.65232**.

What a wrong pick costs per hour,
on whole figures:

- `c2-7` instead of `b2-7`,
  at the same 2 vCPUs and 7 GB:
  CA$0.04940 more,
  41.9 percent.
- `b2-7` instead of `b3-8`,
  for one GB less memory:
  CA$0.01984 more,
  20.2 percent.
- `b3-8` instead of `d2-8`,
  for 2 fewer vCPUs and the same 8 GB:
  CA$0.03407 more,
  53.3 percent.
- `c3-8` instead of `d2-8`,
  at the same 4 vCPUs and 8 GB:
  CA$0.10983 more,
  171.8 percent.
- `b2-7` instead of `d2-8`,
  for one GB less memory and 2 fewer vCPUs:
  CA$0.05390 more,
  84.3 percent.
- `c3-4` instead of `d2-4`,
  at the same 2 vCPUs and 4 GB:
  CA$0.05187 more,
  139.0 percent.
- `b2-15` instead of `b3-16`,
  for one GB less memory:
  CA$0.02777 more,
  14.5 percent.
- `b3-16` instead of `d2-2`:
  CA$0.17033 more,
  9.10 times.
- `b3-16` instead of `d2-8`:
  CA$0.12743 more,
  2.99 times.

## The unit of the catalogue's memory number

Two billed runs measured it on 2026-11-28,
and the record is the section "What a size's memory number comes to inside the guest"
of `doc/decision/mvm-gains-an-ovhcloud-backend-billed-by-the-second.md`,
written from the first run (merge `95f65ed945`)
and the second (commit `b6d8e0020c`).
A size listed with N holds N × 1,000 MiB,
which is neither N gibibytes nor N decimal gigabytes in thousands.

An ask of two cores and 4 GiB took the size `d2-4`,
whose catalogue number for memory is 4,
and the second run read three numbers inside that guest:

- the firmware's memory device reads `Size: 4000 MB`;
- the kernel's boot line reads `Memory: 2901048K/4095460K available`,
  whose second number is 3,999.5 MiB,
  the memory the hypervisor gave the guest less about half a MiB the firmware kept;
- `MemTotal` reads 3,906,276 kB,
  which is 3,814.7 MiB,
  or 3.725 GiB.

So `d2-4` was given 4,000 MiB:
not 4 GiB,
which is 4,096 MiB,
and not 4 GB in thousands,
which is 3,815 MiB.
Its guest kept the other 185 MiB for the kernel,
4.6 percent of what the machine was given.

One size was measured.
OVHcloud's own guide to the OpenStack client prints `4000` in the `d2-4` row of its captured `openstack flavor list`,
and in each of the eight other rows it prints 1,000 times the number in the size's name,
which agrees with the two runs.
A boot line from one `d2-8` guest would check a second size,
and no such reading is on record;
every figure here for a size other than `d2-4` therefore rests on the rule and says so.

### What a floor measures

A floor measures the memory a machine is given,
never the memory its guest reports,
because the kernel keeps part of the machine for itself.
This machine's libvirt guests keep about 251 MiB:
an 8,192 MiB domain reports `MemTotal` 8,131,780 kB,
which is 7,941 MiB,
or 7.755 GiB.
The `d2-4` guest kept 185 MiB.

A split fitted to those two readings,
one sixty-fourth of the domain for the kernel's page tables plus a fixed 122.8 MiB,
predicts the `d2-4` guest to within 0.03 MiB.
On that split,
which is an inference about machines other than the two measured,
an 8,000 MiB size gives a guest of about 7,752 MiB (7.57 GiB)
and a 16,000 MiB size one of about 15,627 MiB (15.3 GiB).

The line a create prints today calls the catalogue's number gibibytes.
The README's captured line for the `d2-4` run reads:

> size at least 2 vCPUs and at least 4 GiB of memory fits d2-4 (2 vCPUs,
> 4 GiB of memory,
> 50 GB of disk),
> 0.033 CAD per hour

and that machine holds 4,000 MiB.
The mislabel is a defect of the built line against the measured unit,
it is on the list of what the owner should hear,
and no decision in this file depends on it.

### What the measured unit does to a request for 4 vCPUs and 8 GiB of memory

8 GiB is 8,192 MiB,
and a size listed with 8 holds 8,000 MiB,
which is 192 MiB,
or 2.34 percent,
less.
Two rules can answer a floor written that way,
and they are Decision 6's two options rather than two readings of one fact:

- a strict floor in gibibytes takes the nearest size whose own number covers 8,192 MiB,
  which is `b3-16` at whole CA$0.19135 (4 vCPUs,
  16,000 MiB,
  100 GB of local storage),
  9.10 times the CA$0.02102 a numberless create makes,
  and its guest holds about 15,627 MiB on the split;
- the size the provider calls by that number is `d2-8` at whole CA$0.06392
  (4 vCPUs,
  8,000 MiB,
  50 GB of disk),
  3.04 times the same numberless create,
  and its guest holds about 7,752 MiB on the split.

Whether either machine holds the measured build is "The workload on record"'s question,
and Decision 6 gives both margins beside the prices.
The unit a person types is Decision 6's question,
and this file no longer offers two readings of the catalogue's number.

## The rule that keeps GPU flavours out

`mvm` keeps a flavour only when its family is one it may pick,
and the built rule reads the family from the name:

> A flavor's family is the catalogue's own naming:
> the token before the first hyphen,
> so `d2-2` and `c3-4-flex` are family `d2` and `c3`.

The names kept are `b2`,
`b3`,
`c2`,
`c3`,
`d2`,
`r2` and `r3`.
Everything else is left out,
including a family OVHcloud adds after this code is written,
so an unknown family cannot be picked by a size request.

The rule guards the sized walk and nothing else.
`isKeptFlavor` is called in one place,
`flavorsThatFit` (`package/mvm/cli/src/backend/ovh/flavors.ts`),
which only a create carrying `--vcpus` or `--memory` reaches.
`selectFlavor` in the same file,
which a create with neither number and an OVHcloud clone both use,
never calls it:
it takes the smallest in-stock Linux flavour of any family.
So a create without numbers or a clone could take a GPU flavour if nothing smaller were in stock,
and the record says the same of this path and calls the guard open.
A round is adding the guard to those two paths,
which the coordinating session files as an issue;
until it lands,
the sentence "no GPU size is ever taken" holds for a sized create,
which is the one the owner's constraint was about,
and not yet for the other two.

What it drops from `BHS5`'s 182 flavours,
and what each drop is:

- the GPU families,
  whose names the price page files under `Cloud GPU` (`t1-*`,
  `t2-*`,
  `l4-*`,
  `l40s-*`,
  `a100-*`,
  `h100-*`,
  `h200-*`,
  `a10-*`,
  `rtx5000-*`,
  `b300-*`,
  `rubin-*`),
  all 18 of `ovh.raid-nvme.t1` and the rest;
- the baremetal type's 3 flavours,
  whose `osType` is not `linux`;
- the storage family `i1`,
  whose 3 Linux names are the ones the page sells under `Storage Optimized`:

  > Get ultra-fast IOPS,
  > with NVMe drives specially designed for databases and big data applications.

The built source gives the same reason:

  > the storage shaped family `i1`,
  > which OVHcloud's own concepts page calls [Storage Optimized][storage-optimized] and sells for NVMe input/output rather than for memory

Leaving it out costs nothing measurable:
with `i1` added to the kept list,
no ask among 2048 changes its cheapest size,
because `i1-45` at CA$0.648 is dearer than `b3-32` at CA$0.3274 and than `r3-64` at CA$0.4237.
What it gives up is the storage family's own NVMe capacity as a pick,
which `--server-type i1-45` still names outright.

Does the rule drop a flavour a person would want?
It drops the GPU models,
which is the owner's own sentence,
the metal instances,
which cannot boot a normal image,
and the NVMe storage family,
which the price order would never take.
What it costs:
a new general family,
or a region reporting names this build does not know,
yields no candidate and the create refuses rather than picking something larger,
and the list needs an edit before such a family can be used.

How sure the mapping is,
and what is not:
the family names come from OVHcloud's key concepts page,
whose table gives each family a description and,
for Discovery,
the words `Cost-Effective, Shared Resources`;
the live `type` strings (`ovh.d2`,
`ovh.ssd.eg`,
and so on) come from the one catalogue read of 2026-11-28 04:48 to 04:50,
and the mapping from those strings to the documented family names is made by the names in them (`d2`,
`b3`,
`c2`,
`r3`),
not by a field OVHcloud documents as a family.
That read covered `BHS5` only,
and it is in no file in this repository.
So the guard on the owner's one explicit constraint,
"Please don't pick GPU?
",
rests on one region and on names.
Three things would strengthen it,
in the order they cost:
commit the 182 names with their `type` and `osType` as a repository record,
read every default region once more,
and check the price page's `Cloud GPU` group against the kept families in the region the create runs in.

Why not the alternatives:

- **The families an image reserves.**
  `flavorType` is an image's list of families it may boot on (`package/mvm/cli/src/backend/ovh/images.ts`),
  and the one live string names `g1,g2,g3,t1,t2,i1,a100,l4,rtx5000,l40s`,
  which omits `h100`,
  `h200`,
  `a10`,
  `b300` and `rubin`,
  all of which the price page files under `Cloud GPU`.
  The same string lists `i1`,
  the storage family the kept list leaves out,
  beside the GPU families,
  and this file does not know why OVHcloud files it there.
  A region selling those would offer them as candidates.
- **A list of names in the source.**
  It drops exactly the names written into it,
  and a family added the next day is admitted until someone edits the code,
  which is the fail-open direction the owner's sentence rules out.

A flavour named whole with `--server-type` is taken as named under every rule,
GPU or not,
as today,
and no number may be given beside it.

## What a request is, and the sizes it ends on

A request is two floors and nothing else (`package/mvm/cli/src/backend/size-request.ts`):
`--vcpus N` asks for at least `N` virtual CPUs,
`--memory M` asks for at least `M` of the unit Decision 6 settles,
each option works alone,
and the other number then has no floor at all.

The candidate steps,
in the order the built code applies them,
with each one's effect in `BHS5`:

1.  In stock:
    `available` is true.
    The catalogue read's ten are all in stock.
2.  An OS the image can boot:
    `osType` is `linux`.
    This drops the three `win-` names and the baremetal type.
3.  A kept family (the section titled "The rule that keeps GPU flavours out").
    This drops the GPU families,
    the metal type and `i1`.
4.  Both floors met:
    `vcpus` at least the ask and the memory number at least the ask.
5.  A published hourly price:
    the catalogue prices the flavour's `planCodes.hourly`.
    A flavour with no published price is not a candidate,
    and the `-flex` twins are the case this file knows about:
    no `-flex` name appears anywhere in the Canadian price page's product data,
    so the built read never sees one.
    What the twins are priced at is known from another page:
    OVHcloud's blog of 2026-08-25 prints `b3-16-flex` at 82 euros against 87 for `b3-16`,
    `c3-8-flex` at 74 against 79,
    and `r3-32-flex` at 104 against 109,
    with a flex twin's local disk fixed at 50 GB whatever the size,
    so on the same sum a Gen 3 twin's whole hour is 4.9 to 10.2 percent lower than its plain size
    once the plain size carries more than 50 GB of disk.
    Whether the Canadian catalogue prices them is not yet known:
    a round is reading that catalogue for issue 6666,
    and this file says nothing about it until that answer exists.
    The built resize reason is the source's own:

    > mvm never resizes a machine,
    > it destroys and remakes one,
    > so the flex benefit is unused

    So the rule is "no published price,
    not a candidate",
    with the suffix as one case of it,
    which is the record's rule and not a separate one;
    and in the numbers-only fallback,
    which runs when no price can be read,
    the order sorts by disk before the flex flag,
    so a twin with a 50 GB disk sorts ahead of a plain size whose disk is larger.
    The record's give-up therefore holds only while the prices are read;
    the owner should hear that before answering.
6.  The image:
    the flavour's disk and memory reach the chosen image's floors,
    which are 0 on every image in `BHS5`.
7.  Quota:
    **not checked**.
    The flavour record carries `quota`,
    "how many more instances of this flavor the project's quota allows",
    and no code reads it before a create;
    the region's instance quota is read only by `mvm --backend ovh update`,
    and no core or memory quota is read anywhere.
    A create OVHcloud refuses for quota fails before any instance exists,
    with OVHcloud's own message;
    that is read from the call's shape and not from a run,
    because no create on record has been refused for quota.
    The create call itself runs once,
    so a size that goes out of stock between the read and the create ends the create with OVHcloud's refusal,
    bills nothing,
    and no next fitting size is tried.

The order among the survivors is Decision 4's.

### The owner's ask: 2 vCPUs and 6 GB

No size has 2 vCPUs and 6 GB,
which is the whole reason the owner's sentence exists.
With the recommended set the candidates are every kept size with at least 2 vCPUs and at least 6 GB,
the cheapest whole hour among them wins,
and it is `d2-8`:

- `d2-8`:
  4 vCPUs,
  8 GB,
  50 GB disk,
  whole **CA$0.06392**.
- `b3-8`:
  2 vCPUs,
  8 GB,
  50 GB local storage,
  whole **CA$0.09799**.
- `b2-7`:
  2 vCPUs,
  7 GB,
  50 GB SSD,
  whole **CA$0.11782**.
- `r3-16`:
  2 vCPUs,
  16 GB,
  50 GB local storage,
  whole **CA$0.12209**.
- `c2-7`:
  2 vCPUs,
  7 GB,
  50 GB SSD,
  whole **CA$0.16722**.
- `c3-8`,
  `c3-16`,
  `b3-16` and larger sizes also fit,
  at more money.

Under the record's own figure,
the same ask is CA$0.0596 an hour;
the line the create prints would say CA$0.0596 today and the machine bills CA$0.06392.

**Its family**
and what cores of its own would cost.
The built order picks `d2-8`,
whose family OVHcloud calls Discovery.
The price page says what that family is:

> Start your Public Cloud journey with instances that have shared resources,
> delivering stable performance at a very affordable price.

and

> These instances are perfect for test,
> development and sandbox environments.

The cheapest fitting size from a family that is not Discovery is `b3-8`,
at CA$0.09799 whole,
CA$0.03407 more per hour and 53.3 percent.
So the owner's ask is answered by a shared-resource machine with four vCPUs of which two are idle,
and the same two vCPUs with cores of their own cost CA$0.03407 an hour more.
Decision 5 puts that to the owner with the veto open.

### The workload's ask: 4 vCPUs and 8 GiB

This ask has one answer per rule,
because the catalogue's number is measured:

- under the provider's label,
  which is what a cloud backend does today,
  the cheapest fitting size is `d2-8` at whole CA$0.06392,
  a Discovery size whose guest holds about 7,752 MiB on the split;
  Decision 5's question does arise here,
  and the cheapest fitting size with cores of its own is `c3-8` at whole CA$0.17375,
  CA$0.10983 more per hour and 171.8 percent;
- under a strict floor in gibibytes the cheapest fitting size is `b3-16` at whole CA$0.19135
  (4 vCPUs,
  16,000 MiB,
  100 GB of local storage),
  whose guest holds about 15,627 MiB on the split;
  the fewest-numbers order would take `b2-15` at CA$0.21912 (4 vCPUs,
  15,000 MiB,
  100 GB SSD),
  CA$0.02777 more for 1,000 MiB less;
  and the cheapest with cores of its own is `b3-16` itself,
  which is General Purpose rather than Discovery,
  so Decision 5 does not arise for this ask.

The two machines stand 2.99 times apart in price,
and their guests about 7,875 MiB apart in total.

## The workload on record

The sizes are being chosen for the wrapper's guest rounds,
which build this repository's crate inside a guest,
and the numbers here are `/proc/meminfo` readings from guests:

- A default local guest asks for 8,192 MiB and reports `MemTotal` 8,131,780 kB,
  which is 7,941 MiB,
  or 7.755 GiB.
- In that guest,
  a `cargo build --release --tests --locked` with no job count given peaked at `AnonPages` 7,641,948 kB,
  7.29 GiB,
  with `MemAvailable` down to 88,740 kB,
  86.7 MiB.
  That build finished.
- With four jobs,
  the same build reached `AnonPages` 7,701,208 kB,
  7.34 GiB,
  and the guest's out-of-memory killer ended it.
- The recipe's own two-job arms reached `AnonPages` 6.59 to 6.91 GiB,
  one `rustc` at 7.0 to 7.07 GiB,
  and `MemAvailable` 0.38 to 0.69 GiB.
- One release `rustc` of this crate's tests takes 6.7 to 6.9 GiB,
  and issue 6611's round measured one at 7,019,956 kB of the guest's 7,941 MiB.

The headroom these figures give,
and what it costs a smaller guest:
the guest that came within 86.7 MiB of running out had 7,941 MiB in total,
and the build it ran touched 7,855 MiB of that total.
An 8,000 MiB size gives a guest of about 7,752 MiB on the split,
which is 103 MiB short of what that build touched,
and it holds the recipe's two-job arms by 200 to 518 MiB,
because those arms left 389 to 707 MiB free in a guest 189 MiB larger.
So the answer to "does an 8-number flavour hold this build" is:
it holds the two-job recipe with 200 to 518 MiB to spare,
and it is about 103 MiB short of the build that ran at default parallelism.
The 8,000 MiB figure is measured for `d2-4` and inferred for `d2-8`;
the reserve is this machine's own,
measured on two guests.
A 16,000 MiB size holds either build with about 7,772 MiB over what the larger one touched.

Each arm's figures keep that arm's own job count:
the four-job figure is the one the killer ended,
and the two-job and one-job figures are those builds.

What the record does not give:

- the disk the build needs,
  though disk is a tie-break in the order among fits and the cloud sizes carry 25,
  50,
  100 and up to 400 GB;
- any build on a shared-vCPU size,
  and none on a `b3-8`;
  one build of this crate has run on a cloud machine,
  on a `b3-16` named through `MVM_OVH_FLAVOR`,
  and it took 9 minutes 40 seconds;
- the job count a cloud guest would use,
  which follows from its vCPU count and not from the recipe's `--jobs 2`.

The rankings in "The decisions" are therefore for the guest rounds.
For an unasked `run_in_vm` of `uname`,
the workload is nothing like this.

## The host cap, and what a memory ask means on libvirt

Every libvirt guest runs inside `mvm-libvirt-session.service`,
and `mvm` sets that unit's `MemoryMax` from `/proc/meminfo` when it starts:
three quarters of the machine's total,
49,664,753,664 bytes here (`package/mvm/cli/src/session-daemon-memory.ts`,
`CAP_NUMERATOR` 3 and `CAP_DENOMINATOR` 4).
The owner chose three quarters on 2026-11-28 (`doc/troubleshooting/desktop-freezes-while-many-agent-runs-thrash-zram-swap.md`,
the section titled "The guests' unit at its own memory cap,
2026-11-28").

Four things follow:

- The cap belongs to the whole unit,
  not to each guest.
  Eight guests that had between them touched 58.2 GiB sat in a unit allowed 30.8 GiB,
  and the unit swapped against its own limit while the machine had memory free.
- A guest is charged what it touches,
  not what its domain asks:
  a domain asking 8 GiB charged a peak of 990 MB in the measurement of 2026-11-21 that the README records.
- When the unit exceeds its cap the kernel kills a process inside it,
  usually the largest guest,
  and `OOMPolicy=continue` keeps that from stopping the unit.
- An ask above the cap is **not**
  refused,
  and the built README says so:
  the domain starts,
  and only the unit's total is limited.
  A check in `create` would refuse a declaration the machine can accept,
  which is why the build does not make one.

## The decisions

Seven decisions.
The output block inside an option is the shape that option would print,
written by hand from the sizes and the prices this file read;
it is not a capture of a command that exists today,
and no claim about a built command rests on one.
Each decision states the question in plain words with the facts it needs,
each option states its mechanism,
a visible result on the prices in "The prices,
whole",
what it gives up,
and what a yes to it leaves undecided,
and each ranking gives a reason for every adjacent pair in what the option delivers for the workload,
never in how much code it needs.

### 1. Does a bare number mean at least, or exactly?

**The question**,
for anyone reading only this section:
`mvm create --vcpus 2 --memory 6 dev` on `BHS5`,
where no flavour has 2 vCPUs and 6 GB.
Does the command mean "at least these numbers",
so a larger flavour is the answer,
or "exactly these numbers",
so the command is refused when none has them?

The owner answered this on 2026-11-28 with "the numbers mean at least",
marked open to veto.

- **At least**
  (recommended,
  and the owner's answer).
  Mechanism:
  the backend keeps every flavour whose `vcpus` and memory number are both at least the ask,
  and the order among the survivors is Decision 4's.
  Result:
  `--vcpus 2 --memory 6` keeps `b2-7`,
  `c2-7`,
  `b3-8`,
  `d2-8`,
  `r3-16` and larger sizes,
  and the cheapest order ends on `d2-8` at whole CA$0.06392.
  `--vcpus 4` alone asks for nothing about memory and keeps every 4-vCPU flavour.
  Gives up:
  the guest can be larger than the ask and the bill with it,
  which on this ask is 2 extra vCPUs and 2,000 extra MiB;
  and a person who wants one exact size must name it with `--server-type`.
- **Exactly.**
  Mechanism:
  a flavour is a candidate only when its vCPUs are the asked count and its memory number is the asked number,
  so the fallback is what answers a miss (Decision 2),
  and reading the number as gibibytes then needs a marker the build does not have:
  a trailing `+`,
  or a second option such as `--at-least`.
  Result:
  `--vcpus 2 --memory 6` refuses,
  because no flavour has a memory number of 6,
  and says so in the shape Decision 2's refusal gives.
  `--vcpus 4 --memory 8` ends on `d2-8` at whole CA$0.06392 when the typed 8 means the catalogue's 8,
  because that size holds 8,000 MiB;
  under Decision 6's strict floor it refuses,
  because no size in `BHS5` holds exactly 8,192 MiB.
  Gives up:
  most cloud asks refuse on a provider with fixed mixes,
  which is the case the owner's own sentence asked `mvm` to handle,
  and every unattended `run` that carries a number gets no machine;
  on libvirt nothing changes,
  because a libvirt guest is made to the exact numbers either way.

Ranking:
at least > exactly.
At least delivers a machine for the owner's own example and for every ask at or below the largest size,
and exactly refuses the owner's example by default and leaves the work without a machine;
the other spelling costs one character in the "at least" answer and one new option in the "exactly" answer,
so neither rule is out of reach.

A yes to at least leaves undecided:
what happens above the largest size (Decision 2),
which size among those that fit (Decision 4),
the unit a person types (Decision 6),
and whether a suffix such as `8G` is a marker for one reading.

A yes to exactly leaves undecided:
the marker's spelling,
what a miss does (Decision 2),
the unit (Decision 6),
and whether libvirt keeps allocating exactly under the same bare number.

### 2. When no size meets the ask, refuse or take the nearest?

**The question**,
for anyone reading only this section:
a command whose numbers no flavour in the region satisfies,
for example `--vcpus 4 --memory 2048` where the largest kept size in stock has 1,024 GB,
or any ask under Decision 1's "exactly" answer.

Under Decision 1's "at least" answer this happens only above the largest size in stock,
which the price page's kept sizes put at 160 vCPUs (`c3-320` and `b3-640`) and 1,024 GB (`r3-1024`);
under "exactly" it happens on almost every command.

What is built today is already a refusal,
in a different shape.
`NoSizeFitsError` (`package/mvm/cli/src/backend/size-request.ts`)
names what was asked,
the largest size the region could have used,
and the largest size the family rule left out,
and it prints no price.
Its message is assembled from the source's own clauses,
`nothing ${place} can be made with ${asked}:`,
then the largest size the region could have used,
then the sizes the family rule left out,
and it ends `name a size outright with ${overrideOptions}`.
The opening clause and the closing one are copied from `NoSizeFitsError` in that file;
the two middle clauses are described here,
and the second is one of two alternatives the constructor picks between.
because its second clause is one of two alternatives the constructor picks between.

So a yes here changes a message the build already prints,
and adds prices to it.

- **Refuse**
  and name the nearest (recommended).
  Mechanism:
  the flavour list is read,
  the sizes closest to the ask are found by vCPUs and then memory in either direction,
  each is printed with its numbers,
  its disk and its whole hourly price and which way it is off,
  and the command exits nonzero having created nothing.
  Result,
  for `--vcpus 4 --memory 2048`:
  the command exits nonzero,
  having created nothing,
  with a message naming the largest size it could have used (`r3-1024`,
  128 vCPUs and 1,024 GB),
  the sizes the family rule left out,
  and,
  if this option is taken,
  the whole hourly price of each.
  No capture of that message is in this file;
  the clauses above are the built ones.
  Gives up:
  a second command,
  and an agent's unattended `run` fails where a larger region or a smaller ask would have run.
- **Take the nearest.**
  Mechanism:
  the same ordering picks the closest size in either direction,
  ties by whole hourly price and then by identifier,
  and the create proceeds.
  Result:
  under a floor nearest has no worked meaning,
  because every candidate is smaller than the ask:
  `--vcpus 4 --memory 2048` takes `r3-1024`,
  the largest size in stock,
  at whole CA$6.87564 an hour;
  `--vcpus 3` takes a 2-vCPU size,
  because vCPUs are compared first;
  `--vcpus 2 --memory 6` takes `b2-7` at whole CA$0.11782,
  the closest size in either direction,
  because its 2 vCPUs match the ask exactly and its 7,000 MiB is the nearest memory above it.
  Under "exactly" this is the only rule that ever creates anything.
  Gives up:
  the guest can be smaller than the ask,
  so a build sized to its memory is killed after minutes of billed work,
  and the kill looks like a defect of the work rather than of the size.

Ranking:
refuse > nearest.
A refusal costs one more command and names the sizes and their whole prices;
a too-small guest costs the round,
the billed minutes until someone notices,
and a failure that reads as the work's own.
The case for nearest is real and is why they are close:
under the "exactly" answer it is the only rule that creates anything at all,
and under "at least" the owner's own example never reaches this decision.

A yes to refuse leaves undecided:
whether the refusal prints a whole hourly price beside each size it names,
and whether it suggests the largest size it could have used as the next command.
A yes to nearest leaves undecided:
the measure (stated vCPUs then memory,
but not whether a memory-first measure was weighed),
and whether a guest smaller than the ask is announced as such.

### 3. How does a person see the size and its price before money is spent?

**The question**,
for anyone reading only this section:
between the command and the create call there is a moment in which the pick can still be changed for nothing.
What does `mvm` put in that moment,
so that a person and an unattended agent both learn what will be spent?

What is built today:
the OVHcloud CLI create logs one line before creating,
naming the flavour,
its numbers and its hourly price,
and it prints that line only on the sized walk (`package/mvm/cli/src/backend/ovh/provisioning.ts`,
`chooseFlavorAndImage`);
a create with neither number prints only its creating line,
with the flavour and its numbers and no price,
although it reads the prices before it looks at whether a number was asked;
the Hetzner create logs the type and its numbers with no price;
the MCP `create_vm` answers `VM <name> created and started` with no size;
there is no `--preview` and no `mvm sizes`.
Both printed lines call the catalogue's number "GiB",
which the measured unit says it is not.

- **The create's own line**,
  completed (recommended first step).
  Mechanism:
  the same resolution pass that picks the size prints one line before the create call,
  with the size,
  its numbers,
  its whole hourly price and the currency and tax basis,
  and no state is written.
  The MCP answer returns the same line instead of a bare "created".
  Result:

  ```text
  creating mvm-dev on ovh in BHS5: d2-8, 4 vCPUs, 8 GB, 50 GB disk, image Ubuntu 26.04,
    CA$0.06392 per hour whole (CA$0.0596 instance + CA$0.00432 IPv4), excl. VAT
  ```

  Gives up:
  nothing new to run,
  but the line arrives after the choice is made,
  so it informs without offering an alternative,
  and an agent that wants to compare sizes has to create one first.
- **A list command**
  `mvm sizes --vcpus 2 --memory 6` (recommended second step).
  Mechanism:
  the same candidate steps and the same price read,
  printed as a list,
  with the size the order would take marked,
  touching nothing.
  Result:

  ```text
  $ mvm --backend ovh sizes --vcpus 2 --memory 6
  asks at least 2 vCPUs and at least 6 GB in BHS5; the four cheapest of the sizes that fit:
    d2-8   4 vCPUs   8 GB   50 GB disk              CA$0.06392/h whole  <- taken
    b3-8   2 vCPUs   8 GB   50 GB local storage     CA$0.09799/h whole
    b2-7   2 vCPUs   7 GB   50 GB SSD               CA$0.11782/h whole
    r3-16  2 vCPUs  16 GB   50 GB local storage     CA$0.12209/h whole
  ```

  Gives up:
  it is a second command,
  nothing ties its answer to the create that follows,
  and `run` has no equivalent,
  so an agent that spends money unattended still sees only the create's line.
- **A preview flag**
  `--preview`,
  on `create` and on `run`.
  Mechanism:
  the whole resolution runs,
  including the price read,
  the line is printed and the command exits 0 without calling the provider's create.
  Result:

  ```text
  $ mvm --backend ovh create --preview --vcpus 2 --memory 6 dev
  would create mvm-dev on ovh in BHS5: d2-8, 4 vCPUs, 8 GB, 50 GB disk, image Ubuntu 26.04,
    CA$0.06392 per hour whole (CA$0.0596 instance + CA$0.00432 IPv4), excl. VAT
  ```

  Gives up:
  one more provider read for every preview,
  and a second code path that must keep printing exactly what the create prints,
  which is a way for the two to disagree.

The three are not alternatives.
The create's line is the only one an unattended agent sees,
and the list command is the only one that shows what else fits,
so the recommended pair is "the create's line,
then the list command",
both of which are independent of the preview flag;
a yes to the pair leaves the flag available and a yes to the flag adds a third way to see the same thing.

Ranking:
the create's line > the list command > the preview flag.
The create's line is what the owner already chose and is what every caller sees,
including the MCP one;
the list command adds the alternatives at the cost of a second command,
and spends nothing while it does it;
the preview flag prints the create's own line and adds a stop before the spend,
which the list command already gives by touching nothing;
keeping its printing in step with the create's is what the preview costs,
and no option ranks lower for the code it needs.

A yes to the create's line leaves undecided:
whether the MCP answer repeats the size,
whether Hetzner prints a price and on what tax basis,
and whether the suffix of a larger-than-asked machine is announced.

A yes to the list command leaves undecided:
whether it exists on `run`,
and whether it names the sizes the family rule left out.

A yes to the preview flag leaves undecided:
whether `run` gets it,
and whether the preview's own read is cached.

### 4. Which size among those that fit?

**The question**,
for anyone reading only this section:
`--vcpus 2 --memory 6` on `BHS5` is satisfied by `b2-7`,
`c2-7`,
`b3-8`,
`d2-8`,
`r3-16` and larger sizes.
Which one is made?

The owner answered this on 2026-11-28 with "Cheapest that fits",
marked open to veto,
and it is built on OVHcloud.

- **The cheapest by the published hourly price**
  (recommended,
  the owner's answer,
  and built).
  The record's chosen text prices by "OVHcloud's public hourly prices"
  and gives the example `d2-8` at "CA$0.0596 per hour",
  which is the instance row;
  the price the build reads and orders by is that row,
  and the whole hourly figures in this file are this file's reading of what a machine bills,
  which issue 6666 records as not what the built line prints.
  Mechanism:
  one read of OVHcloud's public order catalogue per create,
  a flavour's price found by its own `planCodes.hourly`,
  a flavour with no published price never a candidate,
  the lowest price first,
  and a tie broken by vCPUs,
  then memory,
  then disk,
  then the plain size before its `-flex` twin,
  then the flavour's identifier,
  which `orderFlavors` compares with `left.id.localeCompare(right.id)`
  and which is the UUID instance creation takes as `flavorId` (`package/mvm/cli/src/backend/ovh/types.ts`),
  not the flavour's name;
  the tests use the name as the identifier,
  so no test can show that difference.
  When the read cannot be had,
  the order falls back to the numbers alone and the command says so.
  The built source states the point of the order:

  > With the prices in hand the survivors are ordered by hourly price,
  > cheapest first,
  > so the machine costs the least that meets the ask rather than being the smallest that meets it;
  > the project chose that order on 2026-11-28.

and the consequence of an unpriced size:

  > A size the catalogue publishes no hourly price for is never a candidate then.

Result:
`--vcpus 2 --memory 6` ends on `d2-8` at whole CA$0.06392;
`--vcpus 2 --memory 4` ends on `d2-4` at whole CA$0.03732 rather than `c3-4` at CA$0.08919
when the typed 4 is read as the catalogue's 4,
and on `d2-8` at CA$0.06392 under Decision 6's strict floor.
Gives up:
one provider read per create and a source that is not the page a person reads,
a guest larger than the ask (2 extra vCPUs and 2,000 extra MiB on the owner's ask,
the whole reason Decision 5 exists),
and,
until issue 6666 lands,
a printed figure that is the instance row rather than the whole hour,
which the measured grid shows changes no pick over 2048 asks but does not prove for every ask.
- **The fewest numbers.**
  Mechanism:
  the fewest vCPUs that fit,
  then the least memory,
  then the smallest disk,
  then the plain size before its twin,
  then the flavour's identifier,
  and no price is read.
  Result:
  `--vcpus 2 --memory 6` ends on `b2-7` at whole CA$0.11782,
  CA$0.05390 more than `d2-8` and 84.3 percent;
  `--vcpus 2 --memory 4` ends on `c3-4` at whole CA$0.08919,
  CA$0.05187 more than `d2-4` and 139.0 percent;
  `--vcpus 4` with 8 GiB ends on `b2-15` at CA$0.21912,
  CA$0.02777 more than `b3-16` and 14.5 percent.
  Each of the three is a tie on vCPUs,
  memory and disk between a `b`,
  `c` or `d` size:
  `b2-7` against `c2-7`,
  `c3-4` against `d2-4`,
  and `b2-15` against `c2-15`.
  Which one of each pair wins is decided by the identifier,
  whose order this file cannot read for `BHS5`,
  so the percentages are what a tie costs and not a guaranteed result.
  Gives up:
  the bill is whatever the catalogue's numbers imply,
  and those numbers do not order by money:
  every one of the three examples costs more than the cheapest fit,
  while the guest is never larger than the ask.
- **Refuse**
  and list what fits.
  Mechanism:
  every fitting size is printed with its numbers and its whole price and the command exits nonzero without creating.
  This is not an order among fits but a choice to create or not,
  and an owner who wants "cheapest,
  and refuse when more than one fits unless a size is named"
  would add this refusal to the first option,
  which the two decisions leave separable.
  Result:

  ```text
  $ mvm --backend ovh create --vcpus 2 --memory 6 dev
  the cheapest four of the sizes in BHS5 that fit at least 2 vCPUs and at least 6 GB:
    d2-8   4 vCPUs   8 GB   50 GB disk              CA$0.06392/h whole
    b3-8   2 vCPUs   8 GB   50 GB local storage     CA$0.09799/h whole
    b2-7   2 vCPUs   7 GB   50 GB SSD               CA$0.11782/h whole
    r3-16  2 vCPUs  16 GB   50 GB local storage     CA$0.12209/h whole
  name one with --server-type, or drop the floors to take the backend's default
  ```

  Gives up:
  every `run`,
  every `run_in_vm` and every scripted create fails whenever more than one size fits,
  which is most of the time,
  and the person must name a size they may not know.

Ranking:
cheapest > fewest numbers > refuse.
Cheapest is the only one that spends the least money for a fitting guest,
and on this catalogue the other two cost 14.5 to 139.0 percent more per hour for the same or fewer resources;
fewest numbers ranks above a refusal because it still delivers a guest,
while a refusal leaves every unattended create without a machine.
Discovery's own question is Decision 5's,
not this one's.

A yes to cheapest leaves undecided:
whether Discovery sizes are candidates (Decision 5),
whether the printed figure is the whole hour (issue 6666),
which order Hetzner takes among the types that fit,
which is the smallest numbers and which no decision asks,
and the order a real flavour list's identifiers give,
which the built code uses as the last tie-break and which this file cannot read for `BHS5`.

A yes to fewest numbers leaves undecided:
the same last tie-break,
and whether a price is read at all for the printed line.

A yes to refuse leaves undecided:
whether the refusal's list is a command of its own (the list in Decision 3),
and what `run` does under it.

### 5. May the cheapest size be a Discovery one?

**The question**,
for anyone reading only this section:
the cheapest fit for small asks is a `d2` flavour,
and OVHcloud's own documentation says what that family is:

> Discovery | Cost-Effective,
> Shared Resources | Entry-level instances with shared resources,
> offering stable performance at an affordable price.
> Great for testing environments,
> training,
> or proof-of-concept projects.

Its vCPUs are shared resources in OVHcloud's own words,
and the price page calls the family "perfect for test,
development and sandbox environments".
The owner's one explicit constraint was "Please don't pick GPU?
";
this decision asks whether a shared-resource size may be picked.

This is separable from Decision 4:
an owner can take the cheapest order and still keep shared-resource machines out,
and that combination is built by editing one list.

The choice,
and every figure below,
does not depend on Decision 6:
the owner's ask is 6,
which both rules read as needing the 8,000 MiB size.

- **Keep Discovery as a candidate**
  (recommended,
  and what the build does).
  Mechanism:
  `d2` is on the kept list,
  so a `d2` flavour is a candidate whenever it fits,
  and the price order then picks it whenever it is cheapest.
  Result:
  the owner's ask ends on `d2-8`,
  4 shared vCPUs and 8,000 MiB,
  whole CA$0.06392;
  `--vcpus 1` alone ends on `d2-2`,
  whole CA$0.02102;
  a create asking 4 vCPUs and 8 GiB ends on `d2-8` under the provider's label,
  and on `b3-16` under a strict floor,
  where Discovery has no size big enough to matter.
  Gives up:
  the machine's vCPUs are shared with other customers,
  the printed line says nothing about that,
  and no build has been timed on one,
  so what a shared vCPU costs this repository's build in minutes is unmeasured.
  What is known is the money:
  it is the cheapest answer by 53.3 percent on the owner's ask.
- **Leave Discovery out.**
  Mechanism:
  `d2` comes off the kept list,
  which is the same list the build already applies in `flavor-families.ts`,
  and nothing else changes.
  Result:
  the owner's ask ends on `b3-8` at whole CA$0.09799,
  53.3 percent more;
  `--vcpus 1` alone ends on `c3-4` at whole CA$0.08919,
  which is 4.24 times `d2-2`;
  an ask of 4 vCPUs and 8 GiB ends on `c3-8` at whole CA$0.17375 under the provider's label,
  171.8 percent more than `d2-8`,
  and on `b3-16` under a strict floor,
  the same machine either way.
  Gives up:
  CA$0.03407 an hour on the owner's ask and CA$0.10983 on that 8 GiB ask,
  and the smallest machine a sized create can make goes from 2,000 MiB to 4,000 MiB,
  which also removes the only size under CA$0.03 an hour;
  the numberless create and the clone still reach `d2-2`,
  because the family rule guards only the sized walk.

Ranking:
keep > leave out.
Keeping delivers four shared vCPUs and 8,000 MiB for CA$0.06392,
which is 53.3 percent less than the two dedicated vCPUs and 8,000 MiB that leaving it out buys on the owner's ask;
leaving it out buys cores of the machine's own,
which is a real purchase at 1.5 to 4.2 times the hour.
The owner is not being asked to weigh a risk here:
no build has run on a shared-vCPU size,
so nothing is measured on either side,
and a round is timing one build on a `d2-8` and one on a `b3-8` now.
When those come back the minutes are a number,
and the question is the money against that number alone.

A yes to keeping leaves undecided:
whether the printed line marks a shared-resource size,
and whether a guest round may be placed on one by default.

A yes to leaving out leaves undecided:
whether a named `--server-type d2-8` still works (it does under either answer),
and whether a separate family option exists for the storage family `i1`.

### 6. Which unit does a typed `--memory 8` carry?

**The question**,
for anyone reading only this section:
`mvm create --vcpus 4 --memory 8 build-box` on `BHS5`,
where the cheapest size is `d2-8`,
whose catalogue number for memory is 8.
A size listed with N holds N × 1,000 MiB,
which two billed runs measured and "The unit of the catalogue's memory number" gives,
so a machine the catalogue calls 8 carries 8,000 MiB.
The typed 8 therefore cannot mean both "8 GiB of memory" and "the size called 8",
and on this ask the choice runs from CA$0.06392 to CA$0.19135 an hour.

What is built today is the second option below:
on a cloud backend the typed number is compared straight with the catalogue's number
(`package/mvm/cli/src/backend/ovh/flavors.ts`,
`flavor.ram >= request.memoryGiB`,
and `package/mvm/cli/src/backend/hetzner/server-types.ts`,
`type.memory >= request.memoryGiB`),
and on libvirt it becomes `M × 1,024` MiB
(`package/mvm/cli/src/create.ts`,
`memoryGiB * MEBIBYTES_PER_GIBIBYTE`).
So the two kinds of backend already disagree by 2.34 percent about what a typed 8 means.

- **A strict floor in gibibytes**
  (recommended).
  Mechanism:
  the typed number is gibibytes;
  on libvirt nothing changes,
  because the domain is given `M × 1,024` MiB today;
  against a cloud catalogue the comparison converts once,
  so a size is a candidate when its own number N satisfies `N × 1,000 MiB ≥ M × 1,024 MiB`.
  Result,
  with the whole price of each pick at one vCPU and at four:

  - 2 GiB:
    `d2-2` at CA$0.02102 becomes `d2-4` at CA$0.03732,
    1.78 times;
    at four vCPUs the ask already needs `d2-8`,
    so both rules take it,
    at CA$0.06392.
  - 4 GiB:
    `d2-4` at CA$0.03732 becomes `d2-8` at CA$0.06392,
    1.71 times;
    at four vCPUs both rules take `d2-8` at CA$0.06392.
  - 8 GiB:
    `d2-8` at CA$0.06392 becomes `r3-16` at CA$0.12209 at one or two vCPUs,
    1.91 times,
    and `b3-16` at CA$0.19135 at four,
    2.99 times.
  - 16 GiB:
    `r3-16` at CA$0.12209 becomes `r2-30` at CA$0.19252 at one or two vCPUs,
    1.58 times,
    and `b3-16` at CA$0.19135 becomes `r3-32` at CA$0.23955 at four,
    1.25 times.
  - 32 GiB:
    `r3-32` at CA$0.23955 becomes `r2-60` at CA$0.37042 at any of those vCPU counts,
    1.55 times.

  Gives up:
  on every power of two the machine is the next size up,
  so typing 8 buys 16,000 MiB for 2.99 times what the size called 8 costs,
  and the printed line still calls the machine's own number "GiB".
- **The floor means the size the provider calls by that number**
  (built on a cloud backend).
  Mechanism:
  the typed number is compared straight with the catalogue's number,
  which is what both cloud backends do today,
  and libvirt keeps its `M × 1,024` MiB domain,
  so one typed 8 is 8,000 MiB on a cloud and 8,192 MiB here.
  Result,
  the same five asks at the same vCPU counts:
  `d2-2` at CA$0.02102,
  `d2-4` at CA$0.03732,
  `d2-8` at CA$0.06392,
  `r3-16` at CA$0.12209,
  `r3-32` at CA$0.23955.
  Gives up:
  every machine is 2.34 percent below the number asked before the guest's own reserve,
  so 8 GiB buys a guest of about 7,752 MiB,
  and the local backend gives 8,192 MiB for the same typed number,
  which is the 2.34 percent disagreement between backends that exists today.

Ranking:
a strict floor > the provider's label.
A strict floor delivers a machine at or above the number a person typed,
on both kinds of backend,
which is what a floor says,
and it holds both builds the record measured:
its 16,000 MiB machine holds the default-parallelism build with about 7,772 MiB to spare.
The label rule buys 8,000 MiB for the typed 8,
which holds the recipe's two-job arms by 200 to 518 MiB
and is about 103 MiB short of the build that ran at default parallelism,
and the strict floor costs 1.25 to 2.99 times the label rule's price on the asks where the two differ,
so the pair is margin against money and the owner's answer decides it.

The other tools,
for the same question,
on pages fetched 2026-11-28T08:00:39-05:00:

- **Multipass**
  takes the sizes a person types and its own suffixes:

  > -m,
  > --memory <memory> Amount of memory to allocate.
  > Positive integers,
  > in bytes,
  > or decimals,
  > with K,
  > M,
  > G suffix.

so its `G` is the suffix a person writes,
and its default is `1G`.
- **OpenStack's client**
  filters in the units the API uses:

  > --min-disk <min-disk> Filters the flavors by a minimum disk space,
  > in GiB.

and

  > --min-ram <min-ram> Filters the flavors by a minimum RAM,
  > in MiB.

so one command uses two units and names each.
- **SkyPilot**
  spells the unit and then renames the binary one:

  > `<num> : exactly <num> GB`

with

  > GB (gigabytes,
  > 2^30 bytes) (default if not specified)

so its `GB` is a gibibyte under the usual name,
which is the strongest argument in these pages for never leaving a unit unstated.
- **Compute Engine**
  takes memory in MB and refuses what does not fit its grid:

  > In the Compute Engine API,
  > you must provide memory in MB as an integer multiple of 256 MB (or 1,024 MB for G2 VMs).

and its sample code throws `Invalid number of cores requested.`
- **EC2**
  takes vCPUs as integers with a minimum and a maximum and never a unit,
  which is the one reading here that has no unit problem at all,
  because it never states memory in a size attribute.

A yes to a strict floor leaves undecided:
whether the printed size names the machine's own number or the gibibytes asked,
whether Hetzner's documented GB is read the same way,
and whether a suffix such as `8G` is accepted at all.

A yes to the provider's label leaves undecided:
whether libvirt's 8,192 MiB default moves to 8,000 MiB,
whether the printed line says the machine is below the number asked,
and whether a suffix in gibibytes is refused by grammar or by an error.

### 7. What does a create without the numbers get?

**The question**,
for anyone reading only this section:
`mvm create dev` with no `--vcpus` and no `--memory`,
and the same command through `mvm run` and the MCP tools.
Every one of those spends money on a cloud backend,
and none of them carries a size unless the caller writes one.

Who calls it without numbers is counted,
not assumed.
No code outside `package/mvm` and `package/mcp` passes `--vcpus`,
`--memory`,
`--server-type` or `MVM_OVH_FLAVOR` to `mvm`,
and the guest drivers that start the rounds name no size,
so a round that says nothing gets the backend's default.
The one place in the repository that names a size for a round is the guest driver's own documented environment,
`MVM_OVH_FLAVOR=b3-16` in `package/git-policy/cli/e2e/README.md`,
which is how the record's first OVHcloud arm ran on four cores and 16,000 MiB.

- **Today's per-backend defaults**
  (what is built).
  Mechanism:
  a create with neither number follows the path that existed before the options:
  libvirt gets 4 vCPUs and 8,192 MiB,
  OVHcloud gets the smallest in-stock flavour of any family,
  Hetzner gets the cheapest offered non-deprecated type.
  Result:
  `mvm create dev` on OVHcloud makes `d2-2` at whole CA$0.02102 an hour,
  a 1-vCPU,
  2,000 MiB machine whose guest holds about 1,846 MiB on the split,
  which cannot hold the 7.3 GiB build;
  on libvirt the same command makes the 8,192 MiB guest the rounds use today,
  which bills nothing.
  Gives up:
  the same command gives 8,192 MiB here and 2,000 MiB there,
  so a guest round that forgets the numbers dies in the crate's build,
  about ten minutes into a billed round,
  and the failure reads as the work's own.
  A round's whole life in the record is 1,238 seconds,
  or about 20.6 minutes,
  which at `d2-2` costs CA$0.00723 and at `d2-8` CA$0.02198.
- **One default everywhere at 4 vCPUs and 8 GiB**
  (recommended).
  Mechanism:
  a create with neither number is given those two floors and the backend's own order picks among the sizes that fit.
  Result:
  on OVHcloud the default ends on `b3-16` at whole CA$0.19135 under a strict floor
  and on `d2-8` at CA$0.06392 under the provider's label (Decision 6),
  so a whole round costs CA$0.06580 or CA$0.02198;
  on libvirt nothing changes;
  on Hetzner the built rule picks the smallest shared-vCPU type with at least 4 cores and 8 GB,
  at a price this round did not read.
  Gives up:
  every numberless create pays 3.04 to 9.10 times the smallest size,
  including a one-shot `run` of a command that needs none of it.
- **No default on a cloud.**
  Mechanism:
  a cloud create with neither number is refused,
  naming the two options and the smallest size each would take.
  Result:
  the command exits nonzero with the two options named,
  and nothing is billed:

  ```text
  no size is chosen for a create without --vcpus or --memory: ask with both floors,
  or name a size with --server-type
  ```

  That block is written by hand for this proposal;
  no such command exists today.
  Gives up:
  every existing script and every MCP call that creates without numbers breaks,
  which includes `mvm run` and `run_in_vm` on libvirt if the refusal is not exempted there,
  and the person must choose a size for a one-line command.

Ranking:
one default > no default > today's defaults.
The workload ranked for here is the guest rounds,
which build this crate,
and a round that says nothing gets `d2-2` on a cloud today,
which cannot hold the build.
One default makes such a round build at 3.04 to 9.10 times the smallest size,
which is CA$0.02198 to CA$0.06580 for a whole round.
A refusal costs an immediate message and one answered question,
and never bills a machine that cannot finish,
so it ranks above a default that kills a round after billed minutes.
Today's defaults rank last for the guest rounds:
the local backend's 8,192 MiB guest works and a cloud one does not,
so the same command means two different workloads,
and the record's own first arm had to name `b3-16` to run at all.

A yes to today's defaults leaves undecided:
whether the cloud default becomes a size the workload can use,
whether `mvm run` keeps its backend default or gets one size,
and whether the printed line warns when the guest is smaller than the build needs.

A yes to one default leaves undecided:
the number itself,
what `run` and `run_in_vm` do with it,
and the Hetzner price it costs.

A yes to a refusal leaves undecided:
which paths are exempt,
and what error a `run` gives an agent that cannot answer.

## What each command does

`mvm create NAME --vcpus N --memory M`:
resolves the size through the steps in "What a request is,
and the sizes it ends on",
then the image for that size,
prints the create's line from Decision 3,
and creates.
On libvirt the guest gets exactly the numbers;
on OVHcloud the cheapest fitting size with a published price;
on Hetzner the smallest shared-vCPU type that fits.
With `--server-type`,
`MVM_OVH_FLAVOR` or `MVM_HCLOUD_SERVER_TYPE` beside either number,
the command is refused.
With neither number it follows Decision 7.

`mvm run -- <command>`:
takes the same two options.
On every backend they reach the guest's create since merge `95f65ed945`:
`ovhRun` and `hetznerRun` pass `vcpus` and `memoryGiB` into `ephemeralRun`,
which hands them to the same create (`package/mvm/cli/src/backend/ovh/exec.ts`,
`package/mvm/cli/src/backend/hetzner/exec.ts`,
`package/mvm/cli/src/backend/ephemeral-run.ts`).
So a one-shot `run` with the numbers makes a sized machine on OVHcloud and Hetzner today,
and issue 6664,
which recorded the opposite,
is closed.
With `--from` it follows the clone rule below,
and a floor beside a source machine is refused by `ephemeralRun` itself.

The MCP server:
`create_vm` and `run_in_vm` take `vcpus` and `memory`,
and `run_in_vm` hands them to the same backend `run` as the CLI.
The tool's own description now names `server_type` as a size for any backend,
but the argument's description still reads "Hetzner server type (e.g. cx23);
... ignored by the libvirt backend",
which is stale;
and `create_vm` refuses a floor beside `from` in its own words before any backend is resolved.
Both answers name no size,
which Decision 3's first option would change.

`create --from SOURCE`:
either number beside `--from` is refused,
and the refusal gives its reason:
a clone keeps its source's size.
That reason is not yet true.
The libvirt clone writes a fresh domain with 4 vCPUs and 8,192 MiB;
the OVHcloud clone resolves the region's smallest in-stock flavour,
or the one `MVM_OVH_FLAVOR` names,
and never calls the image check;
the Hetzner clone takes the cheapest type of the source's architecture.
So today a clone of a large source is **predicted**
to be a smaller machine on OVHcloud and **measured**
to be a different machine on libvirt,
and a clone of a large source onto a small size is the case nobody has run:
the OVHcloud clone boots a snapshot,
and whether OVHcloud refuses a snapshot whose disk is larger than the flavour's disk is not in this repository.
A clone made to the source's recorded size would also have to say what `MVM_OVH_FLAVOR` means beside it,
because that variable names one size for a whole run and the source names another,
and today the variable wins without saying so.
Two consequences follow for this file's own words:
the refusal's reason has to become true (a clone made to the source's recorded size) or the refusal has to say the truth,
and a clone that must take the source's size needs an answer for that size being out of stock,
which is Decision 2's refusal.

The region fallback:
the default list is the project's own Canadian regions that are up and offer an instance service,
in name order;
`--location` or `MVM_OVH_REGIONS` replaces it.
A region whose stock holds no fitting size is skipped with the reason logged,
and the first region with a fitting size is used.
Under the cheapest order the size is chosen within that region and not across regions,
so a later region with a cheaper fit is not consulted;
a cross-region price search is a different loop and would need its own decision.

`mvm list`:
prints each machine's size on all three backends,
as stated in "What is built today".

`mvm --backend ovh update`:
keeps reporting each region's smallest in-stock flavour,
its image count and the instance quota,
which is no longer the size a create takes;
the create's own line is what answers "what would this create take".

## Combinations that cannot be built

True impossibilities,
whatever the owner answers:

- **The cheapest order on libvirt.**
  libvirt has no price and allocates exactly,
  so there is nothing to compare and the order can never apply.
- **One default under exact numbers**
  on a cloud.
  A default of 4 vCPUs and 8 GiB read as exact asks for a flavour whose memory number is exactly 8,
  which is 8,000 MiB,
  so the machine is `d2-8` at CA$0.06392;
  read as exactly 8 GiB,
  no size in `BHS5` has it,
  so a numberless cloud create would refuse every time.
  Which of the two an exact default means is Decision 6's question,
  and only the second cannot be built.
- **A suffix in the other unit.**
  One grammar,
  one unit:
  under gibibytes,
  `8G` is either refused or means 8 GiB,
  and under the provider's label,
  `8GiB` is the other.

Choices,
not impossibilities,
each refused or excluded today:

- **`--vcpus` or `--memory` beside `--server-type`.**
  Refused today,
  because two names for one size can disagree.
  It is not impossible:
  a named type could win,
  or the numbers could check that it fits,
  and a yes to either would need a new answer for what the create does when they disagree.
- **`--vcpus` or `--memory` beside `--from`.**
  Refused today,
  with a reason that is not yet true (see "What each command does").
  It is not impossible:
  a clone could take the numbers where the backend can make a new machine the asked size from the source's snapshot or image,
  and "clone then enlarge" is a combination the owner might want.
- **A refusal in Decision 7 with `run` and `run_in_vm` unchanged.**
  Not impossible:
  both take the numbers already and hand them on since merge `95f65ed945`,
  so the refusal can be taken with them as they stand.
- **The cheapest order with a `-flex` twin.**
  Not impossible,
  and not only a question of price:
  no `-flex` name appears in the Canadian price page's product data,
  so the built rule never sees one,
  and the euro table of the blog the file quotes does price the twins,
  lower than the plain size by 4.9 to 10.2 percent on the Gen 3 sizes with more than 50 GB of disk.
  Whether the Canadian catalogue prices them is not yet known;
  a round is reading that catalogue for issue 6666.
  If it does,
  the twins become candidates,
  and the numbers-only fallback already sorts by disk first,
  so a twin with a 50 GB disk sorts ahead of a plain size with a larger one.
- **A guest arm on OVHcloud.**
  Built and run:
  the drivers' switch landed on `main` in merge `832b3e25c2`,
  and the driver `e2e:doctor-zfs-route` ran on one OVHcloud machine of four cores and 16,000 MiB,
  taking 2 minutes 3 seconds and passing two of its three guest tests,
  the third failing on a difference both sides of `main` carry.

## What is not known

- The unit of the catalogue's memory number,
  for every size but `d2-4`.
  That one is measured;
  OVHcloud's guide prints 1,000 times each name's number for eight more sizes,
  and a boot line from one `d2-8` guest would check a second.
  A round is not making that run,
  because no question here turns on it:
  the rule is measured where it matters and agreed by the guide elsewhere.
- Which sizes are in stock in `BHS5` above 2 vCPUs and 8 GiB.
  The 04:48 to 04:50 read lists the ten smallest in stock;
  `d2-8`,
  `c3-8`,
  `b2-15`,
  `b3-16` and `c2-15` appear on the price page and their stock is not in the record.
- Whether the catalogue's `disk` field equals the local disk OVHcloud's blog gives each Gen 3 size.
  The figures here use the blog's table,
  which says the capacity is "still fixed by size,
  just like in our current catalogue".
- Whether the public order catalogue prices an IPv4 addon and a local storage addon under the plan codes the price page shows.
  The page sells both;
  the catalogue's addon list was not read for them,
  and the built read never looks them up (issue 6666).
- Whether the catalogue prices any `-flex` name.
  The Canadian price page carries none,
  the euro table of OVHcloud's blog prices them,
  and the Canadian catalogue was not read for them;
  a round is reading that catalogue for issue 6666.
- Whether a shared-vCPU `d2` machine holds this repository's build in the time a `b3` machine does.
  No build has run on either size;
  a round is timing one on each.
- Any Hetzner price,
  whether Hetzner sells any GPU type,
  and which types its catalogue offers in the locations a create asks for.
  This round made no Hetzner API call.
- What a guest arm's own timings cost on the sizes a create picks,
  since the one arm that has run named `b3-16` through `MVM_OVH_FLAVOR`.
- What an OVHcloud clone of a larger source does,
  which is predicted from source and not run.
- What a second billed create would verify:
  whether a public IPv4 is still attached by default after 2026-10-01,
  and the local disk a Gen 3 flavour carries.
  The unit and `MemTotal` are measured,
  by the two runs the record holds.

## The workload's own open item

The recipe builds at two jobs

> Two compiling jobs:
> the guest has four cores and a linking test binary of this crate needs more than a gigabyte

(`package/zfs/request-to-openzfs-kernel-device/e2e/run-fixture.ts`,
in the step's own comment,
which goes on to say an out-of-memory kill there would look like a compile failure),
which is a statement about this machine's four cores and about a linking binary,
not about a cloud machine's core count.
A cloud guest with 2 or 16 vCPUs would meet a `--jobs 2` the recipe sets and would not use the rest;
nothing in this file changes the recipe,
and a round that moves to OVHcloud should keep `--jobs 2` until a build on that size says otherwise.

## Sources

Fetched for this revision,
by this round:

- `https://www.ovhcloud.com/en-ca/public-cloud/prices/`,
  fetched 2026-11-28T07:52:43-05:00,
  its embedded product data parsed whole:
  the 44 priced VM names in five groups,
  the public IPv4 line,
  the local storage lines,
  the family texts,
  and the IPv4 footnote under the three VM tables.
- `https://docs.ovhcloud.com/en/guides/public-cloud/compute/key-concepts.md`,
  fetched 2026-11-28T07:52:42-05:00:
  the family table,
  its third column,
  and Discovery's description.
- `https://blog.ovhcloud.com/en/posts/public-cloud-pricing-update-october-2026/`,
  fetched 2026-11-28T07:58:15-05:00:
  which lines became separate on 2026-10-01,
  which ranges keep storage included,
  the local disk of each Gen 3 size,
  and the percentage each size's bill rose by.
- `https://docs.ovhcloud.com/en/guides/public-cloud/compute/starting-with-nova.md`,
  fetched 2026-11-28T07:58:15-05:00:
  the captured `openstack flavor list` whose `d2-4` row reads 4000 in its RAM column.
- `https://blog.ovhcloud.com/fr/posts/public-cloud-pricing-model-update-2026/`,
  the first of the two October 2026 pricing articles,
  which states the IPv4 billing line for every Public Cloud VM range.
- The five other tools' pages,
  each fetched 2026-11-28T08:00:39-05:00:
  [multipass-launch][],
  [ec2-attribute-selection][],
  [openstack-flavor][],
  [skypilot-spec][],
  [gce-custom-machine-type][].

Read in this worktree,
at `f1d6eccf93` unless a commit is named:

- `package/mvm/cli/src/backend/size-request.ts`,
  `create.ts`,
  `domain-xml.ts`,
  `cli.ts`,
  `run.ts`,
  `config.ts`,
  `meta.ts`,
  `list.ts`,
  `vm-sizes.ts`,
  `index-parsers-cmds.ts`;
- `package/mvm/cli/src/backend/ovh/`:
  `catalog-prices.ts`,
  `read-price-choice.ts`,
  `flavors.ts`,
  `flavor-families.ts`,
  `provisioning.ts`,
  `lifecycle.ts`,
  `images.ts`,
  `instances.ts`,
  `regions.ts`,
  `update.ts`,
  `types.ts`,
  `exec.ts`;
- `package/mvm/cli/src/backend/hetzner/`:
  `server-types.ts`,
  `types.ts`,
  `lifecycle.ts`,
  `exec.ts`;
- `package/mcp/mvm/src/`:
  `tools-exec.ts`,
  `tools-lifecycle-mutate.ts`,
  `tool-arguments.ts`;
- `package/mvm/cli/README.md`,
  its section "Choosing a machine's size";
- `package/zfs/request-to-openzfs-kernel-device/e2e/run-fixture.ts`,
  the step comments that set `--jobs 2`;
- `doc/planning/mvm-guest-memory-held-for-a-rounds-life.md`,
  the sections "A guest build's job count,
  and one set of guest steps" and "The cells,
  one run each";
- `doc/decision/mvm-gains-an-ovhcloud-backend-billed-by-the-second.md`,
  the sections "A standing yes for billed runs,
  2026-11-28",
  "The size taken among those that fit,
  2026-11-28",
  "The first arm of a wrapper driver on OVHcloud",
  and "What a size's memory number comes to inside the guest";
- `package/git-policy/cli/e2e/README.md`,
  the section "The recipe's first OVHcloud run",
  for the size the arm was given;
- `doc/troubleshooting/desktop-freezes-while-many-agent-runs-thrash-zram-swap.md`,
  the section "The guests' unit at its own memory cap,
  2026-11-28".

The price read's own shape is recorded where its reader can check it:
the doc comment of `package/mvm/cli/src/backend/ovh/catalog-prices.ts` says its shape was

> The shape here is copied from one live answer of that call with `ovhSubsidiary=CA`,
> read on 2026-11-28

and gives the plan code and the currency the answer carried.
That is the producer's own record of a read that succeeded,
and it is in the source rather than in a log.

Each quotation in this document was copied from the fetched text or the file it names and checked against it by a script
before the commit that carries it,
with whitespace collapsed on both sides.
OVHcloud's blog is quoted only for sentences about the model change and storage capacity,
and for the euro figures its flex table prints,
because the blog's own rates are in euros;
the Canadian figures come from the price page alone.

[multipass-launch]: <https://canonical.com/multipass/docs/latest/reference/command-line-interface/launch/>
[ec2-attribute-selection]: <https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/ec2-fleet-attribute-based-instance-type-selection.html>
[openstack-flavor]: <https://docs.openstack.org/python-openstackclient/latest/cli/command-objects/flavor.html>
[skypilot-spec]: <https://docs.skypilot.co/en/latest/reference/yaml-spec.html>
[gce-custom-machine-type]: <https://cloud.google.com/compute/docs/instances/creating-instance-with-custom-machine-type>
[storage-optimized]: <https://docs.ovhcloud.com/en/guides/public-cloud/compute/key-concepts#:~:text=Storage%20Optimized>
