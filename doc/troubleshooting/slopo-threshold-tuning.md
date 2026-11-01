# Slopo 0.4.0: raising a threshold for one incidental match hides useful reports

The title pins the version this investigation verified.
`mise.toml:213` installs `pipx:slopo` at `latest`,
so the running version moves without any signal in this file.
The section `Re-verification against slopo 0.8.0 on 2026-10-09` records what moved,
what still holds,
and the state of the ignore inventory today.

## Symptom

Slopo reports the two `fix` callbacks in
`package/oxlint-plugin/stylistic/src/rule/block-body-newline.ts` as cluster
`6be7f9516c9d` with displayed score `0.96`.
The callbacks share Oxlint fixer scaffolding but use different ranges and indentation.
A global threshold increase can remove this report,
but it also removes unrelated reports that contain substantive shared behavior.

The effective repository configuration reported by `slopo show-config` is:

```text
similarity_threshold: 0.92
rerank_threshold: 0.94
body_node_count_threshold: 13
```

Against the copied current index and an empty throwaway ignore file,
the baseline produced `356` clusters.
Changing only `similarity_threshold` to `0.97` produced `225` clusters.
Changing only `rerank_threshold` to `0.97` produced `288` clusters.
The target cluster disappeared in both probes.

## Root cause

This is expected threshold behavior,
not a Slopo defect.
The displayed score is insufficient for deciding a global cutoff because Slopo applies two
non-semantic filters and rounds report scores.

### Raw similarity is filtered before clustering

In [Slopo v0.4.0][slopo-release],
`src/slopo/analysis/command.py:34-53` finds raw-similarity pairs,
clusters those pairs,
reranks them,
and then applies the second threshold:

```python
pairs = find_similar_pairs(embeddings, cfg.similarity_threshold, _BLOCK_SIZE)
# ...
clusters = build_clusters(pairs)

reranked_pairs = rerank_all_clusters(clusters, pairs, units)
clusters = reorder_clusters(clusters, reranked_pairs)
clusters = filter_clusters(clusters, cfg.rerank_threshold)
```

`src/slopo/analysis/similarity.py:18-23` applies `similarity_threshold` directly to cosine similarity:

```python
block = matrix[start:end] @ matrix.T
rows, cols = np.where(block >= similarity_threshold)
```

Raising this threshold changes graph edges before clustering.
It can therefore split or remove clusters rather than merely trim the end of the report.

### Reranking rewards source proximity, not semantic confidence

`src/slopo/analysis/rerank.py:18-27` multiplies cosine similarity by a location boost:

```python
def rerank_pair_score(
    pair: SimilarPair,
    unit_a: UnitRecord,
    unit_b: UnitRecord,
) -> float:
    if unit_a.file_path == unit_b.file_path:
        b = boost.same_file(_line_distance(unit_a, unit_b))
    else:
        b = boost.cross_dir(path_hops(unit_a.file_path, unit_b.file_path))
    return pair.similarity * (1 + b)
```

For same-file units,
`src/slopo/analysis/boost.py:8,16-18` advances the boost in `250`-line steps:

```python
_SAME_FILE_STEP_LINES = 250

def same_file(line_distance: int) -> float:
    steps = line_distance // _SAME_FILE_STEP_LINES
    return _distance_boost(steps, _SAME_FILE_MAX_STEPS, _SAME_FILE_MAX_BOOST)
```

Cluster `6be7f9516c9d` has a `33`-line gap,
so it receives no location boost.
Its raw and reranked score is `0.962768257`.
A cutoff greater than that value removes it,
but the cutoff does not know why the similarity is incidental.

### Reports round scores

`src/slopo/analysis/report/markdown.py:92-95` formats scores to two decimal places:

```python
def _similarity_range(cluster: Cluster) -> str:
    if cluster.min_similarity == cluster.max_similarity:
        return f"{cluster.min_similarity:.2f}"
    return f"{cluster.min_similarity:.2f}-{cluster.max_similarity:.2f}"
```

The displayed `0.96` therefore does not identify the exact cutoff needed to suppress a pair.
It also cannot distinguish shallow scaffolding from useful shared behavior.

### Ignore hashes are the intended review mechanism

