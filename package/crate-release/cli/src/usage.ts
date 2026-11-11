import {
  defaultAssetTemplates,
  defaultAttempts,
  defaultCurlMaxTimeSeconds,
  defaultCpuQuota,
  defaultFollowAttempts,
  defaultMemoryMax,
  defaultTasksMax,
  defaultPollIntervalSeconds,
  defaultPushAttempts,
  defaultReleaseHost,
  defaultRunListLimit,
  defaultTagTemplate,
  defaultUserAgent,
  defaultWorkflow,
  registryFileName,
  releaseHosts,
  releaseWorkflow,
} from './constants.ts';

/**
 Usage text of the release command, written from the defaults so the text and the behavior cannot drift.
 */
export const releaseUsage: string = `Usage: crate-release --crate <name> [options] <from> <to> <note>
       crate-release --crate-name <name> --crate-dir <dir> --scope <scope> [options] <from> <to> <note>

Moves a Rust crate from version <from> to <to>, checks it, dry-runs the publish and commits the bump.
Nothing is pushed or published. Quote a <note> that has spaces; put a lone -- before a <note> that starts with a hyphen.
Releases normally run on a GitHub runner, started by crate-release-dispatch; see the README.

Either name a crate that ${registryFileName} lists:
  --crate <name>        crate name on crates.io; its directory, scope, pins, check binaries and lock directories are
                        read from ${registryFileName}, so none of the options below may be given with it

or spell the crate out:
  --crate-name <name>   crate name, as in its manifest and on crates.io
  --crate-dir <dir>     repository-relative directory of the crate (holds Cargo.toml and Cargo.lock)
  --scope <scope>       commit scope, for example git-policy/cli
  --pin-file <path>     repository-relative Cargo.toml that pins the crate exactly (=<version>); repeatable.
                        Each is moved with the crate and committed with the Cargo.lock beside it
  --check-bin <name>    build only this binary in the cargo check (default: the whole crate)
  --lock-dir <dir>      repository-relative directory whose Cargo.lock gets "cargo update --workspace" after the
                        edit; repeatable, run in order

Options:
  --host <host>         where the release runs: ${releaseHosts.join(' or ')} (default ${defaultReleaseHost}).
                        local keeps cargo offline, caps the dry run in a systemd unit and accepts a manifest that
                        already holds <to>; runner lets cargo use the network, runs the dry run directly and refuses
                        a crate that already holds <to>
  --memory-max <value>  systemd MemoryMax of the dry run on a local host (default ${defaultMemoryMax})
  --cpu-quota <value>   systemd CPUQuota of the dry run on a local host (default ${defaultCpuQuota})
  --tasks-max <value>   systemd TasksMax of the dry run on a local host, threads included (default ${defaultTasksMax})
  --unit-name <name>    systemd unit name of the dry run on a local host (default <crate-name>-publish-dry)
  --root <dir>          repository root (default: the checkout that holds this package)
  --help                print this text
`;

/**
 Usage text of the dispatch command, written from the defaults so the text and the behavior cannot drift.
 */
export const dispatchUsage: string = `Usage: crate-release-dispatch --crate <name> [options] <from> <to> <note>

Starts the ${releaseWorkflow} workflow for a crate on GitHub and prints the URL of its run. Nothing runs on this machine
except gh: the workflow bumps the version, checks it, dry-runs the publish, commits and pushes the bump to the default
branch, and starts the publish workflow. Follow it with crate-release-await. Quote a <note> that has spaces; put a lone
-- before a <note> that starts with a hyphen.

Required:
  --crate <name>        crate name on crates.io, as ${registryFileName} lists it

Options:
  --dry-run             run everything except the push of the bump and the start of the publish; follow it with
                        gh run watch, since no publish run follows
  --repo <owner/name>   repository for gh (default: read from the working directory)
  --root <dir>          repository root, where gh runs (default: the checkout that holds this package)
  --help                print this text
`;

/**
 Usage text of the push command, written from the defaults so the text and the behavior cannot drift.
 */
