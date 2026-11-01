//! Debug-only visual labels for every layout region that matters while tuning scroll behavior.
//!
//! `FM_DEBUG_TINT=1` turns structural GTK regions into explicitly named screenshot artifacts.
//!  Each
//! visible badge carries a fresh three-character code plus a plain-English description,
//!  so a user can
//! point at a screenshot and name the exact widget,
//!  lane,
//!  or scroll surface that looks wrong.

/// What:
///  imports GTK widget-extension traits and `IsA` bounds.
/// Why:
///  helpers accept any widget subtype while still calling shared widget methods.
use gtk4::prelude::*;
/// What:
///  imports concrete GTK widgets used to build badges and overlay wrappers.
/// Why:
///  debug annotations are real GTK children so they appear in screenshots.
use gtk4::{Align, Box as GtkBox, Label, Orientation, Overlay, Widget};

/// What:
///  imports the debug-tint env-var name.
/// Why:
///  labels and wrappers only appear in debug-tint runs.
use crate::constants::DEBUG_TINT_ENV;

/// What:
///  margin around a floating debug badge.
/// Why:
///  keeping the badge off the exact border leaves outlines visible underneath it.
const BADGE_MARGIN: i32 = 4;

/// What:
///  small vertical gap between labels inside a debug overlay region.
/// Why:
///  multiple badges inside one rail stay readable without consuming production layout space.
const BADGE_GAP: i32 = 2;

/// What:
///  named debug region with a screenshot code and human description.
/// Why:
///  the code is short enough to read from a screenshot;
///  the description explains the GTK role.
#[derive(Clone, Copy)]
pub(crate) struct DebugRegion {
    /// Three-character screenshot code.
    pub(crate) code: &'static str,
    /// Human-readable region role.
    pub(crate) description: &'static str,
}

/// What:
///  fixed canvas region inside a static column.
/// Why:
///  names the content surface where pane widgets are positioned by row and lane offset.
pub(crate) const V6C_COLUMN_CANVAS: DebugRegion = DebugRegion {
    code: "V6C",
    description: "column fixed pane canvas",
};

/// What:
///  immediate-child lane overlay region.
/// Why:
///  names the abstract lane shared by panes spawned from the same parent.
pub(crate) const Y6L_CHILD_LANE: DebugRegion = DebugRegion {
    code: "Y6L",
    description: "immediate-child shared lane",
};

/// What:
///  preview-body region.
/// Why:
///  names the non-directory preview area inside a pane shell.
pub(crate) const B6P_PREVIEW_BODY: DebugRegion = DebugRegion {
    code: "B6P",
    description: "preview pane body",
};

/// What:
///  report whether debug tinting is active for this process.
/// Why:
///  production runs should not allocate labels or alter widget wrappers.
pub(crate) fn enabled() -> bool {
    return std::env::var_os(DEBUG_TINT_ENV).is_some()
}

/// What:
///  add tooltip metadata to `widget` when debug tinting is active.
/// Why:
///  screenshots show visible badges,
///  while tooltips provide the same code on hover.
pub(crate) fn tag(widget: &impl IsA<Widget>, region: DebugRegion, detail: Option<&str>) {
    if !enabled() {
        return;
    }
    widget.set_tooltip_text(Some(&label_text(region, detail)));
}

/// What:
///  wrap `child` in a `GtkOverlay` with a top-left badge when debug tinting is active.
/// Why:
///  debug regions can carry labels without consuming layout space or changing child size.
pub(crate) fn wrap(child: &impl IsA<Widget>, region: DebugRegion, detail: Option<&str>) -> Widget {
    if !enabled() {
        return child.as_ref().clone();
    }
    let overlay = Overlay::new();
    overlay.set_hexpand(child.as_ref().hexpands());
    overlay.set_vexpand(child.as_ref().vexpands());
    overlay.set_child(Some(child));
    overlay.add_css_class("fm-debug-overlay");
    overlay.add_overlay(&badge(region, detail));
    tag(&overlay, region, detail);
    return overlay.upcast::<Widget>()
}

/// What:
///  build a fixed-size abstract lane overlay.
/// Why:
///  immediate-child lane rail is not a natural GTK widget,
///  so debug mode draws one explicitly.
pub(crate) fn lane(region: DebugRegion, detail: Option<&str>, width: i32, height: i32) -> Widget {
    let rail = GtkBox::new(Orientation::Vertical, BADGE_GAP);
    rail.set_size_request(width, height);
    rail.set_can_target(false);
    rail.set_can_focus(false);
    rail.add_css_class("fm-debug-lane");
    rail.append(&inline_badge(region, detail));
    tag(&rail, region, detail);
    return rail.upcast::<Widget>()
}

/// What:
///  build a floating badge for overlay wrappers.
/// Why:
///  every overlay label shares the same alignment,
///  margin,
///  and pointer transparency.
fn badge(region: DebugRegion, detail: Option<&str>) -> Label {
    let label = base_badge(region, detail);
    label.set_halign(Align::Start);
    label.set_valign(Align::Start);
    label.set_margin_start(BADGE_MARGIN);
    label.set_margin_top(BADGE_MARGIN);
    return label
}

/// What:
///  build an inline badge for box children and lane rails.
/// Why:
///  inline badges should not request overlay alignment,
///  only text styling.
fn inline_badge(region: DebugRegion, detail: Option<&str>) -> Label {
    return base_badge(region, detail)
}

/// What:
///  build the shared label widget for a region code and description.
/// Why:
///  all debug text should use one CSS class and stay mouse-transparent.
fn base_badge(region: DebugRegion, detail: Option<&str>) -> Label {
    let label = Label::new(Some(&label_text(region, detail)));
    label.set_can_target(false);
    label.set_can_focus(false);
    label.add_css_class("fm-debug-badge");
    return label
}

/// What:
///  turn a debug region plus optional runtime detail into visible badge text.
/// Why:
///  the stable code identifies the region kind,
///  while details distinguish columns and panes.
fn label_text(region: DebugRegion, detail: Option<&str>) -> String {
    let prefix = format!("{} {}", region.code, region.description);
    if let Some(detail_text) = detail {
        return format!("{prefix} {detail_text}");
    }
    return prefix
}