Slopo applies reviewed cluster hashes after thresholding and clustering at
`src/slopo/analysis/command.py:63-69`
(that path is the `0.4.0` layout;
as of `0.8.0`,
re-verified 2026-10-09,
it is `slopo/result/analysis/command.py:65-73`):

```python
ensure_ignore_file(cfg.ignore_file)

ignored = load_ignored(cfg.ignore_file)
if ignored:
    kept = [c for c in clusters if cluster_hash(c, units) not in ignored]
```

The upstream workflow explicitly says that not every similar pair is actionable and directs reviewers
to add discarded cluster hashes to `slopo.ignore.txt`
(`README.md:89-94`).
The repository categorizes comparable tiny fixer matches under
`BOILERPLATE-TRIVIAL` in `slopo.ignore.txt`.

### Path migration made prior ignore hashes stale

`src/slopo/analysis/ignore.py:18-24` includes every unit path and body hash in a cluster hash
(that path is the `0.4.0` layout;
as of `0.8.0`,
re-verified 2026-10-09,
the hash is built in `slopo/result/identity.py:24-30`
and `slopo/result/analysis/ignore.py:14-22` holds only the loader and the header template):

```python
pairs = sorted(
    (units[uid].file_path, units[uid].body_hash) for uid in cluster.unit_ids
)
canonical = "\n".join(f"{path}\0{body_hash}" for path, body_hash in pairs)
```

Commit `ece5b7553` renamed the repository's `packages/` tree to `package/`.
The commit updated path text in `slopo.ignore.txt` but retained all `116` existing hash lines.
Those hashes necessarily changed for clusters containing renamed paths.

After the targeted block-body dismissal was added,
the pre-refresh ignore file contained `120` hashes.
Only `54a947960da7` and the new `6be7f9516c9d` intersected the current unignored report.

The refreshed inventory contains `55` unique hashes,
each present in the current `voyage/voyage-code-4` no-ignore report.
Historical unit identity recovered entries whose paths or bodies changed.
Explicitly labeled semantic remaps cover reviewed clusters that split or merged;
unrecoverable no-effect entries were deleted.

## Verification

Verified against:

- installed Slopo `0.4.0`,
   Python `3.14.6`,
   and SQLite `3.51.2`;
- upstream tag `v0.4.0`,
   commit `9b6296f2a6ab5e10cfdae7d6ed521f9bf3cb79fa`;
- copied repository index containing `8,528` code units and `8,309` stored embedding rows,
   covering every code unit through shared body hashes;
- repository source and reports as of `2026-08-16`.

The current index and every probe use the approved `voyage/voyage-code-4` model.
No other embedding model was invoked during the refresh.

All controlled threshold probes used a copied database,
an empty throwaway ignore file,
and separate report directories under a private throwaway directory.
They did not mutate the repository index or reports.

The probe configuration varied only these values:

```yaml
# Baseline
similarity_threshold: 0.92
rerank_threshold: 0.94

# Raw-similarity probe
similarity_threshold: 0.97
rerank_threshold: 0.94

# Rerank probe
similarity_threshold: 0.92
rerank_threshold: 0.97
```

The analysis command for each copied configuration was:

```bash
slopo --config /path/to/throwaway-config.yaml analyze
```

### Visible at current thresholds

These reports demonstrate that the low-score band contains both incidental scaffolding and substantive duplication:

- `0.962768257`,
   block-body opening and closing fixer callbacks:
  incidental framework shape;
- `0.964223504`,
   duplicate `resolveAddressFamily` implementations in
  `resolve_hosts.ts` and `resolve_storagebox_hosts.ts`:
  substantive shared behavior;
- `0.960972667`,
   duplicate checkbox and radio question CSS builders:
  substantive shared styling;
- `0.947156966`,
   duplicate attached-comment scans in `build-comments.ts` and `comments.ts`:
  substantive shared traversal;
- `0.940766573`,
   parallel JSON HTTP clients in `client-http.ts` and `exa-http.ts`:
  substantive transport behavior.

### Suppressed at a `0.97` threshold

Both `similarity_threshold: 0.97` and `rerank_threshold: 0.97` suppress every pair in the preceding catalog.
The raw-similarity probe produced `131` fewer clusters than the controlled baseline.
The rerank probe produced `68` fewer clusters than the controlled baseline.

