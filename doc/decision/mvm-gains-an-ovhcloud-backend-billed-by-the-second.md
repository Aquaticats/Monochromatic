# mvm gains an OVHcloud backend, because its instances are billed by the second

Status:
decided by the owner on 2026-11-28.
The backend is not built yet,
and nothing here is approved as complete.

## Decision

`mvm` gains a backend that makes its machines as OVHcloud Public Cloud instances,
beside the `libvirt` and `hetzner` backends
(`package/mvm/cli/src/backend/types.ts`,
`BackendKind`).
Its billed live tests are authorised,
and the region is Canada.

## The owner's words

Typed after 2026-11-28T02:10:19-05:00 and before 2026-11-28T02:14:33-05:00:

> I'm on OVHcloud and I can create API keys for it to be used as a mvm backing provider.
> What do I need to do,
> exactly?
> The reason I'm not picking Hetzner is because Hetzner charges by the hour.

Asked whether the backend is built now,
and whether billed live tests may run once the keys are stored,
the owner was shown these ways:
build now and test when the keys exist;
build now and ask before any billed call;
not yet.
The owner chose none of the listed ways and wrote,
before 2026-11-28T02:21:18-05:00:

> Build and test.
> I just did it.
> region:
> ca.

## What OVHcloud's own documents say

Its billing guide,
[Information regarding Public Cloud billing options][billing-second],
read on 2026-11-28 with the page's own date 2026-11-05:

- an instance on the hourly price is metered by the second:
  "Every second of use is billed,
  beyond an initial 1-minute minimum";
- [the meter starts][billing-active] when the instance reaches the `ACTIVE` status,
  so the time in `BUILD` is not billed;
- [the billing stops][billing-stops] only when the instance is deleted,
  and a stopped or paused instance still counts;
- on the monthly price,
  [each commenced month is billed][billing-month],
  even if the instance is deleted before the month ends.

Its guide [First Steps with the OVHcloud APIs][keys-three] says a script's access is three keys,
an application key,
an application secret and a consumer key,
and that [the rights field][keys-rights] limits them to chosen methods and paths.

Its public schema of the cloud API,
`https://eu.api.ovh.com/v1/cloud.json`,
read on 2026-11-28,
lists `POST /cloud/project/{serviceName}/instance` with the fields
`flavorId`,
`imageId`,
`name`,
`region`,
`sshKeyId`,
`userData` and `monthlyBilling`,
and the calls that list and delete instances,
list flavors,
images and regions,
make and delete SSH keys,
and snapshot an instance.
The instance it describes carries no label.

Each sentence quoted here was found on its page with `agent-browser` on 2026-11-28.

## What follows from the decision

- The backend never asks for monthly billing.
- The backend deletes what it made:
  a machine that is only stopped is still billed.
- The backend reads its keys from the environment,
  as the Hetzner one reads `HCLOUD_TOKEN`
  (`package/mvm/cli/src/backend/hetzner/config.ts`),
  and the repository's encrypted `.env.local.json` holds them.
  At 2026-11-28T02:21:18-05:00 that file held these names:
  `OVH_ENDPOINT`,
  `OVH_APPLICATION_KEY`,
  `OVH_APPLICATION_SECRET`,
  `OVH_CONSUMER_KEY` and `MVM_OVH_PROJECT_ID`.
  No value was read.

## The coordinating session's picks, with the veto open

- The backend speaks the OVHcloud API under `/cloud/project/`,
  with the keys the owner made,
  and not the project's OpenStack endpoints,
  which need a second kind of account.
- The first four names are the ones OVHcloud's own clients read
  (`README.rst` of `github.com/ovh/python-ovh`);
  `MVM_OVH_PROJECT_ID` is this session's own name for the project.
- "region:
  ca" is read as the Canadian API endpoint and the project's Canadian regions.
  The backend takes the regions from the project's own list,
  and no region name is written here from memory.
- A sweep finds the backend's machines by `mvm`'s name prefix inside the one project,
  because the schema shows no label to scope by.
- The backend signs its requests with the runtime's own hash and sends them with `fetch`,
  and adds no dependency for it.

## The owner's answers on the billed test, 2026-11-28

Unapproved and non-final:
the backend has created no machine that answered yet.
Times are this machine's clock,
UTC-05:00.

### The first answer: run the test as built

The owner had written "Build and test."
before the test existed,
so the session listed what the test does,
step by step,
and asked.
The owner chose "Run it as built (Recommended)",
whose text was these steps:

1.  merge the branch,
    then widen AppArmor's `ssh-keygen` profile so it may write under `~/.local/share/mvm/ovh/`;
2.  `mise` gives the five key values from the encrypted secrets file to one process,
    `node src/backend/ovh/provision.expensive.unit.test.ts`;
3.  `ssh-keygen` makes one ed25519 keypair with no passphrase under `~/.local/share/mvm/ovh/`,
    which stays on this machine;
4.  reads:
    the clock,
    the project's regions,
    flavors,
    images and SSH keys;
5.  the public key is uploaded to the project under a name beginning `mvm-`,
    and stays there;
6.  one machine is created,
    of the smallest flavor in stock,
    with `monthlyBilling` false;
