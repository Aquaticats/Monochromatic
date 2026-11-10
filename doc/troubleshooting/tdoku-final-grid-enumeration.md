# T-Doku af426180: final `list_grids` range reads past `grid.counts` and exits 139

## Symptom

T-Doku commit `af426180dc53aef89b82868e7b3fdfcf42165654`
can enumerate the first reduced Sudoku grid,
but `grid_tools list_grids` crashes after emitting a range that ends in the final reduced grid.

The final valid reduced-grid index is `3_546_146_300_287`.
The unpatched command exits from signal 11,
reported by Mise as:

```text
[probe:last] ERROR task failed
Command exited with code 139
```

This affects range enumeration through `EnumerateGrids`.
T-Doku's separate `GetGrid` path does not contain the post-callback table read described here.

## Root cause

### `ListGrids` delegates the requested range to `EnumerateGrids`

`src/grid_tools.cc:84-90` memory-maps both generated tables and passes the requested range to
`EnumerateGrids`:

```cpp
void ListGrids(uint64_t grid_id, uint64_t limit) {
    void *table = MMapFile("grid.counts");
    void *index = MMapFile("grid.index");

    EnumerateGrids(grid_id, limit, index, table, [](const char *grid) {
        printf("%.81s\n", grid);
    });
}
```

### The callback consumes the final requested grid

`src/grid_lib.cc:74-110` enters a loop while `remaining > 0`.
Its callback decrements `remaining` after emitting each requested grid:

```cpp
size_t remaining = count;
while (remaining > 0) {
    // ...
    auto skipping_callback=[&](const char *grid){
        if (to_skip > 0) {
            to_skip--;
        } else {
            callback(grid);
            remaining--;
        }
    };
```

When the requested range ends,
`remaining` becomes zero inside `TdokuEnumerate`.

### The loop body still reads the next pattern count

After `TdokuEnumerate` returns,
`src/grid_lib.cc:105-110` increments `current_pattern_idx` and reads another `uint16_t`
without checking the new `remaining` value:

```cpp
TdokuEnumerate(pattern, limit, [](const char *grid, void *thunked_callback) {
    (*static_cast<decltype(skipping_callback)*>(thunked_callback))(grid);
}, &skipping_callback);

current_pattern_idx++;
pattern_count = *(((uint16_t *) table) + current_pattern_idx);
```

For a range ending in the final pattern,
`current_pattern_idx` now points one element past `grid.counts`.
The released `grid.counts` is exactly `2_633_637_888` bytes,
which is divisible by the 4,096-byte page size used in verification.
The next `uint16_t` therefore starts in an unmapped page and the process receives `SIGSEGV`.

The earlier benchmark failure was initially suspected to come from table lookup cost.
That reading was wrong:
the terminal-index failure occurs only after the requested grid has been emitted,
at the unconditional next-pattern read.

## Verification

Verified on 2026-07-22 against:

- T-Doku commit `af426180dc53aef89b82868e7b3fdfcf42165654`.
- T-Doku `tables` release asset `tables.tar.xz`,
  published 2020-05-25 and sized `471_499_152` bytes.
- Extracted `grid.counts`,
   `2_633_637_888` bytes.
- Extracted `grid.index`,
   `20_291_214` bytes.
- Fedora 44 container capped at 4 GiB RAM and 2 CPUs.
- AVX2 build produced inside the same Fedora container image used for execution.

The container image cloned the exact commit and built `grid_tools` as follows:

```Dockerfile
FROM registry.fedoraproject.org/fedora:44
RUN dnf install --assumeyes cmake gcc-c++ git && dnf clean all
RUN git clone https://github.com/t-dillon/tdoku.git /src \
 && git -C /src checkout af426180dc53aef89b82868e7b3fdfcf42165654
RUN cmake -S /src -B /build -DCMAKE_BUILD_TYPE=Release -DAVX2=on \
 && cmake --build /build --target grid_tools --parallel 2
WORKDIR /data
ENTRYPOINT ["/build/grid_tools"]
```

The released tables were downloaded and extracted with:

```sh
gh release download tables \
  --repo t-dillon/tdoku \
  --pattern tables.tar.xz \
  --dir /tmp/tdoku-data
tar --extract --file=/tmp/tdoku-data/tables.tar.xz \
  --directory=/tmp/tdoku-data
```

### Inputs that work before the patch

The first reduced grid is emitted successfully:

```sh
podman run \
  --memory=4g \
  --cpus=2 \
  --rm \
  --volume /tmp/tdoku-data:/data:ro,Z \
  localhost/tdoku-benchmark:af426180 \
  list_grids 0 1
```

Output:

```text
123756489456189723789423156312978645564312978897645312231897564645231897978564231
```

`sample_grids 1000` also completed in every warm-cache benchmark run.
That command uses `GetGrid`,
 not `EnumerateGrids`.

### Input that fails before the patch

```sh
podman run \
  --memory=4g \
  --cpus=2 \
  --rm \
  --volume /tmp/tdoku-data:/data:ro,Z \
  localhost/tdoku-benchmark:af426180 \
  list_grids 3546146300287 1
```

Result:

```text
Command exited with code 139
```

### Inputs that work after the patch

The patched build emits both final reduced grids and exits successfully:

```sh
podman run \
  --memory=4g \
  --cpus=2 \
  --rm \
  --volume /tmp/tdoku-data:/data:ro,Z \
  localhost/tdoku-benchmark:patched \
  list_grids 3546146300286 2
```

