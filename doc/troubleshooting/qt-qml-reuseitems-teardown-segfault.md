# Qt 6.11.1 QML SIGSEGV on teardown in QQmlDelegateModelItem::destroyObjectLater with nested async-Loader reuseItems ListViews

A QML scene built from nested `ListView`s with `reuseItems: true`,
 whose delegates are
wrapped in asynchronous `Loader`s (the shape a virtualized column-strip file manager
needs,
 see [qt-qml-listview-fast-scroll-recycling.md](qt-qml-listview-fast-scroll-recycling.md)),
crashes with SIGSEGV on application teardown.
 The run itself is fine at a steady 60 fps;
the segfault happens only on shutdown,
 inside Qt's reusable-delegate pool drain.

Resolution:
 the crash is on exit only and is accepted (or avoided with a one-line clean
exit);
 Qt 6.12 does not fix it (the change there is a pure refactor),
 and the project does
not want to own a Qt patch.
 Details below.

Platform caveat (important):
 "accept the on-exit crash" holds only on Linux,
 where the
process just exits non-zero silently.
 On macOS the same SIGSEGV is caught by the OS
CrashReporter and raises a user-visible "quit unexpectedly" dialog,
 which is unacceptable for
a shipping app.
 Verified on an Apple Silicon Mac (see Verification).
 This is a decisive reason
the file-manager project chose GTK4 over Qt;
 see
`doc/handover/file-manager-toolkit-exploration.md`.

## Symptom

- The app (or the standalone `qml` runtime) exits with SIGSEGV,
   `rc=139`,
   `core dumped`,
  during teardown at application quit,
   not during the run.
- Reproduces headless (no display),
   so it is teardown logic,
   not rendering:
  `QT_QPA_PLATFORM=offscreen qml strip-virtualization.qml` still segfaults on exit.
- Trigger shape:
   nested `ListView`s with `reuseItems: true` whose pane content is built
  by asynchronous `Loader`s.
   Without `reuseItems` there is no crash (but fast scroll
  stutters to 32 to 35 fps).
   With `reuseItems` the run holds 60 fps and then crashes on
  quit.
- Backtrace (Fedora release build,
   symbols from the system libraries):

```text
#0  QQmlDelegateModelItem::destroyObjectLater            (libQt6QmlModels.so.6)
#1  QQmlDelegateModelPrivate::destroyCacheItem
#2  QQmlReusableDelegateModelItemsPool::drain
#3  QQmlDelegateModelPrivate::~QQmlDelegateModelPrivate
#5  QQmlDelegateModel::~QQmlDelegateModel
#6  QQuickItemView::~QQuickItemView                       (libQt6Quick.so.6)
#7  QQuickListView (QQmlPrivate::QQmlElement<...>::~)
#8  QObjectPrivate::deleteChildren                        (libQt6Core.so.6)
#10 QQuickLoader (QQmlElement<...>::~)                    (a Loader being destroyed)
#13 QQuickRectangle (QQmlElement<...>::~)
#16 QQuickItem (QQmlElement<...>::~)                      (the column delegate)
#19 LoaderApplication::~LoaderApplication                 (qml runtime root)
#20 main
```

- On macOS the same teardown crashes identically,
   but instead of a silent non-zero exit the
  OS writes a CrashReporter `.ips` report and shows a "quit unexpectedly" dialog.
   Verified on
  an Apple Silicon MacBook Air (macOS 26.5.2,
   Homebrew Qt 6.11.1) running the same
  `strip-virtualization.qml`:
   exception `EXC_BAD_ACCESS` / `SIGSEGV`,
  `KERN_INVALID_ADDRESS at 0x0000000000000008` (a null-plus-offset-8 deref,
   consistent with
  the freed-object read),
   faulting-thread top frames matching the Linux trace:

```text
QQmlDelegateModelItem::destroyObjectLater            (QtQmlModels)
QQmlDelegateModelPrivate::destroyCacheItem
QQmlReusableDelegateModelItemsPool::drain
QQmlDelegateModelPrivate::~QQmlDelegateModelPrivate
QQmlDelegateModel::~QQmlDelegateModel
QQuickItemView::~QQuickItemView                       (QtQuick)
```

  The QML log also prints the teardown-race warnings just before the crash:
   `Object or
  context destroyed during incubation` and `There are still "43" items in the process of
  being created at engine destruction.`

## Root cause

The recursive teardown reaches an inner `ListView`,
 whose delegate model drains its
reuse pool,
 and the pool drain dereferences an already-freed delegate object.
 Walking
