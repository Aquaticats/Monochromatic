# @monochromatic-dev/pnpm-override-stub-silent

Ready to publish.

A workspace stub package used by `.pnpmfile.mjs` to substitute for blocked dependencies whose policy is `silent`.

When a package is blocklisted with `action: 'silent'`
in the policy table at the repo root's `.pnpmfile.policies.json`
(read by the hook in `.pnpmfile.mjs`),
 the pnpm install hook rewrites every transitive dependency entry pointing at the blocked package to point here instead.
The stub exports a callable Proxy whose every property access,
 function call,
 and `new` invocation returns the Proxy itself,
 and whose `in` checks return `false`.
This lets shape-probing code (`typeof X === 'function'`,
 `X.someMethod`,
 `new X()`,
 `X()`) run without throwing,
 at the cost of incorrect downstream behavior whenever the stub's value is actually used.

Use this only for soft migrations where you want the build green
and you accept incorrect-but-not-crashing runtime behavior.
For a loud failure,
 use `@monochromatic-dev/pnpm-override-stub-throwing` instead.
For optional dependencies that consumers handle via `try { require } catch`,
 use global removal in `pnpm-workspace.yaml`.

See `doc/dependency-blocklist.md` for the full mechanism and decision rule.