The exact pair scores were calculated directly from the copied database embeddings with the same cosine formula and
boost functions used by Slopo.
Every catalog pair received zero location boost,
so each listed score is both its raw and reranked score.
The target pair was also absent from both generated report directories.

### Refreshed inventory

A disposable analysis with the refreshed ignore file logged
`Ignored 55 previously reviewed clusters` and produced `301` remaining clusters from the `356`-cluster baseline.
All ignored hashes were unique and present in the baseline report.
No ignored hash overlaps the currently visible music-player Android/Rust drift clusters,
and the whole fake Pi harness cluster remains visible.

## Re-verification against slopo 0.8.0 on 2026-10-09

Verified against installed slopo `0.8.0`
(`mise which slopo` resolves to
`~/.local/share/mise/installs/pipx-slopo/0.8.0/bin/slopo`),
the report generated 2026-10-07 22:36:47 in `.slopo.local.dir/`,
and `slopo.ignore.txt` as of that date.
The mechanism this document relies on is unchanged from `0.4.0`:
`config.py:120-121` still defaults `ignore_file` to `slopo.ignore.txt`,
the loader still drops everything from `#` onward,
the filter still runs after clustering,
and the hash is still sha256 over the sorted `(file_path, body_hash)` pairs of a cluster's units,
truncated to 12 hex characters.

### The whole inventory is dead keys

All 55 hashes in `slopo.ignore.txt` match nothing in the 2026-10-07 report.
Two probes on copies of that index,
each with its own report directory under `~/temp/agent`,
confirm it:

- an empty ignore file reported `1008 clusters with 4377 units`;
- the real ignore file reported the same `1008 clusters with 4377 units`
   and printed no `Ignored ... previously reviewed clusters.` line,
   which `slopo/result/analysis/command.py:65-73` emits only when a key matches.

Both runs produced an identical 1008-hash set.
That also means the report is reproducible from the same index on a later date,
and that the earlier `Ignored 55 previously reviewed clusters` measurement in
`Refreshed inventory` describes an older index rather than this one.
The practical consequence:
the inventory currently suppresses nothing,
so a dismissal issue that proposes entries without replacing the keys whose clusters re-surfaced
leaves lines in the file that look like suppressions and are not.

slopo has no dead-key detection.
`load_ignored` returns a set,
the filter logs only an aggregate count of matching clusters,
and nothing compares the file's keys against the current report.
The check is manual:
extract the hashes from `.slopo.local.dir/index.md` and intersect them with the file.

### Open cleanup decision for the 55 dead keys

The 2026-10-07 review ledger records a successor cluster for 32 of the 55:

- 23 succeed into the four dismissal issues,
   which propose refreshed hashes:
   6 keys into #656,
   15 into #657,
   2 into #658,
   and none into #659.
- 9 succeed into clusters owned by code-change issues,
   where deletion is the right action rather than refresh,
   because the code is meant to change:
   `0e41fd1057e6`,
   `40f0c1f3840b`
   and `798d10651694` into #621;
   `1682a0e0531e` and `77653aa954c8` into #653;
   `6ddb97d5cd76` and `da53cb2e3bfa` into #616;
   `8df402384c84` into #647;
   and `b9640ab1d320` into #633.

The other 23 keys have no recorded successor.
Twelve of those 23 are the i18n renderer keys in the `STRUCTURAL-IDIOM` section
(`7ec17c4edbdf`,
`0a43dc0e4435`,
`6ffeaa95d661`,
`e9510c048af2`,
`b3c3ce7ac317`,
`92ae195f9975`,
`e0af1a5083e9`,
`3350cc2028ca`,
`0a6536d03a2b`,
`329c6892e8f0`,
`f563c71baeff`,
`238136eee5b5`),
which #658 discusses as a family.
Deleting a dead key restores no visibility,
because it matches no current cluster;
keeping it records that a review happened but tells a reader the cluster is suppressed when it is not.
The choice is open and belongs to the maintainer.

## Verified workarounds

### Dismiss the reviewed cluster hash

The refreshed `BOILERPLATE-TRIVIAL` section contains:

