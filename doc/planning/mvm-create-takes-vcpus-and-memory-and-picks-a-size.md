# mvm's create takes a number of vCPUs and an amount of memory, and a cloud backend picks the size

Status:
a proposal for the owner,
unapproved and non-final,
revised on 2026-11-28T06:19:05-05:00 against one gate read.
The gate read was a same-vendor second pass
(Claude Sonnet reading a draft written by a Claude session),
not an independent review:
five High findings,
eight Medium and four Low,
delivered 2026-11-28.
The section titled "The gate read,
and what this revision does with each finding"
lists every finding,
what was checked for it,
and whether the draft changed or the finding was wrong.
Nothing here is built,
no machine was created for it,
and no provider API was called for it.
Only the owner declares it accepted.

## Why this exists

The owner wrote two sentences on 2026-11-28 while a diagnostic run waited,
and both stand in `doc/decision/mvm-gains-an-ovhcloud-backend-billed-by-the-second.md`:

> Please don't pick GPU?

(the section titled `The third answer, and "Please don't pick GPU?"`,
line 229),
and:

> My impression is mvm should natively support passing how many vcpu and how many GBs of RAM
> and auto pick the most relevant size if the provider doesn't support all mix and match..

(the section titled "The fourth question,
left without a choice",
line 268).

The same decision record says what the GPU sentence was about:
a run created an instance with the image "Ubuntu 26.04 - NVIDIA - v595",
a GPU driver image,
because the backend then took the newest image whose name starts with `ubuntu` (lines 202 to 204),
and the flavour was `d2-2`,
"the smallest in stock and no GPU machine" (lines 200 to 201).
The record also says:

> A flavor record carries no field that marks a GPU flavor;

(line 244),
which this revision tests against the provider's own pages and the catalogue read.
The image side was fixed in the same range:
a shorthand never stands for an image tagged `application` (lines 251 to 258,
`package/mvm/cli/src/backend/ovh/images.ts`,
lines 314 to 333).

On the same day the owner chose that guest rounds move onto OVHcloud,
and answered where more memory comes from with:

> I would finish that design and bring its questions to you.

(`doc/decision/owner-answers-of-2026-11-28-guests-swap-drives-removal-and-releases.md`,
the section titled "Where more memory comes from",
line 169).
So this document is the design that answer waits for,
and its questions are the ones put to the owner.

## What mvm does today

Read at `8200df4622`,
and `git diff --stat 8200df4622 HEAD -- package/mvm/cli/src package/mcp/mvm/src` is empty,
so every line cited here is the line on this branch's base.

`mvm create` takes `--from`,
`--image`,
`--server-type` and `--location`
(`package/mvm/cli/src/index-parsers-cmds.ts`,
`--from` lines 58 to 66,
`--image` lines 72 to 81,
`--server-type` lines 87 to 96,
`--location` lines 103 to 112,
the command at lines 128 to 150).
It takes no number of vCPUs and no amount of memory.
The `--server-type` help text says "ignored by the libvirt backend" (line 93).

- The `libvirt` backend gives every guest two constants,
  `DEFAULT_MEMORY_MIB = 8_192` and `DEFAULT_VCPUS = 4`
  (`package/mvm/cli/src/config.ts`,
  lines 38 and 43,
  written into the domain at `package/mvm/cli/src/domain-xml.ts`,
  lines 247 and 251),
  and the README lists them as the defaults ("4 vCPUs",
  "8 GiB RAM",
  `package/mvm/cli/README.md`,
  lines 872 to 877).
  Every other caller of `domainXml` carries them too:
  `create.ts` line 224,
  `clone.ts` line 195,
  `template-linux.ts` line 147,
  `template-windows.ts` line 167,
  `template-windows-virtio.ts` line 57.
- The `hetzner` backend takes a server type by name,
  from `--server-type` or `MVM_HCLOUD_SERVER_TYPE`
  (`package/mvm/cli/src/backend/hetzner/config.ts`,
  lines 135 to 141),
  and otherwise takes the cheapest non-deprecated type offered in the target locations,
  by gross hourly price
  (`package/mvm/cli/src/backend/hetzner/server-types.ts`,
  lines 117 to 170).
- The `ovh` backend takes a flavour by name or identifier,
  from `--server-type` or `MVM_OVH_FLAVOR`
  (`package/mvm/cli/src/backend/ovh/config.ts`,
  lines 379 to 385),
  and otherwise takes the smallest in stock by vCPUs,
  then memory,
  then identifier (`package/mvm/cli/src/backend/ovh/flavors.ts`,
  lines 74 to 92 and 153 to 169),
  preferring `osType` linux.
  On the owner's project that was `d2-2`,
  with 1 vCPU and 2 GiB.

The rest of the surface carries no size at all:

- `mvm run -- <command>` takes only `--from` (lines 286 to 304),
  and its backend call creates a guest with a name and nothing else
  (`package/mvm/cli/src/backend/ephemeral-run.ts`,
  lines 183 to 188;
  the arguments are `command` and `from`,
  `package/mvm/cli/src/backend/types.ts`,
  lines 99 to 102).
- The MCP server's `run_in_vm` takes `command`,
  `from` and `backend`
  (`package/mcp/mvm/src/tools-exec.ts`,
  lines 85 to 114),
  and its own instructions tell callers
  "Prefer `run_in_vm` for one-shot commands"
  (`package/mcp/mvm/src/index.ts`,
  line 45).
  So on a cloud every ephemeral guest is the backend's default size today.
- The MCP `create_vm` tool takes `server_type` and `location`,
  and its description still says they "apply to the hetzner backend only"
  (`package/mcp/mvm/src/tools-lifecycle-mutate.ts`,
  line 38),
  which is stale since the OVHcloud backend landed.
  Its success text names the VM and no size
  ("VM <name> created and started",
  lines 96 to 98).
- `mvm create --from SOURCE` dispatches to `backend.clone` and drops `--image`,
  `--server-type` and `--location`
  (`package/mvm/cli/src/cli.ts`,
  lines 165 to 178;
  the MCP tool does the same at lines 81 to 91).
  The libvirt clone writes a fresh domain with the two constants,
  not the source's size (`package/mvm/cli/src/clone.ts`,
  lines 195 to 201).
  The Hetzner clone takes the cheapest type of the source's architecture,
  or `MVM_HCLOUD_SERVER_TYPE`
  (`package/mvm/cli/src/backend/hetzner/lifecycle.ts`,
  lines 380 to 394).
  The OVHcloud clone resolves a flavour from the source's region with only the
  environment override in hand,
  so it takes the smallest in stock or `MVM_OVH_FLAVOR`,
  and it never calls the image check
  (`package/mvm/cli/src/backend/ovh/lifecycle.ts`,
  lines 179 to 195).
  A clone is therefore not the size of its source.
- `mvm list` prints a name and a state
  (`package/mvm/cli/src/list.ts`,
  `VmInfo` at lines 19 to 22;
  `package/mvm/cli/src/cli.ts`,
  lines 196 to 200),
  and no size is stored anywhere for a later command:
  the OVHcloud instance record carries a `flavorId`
  (`package/mvm/cli/src/backend/ovh/types.ts`,
  line 133),
  and the libvirt metadata carries no size.
- `mvm --backend ovh update` reports each Canadian region's smallest in-stock
  flavour by identifier
  (`package/mvm/cli/src/backend/ovh/update.ts`,
  lines 58 to 95).

Billing is the reason the choice costs money rather than time:
the backend bills by the second beyond an initial one-minute minimum,
and a stopped instance still bills until it is deleted
(`package/mvm/cli/README.md`,
lines 585 to 600).

## What the OVHcloud catalogue returned

A read-only read of the owner's project on 2026-11-28 between 04:48 and 04:50 (UTC-05:00).
Region `BHS5`:
182 flavours in seven types.

The ten smallest in-stock Linux flavours,
in the order the backend sorts them
(vCPUs,
then memory,
then name):

- `d2-2`:
  1 vCPU,
  2 GiB,
  disk 25,
  type `ovh.d2`
- `c3-4`:
  2 vCPUs,
  4 GiB,
  disk 50,
  type `ovh.ssd.cpu`
- `c3-4-flex`:
  2 vCPUs,
  4 GiB,
  disk 50,
  type `ovh.ssd.cpu`
- `d2-4`:
  2 vCPUs,
  4 GiB,
  disk 50,
  type `ovh.d2`
- `b2-7`:
  2 vCPUs,
  7 GiB,
  disk 50,
  type `ovh.ssd.eg`
- `b2-7-flex`:
  2 vCPUs,
  7 GiB,
  disk 50,
  type `ovh.ssd.eg`
- `c2-7`:
  2 vCPUs,
  7 GiB,
  disk 50,
  type `ovh.ssd.cpu`
- `c2-7-flex`:
  2 vCPUs,
  7 GiB,
  disk 50,
  type `ovh.ssd.cpu`
