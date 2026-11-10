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

## What the main-only guard stops

The job carries `if: github.ref == 'refs/heads/main'` and the concurrency group is per ref (`playwright-image-<ref>`),
so a **Run workflow** from another branch or a tag skips the job,
and such a run does not queue behind or replace a pending run on `main`.

What it does not stop:

- A branch that edits `.github/workflows/playwright-image.yml` and removes the guard.
  GitHub's pages on `workflow_dispatch` say
  "This event will only trigger a workflow run if the workflow file exists on the default branch"
  and "Once a workflow has run at least once,
  you can dispatch it against any branch or tag via the GitHub API or GitHub CLI"
  ([events that trigger workflows][events-doc]),
  and neither that page,
  the manual-run page nor the REST page says which version of the file a dispatch on another branch runs.
  Measured instead (2026-11-01,
  past dispatch runs of `music-player-release.yml` read through the API):
  run 38785367004 on branch `feat/music-player-license-notices` ran a `notices` job that the default branch's file
  did not have at that time,
  and run 39756720608 on `chore/music-player-ci-merge` ran test jobs that the default branch's file did not have.
  So a dispatch on a branch runs that branch's own version of the file,
  with the `packages: write` token this workflow declares.
- Anyone who can push a branch to this repository can therefore publish a tag from it.
  What would stop that is a protected environment whose deployment branches are limited to `main`,
  with the job naming it.
  That needs a repository setting and was not created;
  this runbook only records that it is the missing piece.

## If the first run stops at "Cannot tell whether the tag exists"

The step **Check whether the tag exists** fails the run,
on purpose,
for any answer it cannot read as "the tag exists" or "the tag does not exist".
Nothing was built or pushed,
so nothing was published and no existing tag was replaced.

- After a registry fault (an HTTP 5xx or 429,
  a timeout,
  a refused connection):
  click **Re-run all jobs** once.
- On the first run,
  before the package exists:
  the answer of the registry to the workflow token for a package that does not exist yet was never observed;
  only an answer to a personal token was,
  and the step treats exactly two answers as "absent",
  a token request answered HTTP 403 and a manifest request answered HTTP 404.
  If the error names another status,
  a rerun fails the same way,
  and the status handling of that step in `.github/workflows/playwright-image.yml` has to be edited for the answer the error names.

## Provenance and its limit

The step **Attest provenance** records a build provenance attestation for the pushed image's digest,
with the same action and permissions as the files of `music-player-release.yml`.
It is stored in GitHub's attestation store and not pushed to the registry.
Consumers pull by tag,
and nothing in `podman pull` or in `mise run prepare:playwright` checks the attestation;
`gh attestation verify oci://ghcr.io/aquaticats/monochromatic-playwright:<tag> --owner Aquaticats` does.
That command and the attestation step have not run yet,
because nothing has been pushed.

If the attestation step fails after the push succeeded,
the image stays published and the run is red.
A rerun does not repair it:
the step **Check whether the tag exists** finds the tag,
and the steps **Build the image**,
**Push the image** and **Attest provenance** are skipped.
What the sources say about redoing it:
the documentation of `actions/attest` (the action `actions/attest-build-provenance` wraps)
places the action "Within the GitHub Actions workflow which builds some artifact you would like to attest"
and says nothing about running it elsewhere,
retrying it,
or attesting a digest again;
`gh attestation --help` lists `download`,
`trusted-root` and `verify`,
and no command that creates an attestation.
So the supported way back is another workflow run that attests the image:
delete that version of the package in its settings (the limits are under **Restore**),
then click **Run workflow** on `main`.
The rebuild installs Bun from its latest installer,
so it has another digest than the deleted version,
and developers who already pulled the old one keep the old bytes in their local `monochromatic-playwright:<tag>`.

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

1.  Have an account with admin permission on the `Aquaticats` organization,
    which gives admin permission on a package the organization owns.
2.  Have the change that adds `.github/workflows/playwright-image.yml` on `main`.
    Its first push to `main` starts the workflow,
    because the workflow file is one of its own triggers.
3.  Have a terminal with `podman` and `sha256sum`,
    for the last check.

## Steps

Status:
TODO

1.  Open `https://github.com/Aquaticats/Monochromatic/actions/workflows/playwright-image.yml`.
    Expected:
    a run named **playwright-image** exists.
    If none exists,
    click **Run workflow**,
    keep **main** as the branch,
    and click **Run workflow** again.
2.  Open that run and wait for the job **Publish the Playwright image** to finish.
    Expected:
    every step is green,
    including **Push the image**,
    and the run page shows the warning
    **Playwright image is not public**.
