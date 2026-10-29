# Depot standalone linker experiment

> Moved 2026-10-29 by the generic package category elimination
> (`doc/planning/generic-category-elimination.md`);
> paths and names below this note are historical:
> `package/cli/forbidden-strings` is now `package/forbidden-strings/cli`;
> `package/rust-module/forbidden-regex` is now `package/forbidden-strings/core`.

## Scope and execution manifest

This experiment exercises a useful standalone agent workload through Depot's Sandbox SDK.
It compares the same compiled scanner inputs through `lld` and `mold`;
it does not select or adopt a replacement linker.

The source archive contains `package/cli/forbidden-strings` and its path dependency
`package/rust-module/forbidden-regex` from trial commit `d6deca5fa5`.
No host home directory,
repository mount,
or release credentials enter the sandbox.
The SDK control process reads the existing Depot login locally.
The cloud workload receives no Depot token.

Sandbox `pt4gd92356psxm9` reports Ubuntu 24.04.5,
2 vCPUs,
2,048 MiB memory,
and 150 GB disk despite the requested 100 GB.
The sandbox expires after 45 minutes and is explicitly stopped after artifact retrieval.
Compilation and linking stay inside this resource boundary.

Setup uploads the existing local `mise` executable and immutable source archive.
Ubuntu package installation provides `lld`,
`mold`,
and compiler prerequisites;
`mise` installs Rust into the disposable environment.
Dependency downloads use the package's existing Cargo lockfile.
The runner records resolved tool versions and help before invoking linker options.

The controlled phase retains the final compiler-driver arguments and temporary object inputs,
warms each linker,
measures repeated unchanged-input baselines,
then alternates the two linkers with the same output destination.
It checks the produced scanner against passing and failing disposable inputs.
Raw timings,
input hashes,
output hashes,
and consumer results are retrieved before cleanup.
Timing differences are interpreted against the unchanged-input spread.

## Results

The experiment completed and the sandbox reached `finished` after explicit cleanup.
The scanner compiled once in the debug profile with Rust 1.99.1.
The measured linkers were Ubuntu LLD 18.1.3 and mold 2.30.0,
both invoked through Clang 18.1.3 with two linker threads.
This is a comparison of the installed distribution versions,
not the newest releases or a release-profile build.
[Complete measurements and consumer output](depot-linker-experiment-2026-10-16.evidence.json)

The harness retained 197 compiler-driver arguments and temporary object inputs.
It produced 40 executables,
with 160 consumer invocations verifying version,
allowed input,
rejected input,
and missing-rule failure.
Each linker produced one stable executable hash throughout its runs.
ELF comments independently identified the selected linker.
The timing probe's intentional 250-millisecond delay measured 251.668 milliseconds.

The unchanged-input baseline contained seven samples per linker:

- LLD median:
  205.721 milliseconds;
  range 197.092 to 220.065 milliseconds.
- mold median:
  198.496 milliseconds;
  range 191.915 to 236.443 milliseconds.

The subsequent alternating comparison contained twelve samples per linker,
reversing their order every round:

- LLD median:
  202.156 milliseconds.
- mold median:
  217.002 milliseconds.

The median difference is 14.847 milliseconds,
smaller than both unchanged-input baseline spreads.
The baseline and alternating medians also reverse their ordering.
This experiment therefore does not establish a linker performance winner.
An independent evidence review checked the arguments,
sample counts,
consumer statuses,
hashes,
and arithmetic without rerunning the workload.

## Retained artifacts and usage

The source archive,
LLD input reproducer,
both final executables,
harness scripts,
and logs are retained under `~/temp/agent/depot-linker-20261016`.
The compressed artifacts were downloaded and decompressed through a hash verifier.
Their hashes match the original remote files;
the evidence JSON records both compressed and decompressed hashes.
The decompressed reproducer is 300,454,912 bytes;
the LLD and mold executables are 78,689,928 and 83,538,928 bytes.

The observed sandbox lifetime was 1,204.403 seconds on two vCPUs.
At the documented SDK compute rate,
its public overage equivalent is **USD 0.1204403**.
This includes setup,
interactive pauses,
a corrected harness assertion,
artifact transfer retries,
and cleanup.
It is neither an optimized automatic-job duration nor an account invoice.
Allowance attribution and storage remain outside the accessible account evidence.
[SDK billing](https://depot.dev/docs/api/overview#sandbox-api)

## Harness corrections and operational limits

The initial directory choice attempted to create a child of `/home` as an unprivileged user.
After probing the actual identity and home directory,
the harness used `/home/runner/linker`.
The first consumer assertion looked for scanner findings only on stdout;
the scanner's diagnostic path uses stderr,
so the corrected probe checks both streams while preserving exact expected exit statuses.
Neither harness error changes the retained timing samples.

Uncompressed artifact retrieval was cancelled after command completion,
then compressed artifacts were retrieved with exact-length and hash checks.
The direct streaming retrieval retained partial files until their complete size was verified.
No partial file was accepted as a completed artifact.

Some completed local SDK clients remained alive;
the bounded reproduction and later passing control are tracked in
[#1051](https://github.com/Aquaticats/Monochromatic/issues/1051)
and the [client-lifecycle record](../troubleshooting/depot-sandbox-client-exit.md).
The remote sandbox was stopped independently after successful artifact retrieval.
