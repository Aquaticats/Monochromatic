//! Presents the chooser for a single URL or local file supplied by the desktop.
//!
//! Read `target` for input resolution, `profiles` for browser metadata, and `browsers` for dispatch.

/// Discovers browser applications and launches the selected row.
mod browsers;
/// Reads profile metadata and builds profile-selection arguments.
mod profiles;
/// Validates and formats the supplied link or file.
mod target;
/// Exercises URI handling and discovery with disposable browser fixtures.
#[cfg(test)]
mod tests;

/// Imports Slint's window operations and model types for the list of labels.
use slint::{ComponentHandle, ModelRc, SharedString, VecModel};

/// Slint emits implicit returns and shadowing in generated Rust. Keep the exemptions inside
/// this module so the maintained source follows the crate lints.
/// See `doc/troubleshooting/slint-generated-rust-implicit-return.md` for the include boundary.
#[allow(clippy::implicit_return, clippy::shadow_reuse, clippy::shadow_same, clippy::shadow_unrelated)]
mod slint_generated {
    slint::include_modules!();
}

/// Imports the typed picker window from the bindings compiled from the markup.
use slint_generated::Picker;

/// What: Shows a Slint picker with callbacks bound to the supplied open request.
/// Why: Every desktop invocation gives the user a choice, even for a single installed browser.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function run(): void;
/// ```
fn run() -> anyhow::Result<()> {
    // Read the OS-native input so paths do not need to decode as UTF-8.
    let mut arguments = std::env::args_os().skip(1);
    let first = arguments.next();
    if first.as_deref() == Some(std::ffi::OsStr::new("--help")) {
        println!("Usage: monochromatic-browser-selector URL-or-file");
        return Ok(());
    }

    // A Slint platform error may contain platform-specific data, so keep its user-facing text.
    let picker = Picker::new().map_err(|error| anyhow::anyhow!("{error}"))?;
    let mut choices = Vec::new();
    let mut uri = String::new();

    if arguments.next().is_some() {
        picker.set_message("Open one link or local file per request.".into());
    } else if let Some(input) = first {
        match target::parse(&input) {
            Ok(incoming) => {
                picker.set_target(incoming.display.into());
                uri = incoming.uri;
                choices = browsers::discover();
                if choices.is_empty() {
                    picker.set_message("No browsers are registered on this desktop.".into());
                }
            }
            Err(error) => picker.set_message(error.to_string().into()),
        }
    } else {
        picker.set_message("Open a link or HTML file to choose a browser.".into());
    }

    // Copy just the labels into the Slint model. The callback owns the matching routes.
    let labels = VecModel::from_iter(choices.iter().map(|choice| {
        return SharedString::from(choice.label.as_str());
    }));
    picker.set_choices(ModelRc::new(labels));
    // Hold a weak window handle so the callback cannot keep the window alive on its own.
    let window = picker.as_weak();
    picker.on_open_choice(move |index| {
        let Ok(row) = usize::try_from(index) else { return; };
        let Some(choice) = choices.get(row) else { return; };
        if let Err(error) = choice.open(&uri) {
            if let Some(picker_window) = window.upgrade() {
                picker_window.set_message(format!("Could not open {}. {error}", choice.label).into());
            }
            return;
        }
        if let Some(picker_window) = window.upgrade()
            && let Err(error) = picker_window.hide()
        {
            picker_window.set_message(error.to_string().into());
        }
    });
    let cancel_window = picker.as_weak();
    picker.on_cancel(move || {
        if let Some(picker_window) = cancel_window.upgrade()
            && let Err(error) = picker_window.hide()
        {
            picker_window.set_message(error.to_string().into());
        }
    });
    picker.run().map_err(|error| anyhow::anyhow!("{error}"))?;
    return Ok(());
}

/// What: The operating system starts this function when it runs the selector.
/// Why: The CLI should report a failure if the picker cannot start at all.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// run(); // top-level failure exits with a diagnostic
/// ```
fn main() -> anyhow::Result<()> {
    return run();
}