```text
# block-body opening/closing fixer stubs, different ranges and indentation
6be7f9516c9d
```

This removes only the reviewed cluster.
Its tradeoff is intentional:
editing either body or moving either path changes the hash,
so Slopo asks for review again.
The inventory refresh replaced the path-invalidated entries and deleted stale hashes that could not be recovered.

### Recording a dismissal

An entry is a hash line preceded by comment lines.
`load_ignored` drops everything from `#` onward,
so comments are inert and only the hash reaches the tool.
The comment is nonetheless the durable half of the record,
because `.slopo.local.dir/` is gitignored by `.gitignore:11` (`*.local.*`)
and `index.md` records nothing but a timestamp.

Rules this repository follows,
each learned from a defect:

1. Write a complete sentence naming both sides by path and symbol.
   Never truncate mid-clause and never append an ellipsis.
   An entry whose reason is cut off is indistinguishable from a bare suppression.
2. Never write a cluster ordinal such as `C187` into the file.
   Ordinals come from `enumerate(clusters, 1)` at report time
   (`slopo/result/report/markdown/analyze.py:22-23`),
   so they denote a different cluster after the next regeneration.
   Ordinals belong in an issue body,
   which carries its report date.
3. Never write line numbers into the file.
   They drift on any edit to the file they point at.
4. Put each hash under the heading of the category the review recorded for that cluster,
   not the category of the issue that happens to carry it,
   and update the count in that heading once every issue in a batch is decided,
   so two issues adding to one heading cannot race.
5. Replace a dead key rather than accumulating it,
   and name in the issue which keys are replaced and by which successors.
6. Verify by hash-set diff,
   not by the logged count.
   Capture the hashes in `.slopo.local.dir/index.md`,
   run `mise run slopo:analyze`,
   and require the removed set to equal exactly the accepted hashes.
   `Ignored N previously reviewed clusters.` is an aggregate over every matching key
   and is not printed at all when the count is zero.
7. Never write a literal triple backtick into a comment that will also be pasted inside a fenced
   block in an issue body.
   It closes the fence early and breaks the rendering of everything after it.
8. Prefix every non-hash line of a proposed block with `#`,
   including trailers such as an entry count.
   `load_ignored` treats any line whose text before the first `#` is non-empty as a hash,
   so a pasted trailer becomes a junk entry that matches nothing and looks like corruption.
   The four dismissal issues were paste-tested this way on 2026-10-09:
    appending the proposed
   blocks to a copy of `slopo.ignore.txt` and calling `load_ignored` on it loaded exactly the
   proposed hashes and no other token.
   That test caught a bare `Entry count:` trailer becoming a junk key.
9. State why the pattern will not keep spreading.
   An entry must name the owner of the pattern it dismisses,
   or state that the pattern is closed,
   or state the policy that accepts the copies.
   "Too small to extract" is not a sufficient reason,
   because size today does not bound size later:
   the error-name constructor idiom in this repository was two sites when it was first written
   and is 210 sites across 146 files today,
   and every one of those copies was the nearest thing to a template when it was written.
   The four dismissal categories in this file are all claims about a pair,
   and none of them asks who owns the pattern,
   which is how twelve clusters of one 210-site idiom were recorded as twelve unrelated instances
   of trivial boilerplate.

### Measuring a family instead of a pair

Three instruments,
 all reading data slopo already writes,
 with no re-index and no embedding cost.
They exist because a cluster is a pair and duplication is a family.

1. Count a family with one query over the index.
   `code_units` holds `body`,
    `body_node_count` and `body_hash` for every indexed unit:

   ```bash
   # How many units instantiate an idiom, in how many files, and how large are they?
   sqlite3 .slopo.local.dir/index.db "select count(*), count(distinct file_id),
     min(body_node_count) from code_units where body like '%this.name = %';"

   # Which bodies are byte-identical, and how many copies of each?
   sqlite3 .slopo.local.dir/index.db "select body_hash, count(*) n from code_units
     group by body_hash having n > 1 order by n desc limit 20;"
   ```

   On 2026-10-09 the first query returned 212 units across 147 files with a minimum of exactly 13
   nodes,
    and the index held 18440 units with 690 duplicated body hashes.
