# Starship 1.26.0: cli-git `for-each-ref` startup exceeds prompt command timeout

Note 2026-11-09T16:47:44-05:00:
`package/git-policy/cli/src/read-only-forwarding.unit.test.ts` left the tree on 2026-10-13 in `642b6d72cb`.

## Symptom

After an interactive command returns in the Monochromatic repository,
Starship can print:

```text
[WARN] - (starship::utils): Executing command "/var/home/user/Monochromatic/node_modules/.bin/git" timed out.
[WARN] - (starship::utils): You can set command_timeout in your config to a higher value to allow longer-running commands to keep executing.
```

The prompt still renders,
but its Git status can omit upstream tracking information.
The preceding `hx .claude/settings.json` process and its `^C` termination were not the blocked process.
Starship started the timed Git request while rendering the next prompt.

The installed versions during diagnosis were:

- Starship 1.26.0,
  release commit `fca92d8dcbd5981b0160af2f7ed7a430b6475a72`;
- cli-git 0.0.1,
  fixed repository revision `268a49aa591331d0776a394912266545b401a4bc`;
- Node v26.7.0;
- Git 2.55.0.

## Root cause

### Starship intentionally resolves `git` from `PATH`

Starship 1.26.0 resolves an external command with `which::which`.
In this repository,
that lookup selected `node_modules/.bin/git` rather than `/usr/bin/git`.
The deciding source is Starship `src/utils/mod.rs:166-180` at tag `v1.26.0`:

```rust
pub fn create_command<T: AsRef<OsStr>>(binary_name: T) -> Result<Command> {
    let binary_name = binary_name.as_ref();
    log::trace!("Creating Command for binary {binary_name:?}");

    let full_path = match which::which(binary_name) {
        Ok(full_path) => {
            log::trace!("Using {full_path:?} as {binary_name:?}");
            full_path
        }
        Err(error) => {
            log::trace!("Unable to find {binary_name:?} in PATH, {error:?}");
            return Err(Error::new(ErrorKind::NotFound, error));
        }
    };
```

This is expected Starship behavior and expected cli-git installation behavior.
The wrapper deliberately shadows Git through `PATH`.

### Upstream tracking asks for external `for-each-ref`

Starship's Git status module uses its embedded Git implementation for status,
but invokes external Git when ahead,
behind,
diverged,
or up-to-date output is enabled.
Starship `src/modules/git_status.rs:445-464` at tag `v1.26.0` contains:

```rust
let has_ahead_behind = !config.ahead.is_empty() || !config.behind.is_empty();
let has_up_to_date_or_diverged =
    !config.up_to_date.is_empty() || !config.diverged.is_empty();
if (has_ahead_behind || has_up_to_date_or_diverged)
    && let Some(branch_name) = gix_repo.head_name().ok().flatten().and_then(|ref_name| {
        Vec::from(gix::bstr::BString::from(ref_name))
            .into_string()
            .ok()
    })
{
    let output = repo.exec_git(
        context,
        ["for-each-ref", "--format", "%(upstream) %(upstream:track)"]
            .into_iter()
            .map(ToOwned::to_owned)
            .chain(Some(branch_name)),
    )?;
```

`Repo::exec_git` adds repository-selection arguments and applies the root command timeout.
Starship `src/context.rs:754-788` at the same tag contains:

```rust
pub fn exec_git<T: AsRef<OsStr> + Debug>(
    &self,
    context: &Context,
    git_args: impl IntoIterator<Item = T>,
) -> Option<CommandOutput> {
    let mut command = create_command("git").ok()?;
    // ...
    command.env("GIT_OPTIONAL_LOCKS", "0").args([
        OsStr::new("-C"),
        context.current_dir.as_os_str(),
        OsStr::new("--git-dir"),
        self.path.as_os_str(),
        OsStr::new("-c"),
        OsStr::new(fsm_config_value),
    ]);
    // ...
    exec_timeout(
        &mut command,
        Duration::from_millis(context.root_config.command_timeout),
    )
}
```

The default timeout is 500 ms in Starship `src/configs/starship_root.rs:155-165`:

```rust
Self {
    // ...
    scan_timeout: 30,
    command_timeout: 500,
    add_newline: true,
    // ...
}
```

### Cli-git inspected the complete native Git binary

Before commit `3a9873d94`,
cli-git checked whether each executable candidate was its own shim by decoding the complete candidate as UTF-8 and
searching it for package markers.
The historical `package/git-policy/cli/src/resolve-git.ts:172-205` at `3a9873d94^` contains:

