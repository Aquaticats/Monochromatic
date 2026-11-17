# Recover the DSH web profile on rc.3

Codex applied the recovery and verified live startup, session reads, and plugin activation.

## Recovery steps

1.  Keep the CLI on `0.2.2-rc.3`. Stage the version-scoped pnpm package extensions from the
    [startup diagnosis](../troubleshooting/dsh-0-2-2-rc-3-web-startup.md#verified-package-repair) in the
    installation that provides `dsh`. Verify that the CLI still reports rc.3 and that the Typert host module
    imports through `dsh-base`. The native loader extension preserves resolution of the Linux x64 glibc
    prebuilt module after reinstalling into the global virtual store.
2.  Stop hosts using the affected DSH home before editing its registry. An active writer could overwrite the
    repaired state. Keep a private backup of `storages/workspace.json` and check the saved state again with
    writers stopped. Keep the older workspace ID and title at the first position previously held by either
    record. Add the newer record's session membership ahead of the older record's existing list, preserving
    the order within each list. Remove the duplicate ID from both `tables.workspaces` and `global.workspaceIds`.
    Keep the default workspace, archive and pin arrays, session files, and other workspace records intact.
    Validate the candidate with the installed DSH schemas and registry validator, then replace the file
    atomically with its private permissions preserved. The backup supports rollback before reopening the profile.
3.  Start the web profile and verify authenticated workspace and session requests. Confirm that both membership
    lists are represented in the surviving workspace, archived sessions are still archived, and a session from
    each source record can be opened. The in-memory check of the saved registry retained all memberships and
    archive flags. The disposable HTTP test exercised the repaired registry with synthetic workspace data.
4.  Upgrade Advisor, NeuralWatt, and Qwen Token Plan Individual together with their DSH catalog dependencies.
    Build and test each package against rc.3 and verify activation through a copied web profile before recording
    rc.3 as an accepted host in the plugin manifests. This keeps the compatibility claim tied to an exercised
    release. The base web recovery can ship before those optional bundles pass their checks.

The two registry records both held sessions. Dropping one row would discard its workspace membership.
The installed registry rejected the original state and accepted the proposed merge in memory. The set of session
IDs and the global archive and pin arrays matched the input after the merge.

## Live verification at 2026-11-17T17:01:07-05:00

- The active DSH installation remained on `0.2.2-rc.3`. Its `dsh-base` bundle resolved the `dsh-web/typert`
  host artifact with `zod` linked to the importing package.
- The workspace merge retained the older ID. The repaired registry preserved the complete session membership,
  archive and pin arrays, default workspace, and private permissions. Codex retained a private copy of the
  original registry and verified session event reads from both original workspace records.
- `d03c8d020c` updates the DSH catalog and regenerated lockfile. Advisor's workspace build needs the new
  `dsh-error` peer of `dsh-llm`, `dsh-sandbox`, `dsh-attachment`, and `dsh-working-directory`, so Advisor's
  development dependencies provide it explicitly. The frozen install passed.
- Advisor, NeuralWatt, and Qwen Token Plan Individual passed their type checks, builds, and unit suites.
  The copied profile reported all three plugin fibers active, then `2a72656b12` recorded the verified host
  version in their manifests. The live profile also reported all three active and served authenticated session
  reads. The verification process exited and released the listener.

## Fallback

Rc.1 is the declared host version for the three local plugins. Its published `dsh-web` package predates the
Typert export that requires `zod`, and a disposable rc.1 install passed the workspace and session API smoke test
with a unique-path registry. It still requires the workspace merge. Treat a rollback as a fallback on copied
data until affected session logs have been opened successfully. The rc.1 reader allows catalog payload versions
0 through 2. Rc.3 also accepts version 3 for native V4 subagent catalog events. [rc.1 reader][rc1-catalog] ·
[rc.3 reader][rc3-catalog]

Repairing rc.3 ranks first because it preserves the current session reader and restores the workspace APIs with
verified package and registry fixes. Rc.1 ranks second because it restores declared plugin compatibility at
the cost of validating backward readability and the current profile's configuration.

[rc1-catalog]: https://github.com/deepseek-ai/deepseek-harness/blob/66491b05e3f05ad35bcdd4f6cd7ed26041d9e3db/packages/session/session-format-v3-to-v4/src/facts.ts#L72-L82
[rc3-catalog]: https://github.com/deepseek-ai/deepseek-harness/blob/38a6a5c965aa99ab525161efb7cc5cf1fe62fced/packages/session/session-format-v3-to-v4/src/validation.ts#L138-L156