- `b3-8`:
  2 vCPUs,
  8 GiB,
  disk 50,
  type `ovh.ssd.eg`
- `b3-8-flex`:
  2 vCPUs,
  8 GiB,
  disk 50,
  type `ovh.ssd.eg`

No size in stock among these has 2 vCPUs and 6 GiB,
which is the owner's "doesn't support all mix and match".
The four `-flex` names carry the same numbers as their twins,
and the record shows nothing that tells them apart but the suffix.

A flavour record carries fourteen keys:
`available`,
`capabilities`,
`disk`,
`id`,
`inboundBandwidth`,
`name`,
`osType`,
`outboundBandwidth`,
`planCodes`,
`quota`,
`ram`,
`region`,
`type` and
`vcpus`.
`planCodes` carries `monthly`,
`hourly` and `license`,
and `hourly` is the plan name a price is looked up by
(`d2-2.consumption` for `d2-2`).
`quota` is "how many more instances of this flavor the project's quota allows".
No field marks a GPU flavour,
and the `capabilities` are the same four on every type read:
`resize`,
`snapshot`,
`volume` and
`failoverip`.

The type counts in `BHS5`:
`ovh.baremetal` 3,
`ovh.d2` 3,
`ovh.iops` 6,
`ovh.raid-nvme.t1` 18,
`ovh.ssd.cpu` 52,
`ovh.ssd.eg` 52,
`ovh.ssd.ram` 48.
`osType` is `baremetal-linux` on the baremetal type and `linux` on `ovh.d2`;
the other five carry `linux` and `windows`.

The region offered 52 images.
The record lists two distinct `flavorType` values,
`null` and the string
`g1,g2,g3,t1,t2,i1,a100,l4,rtx5000,l40s`,
not how many images carry each,
and it does not name the image that carries the string.
The three GPU driver images
("Ubuntu 24.04 - NVIDIA - v580",
"Ubuntu 26.04 - NVIDIA - Latest",
"Ubuntu 26.04 - NVIDIA - v595")
carry `flavorType` null and the tags `application` and `nvidia_ngc`.
Every image states `minDisk` 0 and `minRam` 0.
The image the backend picked before the image fix was "Ubuntu 26.04 - NVIDIA - v595".

## What OVHcloud documents about its families

`https://docs.ovhcloud.com/en/guides/public-cloud/compute/key-concepts/`
(fetched 2026-11-28T06:05:05-05:00,
and its Markdown form
`https://docs.ovhcloud.com/en/guides/public-cloud/compute/key-concepts.md`)
lists the instance families:

> General Purpose | Balanced CPU and Memory | ...
> Compute Optimized | High Processor Performance | ...
> Memory Optimized | High Memory Capacity | ...
> Storage Optimized | High IOPS Performance | Equipped with NVMe storage ...
> Cloud GPU | Hardware-Accelerated Graphics | ...
> Discovery | Cost-Effective,
> Shared Resources | ...
> Metal Instances | Dedicated Resources | ...

(lines 34 to 40 of the Markdown form;
the table's pipes are kept here only to show the page's own shape,
and this document uses no tables elsewhere).

`https://docs.ovhcloud.com/en/guides/public-cloud/compute/deploy-a-gpu-instance/`
(fetched 2026-11-28T06:05:05-05:00) says:

> At the moment,
> most of our old GPU instances (Tesla V100 and V100s) are only available in the GRA7,
> GRA9,
> GRA11,
> and BHS5 regions.
> The newer models (A100,
> H100,
> L4 and L40s) are only available in the GRA11 region for now.

(line 16),
and:

> All the images we offer can be used on a GPU instance.

(line 49),
and:

> We only offer the possibility to install the special images on a few selected models (T1-45,
> T1-90,
> T1-180,
> T2-45,
> T2-90,
> T2-180).

(line 103).
So `BHS5` sells GPU instances,
they are named `T1-*` and `T2-*`,
and they belong to the family the documentation calls `Cloud GPU`.

The same families appear on the provider's price page.
`https://www.ovhcloud.com/en-ca/public-cloud/prices/`
(fetched 2026-11-28T06:02:44-05:00)
groups its instance products under
`iops`,
`cpu`,
`discovery`,
`general-purpose`,
`ram`,
`cloud-gpu` and
`metal`,
under the heading "Virtual Machine Instances",
and the page shows "Cloud GPU" as a family heading.
The `cloud-gpu` group holds 34 names:

`rtx5000-56`,
`t2-180`,
`l40s-180`,
`l40s-360`,
`t2-le-180`,
`t2-le-45`,
`l4-180`,
`rubin-nvl8`,
`l40s-90`,
`a100-720`,
`rtx5000-84`,
`h200-1920`,
`t1-90`,
`t1-45`,
`t1-180`,
`h100-760`,
`t1-le-180`,
`t2-45`,
`a100-360`,
`t2-90`,
`t1-le-90`,
`l4-90`,
`a100-180`,
`l4-360`,
`h100-1520`,
`rtx5000-28`,
`h200-1920-eph`,
`h100-380`,
`a10-90`,
`a10-45`,
`b300-1920`,
`t1-le-45`,
`a10-180`,
`t2-le-90`.

The `iops` group holds `i1-45`,
`i1-90` and `i1-180`,
the storage family the documentation calls Storage Optimized.

Two more documents bear on the `-flex` suffix:

`https://docs.ovhcloud.com/en/guides/public-cloud/compute/resize-instance-manager.md`
(fetched 2026-11-28T06:05:05-05:00):

> Only **Flexible instances** can be upgraded and downgraded while maintaining a fixed disk size of 50GB.

`https://docs.ovhcloud.com/en/guides/public-cloud/compute/revert-a-flex-instance.md`
(fetched at the same time):

> A flex instance is a single size disk (50GB) instance which offers a faster process for snapshots.
> It allows resizing to higher or lower models with a fix storage space,
> whereas classic models can only be resized to higher models.

and:

> The flex option is not available for all models.

So a `-flex` flavour is resizable up and down while keeping its 50 GB disk,
and a classic one only grows.

OVHcloud's instance API documentation
(`https://developers.eu.ovhcloud.com/reference/public-cloud/compute/instances.md`,
fetched 2026-11-28T06:16:00-05:00)
documents a GPU count on the flavour an instance reports,
the field `currentState.flavor.gpus`,
described as:

> Number of physical GPUs provided by the flavor,
> 0 for flavors without GPU

(lines 73 and 173,
and again in the object "List compatible flavors" returns,
line 578).
That field sits on an instance,
and on the flavours that instance can be resized to;
the region's flavour list that `mvm` reads carries no such field,
which is why the rule in the next section uses the family instead.

## Prices

Read from `https://www.ovhcloud.com/en-ca/public-cloud/prices/`
on 2026-11-28T06:02:44-05:00,
Linux,
"Price (excl.
VAT/hour)",
Canadian dollars,
from the product data the page itself carries,
with two rows checked against the rendered page
(`b3-8` at CA$0.082 and `b2-7` at CA$0.1135).
Each price below is the entry whose region list holds `BHS5`,
which every flavour listed here has.

- `d2-2`:
  1 vCPU,
  2 GB,
  25 GB disk,
  CA$0.0167 per hour.
- `d2-4`:
  2 vCPUs,
  4 GB,
  50 GB disk,
  CA$0.033 per hour.
- `c3-4`:
  2 vCPUs,
  4 GB,
  local storage low latency,
  CA$0.0732 per hour.
- `b2-7`:
  2 vCPUs,
  7 GB,
  50 GB SSD,
  CA$0.1135 per hour.
- `c2-7`:
  2 vCPUs,
  7 GB,
  50 GB SSD,
  CA$0.1629 per hour.
- `b3-8`:
  2 vCPUs,
  8 GB,
  local storage low latency,
  CA$0.082 per hour.
- `d2-8`:
  4 vCPUs,
  8 GB,
  50 GB disk,
  CA$0.0596 per hour.
- `c3-8`:
  4 vCPUs,
  8 GB,
  local storage low latency,
  CA$0.1461 per hour.
- `b2-15`:
  4 vCPUs,
  15 GB,
  100 GB SSD,
  CA$0.2148 per hour.
- `b3-16`:
  4 vCPUs,
  16 GB,
  local storage low latency,
  CA$0.1637 per hour.
- `c2-15`:
  4 vCPUs,
  15 GB,
  100 GB SSD,
  CA$0.3162 per hour.

Two things the page does not give:

- No `-flex` name appears in its product data at all,
  so `b2-7-flex`,
  `c2-7-flex`,
  `c3-4-flex` and `b3-8-flex` have no published
  hourly price on the page a person would read.
  The page is the provider's own public price list;
  this document does not guess what a flex twin costs.
- The page's `GB` is the same number the catalogue reports as `ram`,
  and `mvm`'s type comment reads that number as GiB
  (`package/mvm/cli/src/backend/ovh/types.ts`,
  line 307).
  The two readings differ by 7.4 percent
  (8 GiB is 8.59 GB,
  and 8 GB is 7.45 GiB),
  and the section titled
  "4.
  Does a bare `--memory 8` mean 8 GiB or 8 GB?"
  turns that into a decision.