```ts
async function isShimForSelf(candidatePath: string,): Promise<boolean> {
  // ...
  try {
    /**
     * Raw file bytes decoded as UTF-8; scanned below for self-shim markers.
     */
    const content = await readFile(
      candidatePath,
      'utf8',
    );
    return [...SELF_SHIM_MARKERS,].some(function hasSelfShimMarker(marker,) {
      return content.includes(marker,);
    },);
  }
  catch (error: unknown) {
    // ...
    return false;
  }
}
```

The prioritized candidate was `/usr/bin/git`,
a native executable.
Reading and decoding that entire executable was unnecessary because a native binary cannot be a package-manager command
shim.

The fixed `package/git-policy/cli/src/resolve-git.ts:199-280` reads one native header and scans complete text only for
scripts or unknown formats:

```ts
function isNativeExecutableHeader(header: Uint8Array,): boolean {
  const hex = Buffer.from(header,)
    .toString('hex');
  return [...NATIVE_EXECUTABLE_HEX_PREFIXES,].some(function matchesNativePrefix(prefix,) {
    return hex.startsWith(prefix,);
  },);
}

async function isShimForSelf(candidatePath: string,): Promise<boolean> {
  // ...
  try {
    await using candidate = await open(
      candidatePath,
      'r',
    );
    const header = Buffer.alloc(NATIVE_EXECUTABLE_HEADER_BYTES,);
    const { bytesRead, } = await candidate.read(
      header,
      0,
      header.length,
      0,
    );
    if (isNativeExecutableHeader(
      header.subarray(
        0,
        bytesRead,
      ),
    ))
      return false;
    const content = await candidate.readFile('utf8',);
    return [...SELF_SHIM_MARKERS,].some(function hasSelfShimMarker(marker,) {
      return content.includes(marker,);
    },);
  }
  // ...
}
```

The recognized headers cover ELF,
PE,
Mach-O,
and universal Mach-O.
Windows command shims remain text-scanned,
and the marker set includes backslash spellings for both package and bundled-entry paths.

### Cli-git repeated repository metadata requests

A process trace of the original Starship command showed these real-Git roles before the fix:

```text
recovery membership rev-parse
recovery transaction-path rev-parse
worktree identity rev-parse
worktree-root rev-parse
forwarded for-each-ref
```

Commit `9db2715d8` consolidated identity metadata and retained it for known config-free commands.
The current `package/git-policy/cli/src/bin.ts:181-197` contains:

```ts
const gitPath = await resolveGit();
/**
 * Repository identity reused only when classification proves config cannot mutate process selection.
 */
const configFreeIdentity = (!willShortCircuit)
  && (classifyConfigLoading(rawArgs,) === 'skip-config')
  ? await resolveGitWorktreeIdentity({
      args: rawArgs,
      gitPath,
    },)
  : undefined;
if (!willShortCircuit)
  await recoverCommitTransaction({
    args: rawArgs,
    gitPath,
    ...(configFreeIdentity === undefined ? {} : { identity: configFreeIdentity, }),
  },);
```

The paths this record names under `package/git-policy/cli/src/` were removed on 2026-10-13 in
`642b6d72cb` (refactor(*):
delete the TypeScript Git policy wrapper,
its tasks,
bins and trust
workflow);
no file in the working tree carries them now.

The same identity reached final forwarding at
`package/git-policy/cli/src/bin.ts:338-348` at `268a49aa`,
the revision this record was diagnosed at:

```ts
const transactionCommitted = ((typeof commitTransaction) !== 'symbol')
  && commitTransaction.committed;
if (!transactionCommitted) {
  await runGitWithWorktreeCopy({
    args: processedArgs,
    gitPath,
    ...(configFreeIdentity === undefined ? {} : { identity: configFreeIdentity, }),
  },);
}
```

