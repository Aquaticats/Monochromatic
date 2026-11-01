# Depot CLI 2.102.23 native SSH leaves paused-session readback unqualified

## Symptom

The bounded campaign exercised native CI debug access against application source
`e02be6c8b8fe9efb41a7f8f1a0b311c17796588a`.
Both attempts returned matching source-bound remote probe content.
The direct OpenSSH clients exited with `255`;
the native `depot ci ssh` PTY consumers exited with `0`.
Neither attempt produced the required paused-session connection target and final workflow readback.
The result qualifies the observed native PTY command consumer only.
Paused-action attachment and subsequent workflow resumption remain unqualified
([original runtime](../audit/depot-native-ssh-runtime-2026-10-18.evidence.json),
[final runtime](../audit/depot-native-ssh-correction-runtime-2026-10-18.evidence.json)).

A matching remote probe does not establish a successful direct-client lifecycle or paused-action attachment.
The bounded controllers cancelled both incomplete workflow attempts.

## Installer isolation

The remaining paused-session root cause is unresolved.

The original paused action last emitted an APT installation line.
That last log line does not establish the blocking call or an upstream defect.
The inspected `mxschmitt/action-tmate` implementation at
`35b54afac29c97fb54faba5b513f8fbd1882f113`
starts with the `tmate` executable name.
Its `install-dependencies` guard encloses both APT installation and static-binary download.
Setting that input to `false` therefore needs an independently available executable
([action entry point](https://github.com/mxschmitt/action-tmate/blob/35b54afac29c97fb54faba5b513f8fbd1882f113/src/index.js#L75)).

The deciding guard is `src/index.js:82`:

```javascript
// src/index.js:82
let tmateExecutable = "tmate"
if (core.getInput("install-dependencies") !== "false") {
```

The static download remains inside that guard at `src/index.js:111`;
the guard closes at `src/index.js:128`.
This explains the verified prerequisite isolation,
not the subsequent absence of a qualifying target.

The admitted correction downloaded the exact action-selected `tmate` version `2.4.0`,
checked retained archive and executable identities,
and installed it into the job's selected `PATH`.
The archive SHA-256 is `6e503a1a3b0f9117bce6ff7cc30cf61bdc79e9b32d074cf96deb0264e067a60d`.
This is a captured content identity,
not verification of an upstream signed checksum.
Offline executable-consumer and wrong-digest,
truncation and bit-change controls passed before allocation
([correction admission](../audit/depot-native-ssh-correction-2026-10-18.evidence.json),
[parent source checks](../audit/depot-native-ssh-correction-parent-admission-2026-10-18.evidence.json)).

The final run `sn950h8xcj`,
job `lxkdfrx3dh`,
attempt `bv33fqpvnp`,
emitted the selected preinstall success marker.
It still produced no qualifying paused-session target or final readback before watchdog cancellation.
The correction verifies the installation prerequisite;
it does not diagnose the remaining paused-action boundary.
Both admitted attempts are consumed.

## Status and allocation evidence

The CLI's custom status JSON omits allocation timestamps.
The deciding source is `pkg/cmd/ci/status.go` at
`1f96d6d488f938de97ea640b115e9567ebfd6fbf`.
Its selected attempt structure at `pkg/cmd/ci/status.go:164` contains no timestamps:

```go
// pkg/cmd/ci/status.go:164
type statusAttemptJSON struct {
    AttemptID         string `json:"attempt_id"`
    Attempt           int32  `json:"attempt"`
    Status            string `json:"status"`
    SandboxID         string `json:"sandbox_id,omitempty"`
    SessionID         string `json:"session_id,omitempty"`
    LogsCommand       string `json:"logs_command"`
    DownloadAvailable bool   `json:"download_available"`
    DownloadCommand   string `json:"download_command,omitempty"`
    ViewURL           string `json:"view_url"`
    SSHAvailable      bool   `json:"ssh_available"`
    SSHCommand        string `json:"ssh_command,omitempty"`
}
```

The scoped `CIService/GetWorkflow` response retains actual job-attempt timestamps.
The two exact owned workflow responses establish `136` and `138` seconds.
Their combined nominal two-CPU provider-window proxy is `USD 0.0274`,
distinct from the historical `USD 0.036` reservation
([terminal ledger](../audit/depot-owned-allocation-final-2026-10-18.evidence.json)).

Both attempts are terminal.
The correction receipt verifies owned client-process cleanup and askpass-socket absence.
Private connection material and raw authentication state are excluded from publication.
This investigation establishes no account-wide usage,
invoice,
network cause or general tmate-support conclusion.

## Verification

The real CLI version command returns `depot version 2.102.23 (2026-10-15)`.
The action source revision and content identities are retained in the correction admission.
The paused-action source and bounded transport controller are retained in its source archive;
both original and corrected run/attempt identities are terminal.

The passing observations are the offline selected-binary consumer,
wrong-digest/truncation/bit-change rejection controls,
real preinstall marker and source-bound native PTY command responses.
The nonqualifying observations are direct-client exit `255`,
absence of a qualifying paused-session target and absence of final workflow readback.
CLI exit `0` alone is not the paused-session acceptance condition.

The recorded read-only command shapes are:

```sh
# Native CLI observations, with the owned research organization selected.
depot --version
depot ci status sn950h8xcj --org "$DEPOT_RESEARCH_ORG" --output json
```

This review requires no new allocation;
both admitted runtime attempts have been consumed.

## Verified workarounds

No workaround for paused-session attachment or workflow resumption is verified.
The action-selected binary preinstall is verified prerequisite isolation only.
It requires the exact separately supplied executable on `PATH`
when `install-dependencies: false` bypasses both APT and static download.
The tradeoff is an additional explicit binary supply and content-identity check;
it neither upgrades tmate nor qualifies the missing paused consumer.

## What does not work

The original automatic installer did not reach the campaign's paused-session acceptance condition.
The same-version preinstall passed its own boundary but did not resolve that condition.
Treating the last emitted APT line as the established cause is unsupported.
Treating CLI status JSON's missing timestamp fields as absence of provider timestamps is incorrect:
the scoped workflow responses retain both actual attempt intervals.
Using a successful PTY probe as proof of paused-action attachment would also conflate consumers.

## Upstream filing decision

No issue or comment is filed.
The `.out-of-scope/` content search for `depot|tmate` has no match.
The exact all-state issue searches are `tmate` in `depot/cli`
and `depot` in `mxschmitt/action-tmate`;
both returned empty arrays on 2026-10-18.
Those queries do not establish absence of comparable SSH incidents.

- Upstream fault:
  not established at the remaining consumer boundary.
- Fixability:
  not assessed without a deciding failure trace.
- Supported combination:
  no general compatibility conclusion follows from these two attempts.
- Contribution policy:
  not assessed because no substantiated upstream defect is proposed.
- Maintainer intent:
  not inferred from empty issue queries or log silence.
- Minimal fix:
  no upstream patch is justified by the retained evidence.

There is nothing substantiated to add upstream.
The verified prerequisite isolation remains at the consumer boundary.