2. Size what a cluster's units share.
   Tokenise each body from the report,
   take the longest common token run between pairs with `difflib.SequenceMatcher`,
   and classify the tokens that do not match as names,
   literals,
   punctuation,
   `async` and `await` markers,
   or polarity operators.
   Variance that is only names and literals is a parameterisation candidate;
   variance that is only async markers is a twin;
   variance in operators and intrinsics is a real difference.
   Applied to the 147 proposed entries this found 9 clusters with byte-identical bodies and 34
   sharing 50 or more tokens,
   and it is what showed that eight `forbidden-regex` batch wrappers share 67 of 68 tokens.
3. Count what the indexing floor hides,
    without changing it.
   `body_node_count_threshold: 13` is hand-tuned and stays.
   To measure below it,
    copy the database and index into a throwaway one:

   ```bash
   # A second config with its own db_file, report_dir and an empty ignore_file.
   slopo --config <throwaway>/slopo.conf.yaml index
   ```

   Indexing parses only and makes no embedding call,
    so it is free;
    `slopo embed` is the paid step
   and is not needed to count units or group body hashes.
   At threshold 4 the same tree yields 23671 units and 1274 duplicated body hashes against 18440
   and 690 at 13.

One trap in instrument 2.
 Do not size families by normalising identifiers and literals into a
skeleton and grouping on it.
That collapses every one-expression template return in the repository into a single group,
 which on
2026-10-09 produced a spurious 127-unit family across 40 packages out of four unrelated clusters.
Group on a targeted pattern per idiom instead,
 which is how every family count quoted in
`doc/planning/slopo-dismissed-cluster-actionability.md` was produced.

### Raise thresholds only after labeled calibration

A global threshold can be appropriate if a reviewed sample shows that an entire score band lacks useful reports.
Calibrate both thresholds separately because they run at different pipeline stages.

The tradeoff is lower recall across the repository.
Changing `similarity_threshold` can also alter multi-unit cluster membership and therefore cluster hashes because it
filters graph edges before clustering.

### Raise the body-node threshold only for globally unwanted tiny units

`body_node_count_threshold` is a better-shaped control when the unwanted population is consistently tiny functions,
not merely low-scoring functions.
The target callbacks each contain `16` body nodes.

The tradeoff is that every smaller code unit disappears regardless of semantic value.
Slopo also requires re-indexing and re-embedding after this setting changes,
as documented in `README.md:104-120`.

## What does not work

### Treating the displayed score as semantic confidence

The score measures embedding proximity plus an optional location boost.
It is not a probability that refactoring is warranted.
Its two-decimal rendering also hides the exact cutoff.

### Raising a global threshold to dispose of one reviewed cluster

The `0.97` probes suppressed substantive resolver,
CSS,
comment traversal,
and HTTP transport pairs alongside the target fixer callbacks.
This trades a local review decision for repository-wide false negatives.

### Extracting a helper solely to silence the report

A helper would accept a fixer,
a range,
and indentation merely to wrap one `replaceTextRange` call.
It would add a shallow interface while coupling opening and closing boundary policies that can evolve independently.

## Upstream filing artifact

### Upstream filing decision

1. **Is it really upstream's fault?**
    No.
   Both threshold behavior and hash-based dismissal are documented Slopo workflows.
2. **Can upstream fix it?**
    No defect was identified.
   A semantic refactoring verdict cannot be derived from cosine similarity alone.
3. **Are they supporting this use case?**
    Yes.
   `README.md:89-94` explicitly expects reviewers to dismiss non-actionable clusters by hash.
4. **Would the repository welcome a contribution?**
    No restriction was found,
   but the repository contains no `CONTRIBUTING.md`,
    issue template,
    or pull request template.
5. **Will they likely fix it?**
    Not applicable because there is no defect or missing documented mechanism.
6. **Have we prototyped a minimal fix?**
    Not applicable because constraint one fails.

No matching threshold or reranking issue or pull request was found in open or closed tracker searches on
`2026-08-16`.
The repository has no matching `.out-of-scope/` exemption.
There is nothing additive to file upstream,
so no issue or comment draft is retained.

[slopo-release]: https://github.com/rafal-qa/slopo/releases/tag/v0.4.0
