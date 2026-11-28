# An mvm build removes the file the `mvm` shim runs, before writing it again

Unapproved and non-final.
Written 2026-11-28 by an agent round working on
[issue 6668](https://github.com/Aquaticats/Monochromatic/issues/6668);
the owner declares completion.
What was done and checked is under "Fix" and "Verification";
what is open is under "What this does not change".

## Symptom

Twice on 2026-11-28,
a command that started while a build of `package/mvm/cli` was running in a checkout ended with:

```text
Error: Cannot find module '<checkout>/package/mvm/cli/dist/final/node/cli.mjs'
```

raised from `node:internal/modules/cjs/loader`.
Both times the `mvm` was that checkout's `node_modules/.bin/mvm`,
written by an install older than the manifest's `bin` field,
so the shim named the built file rather than the source:

```text
# cmd-shim-target=../../package/mvm/cli/dist/final/node/cli.mjs
```

A shim written by an install of this manifest names `src/cli.ts` instead,
because `package/mvm/cli/package.json` declares `"mvm": "src/cli.ts"`.

`mise run //package/mvm/cli:test:unit` opens the same window,
because that task depends on the package's build.

## Root cause

`package/config/rolldown/src/index.node.ts` gave every Node flavor build
`output.cleanDir: true` (line 221 at commit `65a1d6915b`).
Rolldown reads that as "delete every file of the output directory when writing starts",
and it writes the bundle's files only after the render phase.
The deletion is per directory,
not per file,
so every path under `dist/final/node` is absent from the start of the render until the files
are written again,
which is most of the build.

`package/config/rolldown/src/index.client.ts` and `src/index.ts` set the same option for the
client and neutral flavors.
A build that chooses its output directory through the shared configuration goes through
`nodeConfig`,
and that is where the fix lives.

## The window, measured

Two builds of `package/mvm/cli` on 2026-11-28,
each under
`strace --follow-forks --trace=unlink,unlinkat,rename,renameat,renameat2,openat,write,close,chmod,mkdir`,
taken in the round's worktree and filtered to `package/mvm/cli/dist/final/node`.

Before the fix,
one build deletes all thirteen files of that directory and writes them again:

```text
789000 08:59:44.216139 unlink(".../package/mvm/cli/dist/final/node/cli.mjs") = 0
788998 08:59:44.310532 openat(AT_FDCWD, ".../package/mvm/cli/dist/final/node/cli.mjs", O_WRONLY|O_CREAT|O_TRUNC|O_CLOEXEC, 0666) = 21
788998 08:59:44.310590 write(21, "#!/usr/bin/env node\n// This file"..., 384) = 384
```

The path is absent from the unlink to the create:
94.393 ms,
and empty for the 58 us between the create and the write.
An earlier build of the same shape measured 112.830 ms and 174.408 ms in the same place,
so the figure moves with the machine's load,
and the mechanism does not.

A probe plugin appended to the same config,
recording what the path held at each hook of rolldown's lifecycle,
puts the absence inside the build's own phases.
Times are milliseconds after `bundle.write()` was called,
before the fix,
and the plugin's own tag is `probe-after-shared`:

```text
    19.3 ms probe-after-shared.buildStart (final dir): cli.mjs size=384 ino=20519312
   191.5 ms probe-after-shared.renderStart: cli.mjs ABSENT
   262.8 ms probe-after-shared.generateBundle: cli.mjs ABSENT
   264.8 ms probe-after-shared.writeBundle: cli.mjs size=384 ino=20724880
```

The file is gone before `renderStart` and comes back at `writeBundle`,
and every build passes through both.

## Fix

`package/config/rolldown/src/atomic-output-dir.ts` holds a new rolldown plugin,
`atomicOutputDirectoryPlugin`,
and `nodeConfig` hands the bundler a staging directory beside the output directory
(`<parent>/.<name>.building`) instead of the output directory itself.

- `buildStart` removes what an earlier build left staged and creates the staging directory.
  The name is derived from the output directory alone,
  so a build that died before its write phase leaves one directory that the next build of the
  same package clears,
  and nothing accumulates.
- `writeBundle`,
  in the ordered `post` group,
  renames one staged file into place per emitted name,
  shared chunks first and entry chunks last,
  so a reader that opens an entry finds the files its imports name already in place.
  It then removes the files this build did not emit,
  and then the staging directory.
- The ordering group matters:
  `writeBundle` hooks run as one parallel batch and `shebangExecutablePlugin` chmods a staged
  file in its own hook,
  so a rename in the plain batch could race it.
  The chmod lands on the staged file and the mode survives the rename.

The end state `output.cleanDir` produced is kept:
the output directory holds this build's files only.
`output.cleanDir` now decides that removal and nothing else,
so a consumer that set `cleanDir: false` keeps its sibling build's files,
which is what `package/kwin/key-helper` relies on when the second of its two configs writes
into the first one's output directory.

## Verification

The same build after the fix,
traced the same way,
deletes nothing under `dist/final/node` (no `unlink` names a path there)
and renames each staged file into place:

```text
772839 08:58:44.683097 chmod(".../package/mvm/cli/dist/final/.node.building/cli.mjs", 0755) = 0
772838 08:58:44.689078 rename(".../package/mvm/cli/dist/final/.node.building/cli.mjs", ".../package/mvm/cli/dist/final/node/cli.mjs") = 0
```

The probe plugin,
run the same way after the fix:

```text
    49.7 ms probe-after-shared.buildStart (final dir): cli.mjs size=384 ino=20262924
   369.6 ms probe-after-shared.renderStart: cli.mjs size=384 ino=20262924
   471.1 ms probe-after-shared.generateBundle: cli.mjs size=384 ino=20262924
   474.8 ms probe-after-shared.writeBundle: cli.mjs size=384 ino=20842707
```

The path holds the previous build's complete file at every hook,
and the finished build's file is a different inode:
the name was replaced by a rename,
never written over.

`package/config/rolldown/src/atomic-output-dir.unit.test.ts` holds that as a check.
It reads the path at `renderStart` and `generateBundle` of a real second build,
so it needs no timing and no polling,
and it fails on the deleted-before-written shape.
Four packages' outputs came out byte for byte what they were without the change,
compared by `sha256sum` over every file of each output directory:

- `package/mvm/cli`,
  `dist/final/node`:
  13 files.
- `package/terminal-exec/cli`,
  `dist/final/node`:
  5 files.
- `package/pi-plugin/goal`,
  `dist/final/node`:
  2 files.
- `package/claude-code-plugin/bash-output-filter`,
  `bundle/node`:
  4 files.

`package/mvm/cli:test:unit` passes against the rebuilt command,
and `package/config/rolldown:test:unit` passes.

## What this does not change

- A shim written by an install older than the manifest's `bin` still names the built file.
  `pnpm install` in a checkout is what rewrites it,
  and that run rewrites every shim under the workspace's `node_modules/.bin`,
  re-links each package's dependency folder,
  and reconciles the install state with the lockfile.
  Nothing in this change runs it.
- `perEntryNodeConfig` keeps writing each entry's file in place,
  because it never cleaned the output directory.
  The nine `package/claude-code-plugin/*` commands whose `bin` names
  `bundle/node/index.mjs` are built that way,
  so a reader can still see one of those bundles half written while its build runs;
  no path there is absent,
  because nothing deletes those files.
- `package/config/rolldown/src/index.client.ts` and `src/index.ts` keep `cleanDir: true`.
  No command the repository installs runs from `dist/client` or `dist/final/neutral`.
