# Flatpak 1.18.4 silently replaces caller-set variables with app metadata Environment

## Symptom

Passing an environment variable to `flatpak run` has no effect when the
 app's metadata sets the same name.
 Verbatim:

```bash
$ env WINEPREFIX=/var/home/user/.local/share/wineprefixes/pvzfusion \
    flatpak run --command=sh org.winehq.Wine -c 'echo $WINEPREFIX'
/var/data/wine
```

The caller's value is discarded without a warning,
 a debug line,
 or a non-zero exit.

In the incident this silently routed four consecutive Wine launches
 into the wrong prefix
 (`/var/data/wine`,
 the Flathub Wine app's default),
 including a registry write intended for another prefix.
 The wrong prefix looks healthy
 (a fresh Wine prefix gets created there),
 so the misdirection is invisible until a missing file or setting is
 traced back.

Surface trigger:
 any `flatpak run` invocation that configures an app per run via the
 caller environment
 for a variable the app's metadata `[Environment]` group also defines
 (`WINEPREFIX` for `org.winehq.Wine`,
 display or GPU hints for other apps).
 Affected versions:
 Flatpak 1.18.4 on Bazzite 44.

## Root cause

The sandbox environment is assembled in layers that each overwrite the
 previous one,
 and the caller's environment is only the first layer.
 Source trace against flatpak tag `1.18.4`
 (commit `a02d0ba48abe9aacc377de15699e5d8c024b5669`):

1.  The caller environment is copied first
    (`common/flatpak-run.c:3319` calls `flatpak_bwrap_new`,
    then `common/flatpak-bwrap.c:58`):

```c
  if (env)
    bwrap->envp = g_strdupv (env);
  else
    bwrap->envp = g_get_environ ();
```

2.  The app metadata's `[Environment]` group is parsed into the app
    context
    (`common/flatpak-context.c:2908`):

```c
  if (g_key_file_has_group (metakey, FLATPAK_METADATA_GROUP_ENVIRONMENT))
    {
      ...
          g_autofree char *value = g_key_file_get_string (metakey, FLATPAK_METADATA_GROUP_ENVIRONMENT, key, NULL);
          flatpak_context_set_env_var (context, key, value);
```

    For `org.winehq.Wine` the deployed metadata file
    (`/var/lib/flatpak/app/org.winehq.Wine/x86_64/stable/active/metadata`)
    contains `[Environment] WINEPREFIX=/var/data/wine`.
3.  Override layers merge after metadata
    (`common/flatpak-run.c:3411`):

```c
  if (app_deploy != NULL)
    {
      overrides = flatpak_deploy_get_overrides (app_deploy);
      flatpak_context_merge (app_context, overrides);
    }
```

    with layer order system global,
    system per-app,
    user global,
    user per-app
    (`common/flatpak-dir.c:1913`),
    later layers winning
    (`common/flatpak-context.c:1976`,
    `g_hash_table_insert` on `env_vars`).
4.  `flatpak run --env` merges last,
    labelled `"Command-line overrides"` in its own debug dump
    (`common/flatpak-run.c:3423`):

```c
  if (extra_context)
    {
      flatpak_context_dump (extra_context, "Command-line overrides");
      flatpak_context_merge (app_context, extra_context);
    }
```

5.  The merged context is written over the caller copy with overwrite
    enabled
    (`common/flatpak-run.c:871` calls
    `flatpak_bwrap_set_env (bwrap, var, val, TRUE)`,
    landing in `common/flatpak-bwrap.c:92`,
    `g_environ_setenv (..., overwrite)`).

Resulting precedence,
 lowest to highest:
 caller environment,
 runtime metadata `[Environment]`,
 app metadata `[Environment]`,
 overrides
 (system global,
 system per-app,
 user global,
 user per-app),
 `flatpak run --env`.
 Flatpak's forced variables
 (`PATH`,
 `LD_*`,
 `XDG_*_HOME`,
 `VK_ICD_FILENAMES`,
 and similar,
 applied by `flatpak_run_apply_env_default`)
 take precedence over even `run --env` for those names.

Documentation state:
 `flatpak-run(1)` states the intent
 ("The application metadata can override environment variables, as well
 as the `--env` option",
 `doc/flatpak-run.xml:81`),
 but `flatpak-metadata(5)`'s `[Environment]` section
 (`doc/flatpak-metadata.xml:981`) carries no precedence statement,
 and no document warns that a caller-set variable is replaced silently.

One misreading worth recording:
 `flatpak info --show-permissions` prints the merged context
 (metadata plus every override layer),
 not the app's shipped metadata
 (`app/flatpak-builtins-info.c:408` loads the merged context via
 `flatpak_context_load_for_deploy`).
 For the shipped value,
 read the deployed `metadata` file or `flatpak info --show-metadata`.

## Verification

Versions under test:
 Flatpak 1.18.4,
 `org.winehq.Wine` 11.0 from Flathub
 (build commit `2b7feee9cd37329c1fecd0c45960b0634cd0e684a3f1fd895df8771612ed2865`).

Clean-room harness
 (fresh `XDG_DATA_HOME` so no override layers interfere):

```bash
env XDG_DATA_HOME=~/temp/agent/xdgdata WINEPREFIX=/some/other/path \
  flatpak run --command=sh org.winehq.Wine -c 'echo caller-env-test:$WINEPREFIX'
env XDG_DATA_HOME=~/temp/agent/xdgdata WINEPREFIX=/some/other/path \
  flatpak run --command=sh --env=WINEPREFIX=/run/env/path org.winehq.Wine -c 'echo run-env-test:$WINEPREFIX'
```

### Catalog that works cleanly

- `flatpak run --env=WINEPREFIX=/run/env/path ...`:
  prints `/run/env/path`
  (beats both metadata and a user override).
- `flatpak override --user --env=WINEPREFIX=<path> org.winehq.Wine`
  followed by a bare `flatpak run`:
  prints `<path>`
  (verified live while pinning the game's prefix).

### Catalog that fails

- `env WINEPREFIX=/some/other/path flatpak run ...`:
  prints `/var/data/wine`
  (metadata beats the caller).
- Same failure with any variable the app metadata defines,
  with no diagnostic output of any kind.

## Verified workarounds

### Persist the value as a user override

```bash
flatpak override --user --env=WINEPREFIX=/var/home/user/.local/share/wineprefixes/pvzfusion org.winehq.Wine
```

Verified live.
 Tradeoffs:
 persistent and per app,
 so every future run of that app sees the value,
 including desktop entries;
 per-run variation needs `run --env` anyway.

### Pass the value per run

```bash
flatpak run --env=WINEPREFIX=/path/to/prefix org.winehq.Wine ...
```

Verified in the clean-room harness
 (beats metadata and beats the user override).
 Tradeoffs:
 must be repeated in every launcher,
 including `Exec=` lines of desktop entries.

## What does not work

- Setting the variable in the caller environment.
  This is the core surprise,
  repeated here because every instinct says it should work.
- Reading `flatpak info --show-permissions` to learn an app's own
  metadata values:
  it shows the merged context,
  which silently includes override layers
  (a misreading in this incident).
- Looking for the precedence in `flatpak-metadata(5)`:
  the `[Environment]` section documents syntax only.

## Upstream filing artifact

### Duplicate search

Searched `flatpak/flatpak` issues and PRs on 2026-10-26
 (open and closed,
 about fifteen lexical queries plus one semantic,
 also `flatpak/flatpak-docs`):
 `metadata environment override`,
 `environment variables command line ignored`,
 `WINEPREFIX`,
 `environment variable precedence`,
 `override environment variable`,
 `Environment section metadata`,
 `take precedence environment`,
 `docs document environment variables precedence`,
 and related phrasings.

No issue or PR reports app metadata `[Environment]` overriding a
 caller-set variable,
 and none requests documenting this precedence.
 Nearest neighbors:
 issue 4280
 (whether passing all caller environment variables is a good idea),
 issue 4529
 (forced `XDG_*_HOME` beating `--env`,
 closed as intended),
 issue 5543
 (caller environment leaking in,
 the inverse complaint),
 and PR 6867
 (preventing host environment inheritance with `--clear-env`).

### Upstream filing decision

1.  Is it really upstream's fault?
    No.
    The behavior matches `flatpak-run(1)`
    ("The application metadata can override environment variables"),
    so this is intended behavior,
    not a bug.
    The real gap is documentation clarity
    (no single precedence statement,
    no warning on silent replacement).
2.  Can upstream fix the docs gap?
    Yes;
    a paragraph in `doc/flatpak-run.xml` and
    `doc/flatpak-metadata.xml` would do it.
3.  Are they supporting this use case?
    Environment handling is core to `flatpak run`,
    and `--env` documentation is actively maintained
    (PR 6066).
4.  Would the repo welcome our contribution?
    Yes;
    `README.md` says "Flatpak welcomes contributions from anyone" and
    asks for detailed reports,
    and `CONTRIBUTING.md`,
    `SECURITY.md`,
    `CODE_OF_CONDUCT.md`,
    and the issue templates contain no AI-assisted-filing restriction
    (checked 2026-10-26).
5.  Will they likely fix it?
    Docs-only changes merge routinely;
    no signal against.
6.  Have we prototyped a minimal fix?
    Not applicable to a docs clarification,
    and the Verified workarounds already solve the
    user-facing problem.

Outcome:
 do not file as-is.
 A bug report would fail constraint 1.
 The only upstream-shaped artifact is a docs clarification,
 and since the duplicate search shows none exists,
 it may be worth filing someday as a documentation issue;
 the draft is kept at the end of this file,
 marked do-not-file-as-is,
 so a future session can re-evaluate.
 No `.out-of-scope/` exemption applies
 (checked 2026-10-26:
 no flatpak entry).

~~~md
Docs clarification draft for flatpak/flatpak (do not file as-is)

Title: doc: state the environment precedence chain for flatpak run

Body:

flatpak-run(1) says "The application metadata can override environment
variables, as well as the --env option", but no manual page states the
full chain in one place, and flatpak-metadata(5)'s [Environment] section
documents syntax only. Please consider documenting:

  caller environment < runtime [Environment] < app metadata [Environment]
  < overrides (system global, system per-app, user global, user per-app)
  < flatpak run --env

together with a note that a variable set by the caller is silently
replaced when any later layer defines the same name, and that Flatpak's
forced variables (PATH, LD_*, XDG_*_HOME, VK_ICD_FILENAMES, ...)
override everything for those names.

Observed on Flatpak 1.18.4: `env WINEPREFIX=/other flatpak run
org.winehq.Wine ...` runs with the metadata value /var/data/wine and no
diagnostic. Source: the caller env is seeded at
common/flatpak-run.c:3319 (flatpak-bwrap.c:58) and the merged context is
written with overwrite=TRUE at common/flatpak-run.c:871
(flatpak-bwrap.c:92).
~~~
