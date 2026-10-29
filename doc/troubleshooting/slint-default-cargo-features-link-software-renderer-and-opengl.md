# Slint 1.18.1 default Cargo features link the software renderer and the OpenGL FemtoVG stack, and the `mcp` feature links the software renderer again

Tool under test:
Slint 1.18.1 (`slint`,
`i-slint-backend-selector`,
`i-slint-backend-winit`,
`i-slint-backend-testing`,
all 1.18.1;
tag `v1.18.1`,
commit `372cf0ee5577c3dfec309a45e7b778ba4e81b734`) built for `x86_64-unknown-linux-gnu`.
Trigger:
a crate that depends on `slint` with default features,
or that turns on `slint/mcp`,
while it means to render only
through wgpu.
Failure mode:
crates the application never draws with are compiled and linked (the CPU software renderer,
FemtoVG's OpenGL path with
`glutin`,
and the MCP server's software screenshots),
and a check that "the software renderer is not linked" fails.

This is a documented default and a feature that behaves as designed;
the note records the exact feature graph so that
the IDE's feature list is not rediscovered.
The migration it belongs to is described in `doc/handover/slint-ide-0x.md`.

## Symptom

A scratch crate with `slint = "=1.18.1"` and nothing else links the software renderer and the OpenGL stack:

```text
$ cargo tree --offline --target x86_64-unknown-linux-gnu --edges normal,no-proc-macro --invert i-slint-renderer-software
i-slint-renderer-software v1.18.1
├── i-slint-backend-winit v1.18.1
│   └── i-slint-backend-selector v1.18.1
└── slint v1.18.1
    └── slint-default v0.1.0

$ cargo tree ... --invert glutin
glutin v0.32.3
├── glutin-winit v0.5.0
│   └── i-slint-backend-winit v1.18.1
└── i-slint-backend-winit v1.18.1 (*)
```

The normal dependency graph of that crate has 274 distinct crates,
18 more than the same crate with the default list
rewritten without the two renderer features (256,
below):
`femtovg`,
`glow`,
`glutin`,
`glutin-winit`,
`glutin_egl_sys`,
`glutin_glx_sys`,
`i-slint-renderer-femtovg`,
`i-slint-renderer-software`,
`softbuffer`,
`swash`,
`tiny-xlib`,
`yazi`,
`zeno`,
`imgref`,
`ctor`,
`either`,
`fnv` and `itertools`.
(This counts crates in the graph,
not compile time,
which I did not measure.)

Turning on only the MCP feature on top of the trimmed list brings the software renderer back:

```text
$ cargo tree ... --invert i-slint-renderer-software       # features: std backend-default accessibility compat-1-2 system-tray mcp
i-slint-renderer-software v1.18.1
├── i-slint-backend-testing v1.18.1
│   └── i-slint-backend-selector v1.18.1
│       └── slint v1.18.1
│           └── slint-mcp v0.1.0
└── i-slint-backend-winit v1.18.1
    └── i-slint-backend-selector v1.18.1 (*)
```

## Root cause

The features are written that way in the published manifests.

`slint`'s own default list includes both renderers:

```toml
# slint-1.18.1/Cargo.toml:115-123
default = [
    "std",
    "backend-default",
    "renderer-femtovg",
    "renderer-software",
    "accessibility",
    "compat-1-2",
    "system-tray",
]
```

`renderer-femtovg` is the OpenGL FemtoVG renderer (`i-slint-renderer-femtovg/opengl`,
`slint-1.18.1/Cargo.toml:140-144`);
the wgpu flavor is a separate feature,
`renderer-femtovg-wgpu` (`:145-149`).
`renderer-software` enables the CPU renderer in the selector and `dep:i-slint-renderer-software` (`:165-168`).
In the dependency graph,
the winit backend reaches the software renderer through the selector's `renderer-software`
feature,
which `slint/renderer-software` switches on.

The MCP feature does not name the software renderer in `slint`,
but the layers below do:

```toml
# slint-1.18.1/Cargo.toml:132-135
mcp = [
    "std",
    "i-slint-backend-selector/mcp",
]
```

```toml
# i-slint-backend-selector-1.18.1/Cargo.toml:56-59
mcp = [
    "renderer-software",
    "i-slint-backend-testing/mcp",
]
```

```toml
# i-slint-backend-testing-1.18.1/Cargo.toml:36-53 (abridged)
mcp = [
    "prost",
    ...
    "protox",
    "renderer-software",
]
```

The MCP server takes screenshots with the software renderer,
which is why the feature needs it.
`doc/troubleshooting/slint-mcp-winit-femtovg-screenshot.md` records the symptom on the screenshot side.
A second default-on edge exists in the build script:
Slint's changelog for the 1.18 line lists "slint-build:
Software-renderer resource embedding is now an optional default-on Cargo feature (`renderer-software`),
so it can be
disabled to reduce build dependencies" (`CHANGELOG.md`,
pull request 11554).
That one runs at build time on the host and is not part of the linked binary;
I did not evaluate it further.