The regression fixture
`package/git-policy/cli/src/read-only-forwarding.unit.test.ts`
(removed with the TypeScript wrapper on 2026-10-13 in `642b6d72cb`;
no test in `package/git-policy/cli/src/native/` carries Starship's argument vector today)
used Starship's exact argument vector and required only one identity request followed by the forwarded `for-each-ref`.
It also placed an incomplete transaction directory under retained identity and proved recovery blocks before forwarding.
Removing retained-identity recovery made this guard fail before the production branch was restored.

### Starship emitted the warning correctly

Starship's timeout helper terminates an over-budget process and emits the two observed warning lines.
Starship `src/utils/mod.rs:703-756` at tag `v1.26.0` contains:

```rust
match process
    .controlled_with_output()
    .time_limit(time_limit)
    .terminate_for_timeout()
    .wait()
{
    // ...
    Ok(None) => {
        log::warn!("Executing command {:?} timed out.", cmd.get_program());
        log::warn!(
            "You can set command_timeout in your config to a higher value to allow longer-running commands to keep executing."
        );
        None
    }
    // ...
}
```

The warning was therefore a correct Starship diagnostic for cli-git's elapsed startup,
not a Starship defect and not evidence that Helix remained alive.

## Verification

### Reproduce the consumer boundary

Run Starship's Git status module with trace logging from the repository root:

```console
STARSHIP_LOG=trace starship module git_status
```

The trace must show this executable and command shape:

```text
Using "/var/home/user/Monochromatic/node_modules/.bin/git" as "git"
GIT_OPTIONAL_LOCKS="0" "/var/home/user/Monochromatic/node_modules/.bin/git" \
  "-C" "/var/home/user/Monochromatic" \
  "--git-dir" "/var/home/user/Monochromatic/.git" \
  "-c" "core.fsmonitor=" \
  "--work-tree" "/var/home/user/Monochromatic" \
  "for-each-ref" "--format" "%(upstream) %(upstream:track)" "refs/heads/main"
```

### Amplified timeout harness

A 100 ms configuration makes the pre-fix defect deterministic without changing the command path:

```toml
# /tmp/starship-100.toml
command_timeout = 100
```

```console
STARSHIP_CONFIG=/tmp/starship-100.toml STARSHIP_LOG=trace starship module git_status
```

A 1 ms version is the positive control.
It must still fail after the fix,
proving the harness can expose a timeout rather than reporting an unvalidated null result.

### Timing command

The paired benchmark used the same repository,
arguments,
cache state,
and output behavior:

```console
hyperfine --warmup 3 --runs 20 --shell=none \
  '/usr/bin/git -C /var/home/user/Monochromatic --git-dir /var/home/user/Monochromatic/.git -c core.fsmonitor= --work-tree /var/home/user/Monochromatic for-each-ref --format "%(upstream) %(upstream:track)" refs/heads/main' \
  '/var/home/user/Monochromatic/node_modules/.bin/git -C /var/home/user/Monochromatic --git-dir /var/home/user/Monochromatic/.git -c core.fsmonitor= --work-tree /var/home/user/Monochromatic for-each-ref --format "%(upstream) %(upstream:track)" refs/heads/main'
```

### Failing catalog

- Before either cli-git change,
  the wrapper mean was 108.5 ms across 20 runs,
  with a 104.9 to 114.6 ms range.
- At `command_timeout = 100`,
  5 of 5 Starship runs reported failed Git status execution.
- After metadata reuse alone,
  the wrapper mean was 102.2 ms,
  with a 99.0 to 107.2 ms range;
  4 of 5 Starship runs still failed at 100 ms.
- The 1 ms positive control failed before and after the final fix.

### Clean catalog

- After native-header classification,
  the wrapper mean was 65.3 ms across 20 runs,
  with a 61.4 to 68.5 ms range.
- Direct `/usr/bin/git` measured 1.5 ms mean,
  with a 1.3 to 1.9 ms range in the same final run.
- At `command_timeout = 100`,
  5 of 5 Starship runs completed the external Git request.
- At Starship's default 500 ms timeout,
  20 complete prompt renders produced no timeout warning.
- The complete cli-git unit suite passed after rebuilding the shipped artifact.
- The packed-artifact `perf:lifecycle-latency` gate passed every scenario.
  Its read-only scenario added 60.3 ms median and 72.6 ms maximum against a 300 ms budget;
  its no-config scenario added 62.0 ms median and 89.3 ms maximum against a 275 ms budget.
- `mise run //package/git-policy/cli:lint:types` passed.

`mise run //package/git-policy/cli:lint:oxlint` still reports 96 existing
`test-import(require-eventual-artifact)` errors across legacy package tests.
It reports no warning or error in the production files changed for this fix.
This pre-existing package-wide lint debt is separate from the timeout.

## Verified workarounds

### Use the fixed cli-git artifact

Build the package after commits `9db2715d8` and `3a9873d94`:

```console
mise run //package/git-policy/cli:build
STARSHIP_LOG=trace starship module git_status
```

Tradeoff:
unknown native executable formats do not take the header fast path.
They retain complete text inspection,
which preserves self-shim detection but may retain the earlier resolver cost on an unsupported format.
Supported Linux,
Windows,
and macOS native Git formats take the header path.
Parameterized fixtures cover every recognized ELF,
PE,
thin Mach-O,
and universal Mach-O signature in both represented byte orders.

### Disable Starship upstream-status segments

This verified Starship configuration prevents the external `for-each-ref` call:

```toml
[git_status]
ahead = ''
behind = ''
diverged = ''
up_to_date = ''
```

Verification with `STARSHIP_LOG=trace starship module git_status` emitted no `Executing git command` trace.

Tradeoff:
Starship no longer displays ahead,
behind,
diverged,
or up-to-date status.
This avoids the consumer call rather than improving cli-git,
so the repository fix is preferred.

## What does not work

- **Treating `hx` or `^C` as the timed process.**
  The warning names Starship's selected `node_modules/.bin/git`,
  and trace logging reproduces that call after the editor exits.
- **Only consolidating Git metadata.**
  This reduced the measured mean from 108.5 ms to 102.2 ms,
  but 4 of 5 requests still failed under the 100 ms amplified harness.
- **Only increasing `command_timeout`.**
  This can hide the warning,
  but it leaves prompt latency and resolver work unchanged and broadens the timeout for every Starship command.
- **Adding a lightweight second launcher.**
  This would violate cli-git's accepted single-artifact bin and authoring design,
  duplicate dispatch boundaries,
  and add packaging drift.
  It was proposed during diagnosis,
  rejected before implementation,
  and was unnecessary after identifying the complete native-binary scan.
- **Blaming Starship's PATH lookup.**
  Starship intentionally resolves external commands through `PATH`,
  and cli-git intentionally shadows `git` there.
  The avoidable work was inside cli-git.

## Upstream filing artifact

### Upstream filing decision

No `.out-of-scope/` entry matches Starship,
prompt timeouts,
or cli-git.
The Starship tracker was searched across open and closed issues and pull requests for
`command_timeout`,
`git timed out`,
`node_modules`,
and the exact warning.
The nearest result is
[starship/starship#3271](https://github.com/starship/starship/issues/3271),
a closed WSL and Windows-filesystem latency report.
Its complete discussion attributes the delay to filesystem and shell-environment behavior and recommends environment or
timeout configuration.
It does not describe a Node-based Git wrapper scanning a native binary.
There is nothing additive to post there about Starship behavior.

The six filing constraints resolve as follows:

1. **Is it really upstream's fault?**
   No.
   Starship selected `git` through `PATH`,
   invoked a documented Git query,
   enforced its configured timeout,
   and emitted the exact timeout diagnostic.
   Cli-git supplied the avoidable delay.
2. **Can upstream fix it?**
   No upstream correction is needed for this incident.
   Starship could add more command-selection configuration,
   but that would be a separate feature and would not remove cli-git's resolver defect for other consumers.
3. **Are they supporting this use case?**
   Yes.
   Starship's Git status implementation explicitly invokes external `for-each-ref` when upstream-status segments are
   enabled.
4. **Would the repo welcome our contribution?**
   Generally yes for real performance defects.
   Starship `CONTRIBUTING.md:15-19` explicitly says:

   > We aim to make starship as fast,
   > robust and reliable as possible [...].
   > If you spot anywhere that we could trim some time or reduce the prompt's workload,
   > we will gladly accept new issues or PRs!

   Starship `AI_POLICY.md:11-31` requires disclosure,
   human understanding,
   and verified information:

   > Every Pull Request that utilizes AI-assisted tooling [...] must disclose its usage.
   > Contributors must fully understand all submitted contributions.
   > You are not allowed to reply to user issues or discussions with unverified or raw AI-generated information.

   `AI_POLICY.md:38-41` also says:

   > Contributions via OpenClaw,
   > or any other unsupervised autonomous agent operating in an automated loop,
   > are strictly prohibited.

   No contribution is proposed because constraint 1 fails.
5. **Will they likely fix it?**
   Not applicable as an upstream defect.
   Issue #3271 was closed after environment-specific remedies,
and the v1.26.0 source still intentionally uses `PATH` and `command_timeout` for this call.
6. **Have we prototyped a minimal fix compatible with their architecture?**
   No Starship patch was prototyped because constraints 1 and 2 fail.
   The minimal consumer fix was instead implemented and verified in cli-git.
   The auto-prototype requirement is not triggered when the incident is not upstream's fault.

### Filing result

Do not open a Starship issue or pull request for this incident.
Do not comment on issue #3271:
the cli-git implementation details do not advance its WSL-filesystem diagnosis,
and Starship behaved correctly here.
