# module-fs-path decisions

## Platform split behind `imports` conditions, with a native OPFS backend (2026-09-07)

Decided by the owner during the first-release grilling recorded in `doc/planning/module-fs-path-release.md`.

Before this change the package had one entry whose barrel re-exported `ensure.ts`,
 `empty.ts`,
 and `find-package-root.ts`,
 each with static `node:fs/promises` and `node:path` imports,
 while root discovery and the path operations probed the runtime and used dynamic imports
 (`import('node:fs/promises')`,
 `import('node:path')`,
 `import('happy-opfs')`) to stay cross-runtime.
Both built artifacts therefore imported Node builtins statically and carried every dynamic import,
 so the `default` export condition handed a browser bundler `node:fs/promises`,
 and importing anything from the entry dragged Node into a client bundle
 (the done-postcss incident in `doc/troubleshooting/css-tooling.md`).

The shape now follows `package/logger/core/DECISIONS.md`
 ("Platform-specific sinks live behind `./node` and `./browser`"):

- The root entry `.` is platform-neutral:
   `dirname`,
   `join`,
   `resolve`,
   `normalize`,
   `isAbsolute`,
   `sep`,
   the two trims,
   the three root finders with their cached variants,
   and `GitRepositoryRootNotFoundError`.
- `./node` ships `ensureDir`,
   `ensureFile`,
   `ensurePath`,
   `emptyDir`,
   `emptyFile`,
   `emptyPath`,
   `removeEmptyFilesInDir`,
   `findPackageRoot`,
   and `findPackageRootCached`,
   with static `node:` imports,
   built only by the node config.
- `package.json` `imports` selects the backends at resolution time:
   `#posix-path` (`node`:
   `node:path/posix`;
   `default`:
   the pure-JS implementation)
   and `#root-filesystem` (`node`:
   `node:fs/promises`;
   `default`:
   OPFS or an empty backend).
   No runtime probe,
   no dynamic import in either artifact.
- The OPFS backend talks to `navigator.storage.getDirectory()` directly,
   walking directory and file handles one path segment at a time,
   as the logger's OPFS sink already does.
   `happy-opfs` left the manifest,
   the pnpm catalog,
   and the lockfile:
   929 kB unpacked plus five transitive packages that every Node installer
   would have downloaded for a path no consumer runs.
- `normalize` is public and delegates under `node` like its siblings.
   `dirnameFallback`,
   `joinFallback`,
   and `resolveFallback` left the public entry:
   in the neutral artifact they are the implementation,
   so tests import `dist/final/neutral/index.mjs` directly.
   The top-level `await import` in `index.ts` went with them.

Rejected shapes:

- Node-only 0.1.0 (the root entry pointing at the node build,
   the browser build dropped):
   discards a documented design property the package was extracted with
   (commit `894c84a5d`,
   the `dom` tsconfig,
   the OPFS backend).
- One entry keeping dynamic imports for the Node-only helpers:
   the logger decision measured that every `import()` in a library leaks into every downstream bundle,
   and `ensureDir` would fail at call time in a browser instead of at resolution.

Measured on 2026-09-07 on the built artifacts:

- Node build:
   `index.mjs` 7.15 kB,
   `node.mjs` 3.43 kB,
   one shared logger chunk 20.42 kB;
   neutral build:
   `index.mjs` 32.27 kB.
   Root `index.d.mts` byte-identical across builds.
- Zero `import(` across every chunk of both builds;
   the neutral build names neither `node:fs` nor `node:path`;
   the node build touches neither `navigator.storage` nor a directory handle.
   A seven-case unit test (`artifact-platform-split.unit.test.ts`) reads every chunk with positive controls;
   with the split deliberately broken it failed two cases,
   restored it passed.
- Chromium and Firefox found mise,
   git,
   and pnpm markers written into OPFS and rejected the two negatives with the expected error classes;
   headless WebKit refused OPFS writes and reported skipped.
   With the backend deliberately reporting every path absent,
   Chromium failed the finder case;
   restored it passed.
- The pure-JS path operations match `node:path/posix` over a corpus of edge cases,
   which surfaced one difference,
   fixed:
   `dirname` skipped one trailing slash where node skips the whole run.

