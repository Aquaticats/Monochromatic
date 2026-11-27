//! The AT-SPI role and state names the `a11y` answer prints,
//!  and the two lookups that turn the numbers an application answers with into them.
//!
//! Split out of `a11y_tree.rs` to keep that file under the code-line limit./// AT-SPI state bit numbers this reader names.
///
/// What:
///      `const STATE_NAMES: &[(u32, &str)]`.
/// Why:
///       The state vector is a 64-bit set;
///  a test wants `focused` and `checked`,
///  not bit numbers.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const STATE_NAMES: [number, string][] = [[4, "checked"], [8, "enabled"], ...];
/// ```
const STATE_NAMES: &[(u32, &str)] = &[
    (4, "checked"),
    (5, "collapsed"),
    (7, "editable"),
    (8, "enabled"),
    (10, "expanded"),
    (11, "focusable"),
    (12, "focused"),
    (20, "pressed"),
    (23, "selected"),
    (24, "sensitive"),
    (25, "showing"),
    (30, "visible"),
];

/// AT-SPI role numbers with their role names,
///  from the `Role` enumeration of the AT-SPI specification.
///
/// What:
///      `const ROLE_NAMES: &[&str]`.
///  The index is the role number.
/// Why:
///       Applications answer `GetRole` with the number only,
///  and a test wants `push button`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ROLE_NAMES = ["invalid", "accelerator label", "alert", ...];
/// ```
const ROLE_NAMES: &[&str] = &[
    "invalid",
    "accelerator label",
    "alert",
    "animation",
    "arrow",
    "calendar",
    "canvas",
    "check box",
    "check menu item",
    "color chooser",
    "column header",
    "combo box",
    "date editor",
    "desktop icon",
    "desktop frame",
    "dial",
    "dialog",
    "directory pane",
    "drawing area",
    "file chooser",
    "filler",
    "focus traversable",
    "font chooser",
    "frame",
    "glass pane",
    "html container",
    "icon",
    "image",
    "internal frame",
    "label",
    "layered pane",
    "list",
    "list item",
    "menu",
    "menu bar",
    "menu item",
    "option pane",
    "page tab",
    "page tab list",
    "panel",
    "password text",
    "popup menu",
    "progress bar",
    "push button",
    "radio button",
    "radio menu item",
    "root pane",
    "row header",
    "scroll bar",
    "scroll pane",
    "separator",
    "slider",
    "spin button",
    "split pane",
    "status bar",
    "table",
    "table cell",
    "table column header",
    "table row header",
    "tearoff menu item",
    "terminal",
    "text",
    "toggle button",
    "tool bar",
    "tool tip",
    "tree",
    "tree table",
    "unknown",
    "viewport",
    "window",
    "extended",
    "header",
    "footer",
    "paragraph",
    "ruler",
    "application",
    "autocomplete",
    "edit bar",
    "embedded",
    "entry",
    "chart",
    "caption",
    "document frame",
    "heading",
    "page",
    "section",
    "redundant object",
    "form",
    "link",
    "input method window",
    "table row",
    "tree item",
    "document spreadsheet",
    "document presentation",
    "document text",
    "document web",
    "document email",
    "comment",
    "list box",
    "grouping",
    "image map",
    "notification",
    "info bar",
    "level bar",
    "title bar",
    "block quote",
    "audio",
    "video",
    "definition",
    "article",
    "landmark",
    "log",
    "marquee",
    "math",
    "rating",
    "timer",
    "static",
    "math fraction",
    "math root",
    "subscript",
    "superscript",
    "description list",
    "description term",
    "description value",
    "footnote",
    "content deletion",
    "content insertion",
    "mark",
    "suggestion",
    "push button menu",
];

/// Name an AT-SPI role number.
///
/// What:
///      `pub fn role_name(role: u32) -> String`.
///  `role-N` for a number the table does not know.
/// Why:
///       Newer specification revisions add roles;
///  an unknown one must still print.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const roleName = (role: number) => ROLE_NAMES[role] ?? `role-${role}`;
/// ```
#[must_use]
pub fn role_name(role: u32) -> String {
    match ROLE_NAMES.get(role as usize) {
        Some(name) => return (*name).to_string(),
        None => return format!("role-{role}"),
    }
}

/// Name the set bits of an AT-SPI state vector.
///
/// What:
///      `pub fn state_names(words: &[u32]) -> Vec<&'static str>`.
///  Bit `n` is bit `n % 32` of word `n / 32`.
/// Why:
///       Decoding is pure and tested without a bus.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function stateNames(words: number[]): string[] { ... }
/// ```
#[must_use]
pub fn state_names(words: &[u32]) -> Vec<&'static str> {
    let mut names = Vec::new();
    for (bit, name) in STATE_NAMES {
        let word = words.get((bit / 32) as usize).copied().unwrap_or(0);
        if word & (1 << (bit % 32)) != 0 {
            names.push(*name);
        }
    }
    return names;
}
