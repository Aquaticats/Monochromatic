# JetBrains Context 263.7307.101 reports indexing exit code 1 in IntelliJ IDEA 263.7307.112

## Symptom

Investigated on 2026-11-02 after repeated notifications:

```text
Context indexing failed: Context CLI indexing failed with exit code 1
```

The affected project is `Monochromatic`.
`$HOME/.cache/JetBrains/IntelliJIdea2026.3/log/idea.log:24-27` records the failing invocation:

```text
index --json-output --project-path=<project> --revision=<commit> --index-alias=code-blocks
Context CLI indexing failed with exit code 1
at com.intellij.embark.cli.EmbarkCliIndexingClient.index(EmbarkCliIndexingClient.kt:56)
```

The same log at lines 133-137 records both a failing `status` invocation and another indexing failure.
The command environment contains `JETBRAINS_AI_PLATFORM_TOKEN=<redacted>`.
Do not copy its actual value into diagnostics.

## Root cause

The IDE-authenticated CLI refuses to proceed until the Air Context CLI EAP User Agreement is accepted.
Temporary output capture during normal IDE activity reproduced this for both `status` and `index`.
Both processes exited 1 with an IDE token present and this message on stderr:

```text
This Early Access build of Air Context requires accepting the Air Context CLI EAP User Agreement
(https://www.jetbrains.com/legal/docs/terms/jetbrains-context-cli-eap/).
Accept it in an interactive terminal, or set JBCONTEXT_ACCEPT_EAP_TERMS=1
(plus JBCONTEXT_ACCEPT_EAP_EMAIL=<email> to record it) to accept non-interactively.
```

The diagnostic is one line in the actual JSON error;
it is wrapped here for readability.
Its stack identifies `EapAgreementGate.requireAcceptance(EapAgreementGate.kt:221)`,
`EapAgreementGate.ensureAccepted(EapAgreementGate.kt:200)`,
and `IsLoggedInGuard.authenticate(IsLoggedInGuard.kt:154)`.
The indexing branch then names `IndexCommand.kt:213`;
the status branch names `StatusCommand.kt:94`.
These are observed stack frames,
not independently inspected private source.

The installed plugin descriptor is
`$HOME/.local/share/JetBrains/IntelliJIdea2026.3/embark/lib/embark.jar!/META-INF/plugin.xml:2-4`:

```xml
<!-- META-INF/plugin.xml inside the installed embark.jar -->
<name>JetBrains Context</name>
<id>com.intellij.embark</id>
<version>263.7307.101</version>
```

Its descriptor describes a dedicated **Semantic** tab in **Search Everywhere**.
The notification comes from this plugin's CLI integration.
This evidence does not establish a failure of IntelliJ's ordinary project indexes.

The plugin bytecode was inspected with `javap`.
`EmbarkCliIndexingClient.indexingFailureMessage` contains distinct messages for recognized
authentication failure,
missing index,
and generic failure:

```text
Context CLI indexing failed with exit code <code>: authentication required
Context CLI indexing failed with exit code <code>: missing index
Context CLI indexing failed with exit code <code>
```

`EmbarkCliJson.classifyFailureMessage` recognizes `authentication required`,
`not authenticated`,
and `login`.
These are installed-bytecode observations,
not a source-level diagnosis of the private CLI implementation.

The public `JetBrains/context` repository was cloned at
`b691e901d262b1041531be70c60f1a10f3f8adec`.
Its `README.md:7` explicitly says the CLI is developed in a private repository.
The clone contains public integration assets rather than the CLI implementation.

## Verification

Observed versions:

- IntelliJ build `IU-263.7307.112` from the installation's `build.txt`.
- JetBrains Context plugin `263.7307.101` from its installed descriptor.
- Air Context CLI `0.9.18`,
  build `1257`,
  commit `2c195c2336`,
  channel `stable`,
  from `jbcontext --version`.

The read-only CLI probes used a scope with `CPUQuota=100%`,
`MemoryMax=1G`,
`TasksMax=64`,
and `IOWeight=25`.

Successful probes:

- `jbcontext --help`,
  `jbcontext status --help`,
  `jbcontext login --help`,
  and `jbcontext upgrade --help` exit successfully.
- `jbcontext upgrade --check` reports
  `already on the latest version (channel: stable)`.

