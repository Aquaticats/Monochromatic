# @monochromatic-dev/pnpm-override-stub-throwing

Ready to publish.

A workspace stub package used by `.pnpmfile.mjs` to substitute for blocked dependencies whose policy is `throw`.

When a package is blocklisted with `action: 'throw'`
in the policy table at the repo root's `.pnpmfile.policies.json`
(read by the hook in `.pnpmfile.mjs`),
the pnpm install hook rewrites every transitive dependency entry
pointing at the blocked package to point here instead.
Any consumer that loads the stub at runtime evaluates `index.cjs`,
 which immediately throws an error naming the policy file.
Consumers wrapping the import in `try`/`catch` see the same error inside their catch.

See `doc/dependency-blocklist.md` for the full mechanism and the decision rule for choosing between `throw`,
 `silent`,
 and global removal in `pnpm-workspace.yaml`.
