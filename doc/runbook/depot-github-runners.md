# Connect Depot GitHub Actions runners for the trial

This enables the runner comparison for the public repository `Aquaticats/Monochromatic`.
The Depot Code Access app is already verified by a successful Depot CI checkout.
The separate runner app is absent from the GitHub organization installation listing.
The GitHub runner-group API rejected the available credential because it lacks `admin:org`.
The in-app browser returned no available browser,
and the standalone connection returned `No running Chrome instance found`.
The agent prepared the disposable build comparison on branch `trial/depot-20261015`.

## Setup

Status:
TODO

Use a browser signed in to the existing Depot trial organization and a GitHub account
with owner access to `Aquaticats`.
If signed out,
sign in at [Depot](https://depot.dev/login) and [GitHub](https://github.com/login).
No credential needs to be pasted into chat.

## Steps

Status:
TODO

1.  Open the [Depot dashboard](https://depot.dev/orgs/_).
    Expect the existing trial organization;
    select it if another organization is active.
2.  Open **GitHub Actions**.
    Expect the runner page with **Connect to GitHub**.
3.  Click **Connect to GitHub**.
    Expect GitHub's installation form for the Depot runner app.
4.  Choose **Aquaticats** as the installation owner.
    Expect the organization's repository and permission form.
5.  Complete installation for **Monochromatic**.
    Expect the runner connection to appear in Depot.
6.  Open [Aquaticats runner groups](https://github.com/organizations/Aquaticats/settings/actions/runner-groups).
    Expect the organization runner groups.
7.  Open **Default**.
    Expect its repository access settings.
8.  Enable **Allow public repositories** for the group used by Depot.
    Expect the public repository to be eligible for these runners after saving.
9.  Tell the agent the runner connection is complete.
    Expect verification through a disposable GitHub job on a Depot runner.

The [official quickstart](https://depot.dev/docs/github-actions/quickstart)
documents the installation and public-repository setting.

## What to check

Status:
TODO

The organization installation listing should contain a Depot runner app in addition to `depot-code-access`.
The actual verification is a GitHub Actions job using `depot-ubuntu-24.04`
that starts and completes the trial command.
A queued job alone does not verify the connection.

## Restore

Status:
TODO

The comparison branch does not replace production workflows or publish release artifacts.
Keep the runner connection while the trial is active.
If abandoning the trial,
request removal after the agent checks active jobs and other repositories using the connection.
Record the runner group's previous public-repository setting before changing it so it can be restored.
