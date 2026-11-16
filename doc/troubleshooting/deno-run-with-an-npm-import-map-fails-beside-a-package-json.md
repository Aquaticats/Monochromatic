# Deno 2.9.8: `deno run --import-map` with an `npm:` target stops with "Could not find a matching package" when a `package.json` sits beside the script

Unapproved and not final.
An AI session wrote this document on 2026-11-13,
and the owner has not approved it.
Done and checked by that session:
the invocations and the source reading reported here.
Open:
a model from another vendor reviewed it as of commit `426d33d638`,
and the corrections made after that as of commit `2d06b4ae60`,
where it found nothing to change in this document.
The paragraph on Deno's maintenance state under "Upstream filing decision" was added after that and has had no review.

Found on 2026-11-13 while checking that the scripts of a reproduction repository run on Deno.
The repository holds a `package.json` so that Node treats its scripts as ES modules,
and an import map so that Deno can run them with nothing installed.
The two do not go together without a flag.

## Symptom

In a folder that holds a `package.json` and no `node_modules`:

```sh
deno run --import-map maps/npm-1.3.6.json commit-paths.ts
```

```text
error: Could not find a matching package for 'npm:@optique/core@1.3.6' in the node_modules directory. Ensure you have all your JSR and npm dependencies listed in your deno.json or package.json, then run `deno install`. Alternatively, turn on auto-install by specifying `"nodeModulesDir": "auto"` in your deno.json file.
    at file:///repo/commit-paths.ts:16:24
```

The exit status is 1.

The import map is one line of mapping:

```json
{
  "imports": {
    "@optique/core/": "npm:/@optique/core@1.3.6/"
  }
}
```

The same command in a folder without a `package.json` runs.
The `package.json` here is `{"private": true, "type": "module"}`;
it lists no dependency.

## Root cause

### Step 1: a discovered `package.json` sets the `node_modules` mode to manual

With no flag and no `nodeModulesDir` in a `deno.json`,
Deno decides the mode from what it discovered,
[`libs/resolver/factory.rs` lines 363 to 385][mode]:

```rust
// libs/resolver/factory.rs at tag v2.9.8
    let workspace = &self.workspace_directory()?.workspace;

    if let Some(pkg_json) = workspace.root_pkg_json() {
      if let Ok(deno_dir) = self.deno_dir() {
        let deno_dir = &deno_dir.root;
        // `deno_dir` can be symlink in macOS or on the CI
        if let Ok(deno_dir) =
          canonicalize_path_maybe_not_exists(&self.sys, deno_dir)
          && pkg_json.path.starts_with(deno_dir)
        {
          // if the package.json is in deno_dir, then do not use node_modules
          // next to it as local node_modules dir
          return Ok(NodeModulesDirMode::None);
        }
      }

      Ok(NodeModulesDirMode::Manual)
    } else if workspace.vendor_dir_path().is_some() {
      Ok(NodeModulesDirMode::Auto)
    } else {
      // use the global cache
      Ok(NodeModulesDirMode::None)
    }
```

A `package.json` anywhere it discovers,
whatever it contains,
gives `Manual`.

### Step 2: manual mode means the caller's own `node_modules` is the only source

[`libs/resolver/factory.rs` lines 1340 to 1345][byonm]:

```rust
// libs/resolver/factory.rs at tag v2.9.8
  pub fn use_byonm(&self) -> Result<bool, anyhow::Error> {
    Ok(
      self.workspace_factory.node_modules_dir_mode()?
        == NodeModulesDirMode::Manual,
    )
  }
```

In that mode an `npm:` specifier is looked up in `node_modules` and never fetched.
The import map still applies:
it turns `@optique/core/` into `npm:/@optique/core@1.3.6/`,
and that is the specifier the lookup then fails on,
[`libs/resolver/npm/byonm.rs` lines 43 to 45][error]:

```rust
// libs/resolver/npm/byonm.rs at tag v2.9.8, the message shortened here
  #[class(generic)]
  #[error("Could not find a matching package for 'npm:{}' in the node_modules directory. …")]
  UnmatchedReq(PackageReq),
```

### Step 3: two switches turn the discovery off

`--no-config` disables discovery altogether,
[`cli/factory.rs` line 1594][no config]:

```rust
// cli/factory.rs at tag v2.9.8
      ConfigFlag::Disabled => ConfigDiscoveryOption::Disabled,
```

and the environment variable `DENO_NO_PACKAGE_JSON` disables it for `package.json` alone,
[`libs/resolver/factory.rs` lines 616 to 620][no package json]:

```rust
// libs/resolver/factory.rs at tag v2.9.8
        let discover_pkg_json = !self.options.no_npm
          && !self.has_flag_env_var("DENO_NO_PACKAGE_JSON");
        if !discover_pkg_json {
          log::debug!("package.json auto-discovery is disabled");
        }
```

This is how Deno is meant to behave.
Its own tests expect the message:
`tests/integration/npm_tests.rs` line 1414 asserts it for `npm:chalk@5`.

## Verification

### Version under test

Deno 2.9.8,
the package of `docker.io/cachyos/cachyos-v4:latest` on 2026-11-13,
in a container with `--cpus 2 --memory 4g --pids-limit 512 --blkio-weight 10`.
Source read at tag `v2.9.8`,
commit `fe7ed012d92c642fd8e5607ac5a45d517538b5c5`.

### Harness

