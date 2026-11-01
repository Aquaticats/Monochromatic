# Changesets

Release tooling for the npm packages in this workspace,
 decided in `doc/decision/npm-publishing.md`.

- Add a changeset for a user-visible change:
   `mise run changeset:add`,
   or write `.changeset/<slug>.md` by hand with a frontmatter of package names to bump levels.
- See what the pending changesets will do:
   `mise run changeset:status`.
- The release workflow (`.github/workflows/npm-release.yml`) opens a "Version Packages" pull request
  when changesets are pending and publishes when a package version is missing from the registry.
- The first publish of any package is a manual step:
   `doc/runbook/publish-npm-package-first-time.md`.

`format` is `false`:
 changesets would otherwise auto-detect dprint and run it through `pnpm exec`,
 but this workspace removes the npm dprint package and provides dprint through mise,
 so the detected command fails on a runner.
Write changeset summaries one sentence per line;
 the repository markdown lint checks generated changelogs like any other Markdown.

`ignore` in `config.json` lists every public package that is not yet cleared for release.
Remove a package from that list when it is ready to ship;
 private packages need no entry.
