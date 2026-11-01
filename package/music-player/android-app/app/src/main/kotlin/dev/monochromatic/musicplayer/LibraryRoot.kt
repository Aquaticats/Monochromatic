// File summary (folds in the domain content from the old KDoc on `object LibraryRoot`):
//
// This file remembers the one folder the user picked as their music library "root", and survives
// app restarts and full process death. On Android, the user picks a folder through the Storage
// Access Framework (SAF): the system file picker hands back a "tree URI" (a `content://...` string
// that names a whole subtree of the device's storage). We stash that string in a tiny private
// settings file (SharedPreferences). SEPARATELY, the Activity that launched the picker calls
// `takePersistableUriPermission`, which records a long-lived READ GRANT for that URI so the app can
// still open the folder after a reboot.
//
// The catch: the saved URI string and the read grant can DRIFT apart. The user can revoke the grant
// in system settings, or move/delete the folder, and our saved string is then a dead pointer to a
// folder we can no longer read. So `heldRoot` is the only correct way to read the root back: it
// returns the saved URI only when a live read grant still backs it, and forgets the URI otherwise.
// Handing a dead URI to a scan would throw, and on a headless background service's cold start that
// would take the whole service down.
//
// For a TS reader: think of this whole file as one module that wraps `localStorage` (the
// SharedPreferences) plus a permission check against the OS. There are no classes to instantiate;
// `object LibraryRoot { ... }` (below) is a singleton namespace, like
// `export const LibraryRoot = { save, heldRoot, clear }` in TS.

// What:     `package dev.monochromatic.musicplayer` names the folder/namespace this file lives in.
//           Every other file in this same package can refer to `LibraryRoot` without importing it;
//           files in OTHER packages must write the fully-qualified `dev.monochromatic.musicplayer.
//           LibraryRoot`. The package name MUST mirror the directory path on disk
//           (`dev/monochromatic/musicplayer/`).
// Why:      Gives this code a stable, collision-free home so two different `LibraryRoot` objects in
//           different packages never clash.
//
// In TS you'd write (pseudocode):
// ```ts
// // No keyword for this in TS — the file's location on disk IS its identity.
// // Mentally: this file is `src/dev/monochromatic/musicplayer/LibraryRoot.ts`.
// ```
package dev.monochromatic.musicplayer

// What:     `import android.content.Context` pulls the name `Context` into this file so we can write
//           `Context` instead of the full `android.content.Context`. A `Context` is Android's
//           "handle to the running app/environment": it's how you reach app-wide services like the
//           settings store and the content resolver.
// Why:      Several functions below take a `Context` parameter; without this import the name would be
//           unresolved.
//
// In TS you'd write (pseudocode):
// ```ts
// import { Context } from "android/content";
// ```
import android.content.Context

// What:     `import android.net.Uri` brings in the `Uri` type: Android's parsed representation of a
//           URI string (here a `content://...` SAF tree URI). It is an object, not a bare string.
// Why:      `save` accepts a `Uri`, and `heldRoot` returns one; we compare `Uri` objects for
//           equality below.
//
// In TS you'd write (pseudocode):
// ```ts
// import { Uri } from "android/net"; // like the global `URL` class
// ```
import android.net.Uri

// What:     `import android.util.Log` brings in Android's logging facility. `Log.i(...)` writes an
//           "info" line, `Log.w(...)` writes a "warning" line, both to the device-wide logcat
//           stream you read with `adb logcat`.
// Why:      We log when we save a root and when we discover a remembered root has gone stale, so the
//           on-device verification can trace what happened.
//
// In TS you'd write (pseudocode):
// ```ts
// import { Log } from "android/util"; // Log.i ~ console.info, Log.w ~ console.warn (with a tag)
// ```
import android.util.Log

// What:     `import androidx.core.content.edit` imports an EXTENSION FUNCTION named `edit`. Kotlin
//           lets a library add a method onto an existing type from the outside; `edit` is added onto
//           `SharedPreferences`, so below we can write `prefs(context).edit { ... }` even though the
//           base Android class has no `edit { }` method of its own.
// Why:      It gives us the clean `edit { putString(...) }` block form (begin-edit, mutate,
//           auto-commit) instead of the verbose `.edit()` + `.apply()` dance.
// Gotcha:   This import is what makes the bare token `edit` resolve to a method on a settings object.
//           Importing a *function* (not a type) is unusual to a TS reader; here it's mandatory.
//
// In TS you'd write (pseudocode):
// ```ts
// import { edit } from "androidx/core/content"; // a helper used like prefs.edit(cb)
// ```
import androidx.core.content.edit

