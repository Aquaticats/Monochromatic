# QEMU qemu-img 11.0.1 advertises --compress but rejects it during convert

## Symptom

QEMU `qemu-img` 11.0.1 prints this option in `qemu-img convert --help`:

```text
-c, --compress
   create compressed output image (qcow and qcow2 formats only)
```

The documented long form fails before conversion starts:

```text
qemu-img convert: unrecognized option '--compress'
Try 'qemu-img convert --help' for more information
```

The short form `-c` succeeds with the same source,
target,
and formats.
This issue is unrelated to the CachyOS ZFS recovery procedure.
It appeared while creating a compressed copy of a retained qcow2 validation disk.

## Root cause

The source trace uses QEMU tag `v11.0.1` at commit
`6e9a825c1d4e7b62d072e99a89ecd1a74c7f0d55`.
The inspected clone has origin `https://github.com/qemu/qemu.git`,
which mirrors the upstream GitLab repository.

### The convert parser has separate short and long option registrations

`img_convert()` passes both a short-option string and a long-option table to `getopt_long()`.
The short string contains `c`,
but the table has no `compress` entry
(`qemu-img.c:2263-2300`):

```c
for(;;) {
    static const struct option long_options[] = {
        {"help", no_argument, 0, 'h'},
        {"source-format", required_argument, 0, 'f'},
        /* ... */
        {"no-create", no_argument, 0, 'n'},
        {"target-is-zero", no_argument, 0, OPTION_TARGET_IS_ZERO},
        {"force-share", no_argument, 0, 'U'},
        /* ... */
        {0, 0, 0, 0}
    };
    c = getopt_long(argc, argv, "hf:O:b:B:CcF:o:l:S:pt:T:nm:WUr:q",
                    long_options, NULL);
```

`getopt_long()` can therefore return `c` for `-c`,
but it has no mapping for `--compress`.
The exact diagnostic comes from option parsing before QEMU opens or converts the source image.

### The existing conversion branch already handles the short option

The parser switch enables compressed output when option code `c` is returned
(`qemu-img.c:2449-2459`):

```c
case OPTION_TARGET_IS_ZERO:
    /* ... */
    s.has_zero_init = true;
    break;
case 'c':
    s.compressed = true;
    break;
```

No compression implementation is missing.
Only the long spelling lacks registration.

### Help promises the missing spelling

The same function emits a help line that pairs both spellings
(`qemu-img.c:2303-2349`):

```c
case 'h':
    cmd_help(ccmd, "[-f SRC_FMT | --image-opts] [-T SRC_CACHE]\n"
/* ... */
"        [-n] [--target-is-zero] [-c]\n"
/* ... */
"  -c, --compress\n"
"     create compressed output image (qcow and qcow2 formats only)\n"
```

The help and parser table disagree inside the same function.

### The mismatch came from the convert-help refresh

Commit [`c02276223ff3`][introducing-commit],
`qemu-img: convert: refresh options/--help (short option change)`,
added many missing long-option registrations and introduced the explicit
`-c, --compress`
help text.
Its parser-table hunk did not add `compress`.
The July 2025 [mailing-list patch][introducing-patch] describes its purpose as:

```text
Add missing long options and --help output.
```

Source inspection found the omission in upstream master commit
`d2e570cc0f97b936902a5b1b86b73c0f5998b475`.
No master binary was built or run.
That revision still has the help text at `qemu-img.c:2349-2350`,
the `case 'c'` branch at `qemu-img.c:2458-2459`,
and this adjacent table sequence at `qemu-img.c:2287-2289`:

```c
{"no-create", no_argument, 0, 'n'},
{"target-is-zero", no_argument, 0, OPTION_TARGET_IS_ZERO},
{"force-share", no_argument, 0, 'U'},
```

This source match proves that the parser-table omission is not a downstream-only Flatpak patch.
It does not substitute for the current-master runtime reproduction requested by QEMU's issue template.

## Verification

