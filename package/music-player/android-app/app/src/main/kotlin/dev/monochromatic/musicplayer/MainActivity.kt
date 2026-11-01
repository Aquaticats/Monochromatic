// ============================================================================
// File summary (folds in the old KDoc that sat on the activity and composables)
// ============================================================================
//
// This file is the UI. It has one Android `Activity` (`MainActivity`) plus a
// set of Jetpack Compose UI functions (the `@Composable`-annotated `fun`s).
//
// What Compose is, for a TS reader: a declarative UI toolkit much like React.
// A `@Composable` function is a component; calling it "emits" UI. State is held
// in `remember { mutableStateOf(...) }` (think `useState`), side effects run in
// `LaunchedEffect(key) { ... }` (think `useEffect` with a dependency key), and
// layout is expressed by nesting composables with TRAILING LAMBDAS as their
// "children" (think JSX nesting). `Modifier` chains are the styling/layout
// props. There is no JSX; children are the `{ ... }` block after the call.
//
// The activity itself (`MainActivity`):
//   - The player lives in `PlaybackService` so audio outlives this activity, so
//     the UI BINDS to that service for a direct handle to the service-owned
//     `PlayerController` and drives it (single process, one brain). Binding with
//     `BIND_AUTO_CREATE` also creates the service, which builds the
//     `MediaSession` and goes foreground on play.
//   - A folder picker is registered on the activity (not inside the
//     composition) so it survives the screen leaving composition while the
//     picker round-trip stops the activity.
//
// The composables, folded from their old KDocs:
//   - `appRoot`: the first-run decision + library trigger over a bound
//     controller. Requests audio access once when no folder is held, signals the
//     service to load once a source can be read, and shows `playerScreen`, which
//     draws the first-run pane over the player when the open source needs it.
//   - `playerScreen`: the desktop's narrow layout with responsive progress/transport
//     and volume/end-of-track pairs, source actions, settings, and tracks.
//     Each playback pair shares one line when it fits and wraps as a complete group when
//     the window narrows.
//     Tap a track to play; tap the playing track to pause/resume.
//   - `startingGate`/`loadingNotice`: small placeholder/notice
//     screens. `seekRow`/`volumeRow`/`sourceActionRow`/
//     `settingsPage`/`trackPager`/`trackRow`: pieces of the player screen.
//   - `formatTime`: format a seconds value as `m:ss`.
// ============================================================================

// What:     `package dev.monochromatic.musicplayer` names the namespace everything in
//           this file lives in (the activity is referenced from the Android manifest by
//           `dev.monochromatic.musicplayer.MainActivity`).
// Why:      So the manifest and sibling files can refer to these declarations.
//
// In TS you'd write (pseudocode):
// ```ts
// // No `package` line in TS; the file path is the module identity.
// ```
package dev.monochromatic.musicplayer

// What:     `import android.content.ComponentName` pulls in `ComponentName`, an
//           identifier for an Android component (here a service). The service-connection
//           callbacks receive one.
// Why:      `onServiceConnected`/`onServiceDisconnected` take a `ComponentName?`.
//
// In TS you'd write (pseudocode):
// ```ts
// import { ComponentName } from "android/content";
// ```
import android.content.ComponentName

// What:     `import android.content.Context` pulls in `Context`, Android's app
//           environment handle. We use its `BIND_AUTO_CREATE` flag.
// Why:      `bindService(...)` passes `Context.BIND_AUTO_CREATE`.
//
// In TS you'd write (pseudocode):
// ```ts
// import { Context } from "android/content";
// ```
import android.content.Context

// What:     `import android.content.Intent` pulls in `Intent`, Android's "what to do"
//           message object used to start/bind components.
// Why:      We build an `Intent` to bind the service and use its grant flag.
//
// In TS you'd write (pseudocode):
// ```ts
// import { Intent } from "android/content";
// ```
import android.content.Intent

// What:     `import android.content.ServiceConnection` pulls in `ServiceConnection`, the
//           INTERFACE with `onServiceConnected`/`onServiceDisconnected` callbacks for a
//           bound service.
// Why:      `connection` is an anonymous object implementing this interface.
//
// In TS you'd write (pseudocode):
// ```ts
// import { ServiceConnection } from "android/content";
// ```
import android.content.ServiceConnection

// What:     `import android.net.Uri` pulls in `Uri`, Android's parsed URI type.
// Why:      The folder picker yields a tree `Uri`; `pendingRoot` holds one.
//
// In TS you'd write (pseudocode):
// ```ts
// import { Uri } from "android/net";
// ```
import android.net.Uri

// What:     `import android.os.Bundle` pulls in `Bundle`, Android's key/value state bag
//           passed to `onCreate` (for saved instance state).
// Why:      `onCreate(savedInstanceState: Bundle?)` takes one.
//
// In TS you'd write (pseudocode):
// ```ts
// import { Bundle } from "android/os";
// ```
import android.os.Bundle

// What:     `import android.os.IBinder` pulls in `IBinder`, the interface a bound
//           service hands back; `onServiceConnected` receives one to cast.
// Why:      `onServiceConnected(..., service: IBinder?)` takes an `IBinder?`.
//
// In TS you'd write (pseudocode):
// ```ts
// import { IBinder } from "android/os";
// ```
import android.os.IBinder

// What:     `import android.util.Log` pulls in `Log`, Android's logger (`Log.i`).
// Why:      We log lifecycle and interaction events.
//
// In TS you'd write (pseudocode):
// ```ts
// import { Log } from "android/util";
// ```
import android.util.Log

// What:     `import androidx.activity.ComponentActivity` pulls in `ComponentActivity`,
//           the modern Android base `Activity` class (Compose-friendly).
// Why:      `MainActivity` EXTENDS `ComponentActivity`.
//
// In TS you'd write (pseudocode):
// ```ts
// import { ComponentActivity } from "androidx/activity";
// ```
import androidx.activity.ComponentActivity

// What:     `import androidx.activity.compose.rememberLauncherForActivityResult` pulls in
//           the Compose helper that registers an activity-result launcher INSIDE a
//           composition (the permission prompt below uses it).
// Why:      `appRoot` requests the audio permission via this launcher.
//
// In TS you'd write (pseudocode):
// ```ts
// import { rememberLauncherForActivityResult } from "androidx/activity/compose";
// ```
import androidx.activity.compose.rememberLauncherForActivityResult

// What:     `import androidx.activity.compose.setContent` pulls in `setContent`, the
//           bridge that sets an activity's UI to a Compose tree.
// Why:      `onCreate` calls `setContent { ... }` to mount the UI.
//
// In TS you'd write (pseudocode):
// ```ts
// import { setContent } from "androidx/activity/compose";
// ```
import androidx.activity.compose.setContent

// What:     `import androidx.activity.enableEdgeToEdge` pulls in `enableEdgeToEdge()`,
//           which draws the app behind the system bars.
// Why:      `onCreate` calls it for the edge-to-edge layout.
//
// In TS you'd write (pseudocode):
// ```ts
// import { enableEdgeToEdge } from "androidx/activity";
// ```
import androidx.activity.enableEdgeToEdge

// What:     `import androidx.activity.result.contract.ActivityResultContracts` pulls in
//           `ActivityResultContracts`, a namespace of standard result CONTRACTS
//           (`OpenDocumentTree`, `RequestPermission`) that describe a launch+result pair.
// Why:      The folder picker and the permission request each use one of these contracts.
//
// In TS you'd write (pseudocode):
// ```ts
// import { ActivityResultContracts } from "androidx/activity/result/contract";
// ```
import androidx.activity.result.contract.ActivityResultContracts

// What:     Horizontal scrolling and its remembered state keep oversized groups reachable.
// Why:      Long page names and enlarged system fonts may exceed the screen width.
//
// In TS you'd write (pseudocode):
// ```ts
// import { horizontalScroll, rememberScrollState } from "compose/foundation";
// ```
import androidx.compose.foundation.horizontalScroll
import androidx.compose.foundation.rememberScrollState

// What:     `import androidx.compose.foundation.isSystemInDarkTheme` pulls in
//           `isSystemInDarkTheme()`, a composable that returns whether the device is in
//           dark mode.
// Why:      `onCreate` picks the dark/light color scheme from it.
//
// In TS you'd write (pseudocode):
// ```ts
// import { isSystemInDarkTheme } from "androidx/compose/foundation";
// ```
import androidx.compose.foundation.isSystemInDarkTheme

// What:     `import androidx.compose.foundation.layout.Arrangement` pulls in
//           `Arrangement`, the namespace describing how children are spaced inside a
//           `Row`/`Column` (e.g. `Arrangement.spacedBy(8.dp)`).
// Why:      The layouts space their children with `Arrangement.spacedBy(...)`.
//
// In TS you'd write (pseudocode):
// ```ts
// import { Arrangement } from "androidx/compose/foundation/layout";
// ```
import androidx.compose.foundation.layout.Arrangement

// What:     `import androidx.compose.foundation.layout.Box` pulls in a stacking container.
// Why:      The MD1 page tab uses a thin Box as its selected underline.
//
// In TS you'd write (pseudocode):
// ```ts
// import { Box } from "androidx/compose/foundation/layout";
// ```
import androidx.compose.foundation.layout.Box

// What:     `import androidx.compose.foundation.layout.Column` pulls in `Column`, the
//           vertical layout composable (stacks children top to bottom).
// Why:      Several screens lay their content out in a `Column`.
//
// In TS you'd write (pseudocode):
// ```ts
// import { Column } from "androidx/compose/foundation/layout";
// ```
import androidx.compose.foundation.layout.Column

