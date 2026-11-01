//! GPU choice regressions independent of any device.

use std::path::PathBuf;

use super::*;

/// The render nodes of the host this feature was built on,
/// in the order EGL enumerates them (not path order).
fn two_nodes() -> Vec<PathBuf> {
    return vec![PathBuf::from("/dev/dri/renderD129"), PathBuf::from("/dev/dri/renderD128")]
}

/// `auto`,
/// `software`,
/// and an absolute path parse;
/// anything else names the accepted forms and the listing option.
#[test]
fn parse_accepts_the_three_forms_and_rejects_the_rest() {
    assert_eq!(GpuChoice::parse("auto"), Ok(GpuChoice::Auto));
    assert_eq!(GpuChoice::parse("software"), Ok(GpuChoice::Software));
    assert_eq!(
        GpuChoice::parse("/dev/dri/renderD128"),
        Ok(GpuChoice::RenderNode(PathBuf::from("/dev/dri/renderD128"))),
    );
    for bad in ["amd", "renderD128", "dri/renderD128", "", "Auto", "0"] {
        let message = GpuChoice::parse(bad).unwrap_err();
        assert!(message.contains("auto, software"), "{bad}: {message}");
        assert!(message.contains("--list-gpus"), "{bad}: {message}");
    }
}

/// Only a named device is explicit.
#[test]
fn only_auto_is_not_explicit() {
    assert!(!GpuChoice::Auto.is_explicit());
    assert!(GpuChoice::Software.is_explicit());
    assert!(GpuChoice::RenderNode(PathBuf::from("/dev/dri/renderD128")).is_explicit());
    assert_eq!(GpuChoice::default(), GpuChoice::Auto);
}

/// `auto` tries Mesa's default,
/// then every render node in path order,
/// then software.
#[test]
fn auto_plan_is_default_then_nodes_in_path_order_then_software() {
    assert_eq!(
        plan(&GpuChoice::Auto, &two_nodes()),
        vec![
            Attempt::MesaDefault,
            Attempt::RenderNode(PathBuf::from("/dev/dri/renderD128")),
            Attempt::RenderNode(PathBuf::from("/dev/dri/renderD129")),
            Attempt::Software,
        ],
    );
    assert_eq!(plan(&GpuChoice::Auto, &[]), vec![Attempt::MesaDefault, Attempt::Software]);
}

/// An explicit choice is exactly one attempt,
/// so nothing can fall back.
#[test]
fn explicit_plans_are_a_single_attempt() {
    let node = PathBuf::from("/dev/dri/renderD128");
    assert_eq!(plan(&GpuChoice::Software, &two_nodes()), vec![Attempt::Software]);
    assert_eq!(plan(&GpuChoice::RenderNode(node.clone()), &two_nodes()), vec![Attempt::RenderNode(node)]);
}

/// Opening outcomes by attempt,
/// as a fake device layer.
fn outcomes(
    default: Result<&'static str, &'static str>,
    nodes: [Result<&'static str, &'static str>; 2],
    software: Result<&'static str, &'static str>,
) -> impl FnMut(&Attempt) -> Result<&'static str, String> {
    return move |attempt| {
        let outcome = match attempt {
            Attempt::MesaDefault => default,
            Attempt::RenderNode(path) if path.ends_with("renderD128") => nodes[0],
            Attempt::RenderNode(_) => nodes[1],
            Attempt::Software => software,
        };
        return outcome.map_err(str::to_string)
    }
}

/// A working default ends the search and opens nothing else.
#[test]
fn default_ok_stops_the_search() {
    let mut opened = Vec::new();
    let mut inner = outcomes(Ok("default"), [Ok("d128"), Ok("d129")], Ok("software"));
    let selected = select(&plan(&GpuChoice::Auto, &two_nodes()), |attempt| {
        opened.push(attempt.clone());
        return inner(attempt)
    })
    .expect("a working default device is selected");
    assert_eq!(selected.attempt, Attempt::MesaDefault);
    assert_eq!(selected.value, "default");
    assert!(selected.failures.is_empty());
    assert_eq!(opened, vec![Attempt::MesaDefault]);
}

/// A failed default falls to the first render node that works,
/// and the failure is kept.
#[test]
fn default_fails_and_another_node_works() {
    let selected = select(
        &plan(&GpuChoice::Auto, &two_nodes()),
        outcomes(Err("permission denied"), [Err("no such device"), Ok("d129")], Ok("software")),
    )
    .expect("the first render node that opens is selected after the default fails");
    assert_eq!(selected.attempt, Attempt::RenderNode(PathBuf::from("/dev/dri/renderD129")));
    assert_eq!(selected.value, "d129");
    assert_eq!(
        selected.failures,
        vec![
            Failure { attempt: Attempt::MesaDefault, reason: "permission denied".to_string() },
            Failure {
                attempt: Attempt::RenderNode(PathBuf::from("/dev/dri/renderD128")),
                reason: "no such device".to_string(),
            },
        ],
    );
}