// What:     `import androidx.core.net.toUri` imports another EXTENSION FUNCTION, `toUri`, added onto
//           `String`. It parses a plain string into a `Uri` object, so `saved.toUri()` below turns
//           the remembered string back into a structured URI.
// Why:      We persist the root as a string but need a `Uri` object to compare against the live
//           permission list.
//
// In TS you'd write (pseudocode):
// ```ts
// import { toUri } from "androidx/core/net"; // used as saved.toUri(); think `new URL(saved)`
// ```
import androidx.core.net.toUri

// What:     `object LibraryRoot { ... }` declares a SINGLETON. `object` (not `class`) means Kotlin
//           creates exactly one instance, lazily, the first time something accesses it (and that
//           initialization is thread-safe), and `LibraryRoot` is both the type name and that
//           one instance. You never write `new LibraryRoot()`; you call `LibraryRoot.save(...)`
//           directly. Siblings the reader might expect: `class LibraryRoot` (you'd have to
//           instantiate it), or `companion object` (a singleton nested INSIDE a class). We want a
//           top-level, app-wide, stateless namespace, so the plain `object` is right.
// Why:      All the root-remembering logic is pure functions over a settings file; there is no
//           per-instance state, so one shared namespace is exactly what we want.
// Gotcha:   `object` here is NOT JavaScript's `{}` object literal. It is the Kotlin keyword for a
//           singleton constructed on first access (lazy, thread-safe), more like a `namespace` that
//           can hold state.
//
// In TS you'd write (pseudocode):
// ```ts
// export const LibraryRoot = {
//   // save, heldRoot, clear defined below
// };
// ```
/**
 * Defines library root object for this music-player component; the TypeScript-oriented notes above explain its
 * shared role.
 */
object LibraryRoot {
    // What:     `private const val ROOT_TAG: String = "LibraryRoot"` declares a compile-time
    //           constant string. `val` = read-only binding (vs `var` = reassignable). `const` = the
    //           value is known at COMPILE time and inlined at every use site (stricter than a plain
    //           `val`, which is computed at runtime). `private` = visible only inside this `object`.
    //           `: String` is the explicit type (the sibling here is `CharSequence`, a more general
    //           text interface; we want the concrete `String`).
    // Why:      A single source of truth for the logcat tag string so every `Log.i`/`Log.w` call in
    //           this file files its lines under the same searchable label.
    // Gotcha:   `const val` is more than TS `const`: it must be a compile-time literal and gets
    //           inlined. A runtime-computed read-only value would be a plain `val`, not `const val`.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const ROOT_TAG: string = "LibraryRoot";
    // ```
    /**
     * Defines root tag value for this music-player component; the TypeScript-oriented notes above explain its
     * source and use.
     */
    private const val ROOT_TAG: String = "LibraryRoot"

    // What:     `private const val PREFS_NAME: String = "library_root"` is the FILENAME of the
    //           private settings file (SharedPreferences) that stores our one remembered URI. Same
    //           keyword breakdown as above: compile-time constant, read-only, file-local, typed
    //           `String`.
    // Why:      `getSharedPreferences(PREFS_NAME, ...)` needs a stable name so writes and reads hit
    //           the same on-disk file across the Activity and the background service.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const PREFS_NAME: string = "library_root";
    // ```
    /**
     * Defines prefs name value for this music-player component; the TypeScript-oriented notes above explain its
     * source and use.
     */
    private const val PREFS_NAME: String = "library_root"

    // What:     `private const val KEY_TREE_URI: String = "tree_uri"` is the KEY (the entry name)
    //           under which the URI string lives inside that settings file. SharedPreferences is a
    //           key/value store; this is the key.
    // Why:      `putString(KEY_TREE_URI, ...)` and `getString(KEY_TREE_URI, ...)` must agree on one
    //           key, or a save and a read would touch different slots.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const KEY_TREE_URI: string = "tree_uri";
    // ```
    /**
     * Defines key tree uri value for this music-player component; the TypeScript-oriented notes above explain
     * its source and use.
     */
    private const val KEY_TREE_URI: String = "tree_uri"