export const pushUsage: string = `Usage: crate-release-push --crate <name> --repo <owner/name> --branch <branch> --scratch-dir <dir> [options]

Appends the head commit of this checkout to <branch> on GitHub through the GraphQL API, so that GitHub signs it: a branch
that requires signed commits refuses the unsigned commit a plain git push would send from a runner. The commit is built
on the head's parent, which must be the commit that was checked out, from the head's subject and the whole contents of
the files it changes.

GitHub refuses the commit when the branch has moved since the checkout, which a busy branch does during a release. Before
sending, the command records every path the release depends on: the files of the commit, the directory of the crate, the
directories whose lockfile the release refreshed, and the path dependencies in the closure that cargo metadata reports
for them. After a refusal for a moved branch it reads the new head and compares it with the checkout. When none of those
paths changed, it sends the same commit again on the new head; when one did, it pushes nothing, names the paths and says
to dispatch the release again. A branch that moves before every allowed send ends the push the same way.
The environment must hold GH_TOKEN, a token that may write the repository's contents.

Required:
  --crate <name>        crate that was released, as releasable-crates.json lists it
  --repo <owner/name>   repository to push to
  --branch <branch>     branch to append the commit to
  --scratch-dir <dir>   absolute directory for the request files, outside the checkout

Options:
  --max-attempts <n>    most sends, the first included (default ${String(defaultPushAttempts)})
  --root <dir>          checkout that holds the commit (default: the checkout that holds this package)
  --help                print this text
`;

/**
 Usage text of the await command, written from the defaults so the text and the behavior cannot drift.
 */
export const awaitUsage: string = `Usage: crate-release-await --crate <name> [options] <run> <version>

Waits for the ${releaseWorkflow} run <run> (its URL or numeric id, as crate-release-dispatch printed it) to end, then for
the ${defaultWorkflow} run that it started, then checks crates.io for <version> and the GitHub release for the expected
assets. The ${defaultWorkflow} run is the one that publishes the crate and builds the archives the release holds. Every
verdict comes from what a run says about itself, never from the exit status of a watcher. Exits 0 only when the release
run ended with success, the publish run ended with success, crates.io has the version and the release holds every
expected asset.

Required:
  --crate <name>             crate name on crates.io, as ${registryFileName} lists it

Options:
  --workflow <file>          publish workflow file (default ${defaultWorkflow})
  --repo <owner/name>        repository for gh (default: read from the working directory)
  --attempts <count>         looks for the publish run (default ${String(defaultAttempts)})
  --poll-interval <seconds>  seconds between looks (default ${String(defaultPollIntervalSeconds)})
  --follow-attempts <count>  looks at one run before giving up on its end (default ${String(defaultFollowAttempts)})
  --tag-template <text>      release tag; {crate} and {version} are filled in (default ${defaultTagTemplate})
  --asset-template <text>    expected release asset; repeatable (default ${defaultAssetTemplates.join(' and ')})
  --user-agent <text>        User-Agent for crates.io (default ${defaultUserAgent})
  --max-time <seconds>       seconds before the crates.io request gives up (default ${String(defaultCurlMaxTimeSeconds)})
  --root <dir>               repository root, where gh runs (default: the checkout that holds this package)
  --help                     print this text
`;

/**
 Usage text of the wait command, written from the defaults so the text and the behavior cannot drift.
 */
export const waitUsage: string = `Usage: crate-release-wait --crate-name <name> [options] <sha> <version>

Waits for the ${defaultWorkflow} run of commit <sha> to end, reading what the run says about itself, then checks
crates.io for <version> and the GitHub release for the expected assets. Exits 0 only when all three are in place.

Required:
  --crate-name <name>        crate name, as on crates.io

Options:
  --workflow <file>          workflow file (default ${defaultWorkflow})
  --repo <owner/name>        repository for gh (default: read from the working directory)
  --attempts <count>         looks for the run (default ${String(defaultAttempts)})
  --poll-interval <seconds>  seconds between looks (default ${String(defaultPollIntervalSeconds)})
  --follow-attempts <count>  looks at one run before giving up on its end (default ${String(defaultFollowAttempts)})
  --run-limit <count>        recent runs listed per look (default ${String(defaultRunListLimit)})
  --tag-template <text>      release tag; {crate} and {version} are filled in (default ${defaultTagTemplate})
  --asset-template <text>    expected release asset; repeatable (default ${defaultAssetTemplates.join(' and ')})
  --user-agent <text>        User-Agent for crates.io (default ${defaultUserAgent})
  --max-time <seconds>       seconds before the crates.io request gives up (default ${String(defaultCurlMaxTimeSeconds)})
  --root <dir>               repository root, where gh runs (default: the checkout that holds this package)
  --help                     print this text
`;