### Inputs

- Installed binary:
  QEMU `qemu-img` 11.0.1 from the virt-manager Flatpak extension.
- Release source:
  QEMU `v11.0.1`,
  commit `6e9a825c1d4e7b62d072e99a89ecd1a74c7f0d55`.
- Current upstream check:
  master commit `d2e570cc0f97b936902a5b1b86b73c0f5998b475`.
- Source format:
  raw.
- Target format:
  qcow2.
- Input size:
  1 MiB sparse fixture.

The installed version probe was:

```bash
flatpak run \
  --command=qemu-img \
  org.virt_manager.virt-manager \
  --version
```

It printed:

```text
qemu-img version 11.0.1
```

### Reproduction harness

```bash
scratch=/var/home/user/temp/agent/qemu-img-compress-repro-current
mkdir --parents "$scratch"
truncate --size=1M "$scratch/input.raw"

flatpak run \
  --command=qemu-img \
  org.virt_manager.virt-manager \
  convert \
  --source-format raw \
  --target-format qcow2 \
  --compress \
  "$scratch/input.raw" \
  "$scratch/long.qcow2"
```

The command exits with status `1` and the quoted `unrecognized option` diagnostic.

### Patterns that work cleanly

- `qemu-img convert --help` prints the `-c, --compress` claim.
- `qemu-img convert -c` creates the qcow2 image.
- `--source-format raw` and `--target-format qcow2` are accepted in the same command.
- `qemu-img info --output=json` reports `"compression-type": "zlib"` for the output created with `-c`.

The successful short-form command was:

```bash
flatpak run \
  --command=qemu-img \
  org.virt_manager.virt-manager \
  convert \
  --source-format raw \
  --target-format qcow2 \
  -c \
  "$scratch/input.raw" \
  "$scratch/short.qcow2"
```

### Patterns that fail with the same diagnostic

Both tested positions exit with status `1`:

```bash
flatpak run \
  --command=qemu-img \
  org.virt_manager.virt-manager \
  convert \
  --compress \
  --source-format raw \
  --target-format qcow2 \
  "$scratch/input.raw" \
  "$scratch/long-before.qcow2"

flatpak run \
  --command=qemu-img \
  org.virt_manager.virt-manager \
  convert \
  --source-format raw \
  --target-format qcow2 \
  "$scratch/input.raw" \
  --compress \
  "$scratch/long-after.qcow2"
```

Moving the option does not change the parser failure.

### Minimal source prototype

The disposable upstream clone changed one parser-table line:

```diff
diff --git a/qemu-img.c b/qemu-img.c
index c42dd4e9..f633121a 100644
--- a/qemu-img.c
+++ b/qemu-img.c
@@ -2285,6 +2285,7 @@ static int img_convert(const img_cmd_t *ccmd, int argc, char **argv)
             {"sparse-size", required_argument, 0, 'S'},
             {"no-create", no_argument, 0, 'n'},
             {"target-is-zero", no_argument, 0, OPTION_TARGET_IS_ZERO},
+            {"compress", no_argument, 0, 'c'},
             {"force-share", no_argument, 0, 'U'},
             {"rate-limit", required_argument, 0, 'r'},
             {"parallel", required_argument, 0, 'm'},
```

The patched target was built inside a disposable Fedora 44 container with explicit limits:

```bash
podman run \
  --memory=2g \
  --cpus=2 \
  --rm \
  --volume /home/user/temp/agent/qemu-11.0.1-option.ctVjM95I:/work:Z \
  --workdir /work \
  registry.fedoraproject.org/fedora:44 \
  bash -lc '
    dnf install --assumeyes \
      diffutils gcc gcc-c++ glib2-devel meson ninja-build \
      pixman-devel python3 zlib-ng-compat-devel \
    && rm -rf /work/build-compress-option \
    && mkdir /work/build-compress-option \
    && cd /work/build-compress-option \
    && ../configure \
      --without-default-features \
      --enable-tools \
      --disable-system \
      --disable-user \
      --disable-docs \
      --disable-guest-agent \
    && ninja qemu-img \
    && truncate --size=1M /tmp/qemu-img-compress-input.raw \
    && ./qemu-img convert \
      --source-format raw \
      --target-format qcow2 \
      --compress \
      /tmp/qemu-img-compress-input.raw \
      /tmp/qemu-img-compress-output.qcow2 \
    && ./qemu-img info \
      --output=json \
      /tmp/qemu-img-compress-output.qcow2
  '
```

