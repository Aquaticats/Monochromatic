# cli-git-clone-size

Estimate a git repository's shallow-to-full clone size ratio,
 and the savings a
shallow clone buys,
 without ever doing a full clone.

A shallow clone (`git clone --depth 1`) downloads only the tip snapshot;
 a full
clone pulls every historical version of every blob.
 This tool quantifies that
saving while staying cheaper than the full clone it estimates:
 the heaviest
operation it ever runs is a shallow or filtered clone plus a few small bounded
`--deepen` probes.

## Usage

```sh
git-clone-size [SOURCE] [flags]
```

`SOURCE` is either a remote clone URL (any host) or a local path.
 When omitted,
the current directory is used.

```sh
# Local complete repo: exact measurement
git-clone-size .

# Remote URL: estimated from cheap probes
git-clone-size https://github.com/owner/repo.git

# Tighter, cleaner metric restricted to the default branch
git-clone-size --default-branch-only https://github.com/owner/repo.git

# Force plain output for jq
git-clone-size --color=never . | jq .
```

## Output

Output is JSONL only:
 one `EstimateSnapshot` object per line on stdout,
 nothing
else.
 The first snapshot is emitted the instant the cheapest signal lands,
 and a
refined snapshot is emitted on every further signal,
 ending with the tightest
fused estimate (`done: true`).
 Diagnostics go to stderr,
 so stdout stays valid
for `jq` and other parsers.

Each size carries both a raw byte count and a human string,
 so a line is both
machine- and human-readable:

```jsonl
{"metric":"...","scope":"...","full":{"confidence":"low","point":{"bytes":52428800,"human":"50.0 MiB"},"lo":{"bytes":1048576,"human":"1.0 MiB"},"hi":{"bytes":524288000,"human":"500.0 MiB"}},"basis":["prior (no signals yet)"],"pending":["shallow","deepen","commit-count","refs","churn","host-proxy"],"done":false}
{"metric":"...","scope":"...","shallow":{"bytes":4404019,"human":"4.2 MiB"},"full":{"confidence":"medium","point":{"bytes":99614720,"human":"95 MiB"},"lo":{"bytes":62914560,"human":"60 MiB"},"hi":{"bytes":157286400,"human":"150 MiB"}},"ratio":{"point":0.0442,"lo":0.028,"hi":0.07},"savings":{"point":95.6,"lo":93,"hi":97.2},"basis":["deepen-extrapolation(repack-corrected)+branch correction","snapshot-multiplier prior"],"pending":[],"done":true}
```

`ratio` and `savings` appear only once both `shallow` and `full` exist.

### Color

Color is automatic.
 When stdout is a TTY the JSONL is ANSI-highlighted for
humans;
 when piped or redirected the lines are plain so they stay valid JSON.
`--color=auto|always|never` forces the mode (default `auto`).
 `NO_COLOR` and
`FORCE_COLOR` are honored under `auto`.
 Stripping the ANSI escapes from a colored
line always yields the identical JSON.

## How to read the range and confidence

Every estimate is a range,
 never a single number,
 because a generic remote's
full size is not directly knowable cheaply:
 the git smart-protocol handshake
exposes refs and capabilities only,
 never total size.
 The tool fuses several
cheap,
 independent signals into a credible interval (`lo`,
 `point`,
 `hi`) and a
confidence level:

- `very high`:
   a local complete repo measured exactly by `pack-objects`.
   Tiny
  band,
   treated as ground truth.
- `high`:
   a calibrated host storage proxy,
   the local size-pack huge-repo
  fallback,
   or several independent estimators agreeing within a tight band.
- `medium`:
   the deepen extrapolation with a known commit count,
   no exact source.
- `low`:
   prior-only,
   a capped commit count (lower bound),
   or a probe that hit a
  budget.

Confidence drops one step when estimators conflict (their intervals do not
overlap);
 the fused band then widens to their union rather than silently
averaging.
 The snapshot always reflects the current best belief,
 so ranges
