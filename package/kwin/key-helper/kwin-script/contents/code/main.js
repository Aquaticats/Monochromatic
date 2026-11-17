// KWin script: the KWin-side companion to the key-helper daemon.
//
// This half sees window/key events and owns the window objects; the daemon half
// (org.monochromatic.KeyHelper) does the actuation KWin scripts cannot: spawning
// processes, injecting keys via ydotool, and talking to Neovim over RPC.
//
// The DBUS_* constants below MUST match src/constants.ts in this package. The
// Plasma JavaScript runtime cannot import TypeScript, so they are duplicated.
//
// Responsibilities:
//   - Active window tracking: report the focused window class on every change so
//     the daemon knows whether Neovide is focused (double-shift scoping). The
//     double-shift itself is detected by the daemon via evdev (passive, doesn't
//     consume events).
//   - Ctrl+F4 -> Ctrl+W remap for browsers, and <F16> for Neovide.
//   - Meta+N: launch a new instance of the focused app and focus the new window.

var DBUS_SERVICE = "org.monochromatic.KeyHelper";
var DBUS_PATH    = "/org/monochromatic/KeyHelper";
var DBUS_IFACE   = "org.monochromatic.KeyHelper";

// --- Active window tracking for double-shift scoping ---
// Report window class on every focus change so the daemon knows whether to fire
// F20 when it detects a double-shift via evdev.

workspace.windowActivated.connect(function(client) {
    var cls = client ? client.resourceClass : "";
    callDBus(DBUS_SERVICE, DBUS_PATH, DBUS_IFACE, "SetActiveWindow", cls);
});

// --- Ctrl+F4: Close tab in browsers and Neovide ---
// Browsers: remap to Ctrl+W via ydotool.
// Neovide: send <F16> to Neovim via msgpack-rpc (Neovide swallows Ctrl+F4).

// Helium (Chromium-based) reports the window class "helium" on Plasma 6.8 Wayland,
// which none of the other names match.
var BROWSERS = [
    "librewolf", "firefox", "chrome", "chromium",
    "brave", "brave-browser", "edge", "vivaldi", "opera", "helium"
];

registerShortcut("BrowserCloseTab", "Close Tab (Ctrl+F4)", "Ctrl+F4", function() {
    var w = workspace.activeWindow;
    if (!w) return;
    var cls = w.resourceClass.toLowerCase();

    if (cls === "neovide") {
        callDBus(DBUS_SERVICE, DBUS_PATH, DBUS_IFACE, "SendNvimKeys", "<F16>");
        return;
    }

    for (var i = 0; i < BROWSERS.length; i++) {
        if (cls === BROWSERS[i] || cls.indexOf(BROWSERS[i]) !== -1) {
            callDBus(DBUS_SERVICE, DBUS_PATH, DBUS_IFACE, "SendKeys", "ctrl+w");
            return;
        }
    }
});

// --- Meta+N: New instance of the current app ---
// Read the focused window's app identity and ask the daemon to launch another
// instance (KWin scripts cannot spawn processes). Focusing the new window is done
// here, since we own the window objects: remember the launched app class briefly,
// then activate the next matching window that appears. This guards against
// focus-stealing prevention leaving the new window unfocused.

var newInstancePendingClass = "";
var newInstancePendingUntil = 0;
var NEW_INSTANCE_FOCUS_WINDOW_MS = 4000;

registerShortcut("NewInstanceOfCurrentApp", "New instance of current app (Meta+N)", "Meta+N", function() {
    var w = workspace.activeWindow;
    if (!w) return;
    var cls = w.resourceClass || "";
    var desktopFile = w.desktopFileName || "";
    newInstancePendingClass = cls.toLowerCase();
    newInstancePendingUntil = Date.now() + NEW_INSTANCE_FOCUS_WINDOW_MS;
    callDBus(DBUS_SERVICE, DBUS_PATH, DBUS_IFACE, "LaunchNewInstance", desktopFile, cls);
});

workspace.windowAdded.connect(function(client) {
    if (!client || !newInstancePendingClass) return;
    if (Date.now() > newInstancePendingUntil) { newInstancePendingClass = ""; return; }
    var cls = (client.resourceClass || "").toLowerCase();
    if (cls === newInstancePendingClass) {
        newInstancePendingClass = "";
        workspace.activeWindow = client;
    }
});
