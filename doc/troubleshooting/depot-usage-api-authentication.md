# Depot CLI 2.102.22 user token lists organizations but cannot read usage

## Symptom

On 2026-10-15,
`depot org list --output json` succeeded with the existing credential,
but a read-only request to `depot.core.v1.UsageService/GetUsage` failed.
The Depot API returned this exact error:

```json
{
  "code": "unauthenticated",
  "message": "This endpoint requires an organization token, but a user token was provided. Create an organization token in Organization Settings > API Tokens"
}
```

The successful organization listing does not establish authorization to read usage.
No credential value,
organization identifier,
or organization name is recorded here.

## Root cause

The usage endpoint rejected the credential's token type.
Its response explicitly identifies the supplied credential as a user token
and requires an organization token.
This is an observed authentication requirement,
not an established upstream defect.

The CLI source at tag `v2.102.22`,
[`pkg/helpers/organization.go:22`](https://github.com/depot/cli/blob/v2.102.22/pkg/helpers/organization.go#L22),
shows organization listing calling `ListOrganizations` with the stored API token:

```go
// depot/cli, pkg/helpers/organization.go:22
func RetrieveOrganizations() ([]*Organization, error) {
	client := api.NewOrganizationsClient()
	req := corev1.ListOrganizationsRequest{}
	resp, err := client.ListOrganizations(
		context.Background(),
		api.WithAuthentication(connect.NewRequest(&req), config.GetApiToken()),
	)
```

This client code explains the successful command's authentication path.
The usage endpoint's server authorization implementation was not inspected;
the rejection response is the evidence for its token requirement.

The [official SDK reference][sdk] documents `GetUsage` as a usage read
with mandatory `startAt` and `endAt` timestamps.
It does not make organization listing a test of usage authorization.

## Verification

The installed CLI reported `depot version 2.102.22 (2026-10-14)`.
The usage request and response definitions were also inspected in `depot/sdk-node`
at commit `f9ec4c00dd573219adf2ec6384964aac4e6a1c31`.
The probe completed by `2026-10-15 21:30:41 UTC`.

### Operations that completed

- `depot org list --output json` returned one accessible organization.
  Its returned fields were `OrgId` and `Name`.
- CLI help and public API/source inspection completed without changing authentication.

### Operations that failed

- `GetUsage` with the existing user token returned `unauthenticated`
  with the exact message in `Symptom`.
- The requested usage interval was `2026-10-01T00:00:00Z` to `2026-10-15T21:00:00Z`.
  No usage data was returned for that interval.

### Reproduction

The invocation sends the stored token directly through a pipe into curl's request headers.
It does not put the credential in command arguments or display it in the terminal.
The operation reads usage;
it does not create a token or start compute.

```sh
# Read-only reproduction using the existing Depot CLI credential
depot login token |
  sed 's/^/Authorization: Bearer /' |
  curl --silent --show-error --header @- \
    --header 'Content-Type: application/json' \
    --header 'Connect-Protocol-Version: 1' \
    --request POST \
    --data '{"startAt":"2026-10-01T00:00:00Z","endAt":"2026-10-15T21:00:00Z"}' \
    https://api.depot.dev/depot.core.v1.UsageService/GetUsage
```

The probe retained the JSON error response;
it did not record the HTTP response status.
An absent `DEPOT_TOKEN` environment variable was checked without displaying any credential.

## Verified workarounds

None.
The server names an organization token as the required credential,
but no organization token was created or supplied during this investigation.
Success with that credential type remains unverified in this account.

## What does not work

Reusing the credential that successfully lists organizations does not authorize `GetUsage`.
The organization listing cannot establish the account's plan,
trial expiry,
remaining allowances,
current usage or charges,
or configured spending controls.

The [account documentation][account] places plan and billing information
in the dashboard's `Billing & usage` page.
That page was not inspected.
An aggregate USD 200 hard spending cap therefore remains unverified.

## Upstream filing artifact

Nothing to add upstream.
The server supplied an explicit token-type requirement,
and no defect was established.
No issue,
comment,
or speculative fix was drafted.

### Upstream filing decision

The `.out-of-scope/` entries were checked;
none names Depot or this authentication requirement.
No upstream tracker search or source clone was performed because this scoped task
records expected authentication behavior without proposing a defect report.

1.  Upstream fault:
    not established;
    the observed request used a token type the endpoint rejects explicitly.
2.  Upstream fixability:
    no fix is proposed because no defect was established.
3.  Supported use case:
    the SDK documents usage reads;
    the server requires an organization token for this request.
4.  Contribution policy:
    not investigated because no contribution is proposed.
5.  Maintainer willingness:
    not investigated because no behavior change is proposed.
6.  Minimal fix:
    not applicable;
    no upstream defect or candidate patch was identified.

[sdk]: https://depot.dev/docs/api/sdk-reference#usage-service
[account]: https://depot.dev/docs/account#billing
