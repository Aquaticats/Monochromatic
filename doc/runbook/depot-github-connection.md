# Connect the Depot trial to GitHub

This connects the trial to `Aquaticats/Monochromatic` after its organization transfer.
The Depot CLI preflight reports that Depot Code Access is not installed for `Aquaticats`.
The GitHub installation API returned no installations for that organization.
The in-app browser discovery returned no browser,
and the standalone browser connection returned `No running Chrome instance found`.
The agent can prepare and run container builds while this connection is completed.

## Setup

Status:
TODO

Use a browser signed in to the existing Depot trial organization and a GitHub account
that owns the `Aquaticats` organization.
The repository is public.
The installed Depot CLI is already authenticated on the agent's machine.
No token needs to be pasted into chat.

## Steps

Status:
TODO

1.  Open [Depot organization settings](https://depot.dev/orgs/_/settings).
    Expect the selected organization's settings;
    select the existing trial organization if another organization is displayed.
2.  In **GitHub Code Access**,
    use **Connect to GitHub**.
    Expect the GitHub installation page for **Depot Code Access**.
3.  Choose **Aquaticats** as the installation owner.
    Expect the repository-selection and permission form for that organization.
4.  Restrict repository access to **Monochromatic** and complete the installation form.
    Expect GitHub to confirm the installation or return to Depot.
5.  Tell the agent the connection is complete.
    Expect the agent to rerun the CLI preflight before dispatching a workflow.

The [Depot CI quickstart](https://depot.dev/docs/ci/quickstart)
documents the Code Access connection.
The separate GitHub Actions runner integration is evaluated separately;
this procedure does not claim it is installed.

## What to check

Status:
TODO

The agent runs `depot ci migrate preflight --yes` from the checkout.
It must detect `Aquaticats/Monochromatic`
and no longer report `The Depot Code Access app is not installed for Aquaticats.`
A subsequent disposable workflow run must succeed before the connection is considered verified.

## Restore

Status:
TODO

The procedure creates an organization-level GitHub App installation,
not a release or a replacement production workflow.
Keep it installed while the trial runs.
If the trial is abandoned,
request removal of the trial's Code Access connection;
the agent will inspect active runs and the installation before removing access.
