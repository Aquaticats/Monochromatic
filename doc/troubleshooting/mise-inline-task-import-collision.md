# Mise inline task interpolation can redeclare JavaScript imports

## Symptom

A mise task using Node's inline TypeScript shell fails before running its command:

```txt
SyntaxError: Identifier 'execFileSync' has already been declared
```

The emitting tool is Node while evaluating mise's assembled `run` source.
The affected task was `//package/nested-wayland-session/cli:run`.
Build and test tasks still worked because they used only the shared dispatch snippet.

## Cause

`package/nested-wayland-session/cli/mise.toml` defines `vars.cargo_dispatch` as executable JavaScript.
That snippet imports `execFileSync`.
The `run` task interpolated the snippet and then imported `execFileSync` again:

```ts
const { execFileSync } = await import('node:child_process')
// ...interpolated into the same module scope...
const { execFileSync } = await import('node:child_process')
```

Mise passes the combined text to one `node --input-type=module-typescript -e` invocation.
Both `const` declarations therefore occupy one module scope.
This is a JavaScript binding collision,
not a missing package or compositor failure.

## Fix

Import a shared binding in one place only.
The dispatch snippet deliberately owns `execFileSync`,
so inline follow-up code reuses that binding:

```toml
run = """
{{vars.cargo_dispatch}}
runCargo(['build', '--release'])
const args = process.argv.slice(1)
execFileSync('./target/release/monochromatic-nested-wayland-session', args, { stdio: 'inherit' })
"""
```

Do not work around the collision by changing `const` to `var` or by hiding the diagnostic.
A single owner keeps interpolated task code readable and prevents divergent imports.

## Verification

Run the actual task rather than only its build dependency:

```sh
mise run //package/nested-wayland-session/cli:run -- --color-scheme dark -- app
```

The task must pass argument parsing,
build the release,
and launch the compositor without the duplicate-binding syntax error.
A GUI fixture should then confirm its control socket and hosted client start normally.