// What:     `import androidx.compose.foundation.layout.Row` pulls in `Row`, the horizontal
//           layout composable (places children left to right).
// Why:      Many UI pieces lay out horizontally in a `Row`.
//
// In TS you'd write (pseudocode):
// ```ts
// import { Row } from "androidx/compose/foundation/layout";
// ```
import androidx.compose.foundation.layout.Row

// What:     `import androidx.compose.foundation.layout.FlowRowScope` names the receiver shared by
//           children emitted into a wrapping `FlowRow`; extensions assign per-line weights.
// Why:      Playback groups share a line when they fit and move to the next line when they do not.
//
// In TS you'd write (pseudocode):
// ```ts
// type FlowRowScope = { weight: (share: number, fill?: boolean) => LayoutModifier };
// ```
import androidx.compose.foundation.layout.FlowRowScope

// What:     `import androidx.compose.foundation.layout.fillMaxSize` pulls in the
//           `fillMaxSize` MODIFIER (make a composable occupy all available width AND
//           height).
// Why:      The full-screen surfaces/columns use `Modifier.fillMaxSize()`.
//
// In TS you'd write (pseudocode):
// ```ts
// import { fillMaxSize } from "androidx/compose/foundation/layout";
// ```
import androidx.compose.foundation.layout.fillMaxSize

// What:     `import androidx.compose.foundation.layout.padding` pulls in the `padding`
//           MODIFIER (inner spacing around a composable).
// Why:      Most layouts pad their contents with `Modifier.padding(...)`.
//
// In TS you'd write (pseudocode):
// ```ts
// import { padding } from "androidx/compose/foundation/layout";
// ```
import androidx.compose.foundation.layout.padding

// What:     `import androidx.compose.foundation.layout.size` pulls in the `size` MODIFIER
//           (fix a composable's width and height).
// Why:      The loading spinner uses `Modifier.size(20.dp)`.
//
// In TS you'd write (pseudocode):
// ```ts
// import { size } from "androidx/compose/foundation/layout";
// ```
import androidx.compose.foundation.layout.size

// What:     `import androidx.compose.foundation.layout.WindowInsets` describes the insets the screen pads for.
// Why:      The Scaffold passes no insets to the production content, which pads its own status and navigation bars.
//
// In TS you'd write (pseudocode):
// ```ts
// import { WindowInsets } from "androidx/compose/foundation/layout";
// ```
import androidx.compose.foundation.layout.WindowInsets

// What:     `import androidx.compose.foundation.layout.windowInsetsPadding` pads a node by the given insets.
// Why:      The Settings page pads itself by the safe drawing insets.
//
// In TS you'd write (pseudocode):
// ```ts
// import { windowInsetsPadding } from "androidx/compose/foundation/layout";
// ```
import androidx.compose.foundation.layout.windowInsetsPadding

// What:     `import androidx.compose.foundation.lazy.LazyColumn` pulls in `LazyColumn`, a
//           SCROLLING column that only composes visible items (like a virtualized list).
// Why:      `trackPager` shows the tabs + tracks in one scrolling `LazyColumn`.
//
// In TS you'd write (pseudocode):
// ```ts
// import { LazyColumn } from "androidx/compose/foundation/lazy";
// ```
import androidx.compose.foundation.lazy.LazyColumn

// What:     `import androidx.compose.foundation.lazy.items` pulls in `items(list) { ... }`,
//           the `LazyListScope` builder that emits one row per list element.
// Why:      `trackPager` lists the page's tracks with `items(...)`.
//
// In TS you'd write (pseudocode):
// ```ts
// import { items } from "androidx/compose/foundation/lazy";
// ```
import androidx.compose.foundation.lazy.items

// What:     Material3 ButtonGroup and its defaults provide grouped transport actions.
// Why:      Prev, Play/Pause, and Next need Material's overflow-aware standard group.
//
// In TS you'd write (pseudocode):
// ```ts
// import { ButtonGroup, ButtonGroupDefaults } from "compose/material3";
// ```
import androidx.compose.material3.ButtonGroup
import androidx.compose.material3.ButtonGroupDefaults

// What:     DropdownMenuItem renders an action moved into ButtonGroup overflow.
// Why:      Every transport command remains reachable at narrow widths and large fonts.
//
// In TS you'd write (pseudocode):
// ```ts
// import { DropdownMenuItem } from "compose/material3";
// ```
import androidx.compose.material3.DropdownMenuItem

// What:     This annotation opts into segmented-button APIs in this library release.
// Why:      The compiler requires explicit acknowledgement at the call site.
//
// In TS you'd write (pseudocode):
// ```ts
// // No TypeScript equivalent.
// ```
import androidx.compose.material3.ExperimentalMaterial3Api

// What:     These Material primitives render one joined single-select group.
// Why:      Exactly one playback mode appears selected in the required order.
//
// In TS you'd write (pseudocode):
// ```ts
// import { SegmentedButton, SegmentedButtonDefaults, SingleChoiceSegmentedButtonRow } from "compose/material3";
// ```
import androidx.compose.material3.SegmentedButton
import androidx.compose.material3.SegmentedButtonDefaults
import androidx.compose.material3.SingleChoiceSegmentedButtonRow

// What:     `import androidx.compose.material3.CircularProgressIndicator` pulls in the
//           spinner composable.
// Why:      `loadingNotice` shows a `CircularProgressIndicator`.
//
// In TS you'd write (pseudocode):
// ```ts
// import { CircularProgressIndicator } from "androidx/compose/material3";
// ```
import androidx.compose.material3.CircularProgressIndicator

// What:     `import androidx.compose.material3.Scaffold` pulls in `Scaffold`, a Material
//           layout shell that supplies system-bar inset padding to its content lambda.
// Why:      `playerScreen` wraps its content in a `Scaffold`.
//
// In TS you'd write (pseudocode):
// ```ts
// import { Scaffold } from "androidx/compose/material3";
// ```
import androidx.compose.material3.Scaffold

// What:     `import androidx.compose.material3.Slider` pulls in the `Slider` composable (a
//           draggable value track).
// Why:      The seek bar and volume control are `Slider`s.
//
// In TS you'd write (pseudocode):
// ```ts
// import { Slider } from "androidx/compose/material3";
// ```
import androidx.compose.material3.Slider

// What:     `import androidx.compose.material3.Surface` pulls in `Surface`, a themed
//           background container.
// Why:      `onCreate` wraps the screen in a full-size `Surface`.
//
// In TS you'd write (pseudocode):
// ```ts
// import { Surface } from "androidx/compose/material3";
// ```
import androidx.compose.material3.Surface

// What:     `import androidx.compose.material3.Text` pulls in the `Text` composable
//           (renders a string).
// Why:      Every label/row uses `Text`.
//
// In TS you'd write (pseudocode):
// ```ts
// import { Text } from "androidx/compose/material3";
// ```
import androidx.compose.material3.Text

// What:     `import androidx.compose.runtime.Composable` pulls in the `@Composable`
//           ANNOTATION that marks a function as a Compose UI component.
// Why:      Every UI function below is annotated `@Composable`.
//
// In TS you'd write (pseudocode):
// ```ts
// import { Composable } from "androidx/compose/runtime";
// ```
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import dev.monochromatic.musicplayer.core.folderGrantAttempts
import dev.monochromatic.musicplayer.core.libraryDisplayName

// What:     `import androidx.compose.runtime.LaunchedEffect` pulls in `LaunchedEffect`,
//           the composable that runs a `suspend` side-effect block tied to a key (re-runs
//           when the key changes; cancels on leave).
// Why:      `appRoot` and `playerScreen` use `LaunchedEffect` for permission requests and
//           position polling.
//
// In TS you'd write (pseudocode):
// ```ts
// import { LaunchedEffect } from "androidx/compose/runtime";
// ```
import androidx.compose.runtime.LaunchedEffect

// What:     `import androidx.compose.runtime.DisposableEffect` runs the resume observer for the screen's lifetime.
// Why:      The observer is added when the screen appears and removed when it leaves, so no stale callback remains.
//
// In TS you'd write (pseudocode):
// ```ts
// import { useEffect } from "react";
// ```
import androidx.compose.runtime.DisposableEffect

// What:     `import androidx.lifecycle.Lifecycle` names the resume event that rechecks the audio permission.
// Why:      Android's settings page can grant access while the app is in the background, so the screen rechecks on
//           resume.
//
// In TS you'd write (pseudocode):
// ```ts
// import type { Lifecycle } from "androidx/lifecycle";
// ```
import androidx.lifecycle.lifecycleScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import androidx.lifecycle.Lifecycle

// What:     `import androidx.lifecycle.LifecycleEventObserver` is the observer type that receives lifecycle events.
// Why:      The observer is registered on the activity's lifecycle to recheck access on resume.
//
// In TS you'd write (pseudocode):
// ```ts
// import type { LifecycleEventObserver } from "androidx/lifecycle";
// ```
import androidx.lifecycle.LifecycleEventObserver

// What:     `import androidx.compose.runtime.getValue` imports the `getValue` OPERATOR
//           used to READ a `by`-delegated state property (the `var x by remember { ... }`
//           lines).
// Why:      The `by` state declarations below read through it.
//
// In TS you'd write (pseudocode):
// ```ts
// // no import — TS getters don't need an operator function in scope
// ```
import androidx.compose.runtime.getValue