    // What:     `fun save(context: Context, treeUri: Uri) { ... }` declares a function named `save`
    //           that takes two parameters and returns nothing (no `: ReturnType` after the
    //           parentheses means the return type is `Unit`, Kotlin's "void"). `context: Context` is
    //           the app handle used to reach the settings store; `treeUri: Uri` is the parsed SAF
    //           tree URI to remember. Both are read-only inside the body (Kotlin parameters are
    //           always `val`-like; you cannot reassign them).
    // Why:      This is the public "write" entry point: persist the user's chosen folder so a later
    //           launch can find it.
    // Gotcha:   Kotlin function params are immutable; there is no implicit `let` reassignment of
    //           `context` or `treeUri` inside the body.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // function save(context: Context, treeUri: Uri): void { ... }
    // ```
    /**
     * Defines save behavior for this music-player component; the TypeScript-oriented notes above explain its
     * call shape and effects.
     */
    fun save(context: Context, treeUri: Uri) {
        // What:     `prefs(context).edit { putString(KEY_TREE_URI, treeUri.toString()) }` does three
        //           things on one line:
        //           - `prefs(context)` calls our private helper (bottom of file) to get the
        //             SharedPreferences handle.
        //           - `.edit { ... }` is the imported extension function. The `{ ... }` is a
        //             TRAILING LAMBDA: Kotlin lets you move a function's last argument (here a
        //             callback) OUTSIDE the parentheses into a block. Inside that block, `this` is
        //             the editor, so the bare `putString(...)` is really `editor.putString(...)`.
        //             `edit` auto-commits the changes when the block ends.
        //           - `treeUri.toString()` converts the `Uri` object back into its plain string form
        //             for storage (the store only holds strings).
        // Why:      Write the chosen URI string into the persistent settings file under our key so it
        //           outlives this process.
        // Gotcha:   The `{ ... }` after `edit` is NOT an object literal; it's a lambda body where
        //           `this` is silently rebound to the preferences editor. That implicit-receiver
        //           rebinding has no TS equivalent.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // prefs(context).edit((editor) => {
        //   editor.putString(KEY_TREE_URI, treeUri.toString()); // toString() ~ String(treeUri)
        // });
        // ```
        prefs(context).edit { putString(KEY_TREE_URI, treeUri.toString()) }
        // What:     `Log.i(ROOT_TAG, "saved library root $treeUri")` writes one INFO line to logcat.
        //           `ROOT_TAG` is the filter label. The second argument is a Kotlin STRING TEMPLATE:
        //           `"...$treeUri"` splices the value of `treeUri` into the string (Kotlin calls
        //           `treeUri.toString()` for the `$treeUri` part automatically). `$name` is the
        //           short form; `${expr}` is the long form for full expressions.
        // Why:      Leave a breadcrumb so on-device tracing can confirm the save happened and with
        //           what URI.
        // Gotcha:   `$treeUri` inside a double-quoted Kotlin string is interpolation, not a literal
        //           dollar sign. To print a real `$`, you escape it as `\$`.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // Log.i(ROOT_TAG, `saved library root ${treeUri}`);
        // ```
        Log.i(ROOT_TAG, "saved library root $treeUri")
    }

