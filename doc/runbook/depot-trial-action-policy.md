# Allow the approved Depot trial actions

GitHub rejected the repository policy update with
`Only 'local_only' actions and workflows are allowed in this organization.`
The existing CLI credential lacks `admin:org`,
and both browser bridges are unavailable.
The user approved the exact action revisions listed in this procedure.
The organization policy must allow them before the repository policy can do so.

## Setup

Status:
TODO

Use a browser signed into a GitHub account that owns `Aquaticats`.
If signed out,
sign in at [GitHub](https://github.com/login).
This changes the organization's permitted-action ceiling;
individual repositories can retain stricter policies.

## Steps

Status:
TODO

1.  Open [Aquaticats Actions settings](https://github.com/organizations/Aquaticats/settings/actions).
    Expect the organization's **Policies** settings.
2.  Select **Allow Aquaticats,
    and select non-Aquaticats,
    actions and reusable workflows**.
    Expect controls for specified external actions.
3.  Add the comma-separated list in this section to **Allow or block specified actions and reusable workflows**.
    Expect exactly the listed action revisions in the field.
4.  Keep **Require actions to be pinned to a full-length commit SHA** enabled.
    Expect SHA pinning to remain selected.
5.  Leave **Allow actions created by GitHub** and **Allow Marketplace actions by verified creators** unchecked.
    Expect access to remain limited to the explicit list and organization actions.
6.  Click **Save**.
    Expect the selected policy and exact revisions to persist after refreshing.
7.  Tell the agent the policy is saved.
    Expect the agent to set the repository's exact allow-list and rerun the comparison.

```text
actions/checkout@de0fac2e4500dabe0009e67214ff5f5447ce83dd,jdx/mise-action@c2a87611a18de5b3828c5652fe268e992400cb5c,docker/setup-buildx-action@8d2750c68a42422c14e847fe6c8ac0403b4cbd6f
```

GitHub documents the [selected-actions syntax and organization policy][policy].

## What to check

Status:
TODO

The repository policy update must stop returning HTTP 409.
The GitHub trial must start actual jobs;
an accepted settings update alone does not verify execution.

## Restore

Status:
TODO

The previous organization action policy was `local_only` with SHA pinning required.
Restoring that policy blocks external actions,
including `actions/checkout`,
in every affected repository.
Inspect other workflows and coordinate restoration before reverting a shared organization setting.

[policy]: https://docs.github.com/en/organizations/managing-organization-settings/disabling-or-limiting-github-actions-for-your-organization