generally tighten as signals arrive but a new signal may shift them;
 this is not
a monotonic guarantee.

## Metric contract

The tool states its metric and scope in every line so numerator and denominator
are never mismatched.

- Default metric:
   `shallow` is the object store of `git clone --depth 1 <url>`
  (default branch,
   depth 1);
   `full` is the object store of an ordinary
  `git clone <url>` (all branches and tags,
   full history),
   well-packed into a
  single pack.
   Both sides are measured on the same packing basis (one
  well-packed pack),
   so the ratio is apples-to-apples and reflects transfer pack
  bytes,
   not the client-generated `.idx`.
- `--default-branch-only` flips both sides to default-branch-only:
   a tighter,
  cleaner metric with no branch-coverage correction and a narrower range.
- Scope,
   always printed:
   git object database only;
   excludes working tree,
  submodules,
   and Git LFS payloads.

## Methods

- Local complete repo (exact):
   `git rev-list --objects --branches --tags |
  git pack-objects --stdout --delta-base-offset | wc -c`,
   scoped to the tip for
  the shallow side.
   Correct by reachability,
   so alternates,
   `--shared` clones,
  and linked worktrees are handled without summing the alternate store.
   Above a
  size threshold it falls back to the cheap `count-objects` size-pack proxy.
  Never mutates the repo (no `gc`/`repack` on the user's store).
- Shallow probe (remote):
   a depth-1 bare clone yields the compressed tip size.
- Deepen extrapolation:
   a few bounded `git fetch --deepen` steps measure
  marginal compressed bytes per commit;
   a bounded `repack` of the temp clone
  corrects the incremental-pack bias.
   Then `full ~= C1 + m * (N - 1)`.
- Commit count:
   a host API (GitHub `Link` `rel=last`),
   else a `--filter=tree:0`
  partial clone,
   else the deepen walk (a lower bound if capped).
- Churn:
   a `--filter=blob:none` partial clone relates historical path objects to
  tip files.
   Approximate,
   so it carries a low weight and a wide band.
- Host storage proxy:
   GitHub `.size` (via `gh`) or GitLab `repository_size`
  (via `glab`),
   treated as a proxy for the packed clone size.
- Branch-coverage correction:
   `git ls-remote` counts refs to estimate the
  side-branch contribution the default-metric full includes beyond the
  default-branch history the deepen probe samples.

## Guards and budgets

Probes are bounded so the tool stays cheaper than the clone it estimates:

- `--max-probe-seconds` (default 60):
   a wall-clock budget enforced by aborting
  in-flight clones (the git child is killed),
   not just by ignoring late results.
- `--max-deepen-commits` (default 256):
   caps the deepen walk;
   hitting the cap
  makes the commit count a lower bound (wider range).
- `--max-pack-bytes` (default 2 GiB):
   above this,
   the local path uses the cheap
  size-pack proxy instead of the heavy exact pack.

A probe that fails (unsupported filter,
 missing API,
 detached HEAD) or trips a
budget folds into a wider range and lower confidence,
 named in `basis`;
 it never
crashes or refuses.
 SIGINT aborts the probes and finalizes with the current best
snapshot.

The byte-download and partial-object budgets in the original design are bounded
indirectly here:
 the time budget kills slow clones,
 and the deepen-commit and
pack-byte caps bound the heavy work.
 A mid-clone byte meter is not implemented.

## Caveats

- LFS payloads and submodule contents are excluded;
   the scope line is always
  printed so the object-db size is never conflated with the full
  checkout/network cost.
- The host storage proxy is server storage,
   not the client-side packed object
  store;
   it contributes at `medium` confidence (not `high`) without calibration.
- Under Bun,
   `styleText` from `node:util` does not reliably honor a non-TTY
  stream or `NO_COLOR`,
   so the color decision is made explicitly before any
  styling rather than delegated to `styleText`.
   See
  `src/color.ts` (`shouldColor`).