The targeted build completed all `547` Ninja steps.
The patched long-form conversion exited successfully.
`qemu-img info` reported a 1 MiB qcow2 image with `"compression-type": "zlib"`.
This proves that the one-line table entry reaches the existing compression branch.
It does not substitute for QEMU's complete test suite.

## Verified workarounds

### Use the implemented short option

Use `-c` with QEMU 11.0.1:

```bash
flatpak run \
  --command=qemu-img \
  org.virt_manager.virt-manager \
  convert \
  --source-format qcow2 \
  --target-format qcow2 \
  -c \
  source.qcow2 \
  destination.qcow2
```

This preserves the compression semantics implemented by QEMU.
The tradeoff is syntactic:
the command must use a short flag even in codebases that normally require long flags.
Inspect the resulting image before replacing or depending on it.

### Carry the one-line source patch only in a controlled build

The inline prototype makes `--compress` behave like `-c`.
The tradeoff is package ownership:
a locally built `qemu-img` no longer receives the installed package's updates automatically.
The recovery work therefore used `-c` instead of replacing the Flatpak binary.

## What does not work

- Trusting generated help as parser proof:
  help is a separate string and can name an unregistered option.
- Moving `--compress` before or after other convert options:
  both positions fail with status `1`.
- Retrying the same long form:
  the failure is deterministic in the parser table,
  not an image-content or timing failure.
- Treating the first prototype-build failure as a patch failure:
  the first container lacked `diff` and Meson stopped with
  `Program 'diff' not found or not executable`.
  Adding `diffutils` let configuration,
  compilation,
  and the behavioral test complete.
- Replacing the installed binary for this workflow:
  the verified `-c` workaround avoids an unnecessary local package-maintenance boundary.

## Upstream filing decision

No file under `.out-of-scope/` covers QEMU.
No external issue,
comment,
or patch was posted.
The user requires explicit authorization before any upstream filing.

GitLab title searches covered open and closed work items for:

- `compress long option`;
- `qemu-img convert help`;
- `unrecognized option`;
- `compress parser`;
- `compress option`.

The title-search harness first used `compression_type` as a positive control and returned issue #1345.
The first 4 target title searches returned no results.
The last returned unrelated issue #3198 and the known `compression_type` issue.
A separate exact-diagnostic search for `unrecognized option '--compress'` returned no results.
A broad `qemu-img --compress` body search produced unrelated work items and was not treated as null evidence.

