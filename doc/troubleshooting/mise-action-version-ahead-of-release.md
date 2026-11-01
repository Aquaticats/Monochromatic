# mise-action 4.0.1 installs the version `mise.jdx.dev/VERSION` advertises
and fails every job while that version has no GitHub release

## Symptom

Every job of `.github/workflows/npm-release.yml` for commit `80302aa1f` (run 34228368589,
 2026-09-08T12:50Z) failed in the step "Set up mise (skip auto-install of all tools)",
 before any repository code ran:

```text
[command]/usr/bin/curl -fsSL https://mise.jdx.dev/VERSION
2026.9.3
[command]/usr/bin/sh -c curl -fsSL https://github.com/jdx/mise/releases/download/v2026.9.3/mise-v2026.9.3-linux-x64.tar.zst | tar --zstd -xf - -C /tmp && mv /tmp/mise/bin/mise ...
curl: (22) The requested URL returned error: 404
zstd: /*stdin*\: unexpected end of file
tar: Child returned status 1
##[error]The process '/usr/bin/sh' failed with exit code 2
```

At the same time,
 from this machine:

```text
$ curl -sS https://mise.jdx.dev/VERSION
2026.9.3
$ curl -sI -o /dev/null -w '%{http_code}\n' https://github.com/jdx/mise/releases/download/v2026.9.3/mise-v2026.9.3-linux-x64.tar.zst
404
$ gh release list --repo jdx/mise --limit 1
v2026.9.2: Packslip Backend, SSH Relay, and Reimagined Install Progress   Latest   v2026.9.2   2026-09-07T22:25:21Z
$ curl -sI https://mise.jdx.dev/mise-latest-linux-x64.tar.zst | head -2
HTTP/2 200
content-length: 18029953
```

The version file names a release that does not exist on GitHub yet.
Every workflow in this repository that used `jdx/mise-action` without a `version` input
 (`npm-release.yml`,
 `cargo-publish.yml`,
 `cli-git-performance.yml` (deleted on 2026-10-13 in commit `724d70bfa7`),
 `cli-git-trust.yml` (deleted on 2026-10-13 in commit `642b6d72cb`),
 `kotlin-linter-publish.yml`,
 `logger-fuzz.yml`,
 `readonly-semantic-bridge.yml`,
 `toml-edit-fuzz.yml`)
 failed the same way during such a window.

## Root cause

### The action trusts the version file

`jdx/mise-action` at the pinned commit `1648a7812b9aeae629881980618f079932869151` (tag `v4.0.1`)
 reads the `version` and `fetch_from_github` inputs and hands them to `setupMise`
 (`src/index.ts:57`):

```ts
    const version = core.getInput('version')
    const fetchFromGitHub = core.getBooleanInput('fetch_from_github')
    await setupMise(version, fetchFromGitHub)
```

With no `version`,
 the version to install comes from `latestMiseVersion` (`src/index.ts:248`):

```ts
    let resolvedVersion = version || (await latestMiseVersion())
    resolvedVersion = resolvedVersion.replace(/^v/, '')
    let url: string
    if (!fetchFromGitHub && !version) {
      // Only for latest version
      url = `https://mise.jdx.dev/mise-latest-${await getTarget()}${ext}`
    } else {
      url = `https://github.com/jdx/mise/releases/download/v${resolvedVersion}/mise-v${resolvedVersion}-${await getTarget()}${ext}`
    }