Accepted consequences:

- A Node consumer whose bundler resolves the `default` condition
   gets no filesystem for root discovery and no message beyond the rejection,
   the same as every package in the logger's prior-art sample.
- The inlined logger stays inlined (standing `doc/decision/npm-publishing.md` direction):
   a consumer that also installs `@monochromatic-dev/logger-core`
   runs two logger instances with separate sink discovery.
- No browser CI job exists;
   the Playwright gate runs locally in podman,
   as for the logger.

## Permission repair grants the owner bits, never the raw access constants (2026-09-07)

`ensureDir` and `ensureFile` repaired an inaccessible existing path with `chmod(path, R_OK | W_OK)`.
`R_OK | W_OK` is 6,
 so the resulting mode was `0o006`:
 world read and write,
 owner nothing,
 and the caller stayed locked out.
Surfaced by the first tests written for the family.
The repair now keeps the existing permission bits and adds owner read and write,
 plus owner traverse for directories.

## The walker and its filesystem seam are the root-discovery interface (2026-09-08)

Decided by the owner during the architecture grilling recorded in `doc/planning/module-fs-path-deepening.md`.

Before this change the package exposed four root finders
 (`findMiseMonorepoRoot`,
 `findGitRepoRoot`,
 `findPnpmWorkspaceRoot`,
 `findPackageRoot`),
 each with a cached twin,
 its own error text,
 its own memo,
 and its own log tag,
 while the upward walk they shared sat behind them where no caller or test could reach it.
`findPackageRoot` kept its own recursion and read `node:fs/promises` directly,
 so it shipped from `./node`.
The `RootFilesystem` seam had two real adapters and no test adapter,
 so every marker test built a directory tree under `mkdtemp`.
Error identity was a string compare on the message,
 rewrapped into `GitRepositoryRootNotFoundError` for one finder only.

The shape now:

- `findRoot({ marker, cwd?, fs? })` and `findRootCached({ marker, cwd? })` are the interface.
   A `RootMarker` is `{ name, matches({ dir, fs }) }`;
   the package ships `MISE_MONOREPO`,
   `GIT_REPOSITORY`,
   `PNPM_WORKSPACE`,
   `packageNamed(name)`,
   `fileNamed(name)`,
   and `directoryNamed(name)`.
   A new kind of root is one value.
   Deletion test:
   removing the named finders loses nothing a marker value does not express;
   removing the walker would reappear in every caller.
- The filesystem contract keeps five probes
   (`readTextFile`,
   `readSymbolicLink`,
   `exists`,
   `isDirectory`,
   `isFile`)
   and drops `resolvePath`:
   both adapters implemented it with the same POSIX arithmetic,
   so the Git marker resolves gitfile and `commondir` targets through `#posix-path`.
- `createMemoryRootFilesystem({ files, directories, links })` is the third adapter,
   exported so tests and callers without a disk walk a declared tree through the same seam.
   It follows the `node:fs/promises` rules:
   `exists` and `readSymbolicLink` look at the entry itself,
   the other probes look through links along the whole path.
- One `RootNotFoundError { marker, startDir }` for every marker.
- `findRootCached` keys its memo on the marker name and the start directory resolved at call time,
   and keeps rejections per key.
   A process-pinned key was proposed and withdrawn:
   commit `701dfc88e` chose lock-first as a lazy wrapper,
   not for a reason,
   and nothing in the workspace calls a finder after `process.chdir`.
   Consequence:
   two modules of one package with different `import.meta.dirname` run two walks;
   accepted.
- `packageNamed` moves package-root discovery to the root entry;
   `./node` keeps only the ensure and empty family.
- The `[monorepo]` matcher splits on `\n`,
   trims each line,
   and accepts the header alone or followed by a comment,
   so CRLF files and trailing comments match and a header quoted inside a value does not.
- `fileNamed` probes `isFile` and `directoryNamed` probes `isDirectory`,
   so `PNPM_WORKSPACE` no longer accepts a directory named `pnpm-workspace.yaml`.

Rejected shapes:

- Keeping the named finders as thin wrappers over `findRoot`:
   they would pass the deletion test as pass-throughs,
   and the package has one consumer,
   this repository,
   whose 25 files moved in the same change.