The folder is the reproduction repository `Aquaticat/optique-parse-cost-reproduction` at commit `76b15277`:
`package.json`,
`commit-paths.ts`,
`maps/npm-1.3.6.json`.
`commit-paths.ts` imports `@optique/core/constructs` and five more subpaths and prints one JSON line.

### Invocations that fail

```sh
deno run --import-map maps/npm-1.3.6.json commit-paths.ts --paths 2 --runs 1 --warmup 0
deno cache --import-map maps/npm-1.3.6.json mismatch-count.ts commit-paths.ts token-scaling.ts
```

Both stopped with the message under "Symptom",
the second naming `mismatch-count.ts:22:24`.
The first was run with no network,
with the package already in Deno's cache,
so the failure is not a failed download.

### Invocations that work

Each printed the script's JSON line,
and the line names where the library was loaded from:

- The failing command itself,
   in a copy of the folder that leaves the `package.json` out:
   from Deno's cache.
- `deno run --no-config --import-map maps/npm-1.3.6.json commit-paths.ts`:
   from Deno's cache,
   `/deno-cache/npm/registry.npmjs.org/@optique/core/1.3.6/`.
- `DENO_NO_PACKAGE_JSON=1 deno run --import-map maps/npm-1.3.6.json commit-paths.ts`:
   from Deno's cache.
- `deno run --node-modules-dir=auto --import-map maps/npm-1.3.6.json commit-paths.ts`:
   from `/repo/node_modules/.deno/@optique+core@1.3.6/`,
   which Deno created.
- After `npm install` of the package,
   with no import map,
   `deno run commit-paths.ts`:
   from the folder's own `node_modules`.

The records are `cross-runtime/deno-variants.log` and `close-review-2-gaps.jsonl`,
under `measurements-2026-11-13`
in `doc/research/optique-typo-suggestion-overhead-upstream-record.evidence.zip`.
`optique-typo-suggestion-overhead-upstream-record.evidence.zip` left the tree on 2026-11-13.
Its cited records are in `doc/research/optique-typo-suggestion-overhead-upstream-record.evidence/`.
`doc/audit/removed-archives-and-where-history-holds-each.md` holds the commit and the command that prints it.

## Verified workarounds

### Pass `--no-config`

What the reproduction repository's README now tells its reader.

Tradeoff:
Deno also ignores any `deno.json` there,
so tasks,
compiler options and a lockfile from it do not apply.

### Set `DENO_NO_PACKAGE_JSON=1`

Tradeoff:
it is an environment variable,
easy to lose in a copied command,
and Deno then reads nothing from the `package.json`.

### Pass `--node-modules-dir=auto`

Tradeoff:
Deno writes a `node_modules` folder into the working folder.
What Node and Bun then resolve from it was not tried.

### Install the package and drop the import map

Tradeoff:
the package has to be installed,
which the import map existed to avoid,
and the version is whatever was installed.

## What does not work

- The import map alone.
   It is applied,
   and the `npm:` specifier it produces is then resolved in manual mode.
- Removing the `package.json`.
   Deno then runs the command,
   as the first entry under "Invocations that work" records,
   but the file is there so that Node does not have to detect the module type of each `.ts` script.

## Upstream filing artifact

### Upstream filing decision

Decision:
nothing to file.

`.out-of-scope/`,
read 2026-11-13:
no entry mentions Deno.

Duplicate search,
2026-11-13:
`gh issue list --repo denoland/deno --state all --search '"Could not find a matching package for" "import map" package.json in:title,body'`
returns one closed issue,
23459,
about mapping the dependencies of npm packages,
which is another matter.
A control query for `byonm` returned results,
so the search was working.

The six constraints:

1.  Upstream's fault:
    no.
    The branch in step 1 is explicit and a test asserts the message.
2.  Upstream can fix it:
    not applicable.
3.  Supported use case:
    a `package.json` beside an import map with `npm:` targets is supported in the sense that four invocations run it.
4.  Contribution welcome:
    not checked,
    since nothing is to be sent.
5.  Likely to fix:
    not applicable.
6.  Minimal fix prototyped:
    not applicable.

The message itself could say that a discovered `package.json` is why `node_modules` is being required;
it names `deno.json` and `nodeModulesDir` and not `--no-config`.
That is a wording preference and no report was drafted for it.

Deno's own state bears on how much this matters.
Its maintainers wrote on 2026-10-15 that they [will maintain Deno until October 2027][deno state],
with monthly maintenance releases and no new feature work.
So the scripts' Node and Bun commands are the ones to rely on,
and the Deno route is kept only because a source checkout runs on it without a build.

[mode]: https://github.com/denoland/deno/blob/fe7ed012d92c642fd8e5607ac5a45d517538b5c5/libs/resolver/factory.rs#L363-L385
[byonm]: https://github.com/denoland/deno/blob/fe7ed012d92c642fd8e5607ac5a45d517538b5c5/libs/resolver/factory.rs#L1340-L1345
[error]: https://github.com/denoland/deno/blob/fe7ed012d92c642fd8e5607ac5a45d517538b5c5/libs/resolver/npm/byonm.rs#L43-L45
[no config]: https://github.com/denoland/deno/blob/fe7ed012d92c642fd8e5607ac5a45d517538b5c5/cli/factory.rs#L1594
[no package json]: https://github.com/denoland/deno/blob/fe7ed012d92c642fd8e5607ac5a45d517538b5c5/libs/resolver/factory.rs#L616-L620
[deno state]: https://github.com/denoland/deno/issues/37030#:~:text=will%20continue%20maintain%20Deno%20until%20October%202027
