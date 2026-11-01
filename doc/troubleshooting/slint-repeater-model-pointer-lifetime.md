# Slint 1.18.1 replaces repeated items when a search result model changes

## Symptom and rejected assumption

A search overlay initially copied the tree's held-pointer guard.
The native test passed with that guard present,
but also passed when the guard was removed from a disposable build.
That test therefore did not establish a need for the extra search guard.

This is distinct from the tree's demonstrated fixed-slot reuse problem.
The tree repeats stable viewport slots and changes the entries displayed by those slots.
Search repeats the result model directly and replaces that model as a whole.
The tree guard remains necessary;
its prior failing controls are not evidence that the search overlay needs the same mechanism.

## Root cause

The inspected installed dependency is `i-slint-core 1.18.1`,
corresponding to Slint v1.18.1 commit `372cf0ee5577c3dfec309a45e7b778ba4e81b734`.
The upstream checkout was also inspected;
its separate ListView prototype edits do not supply the installed-source line numbers cited here.

`package/ide/app/src/native/navigation/search/present.rs:61`
creates a new model allocation for every accepted result publication:

```rust
// package/ide/app/src/native/navigation/search/present.rs
window.set_search_entries(ModelRc::from(Rc::new(VecModel::from(rows))));
```

Installed `i-slint-core-1.18.1/model.rs:898` compares model identity by pointer:

```rust
// Slint internal/core/model.rs
impl<T> core::cmp::PartialEq for ModelRc<T> {
    fn eq(&self, other: &Self) -> bool {
        match (&self.0, &other.0) {
            (None, None) => true,
            (Some(a), Some(b)) => core::ptr::eq(
                (&**a) as *const dyn Model<Data = T> as *const u8,
                (&**b) as *const dyn Model<Data = T> as *const u8,
            ),
            _ => false,
        }
    }
}
```

Installed `i-slint-core-1.18.1/model/repeater.rs:666` resets the repeated-instance storage
when that identity changes:

```rust
// Slint internal/core/model/repeater.rs
if model.is_dirty() {
    let old_model = model.get_internal();
    let m = model.get();
    if old_model != m {
        *self.data().inner.borrow_mut() = RepeaterInner::default();
        self.data().is_dirty.set(true);
        let peer = self.project_ref().0.model_peer();
        m.model_tracker().attach_peer(peer);
    }
```

The replacement click test observes the resulting input behavior:
the old pressed row does not activate a new result when released.
The source trace establishes model-instance replacement;
the event test establishes cancellation of this particular pending click.
Neither is a guarantee about arbitrary in-place row mutation.

## Verification

`package/ide/app/src/native/search_pointer_tests.rs:24`
exercises the real `ProjectSearch` markup with native pointer events.
It uses a fixed window size and a successful initial click to validate the hit coordinates.

The successful catalog is:

- A fresh press/release on the original model activates its result.
- A fresh press/release on the replacement model activates its result.

The rejected-action catalog is:

- Press the original row,
  replace the whole model,
  render the replacement,
  and release at the same coordinates.
  No result is activated.

The first disposable run is
`/home/user/temp/agent/ide-search-guard-4l1AEc/held-result-click-removed.log`.
Cargo rebuilt the application,
nextest selected the named pointer test,
and the test still passed after replacing the custom guard with an unconditional callback.
That was an unexpected pass for the proposed guard-removal check,
not a guard success.

The subsequent controls classify this as a toolkit lifecycle regression instead:

```sh
# package/ide/app/mise.toml
mise run //package/ide/app:test:native
mise run //package/ide/app:inspect:search-guards -- "${HOME}/temp/agent/ide-tree-guard-zlIbRv/package/target"
```

The target-cache argument must resolve below the private `~/temp/agent` directory.
The guard task copies application files to another disposable directory and runs containers with
2 GiB memory,
2 CPUs,
512 processes,
4096 descriptors,
and no network.
It never mutates main-worktree source files.

`proc_f5f1` completed the controls at
`/home/user/temp/agent/ide-search-guard-Vpx5nj/results.json`.
The search model replacement case passes as a lifecycle control.
Generation,
record size,
scope containment,
EOF cancellation,
and source-focus guards each fail their named regression when removed,
then pass after restoration.

## Verified consumer simplification

The search row now calls `choose(index)` directly.
Its presenter continues to replace the entire bounded result model.
The held-pointer test remains to detect a future toolkit or publication-policy change.

This avoids carrying an ineffective custom guard.
The tradeoff is reliance on the verified whole-model lifecycle;
changing result publication to mutate rows in place requires renewed ownership analysis.
The bounded search list contains at most 50 entries and is not a windowed subset replaced during scrolling.

## What does not work

- Counting a passing test as proof of a custom guard before removing that guard.
- Treating the tree's fixed-slot repeater as equivalent to search's direct model repeater.
- Inferring protection for in-place row edits from the whole-model replacement test.
- Attributing the pointer outcome to the ListView random-seek prototype.
  The shipped toolkit is unpatched and this test uses `ScrollView` with direct repeated rows.

## Upstream filing decision

Nothing to file or add as an upstream comment.
The observed behavior is useful to the consumer and does not establish a Slint defect.
No applicable `.out-of-scope/` exemption was found.
An upstream issue search for `repeater model pointer click` returned no matches;
the [related model replacement investigation][scroll] is a separate scroll-gesture incident,
not a claim of identical input behavior.

1.  Upstream fault: not demonstrated.
    Replacement correctly prevents the tested stale click.
2.  Fixability: no corrective toolkit change is required for this consumer.
3.  Supported use: the installed repeater explicitly handles model replacement,
    and the application test exercises its actual event boundary.
4.  Contribution policy: no upstream feature or correction is being proposed.
5.  Corrective change: none is requested for the verified behavior.
6.  Prototype: the consumer guard-removal experiment is complete.
    An upstream patch is not warranted because the first condition is unmet.

[scroll]: slint-flickable-windowed-model-scroll.md