    // What:     `fun heldRoot(context: Context): Uri? { ... }` declares a function returning `Uri?`.
    //           The trailing `?` makes the type NULLABLE: the result is "a `Uri`, OR `null`". Kotlin
    //           splits every type into non-null (`Uri`) and nullable (`Uri?`) variants, and the
    //           compiler forbids using a `Uri?` as if it were a guaranteed `Uri` until you've
    //           null-checked it. The sibling type is plain `Uri` (never null).
    // Why:      This is the public "read" entry point. It must be able to answer "no remembered,
    //           still-readable root" with `null`, so callers can fall back to the device-wide
    //           MediaStore source instead of scanning a folder they cannot open.
    // Gotcha:   Unlike TS (where any object can be `null` at runtime regardless of its type), Kotlin
    //           ENFORCES the `?`: you literally cannot dereference a `Uri?` without handling the null
    //           case first.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // function heldRoot(context: Context): Uri | null { ... }
    // ```
    /**
     * Defines held root behavior for this music-player component; the TypeScript-oriented notes above explain
     * its call shape and effects.
     */
    fun heldRoot(context: Context): Uri? {
        // What:     `val saved: String = prefs(context).getString(KEY_TREE_URI, null) ?: return null`
        //           reads the remembered string and bails early if absent.
        //           - `getString(KEY_TREE_URI, null)` reads the stored value, returning `null` (the
        //             second argument is the default) when the key is missing. Its return type is
        //             `String?` (nullable).
        //           - `?:` is the ELVIS OPERATOR: `a ?: b` evaluates to `a` when `a` is non-null,
        //             otherwise to `b`. Here `b` is `return null`, so if nothing is stored we leave
        //             the whole function immediately with `null`.
        //           - Because Elvis stripped the null possibility, `saved` is typed as non-null
        //             `String` (the sibling `String?` is what `getString` actually returned).
        // Why:      If no root was ever saved, there is nothing to validate; short-circuit out with
        //           "no root" rather than parsing a null.
        // Gotcha:   `?:` is Kotlin's Elvis (null-coalescing), close to TS `??`, NOT a ternary. The
        //           magic part is that its right side can be a `return`, which TS `??` cannot do.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const maybe = prefs(context).getString(KEY_TREE_URI, null); // string | null
        // if (maybe == null) return null;
        // const saved: string = maybe;
        // ```
        /**
         * Defines saved value for this music-player component; the TypeScript-oriented notes above explain its
         * source and use.
         */
        val saved: String = prefs(context).getString(KEY_TREE_URI, null) ?: return null
        // What:     `val uri: Uri = saved.toUri()` parses the stored string into a `Uri` object.
        //           `.toUri()` is the imported extension function on `String`; it returns a non-null
        //           `Uri`. `uri` is declared with the explicit non-null type `Uri` (sibling: the
        //           nullable `Uri?`, which we do NOT want here because parsing a known string yields
        //           a real object).
        // Why:      We need a structured `Uri` (not a raw string) to compare against the OS's list of
        //           live permissions on the next line.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const uri: Uri = new URL(saved); // toUri() parses the string into a Uri object
        // ```
        /**
         * Defines uri value for this music-player component; the TypeScript-oriented notes above explain its
         * source and use.
         */
        val uri: Uri = saved.toUri()
        // What:     `val held: Boolean = context.contentResolver.persistedUriPermissions.any {
        //           permission -> permission.uri == uri && permission.isReadPermission }` asks the OS
        //           "do we still hold a live READ grant for this exact URI?".
        //           - `context.contentResolver` is the OS object that brokers access to `content://`
        //             URIs.
        //           - `.persistedUriPermissions` is the list of long-lived grants the app currently
        //             holds (this is the SECOND source of truth that can drift from our saved
        //             string).
        //           - `.any { permission -> ... }` is a collection helper that returns `true` if AT
        //             LEAST ONE element matches the predicate. `{ permission -> ... }` is a LAMBDA
        //             (anonymous function); `permission` is its single parameter, and `->` separates
        //             the parameter list from the body (Kotlin's lambda arrow, not a return-type
        //             arrow).
        //           - Inside: `permission.uri == uri` checks the grant is for our URI. Kotlin's `==`
        //             calls `.equals()` (structural/value equality), so it compares URI CONTENTS, not
        //             object identity. `&&` is plain logical AND. `permission.isReadPermission`
        //             ensures it's specifically a READ grant.
        //           - `held` is typed `Boolean` (the sibling would be nullable `Boolean?`; `.any`
        //             always returns a real `true`/`false`, so non-null is correct).
        // Why:      The saved string alone proves nothing; the folder is only usable if a live read
        //           grant still backs it. This line is the drift check at the heart of the file.
        // Gotcha:   Kotlin `==` is VALUE equality (it calls `.equals()`), unlike JS `==`/`===` which
        //           on objects is reference identity. Comparing two `Uri` objects with `==` here
        //           correctly compares their string contents.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const held: boolean = context.contentResolver.persistedUriPermissions.some(
        //   (permission) => permission.uri.equals(uri) && permission.isReadPermission,
        // ); // .any -> .some; Kotlin `==` -> .equals() value comparison
        // ```
        /**
         * Defines held value for this music-player component; the TypeScript-oriented notes above explain its
         * source and use.
         */
        val held: Boolean = context.contentResolver.persistedUriPermissions.any { permission ->
            permission.uri == uri && permission.isReadPermission
        }
        // What:     `if (!held) { ... }` runs the block only when no live read grant was found.
        //           `!held` is plain logical NOT on a `Boolean`. Nothing exotic here; this is an
        //           ordinary `if` with a braced body.
        // Why:      A remembered URI with no backing grant is a dead pointer; we must not return it.
        //           This branch handles the "stale root" case.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (!held) { ... }
        // ```
        if (!held) {
            // What:     `Log.w(ROOT_TAG, "remembered root $uri has no live read grant; forgetting
            //           it")` writes a WARNING line to logcat. `Log.w` is the warning-level sibling
            //           of `Log.i`. `$uri` is a string-template interpolation of the `Uri` (its
            //           `.toString()` is called automatically).
            // Why:      Record WHY we are about to forget a remembered root, so the on-device trace
            //           explains the fallback to the device-wide source.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // Log.w(ROOT_TAG, `remembered root ${uri} has no live read grant; forgetting it`);
            // ```
            Log.w(ROOT_TAG, "remembered root $uri has no live read grant; forgetting it")
            // What:     `clear(context)` calls the sibling function below to delete the stale entry
            //           from the settings file. Plain function call, no special syntax.
            // Why:      Forget the dead pointer so future reads short-circuit at the very first step
            //           (the `?: return null` above) instead of re-failing this grant check every
            //           time.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // clear(context);
            // ```
            clear(context)
            // What:     `return null` exits `heldRoot` with `null`, signalling "no usable remembered
            //           root". `null` is the explicit empty value; it satisfies the function's
            //           nullable `Uri?` return type.
            // Why:      Tell the caller there is nothing live to scan, so it can fall back to the
            //           device-wide MediaStore source.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // return null;
            // ```
            return null
        }
        // What:     `return uri` is the success path: we reached here only when `held` was true, so
        //           the remembered URI is live and readable. We hand back the non-null `Uri`. This is
        //           an explicit `return` (Kotlin also allows tail-expression returns, but here it's
        //           spelled out). The value `uri` is a non-null `Uri`, which fits the `Uri?` return
        //           type (a non-null value is always a valid nullable value).
        // Why:      Give the caller the validated, still-openable library root.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return uri;
        // ```
        return uri
    }