3.  Open `https://github.com/orgs/Aquaticats/packages`.
    Expected:
    a package named **monochromatic-playwright** is listed.
4.  Click **monochromatic-playwright**.
    Expected:
    the package page lists a tag of 16 hex digits and links to the repository `Aquaticats/Monochromatic`.
5.  Click **Package settings**.
    Expected:
    the settings page of the package opens.
6.  Scroll to **Danger Zone** and click **Change visibility**.
    Expected:
    a choice of **Private** and **Public** appears.
    If **Public** cannot be chosen,
    ask an organization owner to open the organization's **Settings**,
    click **Packages** in the sidebar,
    and under **Package Creation** click **Public**.
    That is the setting GitHub documents for the visibility of packages that organization members create
    ([GitHub Docs, "Package creation visibility for organization members"][package-creation]);
    whether it also gates switching an existing package to public was not tested.
7.  Choose **Public** and confirm in the dialog.
    Expected:
    the package page shows **Public** beside its name.
8.  Open the run of step 1 again and click **Re-run all jobs**.
    Expected:
    the job **Publish the Playwright image** finishes green,
    the step **Compute the tag** prints `tag ` and the 16 hex digits,
    the step **Check whether the tag exists** prints the image and tag followed by `exists (HTTP 200); nothing to build`,
    and the steps **Build the image** and **Push the image** are skipped.

## What to check

Status:
TODO

1.  The step **Check that anyone can pull it** prints `anyone can pull ghcr.io/aquaticats/monochromatic-playwright:`
    followed by the tag,
    and no warning **Playwright image is not public** is on the run page.
2.  From a repository checkout,
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
3.  After the change that makes `prepare:playwright` pull first is merged
    (the `task-playwright-image` bin of `package/monorepo-task/core-exec`;
    before that the task builds locally),
    remove any local copy with `podman rmi monochromatic-playwright monochromatic-playwright:<tag>`
    and run `mise run prepare:playwright`.
    Expected:
    it prints `[prepare:playwright] pulled ghcr.io/aquaticats/monochromatic-playwright:` followed by the tag,
    and `podman image exists monochromatic-playwright` exits `0`.
    On a machine that still has a local `monochromatic-playwright:<tag>` the task prints
    `[prepare:playwright] using monochromatic-playwright:<tag> (already local)` instead and pulls nothing,
    which is also correct.

## Restore

Status:
TODO

The visibility change cannot be undone.
To remove the image:
open the package,
click **Package settings**,
and use the deletion option in the **Danger Zone**.
A local copy is removed with `podman rmi monochromatic-playwright`.

GitHub's limits on deleting and restoring a package
([deleting and restoring a package][deleting-doc]):

- "You cannot delete a public package if any version of the package has more than 5,000 downloads."
  Once the package is public and a version crosses that count,
  the package can no longer be deleted.
- A deleted package can be restored only "within 30 days of its deletion",
  and only if "The same package namespace is still available and not used for a new package."
- A package created again after a deletion is a new package,
  and GitHub says "the default visibility is private" for a new one
  ([package access control and visibility][visibility-doc]),
  so it has to be made public again by the same owner step.
  The deletion page says nothing about the visibility of a restored package,
  so check it after a restore.

Publishing the tag again after a deletion rebuilds `playwright.Dockerfile` with a fresh Bun,
because the file installs Bun from its latest installer.
The bytes under the same tag then differ from the deleted ones,
while developer machines that kept a local `monochromatic-playwright:<tag>` keep the old bytes.
The workflow publishes the tag again on its next run when the tag is missing
(a run with **Run workflow** on `main`).

[package-creation]: https://docs.github.com/en/packages/learn-github-packages/configuring-a-packages-access-control-and-visibility#:~:text=choose%20whether%20you%20want%20to%20enable%20the%20creation%20of%20public%2C%20private%2C%20or%20internal%20packages
[events-doc]: https://docs.github.com/en/actions/reference/workflows-and-actions/events-that-trigger-workflows#:~:text=This%20event%20will%20only%20trigger%20a%20workflow%20run%20if%20the%20workflow%20file%20exists%20on%20the%20default%20branch
[deleting-doc]: https://docs.github.com/en/packages/learn-github-packages/deleting-and-restoring-a-package#:~:text=You%20cannot%20delete%20a%20public%20package%20if%20any%20version%20of%20the%20package%20has%20more%20than%205%2C000%20downloads
[visibility-doc]: https://docs.github.com/en/packages/learn-github-packages/configuring-a-packages-access-control-and-visibility#:~:text=the%20default%20visibility%20is%20private