No title or exact-diagnostic match for this parser/help mismatch was found.
Related issue [#1345][issue-1345] concerns documentation of `compression_type`.
Issue [#1959][issue-1959] requests compression-level control.
Issues [#80][issue-80] and [#81][issue-81] concern parallel compression and recompression.
None reports the rejected long option.
The 2024 and 2025 mailing-list option-refresh threads were also checked and contain no follow-up fixing this omission.

1.  **Is it really upstream's fault?**
    Yes.
    The release source and current master contain help for the long spelling while omitting it from the same command's
    option table.
2.  **Can upstream fix it?**
    Yes.
    A one-line table entry reaches the existing `case 'c'` behavior.
3.  **Are they supporting this use case?**
    Yes.
    QEMU supports compressed `qemu-img convert` output and its command help explicitly advertises `--compress`.
4.  **Would the repository welcome the contribution?**
    No,
    not from this artifact.
    `README.rst:131-146` routes upstream-release bugs to GitLab.
    `.gitlab/issue_templates/bug.md:1-14` asks reporters to test current master.
    `docs/devel/submitting-a-patch.rst:1-27` welcomes fixes and defines its mailing-list process.
    Its linked provenance policy explicitly declines AI-derived contributions
    (`docs/devel/code-provenance.rst:288-299`):

    ```text
    Current QEMU project policy is to DECLINE any contributions which are
    believed to include or derive from AI generated content.
    ```

    The same policy requires contributors to refrain from AI-generated patches
    (`docs/devel/code-provenance.rst:321-331`).
    Research and debugging may use AI only when their output is not included in a contribution.
    This report and prototype are AI-assisted,
    so neither may be submitted to QEMU.
5.  **Will upstream likely fix it?**
    Plausibly yes.
    The project has no stated non-goal for command-help accuracy,
    and the introducing series was specifically intended to add missing long options.
    No maintainer rejection was found.
6.  **Has a minimal compatible fix been prototyped?**
    Yes,
    for local diagnosis only.
    The one-line registration built in a bounded container and passed real conversion plus image-info checks.
    QEMU's AI-content policy prevents submitting this prototype.

Constraint 4 fails.
The user's explicit authorization requirement independently prevents publication.
Nothing from this document may be filed or sent upstream.
A future human may use the measured facts for independent research,
but must not submit this AI-generated wording or patch.

### Upstream filing artifact

Do not file as-is.
The retained draft is an audit record,
not a fileable contribution.

~~~md
Title: `qemu-img convert --help` advertises `--compress`, but the parser rejects it

Labels: kind::Bug

## Host environment

- Operating system: Bazzite 44
- Architecture: x86_64
- QEMU flavor: qemu-img from the virt-manager Flatpak extension
- QEMU version: 11.0.1

## Description

`qemu-img convert --help` prints `-c, --compress`, but using the long spelling exits with status 1:

```text
qemu-img convert: unrecognized option '--compress'
Try 'qemu-img convert --help' for more information
```

The short spelling `-c` succeeds with the same input and creates compressed qcow2 output.
Source inspection also found the omission in `qemu-img.c` at current master commit
`d2e570cc0f97b936902a5b1b86b73c0f5998b475`.
A current-master binary was not built or run.

## Steps to reproduce

```bash
truncate --size=1M input.raw
qemu-img convert \
  --source-format raw \
  --target-format qcow2 \
  --compress \
  input.raw \
  output.qcow2
```

## Source trace

In `qemu-img.c`,
`img_convert()` includes `c` in the short-option string and handles `case 'c'` by setting `s.compressed = true`.
Its `long_options[]` table does not contain `{"compress", no_argument, 0, 'c'}` even though the same function's help
prints `-c, --compress`.

The mismatch appears to originate in commit `c02276223ff312dd20d6535e243a94192342b170`,
which refreshed convert options and help.

## Suggested fix

Add this entry beside the other no-argument output controls in `img_convert()`:

```c
{"compress", no_argument, 0, 'c'},
```

A QEMU 11.0.1 build with that one-line change accepted the reproduction command and produced a qcow2 image whose
`qemu-img info --output=json` output reported `"compression-type": "zlib"`.

This draft and prototype were prepared with AI assistance.
QEMU's current provenance policy declines AI-derived contributions.
Do not submit this wording or patch.
~~~

[introducing-commit]: https://github.com/qemu/qemu/commit/c02276223ff312dd20d6535e243a94192342b170
[introducing-patch]: https://lists.nongnu.org/archive/html/qemu-devel/2025-07/msg04053.html
[issue-1345]: https://gitlab.com/qemu-project/qemu/-/work_items/1345
[issue-1959]: https://gitlab.com/qemu-project/qemu/-/work_items/1959
[issue-80]: https://gitlab.com/qemu-project/qemu/-/work_items/80
[issue-81]: https://gitlab.com/qemu-project/qemu/-/work_items/81
