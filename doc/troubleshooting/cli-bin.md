# `node_modules/.bin/` CLI tools without shebang hang on Unix with ImageMagick `import` errors

## Symptom

Running a CLI tool installed via `package.json` `bin` (resolved through
`node_modules/.bin/<name>`) hangs indefinitely,
 flooding stderr with a
wall of ImageMagick messages:

```text
import: unable to grab mouse '': Resource temporarily unavailable @ error/xwindow.c/XSelectWindow/9351
import: unable to read X window image '': Success @ error/xwindow.c/XImportImage/4961
import: missing an image filename `node:util' @ error/import.c/ImportImageCommand/1289
```

Affects `*nix` only.
 The same `bun run <file>` invocation works because
Bun dispatches on the file itself rather than the OS execve path.

The trap appears on every TypeScript CLI entry point in this workspace
whose first line is not a shebang.

## Root cause

When a `bin` entry resolves to a script with no shebang line,
 the kernel
cannot determine which interpreter to invoke.
 POSIX (`execve(2)`) falls
back to `/bin/sh` in that case:

> "If the new process image file has appropriate privilege and is in
> a recognized executable file format,
>  but is in a format the system
> does not recognize,
>  the equivalent of `/bin/sh` shall be invoked.
> "

`/bin/sh` reads the file as a shell script and walks line by line.
 The
first TypeScript line is typically an import:

```typescript
import { parseArgs, } from 'node:util';
```

Bash parses `import` as a command.
 ImageMagick installs an `import`
binary (a screenshot tool that grabs the X display),
 so `/bin/sh`
resolves the word,
 invokes it,
 and the script is now feeding `import`
its remaining tokens (`{`,
 `parseArgs,`,
 `}`,
 `from`,
 `'node:util'`)
as if they were filenames.
 `import` opens a connection to the X server,
blocks waiting for a mouse selection that never arrives,
 eventually
errors,
 and the script either hangs or floods stderr with X11 errors.

This is not a bug in the package manager;
 the package manager is wiring
the bin entry exactly as `package.json` declares it.
 The bin script is
malformed:
 a `bin` entry must announce its interpreter via a shebang on
the first line.

## Verification

Verified against:

- The workspace's mise-managed Node runtime
- pnpm 10.
  x,
   npm 11.
  x,
   yarn 4.
  x (all generate identical bin symlinks on
  Unix)
- ImageMagick `import` 7.
  x present in `PATH` (the default on most
  Linux desktop installs and on CI runners with X11 packages)

Reproduce:
 install a CLI package whose entry point file omits the
shebang,
 then invoke through the bin symlink:

```bash
echo 'import { argv } from "node:process";
console.log(argv.slice(2,));' > cli.ts
chmod +x cli.ts

ln -s "$PWD/cli.ts" "$PWD/cli"  # mimic node_modules/.bin/cli
./cli --hello
# Floods stderr with `import: unable to grab mouse`...
```

Now add a shebang:

```bash
sed -i '1i #!/usr/bin/env node' cli.ts
./cli --hello
# Prints: [ "--hello" ]
```

Same script,
 same kernel,
 same package manager wiring.
 The shebang is
the entire difference.

## Verified workarounds

### Add the shebang to the entry point

```typescript
#!/usr/bin/env node
import { parseArgs, } from 'node:util';
// ...
```

Reinstall so the bin symlink/wrapper is regenerated with the corrected
file in place:

```bash
pnpm install
```

Tradeoff:
 every CLI package's entry point now has a non-TypeScript first
line.
 `dprint`,
 `oxlint`,
 and the workspace TSDoc tooling all tolerate
the shebang (the `#!` line is treated as a comment by every TS-aware
tool we use),
 so this cost is purely cosmetic.

### Pin the interpreter explicitly via `node`

If editing the entry file is impossible (vendored fixture,
 generated
file),
 invoke the script through `node <file>` instead of the
`bin` symlink.
 This bypasses the kernel's shebang inspection entirely
because `node` parses the file itself.

Tradeoff:
 callers must know to use `node`;
 the bin symlink remains
broken for anyone who doesn't.

## What does not work

- Adding `"type": "module"` or `"engines"` constraints to
  `package.json`:
   those fields are advisory metadata read by tooling,
  not by the kernel.
   They do not affect what `/bin/sh` does on
  shebang-less files.
- Marking the file executable with `chmod +x` but no shebang:
   makes the
  symlink point at an executable file,
   which is still in an unknown
  format to the kernel,
   which still falls back to `/bin/sh`.
- Renaming the entry file to `.js`:
   extension does not matter;
   the
  kernel only inspects the first bytes of the file,
   and bash still sees
  TypeScript syntax it cannot parse.
- Relying on `node` always being PATH-resolved first:
   if a developer has
  no `node` on PATH (e.g. CI mis-provisioning),
   `/usr/bin/env node` fails
  loudly with `env: 'node': No such file or directory`.
   That is the
  intended failure mode,
   distinct from the silent ImageMagick hang.

## Cross-platform notes

The shebang is a Unix mechanism;
 Windows ignores it entirely.
 On
Windows,
 package managers (npm,
 pnpm,
 yarn) generate `.cmd` and `.ps1`
wrapper scripts in `node_modules/.bin/` that invoke the runtime
explicitly,
 so the shebang in the source file is never read by Windows
at all.

Both mechanisms are required for full cross-platform support:

- **Unix**:
   shebang tells the kernel which interpreter to use.
- **Windows**:
   generated `.cmd` wrapper invokes the runtime;
   shebang is
  ignored.

Adding a shebang never breaks Windows;
 it is purely additive.

## Why we do not file this upstream

There is no single upstream to file.
 The interaction involves the OS
execve fallback,
 ImageMagick's choice of `import` as a binary name,
bash's command lookup,
 and the absence of a shebang in our own code.
Walking the five constraints anyway:

1.  **Is it really upstream's fault?
    ** No. Every component behaves as
    documented;
     the workspace's CLI scripts were the ones missing the
    shebang.
2.  **Can upstream fix it?
    ** Each upstream is correct individually.
    Asking ImageMagick to rename `import` would break decades of
    existing scripts;
     asking bash to refuse running unknown-format files
    would break the POSIX fallback contract.
3.  **Are they supporting this use case?
    ** ImageMagick supports
    `import`;
     bash supports executing scripts that happen to call it.
    The use case is "run a malformed script";
     no upstream is required
    to make that work.
4.  **Will they likely fix it?
    ** N/A.
5.  **Have we prototyped a minimal fix?
    ** Yes,
     on our side:
     add the
    shebang.
     There is no upstream fix to prototype.

Decision:
 no upstream report.
 The fix lives at our boundary (the
shebang on each CLI entry file) and is captured by the `AGENTS.md`
workspace convention for adding new packages.