// What:     `import androidx.compose.runtime.mutableDoubleStateOf` imports
//           `mutableDoubleStateOf(x)`, a `Double`-specialized observable state holder (a
//           `mutableStateOf` variant that avoids boxing the number).
// Why:      `playerScreen` holds `position`/`duration` (both `Double`) in it.
//
// In TS you'd write (pseudocode):
// ```ts
// import { mutableDoubleStateOf } from "androidx/compose/runtime";
// ```
import androidx.compose.runtime.mutableDoubleStateOf

// What:     `import androidx.compose.runtime.mutableStateOf` imports `mutableStateOf(x)`,
//           the general observable state holder.
// Why:      `boundController` and `appRoot`'s `hasAudioAccess` use it.
//
// In TS you'd write (pseudocode):
// ```ts
// import { mutableStateOf } from "androidx/compose/runtime";
// ```
import androidx.compose.runtime.mutableStateOf

// What:     `import androidx.compose.runtime.remember` imports `remember { ... }`, the
//           composable that COMPUTES a value once and keeps it across recompositions
//           (until its keys change).
// Why:      State holders are wrapped in `remember { ... }` so they survive recomposition.
//
// In TS you'd write (pseudocode):
// ```ts
// import { remember } from "androidx/compose/runtime";
// ```
import androidx.compose.runtime.remember

// What:     `rememberSaveable` retains simple Compose state through activity recreation.
// Why:      Portrait expansion survives device rotation during one running app.
//
// In TS you'd write (pseudocode):
// ```ts
// import { useRestorableState } from "compose/runtime/saveable";
// ```
import androidx.compose.runtime.saveable.rememberSaveable

// What:     `import androidx.compose.runtime.setValue` imports the `setValue` OPERATOR
//           used to WRITE a `by`-delegated state property.
// Why:      Assigning to the `by` state vars below goes through it.
//
// In TS you'd write (pseudocode):
// ```ts
// // no import — TS setters don't need an operator function in scope
// ```
import androidx.compose.runtime.setValue

// What:     `import androidx.compose.ui.Alignment` pulls in `Alignment`, the namespace of
//           alignment constants (`CenterVertically`, `CenterHorizontally`).
// Why:      Layouts align children with `Alignment.*`.
//
// In TS you'd write (pseudocode):
// ```ts
// import { Alignment } from "androidx/compose/ui";
// ```
import androidx.compose.ui.Alignment

// What:     `import androidx.compose.ui.Modifier` pulls in `Modifier`, the chainable
//           styling/layout descriptor passed to composables.
// Why:      Nearly every composable takes a `Modifier` chain.
//
// In TS you'd write (pseudocode):
// ```ts
// import { Modifier } from "androidx/compose/ui";
// ```
import androidx.compose.ui.Modifier

// What:     `import androidx.compose.ui.geometry.Offset` names a two-dimensional pixel offset.
// Why:      Active label light uses a centered text-shadow glow.
//
// In TS you'd write (pseudocode):
// ```ts
// import type { Offset } from "androidx/compose/ui/geometry";
// ```
import androidx.compose.ui.geometry.Offset

// What:     `import androidx.compose.ui.platform.LocalContext` pulls in `LocalContext`, a
//           Compose CompositionLocal whose `.current` gives the current Android `Context`.
// Why:      `appRoot` reads `LocalContext.current` to check the permission.
//
// In TS you'd write (pseudocode):
// ```ts
// import { LocalContext } from "androidx/compose/ui/platform";
// ```
import androidx.compose.ui.platform.LocalContext

// What:     `import androidx.compose.ui.text.font.FontWeight` supplies named text weights.
// Why:      Hardware legends use the supplied design's semibold printed ink.
//
// In TS you'd write (pseudocode):
// ```ts
// import { FontWeight } from "androidx/compose/ui/text/font";
// ```
import androidx.compose.ui.text.font.FontWeight

// What:     `import androidx.compose.ui.unit.dp` imports the `dp` EXTENSION PROPERTY on
//           numbers: writing `24.dp` produces a density-independent-pixel dimension. It
//           is an extension on `Int`/`Float`, so importing it enables the `<number>.dp`
//           syntax.
// Why:      Every spacing/size value uses `.dp` (e.g. `24.dp`, `8.dp`).
// Gotcha:   `24.dp` is calling an extension PROPERTY on the literal `24`; there is no such
//           "number.unit" syntax in TS.
//
// In TS you'd write (pseudocode):
// ```ts
// import { dp } from "androidx/compose/ui/unit"; // 24.dp ~ dp(24)
// ```
import androidx.compose.ui.unit.dp

// What:     `import dev.monochromatic.musicplayer.core.PageEntry` imports the `PageEntry`
//           record (`{ index: Int; name: String }`) from `.core`.
// Why:      `trackRow` takes a `PageEntry`.
//
// In TS you'd write (pseudocode):
// ```ts
// import { PageEntry } from "./core/Page";
// ```
import dev.monochromatic.musicplayer.core.PageEntry

// What:     `import dev.monochromatic.musicplayer.core.AllowAccessAction` and the first-run decision types.
// Why:      The first-run decision picks the screen and the Allow access action from the facts this screen reads.
//
// In TS you'd write (pseudocode):
// ```ts
// import { AllowAccessAction, FirstRunDecision, FirstRunInputs, firstRunDecisionFor } from "./core/FirstRunDecision";
// ```
import dev.monochromatic.musicplayer.core.AllowAccessAction
import dev.monochromatic.musicplayer.core.FirstRunDecision
import dev.monochromatic.musicplayer.core.FirstRunInputs
import dev.monochromatic.musicplayer.core.firstRunDecisionFor

// What:     `import dev.monochromatic.musicplayer.core.PlaybackMode` imports the
//           three-value enum `PlaybackMode` (`OFF`/`WITHIN_PAGE`/`ALL`).
// Why:      `controlRow` compares and sets shuffle modes.
//
// In TS you'd write (pseudocode):
// ```ts
// import { PlaybackMode } from "./core/PlaybackMode";
// ```
import dev.monochromatic.musicplayer.core.PlaybackMode

// What:     `import dev.monochromatic.musicplayer.core.rowDisplay` imports the
//           `rowDisplay(label, name)` FUNCTION that strips a folder tab's `<label>/` prefix
//           from a track's display name (and leaves letter / `#` tab names whole).
// Why:      `trackRow` shows the path BELOW the active folder tab, not the full relative path.
//
// In TS you'd write (pseudocode):
// ```ts
// import { rowDisplay } from "./core/Pagination";
// ```
import dev.monochromatic.musicplayer.core.rowDisplay

// What:     `import kotlinx.coroutines.delay` imports `delay(ms)`, a `suspend` function
//           that pauses the coroutine for the given milliseconds without blocking a
//           thread.
// Why:      `playerScreen`'s polling loop uses `delay(POSITION_POLL_MS)`.
//
// In TS you'd write (pseudocode):
// ```ts
// import { delay } from "kotlinx/coroutines"; // await delay(ms) ~ await sleep(ms)
// ```
import kotlinx.coroutines.delay

// What:     `const val LOG_TAG = "MusicPlayer"` declares a top-level PUBLIC (Kotlin
//           default) compile-time constant. `const` = compile-time + inlined; `val` =
//           never reassigned. No explicit type; Kotlin INFERS `String` from the literal.
// Why:      The shared logcat tag the on-device verification reads back; public and
//           top-level so `PlaybackService` (same package) reuses it.
//
// In TS you'd write (pseudocode):
// ```ts
// export const LOG_TAG = "MusicPlayer";
// ```
/**
 * Defines log tag value for this music-player component; the TypeScript-oriented notes above explain its source
 * and use.
 */
const val LOG_TAG = "MusicPlayer"

// What:     `private const val POSITION_POLL_MS: Long = 200L` declares a private
//           compile-time `Long` constant (64-bit; the `L` suffix forces `Long`, a bare
//           `200` would be `Int`).
// Why:      Position-poll cadence for the seek bar, in milliseconds (the desktop emits
//           every 0.1s); `Long` because `delay(...)` takes a `Long` millisecond count.
// Gotcha:   The `L` makes it a `Long` to match `delay(timeMillis: Long)`.
//
// In TS you'd write (pseudocode):
// ```ts
// const POSITION_POLL_MS = 200;
// ```
/**
 * Defines position poll ms value for this music-player component; the TypeScript-oriented notes above explain
 * its source and use.
 */
private const val POSITION_POLL_MS: Long = 200L

// What:     `class MainActivity : ComponentActivity() { ... }` declares the activity class
//           that EXTENDS `ComponentActivity` (the `: Super()` is the implicit `super()`
//           call). Android instantiates it, so there is no explicit primary constructor.
// Why:      Single-activity host. The player lives in `PlaybackService` so audio outlives
//           this activity; the UI binds to that service for a direct handle to the
//           service-owned `PlayerController` and drives it. Binding with
//           `BIND_AUTO_CREATE` also creates the service, which builds the `MediaSession`
//           and goes foreground on play.
//
// In TS you'd write (pseudocode):
// ```ts
// class MainActivity extends ComponentActivity {
//   // ...fields and lifecycle methods below...
// }
// ```
/**
 * Defines main activity type for this music-player component; the TypeScript-oriented notes above explain its
 * role.
 */
