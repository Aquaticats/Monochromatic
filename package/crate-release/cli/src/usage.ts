import {
  defaultAssetTemplates,
  defaultAttempts,
  defaultCurlMaxTimeSeconds,
  defaultMemoryMax,
  defaultPollIntervalSeconds,
  defaultRunListLimit,
  defaultTagTemplate,
  defaultUserAgent,
  defaultWorkflow,
} from './constants.ts';

/**
 Usage text of the release command, written from the defaults so the text and the behavior cannot drift.
 */
export const releaseUsage: string = `Usage: crate-release --crate-name <name> --crate-dir <dir> --scope <scope> [options] <from> <to> <note>

Moves a Rust crate from version <from> to <to>, checks it, dry-runs the publish and commits the bump.
Nothing is pushed or published. Quote a <note> that has spaces; put a lone -- before a <note> that starts with a hyphen.

Required:
  --crate-name <name>   crate name, as in its manifest and on crates.io
  --crate-dir <dir>     repository-relative directory of the crate (holds Cargo.toml and Cargo.lock)
  --scope <scope>       commit scope, for example git-policy/cli

Options:
  --pin-file <path>     repository-relative Cargo.toml that pins the crate exactly (=<version>); repeatable.
                        Each is moved with the crate and committed with the Cargo.lock beside it
  --check-bin <name>    build only this binary in the offline cargo check (default: the whole crate)
  --lock-task <task>    mise task that refreshes lockfiles after the edit; repeatable, run in order
  --memory-max <value>  systemd MemoryMax of the publish dry run (default ${defaultMemoryMax})
  --unit-name <name>    systemd unit name of the dry run (default <crate-name>-publish-dry)
  --root <dir>          repository root (default: the checkout that holds this package)
  --help                print this text
`;

/**
 Usage text of the wait command, written from the defaults so the text and the behavior cannot drift.
 */
export const waitUsage: string = `Usage: crate-release-wait --crate-name <name> [options] <sha> <version>

Waits for the publish workflow run of commit <sha>, watches it, then checks crates.io for <version> and the GitHub
release for the expected assets. Exits 0 only when all three are in place.

Required:
  --crate-name <name>        crate name, as on crates.io

Options:
  --workflow <file>          workflow file (default ${defaultWorkflow})
  --repo <owner/name>        repository for gh (default: read from the working directory)
  --attempts <count>         looks for the run (default ${String(defaultAttempts)})
  --poll-interval <seconds>  seconds between looks (default ${String(defaultPollIntervalSeconds)})
  --run-limit <count>        recent runs listed per look (default ${String(defaultRunListLimit)})
  --tag-template <text>      release tag; {crate} and {version} are filled in (default ${defaultTagTemplate})
  --asset-template <text>    expected release asset; repeatable (default ${defaultAssetTemplates.join(' and ')})
  --user-agent <text>        User-Agent for crates.io (default ${defaultUserAgent})
  --max-time <seconds>       seconds before the crates.io request gives up (default ${String(defaultCurlMaxTimeSeconds)})
  --root <dir>               repository root, where gh runs (default: the checkout that holds this package)
  --help                     print this text
`;