What a wrong pick costs,
per hour,
on these prices:

- `c2-7` instead of `b2-7`,
  at the same 2 vCPUs and 7 GB:
  CA$0.0494 more per hour,
  43.5 percent more.
- `b2-7` instead of `b3-8`,
  for one GB less memory:
  CA$0.0315 more per hour,
  38.4 percent more.
- `b3-8` instead of `d2-8`,
  for 2 fewer vCPUs and the same 8 GB:
  CA$0.0224 more per hour,
  37.6 percent more.
- `b2-7` instead of `d2-8`,
  for one GB less memory and 2 fewer vCPUs:
  CA$0.0539 more per hour,
  90.4 percent more.
- `c3-4` instead of `d2-4`,
  at the same 2 vCPUs and 4 GB:
  CA$0.0402 more per hour,
  121.8 percent more.
- `b2-15` instead of `b3-16`,
  for one GB less memory:
  CA$0.0511 more per hour,
  31.2 percent more.
- the smallest size,
  `d2-2` at CA$0.0167 per hour,
  against `b3-16` at
  CA$0.1637:
  9.80 times,
  CA$0.1470 more per hour,
  or against `d2-8` at CA$0.0596:
  3.57 times,
  CA$0.0429 more per hour,
  and `d2-2` cannot hold the build this repository's guest rounds run
  (the section titled "The workload on record").

## What Hetzner's documented server type object carries

`https://docs.hetzner.cloud/cloud.spec.json`
(fetched 2026-11-28T06:08:00-05:00)
describes the object `GET /server_types` returns.
Its fields,
with the file's own descriptions:

- `id`,
  an integer,
  "ID of the Server type."
- `name`,
  a string,
  "Unique identifier of the Server type."
  (`cpx22`)
- `description`,
  a string (`CPX22`)
- `cores`,
  a number,
  "Number of cpu cores a Server of this type will have."
- `memory`,
  a number,
  "Memory a Server of this type will have in GB."
- `disk`,
  a number,
  "Disk size a Server of this type will have in GB."
- `prices`,
  per location:
  `price_hourly` and `price_monthly`,
  each with `net` ("Price without VAT.")
  and `gross` ("Price with VAT added."),
  plus `included_traffic` and `price_per_tb_traffic`.
- `storage_type`,
  an enum of `local` and `network`,
  "Type of Server boot drive.
  Local has higher speed.
  Network has better availability."
- `cpu_type`,
  an enum of `shared` and `dedicated`,
  "Type of cpu."
- `category`,
  a string,
  "Category of Server Type.",
  example `Shared vCPU`.
- `architecture`,
  an enum of `x86` and `arm`.
- `deprecation`,
  marked deprecated in favour of `locations[].deprecation`.
- `locations`,
  each with `id`,
  `name`,
  `deprecation`,
  `recommended` and `available`,
  where a type is
  "temporarily unavailable in the Location when the `deprecation.available` is false"
  and `available` itself is described as
  "Whether the Server Type is temporarily unavailable in this Location."

What the backend reads today
(`package/mvm/cli/src/backend/hetzner/types.ts`,
lines 91 to 129)
is `name`,
`architecture`,
the deprecated top-level `deprecation`,
and `prices[].location` with `prices[].price_hourly.gross`.
It reads neither `cores` nor `memory`,
which is why a default in vCPUs and memory cannot be resolved on Hetzner
without adding those two reads.

What the same question looks like on Hetzner,
"what keeps special hardware out":
the documented object carries no field that marks a GPU,
and no enumerated family:
`category` is free text with one example,
and neither `storage_type`,
`cpu_type` nor `architecture` names hardware classes
as `Cloud GPU` does on OVHcloud.
An allow-list of `category` strings would rest on free text the documentation
does not enumerate.
Whether Hetzner Cloud sells any GPU server type is not in this document:
reading it needs a catalogue call,
and this round made none.
Today's backend takes the cheapest non-deprecated type across the locations
by gross hourly price,
which is a price floor,
not a hardware guard.

## How other tools take a size

Each page was fetched on 2026-11-28,
each sentence is copied from the fetched page,
and each link carries a `#:~:text=` fragment that was opened in
headless Chromium through `agent-browser` on 2026-11-28:
the page with the fragment scrolled to the text,
and the same page without it,
and with a deliberately wrong fragment,
did not.

**Multipass** allocates exactly what is asked,
on its own hypervisor:

> `-c, --cpus <cpus>` Number of CPUs to allocate.
> Minimum:
> 1,
> default: 1.

and

> `-m, --memory <memory>` Amount of memory to allocate.
> Positive integers,
> in bytes,
> or decimals,
> with K,
> M,
> G suffix.
> Minimum:
> 512M,
> default:
> 1G.