class MainActivity : ComponentActivity() {
    // What:     `private val boundController = mutableStateOf<PlayerController?>(null)`
    //           declares a private read-only field holding a Compose MUTABLE-STATE OBJECT
    //           (NOT a `by`-delegated property): `mutableStateOf<PlayerController?>(null)`
    //           creates an observable holder whose VALUE type is the nullable
    //           `PlayerController?` (the `<...>` is the generic type argument). You read/
    //           write the held value through `.value` (see `boundController.value` below).
    // Why:      Service-owned brain, observable so the Compose tree swaps off the loading
    //           state once bound. Held as the state OBJECT (not via `by`) so the activity
    //           can read/write `.value` from lifecycle callbacks AND the composition can
    //           observe it.
    // Gotcha:   This is the state OBJECT itself (accessed via `.value`), unlike the `by`
    //           form used elsewhere where the delegate hides `.value`.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // private readonly boundController = signal<PlayerController | null>(null);
    // ```
    /**
     * Defines bound controller value for this music-player component; the TypeScript-oriented notes above
     * explain its source and use.
     */
    private val boundController = mutableStateOf<PlayerController?>(null)

    // What:     `private var binder: PlaybackService.LocalBinder? = null` declares a
    //           private, reassignable field of the NULLABLE nested type
    //           `PlaybackService.LocalBinder?` (the binder handle, or null while unbound),
    //           initialised `null`.
    // Why:      Live binder for the post-grant library-load signal; null while unbound.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // private binder: PlaybackService.LocalBinder | null = null;
    // ```
    /**
     * Defines binder value for this music-player component; the TypeScript-oriented notes above explain its
     * source and use.
     */
    private var binder: PlaybackService.LocalBinder? = null

    // What:     `private var pendingRoot: Uri? = null` declares a private, reassignable
    //           NULLABLE `Uri?` field, initialised `null`.
    // Why:      A folder picked while the service was unbound, waiting to be applied;
    //           `connection`'s `onServiceConnected` consumes it once the rebind completes.
    //           The picker round-trip stops this activity, which unbinds the service, so
    //           the binder is often null at the moment the pick arrives.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // private pendingRoot: Uri | null = null;
    // ```
    /**
     * Defines pending root value for this music-player component; the TypeScript-oriented notes above explain
     * its source and use.
     */
    private var pendingRoot: Uri? = null

    // What:     `private val feedback: FeedbackHostState = FeedbackHostState()` is the activity's feedback queue.
    // Why:      The folder result runs outside composition, so the activity owns the queue that the player host draws
    //           and that this activity's producers post to.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // private readonly feedback = new FeedbackHostState();
    // ```
    /** Feedback queue that the player host draws, and that activity code posts messages to. */
    private val feedback: FeedbackHostState = FeedbackHostState()

    // What:     `private val trashResultLauncher` registers the result of Android's media trash and restore
    //           confirmations.
    // Why:      The confirmation is a system dialog whose answer returns to the activity, so the launcher must be
    //           registered here before the activity starts.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const trashResultLauncher = useActivityResult(StartIntentSenderForResult, (r) => trashRequest.deliver(r.code));
    // ```
    /** Activity result launcher for the system confirmations of media trash and restore requests. */
    private val trashResultLauncher = registerForActivityResult(
        ActivityResultContracts.StartIntentSenderForResult(),
    ) { result ->
        trashRequest.deliver(result.resultCode)
    }

    // What:     `private val trashRequest` routes the launcher's result to the request that started it.
    // Why:      The player host starts a request and waits for its answer, and this holder keeps the waiting callback.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const trashRequest = new TrashMediaStoreRequest(trashResultLauncher);
    // ```
    /** Holder that starts media trash and restore requests through the launcher and delivers each result. */
    private val trashRequest: TrashMediaStoreRequest = TrashMediaStoreRequest(trashResultLauncher)

    // What:     `private val analysisNotificationPermission` asks for the notification permission once.
    // Why:      D107 puts the true-peak analysis progress in a system notification, which Android 13 and later gate
    //           behind a permission; the asker registers its launcher here because that must happen before start.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // private readonly analysisNotificationPermission = new AnalysisNotificationPermission(this);
    // ```
    /** Asks for the notification permission once, when the first true-peak analysis starts. */
    private val analysisNotificationPermission: AnalysisNotificationPermission = AnalysisNotificationPermission(this)

    // What:     `private var heldFolderName: String? by mutableStateOf(null)` is the held folder's display name.
    // Why:      The first-run copy names the folder, and composition must redraw when the held folder changes.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // private heldFolderName: string | null = null; // observable
    // ```
    /** Display name of the folder whose grant is held, or null when no folder grant is held. */
    private var heldFolderName: String? by mutableStateOf(null)

    // What:     `private val folderPicker = registerForActivityResult(ActivityResultContracts.OpenDocumentTree()) {
    //           tree -> ... }`
    //           declares a private read-only field by calling `registerForActivityResult`
    //           with a CONTRACT (`OpenDocumentTree()`) and a TRAILING LAMBDA callback
    //           `{ tree -> ... }` (its parameter `tree` is the picked `Uri?`). The call
    //           returns a launcher you `launch(...)` later.
    // Why:      Activity-scoped folder picker, registered on the activity rather than
    //           inside the composition. `registerForActivityResult` must be called
    //           unconditionally while the activity is being created (the API forbids
    //           registering once it is STARTED), so the launcher cannot live inside a
    //           composable that may leave and re-enter composition. Registering on the
    //           activity ties the launcher to the activity lifecycle, so it is still
    //           registered when the picker result arrives and receives the granted tree.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // private readonly folderPicker = registerForActivityResult(
    //   new ActivityResultContracts.OpenDocumentTree(),
    //   (tree) => { if (tree !== null) this.onFolderChosen(tree); },
    // );
    // ```
    /**
     * Defines folder picker value for this music-player component; the TypeScript-oriented notes above explain
     * its source and use.
     */
    private val folderPicker = registerForActivityResult(ActivityResultContracts.OpenDocumentTree()) { tree ->
        // What:     `if (tree != null) { onFolderChosen(tree) }` null-checks the picked
        //           `Uri?`. Inside the block `tree` is SMART-CAST to a non-null `Uri`.
        // Why:      A cancelled picker yields null; only act on a real pick.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (tree !== null) this.onFolderChosen(tree);
        // ```
        if (tree != null) {
            // What:     `onFolderChosen(tree)` applies the chosen folder (persist grant +
            //           reload). `tree` is the smart-cast non-null `Uri`.
            // Why:      Make the picked folder the live library.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // this.onFolderChosen(tree);
            // ```
            onFolderChosen(tree)
        }
    }

    // What:     `private val connection = object : ServiceConnection { ... }` declares a
    //           private field holding an ANONYMOUS OBJECT that IMPLEMENTS the
    //           `ServiceConnection` interface. `object : Interface { ... }` is Kotlin's
    //           inline implementation of an interface (no named class), like a Java
    //           anonymous class or a TS object literal that satisfies an interface.
    // Why:      The service-connection callbacks that wire the bound brain into
    //           `boundController` and clear it on disconnect.
    // Gotcha:   `object : ServiceConnection { ... }` is an anonymous IMPLEMENTING object,
    //           NOT a singleton declaration and NOT a type cast.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // private readonly connection: ServiceConnection = {
    //   onServiceConnected: (name, service) => { ... },
    //   onServiceDisconnected: (name) => { ... },
    // };
    // ```
    /**
     * Defines connection value for this music-player component; the TypeScript-oriented notes above explain its
     * source and use.
     */
    private val connection = object : ServiceConnection {
        // What:     `override fun onServiceConnected(name: ComponentName?, service: IBinder?) { ... }`
        //           overrides the interface callback Android calls when the bind completes.
        //           `name` is the bound component (unused); `service` is the raw `IBinder?`.
        // Why:      Capture the binder and publish the brain to the observable state.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // onServiceConnected(name, service) { ... }
        // ```
        /**
         * Defines on service connected behavior for this music-player component; the TypeScript-oriented notes
         * above explain its call shape and effects.
         */
        override fun onServiceConnected(name: ComponentName?, service: IBinder?) {
            // What:     `val local = service as PlaybackService.LocalBinder` declares a local
            //           `local` by CASTING the `IBinder?` `service` to the concrete
            //           `PlaybackService.LocalBinder`. `as` is the UNSAFE cast: it throws
            //           `ClassCastException` at runtime if `service` is not actually that
            //           type (and would throw on null here too).
            // Why:      We know our own service hands back a `LocalBinder`, so we narrow to
            //           it to reach `.controller`/`.reloadFromRoot`.
            // Gotcha:   Kotlin `as` is a CHECKED cast that throws on mismatch; TS `as` is
            //           erased and never throws. The safe variant is `as?` (returns null on
            //           mismatch), not used here.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // const local = service as PlaybackService.LocalBinder;
            // ```
            /**
             * Defines local value for this music-player component; the TypeScript-oriented notes above explain
             * its source and use.
             */
            val local = service as PlaybackService.LocalBinder
            // What:     `binder = local` stores the binder handle.
            // Why:      Keep it for the post-grant library-load signal.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // this.binder = local;
            // ```
            binder = local
            // What:     `boundController.value = local.controller` writes the service-owned
            //           brain into the observable state via its `.value` SETTER (this is the
            //           state OBJECT, so `.value` is explicit, not hidden by `by`).
            // Why:      Publishing the controller makes the composition swap off the loading
            //           state and render the player.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // this.boundController.value = local.controller;
            // ```
            boundController.value = local.controller
            // What:     `Log.i(LOG_TAG, "bound to PlaybackService")` logs the bind.
            // Why:      Trace binding for verification.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // console.info(`[${LOG_TAG}] bound to PlaybackService`);
            // ```
            Log.i(LOG_TAG, "bound to PlaybackService")
            // What:     `val pending = pendingRoot; if (pending != null) { ... } else { local.rescan() }`
            //           reads the nullable `pendingRoot` into a local (smart-cast to non-null in
            //           the `then` branch), then branches on whether a folder pick is waiting.
            // Why:      Runs ONCE per activity, when the single `onCreate` bind connects (the
            //           bind is held until `onDestroy`, so this no longer fires per foreground;
            //           the per-foreground hook is now `onStart` -> `rescan`). Two cases at the
            //           initial connect. (1) A folder was picked before the bind completed
            //           (`pendingRoot` set): apply it now (a full reload that clears selection,
            //           the explicit-Open semantic). (2) Otherwise rescan/reconcile (LIVE
            //           UPDATE). `rescan()` is a no-op before the first load or while a load is
            //           in flight, so during the usual cold start (the service's own `onCreate`
            //           is still running `ensureLibraryLoaded`) it does nothing and cannot
            //           disturb the restore; if the service was already alive and loaded (e.g.
            //           headless playback), it reconciles.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // const pending = this.pendingRoot;
            // if (pending !== null) { local.reloadFromRoot(pending); this.pendingRoot = null; }
            // else { local.rescan(); }
            // ```
            /**
             * Defines pending value for this music-player component; the TypeScript-oriented notes above explain
             * its source and use.
             */
            val pending = pendingRoot
            if (pending != null) {
                // What:     `local.reloadFromRoot(pending)` tells the service to load the pending
                //           folder; `pendingRoot = null` clears it so it is not reapplied.
                // Why:      Apply the deferred explicit pick exactly once.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // local.reloadFromRoot(pending); this.pendingRoot = null;
                // ```
                local.reloadFromRoot(pending)
                pendingRoot = null
            } else {
                // What:     `local.rescan()` asks the service to re-scan the current source and
                //           reconcile the queue (live update), preserving the playing track.
                // Why:      The foreground signal: pick up files added/removed/renamed while the
                //           app was away. Safe no-op during the first load (cold-start restore).
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // local.rescan();
                // ```
                local.rescanOnForeground()
            }
        }

        // What:     `override fun onServiceDisconnected(name: ComponentName?) { ... }`
        //           overrides the callback Android calls when the service connection drops.
        // Why:      Clear our handles so nothing uses a dead connection.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // onServiceDisconnected(name) { ... }
        // ```
        /**
         * Defines on service disconnected behavior for this music-player component; the TypeScript-oriented
         * notes above explain its call shape and effects.
         */
        override fun onServiceDisconnected(name: ComponentName?) {
            // What:     `binder = null` clears the binder handle.
            // Why:      The connection is gone.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // this.binder = null;
            // ```
            binder = null
            // What:     `boundController.value = null` clears the observable brain (via the
            //           state object's `.value`), which makes the composition show the
            //           starting/loading gate again.
            // Why:      No brain while disconnected.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // this.boundController.value = null;
            // ```
            boundController.value = null
            // What:     `Log.i(LOG_TAG, "PlaybackService disconnected")` logs the drop.
            // Why:      Trace disconnects for verification.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // console.info(`[${LOG_TAG}] PlaybackService disconnected`);
            // ```
            Log.i(LOG_TAG, "PlaybackService disconnected")
        }
    }