Failing probe:

```bash
# Run from the Monochromatic checkout; this queries status and does not request indexing.
systemd-run --user --scope \
  --property=CPUQuota=100% --property=MemoryMax=1G \
  --property=TasksMax=64 --property=IOWeight=25 \
  "$HOME/.jbcontext/bin/jbcontext" status --json-output \
  --project-path="$PWD" --index-alias=code-blocks
```

This exits 1 and emits:

```json
{"type":"error","message":"Authentication required"}
```

The actual JSON also contains a stack trace through `IsLoggedInGuard.reportNoAuth`.
This terminal invocation lacks the IDE-supplied token,
so it is a separate failure and does not reproduce the authenticated IDE failure.
The subsequent IDE-authenticated capture reproduced the actual agreement gate at
`2026-11-02 23:25:02` for status and `23:25:04` for indexing.

The CLI log at `$HOME/.jbcontext/logs/jbcontext.log` contains certificate-loading entries
at the IDE failure times but no corresponding failure detail.
The IDE's local MCP tool catalog was read through its HTTP interface;
it exposes project and debugger operations but no direct Context authentication or diagnostic action.

## Verified workarounds

Acceptance is being applied;
successful indexing remains unverified.
The user chose "Keep Semantic search and diagnose" on 2026-11-02.
After the agreement requirement was identified,
the user explicitly instructed "Accept it for me."
The plugin remains enabled.
The supported `JBCONTEXT_ACCEPT_EAP_TERMS=1` mechanism is being applied to an IDE-authenticated
invocation so the acceptance uses the same authentication environment as the failing command.
No synthetic account identity is supplied.

A temporary launcher wrapper captures stdout and stderr of ordinary IDE-triggered
`status` and `index` commands with inherited token values redacted.
It forwards the original output and exit status.
The observation controller restores the original launcher after an authenticated indexing failure
or after a 60-second deadline,
and compares the restored bytes with the original.
The initial capture restored the launcher successfully and compared its bytes with the original.
The acceptance capture uses the same restoration mechanism.
The diagnostic `jbcontext login` invocation opened browser authentication and was canceled before completion.
It did not accept the agreement.

## What does not work

- Treating absent help text as proof that `--index-alias` is unsupported:
  the real invocation accepts the argument and reaches authentication.
  The initial version-mismatch hypothesis was rejected.
- Upgrading based only on the notification:
  the installed CLI's update check reports the latest stable version.
  Other release channels were not tested.
- Attributing the IDE failure to the terminal's `Authentication required` response:
  the authentication environments differ,
  and the plugin has a distinct recognized-authentication message.
- Prescribing cache invalidation:
  there is no evidence connecting IntelliJ project-cache corruption to this CLI failure.

## Upstream filing decision

No report was posted or recommended.
The runtime error establishes an unaccepted agreement rather than an indexing implementation defect.
The generic IDE notification omits that actionable detail.
Posting a weak report risks wasting maintainers' time and the poster's reputation;
filing remains the poster's decision.

- Upstream fault:
  the CLI enforces an agreement gate;
  the notification's lack of detail is distinct from that expected gate.
- Fixability:
  unknown because the private CLI implementation is unavailable.
- Supported use case:
  the installed plugin explicitly advertises semantic search and repository indexing.
- Contribution acceptance:
  not audited because no concrete defect is established.
- Maintainer intent:
  not established for this incident.
- Prototype:
  none;
  no identified cause supports a corrective patch.

No matching exemption was found among `.out-of-scope/` file names.
Tracker queries used `gh search issues --repo JetBrains/context` with `indexing`,
`"exit code 1"`,
`EAP terms`,
and `agreement`,
and `gh search prs --repo JetBrains/context indexing`.
The issue search found issue 39 about repeated CLI login;
its comments describe a daemon returning `204 No Content`,
which is not present in this incident's observed log.
The PR search found merged PR 10 about installation-skill wording,
not the IDE failure.
These are related search results,
not confirmed duplicates or fixes.

## Remaining verification

Verify acceptance persists after restoring the original launcher,
then confirm IntelliJ's index operation completes and semantic search returns results.
The EAP agreement is version 1.0,
effective July 15, 2026,
as fetched from JetBrains on the investigation date.