Output:

```text
123456789456987321789321654297645138564813297831279546378564912645192873912738465
123456789456987321789321654297564138564813297831279546378645912645192873912738465
```

The patched first-index probe also emitted the same first grid as the unpatched build.

## Verified workaround

Return immediately after `TdokuEnumerate` satisfies the requested range:

```diff
diff --git a/src/grid_lib.cc b/src/grid_lib.cc
index 32c3411..b0bbf06 100644
--- a/src/grid_lib.cc
+++ b/src/grid_lib.cc
@@ -106,6 +106,7 @@ void EnumerateGrids(size_t first_grid_idx, size_t count,
             (*static_cast<decltype(skipping_callback)*>(thunked_callback))(grid);
         }, &skipping_callback);
 
+        if (remaining == 0) return;
         current_pattern_idx++;
         pattern_count = *(((uint16_t *) table) + current_pattern_idx);
     }
```

This preserves multi-pattern range enumeration because the function advances only when requested grids remain.
It avoids every unnecessary next-pattern read after a range is complete,
including the terminal out-of-bounds read.

Tradeoff:
the function now returns from inside the loop rather than letting the loop condition perform the exit.
No output or ordering semantics change.
The first-index and final-range probes verify both continuation and terminal behavior.

## What does not work

- Avoiding the final reduced-grid indices evades the crash,
  but violates the documented ability to retrieve any numbered reduced grid in range.
- Padding `grid.counts` could make this particular read land in mapped storage,
  but leaves the out-of-bounds logic intact and couples correctness to file layout.
- Running multiple Podman probes concurrently with private `:Z` volume labels caused the probes to relabel the same
  table directory for different containers.
  Verification therefore ran terminal probes sequentially.
  Shared-label `:z` is the alternative when concurrent readers are required.
- The original cell-by-cell Sudoku completion counter is unrelated to this crash.
  It remains unsuitable as the package's random-access design because it repeats completion counting per cell.

## Upstream filing artifact

### Upstream filing decision

No `.out-of-scope/` entry covers T-Doku or Sudoku enumeration.
Searches across open and closed T-Doku issues and pull requests for
`last grid list_grids`,
 `EnumerateGrids segfault`,
 and related container-crash terms found no duplicate.

1.  **Is it really upstream's fault?
    ** Yes.
    `EnumerateGrids` reads `grid.counts[current_pattern_idx + 1]` after its callback has reduced
    `remaining` to zero.
2.  **Can upstream fix it?
    ** Yes.
    The verified guard prevents the read without changing range semantics.
3.  **Are they supporting this use case?
    ** Yes.
    `src/grid_tools.cc:154-159` states that the generated tables map any numbered reduced grid to a
    configuration and solution offset,
    and the command accepts an arbitrary first grid ID and limit.
4.  **Would the repo welcome our contribution?
    ** Yes.
    The repository has no `CONTRIBUTING.md`,
     issue template,
     pull-request template,
    or AI-assistance policy.
    External pull requests
    [#1](https://github.com/t-dillon/tdoku/pull/1) and
    [#8](https://github.com/t-dillon/tdoku/pull/8) were merged.
5.  **Will they likely fix it?
    ** Yes,
     with no contrary upstream signal.
    The tracker has no rejection of terminal range support.
    The relevant path has not changed since commit
    `cc773883024762971a44dabf7c9eaab16e92b422`,
    but policy treats silence as neutral rather than a failure.
6.  **Have we prototyped a minimal compatible fix?
    ** Yes.
    The one-line guard was applied in a fresh clone of exact commit `af426180`,
    rebuilt in the Fedora 44 container,
    and verified against both the first index and the final two-grid range.

All filing constraints pass.
The draft is fileable after a human chooses to make the external communication.
It has not been posted.

### Draft issue

~~~md
Title: `grid_tools list_grids` segfaults after a range ending in the final reduced-grid pattern

Labels: `bug`

## Description

`grid_tools list_grids 3546146300287 1` emits the final reduced grid and then exits from `SIGSEGV`.
The same occurs when a requested range ends in the final pattern.

`EnumerateGrids` decrements `remaining` to zero inside its callback,
but after `TdokuEnumerate` returns it unconditionally increments `current_pattern_idx` and reads the next
`uint16_t` from `grid.counts` (`src/grid_lib.cc:105-110`).
For the final pattern that read is one element past the released count table.

## Reproduction

Tested at commit `af426180dc53aef89b82868e7b3fdfcf42165654`
with the released `tables` asset:

```sh
./build/grid_tools list_grids 3546146300287 1
echo "$?"
```

Observed exit status:

```text
139
```

Control case:

```sh
./build/grid_tools list_grids 0 1
```

This emits the first grid and exits successfully.

## Suggested fix

Return after the requested range has been emitted,
before advancing to another pattern:

```diff
         TdokuEnumerate(pattern, limit, [](const char *grid, void *thunked_callback) {
             (*static_cast<decltype(skipping_callback)*>(thunked_callback))(grid);
         }, &skipping_callback);
 
+        if (remaining == 0) return;
         current_pattern_idx++;
         pattern_count = *(((uint16_t *) table) + current_pattern_idx);
```

I rebuilt this patch and verified that:

- `list_grids 0 1` still emits the first grid.
- `list_grids 3546146300286 2` emits both final grids and exits successfully.
~~~