    // What:     `override fun onCreate(savedInstanceState: Bundle?) { ... }` overrides the
    //           activity-creation lifecycle hook. `savedInstanceState` is the saved-state
    //           bag (or null on a fresh start).
    // Why:      Set up edge-to-edge drawing and mount the Compose UI.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // override onCreate(savedInstanceState: Bundle | null): void { ... }
    // ```
    /**
     * Defines on create behavior for this music-player component; the TypeScript-oriented notes above explain
     * its call shape and effects.
     */
    override fun onCreate(savedInstanceState: Bundle?) {
        // What:     `super.onCreate(savedInstanceState)` calls the base class first.
        // Why:      The framework must initialize the activity before we touch it.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // super.onCreate(savedInstanceState);
        // ```
        super.onCreate(savedInstanceState)
        // What:     `heldFolderName = LibraryRoot.heldRoot(this)?.let { ... }` reads the remembered folder's name.
        // Why:      The first-run copy names the held folder on the first frame, before any folder is picked again.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // this.heldFolderName = heldRoot(this)?.name ?? null;
        // ```
        // The grant is read on the I/O dispatcher (its preference file and permission list are disk reads), so the
        // library heading shows its generic name for the first frame when a folder is held.
        lifecycleScope.launch {
            heldFolderName = withContext(Dispatchers.IO) {
                LibraryRoot.heldRoot(this@MainActivity)?.let { treeUri -> heldFolderNameOf(treeUri) }
            }
        }
        // What:     `Log.i(LOG_TAG, "MainActivity.onCreate")` logs an info line at activity
        //           creation. There is no variant suffix because the current build has one engine
        //           and no `BuildConfig.FLAVOR` constant.
        // Why:      Trace activity launch in logcat, for verification.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // console.info(`[${LOG_TAG}] MainActivity.onCreate`);
        // ```
        Log.i(LOG_TAG, "MainActivity.onCreate")
        // Drop the notification channel of builds before D107 on every launch, because an upgrade after the first
        // analysis finished never starts the sweep service that would otherwise do it.
        deleteLegacyAnalysisChannel(this)
        // The one-time parallel true-peak index starts from the `onAudioGranted` callback that `appRoot` receives,
        // once a music source is readable, because a start here on a first launch finds no library before access.
        // Ask for the notification permission once, when the analysis that just started reports its first total.
        analysisNotificationPermission.start()
        // What:     `enableEdgeToEdge()` draws the app behind the system bars. (Folds in the
        //           old inline note: draw edge to edge, the platform default on targetSdk
        //           35+, and let the `Scaffold` apply the system-bar insets.)
        // Why:      Modern full-bleed layout; the `Scaffold` supplies inset padding.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // enableEdgeToEdge();
        // ```
        enableEdgeToEdge()
        // What:     `bindService(Intent(this,
        //           PlaybackService::class.java).setAction(PlaybackService.ACTION_LOCAL_BIND), connection,
        //           Context.BIND_AUTO_CREATE)`
        //           binds to the service ONCE for the activity's whole lifetime (paired with the
        //           single `unbindService` in `onDestroy`). `BIND_AUTO_CREATE` also creates the
        //           service if it is not already running; `connection.onServiceConnected`
        //           publishes the brain into `boundController`.
        // Why:      Hold the binding across `onStop`/`onStart` so the bind-only
        //           `MediaSessionService` is NOT torn down on every background. The bind used to
        //           live in `onStart`/`onStop`, so each app switch dropped the last client, the
        //           service was destroyed, and the next foreground re-created it and re-ran
        //           `ensureLibraryLoaded` (a full ~3.6k-track rescan with a blocking loading
        //           state). Binding once keeps the service, its engine, and the loaded library
        //           alive, so a foreground is a cheap non-blocking reconcile (`onStart` ->
        //           `rescan`). media3 keeps the service foreground while playing, so background
        //           audio is unaffected.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // this.bindService(
        //   new Intent(this, PlaybackService).setAction(PlaybackService.ACTION_LOCAL_BIND),
        //   this.connection,
        //   Context.BIND_AUTO_CREATE,
        // );
        // ```
        bindService(
            Intent(this, PlaybackService::class.java).setAction(PlaybackService.ACTION_LOCAL_BIND),
            connection,
            Context.BIND_AUTO_CREATE,
        )
        // What:     `setContent { ... }` mounts a Compose UI tree as this activity's
        //           content; the trailing lambda IS the root composable.
        // Why:      Render the app UI.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // setContent(() => {
        //   return <PlayerTheme> ... </PlayerTheme>;
        // });
        // ```
        setContent {
            // What:     `playerTheme { ... }` themes its trailing-lambda child UI with the palette derived from the
            //           system accent (A3, D108) and provides the derived roles that Material has no slot for.
            // Why:      Every color of the player is derived from one root in OKLCH, so the platform's own dynamic
            //           color roles are not used.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // <PlayerTheme> <Surface modifier={Modifier.fillMaxSize()}> ... </Surface> </PlayerTheme>
            // ```
            playerTheme {
                // What:     `Surface(modifier = Modifier.fillMaxSize()) { ... }` calls the
                //           `Surface` composable with a `Modifier.fillMaxSize()` chain (occupy
                //           the whole screen) and a trailing-lambda child.
                // Why:      A themed full-screen background to host the content.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // <Surface modifier={Modifier.fillMaxSize()}> ... </Surface>
                // ```
                Surface(modifier = Modifier.fillMaxSize()) {
                    // What:     `val controller = boundController.value` reads the observable
                    //           state's `.value` (a `PlayerController?`). Reading `.value`
                    //           inside a composable SUBSCRIBES this UI to changes, so it
                    //           recomposes when the brain binds/unbinds.
                    // Why:      Decide whether to show the loading gate or the bound app.
                    //
                    // In TS you'd write (pseudocode):
                    // ```ts
                    // const controller = this.boundController.value;
                    // ```
                    /**
                     * Defines controller value for this music-player component; the TypeScript-oriented notes
                     * above explain its source and use.
                     */
                    val controller = boundController.value
                    // What:     `if (controller == null) { startingGate() } else { appRoot(...) }`
                    //           branches the UI: no brain yet -> show `startingGate`; bound ->
                    //           show `appRoot` (smart-cast `controller` to non-null in the else).
                    // Why:      Render the right screen for the bind state.
                    //
                    // In TS you'd write (pseudocode):
                    // ```ts
                    // if (controller === null) return <startingGate/>;
                    // return (
                    //   <appRoot
                    //     controller={controller}
                    //     onAudioGranted={() => this.binder?.ensureLibraryLoaded()}
                    //     onChooseFolder={() => this.folderPicker.launch(null)}
                    //   />
                    // );
                    // ```
                    if (controller == null) {
                        // What:     `startingGate()` calls the placeholder composable shown
                        //           while binding.
                        // Why:      Show "Starting..." until the service binds.
                        //
                        // In TS you'd write (pseudocode):
                        // ```ts
                        // <startingGate/>
                        // ```
                        startingGate()
                    } else {
                        // What:     `appRoot(controller = controller, onAudioGranted = { binder?.ensureLibraryLoaded()
                        //           }, onChooseFolder = { folderPicker.launch(null) },)`
                        //           calls the `appRoot` composable with named arguments. Two of
                        //           them are LAMBDA callbacks: `onAudioGranted` safe-calls
                        //           `binder?.ensureLibraryLoaded()`; `onChooseFolder` launches the
                        //           folder picker with `folderPicker.launch(null)`.
                        // Why:      Hand the bound brain and the two activity-level actions to the
                        //           permission/library gate.
                        //
                        // In TS you'd write (pseudocode):
                        // ```ts
                        // <appRoot
                        //   controller={controller}
                        //   onAudioGranted={() => this.binder?.ensureLibraryLoaded()}
                        //   onChooseFolder={() => this.folderPicker.launch(null)}
                        // />
                        // ```
                        CompositionLocalProvider(LocalTrashMediaStoreRequest provides trashRequest) {
                            appRoot(
                                controller = controller,
                                onAudioGranted = {
                                    binder?.ensureLibraryLoaded()
                                    // The true-peak analysis starts from the playback service after every completed
                                    // library load or rescan, for the tracks it has not looked at yet (M9).
                                },
                                onChooseFolder = { folderPicker.launch(null) },
                                feedback = feedback,
                                folderName = heldFolderName,
                            )
                        }
                    }
                }
            }
        }
    }