- A marker registry keyed by string:
   nothing varies across it that a value does not already express.
- Deprecating instead of removing:
   an unpublished-to-anyone-else 0.1.0 has no external consumer to protect.

Measured on 2026-09-08:

- Marker behaviour runs as data tables over the memory adapter:
   18 mise header shapes,
   36 Git marker shapes (every branch of `git-marker.ts`),
   4 pnpm,
   3 `fileNamed`,
   3 `directoryNamed`,
   6 `packageNamed`;
   with the matcher reverted to the old `\n[monorepo]\n` framing,
   the CRLF,
   comment,
   trailing-space,
   and indented cases failed (5 red),
   restored they passed.
- Seven real-filesystem cases remain,
   proving the Node adapter reads CRLF files,
   gitfiles,
   and symbolic-link HEADs,
   and that the three presets converge on this checkout.
- Node build `index.mjs` 9.16 kB,
   `node.mjs` 2.91 kB;
   the leak guard still finds no `import(`,
   no `node:` in the neutral build,
   and no `navigator.storage` in the node build.

## ensureFile's create is exclusive, and the parent is created only when a create needs it (2026-11-10)

`ensureFile` decided to create the file from a `stat` result and then opened
 it with flag `w`,
 which truncates.
The `stat` proved absence only at that moment:
a second writer that created and wrote the file in between had its contents
 destroyed,
 measured as 40 truncations in 40 interleaved rounds against the previous code.

The shape now:

- `ensureFile` creates with flag `wx`,
   so an entry that appears in the window is reported as `EEXIST` instead of
   being opened for writing.
   `EEXIST` falls back to the existing-entry path:
   re-stat,
   refuse a non-file,
   and repair the owner bits,
   so the second writer's contents survive.
- The parent directory is created when the exclusive create reports `ENOENT`
   or `EACCES`,
   not before it,
   so a call that finds the file already there touches no parent.
   A bare name has no parent segment and skips the step entirely,
   which is the `mkdir('')` failure of issue 1410.
- A path that `EEXIST` reported while `stat` reports no entry is a symbolic
   link whose target is absent,
   since `stat` follows links.
   Its target is created with flag `a`:
   an append creates the file when it is absent and writes nothing to a file
   that exists,
   so no path through `ensureFile` opens a file for truncation.
   The previous code created the target through the link,
   and that behaviour is kept.

Rejected shapes:

- Keeping the `stat` before the create and opening with `wx` after it:
   the window stays,
   the fallback becomes unreachable in practice,
   and no test can drive it without a seam.
- Fixing only the flag and throwing on `EEXIST`:
   `ensureFile` on an ordinary existing file would then report a failure.
- Creating the parent directory unconditionally, as before:
   it repairs and stats a directory the call does not need,
   and a bare name has no parent string to pass.

## One `?query` suffix policy for the ensure and empty families (2026-11-10)

`ensurePath` parsed the path as given,
 so `dist/x.js?raw` counted `.js?raw` as its extension and `ensureFile` created
 a file literally named `x.js?raw`,
 while `emptyPath` stripped the suffix and truncated the unrelated `dist/x.js`.
A caller that ensured a bundler-style spelling and then emptied it lost the
 contents of a file it never named.

The shape now:

- `query-suffix.ts` owns the spelling:
   `stripQuerySuffix` cuts everything from the first `?`,
   and every function in both families calls it before parsing a path or
    touching the disk.
   `ensurePath` decides file or directory from the stripped path,
    so `.js?raw` is a file and `logs?raw` a directory,
    the same decision `emptyPath` makes.
- Each function returns the spelling it was given,
   suffix included.
   `emptyPath` returned the stripped path down the directory branch and the
    given spelling down the file branch,
    which the function's own documentation contradicts;
    both branches now return the given spelling.

Rejected shapes:

- Refusing a query suffix in the ensure family:
   the empty family already accepted these spellings,
    and a bundler is where they come from,
    so the caller would have to strip on one side and not the other.
- Keeping the strip inside `emptyPath` and `emptyFile` only:
   the two families would still hold two spellings of one path,
    which is the defect.