```

and `latestMiseVersion` is one fetch of the version file (`src/index.ts:327`):

```ts
async function latestMiseVersion(): Promise<string> {
  const rsp = await exec.getExecOutput('curl', [
    '-fsSL',
    'https://mise.jdx.dev/VERSION'
  ])
  return rsp.stdout.trim()
}
```

`fetch_from_github` defaults to `"true"` (`action.yml:80`),
 so the default path is:
 read the version file,
 then download that exact version from the GitHub release.
Nothing checks that the release exists,
 and the action never consults the GitHub releases API on this path.
The action's maintainer explained on
 [jdx/mise-action#616](https://github.com/jdx/mise-action/issues/616) (2026-09-08)
 that reading the version file instead of the releases API
 "is done this way on purpose to prevent rate limit issues".

### Upstream publishes the version file before the GitHub release is public

`jdx/mise`'s `.github/workflows/release.yml` (read 2026-09-08 from the default branch) orders the steps this way.
The `release` job creates the GitHub release as a draft (`release.yml:565`):

```yaml
      - name: Create Draft GitHub Release
        id: create-release
        if: startsWith(github.ref, 'refs/tags/v') && steps.check-release.outputs.exists != 'true'
        run: |
          VERSION="$(./scripts/get-version.sh)"
          if [[ "$EXISTING_DRAFT" == "true" ]]; then
            gh release upload "$VERSION" --clobber "releases/$VERSION"/*
            RELEASE_ID="$EXISTING_RELEASE_ID"
          else
            RELEASE_URL="$(gh release create "$VERSION" \
              --title "$RELEASE_TITLE" \
              --notes-file /tmp/release-notes.txt \
              --verify-tag \
              --draft \
              "releases/$VERSION"/*)"
```

The very next step of the same job publishes to the CDN (`release.yml:589`):

```yaml
      - name: Publish Release Assets to CDN
        if: startsWith(github.ref, 'refs/tags/v') && steps.check-release.outputs.exists != 'true'
        run: mise x -- scripts/publish-release.sh
```

`scripts/publish-release.sh:14` writes the version file and `scripts/publish-s3.sh` uploads it
 (`publish-s3.sh:26`):

```bash
echo "$MISE_VERSION" | tr -d 'v' >VERSION
```

```bash
aws s3 cp "$RELEASE_DIR/VERSION" "s3://$AWS_S3_BUCKET/" --cache-control "$cache_day" --no-progress --content-type "text/plain"
```

The draft leaves draft state only in the later `publish-release` job,
 which waits for `attest-release` and `packslip` (`release.yml:717`):

```yaml
  publish-release:
    needs:
      - release
      - attest-release
      - packslip
    if: ${{ !cancelled() && needs.release.result == 'success' && needs.attest-release.result == 'success' && needs.packslip.result == 'success' && needs.release.outputs.created == 'true' }}
```

```yaml
          gh api --method PATCH "repos/$GITHUB_REPOSITORY/releases/$RELEASE_ID" -F draft=false
```

Assets of a draft release are not served from `releases/download/`,
 so between the CDN publish and the un-draft every unpinned `mise-action` job fails.
On 2026-09-08 the window ran from about 08:27Z
 (`Last-Modified` of `https://mise.jdx.dev/mise-latest-linux-x64`,
 which the same script uploads)
 to 13:55:29Z
 (`published_at` of `v2026.9.3` from `gh api repos/jdx/mise/releases/latest`),
 because the attestation stage failed and had to be rerun
 (a maintainer's comment on [jdx/mise#12934](https://github.com/jdx/mise/pull/12934):
 "Due to a recent CI failure,
 release v2026.9.3 has not been published at this time").
The same ordering produced the 2025.8.19 window reported as
 [jdx/mise-action#253](https://github.com/jdx/mise-action/issues/253).

### The CDN archives are not a way out

`publish-s3.sh:20` uploads the `mise-latest-*` files but excludes every archive extension:

```bash
# Upload mise-latest-* binaries (excluding archives to avoid conflicts)
# Note: --exclude must come before --include for proper filtering
aws s3 cp "$RELEASE_DIR" "s3://$AWS_S3_BUCKET/" --cache-control "$cache_day" --no-progress --recursive --exclude "*" --include "mise-latest-*" --exclude "*.tar.gz" --exclude "*.tar.xz" --exclude "*.tar.zst" --exclude "*.zip"
```

That exclusion arrived in `jdx/mise` commit `caafbe3a2` (2025-10-21,
 `fix(release): prevent S3 rate limiting errors during CDN upload (#6705)`);
 before it the trailing `--include "mise-latest-*"` re-included the archives.
Since then `https://mise.jdx.dev/mise-latest-<target>.tar.zst` has not changed
 (`Last-Modified: Tue, 21 Oct 2025 16:13:00 GMT`),
 while the bare `mise-latest-<target>` binary is refreshed on every release.
`mise-action` 4.0.1 and 4.2.0 download the archive form on the `fetch_from_github: false` path
 (`src/index.ts:253` quoted under "The action trusts the version file"),
 so that input installs mise 2025.10.13 today.
`mise-action` 4.3.0 downloads the bare binary instead (`src/index.ts:445` at tag `v4.3.0`).

### Which release the action installs with `minimum_release_age`

`mise-action` 4.3.0 (commit `c2a87611a18de5b3828c5652fe268e992400cb5c`,
 released 2026-08-25;
 the input landed in commit `bfedd8789`,
 "feat:
 add minimum release age for mise (#604)")
 resolves the version through the GitHub releases API when `minimum_release_age` is set and `version` is not
 (`src/index.ts:407` at tag `v4.3.0`):

```ts
  const useMinimumReleaseAge = !version && Boolean(minimumReleaseAge.trim())
```

```ts
async function latestMiseVersion(minimumReleaseAge?: string): Promise<string> {
  if (!minimumReleaseAge) {
    return downloadText('https://mise.jdx.dev/VERSION')
  }

  const cutoff = minimumReleaseAgeCutoff(minimumReleaseAge)
  let newestRelease: GitHubRelease | undefined
  for (let page = 1; ; page++) {
    const releases = await githubMiseReleases(page)
    for (const release of releases) {
      if (release.draft || release.prerelease) continue
      const releasedAt = new Date(release.published_at || release.created_at)
      if (Number.isNaN(releasedAt.getTime()) || releasedAt > cutoff) continue
```

`githubMiseReleases` sends the `github_token` input as a bearer token
 (default `${{ github.token }}`,
 `action.yml`),
 so the call counts against the job's own rate limit.
Drafts are skipped,
 which is exactly the state `v2026.9.3` was in during the window,
 and the chosen version is then downloaded from GitHub by its exact tag
 (`src/index.ts:443`,
 "An age-filtered release must be downloaded by its exact version from GitHub").
Without the input,
 `latestMiseVersion` still reads the version file,
 so the release age is what switches the resolution path.

### The release-age lookup can fail with 403

Run 38688512043 (`music-player-release.yml`,
 a push to `main`,
 2026-10-15) failed attempt 1 in two jobs with
 `##[error]GitHub releases API returned 403 Forbidden`:
 the Android job at "Set up mise (skip auto-install of all tools)" (05:46:17Z)
 and the macOS desktop job at "Install node (runs the repository's bundle:macos task)" (05:51:20Z).
Attempt 2 passed,
 and its Android job logged
 `Selected mise 2026.10.9, released 2026-10-13T14:53:38Z, with minimum_release_age=1d`.
The cause of the 403 (a rate limit,
 a secondary limit,
 or the token) was not established:
 the job logs carry no response headers.

The same diagnostic recurred in Depot runner-sizing run `38821765989` at 23:09 UTC on 2026-10-15.
All six jobs stopped in the mise action before compilation.
Trial commit `764194142b` selected published version `2026.10.9` and removed `minimum_release_age`.
The next run,
`38822116503`,
passed setup in every job and reached the compilation task.
This is an executed check of the pinned-version path;
it does not identify the server's reason for the original 403 or establish permanent service recovery.

The action treats the response as final.
In `githubMiseReleases` (`src/index.ts:985` at `c2a87611a18de5b3828c5652fe268e992400cb5c`):

```ts
    if (!response.ok) {
      const message = `GitHub releases API returned ${response.status} ${response.statusText}`
      if (
        response.status >= 400 &&
        response.status < 500 &&
        response.status !== 429
      ) {
        throw new NonRetryableError(message)
      }
```

so any 4xx except 429 fails the step without a retry.
That function is called only from `latestMiseVersion` (`src/index.ts:1013`),
 which `setupMise` calls only when no
 `version` input is given (`src/index.ts:413`):

```ts
  let resolvedVersion = cleanVersion(version)
  if (
    !resolvedVersion &&
    (!fs.existsSync(miseBinPath) || useMinimumReleaseAge)
  ) {
```

and `useMinimumReleaseAge` is `!version && Boolean(minimumReleaseAge.trim())` (`src/index.ts:407`).
With `version` set,
 the action logs "`minimum_release_age` is ignored because an explicit mise version was provided"
 (`src/index.ts:408`),
 so `minimum_release_age` does nothing and the pinned workflow does not set it.
The remaining requests go to `github.com/jdx/mise/releases/download/` (the archive and `SHASUMS256.txt`
 with its `.minisig`,
 `src/index.ts:685` to `src/index.ts:740`);
 `rg api.github.com src/index.ts` finds the releases
 lookup as the only API call.
The pinned version must be a published release with its assets:
 `gh release view v2026.10.9 --repo jdx/mise` shows `isDraft` false and the `linux-x64`,
 `macos-arm64` and
 `windows-*` assets.
The pin was checked only by reading the source and by the dry run of the pin's branch (recorded in
 `doc/decision/music-player-releases.md`);
 it has not met a 403 since.

## Verification

Version under test:
 `jdx/mise-action` commit `1648a7812b9aeae629881980618f079932869151` (`v4.0.1`),
 inputs `install: false`,
 `cache: true`,
 no `version`,
 `fetch_from_github` at its default.
Harness,
 reproducing the action's two requests from any shell:

```bash
version="$(curl -fsSL https://mise.jdx.dev/VERSION)"
curl -sI -o /dev/null -w '%{http_code}\n' "https://github.com/jdx/mise/releases/download/v${version}/mise-v${version}-linux-x64.tar.zst"
```

Harness for the release-age path,
 the query `mise-action` 4.3.0 runs (drafts and prereleases dropped,
 newest release older than a day):

```bash
cutoff="$(date --utc --date '1 day ago' +%Y-%m-%dT%H:%M:%SZ)"
gh api 'repos/jdx/mise/releases?per_page=100' --jq "[.[] | select(.draft | not) | select(.prerelease | not) | select(.published_at < \"${cutoff}\")] | first | .tag_name"
```

Patterns that fail:

- Any job during a window where the second command of the first harness prints `404`
   (2026-09-08,
   `2026.9.3`,
   from about 08:27Z until the asset answered 200 at 13:55Z,
   polled once a minute from 13:05Z).

Patterns that work:

- The same workflow once the release carries its assets
   (run 34111678810 on 2026-09-07 installed `2026.9.2` this way;
   the rerun of 34228368589 at 13:57Z installed `2026.9.3`).
- Every `jdx/mise-action` step at `v4.3.0` with `minimum_release_age: 1d`:
   commit `02fe85b3d` (2026-09-08) triggered `npm-release.yml`,
   `cli-git-trust.yml` (Linux and Windows; deleted on 2026-10-13 in commit `642b6d72cb`),
   `readonly-semantic-bridge.yml` (Linux and Windows),
   `logger-fuzz.yml`,
   and `toml-edit-fuzz.yml`.
   Every "Set up mise" step succeeded (runs 34254773194,
   34254772974,
   34254772966,
   34254772916,
   34254772888),
   and each log carries the line
   `Selected mise 2026.9.1, released 2026-09-02T13:48:24Z, with minimum_release_age=1d`
   on `ubuntu-latest`,
   `windows-latest`,
   and `macos-15` alike,
   followed by `mise --version` printing `2026.9.1`:
   `2026.9.2` (published 2026-09-07T22:25Z) was younger than a day at 17:03Z,
   and `2026.9.3` younger still.

## Verified workarounds

1.   Set `minimum_release_age: 1d` on every `jdx/mise-action` step and pin the action to `v4.3.0`
     (commit `c2a87611a18de5b3828c5652fe268e992400cb5c`).
     Applied to all thirteen steps in the eight workflows on 2026-09-08 (commit `02fe85b3d`).
     One day mirrors `minimumReleaseAge: 1440` in `pnpm-workspace.yaml`,
     the repository's existing release-age policy for npm packages.
     Tradeoffs:
     one authenticated GitHub API call per job;
     the installed mise lags a release by up to a day;
     the action ignores the input when `version` is set,
     so the two must not be combined.
     The cache key includes the resolved version on this path (`src/index.ts:530` at `v4.3.0`),
     so a new release also means one cold cache per workflow.
2.   Rerun the failed run once the release exists:
     `gh run rerun --failed <run id>`.
     Verified for run 34228368589 as recorded in `doc/planning/module-fs-path-deepening.md`.
     Tradeoff:
     manual,
     and the window length is upstream's;
     a run that must happen now cannot.
3.   Pin `version:` to a release known to exist.
     The action builds the GitHub URL from the pinned value and never reads the version file
     and never calls the releases API (see "The release-age lookup can fail with 403").
     Applied on 2026-10-15 to the two `jdx/mise-action` steps of `.github/workflows/music-player-release.yml`
     (`version: 2026.10.9`,
      without `minimum_release_age`).
     Not applied to the other workflows.
     Tradeoff:
     the pin needs a maintenance rule (the repository's `PIN` rule wants a comment saying why),
     no bot in this repository bumps action inputs,
     so the owner of the workflow updates it by hand:
     the steps are in `doc/runbook/release-the-music-player.md`,
      section "The mise version pin".

## What does not work

- `fetch_from_github: false` on `mise-action` 4.0.1 or 4.2.0:
   it downloads the CDN archive that upstream stopped refreshing on 2025-10-21
   (see "The CDN archives are not a way out"),
   so the job runs mise 2025.10.13.
   Measured 2026-09-08 by extracting `https://mise.jdx.dev/mise-latest-linux-x64.tar.zst` and running `mise --version`
   while the version file said `2026.9.3`.
   An earlier revision of this note listed the input as a workaround verified "at the HTTP level";
   the 200 was real,
   the content was a year old.
- `fetch_from_github: false` on 4.3.0 downloads the bare CDN binary,
   which was current (`2026.9.3`) throughout the window,
   but the version installed is whatever the CDN holds at that second with no release-age guard;
   not run in CI and not chosen.
- `minimum_release_age` on 4.0.1 or 4.2.0:
   the input does not exist at those tags (`action.yml` at `v4.2.0` has no such key),
   and the version-file path runs unchanged.
- Waiting inside the job:
   the action has no fallback on the release download,
   so a rerun is the only way to try again.
- Reading the version from `mise.lock`:
   this repository locks tools,
   not mise itself,
   so there is no committed mise version to pin to without adding one.

## Upstream filing decision

`.out-of-scope/` holds no entry for mise or mise-action (checked 2026-09-08 with `rg -i mise .out-of-scope`).

Existing reports of this behaviour,
 found with `gh search issues --repo jdx/mise-action 'fetch_from_github'`
 and `gh search issues --repo jdx/mise-action 'mise-latest'`:

- [jdx/mise-action#253](https://github.com/jdx/mise-action/issues/253),
   the 2025.8.19 window,
   closed 2025-08-22 once the release appeared.
- [jdx/mise-action#616](https://github.com/jdx/mise-action/issues/616)
   and [jdx/mise-action#617](https://github.com/jdx/mise-action/issues/617),
   the 2026.9.3 window,
   both closed on 2026-09-08 when the release was published.
   A commenter proposed resolving "latest" through `releases/latest`;
   the maintainer answered that the version file is read on purpose to avoid rate limits
   and that changing the order "may not be possible".
   Another commenter tried `minimum_release_age: 3mo` and saw `fetch failed` five times
   on a runner whose environment shows a custom tool cache;
   this repository's runs with `minimum_release_age: 1d` are the counter-evidence recorded under "Verification".

Walking the six constraints for the window itself:

1.   Is it really upstream's fault?
     Yes,
     and it is split:
     `jdx/mise` publishes the version file before un-drafting the release,
     and `jdx/mise-action` trusts the version file without consulting the release.
2.   Can upstream fix it?
     Yes.
     `jdx/mise` could run `publish-release.sh` from the `publish-release` job after the un-draft,
     or `jdx/mise-action` could fall back to the releases API on a 404.
3.   Are they supporting this use case?
     Yes:
     no `version` is the documented default,
     and 4.3.0 ships `minimum_release_age` precisely to pick published releases.
4.   Would the repo welcome our contribution?
     `jdx/mise-action` at `v4.0.1` has no `CONTRIBUTING.md`,
     no `AGENTS.md`,
     and no issue template;
     no ban on outside or AI-assisted reports was found in `README.md`.
5.   Will they likely fix it?
     Leaning no on the action side (the maintainer's #616 reply);
     no signal on the `jdx/mise` side.
6.   Have we prototyped a minimal fix compatible with their architecture?
     No,
     and none is needed for this repository:
     the shipped `minimum_release_age` input is the fix,
     applied here.

Decision:
 do not open a new issue;
 both trackers already hold the report.
The one thing this note adds that neither thread has is the exact `release.yml` ordering
 and a consumer-side fix that was run in CI.
Draft comment for #617,
 to post only with the owner's explicit approval because both threads are closed:

~~~md
For anyone else landing here: the ordering is in `jdx/mise`'s `release.yml`. The
`release` job creates the GitHub release with `--draft` and, in its very next
step, runs `scripts/publish-release.sh`, which writes `mise.jdx.dev/VERSION`.
The release leaves draft state only in the later `publish-release` job, after
`attest-release` and `packslip`. Draft assets are not served from
`releases/download/`, so every unpinned `mise-action` job fails between those two
points (about 08:27Z to 13:55Z on 2026-09-08).

Consumer-side fix that worked for us on `jdx/mise-action@v4.3.0`: set
`minimum_release_age: 1d` (any positive duration switches resolution to the
releases API, which skips drafts). Verified across five workflows including
`windows-latest` runners.
~~~

The stale CDN archives (see "The CDN archives are not a way out") are a separate behaviour and are not filed:
 the archive URLs are not documented,
 `jdx/mise`'s own installer downloads from GitHub releases (`packaging/standalone/install.envsubst:300`),
 and `mise-action` 4.3.0 no longer downloads them,
 so constraint 3 fails.

## Upstream filing decision for the releases API 403

Checked 2026-10-15 for the failure in "The release-age lookup can fail with 403".

`.out-of-scope/` holds no entry for mise or mise-action (`rg -i mise .out-of-scope` matched only unrelated text in
 `typescript-project-references.md` and `cargo-workspace.md`).
Duplicate search with `gh search issues --repo jdx/mise-action`:
 `"releases API"` found nothing;
 `"403"` found [jdx/mise-action#200](https://github.com/jdx/mise-action/issues/200) (closed 2025-06-25),
 which is about `mise` itself making unauthenticated requests and hitting the rate limit,
 not about the action's
 release lookup;
 `"minimum_release_age"` found #603,
 #617,
 #641,
 #664 and #538,
 none about a 403 from the lookup.
So no existing report covers it.

Walking the six constraints:

1.   Is it really upstream's fault?
     Not established.
     The cause of the 403 is unknown:
      the job logs carry no response headers and no message body.
     GitHub's documentation (`content/rest/using-the-rest-api/rate-limits-for-the-rest-api.md` in `github/docs`,
     read 2026-10-15) says that exceeding either the primary or a secondary rate limit returns `403` or `429`.
     If the 403 was a rate limit,
      the action's classification (any 4xx except 429 is final) turns a retryable
     condition into a failure;
      if it was a token or permission problem,
      the action behaved correctly.
2.   Can upstream fix it?
     Yes:
      retry a 403 that carries rate-limit headers,
      or fall back to the version file.
3.   Are they supporting this use case?
     Yes:
      `minimum_release_age` is a shipped input and its lookup is the documented path.
4.   Would the repo welcome our contribution?
     `jdx/mise-action` has a `CLAUDE.md` for agent work and no `CONTRIBUTING.md` and no issue template in
     its root or `.github` (checked by listing both);
      no ban on outside or AI-assisted reports was found.
5.   Will they likely fix it?
     No signal.
     On #616 the maintainer defended the version file over the releases API for rate-limit reasons.
6.   Have we prototyped a minimal fix?
     No,
      and none is possible without knowing what the 403 was.

Decision:
 do not file,
 and keep no draft.
A report needs the response status text,
 `x-ratelimit-remaining`,
 `retry-after` and the body of a failing call,
 which no log of this repository holds;
 the action does not print them.
If the 403 recurs for a workflow that does not pin `version`,
 capture those headers first.