    // What:     `override fun onStart() { ... }` overrides the lifecycle hook called when
    //           the activity becomes visible (every foreground, including app switches).
    // Why:      Trigger the foreground LIVE UPDATE. Binding happens once in `onCreate` and is
    //           held until `onDestroy`, so `onStart` does not bind; it just asks the
    //           already-bound service to rescan and reconcile the queue.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // override onStart(): void { this.binder?.rescan(); }
    // ```
    /**
     * Defines on start behavior for this music-player component; the TypeScript-oriented notes above explain its
     * call shape and effects.
     */
    override fun onStart() {
        // What:     `super.onStart()` calls the base class first.
        // Why:      Let the framework run its own start logic.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // super.onStart();
        // ```
        super.onStart()
        // What:     `binder?.rescan()` SAFE-CALLs the service's `rescan` (the live-update entry
        //           point) when the binder is present. On the very first foreground the bind
        //           from `onCreate` may not have completed yet (`binder` null), so this no-ops
        //           and the initial library load is the service's own `onCreate`
        //           (`ensureLibraryLoaded`). On every later foreground the binder is set, so this
        //           re-scans the current source and reconciles the queue WITHOUT a loading state
        //           (`reconcileLibrary`), preserving the playing track.
        // Why:      Pick up files added/removed/renamed while the app was away, non-blockingly
        //           (the desktop "Rescan" analog; see
        //           doc/decision/music-player-live-update-rescan.md). `rescan` itself no-ops
        //           before the first load or while a load is in flight, so a foreground arriving
        //           during the cold-start restore cannot disturb it.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // this.binder?.rescan();
        // ```
        binder?.rescanOnForeground()
    }

    // What:     `override fun onStop() { ... }` overrides the lifecycle hook called when the
    //           activity is no longer visible (backgrounded, e.g. an app switch).
    // Why:      Persist the resume position. It does NOT unbind: the binding is held from
    //           `onCreate` to `onDestroy` so the bind-only service is not torn down on a
    //           background (which previously forced a full reload on the next foreground). The
    //           service keeps running, and media3 keeps it foreground while playing, so audio is
    //           unaffected.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // override onStop(): void { this.binder?.saveSession(); }
    // ```
    /**
     * Defines on stop behavior for this music-player component; the TypeScript-oriented notes above explain its
     * call shape and effects.
     */
    override fun onStop() {
        // What:     `super.onStop()` calls the base class first.
        // Why:      Let the framework run its own stop logic.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // super.onStop();
        // ```
        super.onStop()
        // What:     `binder?.saveSession()` SAFE-CALLs the service's `saveSession` while the
        //           binder is valid.
        // Why:      Capture the resume position when the user backgrounds mid-track while
        //           playing: no state-change fires then, so the controller's `onPersist` would
        //           miss it, and `onDestroy` may not run if the process is later killed. Saving
        //           here, on the live service controller, makes the backgrounded position
        //           durable. No-op if the library has not loaded.
        // Note:     We intentionally do NOT `unbindService` here, and we keep `boundController`.
        //           Holding the binding keeps the service, its engine, and the loaded library
        //           alive across the background, so the next foreground is a cheap non-blocking
        //           reconcile (`onStart` -> `rescan`) instead of a destroy + full reload. The
        //           binding is released once, in `onDestroy`.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // this.binder?.saveSession();
        // ```
        binder?.saveSession()
    }

    // What:     `override fun onDestroy() { ... }` overrides the final lifecycle hook, called
    //           when the activity is being destroyed.
    // Why:      Release the binding taken in `onCreate`. This single unbind (held until now) is
    //           what keeps the bind-only `MediaSessionService` alive across `onStop`/`onStart`,
    //           fixing the reload-on-every-app-switch. media3 keeps the service foreground while
    //           playing, so background audio survives the activity's destruction.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // override onDestroy(): void { this.unbindService(this.connection); this.binder = null; }
    // ```
    /**
     * Defines on destroy behavior for this music-player component; the TypeScript-oriented notes above explain
     * its call shape and effects.
     */
    override fun onDestroy() {
        // What:     `super.onDestroy()` calls the base class first.
        // Why:      Let the framework run its own destroy logic.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // super.onDestroy();
        // ```
        super.onDestroy()
        // What:     `unbindService(connection)` releases the connection taken in `onCreate`
        //           (paired one-to-one with that single `bindService`), then `binder = null`
        //           clears the handle.
        // Why:      Drop our reference so the service can be reclaimed when nothing else keeps it
        //           alive (e.g. not playing).
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // this.unbindService(this.connection); this.binder = null;
        // ```
        unbindService(connection)
        binder = null
    }

    // What:     `private fun takeFolderGrant(treeUri: Uri)` persists the picker's grant, write first and read only
    //           second.
    // Why:      The copy-back trash needs write access, and a picker that grants read only must still persist the
    //           library.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // for (const flags of folderGrantAttempts(READ, WRITE)) { try { takePersistable(treeUri, flags); return; } catch
    //   (e) { /* next */ } }
    // ```
    /**
     * Persists the grant for the picked tree with each flag set in order, keeping the first one the provider accepts.
     *
     * @throws SecurityException when the provider refuses even the read-only grant
     */
    private fun takeFolderGrant(treeUri: Uri) {
        /** Flag sets to request, read and write first and read only as the fallback. */
        val attempts: List<Int> = folderGrantAttempts(
            readFlag = Intent.FLAG_GRANT_READ_URI_PERMISSION,
            writeFlag = Intent.FLAG_GRANT_WRITE_URI_PERMISSION,
        )
        for ((index, flags) in attempts.withIndex()) {
            try {
                contentResolver.takePersistableUriPermission(treeUri, flags)
                /** Whether this kept grant includes write access. */
                val writeKept: Boolean = (flags and Intent.FLAG_GRANT_WRITE_URI_PERMISSION) != 0
                Log.i(LOG_TAG, "folder grant kept write=$writeKept tree=$treeUri")
                return
            } catch (grantFailure: SecurityException) {
                if (index == attempts.lastIndex) {
                    throw grantFailure
                }
                Log.w(LOG_TAG, "folder write grant refused, trying read only tree=$treeUri", grantFailure)
            }
        }
    }