/// Every GPU failing leaves software,
/// and the session reports a fallback.
#[test]
fn all_gpus_fail_then_software() {
    let selected = select(
        &plan(&GpuChoice::Auto, &two_nodes()),
        outcomes(Err("a"), [Err("b"), Err("c")], Ok("software")),
    )
    .expect("the software device is selected after every GPU fails");
    assert_eq!(selected.attempt, Attempt::Software);
    assert_eq!(selected.failures.len(), 3);
}

/// With software unavailable too,
/// the error lists every attempt and its reason,
/// and says an explicit choice never falls back.
#[test]
fn nothing_usable_lists_everything_tried() {
    let failures = select(
        &plan(&GpuChoice::Auto, &two_nodes()),
        outcomes(Err("default broke"), [Err("d128 broke"), Err("d129 broke")], Err("no llvmpipe")),
    )
    .unwrap_err();
    assert_eq!(failures.len(), 4);
    let message = failure_message(&GpuChoice::Auto, &failures, "Run --list-gpus.");
    for expected in [
        "Mesa's default render node: default broke",
        "render node /dev/dri/renderD128: d128 broke",
        "render node /dev/dri/renderD129: d129 broke",
        "the software renderer (llvmpipe): no llvmpipe",
        "never falls back",
        "Run --list-gpus.",
    ] {
        assert!(message.contains(expected), "{expected} in {message}");
    }
}

/// An explicit choice that fails stops with that device's reason,
/// even when another device would have worked.
#[test]
fn explicit_choice_never_falls_back() {
    let node = GpuChoice::RenderNode(PathBuf::from("/dev/dri/renderD128"));
    let mut opened = Vec::new();
    let mut inner = outcomes(Ok("default"), [Err("cannot open the node"), Ok("d129")], Ok("software"));
    let failures = select(&plan(&node, &two_nodes()), |attempt| {
        opened.push(attempt.clone());
        return inner(attempt)
    })
    .unwrap_err();
    assert_eq!(opened, vec![Attempt::RenderNode(PathBuf::from("/dev/dri/renderD128"))]);
    let message = failure_message(&node, &failures, "Render nodes: renderD128, renderD129.");
    assert!(message.starts_with("--gpu /dev/dri/renderD128 cannot be used: cannot open the node"), "{message}");
    assert!(message.contains("Render nodes: renderD128, renderD129."), "{message}");
    assert!(!message.contains("software renderer"), "{message}");

    let software = select(&plan(&GpuChoice::Software, &two_nodes()), outcomes(Ok("default"), [Ok("a"), Ok("b")], Err("no llvmpipe")))
        .unwrap_err();
    let software_message = failure_message(&GpuChoice::Software, &software, "");
    assert!(software_message.starts_with("--gpu software cannot be used: no llvmpipe"), "{software_message}");
}

/// A software device where a default GPU was expected is rejected.
#[test]
fn a_software_default_is_not_a_gpu() {
    assert!(default_device_rejection(true).unwrap().contains("software rasterizer"));
    assert_eq!(default_device_rejection(false), None);
}

/// Software puts `LIBGL_ALWAYS_SOFTWARE=1` and removes `DRI_PRIME`;
/// a pinned GPU removes `DRI_PRIME` only;
/// plain `auto` leaves the environment alone.
#[test]
fn child_environment_follows_the_final_choice() {
    assert_eq!(child_variables(child_environment(&GpuChoice::Auto, false, false)), vec![]);
    assert_eq!(
        child_variables(child_environment(&GpuChoice::Software, true, false)),
        vec![("LIBGL_ALWAYS_SOFTWARE", Some("1")), ("DRI_PRIME", None)],
    );
    let node = GpuChoice::RenderNode(PathBuf::from("/dev/dri/renderD128"));
    assert_eq!(child_variables(child_environment(&node, false, false)), vec![("DRI_PRIME", None)]);
    assert_eq!(child_variables(child_environment(&GpuChoice::Auto, false, true)), vec![("DRI_PRIME", None)]);
    assert_eq!(
        child_variables(child_environment(&GpuChoice::Auto, true, true)),
        vec![("LIBGL_ALWAYS_SOFTWARE", Some("1")), ("DRI_PRIME", None)],
    );
}

/// The control answer keeps the renderer last and spells the fallback.
#[test]
fn report_answer_is_one_line_with_the_renderer_last() {
    let report = GpuReport {
        choice: "auto".to_string(),
        device: "software".to_string(),
        renderer: "llvmpipe (LLVM 22.1.8, 256 bits)".to_string(),
        fell_back: true,
    };
    assert_eq!(report.answer(), "device=software choice=auto fallback=yes renderer=llvmpipe (LLVM 22.1.8, 256 bits)");
}
