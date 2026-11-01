# Runbook: publish the Playwright image and make it public

Publishes `ghcr.io/aquaticats/monochromatic-playwright:<tag>` from `playwright.Dockerfile` with the workflow
`.github/workflows/playwright-image.yml`,
and makes the new package public once.
`<tag>` is the first 16 hex digits of the SHA-256 of `playwright.Dockerfile`:
`sha256sum playwright.Dockerfile | cut --characters=1-16`.
`mise run prepare:playwright` pulls that tag and builds the file locally only when the pull fails.

## What this proves

- The workflow builds the image on a GitHub runner and pushes it with its own `GITHUB_TOKEN`.
- A person with no GitHub login can pull the tag.
- A second run of the workflow finds the tag and builds nothing.

## Why a person runs the visibility step

Bridges tried before writing this:
the REST API for packages (the documentation lists 27 endpoints,
none changes visibility),
a workflow step (the documentation describes none),
and the documentation of the package settings page,
which is the only place it names:
"anyone with admin permissions to the package can set the package to private or public"
and "the default visibility is private".
A browser session with an admin's login could do the clicks,
and no such session is shared with agents.
The organization-level setting named in step 6 was found in the same documentation
(the page sentence was confirmed in the rendered page with `agent-browser` on 2026-11-01),
and it needs an organization owner,
not an agent.
The step cannot be undone:
GitHub states "Once you make a package public,
you cannot make it private again."

## Setup

Status:
TODO

1. Have an account with admin permission on the `Aquaticats` organization,
   which gives admin permission on a package the organization owns.
2. Have the change that adds `.github/workflows/playwright-image.yml` on `main`.
   Its first push to `main` starts the workflow,
   because the workflow file is one of its own triggers.
3. Have a terminal with `podman` and `sha256sum`,
   for the last check.

## Steps

Status:
TODO

1. Open `https://github.com/Aquaticats/Monochromatic/actions/workflows/playwright-image.yml`.
   Expected:
   a run named **playwright-image** exists.
   If none exists,
   click **Run workflow**,
   keep **Branch:
   main**,
   and click **Run workflow** again.
2. Open that run and wait for the job **publish** to finish.
   Expected:
   every step is green,
   including **Push the image**,
   and the run page shows the warning
   **Playwright image is not public**.
3. Open `https://github.com/orgs/Aquaticats/packages`.
   Expected:
   a package named **monochromatic-playwright** is listed.
4. Click **monochromatic-playwright**.
   Expected:
   the package page lists a tag of 16 hex digits and links to the repository `Aquaticats/Monochromatic`.
5. Click **Package settings**.
   Expected:
   the settings page of the package opens.
6. Scroll to **Danger Zone** and click **Change visibility**.
   Expected:
   a choice of **Private** and **Public** appears.
   If **Public** cannot be chosen,
   ask an organization owner to open the organization's **Settings**,
   click **Packages** in the sidebar,
   and under **Package Creation** click **Public**.
   That is the setting GitHub documents for the visibility of packages that organization members create
   ([GitHub Docs, "Package creation visibility for organization members"][package-creation]);
   whether it also gates switching an existing package to public was not tested.
7. Choose **Public** and confirm in the dialog.
   Expected:
   the package page shows **Public** beside its name.
8. Open the run of step 1 again and click **Re-run all jobs**.
   Expected:
   the job **publish** finishes green,
   the step **Compute the tag** prints `tag ` and the 16 hex digits,
   the step **Check whether the tag exists** prints `exists (HTTP 200); nothing to build`,
   and the steps **Build the image** and **Push the image** are skipped.

## What to check

Status:
TODO

1. The step **Check that anyone can pull it** prints `anyone can pull ghcr.io/aquaticats/monochromatic-playwright:`
   followed by the tag,
   and no warning **Playwright image is not public** is on the run page.
2. From a repository checkout,
   with no login to `ghcr.io`:

   ```sh
   # doc/runbook/publish-playwright-image.md, check 2
   tag="$(sha256sum playwright.Dockerfile | cut --characters=1-16)"
   podman logout ghcr.io
   podman pull "ghcr.io/aquaticats/monochromatic-playwright:${tag}"
   ```

   Expected:
   `podman logout` succeeds or says there is no login to remove,
   and the pull prints an image id and no `unauthorized` or `denied`.
3. `mise run prepare:playwright` prints `pulled ghcr.io/aquaticats/monochromatic-playwright:` followed by the tag,
   and `podman image exists monochromatic-playwright` exits `0`.

## Restore

Status:
TODO

The visibility change cannot be undone.
To remove the image:
open the package,
click **Package settings**,
and use the deletion option in the **Danger Zone**;
the workflow publishes the tag again on its next run (a run with **Run workflow**).
A local copy is removed with `podman rmi monochromatic-playwright`.

[package-creation]: https://docs.github.com/en/packages/learn-github-packages/configuring-a-packages-access-control-and-visibility#:~:text=choose%20whether%20you%20want%20to%20enable%20the%20creation%20of%20public%2C%20private%2C%20or%20internal%20packages
