# OpenCodeReview v1.13.3 cannot resume a saved session through a different repository path alias

## Symptom

On 2026-11-02,
`ocr scan --resume efa76c36-6fb5-484b-a7a4-c84b21a7ed39 --timeout 0`
from `/var/home/user/Monochromatic` reported:

```text
load resume session: open resume session "efa76c36-6fb5-484b-a7a4-c84b21a7ed39"
open /home/user/.opencodereview/sessions/var-home-user-Monochromatic/efa76c36-6fb5-484b-a7a4-c84b21a7ed39.jsonl: no such file or directory
```

These are excerpts of the diagnostic;
the complete message joins them and suggests `ocr session list`.
Changing to the parent directory changed the attempted session namespace to `var-home-user`.
The repository's default session listing showed an unrelated August session.

The requested file exists at:

```text
/home/user/.opencodereview/sessions/home-user-Monochromatic/efa76c36-6fb5-484b-a7a4-c84b21a7ed39.jsonl
```

The missing-file error is explained by the repository path alias.
It does not establish deletion of this session.
The investigation did not audit the machine for unrelated data loss.

## Root cause

Installed binary:
`open-code-review v1.13.3 (375f64c) linux/amd64`.
Source read at tag `v1.13.3`,
commit `375f64cd6f67a48a135da83cfcf59c52669a09cb`.

The filesystem inspection found `/home -> var/home`.
`stat` reported the same directory inode for `/home/user/Monochromatic` and `/var/home/user/Monochromatic`.
The saved `session_start.cwd` is `/home/user/Monochromatic`.

### Scan uses the supplied absolute path spelling

[`cmd/opencodereview/shared.go:153`][working-dir] resolves an absolute path,
then returns that spelling for the scan branch:

```go
// cmd/opencodereview/shared.go:153-163, 193
func resolveWorkingDir(input string, requireGit bool) (string, bool, error) {
    if input == "" {
        wd, err := os.Getwd()
        // Error handling omitted from this excerpt.
        input = wd
    }
    absPath, err := filepath.Abs(input)
    // Validation and the requireGit-only review branch omitted.
    return absPath, isGit, nil
}
```

[`cmd/opencodereview/scan_cmd.go:126`][scan] passes `false` for `requireGit`:

```go
// cmd/opencodereview/scan_cmd.go:126
cc, err := loadCommonContext(opts.repoDir, opts.rulePath, "", opts.maxTools, opts.maxGitProcs, false)
```

### Session storage encodes the spelling into a directory name

[`internal/session/persist.go:78`][encoding] transforms separators into hyphens:

```go
// internal/session/persist.go:84-98, selected statements
vol := filepath.VolumeName(p)
p = p[len(vol):]
p = strings.TrimLeft(p, "/\\")
p = strings.ReplaceAll(p, "/", "-")
p = strings.ReplaceAll(p, "\\", "-")
vol = strings.ReplaceAll(vol, ":", "_")
result := vol + p
```

Consequently,
the aliases select `home-user-Monochromatic` and `var-home-user-Monochromatic` respectively.

[`internal/session/resume.go:81`][resume] constructs that exact session path and opens it:

```go
// internal/session/resume.go:89
return filepath.Join(home, ".opencodereview", sessionSubDir, encodeRepoPath(repoDir), sessionID+".jsonl"), nil
```

```go
// internal/session/resume.go:112-118, selected statements
path, err := SessionFilePath(repoDir, sessionID)
f, err := os.Open(path)
if err != nil {
    return nil, fmt.Errorf("open resume session %q: %w", sessionID, err)
}
```

The scan lookup at [`cmd/opencodereview/scan_cmd.go:268`][scan-resume] calls this loader.
It does not search the other alias's session directory.

```go
// cmd/opencodereview/scan_cmd.go:268-270
state, err := session.LoadResumeState(repoDir, opts.resume)
if err != nil {
    return nil, fmt.Errorf("load resume session: %w (run 'ocr session list' to see available sessions)", err)
}
```

## Verification

### Working catalog

Read-only inspection succeeded:

```sh
# Run from the checkout; explicitly preserve the recorded repository spelling.
ocr session show efa76c36-6fb5-484b-a7a4-c84b21a7ed39 --repo /home/user/Monochromatic
ocr session list --repo /home/user/Monochromatic --limit 0
```

The target session reports `full_scan`,
status `aborted`,
one completed file,
one failed file,
and five comments.
All 42 JSON records in its 302477-byte file parse.
SHA-256 at inspection:

```text
f732967c4545aefa1f63740651a312e6b9b88061a66ee537f9641435f46d6de5
```

The final record is an `llm_request` without a trailing newline;
there is no `session_end` record.
This limits the conclusion to saved records,
not successful completion of the original scan.
The current resume reader stops before parsing an unterminated final record
([`internal/session/resume.go:129-133`][resume]).
The isolated scan probe nevertheless accepted the completed checkpoint.

### Failing catalog

The same read-only command with the physical path reproduces the lookup error:

```sh
# This selects a different storage directory despite identifying the same checkout.
ocr session show efa76c36-6fb5-484b-a7a4-c84b21a7ed39 --repo /var/home/user/Monochromatic
```

### Isolated scan probe

A private disposable home held an unchanged copy of the session under
`.opencodereview/sessions/home-user-Monochromatic/`.
Both invocations used that home and an intentionally unconfigured provider:

```sh
# OCR_PROBE_HOME names the disposable home containing the copied session, not the real home.
HOME="$OCR_PROBE_HOME" ocr scan --repo /var/home/user/Monochromatic \
  --resume efa76c36-6fb5-484b-a7a4-c84b21a7ed39 --timeout 0 \
  --provider ocr-session-lookup-probe-unconfigured
HOME="$OCR_PROBE_HOME" ocr scan --repo /home/user/Monochromatic \
  --resume efa76c36-6fb5-484b-a7a4-c84b21a7ed39 --timeout 0 \
  --provider ocr-session-lookup-probe-unconfigured
```

The physical spelling failed with `load resume session: open resume session` and `no such file or directory`.
The recorded spelling passed checkpoint loading and failed later with:

```text
Error: resolve LLM endpoint: resolve OCR config file: provider "ocr-session-lookup-probe-unconfigured" is not configured in custom_providers section because the config file does not exist
```

The source ordering places resume loading before provider setup
([`cmd/opencodereview/scan_cmd.go:168-179`][scan]).
No model request ran.
The original and copied session remained byte-identical.
This verifies checkpoint acceptance;
it does not claim that a subsequent full scan or provider request completed.

Local reproduction script:
`~/temp/agent/ocr-session-lookup-probe.ts`.
The job used `CPUQuota=100%`,
`MemoryMax=512M`,
`TasksMax=64`,
and `IOWeight=25`.

### Later saved progress

The listing also contains these sessions,
whose `session_start.resumedFrom` fields each name the requested session:

- `606e89ca-fe20-49e5-b202-11ac7375c9f4`,
  started October 19,
  lists 12 completed files and 25 comments.
- `bf11f85f-2a64-47bc-9742-edd1bea801d3`,
  started October 21,
  lists 125 completed files and 291 comments.
- `c6efff37-da26-4af6-bd48-241909499128`,
  started October 30,
  lists 24 completed files and 68 comments.
- `a1d021b9-b173-4762-a698-c2bdf84ffb70`,
  started November 1,
  lists 6 completed files and 21 comments.

The listing marks these runs `aborted` and reports one reused file in each.
They are separate children of the requested checkpoint,
not a verified cumulative chain.
Their complete logs and resume eligibility were not audited.
Do not infer that the newest session includes the October 21 run's progress.

## Verified workaround

Select the original repository spelling explicitly:

```sh
# Checkpoint lookup and acceptance verified using the isolated probe.
ocr scan --repo /home/user/Monochromatic \
  --resume efa76c36-6fb5-484b-a7a4-c84b21a7ed39 --timeout 0
```

Tradeoff:
the option selects this existing path namespace for the invocation;
it does not merge histories from other aliases or combine later runs.
The investigation did not move,
rewrite,
or delete saved sessions,
and did not start a live scan.

## What does not work

- Changing to `/var/home/user`:
  the reported error selects `var-home-user`,
  another storage namespace.
- Listing without the recorded `--repo` spelling:
  the user's output does not inventory the sibling namespace containing the missing session.
- Changing `--timeout`:
  the isolated probes use the same value and differ at filesystem lookup.
- Inferring full scan success from valid JSON:
  the saved session has an unfinished request and no terminal record.

## Upstream filing artifact

No new upstream report or comment was sent.
No storage migration or upstream patch is proposed by this incident diagnosis.

### Upstream filing decision

The `.out-of-scope/` catalog had no OpenCodeReview exemption on 2026-11-02.
Tracker searches included `session symlink`,
`session "symbolic"`,
and `encodeRepoPath`,
with issues and pull requests included.
The alias searches returned no matches.
The encoding search found issue `#894` and related work;
the issue and its comments were read in full using `gh issue view`.

1.  Upstream behavior:
    path spelling determines lookup;
    this incident is reproducible without deleting any saved data.
2.  Fixability:
    a future alias lookup could address discovery,
    but its identity checks and handling of existing directories would need design and testing.
3.  Supported use:
    explicit `--repo` selection is supported and verified;
    automatic lookup across symlink aliases is not established as a promised contract.
4.  Contributions:
    the inspected `CONTRIBUTING.md` permits AI assistance with disclosure and human understanding.
    No contribution is being requested here.
5.  Upstream direction:
    the September 24 maintainer response in `#894` deferred changes to the related path encoding
    because existing directories would require migration or dual lookup.
    That collision report is distinct from this alias lookup incident;
    it is relevant precedent against assuming a storage-key change would be welcomed.
6.  Prototype:
    the consumer workaround passed the isolated probe.
    No upstream change was prototyped or represented as ready to file.

Latest-release testing,
an upstream patch,
a public reproduction workflow,
and a filing-day trust/work-in-progress audit were not performed.
There is no filing recommendation.
Any later filing is the poster's decision and carries public-account and reputation risk.

[working-dir]: https://github.com/alibaba/open-code-review/blob/375f64cd6f67a48a135da83cfcf59c52669a09cb/cmd/opencodereview/shared.go#L153-L194
[scan]: https://github.com/alibaba/open-code-review/blob/375f64cd6f67a48a135da83cfcf59c52669a09cb/cmd/opencodereview/scan_cmd.go#L126-L179
[encoding]: https://github.com/alibaba/open-code-review/blob/375f64cd6f67a48a135da83cfcf59c52669a09cb/internal/session/persist.go#L78-L103
[resume]: https://github.com/alibaba/open-code-review/blob/375f64cd6f67a48a135da83cfcf59c52669a09cb/internal/session/resume.go#L81-L143
[scan-resume]: https://github.com/alibaba/open-code-review/blob/375f64cd6f67a48a135da83cfcf59c52669a09cb/cmd/opencodereview/scan_cmd.go#L264-L279