the chain in `qtdeclarative` v6.11.1 (`src/qmlmodels/qqmldelegatemodel.cpp`):

The delegate-model destructor unconditionally drains the reuse pool:

```cpp
// qqmldelegatemodel.cpp:182
QQmlDelegateModelPrivate::~QQmlDelegateModelPrivate()
{
    qDeleteAll(m_finishedIncubating);

    // Free up all items in the pool
    drainReusableItemsPool(0);
}
```

`drainReusableItemsPool(0)` releases every pooled item through `destroyCacheItem`:

```cpp
// qqmldelegatemodel.cpp:1142
void QQmlDelegateModelPrivate::drainReusableItemsPool(int maxPoolTime)
{
    m_reusableItemsPool.drain(maxPoolTime, [this](QQmlDelegateModelItem *cacheItem){ destroyCacheItem(cacheItem); });
}
```

`destroyCacheItem` calls `destroyObjectLater` on any item that still has an object:

```cpp
// qqmldelegatemodel.cpp:637
void QQmlDelegateModelPrivate::destroyCacheItem(QQmlDelegateModelItem *cacheItem)
{
    if (QObject *object = cacheItem->object()) {
        emitDestroyingItem(object);
        cacheItem->destroyObjectLater();
    }
    ...
```

`destroyObjectLater` reads the object's `QQmlData` and tears down its context:

```cpp
// qqmldelegatemodel.cpp:2574
void QQmlDelegateModelItem::destroyObjectLater()
{
    Q_ASSERT(m_object);
    Q_ASSERT(m_contextData);

    QQmlData *data = QQmlData::get(m_object);
    Q_ASSERT(data);
    if (data->ownContext) {
        data->ownContext->clearContext();
        data->ownContext->deepClearContextObject(m_object);   // <- fault
        data->ownContext.reset();
        data->context = nullptr;
    }
    ...
```

In a release build the `Q_ASSERT`s are compiled out.
 When the outer column delegate (an
async-`Loader`-wrapped `Item`) is destroyed by `QObject::deleteChildren` during the
recursive app-exit teardown,
 the inner `ListView`'s reuse pool still holds pooled
delegate items whose backing object and context have already been freed by that same
recursive destruction.
 The pool drain then runs `QQmlData::get(m_object)` and
`ownContext->...` over freed memory,
 giving the use-after-free SIGSEGV.
 The function
already carries a `QTBUG-87228` comment about app-exit destruction,
 so this exact path
has prior teardown-bug history.

Refuted hypotheses (do not re-derive these):

- "It is `reuseItems` alone."
   Disproven:
   a single `ListView` with `reuseItems` and a
  100000-row model (`min1.qml`) tears down cleanly,
   `rc=0`.
- "It is any 2-level nesting."
   Disproven:
   a horizontal `ListView` of columns,
   each a
  vertical `ListView` (`min2.qml`,
   `reuseItems` on both),
   also tears down cleanly,
  `rc=0`.
   The crash needs the deeper structure with async `Loader`-wrapped nested
  ListViews,
   as in the full harness.

## Verification

- Version under test:
   `qt6-qtdeclarative-6.11.1-2.fc44`.
   Source cross-checked against
  `qt/qtdeclarative` tag `v6.11.1` (`qqmldelegatemodel.cpp` blob
  `52d9db5da0c5db0207f40bf4b3b6b1310d7cdc20`).
- Harness:
   `package/file-manager/variant-qt/bench/strip-virtualization.qml`.
  Headless repro:

```sh
QT_QPA_PLATFORM=offscreen QT_FORCE_STDERR_LOGGING=1 \
  QT_LOGGING_RULES="*.debug=false;qml.debug=true" \
  /usr/lib64/qt6/bin/qml strip-virtualization.qml   # -> Segmentation fault (rc 139) on exit
```

- Crash-threshold catalog (headless,
   `QT_QPA_PLATFORM=offscreen`,
   deterministic over 5
  runs each;
   repro files in `package/file-manager/variant-qt/bench/`):
  - Clean,
     `rc=0`:
     1-level single `ListView` + `reuseItems` + 100000 rows (`min1.qml`).
  - Clean,
     `rc=0`:
     2-level plain columns-of-row-lists + `reuseItems` (`min2.qml`).
  - SIGSEGV,
     `rc=139`:
     2-level,
     mixed column types (row `ListView` vs preview,
     by
    visibility,
     no `Loader`) (`min2img.qml`).
  - SIGSEGV,
     `rc=139`:
     2-level with the inner `ListView` in a `Loader`,
     both
    `asynchronous: true` (`min2loader.qml`) and `asynchronous: false`
    (`min2syncloader.qml`).
  - SIGSEGV,
     `rc=139`:
     3-level plain columns to panes to rows,
     no `Loader`
    (`min3plain.qml`).
  - SIGSEGV,
     `rc=139`:
     the full harness (3-level with async `Loader`s).
  - Clean,
     `rc=0`,
     with `reuseItems: false`:
     no crash,
     but fast scroll drops to 32 to 35 fps.
