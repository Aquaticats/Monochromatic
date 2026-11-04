/**
 Defaults and fixed protocol values of the crate release helpers.

 Every value a caller may want to change has a command-line option that overrides it; the constants only name the
 defaults and the accepted shapes, so no number or text appears bare in the logic.
 */

//region Release step defaults

/**
 Memory cap, as a systemd `MemoryMax` value, of the unit that runs the publish dry run; packaging a large crate
 compiles it, which can exhaust a shared host without a cap.
 */
export const defaultMemoryMax = '12G';

/**
 Suffix appended to the crate name to name the transient unit of the publish dry run when `--unit-name` is absent.
 */
export const defaultUnitSuffix = '-publish-dry';

/**
 Conventional commit type of a release commit.
 */
export const commitType = 'chore';

/**
 Longest release note, in characters, that a commit subject carries; longer notes belong in a commit body.
 */
export const maximumNoteLength = 200;

/**
 Exact-pin marker of a Cargo version requirement: `=1.2.3` accepts only that version.
 */
export const exactPinPrefix = '=';

/**
 Name of a Cargo manifest.
 */
export const manifestFileName = 'Cargo.toml';

/**
 Name of the lockfile that sits beside a Cargo manifest.
 */
export const lockFileName = 'Cargo.lock';

//endregion Release step defaults

//region Release hosts, registry and dispatch

/**
 Where a release runs. `local` is a developer machine: cargo works offline, the publish dry run is capped in a transient
 systemd unit, and a manifest that already holds the target is a safe re-run. `runner` is a fresh CI checkout: cargo
 may use the network, the dry run needs no cap, and a manifest that already holds the target means the bump is already
 on the branch, which is refused.
 */
export const releaseHosts = [
  'local',
  'runner',
] as const;

/**
 Host a release runs on when `--host` is absent.
 */
export const defaultReleaseHost = 'local';

/**
 File, beside this package's `package.json`, that holds the arguments of every crate that can be released: the one
 source the release command, the dispatch command, the wait command and the release workflow's choices all read.
 */
export const registryFileName = 'releasable-crates.json';

/**
 Workflow file that runs a release on a GitHub runner.
 */
export const releaseWorkflow = 'crate-release.yml';

/**
 Names of the release workflow's inputs, which the dispatch command fills and the workflow file declares.
 */
export const releaseWorkflowInputs = {
  crate: 'crate',
  from: 'from',
  to: 'to',
  note: 'note',
  dryRun: 'dry-run',
} as const;

//endregion Release hosts, registry and dispatch

//region Origin containment before a dispatch

/**
 Name of the Git remote that holds the repository the release workflow builds from.
 */
export const originRemote = 'origin';

/**
 Exit statuses of `git merge-base --is-ancestor`: zero when the first commit is contained in the second, one when it is
 not; any other status means Git could not answer.
 */
export const ancestorStatuses = {
  contained: 0,
  notContained: 1,
} as const;

/**
 Prefix of the branch a symbolic `HEAD` reference names in the answer of `git ls-remote --symref`.
 */
export const symbolicHeadPrefix = 'ref: refs/heads/';

//endregion Origin containment before a dispatch

//region Push of the release commit

/**
 Most times the push command sends the release commit before it gives up, the first send included. A send is refused
 when the branch moved since the checkout; each refusal that proves the movement harmless is followed by one more send on
 the new head, and a branch that keeps moving past this limit ends the push with a refusal.
 */
export const defaultPushAttempts = 6;

/**
 Words of GitHub's refusal of a commit whose expected head is no longer the head of the branch.
 */
export const staleHeadMessage = 'Expected branch to point to';

/**
 Most changed files the compare endpoint lists for one comparison, whatever the comparison holds: a list this long may
 have been cut, and then it cannot show that a path is unchanged.
 */
export const compareFileLimit = 300;

/**
 How a comparison of two commits stands, as the compare endpoint names it; only a head that is ahead of the base, or the
 base itself, is a branch that moved forward.
 */
export const comparisonStatuses = {
  ahead: 'ahead',
  identical: 'identical',
} as const;

/**
 Most changed files a log line lists by name; the rest of a longer list is counted.
 */
export const loggedPathLimit = 20;

//endregion Push of the release commit

//region Publish check defaults

/**
 Workflow file whose run publishes a crate when its version changes on the default branch.
 */
export const defaultWorkflow = 'cargo-publish.yml';

/**
 Runs of the workflow looked for before the wait gives up.
 With the default interval, forty-five attempts wait fifteen minutes, which covers the runner queue and the checkout.
 */
export const defaultAttempts = 45;

/**
 Seconds between two looks for the run.
 */
export const defaultPollIntervalSeconds = 20;