    // What:     `private fun onFolderChosen(treeUri: Uri) { ... }` declares a private method
    //           taking a tree `Uri`, block body, `Unit`.
    // Why:      Persist read access to a just-picked SAF folder and make it the live
    //           library. Taking a persistable grant lets a later cold start re-read the
    //           folder with no re-pick; the bound service is told to rescan now. Only the
    //           activity can do this: the persistable grant is delivered to the component
    //           that launched the picker.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // private onFolderChosen(treeUri: Uri): void { ... }
    // ```
    /**
     * Defines on folder chosen behavior for this music-player component; the TypeScript-oriented notes above
     * explain its call shape and effects.
     */
    private fun onFolderChosen(treeUri: Uri) {
        // What:     `takeFolderGrant(treeUri)` takes a persistable grant for the picked tree, read and write when
        //           the picker allows it and read only otherwise. `contentResolver` is the activity's resolver.
        //           The OpenDocumentTree contract returns only the tree Uri, so the result flags are not visible here.
        // Why:      So a later cold start can re-read the folder without re-prompting.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // this.contentResolver.takePersistableUriPermission(treeUri, Intent.FLAG_GRANT_READ_URI_PERMISSION);
        // ```
        try {
            takeFolderGrant(treeUri)
        } catch (grantFailure: SecurityException) {
            // What:     `feedback.showFeedback(feedbackFolderOpenFailed())` posts a two-line error and returns.
            // Why:      The provider refused a lasting grant, so the folder is not saved and the user is told to choose
            //           it again; the saved root and the library stay as they were.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // catch (grantFailure) { feedback.showFeedback(feedbackFolderOpenFailed()); return; }
            // ```
            Log.w(LOG_TAG, "folder grant could not be kept for $treeUri", grantFailure)
            feedback.showFeedback(feedbackFolderOpenFailed())
            return
        }
        // What:     `LibraryRoot.save(this, treeUri)` persists the chosen folder URI so a
        //           restart remembers it.
        // Why:      Remembering the choice backs a later restart.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // LibraryRoot.save(this, treeUri);
        // ```
        LibraryRoot.save(this, treeUri)
        // What:     `heldFolderName = heldFolderNameOf(treeUri)` updates the name the first-run copy shows.
        // Why:      A new folder replaces the old one, so the first-run decision reads the new name on the next frame.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // this.heldFolderName = heldFolderNameOf(treeUri);
        // ```
        heldFolderName = heldFolderNameOf(treeUri)
        // What:     `val bound = binder` declares a local `bound` (inferred nullable
        //           `PlaybackService.LocalBinder?`) snapshotting the current binder.
        // Why:      Snapshot lets us branch on bound-vs-unbound and smart-cast below.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const bound = this.binder;
        // ```
        /**
         * Defines bound value for this music-player component; the TypeScript-oriented notes above explain its
         * source and use.
         */
        val bound = binder
        // What:     `if (bound != null) { bound.reloadFromRoot(treeUri) } else { ... }`
        //           branches on whether we are bound. `bound != null` smart-casts `bound` to
        //           non-null in the `then` branch.
        // Why:      If bound, reload immediately; otherwise defer the pick until the rebind.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (bound !== null) bound.reloadFromRoot(treeUri);
        // else this.pendingRoot = treeUri;
        // ```
        if (bound != null) {
            // What:     `bound.reloadFromRoot(treeUri)` tells the bound service to rescan the
            //           picked folder now.
            // Why:      Apply the pick immediately when connected.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // bound.reloadFromRoot(treeUri);
            // ```
            bound.reloadFromRoot(treeUri)
        } else {
            // What:     `pendingRoot = treeUri` stores the pick for later. (Folds in the old
            //           inline note: the picker stopped this activity, which unbound the
            //           service; the pick is applied when the rebind connects, see
            //           `connection`.)
            // Why:      Defer until `onServiceConnected` can apply it.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // this.pendingRoot = treeUri;
            // ```
            pendingRoot = treeUri
        }
        // What:     `Log.i(LOG_TAG, "folder chosen: $treeUri")` logs the chosen folder.
        // Why:      Trace folder selection for verification.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // console.info(`[${LOG_TAG}] folder chosen: ${treeUri}`);
        // ```
        Log.i(LOG_TAG, "folder chosen: $treeUri")
    }
}

// What:     `@Composable` is the ANNOTATION marking the next function as a Compose UI
//           component (it may emit UI and read composition state).
// Why:      `appRoot` is a UI component.
//
// In TS you'd write (pseudocode):
// ```ts
// // (no annotation; a function returning UI is a component)
// ```
@Composable
// What:     `private fun appRoot(...)` declares the composable that draws the player over the bound controller.
// Why:      The first-run decision, the audio permission, and the library trigger share one set of states here, so
//           the pane and the load agree on which source is open.
//
// In TS you'd write (pseudocode):
// ```ts
// function appRoot(props: {
//   controller: PlayerController;
//   onAudioGranted: () => void;
//   onChooseFolder: () => void;
//   feedback: FeedbackHostState;
//   folderName: string | null;
// }) { ... }
// ```
/**
 * Draws the player over the bound controller, asks for audio access when no folder is held, starts the library
 * load once a source can be read, and draws the first-run pane when the open source needs it.
 *
 * @param controller bound controller that owns the library and playback
 * @param onAudioGranted starts the library load and the initial true-peak analysis once a source can be read
 * @param onChooseFolder opens the folder picker
 * @param feedback feedback queue that the player host draws
 * @param folderName display name of the held folder, or null when no folder grant is held
 */
private fun appRoot(
    controller: PlayerController,
    onAudioGranted: () -> Unit,
    onChooseFolder: () -> Unit,
    feedback: FeedbackHostState,
    folderName: String?,
) {
    // What:     `val context = LocalContext.current` reads the Android context from the composition.
    // Why:      The permission check, the saved request flag, and the settings intent all need it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const context = useContext(LocalContext);
    // ```
    /** Android context that the permission check, the saved request flag, and the settings intent use. */
    val context = LocalContext.current
    // What:     `checkNotNull(context as? ComponentActivity)` narrows the context to the activity.
    // Why:      The rationale rule and the lifecycle observer exist only on an activity, and the screen always runs
    //           inside one, so anything else is a programming error that fails loudly.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const activity = context instanceof ComponentActivity ? context : throwUnreachable();
    // ```
    /** Activity that hosts this screen, which reports the permission rationale and the lifecycle. */
    val activity: ComponentActivity = checkNotNull(context as? ComponentActivity) {
        "The first-run screen must run inside a ComponentActivity."
    }
    // What:     `var hasAudioAccess by remember { mutableStateOf(hasAudioPermission(context)) }` holds the grant.
    // Why:      It is seeded from the system and rechecked when the screen resumes, so a grant made in Android's
    //           settings takes effect without a restart.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const [hasAudioAccess, setHasAudioAccess] = useState(hasAudioPermission(context));
    // ```
    /** Whether Android grants access to music on this device, rechecked when the screen resumes. */
    var hasAudioAccess by remember { mutableStateOf(hasAudioPermission(context)) }
    // What:     `var canPrompt by remember { mutableStateOf(canPromptAudioAccess(activity)) }` holds the prompt rule.
    // Why:      Once Android stops showing its prompt, the Allow access button can only open the app's settings page.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const [canPrompt, setCanPrompt] = useState(canPromptAudioAccess(activity));
    // ```
    /** Whether Android still shows its permission prompt, recomputed after each answer and each resume. */
    var canPrompt by remember { mutableStateOf(canPromptAudioAccess(activity)) }
    // What:     `val permissionLauncher = rememberLauncherForActivityResult(...)` registers the permission prompt.
    // Why:      The answer updates the grant and the prompt rule, so the first-run decision follows the user's choice.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const permissionLauncher = useActivityResult(RequestPermission, (granted) => { ... });
    // ```
    /**
     * Whether the permission prompt is open and unanswered. It starts true when the launch will open the prompt, so
     * no error shows for the frame before the prompt appears or while it is up.
     */
    var promptShowing by remember { mutableStateOf(!hasAudioAccess && folderName == null && canPrompt) }
    /** Launcher for the audio permission prompt, whose answer updates the grant and the prompt rule. */
    val permissionLauncher = rememberLauncherForActivityResult(
        ActivityResultContracts.RequestPermission(),
    ) { granted ->
        Log.i(LOG_TAG, "audio permission granted=$granted")
        hasAudioAccess = granted
        canPrompt = canPromptAudioAccess(activity)
        promptShowing = false
    }
    // What:     `fun requestAudioAccess()` records the request and then opens the permission prompt.
    // Why:      The request flag is saved first, so the rule can tell a first refusal from a permanent one later.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const requestAudioAccess = () => { saveRequested(); permissionLauncher.launch(AUDIO_PERMISSION); };
    // ```
    /** Records that this app asked for music access, then opens Android's permission prompt. */
    fun requestAudioAccess() {
        SessionStore.saveAudioAccessRequested(context, requested = true)
        promptShowing = true
        permissionLauncher.launch(audioPermission())
    }
    // What:     `fun runAllowAccess(action: AllowAccessAction)` performs the action that the decision chose.
    // Why:      The decision picks the prompt while Android still shows it and the app's settings page after that.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const runAllowAccess = (action) => action === "requestPermission" ? requestAudioAccess() : openSettings();
    // ```
    /** Performs the Allow access action: the permission prompt, or the app's page in Android's settings. */
    fun runAllowAccess(action: AllowAccessAction) {
        when (action) {
            AllowAccessAction.RequestPermission -> requestAudioAccess()
            AllowAccessAction.OpenAppSettings -> {
                Log.i(LOG_TAG, "opening app settings for audio access")
                context.startActivity(appDetailsSettingsIntent(context.packageName))
            }
        }
    }
    // What:     `DisposableEffect(activity) { ... }` observes the lifecycle while this screen is shown.
    // Why:      On each resume the grant and the prompt rule are read again, because Android's settings page can
    //           change them while the app is in the background.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // useEffect(() => { const off = onResume(recheck); return off; }, [activity]);
    // ```
    DisposableEffect(activity) {
        // What:     `val observer = LifecycleEventObserver { ... }` reacts to lifecycle events.
        // Why:      Only the resume event needs a recheck, so other events are ignored.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const observer = (event) => { if (event === "resume") recheck(); };
        // ```
        /** Observer that rechecks the grant and the prompt rule when the activity resumes. */
        val observer = LifecycleEventObserver { _, event ->
            if (event == Lifecycle.Event.ON_RESUME) {
                Log.i(LOG_TAG, "audio access rechecked after resume")
                hasAudioAccess = hasAudioPermission(context)
                canPrompt = canPromptAudioAccess(activity)
            }
        }
        activity.lifecycle.addObserver(observer)
        onDispose {
            activity.lifecycle.removeObserver(observer)
        }
    }
    // What:     `LaunchedEffect(Unit)` asks for access once when the screen first appears.
    // Why:      The first launch asks as before, but not when a folder is held (no permission is needed to read it) or
    //           when Android no longer shows the prompt.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // useEffect(() => { if (!hasAudioAccess && !folderName && canPrompt) requestAudioAccess(); }, []);
    // ```
    LaunchedEffect(Unit) {
        if (!hasAudioAccess && folderName == null && canPrompt) {
            requestAudioAccess()
        }
    }
    // What:     `LaunchedEffect(hasAudioAccess, folderName)` starts the library load once a source can be read.
    // Why:      A grant or a held folder makes the device library or the folder readable, and the load is idempotent.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // useEffect(() => { if (hasAudioAccess || folderName) onAudioGranted(); }, [hasAudioAccess, folderName]);
    // ```
    LaunchedEffect(hasAudioAccess, folderName) {
        if (hasAudioAccess || folderName != null) {
            onAudioGranted()
        }
    }
    // What:     `val inputs = FirstRunInputs(...)` gathers the facts that the first-run decision reads this frame.
    // Why:      The controller snapshot supplies the load state and the track count, and the access states supply the
    //           rest.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const inputs = { permissionGranted: hasAudioAccess, heldFolderName: folderName, ... };
    // ```
    /** Facts the first-run decision reads this frame, taken from the controller snapshot and the access states. */
    val inputs: FirstRunInputs = FirstRunInputs(
        permissionGranted = hasAudioAccess,
        heldFolderName = folderName,
        loading = controller.uiState.loading,
        trackCount = controller.uiState.queueSize,
        canPromptForPermission = canPrompt,
        promptShowing = promptShowing,
    )
    /** First-run screen to draw, or null when the player shows, with the Allow access action. */
    val decision: FirstRunDecision = firstRunDecisionFor(inputs)
    // What:     `LaunchedEffect(decision)` logs each change of the decided screen.
    // Why:      A device run shows which screen was reached and which action the Allow access button performs.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // useEffect(() => console.log("first run", decision), [decision]);
    // ```
    LaunchedEffect(decision) {
        Log.i(LOG_TAG, "first-run state=${decision.state?.name ?: "PLAYER"} allowAccess=${decision.allowAccess}")
    }
    // What:     `val firstRunContent = ...` builds the pane content for the decided state, or null for the player.
    // Why:      The host draws the pane in its own layout so the Settings route stays reachable from it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const firstRunContent = decision.state ? firstRunPaneContent(decision.state, ...) : null;
    // ```
    /** Pane content for the decided state, or null when the player shows. */
    val firstRunContent: (@Composable (onSettings: () -> Unit) -> Unit)? = if (decision.awaitingAnswer) {
        // Until the user answers the permission prompt nothing is known about the library, so the screen stays blank.
        { _ -> Box(modifier = Modifier.fillMaxSize()) }
    } else {
        decision.state?.let { shown ->
            firstRunPaneContent(
                state = shown,
                folderName = folderName,
                onAllowAccess = { runAllowAccess(decision.allowAccess) },
                onOpenFolder = onChooseFolder,
            )
        }
    }
    playerScreen(
        controller = controller,
        onChooseFolder = onChooseFolder,
        feedback = feedback,
        firstRunContent = firstRunContent,
        libraryName = libraryDisplayName(folderName),
    )
}