7.  up to 300 s for `ACTIVE`,
    then `ssh` as the image's own user with host-key checking off,
    `uname -a`,
    and one file copied in and out of `/tmp`;
8.  the machine is deleted,
    the project is listed,
    and on any failure a guard deletes every machine and snapshot whose name begins `mvm-`;
9.  the test gives up after 900 s.

Not chosen:
the same run followed by deleting the uploaded key and the keypair,
and not running it.

The run of 04:18:56 ended after 8.7 s and created nothing.
OVHcloud answered 404 to a flavor list:
the project lists regions that offer storage only beside the regions that create instances,
both Canadian and both `UP`,
and the name order put a storage-only one first.

### The second answer: diagnose, fix, run again

Asked what may follow,
the owner chose "Diagnose,
fix,
rerun (Recommended)":
read-only signed reads of the region list,
each region's record and each region's flavor list,
kept in scratch for a day with the project identifier masked;
a change to how the backend picks a region,
with a unit test;
and the same nine steps once more.

The reads showed that a region record carries the services the region offers,
and that every region listing a service named `instance` answered the flavor list
while every region without it answered 404.
The default regions now also require that service to be `UP`
(commits `0a899056f5` and `c5f7e90216`).

Before the second run the session also let `ssh` read `~/.local/share/mvm/ovh/`,
in `/etc/apparmor.d/local/ssh`:
the profile lets `ssh` read keys only in the account's `.ssh` folder,
so step 7 would have failed after a machine existed.
That widening was in neither option's text;
it was done under the owner's standing "Always widen apparmor."
and the owner was told in the next question.

The run of 04:38:34 created a machine:
OVHcloud accepted the create with `monthlyBilling` false.
The machine did not reach `ACTIVE` within the 300 s,
the backend deleted it at 04:43:45,
the project listed it as `DELETING` until 04:48:42,
and listed no instance at 04:49:59.
The flavor was `d2-2`,
the smallest in stock and no GPU machine;
the image was "Ubuntu 26.04 - NVIDIA - v595",
a GPU driver image,
because the backend took the newest image whose name starts with `ubuntu`.
The billing guide quoted under "What OVHcloud's own documents say" says
the time in `BUILD` is not billed;
which status the machine stood in was not logged,
so whether it was billed is not known.

### The third answer, and "Please don't pick GPU?"

Asked again,
the owner chose "Diagnose,
fix,
third run (Recommended)":
read-only reads of the region's images and flavors,
the project's SSH key names and its instances;
a change to the image choice,
stated as
"among active public images whose name starts with the asked name,
take the newest one the chosen flavor can boot,
reading the image's own minDisk,
minRam and flavorType";
a wait that prints each status and names the last one when it gives up;
and the nine steps a third time.

While that ran the owner wrote:

> Please don't pick GPU?

The reads showed that the three fields named in the option cannot tell a GPU driver image from a plain one:
every Ubuntu image in the region states a disk floor of 0,
a memory floor of 0 and no flavor type.
What does tell them apart is the image's tags:
a plain distribution image carries none,
and an image with preinstalled software carries `application` beside a tag naming the software,
`nvidia_ngc` on the GPU driver images.
A second thing would still have gone wrong:
each release has a "- UEFI" variant published a minute after it,
so the newest plain-tagged match is a variant.

So the mechanism differs from the option's text,
and the session put it to the owner again before the third run.
A flavor record carries no field that marks a GPU flavor;
the chooser takes the smallest flavor in stock,
which can be a GPU flavor only when nothing smaller is in stock,
and an explicit guard for that is open.

## What is not known

- Whether the key the owner made is limited to one project,
  which this session recommended,
  and whether that project is used for nothing else.
- The project's quota of instances,
  cores and memory in a Canadian region.
- How Hetzner bills:
  the statement is the owner's,
  and this session did not check it.
- Whether the wrapper's guest arms run on such a machine.
  Their drivers name the `libvirt` backend today,
  and moving them is separate work.

[billing-second]: https://docs.ovhcloud.com/en/guides/public-cloud/cross-functional/analyze-billing#:~:text=Every%20second%20of%20use%20is%20billed%2C%20beyond%20an%20initial%201%2Dminute%20minimum
[billing-active]: https://docs.ovhcloud.com/en/guides/public-cloud/cross-functional/analyze-billing#:~:text=the%20billing%20meter%20starts%20when%20an%20instance%20reaches%20the
[billing-stops]: https://docs.ovhcloud.com/en/guides/public-cloud/cross-functional/analyze-billing#:~:text=The%20billing%20for%20an%20instance%20stops%20when%20the%20instance%20is%20deleted
[billing-month]: https://docs.ovhcloud.com/en/guides/public-cloud/cross-functional/analyze-billing#:~:text=Each%20commenced%20month%20is%20billed%2C%20even%20if%20the%20instance%20is%20deleted%20before%20the%20end%20of%20the%20month.
[keys-three]: https://docs.ovhcloud.com/en/guides/manage-and-operate/api/first-steps#:~:text=you%20will%20be%20issued%20three%20keys
[keys-rights]: https://docs.ovhcloud.com/en/guides/manage-and-operate/api/first-steps#:~:text=field%20allows%20you%20to%20restrict%20the%20use%20of%20the%20application%20to%20certain%20APIs
