# Depot trust and identity controls

Status:
work item 6 verification.
Scope:
fork and Dependabot admission,
publishing host,
secret boundary and cache isolation.

## Fork and Dependabot events

- Depot native workflows are dispatch-only.
  The two files under `.depot/workflows` are `current-baseline.yml` and `weekly-native.yml`;
  neither has a `push`,
  `pull_request`,
  `schedule`,
  or Dependabot trigger.
- A fork or Dependabot event therefore cannot start a Depot native workflow.
- The only GitHub workflow that requests a Depot-managed runner is
  `.github/workflows/current-baseline.yml`,
  and it is `workflow_dispatch` only.
- The advisory workflow runs on GitHub-hosted Linux and has no Depot runner.
- The repository's fork pull request approval policy is `first_time_contributors`
  from the GitHub Actions permissions endpoint.
  Returning external contributors can run GitHub-hosted workflows without a fresh approval,
  but those workflows do not request Depot runners.
- No Dependabot configuration file exists at the checked revision.
  If Dependabot is enabled later,
  its pull requests must be kept off any Depot runner label.

## Publishing host

Every publishing workflow runs on GitHub-hosted Ubuntu.
The checked workflows are `cargo-publish.yml`,
`npm-publish.yml`,
`npm-release.yml`,
`kotlin-linter-publish.yml` and
`pnpr-publish.yml`;
their jobs use `ubuntu-latest` or a GitHub-hosted Ubuntu matrix runner,
never a `depot-` label.
The music player release workflow also uses GitHub-hosted Ubuntu.
Publication stays outside Depot.

## Secret boundary

Depot CI has no secrets and no variables at this revision.
The native matrix workflow does not reference a secret.
GitHub Actions secrets stay in GitHub and are available only to GitHub-hosted workflows.
A Depot job therefore cannot read a publishing secret unless a future change explicitly adds one
to Depot,
which this rollout does not do.

## Cache isolation

- The release workflow's `detect` job sets `save_cache` only for `refs/heads/main` or a
  `workflow_dispatch`.
- The Gradle save uses a separate `save_gradle_cache` flag that is false until its controls pass.
- Pull request and merge group runs restore but never save.
- The cache decision requires main-only writes for every cache system.
- No Depot cache step exists in the native matrix.

## Acceptance

- A fork or Dependabot event cannot reach a Depot runner:
  verified by trigger inspection and the absence of Depot labels in PR-triggered workflows.
- A publish workflow stays GitHub-hosted:
  verified by the runner labels in the publishing workflows.
- Cache writes from non-main refs are rejected:
  verified by the `save_cache` and `save_gradle_cache` guards.

## Residual limits

- Fork approval is not `all_external`.
  This is acceptable for Depot isolation because fork workflows do not use Depot runners,
  but a future Depot-backed pull request profile would need a stronger policy first.
- The checks are configuration inspection,
  not a live fork or Dependabot event.
- Publishing workflows are unchanged by this verification.