/**
 Most recent workflow runs listed on each look; a commit's run is among the latest when it was just pushed.
 */
export const defaultRunListLimit = 10;

/**
 Seconds before `curl` gives up on the crates.io answer.
 */
export const defaultCurlMaxTimeSeconds = 30;

/**
 `User-Agent` sent to crates.io, which rejects requests that do not name their caller.
 */
export const defaultUserAgent = 'crate-release-check';

/**
 Placeholder of a template that stands for the crate name.
 */
export const crateNamePlaceholder = '{crate}';

/**
 Placeholder of a template that stands for the version; every template must hold it, because without it every
 release would share one tag or one asset name.
 */
export const versionPlaceholder = '{version}';

/**
 Tag of a crate's GitHub release.
 */
export const defaultTagTemplate: string = `${crateNamePlaceholder}-v${versionPlaceholder}`;

/**
 Archive assets a crate's GitHub release is expected to hold: the x86_64 and aarch64 Linux tarballs.
 */
export const defaultAssetTemplates: readonly string[] = [
  `${crateNamePlaceholder}-${versionPlaceholder}-x86_64-unknown-linux-gnu.tar.gz`,
  `${crateNamePlaceholder}-${versionPlaceholder}-aarch64-unknown-linux-gnu.tar.gz`,
];

/**
 Base of the crates.io API address of one crate.
 */
export const cratesIoApiBase = 'https://crates.io/api/v1/crates';

/**
 Fewest hexadecimal characters accepted as a commit hash, the length Git itself abbreviates to.
 */
export const minimumShaLength = 7;

/**
 Most hexadecimal characters of a commit hash.
 */
export const maximumShaLength = 40;

//endregion Publish check defaults

//region Programs

/**
 Program that type-checks the crate.
 */
export const cargoProgram = 'cargo';

/**
 Program that starts a capped transient unit and waits for it.
 */
export const systemdRunProgram = 'systemd-run';

/**
 Version control program that records the release commit.
 */
export const gitProgram = 'git';

/**
 GitHub command-line client.
 */
export const ghProgram = 'gh';

/**
 HTTP client that asks crates.io.
 */
export const curlProgram = 'curl';

//endregion Programs

//region Locations and exit statuses

/**
 File whose presence marks the root of this repository, searched for upward from the package directory.
 */
export const repositoryMarkerFile = 'pnpm-workspace.yaml';

/**
 Scratch location under the home directory, as path segments: scratch never lives in the system temporary directory,
 whose backing device is unknowable.
 */
export const scratchSegments = [
  'temp',
  'agent',
] as const;

/**
 Exit status of a run that stopped because its input was refused or a step failed to finish its checks.
 */
export const failureExitCode = 1;

/**
 Exit status of a run that refused its command line before doing anything.
 */
export const usageExitCode = 2;

/**
 Milliseconds in a second, for turning the poll interval into a sleep.
 */
export const millisecondsPerSecond = 1_000;

/**
 Seconds in a minute, for writing the wait bound.
 */
export const secondsPerMinute = 60;

//endregion Locations and exit statuses

//region Accepted characters

/**
 Longest version text, in characters, accepted; Cargo versions are far shorter, and the bound keeps URLs and tags small.
 */
export const maximumVersionLength = 64;

/**
 Longest text accepted as a whole number option, so the value converts to a number exactly.
 */
export const maximumWholeNumberDigits = 10;

/**
 Characters of a crate or binary name after its first character, which must be a letter.
 */
export const crateNameCharacters = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789_-';

/**
 Characters of a commit scope: letters, digits and the separators of a package path.
 */
export const commitScopeCharacters = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789_.-/';

/**
 Characters of a systemd resource value such as `12G`, `500%` or `infinity`.
 */
export const systemdValueCharacters = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789.%';

/**
 Characters of a systemd unit name part: letters, digits, underscores and hyphens.
 */
export const unitNameCharacters = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789_-';

/**
 Characters of a branch name this package accepts: letters, digits and the separators of a path.
 */
export const branchNameCharacters = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789_.-/';

/**
 Characters of a workflow file name.
 */
export const workflowNameCharacters = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789_-.';

/**
 Characters of a GitHub owner or repository name.
 */
export const repositoryPartCharacters = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789_-.';

/**
 Hexadecimal digits of a commit hash, lower case.
 */
export const hexadecimalCharacters = '0123456789abcdef';

/**
 Characters of a version after its numeric core: identifiers of letters, digits and hyphens.
 */
export const versionIdentifierCharacters = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-';

/**
 Characters a shell reads literally even outside quotes; a word of only these needs no quoting in a printed command.
 */
export const shellPlainCharacters = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789_@%+=:,./-';

//endregion Accepted characters