    // What:     `fun clear(context: Context) { ... }` declares a function named `clear` taking one
    //           `Context` and returning `Unit` (void). `context: Context` is the app handle used to
    //           reach the settings store.
    // Why:      Public "forget" entry point: drop the remembered root so the next load falls back to
    //           the device-wide source. Also called internally by `heldRoot` when it detects a stale
    //           grant.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // function clear(context: Context): void { ... }
    // ```
    /** Reads the saved root's file in full, off the main thread, so a later main-thread read finds it in memory. */
    fun warmUp(context: Context) {
        Log.i(ROOT_TAG, "warm-up read ${prefs(context).all.size} saved values")
    }

    /**
     * Defines clear behavior for this music-player component; the TypeScript-oriented notes above explain its
     * call shape and effects.
     */
    fun clear(context: Context) {
        // What:     `prefs(context).edit { remove(KEY_TREE_URI) }` opens the settings editor and
        //           deletes our key. Same machinery as `save`: `prefs(context)` fetches the handle,
        //           `.edit { ... }` is the imported extension with a TRAILING LAMBDA whose `this` is
        //           the editor (so bare `remove(...)` is `editor.remove(...)`), and the change
        //           auto-commits when the block ends. `remove(KEY_TREE_URI)` erases the stored URI
        //           string.
        // Why:      Physically remove the dead/unwanted entry so subsequent reads find nothing and
        //           short-circuit to the fallback source.
        // Gotcha:   Same implicit-receiver rebinding as in `save`: the `{ ... }` is a lambda where
        //           `this` is the editor, not an object literal.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // prefs(context).edit((editor) => { editor.remove(KEY_TREE_URI); });
        // ```
        prefs(context).edit { remove(KEY_TREE_URI) }
    }

    // What:     `private fun prefs(context: Context) = context.getSharedPreferences(PREFS_NAME,
    //           Context.MODE_PRIVATE)` declares a private helper with an EXPRESSION BODY: the `=`
    //           form means "this function returns the value of the single expression on the right",
    //           with no `{ }` block and no `return`. There is no written return type; Kotlin INFERS
    //           it from the expression (here `SharedPreferences`).
    //           - `context.getSharedPreferences(PREFS_NAME, Context.MODE_PRIVATE)` opens (or creates)
    //             the named private settings file. `Context.MODE_PRIVATE` is a constant flag meaning
    //             "readable/writable only by this app".
    // Why:      One place that knows the file name and access mode, so `save`, `heldRoot`, and
    //           `clear` all agree on which file they touch. The settings file is process-wide, so the
    //           Activity's write is visible to the background service's read.
    // Gotcha:   The `=` (instead of `{ return ... }`) is Kotlin's single-expression function form;
    //           the return type is inferred, not written. A TS reader should not look for an explicit
    //           return type here.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const prefs = (context: Context) =>
    //   context.getSharedPreferences(PREFS_NAME, Context.MODE_PRIVATE);
    // ```
    /**
     * Defines prefs behavior for this music-player component; the TypeScript-oriented notes above explain its
     * call shape and effects.
     */
    private fun prefs(context: Context) =
        context.applicationContext.getSharedPreferences(PREFS_NAME, Context.MODE_PRIVATE)
}
