# KWin window tabs prototype decision

Records the decision not to build a KDE/KWin window tabbing prototype after scoping
implementation and maintenance cost.
 Future sessions consult this before re-proposing
WindowTabs-style work for the current KDE environment.

This document is appended to,
 not rewritten.
 When new constraints force re-evaluation,
mark this decision superseded;
 do not delete it.

## Context

The user asked what it would take to add browser-style window tabs to KDE Plasma,
using `leafOfTree/WindowTabs` as the motivating example:
<https://github.com/leafOfTree/WindowTabs>

The current environment during the evaluation was KDE Plasma/KWin `6.6.4` on Wayland.
That matters because an external utility cannot control native Wayland windows through
Win32-style hooks,
 global input hooks,
 top-level window enumeration,
 or taskbar APIs.
A KDE implementation needs KWin cooperation.

KDE has direct prior art.
 Plasma 4.4 shipped window tabbing,
 described in KDE's
release announcement as grouping windows into tabs in the upper window border,
with middle-mouse drag onto another decoration bar or a titlebar context menu action:
<https://kde.org/announcements/4/4.4.0/plasma/>

Old KWin/X11 contained core tab-group implementation.
 The `KDE/kwin-x11` Plasma 5.0
branch had `TabGroup`,
 with behavior for adding/removing tabs,
 switching current tabs,
closing groups,
 and syncing window state:
<https://raw.githubusercontent.com/KDE/kwin-x11/Plasma/5.0/tabgroup.h>
<https://raw.githubusercontent.com/KDE/kwin-x11/Plasma/5.0/tabgroup.cpp>

Old KWin user actions exposed tab operations such as `Attach as tab to`,
 `Switch to Tab`,
`Untab`,
 and `Close Entire Group`:
<https://raw.githubusercontent.com/KDE/kwin-x11/Plasma/5.0/useractions.cpp>

Current KWin still contains config fossils for tab behavior.
 A local source search of
`/tmp/kilo/kwin-shallow` found `InactiveTabsSkipTaskbar`,
 `AutogroupSimilarWindows`,
and `AutogroupInForeground` in `src/kwin.kcfg`,
 but found no active `TabGroup` or
`tabGroup` implementation.

KDE upstream has already rejected restoring the feature in its old form.
 Bug 343690
is resolved intentional,
 with discussion around missing Plasma 5 tabbing and CSD
(client-side decoration) problems:
<https://bugs.kde.org/show_bug.cgi?id=343690>

Bug 474739 was closed as a duplicate of 343690 with the response that KDE is not bringing
this back:
<https://bugs.kde.org/show_bug.cgi?id=474739>

A KDecoration2 API attempt,
 Phabricator D3472,
 was abandoned.
 Review feedback called out
duplicated tab logic and the risk of decoration-local state diverging from core tab state:
<https://phabricator.kde.org/D3472>

A modern prototype exists:
 `Aziroshin/kwin-window-tabbing`.
 It uses a KWin script plus a
Python/PySide tab-bar helper,
 and its README says it is semi-usable for simple cases,
glitchy,
 missing features,
 and not Plasma 6-ready:
<https://github.com/Aziroshin/kwin-window-tabbing>

A related KWin script,
 `hnjae/kwin-scripts` Simple Window Groups,
 demonstrates script-level
window grouping,
 show/hide behavior,
 shortcuts,
 and titlebar menu integration,
 but it is not
persistent frame-level tabbing:
<https://github.com/hnjae/kwin-scripts>

## Decision

Do not build the KDE/KWin window tabs prototype now.

The expected value does not justify the implementation and maintenance burden for this
workspace.
 A scoped prototype is technically plausible,
 but it would compete with higher-value
work and would still leave hard problems unsolved:
 CSD behavior,
 KWin API churn,
 compositor
UI integration,
 multi-monitor scaling,
 focus/stacking edge cases,
 task manager semantics,
and user-visible failure recovery.

This is a decision to stop the current line of work,
 not a claim that KDE window tabbing is
impossible.

## Cost estimate that drove the decision

Using the user's calibration that the initial `editord` prototype took about 70 hours,
the estimate was:

- **No-UI proof spike:
  ** 10 to 20 hours.
