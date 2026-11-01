//! Render actual toolkit Text items to verify variable-weight and italic cache invalidation.

/// Windows are drawn on the hardware GPU,
///  so the platform is installed before the thread's first window.
use super::gpu_snapshot::install_gpu_platform;
/// Public toolkit snapshot and event-loop update APIs exercise the native text renderer.
use slint::{ComponentHandle, SharedString, platform::update_timers_and_animations};

// The fixture is test-only and does not add font controls to the application interface.
slint::slint! {
    import "../../asset/font/InterVariable.ttf";
    import "../../asset/font/InterVariable-Italic.ttf";
    import "../../asset/font/JetBrainsMono-Variable.ttf";
    import "../../asset/font/JetBrainsMono-VariableItalic.ttf";
    export component FontProbe inherits Window {
        preferred-width: 600px;
        preferred-height: 100px;
        background: white;
        in property <string> family: "Inter Variable";
        in property <int> weight: 400;
        in property <bool> italic: false;
        Text {
            x: 12px;
            y: 12px;
            text: "affine office === != 012345";
            color: black;
            font-family: root.family;
            font-weight: root.weight;
            font-italic: root.italic;
            font-size: 28px;
        }
    }
}

/// Capture rendered pixels only after processing the pending property changes.
fn snapshot(window: &FontProbe) -> Vec<u8> {
    update_timers_and_animations();
    return window
        .window()
        .take_snapshot()
        .expect("native font snapshot")
        .as_bytes()
        .to_vec();
}

/// Weight/style requests repaint both bundled families;
///  exact face provenance has separate font-stack tests.
#[test]
fn native_text_repaints_for_weight_and_italic_requests() {
    install_gpu_platform();
    let window = FontProbe::new().expect("headless typography fixture");
    window.show().expect("show typography fixture");
    window.set_family(SharedString::from("IDE unregistered font control"));
    let fallback = snapshot(&window);
    for family in ["Inter Variable", "JetBrains Mono"] {
        window.set_family(SharedString::from(family));
        window.set_weight(400);
        window.set_italic(false);
        let regular = snapshot(&window);
        assert!(
            regular != fallback,
            "{family} must differ from the unregistered-family control"
        );
        window.set_weight(537);
        let intermediate = snapshot(&window);
        assert_ne!(
            regular, intermediate,
            "variable weight must affect native {family} pixels"
        );
        window.set_italic(true);
        let italic = snapshot(&window);
        assert_ne!(
            intermediate, italic,
            "italic request must affect native {family} pixels"
        );
        window.set_italic(false);
        window.set_weight(400);
        assert_eq!(
            snapshot(&window),
            regular,
            "font cache must restore {family} defaults"
        );
    }
    window.hide().expect("close typography fixture");
}