The `compat-1-2` feature,
which a crate that disables default features is told to select,
does not include
`renderer-software`:
only `compat-1-0` does (`slint-1.18.1/Cargo.toml.orig:39-40`).

## Verification

Scratch crates,
each `edition = "2024"`,
an empty `fn main() {}`,
`[workspace]` in its manifest,
`cargo generate-lockfile --offline` and then `cargo tree --offline`,
with cargo from rustc 1.101.0-nightly (2026-10-17):

```toml
# default/Cargo.toml
[dependencies]
slint = "=1.18.1"
```

```toml
# listed/Cargo.toml
[dependencies]
slint = { version = "=1.18.1", default-features = false, features = [
    "std", "backend-default", "accessibility", "compat-1-2", "system-tray",
] }
```

```toml
# mcp/Cargo.toml
[dependencies]
slint = { version = "=1.18.1", default-features = false, features = [
    "std", "backend-default", "accessibility", "compat-1-2", "system-tray", "mcp",
] }
```

Each tree was taken with `--target x86_64-unknown-linux-gnu --edges normal,no-proc-macro`.

### Runs that show the software renderer and OpenGL stack

The `default` crate (above),
and the `mcp` crate for the software renderer (above).

### Runs that do not

The `listed` crate prints `warning: nothing to print` for `--invert i-slint-renderer-software` and for
`--invert glutin`,
and `--invert i-slint-renderer-femtovg` does not match a package.

The IDE's own manifest (`package/ide/app/Cargo.toml:84`) uses
`default-features = false` with `std`,
`backend-default`,
`backend-winit`,
`renderer-femtovg-wgpu`,
`unstable-wgpu-30`,
`unstable-winit-030`,
`accessibility`,
`compat-1-2` and `system-tray`.
Run from `package/ide/app`,
the same three inverted trees print `nothing to print` for
`i-slint-renderer-software` and `glutin`,
and `i-slint-renderer-femtovg` is reached only through the selector,
the winit
backend and `slint` (which is the wgpu flavor there).
The package's task `inspect:no-software-renderer` (`package/ide/app/mise.toml:175-192`) runs this check inside the
container with `--all-features`,
and takes `--features slint/renderer-software` and `--features slint/mcp` as positive
controls that must reach the renderer;
I read the task and ran the equivalent `cargo tree` on the host,
not the task.

## Verified workarounds

### Disable default features and list what is needed

```toml
slint = { version = "1.18.1", default-features = false, features = ["std", "backend-default", "renderer-femtovg-wgpu", "accessibility", "compat-1-2", "system-tray"] }
```

Tradeoff:
the list must be kept in step with Slint's defaults on every upgrade (for example a new default-on feature
that `compat-1-2` guards),
and `renderer-femtovg-wgpu` and the wgpu hooks used by the IDE are marked `unstable-*`,
which Slint exempts from semver.

### Do not turn on `mcp` in builds that must not link the software renderer

`slint/mcp` cannot be combined with that goal,
because the selector's `mcp` feature enables `renderer-software`.
The owner decided on 2026-10-20 to drop the IDE's MCP builds and rebuild their checks without Slint's MCP server
(`doc/handover/slint-ide-0x.md`,
"Owner answers of 2026-10-20");
the positive control `--features slint/mcp` in the
package's `inspect:no-software-renderer` task keeps proving the edge.
The applications that `doc/troubleshooting/slint-embedded-mcp-server.md` is about (the music player and the terminal)
are other packages and are not covered by this note.

