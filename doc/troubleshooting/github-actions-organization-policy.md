# GitHub Actions rejects external actions before starting the Depot trial

## Symptom

On 2026-10-15,
[run 38812935669][failed-run] ended with `startup_failure` and no jobs.
The annotation rejected pinned `actions/checkout`,
`jdx/mise-action`,
and `docker/setup-buildx-action` because the policy required organization-owned actions.

The repository API returned:

```json
{"enabled":true,"allowed_actions":"local_only","sha_pinning_required":true}
```

The allowed-actions query returned HTTP 409 with
`Only local actions and workflows are allowed on this repository`.

## Root cause

This was an organization action-policy restriction,
not a Depot runner startup failure.
The GitHub-hosted reference job was blocked too.
The repository update then returned HTTP 409 with
`Only 'local_only' actions and workflows are allowed in this organization.`

GitHub's [organization policy documentation][policy] explicitly says this policy blocks GitHub-authored actions,
including checkout.
The [permissions API documentation][api] describes the repository and organization endpoints.
GitHub's server-side policy implementation is not public;
the diagnosis rests on the deciding API response and the run annotation.
The historical selected-action policy documented for the repository's former owner was not current evidence.

## Verification

The failing workflow was `.github/workflows/depot-trial.yml` at commit `a4d88c7516`.
The three external actions were already pinned to full commit SHAs.
Pinning alone did not satisfy the owner restriction.

```sh
# Repository policy and the observed run
gh api repos/Aquaticats/Monochromatic/actions/permissions
gh api repos/Aquaticats/Monochromatic/actions/permissions/selected-actions
gh run view 38812935669 --repo Aquaticats/Monochromatic
```

The user initially approved the exact revisions,
then deliberately selected broader organization patterns:
`actions/*`,
`jdx/*`,
and `docker/*`,
with GitHub-owned actions and verified creators also allowed.
They explicitly confirmed that broader choice.
The repository policy was set to `selected` while retaining SHA pinning.
The allowed-actions response then exposed that inherited organization list.

[Run 38813764705][comparison-run] subsequently started both the GitHub-hosted job
and the Depot-managed runner job.
That verifies policy admission and actual runner allocation;
it does not by itself establish successful application compilation.

## Verified workaround

Use the user-approved organization action policy and inherit it at the repository boundary.
The [manual procedure](../runbook/depot-trial-action-policy.md) records the originally proposed exact-revision option.
The actual selected policy is broader,
as recorded in this document.
Its tradeoff is permitting additional actions at organization scope;
the trial workflow continues to use exact revisions.
No organization credential was created or broadened in this session.

## What does not work

- Setting only the repository's `allowed_actions` cannot override an organization `local_only` restriction.
- Writing a repository-specific pattern list after the organization controls it returned
  `Selected actions and workflows are already set at the organization or enterprise level`.
- `gh run rerun 38812935669` returned `This workflow run cannot be retried`.
  A new push to the trial branch created the successful admission check.
- Changing the runner label alone cannot fix an action policy rejection.

## Upstream filing decision

No upstream issue or draft is warranted:

- Upstream fault:
  no;
  the configured policy was enforced.
- Fixability:
  configuration already supplies the supported control.
- Supported use case:
  selected external actions are documented.
- Contribution acceptance:
  not investigated because no defect or contribution is proposed.
- Likely upstream change:
  none is requested for intentional policy enforcement.
- Prototype:
  no upstream patch is needed;
  the authorized configuration change admitted real jobs.

No upstream message was sent.

[failed-run]: https://github.com/Aquaticats/Monochromatic/actions/runs/38812935669
[comparison-run]: https://github.com/Aquaticats/Monochromatic/actions/runs/38813764705
[policy]: https://docs.github.com/en/organizations/managing-organization-settings/disabling-or-limiting-github-actions-for-your-organization
[api]: https://docs.github.com/en/rest/actions/permissions
