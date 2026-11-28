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