- Trigger summary:
   the crash fires from any `Loader` (sync or async) wrapping a nested
  `reuseItems` `ListView`,
   from three or more nested levels,
   or even from a mixed 2-level
  layout.
   Only the simplest uniform 1- and 2-level cases are clean,
   and that boundary is
  fragile (the near-identical `min2img.qml` already crashes),
   so there is no robust
  structural way to keep `reuseItems` and avoid the crash.
- macOS reproduction (Apple Silicon MacBook Air,
   macOS 26.5.2,
   Homebrew `qt` 6.11.1):

```sh
# with imgs/ present next to the qml file
/opt/homebrew/opt/qt/bin/qml strip-virtualization.qml   # window renders, then crashes on Qt.quit()
```

  Confirms the same use-after-free on a different OS and a different windowing/graphics stack
  (Cocoa QPA,
   Metal RHI,
   not Wayland/OpenGL),
   so the bug is in the delegate-model teardown
  logic,
   not platform integration.
   Unlike Linux,
   macOS surfaces it as a CrashReporter
  `.ips` report in `~/Library/Logs/DiagnosticReports/` and a "quit unexpectedly" dialog.
   Read
  the report's exception and faulting-thread frames with:

```sh
F=~/Library/Logs/DiagnosticReports/qml-*.ips
python3 -c 'import json,sys; d=json.loads(open(sys.argv[1]).read().split(chr(10),1)[1]); \
print(d["exception"]); ft=d["faultingThread"]; \
print(chr(10).join(d["threads"][ft]["frames"][i].get("symbol","?") for i in range(6)))' "$F"
```

## Qt 6.12 does not fix it: the crash-site change is a pure refactor

The crash-site code changed between v6.11.1 and 6.12,
 but the change is behaviorally
identical,
 so it is not a fix.
 In `destroyObjectLater`,
 v6.11.1 calls

```cpp
data->ownContext->clearContext();
```

and 6.12 (tag `v6.12.0-beta1`,
 and `dev`) calls

```cpp
data->ownContext->emitDestruction();
data->ownContext->clearExpressions();
```

Reading the `QQmlContextData` implementations shows these are the same operation.
v6.11.1's `clearContext()` is exactly `emitDestruction()` followed by the
expression-clearing loop:

```cpp
// 6.11.1 src/qml/qml/qqmlcontextdata.cpp
void QQmlContextData::clearContext()
{
    emitDestruction();
    QQmlJavaScriptExpression *expression = m_expressions;
    while (expression) {
        // ... setContext(nullptr) ...
    }
    m_expressions = nullptr;
}
```

6.12 merely split that body into two named methods,
 `emitDestruction()` and
`clearExpressions()` (the "Split ~QQmlContextData() into smaller parts" refactor),
 and
`destroyObjectLater` calls them separately.
 6.12's `clearExpressions()` is byte-identical
to the loop inside 6.11.1's `clearContext()`,
 and `destroyCacheItem` and
`~QQmlDelegateModelPrivate` are byte-identical between the two versions.
 The
use-after-free root cause (the pool holds delegate items whose `m_object` is already
freed during nested teardown) is untouched.

Conclusion:
 updating to Qt 6.12 does not fix this crash.
 The reading of the crash-site
implementations is conclusive,
 so the multi-hour 6.12 build was not run.
 If a later `dev`
change alters the pool or teardown ownership (rather than just this refactor),
re-evaluate against that commit.

## Resolution: accept the on-exit crash, or exit clean without owning a Qt patch

The crash is purely on teardown,
 after the event loop exits;
 the running app is stable at
60 fps.
 Qt 6.12 does not fix it and a forked or backported Qt is more surface to own over
time,
 so the options are to accept the crash (Linux only) or force a clean exit.

