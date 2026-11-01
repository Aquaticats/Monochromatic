# Depot and GitHub sibling-branch cache scope

## Purpose and admission

This experiment compares actual benign-marker restoration from sibling branches in this repository.
It executes no application checkout or application tests.
The [machine receipt](../audit/depot-cache-trust-scope-2026-10-18.evidence.json)
records the admission before dispatch and will retain actual outcomes.

The separate compute-proxy guard is `USD 0.10`.
The initial queue contains six sequential jobs with two-minute job limits:
seed on branch A,
restore on branch A,
then restore on branch B,
first on standard public GitHub runners and then on Depot GitHub runners.
Only one job in this campaign may be active.
Cloud starts also require coordination with the parent research lanes.

The uniform two-CPU reference proxy is `6 × 2 × USD 0.006 = USD 0.072`.
One correction window adds `USD 0.012`,
giving an admitted maximum of `USD 0.084`.
This reference proxy is distinct from each product's nominal compute:
standard public GitHub compute is zero,
and the three requested Depot two-CPU jobs have a full-window nominal reservation of `USD 0.036`.
Neither quantity measures an invoice,
VM lifetime,
or cache storage charges.

## Deciding sources and controls

The complete current [Depot cache integration documentation][depot-cache]
has SHA-256 `0f0d228364a9bf30fa1b17080c4df92ac3549f88df5d34859ba2106a4848020f`.
It says the cache is scoped by repository and does not enforce branch isolation.
The complete current [GitHub cache reference][github-cache]
says a workflow cannot restore a sibling branch's cache.
Both are claims to compare against execution,
not expected outcomes to substitute for execution.

The latest public `actions/cache` release is `v6.1.0`,
resolved to `55cc8345863c7cc4c66a329aec7e433d2d1c52a9`.
The read-only source clone is
`/var/home/user/temp/agent/depot-cache-action-source-20261018`.
The probe uses the separate `save` and `restore` actions at that immutable revision.
Restore jobs cannot create new cache entries through a post-save hook.
The source's bundled `getCacheVersion()` hashes the literal input paths,
compression method,
platform distinction where applicable,
and version salt `1.0`.
The probe records those inputs and the resulting version.

Branch A and branch B will identify the same workflow commit.
Each provider gets a separate nonce key;
its exact key and literal path remain identical across its three phases.
The same-branch restore must download and hash the seeded marker before a sibling-branch result is admitted.
Every restore starts with an absent destination.
The only cached file is the benign marker.
Artifacts retain marker hashes,
source refs,
safe endpoint hostnames,
cache outputs,
and actual resource observations.
No runtime token,
signed URL,
or broad credential is printed or injected.

## Queue and cleanup

The admission checkpoint preceded dispatch.
All six jobs are now terminal and successful;
no correction job was allocated.
The observed sibling result is a GitHub miss and a Depot hit,
after actual same-branch exact-hit controls for both products.
The [source-traced troubleshooting note](../troubleshooting/depot-github-cache-sibling-branches.md)
records exact runs,
marker hashes,
client behavior,
and the source limits.
Executable workflow changes live only in the owned research worktree.
The private controller is
`/var/home/user/temp/agent/depot-cache-trust-scope-20261018`.
The source branches are `research/depot-cache-scope-a-20261018`
and `research/depot-cache-scope-b-20261018`.
The default-branch workflow remains unchanged.

Both owned remote branches were removed after terminal collection,
and direct remote discovery verified their absence.
The local probe branch was deleted;
the existing research worktree returned to its original branch.
Every job retained a cleanup record confirming both owned temporary paths absent.
The twelve original ZIPs and their source/API bodies remain in the linked source archive.
Cache deletion and organization settings changes are outside this experiment.
The benign cache entries may remain under existing retention and eviction policies.
Depot documents a default of fourteen days and configurable retention;
GitHub documents removal after more than seven days without access.
The actual organization settings and cache eviction times are unmeasured.
Branch deletion will not be treated as proof of cache deletion.

The observed Depot job intervals total `25` seconds,
giving an elapsed list-compute proxy of `USD 0.0025`.
Standard public GitHub's nominal compute remains zero.
The uniform two-CPU rounded-completed proxy is `USD 0.036`;
all remaining reservations and the unused correction window are released.
No invoice or VM billing interval was measured.

[depot-cache]: https://depot.dev/docs/cache/integrations/github-actions
[github-cache]: https://docs.github.com/en/actions/reference/workflows-and-actions/dependency-caching