- **Credible personal MVP:
  ** 55 to 90 hours,
   roughly 0.8x to 1.3x the `editord` prototype.
- **Daily-driver personal version:
  ** 140 to 230 hours,
   roughly 2x to 3.3x the `editord` prototype.
- **KDE-upstreamable implementation:
  ** 500 to 900+ hours,
   plus upstream design negotiation.

Closest cloned prior art (`Aziroshin/kwin-window-tabbing` plus `simple-window-groups`) measured
1,981 code LOC across 38 files with `tokei`,
 and that prior art still does not deliver a Plasma 6
daily-driver implementation.

## Maintenance estimate that drove the decision

Expected maintenance was estimated as:

- **Personal prototype pinned to one machine:
  ** 2 to 6 hours per month.
- **Personal daily-driver across Plasma updates:
  ** 4 to 12 hours per month,
   plus 8 to 24 hours per
  Plasma feature release.
- **Public extension:
  ** 8 to 20 hours per month,
   plus 20 to 60 hours per Plasma feature release.
- **Native/upstream KDE feature:
  ** a multi-person maintenance burden during active development.

KDE's Plasma 6 schedule currently targets feature releases around a four-month cadence,
 with
multiple bugfix releases after each stable release:
<https://community.kde.org/Schedules/Plasma_6>

KWin scripting API compatibility is a concrete risk.
 KDE Discuss includes a Plasma 5 to Plasma 6
KWin scripting thread where a KDE developer says Plasma 6 brought breaking changes to KWin scripts:
<https://discuss.kde.org/t/kwin-scripting-from-5-x-to-6-x-compatible/2905>

## Rejected path

### Build a personal KWin TypeScript script plus QML effect

This was the recommended architecture if the project proceeded.
The script would own tab-group state,
 shortcuts,
 titlebar menu actions,
 window activation,
geometry sync,
 minimize/detach/close behavior,
 and recovery actions.
 The QML effect would draw
compositor-owned tab UI and emit user intents back to the controller.

Rejected because the maintenance burden is still too high for the expected value.

### Build an external Qt/PySide helper

This resembles the existing `Aziroshin/kwin-window-tabbing` prototype.

Rejected because external helper windows add recurring focus,
 stacking,
 taskbar visibility,
and monitor-scaling bugs on Wayland.
 It is acceptable as a spike,
 but not as a long-term design.

### Build a native KWin/KDecoration2/Breeze implementation

This is the only path that could become a polished KDE feature.

Rejected because it front-loads KWin core work,
 decoration API work,
 Breeze integration,
task manager semantics,
 session restore,
 tests,
 CSD policy,
 and upstream consensus.
It is not appropriate as a solo prototype.

### Port `leafOfTree/WindowTabs`

Rejected because it relies on Win32-specific mechanisms:
 Win32 window enumeration and filtering,
layered popup windows,
 `SetWinEventHook`,
 Win32 geometry/restack APIs,
 DWM thumbnails,
 Windows
taskbar APIs,
 and Explorer/taskbar internals.
 Those mechanisms do not map to KDE Wayland.

## Conditions for re-evaluation

Reconsider only if at least one of these becomes true:

- KWin exposes or restores stable first-class tab-group APIs.
- KDE upstream reopens the design with an accepted CSD policy.
- A maintained Plasma 6-compatible prototype appears and survives at least two Plasma feature
  releases.
- The user explicitly decides the personal workflow value justifies 100+ hours of build and
  maintenance cost.
- A smaller adjacent problem emerges,
   such as non-tabbed application grouping or shortcut-only
  stacked groups,
   with a budget below 20 hours.

## If future work resumes

Start with a hard-gated spike,
 not a product build:

1.  Spend at most 15 hours on a KWin-script-only no-UI spike.
2.  Validate group creation,
     tab switching,
     geometry sync,
     minimize/restore,
     close cleanup,
    and panic recovery on the current Plasma Wayland session.
3.  Stop if focus,
     stacking,
     or geometry behavior is unreliable before adding visible tab UI.
4.  Add QML effect UI only after the no-UI behavior feels stable.
5.  Do not add task manager integration,
     persistence,
     or CSD polish in v1.
