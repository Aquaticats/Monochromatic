# mise 2026.7.0: any invocation fails with "no data key retrieved from metadata"
inside containers that carry an age-encrypted env file but no age identity

## Symptom

A `podman build` step (or any in-container command) that runs mise in a
copy of this repository fails before doing its job:

```txt
[WARN] migrate: failed to decrypt sops file
mise trusted /baked
[WARN] migrate: failed to decrypt sops file
mise ERROR failed to decrypt sops file
mise ERROR unable to decrypt file metadata
mise ERROR unable to retrieve data key
mise ERROR no data key retrieved from metadata, make sure at least one private integration key is present
mise ERROR Version: 2026.7.0 linux-x64 (2026-07-02)
```

Trigger:
 the build context includes `.env.local.json` (this repo's
sops/age-encrypted secrets file,
 declared as an env source by root
`mise.toml`),
 and the container has no age identity.
 Observed while
building the `cli-mutation-test` runtime image,
 whose Containerfile runs
`mise trust` and `mise install` against the baked repo copy.

When `.env.local.json` is absent from the context,
 the same mise
invocations succeed.

## Root cause

mise loads declared sops env files at config-load time,
 so every
subcommand hits the decryption path.
 With no age key available and
strict mode on,
 the rops decrypt error propagates and mise exits
nonzero.

The skip-versus-fail fork,
 `src/sops.rs:130` (mise commit
`d6ccee0b98933300be1ee79cfdf6d1c41f192e35`):

```rust
if age.is_none() && !Settings::get().sops.strict {
    debug!("age key not found, skipping decryption in non-strict mode");
    return Ok(String::new());
}
```

The failing decrypt whose message appears verbatim in the symptom,
`src/sops.rs:156`:

```rust
.and_then(|file| file.decrypt::<F>().wrap_err("failed to decrypt sops file"))
```

Strict mode is the default.
 `settings.toml` in the same commit:

```toml
[sops.strict]
default = true
description = "If true, fail when sops decryption fails (including when sops is not available, the key is missing, or the key is invalid). If false, skip decryption and continue in these cases."
env = "MISE_SOPS_STRICT"
```

This repo does not override `sops.strict`,
 so containers inherit the
fail-hard default.
 The env directive reaches the decrypt through
`src/config/env_directive/file.rs:63` (`sops::decrypt::<_,
JsonFileFormat>` when the parsed file carries sops metadata).

Earlier wrong hypothesis,
 recorded so it is not re-derived:
 the first
diagnosis blamed the file merely being present in the image.
 Presence
alone is not the trigger;
 presence combined with strict mode and a
missing key is.
 The old dev-script-mutation-test image never hit this
because its hand-assembled build context allowlisted specific files and
never shipped `.env.local.json`.

## Verification

Version under test:
 mise 2026.7.0 linux-x64 (2026-07-02),
 source at
commit `d6ccee0b98933300be1ee79cfdf6d1c41f192e35`;
 reproduced on
2026-07-05 while building `localhost/mutation-test-runtime` images.

Failing harness (repo copy including `.env.local.json`,
 no age key):

```bash
podman build --file package/mutation-test/cli/runtime/Containerfile \
  --tag localhost/mutation-test-runtime:probe /path/to/repo-with-env-local
# fails at the RUN step that invokes mise, with the symptom above
```

Working harness (identical build with the secrets file excluded):

```bash
podman build --file package/mutation-test/cli/runtime/Containerfile \
  --ignorefile package/mutation-test/cli/runtime/containerignore \
  --tag localhost/mutation-test-runtime:probe /path/to/repo
# mise trust + mise install + pnpm install complete
```

Patterns that fail:
 any mise subcommand (`trust`,
 `install`,
 `run`) in
an environment where the declared sops env file exists undecryptable.
Patterns that work:
 same subcommands with the file absent;
 host
invocations where the age identity is available.

## Verified workarounds

- When a GUI fixture needs a disposable `XDG_CONFIG_HOME`,
  do not set it on the parent mise invocation.
  Mise resolves its age identity while loading repository configuration,
  before the task starts.
  A different XDG config root can hide the host identity and produce the same decryption failure.
  Pass `env XDG_CONFIG_HOME=<fixture> ...` as the hosted child command instead,
  so mise keeps its normal identity location and only the app receives disposable configuration.
  This was verified with `//package/nested-wayland-session/cli:run`.
- Exclude `.env.local.json` (and `mise.local.toml`) from the container
  build context:
   `package/mutation-test/cli/runtime/containerignore`,
  passed via `podman build --ignorefile` and hashed into the image tag.
  Verified by the working harness above and by the subsequent green
  mutation runs.
   Tradeoff:
   no secrets exist in-container,
   so anything
  running there must not need them;
   for this repo that is the desired
  posture (secrets must never bake into images,
   encrypted or not).

## Documented alternative, not used here

- `MISE_SOPS_STRICT=false` in the container environment takes the
  `src/sops.rs:130` skip branch (setting definition quoted above).
   Not
  exercised in this repo because it would still ship the encrypted
  secrets file into the image layer;
   exclusion dominates.
   Tradeoff if
  used:
   every sops decryption failure everywhere becomes silent
  (declared secret env vars resolve to nothing),
   not just this one.

## What does not work

- Shipping the file and relying on mise to skip it:
   strict mode is the
  default,
   so mise fails hard;
   there is no per-file optional marker in
  the env directive.
- Building from the repo root without any ignore file:
   besides this
  failure,
   the unfiltered context sweeps `node_modules`,
   `.git`,
   and
  `dist` into the COPY layer (observed 36-minute context upload,
   then a
  layer-commit failure on a near-full disk).

## Upstream filing decision

- Really upstream's fault?
   No. The behavior is intended,
   documented in
  the `sops.strict` setting description,
   and configurable per
  environment.
   The failure was a consumer-side context-assembly bug.
- Can upstream fix it?
   Moot given the above;
   the knob already exists.
- Supporting this use case?
   Yes,
   explicitly:
   the setting description
  enumerates exactly this "key is missing" case for both modes.
- Would the repo welcome the contribution?
   Not evaluated further;
  nothing to contribute.
- Will they likely fix it?
   Nothing to fix.
- Prototyped a minimal fix?
   Not applicable;
   the gate fails at
  constraint 1,
   so no prototype is required.

`.out-of-scope/` was checked (no mise or sops entry).
 No duplicate
search was run because the decision is that there is nothing to file:
no issue draft and no comment draft are kept,
 and this section is the
explicit record of that outcome.