(<https://canonical.com/multipass/docs/latest/reference/command-line-interface/launch/#:~:text=Number%20of%20CPUs%20to%20allocate>,
<https://canonical.com/multipass/docs/latest/reference/command-line-interface/launch/#:~:text=Amount%20of%20memory%20to%20allocate>).
A size no machine has exactly does not arise:
the guest is made to the asked size,
and a value below the stated minimum is refused.

**Amazon EC2** calls the cloud case attribute-based instance type selection:

> As an alternative to manually specifying the instance types,
> you can specify the attributes that an instance must have,
> and Amazon EC2 will identify all the instance types with those attributes.

and

> Price protection is a feature that prevents your EC2 Fleet or Spot Fleet from using instance types that you would consider too expensive even if they happen to fit the attributes that you specified.

and

> For On-Demand Instances,
> attribute-based instance type selection supports the lowest price allocation strategy.

(<https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/ec2-fleet-attribute-based-instance-type-selection.html#:~:text=you%20can%20specify%20the%20attributes%20that%20an%20instance%20must%20have>,
<https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/ec2-fleet-attribute-based-instance-type-selection.html#:~:text=even%20if%20they%20happen%20to%20fit%20the%20attributes%20that%20you%20specified>,
and the page's section "Preview instance types with specified attributes").
An ask no type has exactly is met by the fitting types it does have,
the cheapest first,
and the page offers a preview of the types an attribute set would match.

**OpenStack's client**,
the family OVHcloud's Public Cloud belongs to,
filters rather than chooses:

> `--min-disk <min-disk>` Filters the flavors by a minimum disk space,
> in GiB.

and

> `--min-ram <min-ram>` Filters the flavors by a minimum RAM,
> in MiB.

(<https://docs.openstack.org/python-openstackclient/latest/cli/command-objects/flavor.html#:~:text=Filters%20the%20flavors%20by%20a%20minimum%20RAM>).
It filters the output of `flavor list`;
`server create` takes one named `--flavor`,
so the client itself picks nothing,
and an ask no flavour has exactly narrows the list and stops there.

**SkyPilot** makes the two readings two spellings:

> `<count>` :
> exactly `<count>` vCPUs
> `<count>+` :
> at least `<count>` vCPUs

and

> Example:
> 4+ means first try to find an instance type with >= 4 vCPUs.
> If not found,
> use the next cheapest instance with more than 4 vCPUs.

(<https://docs.skypilot.co/en/latest/reference/yaml-spec.html#:~:text=first%20try%20to%20find%20an%20instance%20type%20with>).
An ask no type has exactly is met by the cheapest type at or above it
for the `+` spelling,
and refused for the exact spelling.

**Google Compute Engine** builds the size instead of choosing one:

> Specifically,
> you can create a VM instance with a custom number of vCPUs and amount of memory,
> effectively using a custom machine type.

(<https://cloud.google.com/compute/docs/instances/creating-instance-with-custom-machine-type#:~:text=effectively%20using%20a%20custom%20machine%20type>).
Its sample code refuses a mix the machine series cannot make,
with messages such as `Requested memory must be a multiple of 256 MB.`
and `Invalid number of cores requested. Allowed number of cores for ...`.
An ask no machine has exactly is either made to order within the series' limits
or refused with the limit named.

So of the five,
two allocate or build what is asked (Multipass,
Compute Engine),
one narrows a list (OpenStack),
one picks by lowest price within attributes and offers a preview (EC2),
and one spells exact and at-least separately and picks the cheapest (SkyPilot).
None of them reads an ask as "the least size with at least those numbers"
except OpenStack's filter,
which chooses nothing.

## The rule that keeps GPU flavours out

The rule is an allow-list of flavour families,
stated as a mechanism rather than a promise:

- A flavour is a candidate only when its `type` is one of
  `ovh.d2`,
  `ovh.iops`,
  `ovh.ssd.cpu`,
  `ovh.ssd.eg` or
  `ovh.ssd.ram`.
- The list is written in the source.
  It is the five families OVHcloud documents as
  Discovery,
  Storage Optimized,
  Compute Optimized,
  General Purpose and Memory Optimized,
  and their type strings are the ones the catalogue reports for them.
- A flavour of any other type is not a candidate,
  including every family the provider adds later,
  so a new GPU family cannot be picked by accident.

Tested by hand against the real list in `BHS5`:

- It drops all 18 flavours of `ovh.raid-nvme.t1`,
  whose names begin `t1-` and `t1-le-` in the six the record shows
  (`t1-180`,
  `t1-45`,
  `t1-90`,
  `t1-le-180`,
  `t1-le-45`,
  `t1-le-90`).
  The provider files that family under `Cloud GPU`,
  and its documentation names `T1-45`,
  `T1-90`,
  `T1-180`,
  `T2-45`,
  `T2-90` and `T2-180` as the GPU models.
- It drops all 3 flavours of `ovh.baremetal`
  (`bm-s1`,
  `bm-l1`,
  `bm-m1`),
  which today's Linux preference already drops,
  because their `osType` is `baremetal-linux`.
- It keeps the 6 flavours of `ovh.iops`
  (`i1-45`,
  `i1-90`,
  `i1-180` and their three `win-` names),
  the family the documentation calls Storage Optimized,
  which is NVMe local storage and no GPU.
  A person who wants local NVMe therefore keeps it.
- It keeps the 155 flavours of `ovh.d2`,
  `ovh.ssd.cpu`,
  `ovh.ssd.eg` and `ovh.ssd.ram`,
  which is 3,
  52,
  52 and 48.

Does it drop a flavour a person would want?
It drops the GPU models,
which is the owner's sentence,
and it drops the baremetal metal instances,
which cannot boot a normal image.
The family a person might miss,
`i1-*`,
is kept.

What it costs:
a new general-purpose family,
or a region that reports different type strings,
yields no candidate,
and the create refuses rather than picks something larger.
The list needs an edit before such a family can be used.

Why not the two alternatives:

- **The families an image reserves**,
  which the previous draft proposed,
  reads one image's `flavorType` and drops the flavours whose names open with
  those families.
  It fails by construction in two ways.
  The string it reads names no `h100`,
  `a10`,
  `h200`,
  `b300` or `rubin`,
  and the provider's price page files all five under `Cloud GPU`,
  so a region selling them would offer them as candidates.
  And the record does not name the image that carries the string,
  while the three GPU driver images carry `null`,
  so the guarantee rests on an image nobody has identified.
  It also drops `i1-*`,
  the storage family a person would want.
  `flavorType` is an image's list of families it may boot on
  (`package/mvm/cli/src/backend/ovh/images.ts`,
  lines 200 to 228),
  not a list of families held back from other images.
- **A list of names in the source**
  drops exactly the names on it.
  Written from the price page today it would name `h100`,
  `a10`,
  `l4`,
  `l40s`,
  `a100`,
  `rtx5000`,
  `t1`,
  `t2`,
  `h200`,
  `b300` and `rubin`;
  a family the provider adds tomorrow is admitted until someone edits the list,
  which is the fail-open direction the owner's sentence rules out.

A flavour named whole with `--server-type` is taken as named under every rule,
GPU or not,
as today.

## The sizes the examples use, and what each rule ends with

The candidate steps,
in order,
and each one's effect in `BHS5`:

1.  In stock:
    `available` is true.
    The record's ten are all in stock.
2.  Linux:
    `osType` is `linux`.
    This drops the three `win-` names of `ovh.iops` and the baremetal type.
3.  Family:
    `type` is on the allow-list (the section titled
    "The rule that keeps GPU flavours out").
    This drops 18 GPU flavours and 3 metal ones.
4.  Classic,
    not `-flex`:
    a name ending `-flex` is not a candidate.
    This drops `c3-4-flex`,
    `b2-7-flex`,
    `c2-7-flex` and `b3-8-flex`,
    for the reason in the section titled "Prices":
    the provider's public price page publishes no hourly price for them,
    so a rule that picks by price could not state what it picked,
    and the resize the suffix buys is a later operation the workload does not
    perform.
    This step is a stated choice,
    not a measurement,
    and the owner may veto it;
    a create that cannot use a flex twin falls to the next size the rules pick.
5.  The image:
    the flavour's disk and memory reach the chosen image's floors
    (`minDisk` and `minRam`,
    0 on every image in `BHS5`).
6.  Quota:
    the flavour's `quota` is above 0,
    and the project's per-region core and memory quota
    (`package/mvm/cli/README.md`,
    lines 682 to 686) has room.

Two asks are worked through.
The record's stock list covers only the ten smallest,
so an answer that lands on a larger size says that its stock is not in the record.

### The owner's ask: 2 vCPUs and 6 GB

The candidates after the steps are `d2-2`,
`c3-4`,
`d2-4`,
`b2-7`,
`c2-7` and `b3-8`,
plus larger sizes the record does not list.

- At least 2 vCPUs and 6 GB,
  then the fewest numbers:
  the 7 GB sizes are the least that fit,
  `b2-7` and `c2-7` tie on numbers and disk,
  and the name order takes `b2-7` at CA$0.1135 per hour.
  The flex twins would have made it four,
  and the name order would still have taken `b2-7`.
- At least 2 vCPUs and 6 GB,
  then the cheapest:
  on the price page `d2-8` (4 vCPUs,
  8 GB,
  CA$0.0596 per hour) is the cheapest size that fits,
  and among the ten the record shows in stock `b3-8` (2 vCPUs,
  8 GB,
  CA$0.082) is.
  `d2-8` has 2 more vCPUs than the ask and costs CA$0.0224 per hour less than
  `b3-8`,
  which itself costs CA$0.0315 less than `b2-7`.
- Exactly 2 vCPUs and 6 GB:
  refused,
  because no size has 6 GB.
  The nearest by vCPUs then memory are `b2-7` and `c2-7` at 1 GB above,
  and the sizes at least as large are those two plus `b3-8`,
  `d2-8` and larger ones.
- The nearest in either direction:
  `b2-7` and `c2-7` tie at 1 GB above,
  the price rule then takes `b2-7` at CA$0.1135 per hour,
  the name rule also takes `b2-7`.

The money in the example:
the cheapest fitting size on the price page costs CA$0.0596 per hour (`d2-8`),
the cheapest the record shows in stock CA$0.082 (`b3-8`),
and the fewest-numbers pick CA$0.1135 (`b2-7`),
which is 37.6 to 90.4 percent more than those two.

### The default ask: 4 vCPUs and 8 GiB

Read as GiB,
8 GiB is 8.59 GB,
so an 8 GB flavour is not enough.
The record lists no 4-vCPU size among the ten smallest in stock,
so these sizes come from the price page and their stock is not in the record.

- At least 4 vCPUs and 8 GiB,
  then the fewest numbers:
  `b2-15` (4 vCPUs,
  15 GB) at CA$0.2148 per hour.
- At least 4 vCPUs and 8 GiB,
  then the cheapest:
  `b3-16` (4 vCPUs,
  16 GB) at CA$0.1637 per hour.
  The two differ by CA$0.0511 per hour,
  and the cheaper one has 1 GB more.

Read as GB,
8 GB is the number the catalogue reports,
and the answer is `d2-8` (4 vCPUs,
8 GB) at CA$0.0596 per hour under both rules,
because it is both the least that fits and the cheapest.

The distance between the two readings is the whole of the question in the section
titled "4.
Does a bare `--memory 8` mean 8 GiB or 8 GB?":
one reading lands on CA$0.0596 per hour and 7.45 GiB,
the other on CA$0.1637 per hour and 14.9 GiB,
2.75 times as much.

## The host cap, and what a memory ask means on libvirt

On this machine,
every guest `mvm` starts runs inside
`mvm-libvirt-session.service`,
and `mvm` sets that unit's `MemoryMax` from `/proc/meminfo` when it starts:
half of the machine's total,
33,109,835,776 bytes here
(`package/mvm/cli/src/session-daemon-memory.ts`,
and `package/mvm/cli/README.md`,
lines 219 to 223).

On 2026-11-28 the owner raised it to three quarters of the machine's memory
and applied it to the running unit at 05:45:26,
moving `memory.max` from 33,109,835,776 to 49,664,753,664 bytes
(`doc/troubleshooting/desktop-freezes-while-many-agent-runs-thrash-zram-swap.md`,
the section titled "The guests' unit at its own memory cap,
2026-11-28",
and `doc/decision/owner-answers-of-2026-11-28-guests-swap-drives-removal-and-releases.md`,
the section titled "The guests' memory:
the unit's cap,
builds outside the guest,
and where more memory comes from").
That change was applied at runtime,
so the source still computes half,
and the cap returns to half when the unit next starts unless the source changes.

Four things follow for an ask:

- The cap belongs to the whole unit,
  not to each guest.
  Eight guests that had between them touched 58.2 GiB sat in a unit allowed
  30.8 GiB,
  and the unit swapped against its own limit while the machine had
  memory free (same section of the troubleshooting record).
  A single guest asking 32 GB is below three quarters of this machine,
  and two such guests are above it.
- A guest is charged what it touches,
  not what its domain asks:
  a domain asking 8 GiB charged a peak of 990 MB in the 2026-11-21 measurement
  (`package/mvm/cli/README.md`,
  lines 226 to 230).
  The ask therefore sets the domain's ceiling,
  and the guest's own work decides what the unit pays.
- When the unit exceeds its cap the kernel kills a process inside it,
  usually the largest guest,
  and `OOMPolicy=continue` keeps that from stopping the unit
  (`package/mvm/cli/README.md`,
  lines 234 to 238).
- An ask above the unit's cap can never be satisfied,
  so `--memory` refuses a value above three quarters of the machine's total,
  naming the cap and the file it came from.
  It does not refuse the sum of several guests,
  because it cannot see the guests other sessions run in the same unit,
  and it says so in that refusal.

## The workload on record

The work these sizes are for is the wrapper's guest rounds:
the recipe creates a guest with `mvm --backend libvirt create` and no size
(`package/zfs/request-to-openzfs-kernel-device/e2e/run-fixture.ts`,
lines 360 to 375),
the drivers name `--backend libvirt` for `exec`,
`push` and `pull`
(`package/git-policy/cli/bin/mvm-guest-detached-script.ts`,
lines 212 and 314),
and the recipe's build runs two jobs "because the guest has four cores"
(`run-fixture.ts`,
lines 741 to 745).
The measured memory is what the guest must hold:

> one release rustc of this crate's tests takes 6.7 to 6.9 GiB of the guest's 8 GiB

and issue 6611's round

> measured one at 7,019,956 kB of the guest's 7,755 MiB while four jobs reached 7,701,208 kB and the guest's killer ended the build

(`doc/planning/mvm-guest-memory-held-for-a-rounds-life.md`,
lines 491 to 495).

So the size that holds this work is 8 GiB,
and 4 vCPUs is the count the recipe assumed.
The record also notes one `rustc` at 7,355,484 kB (7.01 GiB) in another arm
(lines 366 to 372).
The cloud sizes in this document are therefore judged by one test:
does the guest hold 8 GiB,
and does it have 4 vCPUs,
at the lowest hourly price that does.

The record says nothing about which sizes the guest arms need on a cloud
machine,
because none has run there.

## The decisions

Six decisions.
Each is written to stand on its own,
with its options' mechanism,
a visible result on the prices in the section titled "Prices",
what each option gives up,
and a ranking with a reason for each adjacent pair.
A ranking is by what each option delivers for the workload on record,
never by how much code it needs.

What is fixed before these decisions and not asked again:

- The flavour rule:
  the family allow-list in the section titled
  "The rule that keeps GPU flavours out".
- The candidate steps,
  including the `-flex` drop,
  in the section titled "The sizes the examples use".
- `--server-type` keeps meaning one flavour by its provider name,
  and beside `--vcpus` or `--memory` it is refused,
  because two sources for one size can disagree.
- A size asked for on `libvirt` allocates exactly,
  within the host cap,
  because there is nothing to choose.
- Disk size is not part of this:
  a cloud size fixes its disk,
  and `libvirt` has its own 20 GiB default.

### 1. Does a bare number mean at least, or exactly?

What is being decided:
`mvm create --vcpus 2 --memory 6 dev` on `BHS5`,
where no size has 2 vCPUs and 6 GB.
The other reading stays available either way,
spelled on the number itself,
so both are always reachable and the question is only which spelling is bare.

- **At least.**
  A bare number is a floor.
  The backend keeps the sizes with at least that many vCPUs and that much memory,
  and the choice among them is the decision in the section titled
  "3.
  Among the sizes that fit,
  take the cheapest,
  the smallest numbers,
  or refuse?".
  `--memory 6` keeps,
  among others,
  `b2-7`,
  `c2-7`,
  `b3-8` and `d2-8`;
  with the cheapest choice it ends at `d2-8` (4 vCPUs,
  8 GB,
  CA$0.0596 per hour),
  and among the sizes the record shows in stock at `b3-8` (2 vCPUs,
  8 GB,
  CA$0.082 per hour),
  with the fewest-numbers choice at `b2-7` (2 vCPUs,
  7 GB,
  CA$0.1135 per hour).
  A trailing `=` asks for one exact size.
  Gives up:
  the guest can be larger than the ask,
  and the bill with it.
- **Exactly.**
  A bare number is exact.
  `--memory 6` asks for 6 GB and refuses when no size has it,
  unless the decision in the section titled
  "2.
  When no size meets the ask,
  refuse or take the nearest?"
  takes the nearest,
  which here ends at `b2-7` or `c2-7` (2 vCPUs,
  7 GB,
  CA$0.1135 or CA$0.1629).
  A trailing `+` asks for a floor.
  Gives up:
  on a provider with fixed mixes most asks are refused,
  which is the case the owner's sentence asks `mvm` to handle itself.

Ranking:
at least > exactly.
Only the first gives the build a guest that holds its 7 GiB whenever the provider
has any size that does,
and the second still costs the owner nothing in spelling,
because `=` asks it.
Exactly over at least is not offered,
because it refuses the owner's own example
by default and leaves the work without a machine.

A yes to at least leaves open:
the fallback when nothing meets the floor (the section titled "2.
..."),
the choice among the fitting sizes ("3.
..."),
the unit ("4.
..."),
and whether `+` and `=` are the two markers.

### 2. When no size meets the ask, refuse or take the nearest?

What is being decided:
a command whose ask no size in the region satisfies.
With a floor that happens only above the largest size,
and with an exact ask it happens on almost every command.

- **Refuse and name the nearest.**
  The create prints the sizes closest to the ask
  by vCPUs,
  then memory,
  with their prices,
  says which way each is off,
  and exits nonzero.
  `--vcpus 2 --memory 6` exactly would print `b2-7` (2 vCPUs,
  7 GB,
  CA$0.1135 per hour,
  1 GB above) and `c2-7` (2,
  7,
  CA$0.1629,
  1 GB above).
  It stores nothing and creates nothing.
  Gives up:
  a second command,
  and an agent's `run` fails where it could have run
  a slightly different size.
- **Take the nearest.**
  The backend takes the size closest to the ask
  by vCPUs first and memory second,
  in either direction,
  with the tie rule of the decision in the section titled
  "3.
  Among the sizes that fit,
  take the cheapest,
  the smallest numbers,
  or refuse?".
  `--vcpus 2 --memory 6` ends at `b2-7` (2 vCPUs,
  7 GB,
  1 GB above,
  CA$0.1135 per hour) after the price rule picks it over `c2-7`.
  Gives up:
  the guest can be smaller than the ask
  (`--vcpus 3` would take a 2-vCPU size),
  so a build sized to its memory fails in it after minutes of billed work,
  and the failure looks like a defect of the work.

Ranking:
refuse > nearest.
A refusal costs one more command and names the sizes and their prices;
a too-small guest costs the run,
the billed seconds until someone notices,
and a misleading failure.
The case for nearest is real and is why the two are close:
the owner's own example,
2 vCPUs and 6 GB,
is exactly the case nearest serves,
and with the floor reading from the decision in the section titled
"1.
Does a bare number mean at least,
or exactly?"
that example never reaches
this decision.

A yes to either leaves open:
the measure if nearest,
which the option states as vCPUs then memory,
and whether the refusal names the prices as well as the sizes.

### 3. Among the sizes that fit, take the cheapest, the smallest numbers, or refuse?

What is being decided:
which of several fitting sizes a create ends with.
On these prices the three answers differ in money and in size,
and the differences are not small.

- **The cheapest published hourly price.**
  The backend reads each fitting size's
  hourly price and takes the lowest,
  ties by name.
  For 2 vCPUs and at least 6 GB the fitting sizes are `b2-7` (CA$0.1135),
  `c2-7` (CA$0.1629),
  `b3-8` (CA$0.082) and `d2-8` (CA$0.0596),
  so it ends at `d2-8`,
  4 vCPUs,
  8 GB,
  CA$0.0596 per hour,
  or at `b3-8`,
  2 vCPUs,
  8 GB,
  CA$0.082 per hour,
  when only the sizes the record shows in stock are counted.
  For 2 vCPUs and at least 4 GB it ends at `d2-4` (CA$0.033) rather than
  `c3-4` (CA$0.0732),
  at the same numbers and CA$0.0402 per hour less
  (`c3-4` costs 121.8 percent more).
  The price comes from the provider's public price data by plan name
  (the flavour record carries `planCodes.hourly`,
  `b3-8.consumption` for `b3-8`);
  this document did not call that source,
  so the read is a lead,
  not a fact,
  and the mechanism needs a fallback:
  when the price cannot be read,
  the create logs it and takes the smallest numbers,
  which is the second option.
  Gives up:
  one more read per create,
  a source this proposal has not verified,
  and a guest that can be larger than the ask
  (`d2-8` has 4 vCPUs for a 2-vCPU ask,
  `b3-8` is 2 GB above a 6 GB ask,
  `b3-16` 8 GB above an 8 GiB ask).
- **The fewest numbers,
  then the smallest disk,
  then the name.**
  The backend keeps the fewest vCPUs that fit,
  then the least memory,
  then the smallest disk,
  then the first name,
  and reads no price.
  It ends at `b2-7` (CA$0.1135) for 2 vCPUs and 6 GB,
  38.4 percent more per hour than `b3-8`,
  which has 1 GB more,
  and 90.4 percent more than `d2-8`,
  which has 2 more vCPUs;
  at `c3-4` (CA$0.0732) for 2 vCPUs and 4 GB,
  121.8 percent more than `d2-4` at the same numbers;
  and at `b2-15` (CA$0.2148) for 4 vCPUs and 8 GiB,
  31.2 percent more than `b3-16`,
  which has 1 GB more.
  Gives up:
  the bill is whatever the numbers imply,
  and the catalogue's numbers do not order by money.
- **Refuse and list them.**
  The create prints every fitting size with its price
  and exits nonzero.
  Gives up:
  `mvm run` and the MCP `run_in_vm`,
  which have nobody to answer,
  fail on every create where more than one size fits.

Ranking:
cheapest > fewest numbers > refuse.
Cheapest is the only one that spends the lowest money for a fitting guest,
and on this catalogue the other two cost 31.2 to 121.8 percent more per hour
for the same or fewer resources.
Fewest numbers over a refusal because it still delivers a guest,
while a refusal leaves every agent-run create without one.

A yes to cheapest leaves open:
the price source and its cache,
which this proposal names but did not call;
the tie rule when two sizes cost the same,
stated as the name;
and what happens when the read fails,
stated as fewest numbers plus a log line.

### 4. Does a bare `--memory 8` mean 8 GiB or 8 GB?

What is being decided:
the unit of the number a person types.
The providers state sizes in GB,
`mvm`'s OVHcloud type reads the catalogue number as GiB,
and `libvirt` takes MiB.
The two readings differ by 7.4 percent,
which is one catalogue step on these sizes.

- **GiB,
  the binary unit.**
  `--memory 8` means 8 GiB.
  Against OVHcloud's numbers the rule converts:
  8 GiB is 8.59 GB,
  so the least size that holds it is `b2-15` (4 vCPUs,
  15 GB) under the
  fewest-numbers choice,
  or `b3-16` (4 vCPUs,
  16 GB,
  CA$0.1637 per hour)
  under the cheapest choice.
  On `libvirt` 8 GiB is today's 8,192 MiB default,
  unchanged.
  Gives up:
  the number typed does not match the number on the provider's page,
  and `--memory 8` lands on a size labelled 15 GB or 16 GB.
- **GB,
  the decimal unit the providers use.**
  `--memory 8` means 8 GB.
  The rule reads the catalogue number directly and ends at `d2-8`
  (4 vCPUs,
  8 GB,
  CA$0.0596 per hour),
  where the GiB reading ends at `b3-16` for CA$0.1637,
  2.75 times as much.
  On `libvirt` 8 GB is 7.45 GiB,
  6.9 percent below today's 8,192 MiB default,
  and it leaves 0.1 GiB over the 7.35 GiB peak the four-job build reached,
  while the arms' own peaks spread 0.76 GiB (6.59 to 7.35 GiB).
  Gives up:
  the same command gives a smaller host guest than it does today,
  and the build's own run-to-run spread is wider than the headroom left over
  the largest measured peak.

Ranking:
GiB > GB.
The workload's figures are GiB,
`libvirt`'s default is GiB,
and the arms' measured peaks spread 0.76 GiB,
so a number that means GiB delivers a guest that holds the work on every arm it
was measured on;
its cost is a conversion a person can see on the printed line.
GB matches the providers' pages with no conversion and lands on `d2-8` at
CA$0.0596 per hour,
where the GiB answer costs CA$0.1637,
2.75 times as much,
at the cost of 0.1 GiB of headroom over the largest measured peak,
which the spread between arms already exceeds,
and a headroom that thin is what ends a build in a kill.

A yes to GiB leaves open:
whether a suffix is also accepted (`8G`,
`8GiB`),
and whether the 8 GiB default stays 8 GiB or follows the owner's chosen cap.

### 5. What does a create without the numbers get?

What is being decided:
`mvm create dev` with no `--vcpus` and no `--memory` on a cloud backend,
and the same command on `libvirt`.
This is the decision that spends money on every `run` and every agent command,
because neither carries a size.

- **One default everywhere:
  4 vCPUs and 8 GiB.**
  The same numbers on every backend.
  On `BHS5` with the cheapest choice that is `b3-16` (4 vCPUs,
  16 GB) at
  CA$0.1637 per hour,
  or `b2-15` (4 vCPUs,
  15 GB) at CA$0.2148 under
  fewest numbers;
  on `libvirt` it is today's 4 vCPUs and 8 GiB.
  Gives up:
  CA$0.1470 more per hour than today's smallest size,
  9.80 times,
  on every create that names no size,
  including every `run`.
  With the GB reading of the decision in the section titled
  "4.
  Does a bare `--memory 8` mean 8 GiB or 8 GB?"
  the same default is `d2-8` (4 vCPUs,
  8 GB) at CA$0.0596 per hour,
  3.57 times today's smallest,
  and then the guest holds 7.45 GiB.
- **Today's defaults,
  per backend.**
  `libvirt` keeps 4 vCPUs and 8 GiB,
  `ovh` the smallest in stock (`d2-2`,
  1 vCPU,
  2 GB,
  CA$0.0167 per hour),
  `hetzner` the cheapest type.
  Gives up:
  the same command gives a guest of 8 GiB on this machine and 2 GB
  on OVHcloud,
  where the 7 GiB build cannot run at all,
  so the wrapper's guest arms stay a host-only thing.
- **No default on a cloud.**
  A cloud create without the numbers is refused,
  naming them.
  Gives up:
  `mvm run`,
  the MCP `run_in_vm` and `create --from` are refused too,
  because none of them carries a size today,
  so they must gain the options in the same change or the refusal must exempt them.

Ranking:
one default > today's defaults > refuse.
One default is the only answer under which the same command holds the measured
build on both machines,
which is what moving the guest rounds to OVHcloud needs;
it pays 9.80 times the smallest size for that,
or 3.57 times with the GB reading of the decision in the section titled
"4.
Does a bare `--memory 8` mean 8 GiB or 8 GB?".
Today's defaults keep the cheapest cloud guest and never pay for what they do not ask,
at the cost that the cloud guest cannot run the build.
A refusal breaks the three paths agents use by default,
which is a larger loss than a larger bill for a guest that works.

A yes to one default leaves open:
the Hetzner side,
which cannot resolve 4 vCPUs and 8 GiB until the backend reads
`cores` and `memory`,
and the clone path.

### 6. How does a person see the size and its price before money is spent?

What is being decided:
the moment between the command and the create call,
where a size and its price can still be changed for nothing.

- **`--preview`,
  on `create` and on `run`.**
  The command resolves the flavour and the image through the same code path,
  reads the hourly price,
  prints one line and exits 0 without creating anything:
  `would create mvm-dev on ovh: d2-8, 4 vCPU, 8 GB, 50 GB disk, BHS5,
  image Ubuntu 26.04, CA$0.0596/h excl. VAT (plan d2-8.consumption)`.
  With no price readable it prints the size and says the price is unavailable,
  and exits 0.
  The MCP `create_vm` and `run_in_vm` take the same argument and return that line
  without creating.
  Mechanism:
  one resolution pass and one price read,
  no state written,
  no instance.
  Gives up:
  a price source this proposal did not call,
  and one more provider read on every preview.
- **A separate `mvm sizes --vcpus 2 --memory 6`.**
  The command lists every candidate with its numbers,
  disk,
  storage kind and
  hourly price,
  and marks the one the rules would take.
  Gives up:
  it is a second command,
  nothing ties its answer to the create that follows,
  and `run` has no equivalent,
  so an agent that spends money unattended still sees nothing.
- **Nothing in `mvm`.**
  The create's own log line is the only sight
  (`creating <name> (flavor <flavor>, image <image>) in <region>`,
  `package/mvm/cli/src/backend/ovh/provisioning.ts`,
  lines 120 to 129),
  and it is printed as the create call starts,
  after the choice and with no price.
  A person reads the price afterwards on the provider's page;
  `run` and the MCP tools print no size at all.
  Gives up:
  the pick and its price are learned after the money is committed.

Ranking:
preview > sizes > nothing.
The preview shows the exact pick and price for the exact command,
works for `run` and for the MCP tools where an agent spends money unattended,
and spends nothing;
the sizes command shows more but is disconnected from the create and absent from `run`;
nothing leaves the pick unknown until the instance exists and bills.

A yes to preview leaves open:
the price source and its behaviour when unreadable,
whether the preview also lists the sizes it rejected,
and whether the MCP success text then repeats the size it created.

## What each command does

`mvm create NAME --vcpus N --memory M`:
resolves the flavour through the candidate steps,
then the image for that flavour,
prints the preview line when asked,
and creates.
The choice among fitting sizes is the decision in the section titled
"3.
Among the sizes that fit,
take the cheapest,
the smallest numbers,
or refuse?".
With `--server-type`,
the two numbers are refused.

`mvm run -- <command>`:
takes the same two options and passes them to the guest it creates.
Without them it follows the decision in the section titled
"5.
What does a create without the numbers get?":
under one default it creates the default guest,
under today's defaults the backend's own default,
and under a refusal on a cloud it is refused before creating.
With `--from` it follows the clone rule stated under
`create --from SOURCE` in this section.

The MCP server:
`create_vm` gains `vcpus` and `memory`,
and its description loses the stale "apply to the hetzner backend only" line.
`run_in_vm` gains the same two arguments and hands them to the same path.
Both return the size they created,
because a caller cannot otherwise learn it:
`VM dev created (d2-8, 4 vCPU, 8 GB) and started`.
`list_vms` does not change (see `mvm list`).

`create --from SOURCE`:
the two numbers are refused beside `--from`,
because a clone copies the source's shape and two sources for one size can
disagree,
and the clone's size is the source's size where the provider records it:
the OVHcloud instance record carries `flavorId`,
and a Hetzner server carries its `server_type`.
Where nothing is recorded (`libvirt`,
whose metadata has no size),
the clone takes the backend's default.
Today the OVHcloud clone takes the smallest flavour in stock instead
(the section titled "What mvm does today"),
so a clone of a large source is a smaller machine.

The region fallback,
with a numbered ask:
a region is skipped when it holds no size that fits,
not only when it holds nothing in stock,
and the create logs each skip with its reason.
The first region holding a fitting size wins,
which is today's order.
Under the cheapest choice the size is chosen within that region,
not across regions,
so a later region with a cheaper fitting size is not consulted;
a cross-region price search is a different loop and would need its own decision.

The environment variables `MVM_OVH_FLAVOR` and `MVM_HCLOUD_SERVER_TYPE`:
one of them names one size for a whole run,
and applies to clones.
Given together with `--vcpus` or `--memory`,
the command line wins and the create
logs the ignored variable,
which is the precedence `--server-type` already has over the environment.
A named `--server-type` and the environment for the other backend are unaffected.

`mvm list`:
prints a name and a state,
as today.
No size is stored for a later command,
so `list` cannot show one,
and the asked numbers are gone once the create returns.
The preview line,
the create's own log line and the MCP success text are where a
size is seen.
Recording the size so `list` can print it needs a stored field
(the libvirt metadata has none,
and the OVHcloud instance record's `flavorId` would need a name lookup);
that is not part of this proposal.

`mvm --backend ovh update`:
keeps reporting each region's smallest in-stock flavour by identifier,
which stops being the default under one default everywhere;
the preview is the command that answers "what would this create take".

## Combinations that cannot be built

- **The cheapest choice on `libvirt`.**
  `libvirt` has no price and allocates exactly,
  so a price rule has nothing to
  compare.
  Named,
  not offered.
- **The cheapest choice with `-flex` twins as candidates.**
  The provider's public price page publishes no hourly price for a `-flex` name,
  so such a rule would pick a size whose money it cannot state.
  The candidate steps drop `-flex` for this reason.
- **One default everywhere on Hetzner without two more reads.**
  The documented object carries `cores` and `memory`,
  and the backend reads neither,
  so 4 vCPUs and 8 GiB cannot be resolved there today.
  Either the backend reads both,
  or Hetzner keeps the cheapest type and the
  default is not "everywhere".
- **A refusal on a cloud with `run`,
  `run_in_vm` and `--from` unchanged.**
  None of the three carries a size,
  so a refusal would hit them all;
  they gain the options in the same change,
  or the refusal exempts them.
- **`--vcpus` or `--memory` beside `--server-type`.**
  Refused:
  two names for one size can disagree.
- **`--vcpus` or `--memory` beside `--from`.**
  Refused:
  a clone copies the source's shape,
  as stated under `create --from SOURCE` in the section titled
  "What each command does".
- **A `--memory` suffix in the other unit.**
  One grammar,
  one unit:
  under GiB,
  `8G` is refused unless the grammar says otherwise,
  and under GB,
  `8GiB` is.
- **Exactly,
  with no fallback,
  on a fixed catalogue.**
  Buildable,
  and it is the combination that refuses the owner's own example;
  named so that a yes to exactly is understood as a yes to those refusals.

## What is not known

- Whether OVHcloud's public order catalogue serves each flavour's hourly price
  to an unauthenticated read,
  which is the mechanism the cheapest choice and the preview would use.
  The flavour record's `planCodes.hourly` is the plan name to match,
  and the price page shows that plan name beside that price;
  the catalogue itself was not called,
  because this round may not.
- Which sizes are in stock in `BHS5` above 2 vCPUs and 8 GiB.
  The read lists the ten smallest in stock,
  so `d2-8`,
  `c3-8`,
  `b2-15`,
  `b3-16` and `c2-15` appear on the price page
  and their stock is not in the record.
- Whether the catalogue's `ram` number is GiB (the source comment's reading)
  or GB (the price page's wording).
  The section titled "4.
  Does a bare `--memory 8` mean 8 GiB or 8 GB?"
  is the decision that settles what `mvm` treats it as.
- What a `-flex` twin costs per hour,
  and whether it differs from its classic twin.
  The price page does not list it.
- Whether Hetzner Cloud sells any GPU server type,
  and whether a `category` allow-list would be complete there.
- Which sizes the wrapper's guest arms need on a cloud machine.
  None has run there,
  and the recipe is tuned to four cores.
- Whether a GPU family could reach `ovh.iops`,
  `ovh.d2`,
  `ovh.ssd.cpu`,
  `ovh.ssd.eg` or `ovh.ssd.ram`.
  The provider's price page files every GPU name it lists under `cloud-gpu`,
  and the allow-list refuses any type it does not name,
  so the direction of a mistake is a refusal,
  not a GPU guest.

## The gate read, and what this revision does with each finding

The read was delivered in scratch that expires,
so each finding is restated here in its own words,
with what was checked and what changed.
"A line" without a path means a line of the previous draft.
It counts five High findings,
eight Medium and four Low,
which is every finding the read reported.

### High

**H1,
the worked examples had two sizes where there are four.**
Checked:
the ten in-stock Linux flavours include `c3-4-flex`,
`b2-7-flex`,
`c2-7-flex` and `b3-8-flex`,
each with its twin's numbers and disk.
The finding is right.
Changed:
the list in "What the OVHcloud catalogue returned" has all ten,
the `-flex` step is stated with its reason and its cost,
and each rule's ending size is named in "The sizes the examples use".

**H2,
no option kept a GPU size out by construction,
and the image premise was
not in the evidence.**
Checked:
the three GPU driver images carry `flavorType` null,
the record lists distinct `flavorType` values and not counts,
`flavorType` is an image's admission list (`images.ts`,
lines 200 to 228),
and the provider's price page files `h100`,
`a10`,
`h200`,
`b300` and `rubin`
under `cloud-gpu`,
none of which the string names.
The finding is right.
Two things the read did not have:
OVHcloud's instance API documents `currentState.flavor.gpus`
(only on an instance,
not on the list `mvm` reads),
and the price page's `cloud-gpu` group gives the family list the allow-list needs.
Changed:
the image-derived rule is replaced by the family allow-list,
tested by hand against the 182 flavours,
with the drops and keeps named,
and the same question answered for Hetzner from its documented object.

**H3,
the decisions were not separable,
and one real option was missing.**
Checked:
"at least" fixed the order among fitting sizes,
so "cheapest that fits" was
unreachable;
exact and at-least were offered as alternatives;
"no default on a cloud" was not followed through to `run`.
The finding is right.
Changed:
the meaning of a number,
the fallback,
and the choice among fits are three
decisions,
both spellings are reachable on the number,
the cheapest choice is added with prices,
and the impossible combinations are listed.

**H4,
paths that create a guest with no size were undecided.**
Checked in source:
`run` (parser lines 286 to 304,
`ephemeralRun` at lines 183 to 188),
`run_in_vm` (`tools-exec.ts`,
lines 85 to 114),
`create --from` dropping `--image`,
`--server-type` and `--location`
(`cli.ts`,
lines 165 to 178),
the region loop (`provisioning.ts`,
lines 176 to 202),
the two environment variables,
`list` (name and state only),
and the OVHcloud clone taking the smallest in stock
(`ovh/lifecycle.ts`,
lines 179 to 195).
The finding is right.
Changed:
"What each command does" states each one,
the clone rule is stated,
and the combinations that cannot be built are named.

**H5,
no money and no way to see the pick.**
Checked:
the backend bills by the second beyond a one-minute minimum and a stopped
instance still bills (README,
lines 585 to 600),
the create logs the flavour and image just before the call
(`provisioning.ts`,
lines 120 to 129),
and the MCP success text names no size.
The finding is right.
Changed:
"Prices" gives the hourly price of every size an example ends with,
with its page and date,
each example states what a wrong pick costs per hour,
and the preview is a decision with its mechanism.

### Medium

**M1,
the precedent sentence was wrong for two of three tools.**
Checked:
Multipass allocates exactly,
OpenStack's `--min-disk` and `--min-ram` filter `flavor list`,
and EC2 takes attributes and picks by lowest price with price protection.
The finding is right.
Changed:
each tool is stated on its own,
with its sentence quoted,
a fragment checked in a browser,
and what it does with an ask no machine has exactly.

**M2,
options were named by a label or showed no result.**
Checked:
each of the previous draft's options against the rule.
The finding is right.
Changed:
every option states its mechanism,
gives a command line and the size it ends with,
its hourly price,
and what happens when nothing fits.

**M3,
the candidate rule ignored what already decides a flavour.**
Checked in source:
`flavorCanBoot` reads the image's `minDisk`,
`minRam` and `flavorType`
(`images.ts`,
lines 200 to 228),
`selectFlavor` filters on `available` and prefers `osType` linux
(`flavors.ts`,
lines 125 to 127 and 153 to 169),
the flavour carries `quota` (`ovh/types.ts`,
lines 327 to 330),
and the image is chosen after the flavour
(`provisioning.ts`,
lines 108 to 119).
The finding is right.
Changed:
the candidate steps list all six,
in order.

**M4,
the tie today is by identifier,
which is not the name,
and errors print
identifiers.**
Checked:
`orderFlavors` breaks ties with `left.id.localeCompare(right.id)`
(`flavors.ts`,
lines 89 to 90),
the refusal lists `flavor.id` (lines 137 to 143),
`update.ts` prints the identifier (line 88),
the type comment claims the name "repeats the identifier"
(`ovh/types.ts`,
line 295),
and the record shows an identifier that is not the name `d2-2`.
The finding is right,
and the comment is wrong for that record.
Changed:
the tie rules use price,
disk and name;
refusals name flavours and prices.
The comment itself is a defect of the package and is left to its own fix,
not edited by this proposal.

**M5,
ranking reasons and the workload.**
Checked:
the guest drivers name `--backend libvirt` (`mvm-guest-detached-script.ts`,
lines 212 and 314),
the recipe creates with no size and tunes two jobs to four cores
(`run-fixture.ts`,
lines 360 to 375 and 741 to 745),
and the measured build holds 6.7 to 7.35 GiB.
The finding is right.
Changed:
"The workload on record" states the figures,
and the rankings are by that workload and by money,
with no reason resting on what exists or on how much code a choice needs.

**M6,
Hetzner was called unread.**
Checked:
the backend reads `name`,
`architecture`,
the top-level `deprecation` and
`prices[].price_hourly.gross`,
and the documented object carries `cores`,
`memory`,
`disk`,
`storage_type`,
`cpu_type`,
`category`,
`architecture` and `locations[]`.
The finding is right.
Changed:
the Hetzner section quotes the documented fields,
says what the backend reads and does not read,
and says what the same question looks like there.

**M7,
libvirt needed a bound,
and `--server-type` interplay.**
Checked:
the unit's cap is half the machine's total in source,
the owner raised it to three quarters at runtime on 2026-11-28,
a cap belongs to the whole unit,
and a guest is charged what it touches.
The finding is right.
Changed:
"The host cap,
and what a memory ask means on libvirt" states all of it,
and the ask above the cap is refused.

**M8,
the questions did not stand alone.**
Checked:
each of the previous five questions against the rule.
The finding is right.
Changed:
six decisions,
each with its context,
options,
mechanism,
example,
cost,
ranking and what a yes leaves open.

### Low

**L1,
line citations.**
Checked:
the create parser is lines 128 to 150,
`--from` 58 to 66,
`--image` 72 to 81,
`--server-type` 87 to 96,
`--location` 103 to 112,
and `run` 286 to 304.
The finding is right.
Changed:
the citations are redone against this branch's base.

**L2,
other "today" omissions.**
Checked:
the README lists the defaults at lines 872 to 877,
the `--server-type` help text at line 93,
the MCP description at line 38,
and five callers of `domainXml` carry the constants.
The finding is right.
Changed:
"What mvm does today" names them.

**L3,
provider facts.**
Checked:
a flavour record has fourteen keys,
the `capabilities` are the same four on every type sampled,
the type counts sum to 182,
and `planCodes.hourly` is the plan name.
The finding is right.
Changed:
all four are stated.

**L4,
the owner's first sentence and the units.**
Checked:
both sentences stand in
`doc/decision/mvm-gains-an-ovhcloud-backend-billed-by-the-second.md`,
the GPU sentence at line 229 under the section titled
`The third answer, and "Please don't pick GPU?"`,
and the vCPU sentence at line 268 under "The fourth question,
left without a choice".
So the finding's claim that the first sentence "appears in no record but this
proposal" is wrong.
The record gives no second-precise time for the GPU sentence either,
so this revision cites the section instead of the draft's "a few minutes before".
The unit point is right:
the owner wrote "GBs",
the previous draft read GiB without saying so,
and the section titled "4.
Does a bare `--memory 8` mean 8 GiB or 8 GB?"
now puts it to the owner.

## Sources

Pages fetched on 2026-11-28 for this revision,
each with what was taken from it:

- `https://www.ovhcloud.com/en-ca/public-cloud/prices/`,
  fetched 2026-11-28T06:02:44-05:00 and loaded in headless Chromium:
  the hourly and monthly prices,
  the family groups,
  the `GB` wording and the "excl.
  VAT" basis,
  and the absence of any `-flex` name.
- `https://docs.ovhcloud.com/en/guides/public-cloud/compute/key-concepts/`
  and its Markdown form `.md`,
  fetched 2026-11-28T06:05:05-05:00:
  the seven families and their descriptions.
- `https://docs.ovhcloud.com/en/guides/public-cloud/compute/deploy-a-gpu-instance/`
  and its Markdown form,
  fetched at the same time:
  the GPU regions,
  the models named,
  and the image sentence.
- `https://docs.ovhcloud.com/en/guides/public-cloud/compute/resize-instance-manager.md`,
  fetched at the same time:
  the Flexible instances sentence.
- `https://docs.ovhcloud.com/en/guides/public-cloud/compute/revert-a-flex-instance.md`,
  fetched at the same time:
  what a flex instance is and that the option is not on every model.
- `https://developers.eu.ovhcloud.com/reference/public-cloud/compute/instances.md`,
  fetched 2026-11-28T06:16:00-05:00:
  `currentState.flavor.gpus` and the unit of `currentState.flavor.ram`.
- `https://docs.hetzner.cloud/cloud.spec.json`,
  fetched 2026-11-28T06:08:00-05:00,
  with `https://docs.hetzner.cloud/`:
  the documented server type object's fields and descriptions.
- `https://canonical.com/multipass/docs/latest/reference/command-line-interface/launch/`
  (the address `https://documentation.ubuntu.com/multipass/en/latest/reference/command-line-interface/launch/`
  redirects to it),
  fetched 2026-11-28T06:05:05-05:00:
  the `--cpus` and `--memory` help text.
- `https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/ec2-fleet-attribute-based-instance-type-selection.html`,
  fetched at the same time:
  the attribute sentence,
  price protection,
  the lowest price allocation strategy,
  and the preview section.
- `https://docs.openstack.org/python-openstackclient/latest/cli/command-objects/flavor.html`,
  fetched at the same time:
  `--min-disk` and `--min-ram`.
- `https://docs.skypilot.co/en/latest/reference/yaml-spec.html`,
  fetched at the same time:
  the `cpus` and `memory` formats and the `4+` example.
- `https://cloud.google.com/compute/docs/instances/creating-instance-with-custom-machine-type`,
  fetched at the same time:
  the custom machine type sentence and the sample code's refusals.

The catalogue facts come from a read-only read of the owner's project on
2026-11-28 between 04:48 and 04:50,
which is also where the ten in-stock flavours,
the type counts,
the flavour keys,
the image data and the quota field come from.

Each quotation in this document was copied from the fetched text and checked
against it by a script before the commit that carries it.
Each `#:~:text=` fragment was opened in headless Chromium through `agent-browser`
on 2026-11-28,
and counted as confirmed only when the page with the fragment scrolled to the
text and the same page without it,
and with a deliberately wrong fragment,
did not.

The OVHcloud documentation pages and the price page are page links and not
fragments.
Their content is rendered by a client-side framework that redraws the page
after load,
so a fragment does not move the page there:
`Types of Instances` and `Hardware-Accelerated Graphics` both left the page
where it was,
with the words present in the text.
The words quoted from them were confirmed in the fetched text by script instead,
and the section each sits in is named beside the quote.