// What:     `@Composable` marks the next function as a Compose component.
// Why:      `startingGate` is a UI component.
//
// In TS you'd write (pseudocode):
// ```ts
// // (component function)
// ```
@Composable
// What:     `private fun startingGate() { ... }` declares a private no-arg composable.
// Why:      Brief placeholder shown while the activity binds to `PlaybackService`.
//
// In TS you'd write (pseudocode):
// ```ts
// function startingGate() { ... }
// ```
/**
 * Defines starting gate behavior for this music-player component; the TypeScript-oriented notes above explain
 * its call shape and effects.
 */
private fun startingGate() {
    // What:     `Column( modifier = Modifier.fillMaxSize().padding(24.dp), verticalArrangement =
    //           Arrangement.spacedBy(12.dp, Alignment.CenterVertically), horizontalAlignment =
    //           Alignment.CenterHorizontally, ) { ... }`
    //           calls the `Column` layout composable with named args and a trailing-lambda
    //           child. `Modifier.fillMaxSize().padding(24.dp)` is a chained modifier (fill the
    //           screen, then 24dp padding; `24.dp` is the `dp` extension on the literal).
    //           `Arrangement.spacedBy(12.dp, Alignment.CenterVertically)` spaces children 12dp
    //           apart, centered vertically; `Alignment.CenterHorizontally` centers them across.
    // Why:      Center the "Starting..." text on screen.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // <Column
    //   modifier={Modifier.fillMaxSize().padding(dp(24))}
    //   verticalArrangement={Arrangement.spacedBy(dp(12), Alignment.CenterVertically)}
    //   horizontalAlignment={Alignment.CenterHorizontally}
    // >
    //   <Text>Starting Music Player...</Text>
    // </Column>
    // ```
    Column(
        modifier = Modifier
            .fillMaxSize()
            .padding(24.dp),
        verticalArrangement = Arrangement.spacedBy(12.dp, Alignment.CenterVertically),
        horizontalAlignment = Alignment.CenterHorizontally,
    ) {
        // What:     `Text("Starting Music Player...")` renders the placeholder text.
        // Why:      Tell the user the app is starting.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // <Text>Starting Music Player...</Text>
        // ```
        Text("Starting Music Player...")
    }
}

/** Holds live seek position and duration sampled from current controller. */
internal data class PlaybackProgress(
    /** Stores elapsed playback seconds. */
    val position: Double,
    /** Stores current track duration seconds. */
    val duration: Double,
)

/** Returns Compose-observable playback progress retargeted when controller changes. */
@Composable
internal fun rememberPlaybackProgress(controller: PlayerController): PlaybackProgress {
    /** Holds live playback position. */
    var position: Double by remember { mutableDoubleStateOf(0.0) }
    /** Holds live track duration. */
    var duration: Double by remember { mutableDoubleStateOf(0.0) }
    LaunchedEffect(controller) {
        while (true) {
            position = controller.positionSec()
            duration = controller.durationSec()
            delay(POSITION_POLL_MS)
        }
    }
    return PlaybackProgress(position = position, duration = duration)
}

// What:     `@Composable fun playerScreen(controller: PlayerController, onChooseFolder: () -> Unit)` hosts the
//           production player screen and its Search, Settings, and template destinations.
// Why:      The host in PlayerHost.kt owns the route and the screen state, while every playback and browsing
//           change goes through the controller so the snapshot stays the single source of truth.
//
// In TS you'd write (pseudocode):
// ```ts
// function playerScreen(props: { controller: PlayerController; onChooseFolder: () => void }): UIElement { /* ... */ }
// ```
/**
 * Draws the production player screen from the controller state and routes each action to the controller.
 *
 * @param feedback feedback queue that the player host draws and expires
 * @param firstRunContent first-run pane to draw over the player, or null when the player shows
 * @param libraryName name of the opened folder, or the name of the MediaStore library, for the folder pane heading
 */
@Composable
fun playerScreen(
    controller: PlayerController,
    onChooseFolder: () -> Unit,
    feedback: FeedbackHostState,
    firstRunContent: (@Composable (onSettings: () -> Unit) -> Unit)?,
    libraryName: String,
) {
    Scaffold(contentWindowInsets = WindowInsets(0, 0, 0, 0)) { innerPadding ->
        Box(modifier = Modifier.fillMaxSize().padding(innerPadding)) {
            playerHost(
                controller = controller,
                feedback = feedback,
                onChooseFolder = onChooseFolder,
                firstRunContent = firstRunContent,
                libraryName = libraryName,
                emptyNotice = { state ->
                    if (state.queueSize == 0) {
                        Box(modifier = Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
                            if (state.loading) {
                                loadingNotice()
                            } else {
                                Text("No music found in your audio library.")
                            }
                        }
                    }
                },
            )
        }
    }
}

// What:     `@Composable` marks the next function as a Compose component.
// Why:      `loadingNotice` is a UI component.
//
// In TS you'd write (pseudocode):
// ```ts
// // (component function)
// ```
@Composable
// What:     `private fun loadingNotice() { ... }` declares a private no-arg composable.
// Why:      Shown while a library load or folder scan runs: a spinner and a short loading
//           line.
//
// In TS you'd write (pseudocode):
// ```ts
// function loadingNotice() { ... }
// ```
/**
 * Defines loading notice behavior for this music-player component; the TypeScript-oriented notes above explain
 * its call shape and effects.
 */
private fun loadingNotice() {
    // What:     `Row( verticalAlignment = Alignment.CenterVertically, horizontalArrangement =
    //           Arrangement.spacedBy(12.dp), modifier = Modifier.padding(vertical = 12.dp), ) { ... }`
    //           lays the spinner and text on one centered line, 12dp apart, with vertical
    //           padding.
    // Why:      Put the spinner next to its label.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // <Row
    //   verticalAlignment={Alignment.CenterVertically}
    //   horizontalArrangement={Arrangement.spacedBy(dp(12))}
    //   modifier={Modifier.padding({ vertical: dp(12) })}
    // > ... </Row>
    // ```
    Row(
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(12.dp),
        modifier = Modifier.padding(vertical = 12.dp),
    ) {
        // What:     `CircularProgressIndicator(modifier = Modifier.size(20.dp))` renders a 20dp
        //           spinner.
        // Why:      Show indeterminate progress.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // <CircularProgressIndicator modifier={Modifier.size(dp(20))}/>
        // ```
        CircularProgressIndicator(modifier = Modifier.size(20.dp))
        // What:     `Text("Loading your library…")` shows the loading line.
        // Why:      Tell the user the library is loading.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // <Text>Loading your library…</Text>
        // ```
        Text("Loading your library…")
    }
}