## What does not work

- `slint = "1.18.1"` plus a hope that wgpu is chosen at run time:
  the OpenGL and CPU renderers are still compiled in.
- `--no-default-features` without `compat-1-2`:
  Slint requires that feature for forward compatibility
  (`slint-1.18.1/Cargo.toml.orig:33-37`),
  so the list above names it.
- `features = ["mcp"]` and then expecting no CPU renderer:
  the first inverted tree above.

## Upstream filing artifact

### Upstream filing decision

`.out-of-scope/` entries (`bun-install.md`,
`cargo-workspace.md`,
`claude-code-upstream-bugs.md`,
`codex-harness.md`,
`jsr.md`,
`lightningcss.md`,
`low-impact-typescript-formatting.md`,
`module-es-monolith.md`,
`pi-gpt55-long-context.md`,
`terminal-title-fork-parity-tests.md`,
`typescript-project-references.md`) do not name Slint features.

Duplicate search,
`gh search issues` and `gh search prs` on `slint-ui/slint`,
open and closed:
`default features software renderer`,
`renderer-software default`,
`mcp feature software renderer`,
`reduce compile time default features`,
`disable default features femtovg`,
`renderer-femtovg default opengl`,
`mcp take_screenshot software renderer`,
`mcp screenshot FemtoVG`,
`default-features = false compat-1-2`,
`slint-build renderer-software feature`,
`OpenGL dependency default feature wgpu only`.
No issue or pull request asks to change the default list or the `mcp` edge.
The nearest hits are unrelated:
issue 10213 (a CPU-type error in `slint-lsp`) and issue 13766 (an MCP element-handle
regression,
closed).
Slint's own changelog shows the direction they chose for the build side:
the `renderer-software` feature of
`slint-build` was made optional but default-on (pull request 11554).

The six constraints:

1. **Is it really upstream's fault?**
   No.
   The default list and the `mcp` to `renderer-software` edge are intentional and documented in the manifests,
   and the
   `compat-1-2` mechanism exists so that defaults stay stable across patch releases.
2. **Can upstream fix it?**
   Not without breaking users of the defaults;
   trimming a default feature is a semver-visible change.
3. **Are they supporting this use case?**
   Partly:
   disabling default features is the documented path (the Linux KMS page shows it),
   and the wgpu FemtoVG
   renderer is a supported feature,
   but I found no document that promises a build free of the CPU renderer with `mcp`.
4. **Would the repo welcome our contribution?**
   Nothing bans it:
   `CONTRIBUTING.md` of `v1.18.1` tells AI coding assistants to read `AGENTS.md`,
   asks for small
   focused pull requests,
   and asks that a design change be discussed in an issue first.
   A change to the defaults would be that kind of discussion.
5. **Will they likely fix it?**
   No:
   nothing in the tracker asks for it,
   and the design points the other way.
6. **Have we prototyped a minimal fix compatible with their architecture?**
   No,
   and none is warranted:
   constraint 1 fails.

Constraint 1 fails,
so the auto-prototype step is not triggered and nothing is filed.
Draft:
none.
The follow-up that is ours:
when `slint` is upgraded,
rerun the three inverted trees (or `inspect:no-software-renderer`) and
compare the default list in `slint-1.18.x/Cargo.toml` with the IDE's.

## Evidence and limits

- Read `slint-1.18.1`,
  `i-slint-backend-selector-1.18.1` and `i-slint-backend-testing-1.18.1` manifests from the cargo
  registry,
  and a `--depth 1` clone of `v1.18.1` for the documentation pages.
- The crate counts are from `cargo tree` and measure the graph only,
  not compile time or binary size.
- Whether `slint-build`'s default `renderer-software` pulls extra build-time crates into the IDE was not measured.
- I did not run the package's container task `inspect:no-software-renderer`.
- The two existing MCP notes (`doc/troubleshooting/slint-embedded-mcp-server.md`,
  `doc/troubleshooting/slint-mcp-winit-femtovg-screenshot.md`) never name the IDE (searched for `ide/app`,
  `editor`,
  `helix` and `both apps`;
  their "both apps" are the music player and the terminal),
  so the owner's decision
  to drop the IDE's MCP builds makes nothing in them false and they were not edited.