- Accept the crash on exit (Linux only).
   The process is already shutting down,
   and on Linux
  a SIGSEGV during teardown is just a non-zero exit code with no user-facing artifact.
   Save
  any persistent state (window geometry,
   settings) explicitly before quit,
   not in a Qt/QML
  teardown handler,
   so nothing depends on the teardown completing.
   This does NOT hold on
  macOS:
   the OS CrashReporter turns the same crash into a "quit unexpectedly" dialog,
   so
  accepting it is not viable for a shipping macOS app.
- Clean exit without a Qt patch (mandatory on macOS if Qt is used at all).
   After
  `app.exec()` returns,
   terminate before the QML engine is destroyed so Qt's teardown never
  runs.
   In the Rust `main`,
   `std::process::exit(0)` after `exec()` skips the Drop of the
  `QGuiApplication` and `QQmlApplicationEngine` locals,
   giving `rc=0` on Linux.
   Flush
  anything that must persist (logs,
   settings) first,
   because destructors are skipped.
   Caveats
  not yet verified on macOS:
   that `exit()`/`_exit()` actually suppresses the CrashReporter
  dialog (static C++ destructors run by `exit()` could still hit the freed object,
   so
  `libc::_exit` may be needed),
   and that every exit path (window close,
   Cmd-Q,
   dock quit,
  signal) is routed through it,
   since a single missed path brings the dialog back.
- For the file-manager project specifically,
   the resolution was to not use Qt:
   GTK4 has clean
  teardown (`rc 0`) on every platform with no such hazard,
   so the toolkit choice is GTK4.
  See `doc/handover/file-manager-toolkit-exploration.md`.
   The clean-exit workaround above
  remains documented for anyone who must ship Qt with nested `reuseItems`.

Confirmed not reliable:

- Staying at two plain levels:
   `min2.qml` is clean but the near-identical `min2img.qml`
  crashes,
   so the boundary is too fragile to depend on.
- `reuseItems: false`:
   removes the crash but drops fast scroll to 32 to 35 fps.
- Updating to Qt 6.12:
   pure refactor,
   no behavior change (see above).
- Forking or backporting Qt:
   rejected because of the ongoing ownership.

## What does not work

- `reuseItems: true` is required for 60 fps fast scroll (without it,
   32 to 35 fps) but
  is exactly what triggers the teardown crash in the nested async-`Loader` structure.
- A larger `cacheBuffer` does not help:
   it made the fast-scroll warmup dip worse (17 fps)
  and did not prevent the teardown crash.

## Upstream filing decision

`.out-of-scope/` checked:
 no Qt or QML exemption,
 so upstream tracking would be in scope.

Duplicate search (`bugreports.qt.io`,
 web and tracker):
 the delegate-model and ListView
teardown-crash family is well-reported,
 for example QTBUG-29727 ("ListView contains all
items as children even destructed ones",
 SIGSEGV),
 QTBUG-58255 (SIGSEGV in
`QQmlDelegateModelPrivate::object`),
 QTBUG-50992 ("Object destroyed during incubation"),
and QTBUG-65225.
 None is the exact nested-`reuseItems`-on-teardown variant reproduced
here,
 but the class is clearly known upstream.

Six-constraint check:

1. Upstream's fault:
    yes,
    entirely inside `QQmlReusableDelegateModelItemsPool::drain`;
    no
   app code is in the trace.
2. Can upstream fix it:
    yes,
    it is a teardown-ownership fix,
    not an algebraic-core limit.
3. Supporting the use case:
    `reuseItems` is a documented,
    supported feature,
    and nesting
   is not documented as unsupported.
4. Would they welcome it:
    Qt takes reports at `bugreports.qt.io` and patches via Gerrit;
   not investigated in depth because filing is not being pursued.
5. Will they fix it:
    the family has open reports and no won't-fix signal.
6. Minimal fix prototype:
    not done.
    The real fix is in Qt's pool-teardown ownership;
   prototyping it needs a qtdeclarative build,
    a local backport was explicitly rejected
   (ownership),
    and 6.12 does not fix it,
    so there is no minimal patch to attach.

Decision:
 do not file a new issue now.
 The crash is accepted on exit,
 the bug family is
already well-reported upstream,
 and the project does not want to own a fix.
 This is
recorded rather than filed,
 per the do-not-file-by-default policy.
 If that changes,
 the
minimal reproductions (`min2img.qml`,
 `min2loader.qml`) plus the root-cause trace above
are ready to attach to a new report,
 or as a comment on the closest existing issue if one
proves to be the same bug.

## Related

- [qt-qml-listview-fast-scroll-recycling.md](qt-qml-listview-fast-scroll-recycling.md):
  why `reuseItems` is required (60 fps vs 11 fps),
   and the benchmark harness.
