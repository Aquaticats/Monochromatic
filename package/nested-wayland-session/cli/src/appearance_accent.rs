//! Accent color switching on the private appearance portal.
//!
//! This module belongs to `appearance_portal` and shares its private state:
//!  the cell the Settings interface reads,
//!  and the connection that owns the portal name.

/// Imports `Ordering` from `std::sync::atomic`.
use std::sync::atomic::Ordering;

/// Imports `Context`,
///  `Result` from `anyhow`.
use anyhow::{Context, Result};

/// Imports the items this module uses from `super`.
use super::{
    accent_variant, AppearancePortal, SwitchOutcome, ACCENT_COLOR_KEY, APPEARANCE_NAMESPACE, NOT_SERVED,
    PORTAL_OBJECT_PATH, SETTINGS_INTERFACE, SETTING_CHANGED_SIGNAL,
};
/// Imports `AccentColor` from `crate::accent_color`.
use crate::accent_color::AccentColor;

/// Switching the accent color a hosted toolkit reads.
impl AppearancePortal {
    /// Tells whether `--accent-color` was given,
    ///  so the `accent-color` verb has a value to switch.
    #[must_use]
    pub fn serves_accent_color(&self) -> bool {
        return self.served_accent.load(Ordering::SeqCst) != NOT_SERVED;
    }

    /// Serves `color` from now on and notifies subscribed clients on the private bus.
    ///
    /// What:
    ///      `pub fn set_accent_color(&self, color: AccentColor) -> Result<SwitchOutcome>`.
    ///  The channels are compared exactly;
    ///  the same color answers `Unchanged` and emits nothing.
    ///  The signal body is `(s, s, v)` with a `(ddd)` structure inside the variant,
    ///  the shape Slint decodes for `org.freedesktop.appearance` `accent-color`.
    /// Why:
    ///       The portal defines `SettingChanged` as emitted when a setting changes,
    ///  and a refused change must leave the served value as it was.
    pub fn set_accent_color(&self, color: AccentColor) -> Result<SwitchOutcome> {
        if !self.serves_accent_color() {
            anyhow::bail!("the private appearance portal was started without --accent-color");
        }
        let packed = color.pack();
        let previous = self.served_accent.swap(packed, Ordering::SeqCst);
        if previous == packed {
            tracing::debug!(%color, "accent color already served; no signal emitted");
            return Ok(SwitchOutcome::Unchanged);
        }
        let emitted = self.connection.emit_signal(
            None::<&str>,
            PORTAL_OBJECT_PATH,
            SETTINGS_INTERFACE,
            SETTING_CHANGED_SIGNAL,
            &(APPEARANCE_NAMESPACE, ACCENT_COLOR_KEY, accent_variant(color)),
        );
        // A change nobody was told about is not a change: restore the old color so a retry emits again.
        if let Err(error) = emitted {
            self.served_accent.store(previous, Ordering::SeqCst);
            return Err(error).context("emitting SettingChanged for the accent color on the private appearance bus");
        }
        tracing::info!(%color, "switched isolated accent color and emitted SettingChanged");
        return Ok(SwitchOutcome::Changed);
    }
}
