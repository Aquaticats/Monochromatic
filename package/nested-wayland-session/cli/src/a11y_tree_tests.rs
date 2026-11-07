//! Unit tests for the accessibility answer formatting.

use super::{format_tree, read_all_within, role_name, state_names, Node};

/// The named bits decode from the two-word state vector.
#[test]
fn state_bits_decode_across_words() {
    assert_eq!(state_names(&[0, 0]), Vec::<&str>::new());
    assert_eq!(state_names(&[(1 << 11) | (1 << 12), 0]), vec!["focusable", "focused"]);
    assert_eq!(state_names(&[(1 << 8) | (1 << 30), 0]), vec!["enabled", "visible"]);
    assert_eq!(state_names(&[0, 1 << 1]), Vec::<&str>::new());
    assert_eq!(state_names(&[1 << 4]), vec!["checked"]);
}

/// A node record carries depth,
///  role,
///  name,
///  states,
///  and bounds,
///  with text percent-encoded.
#[test]
fn records_encode_text_and_carry_bounds() {
    let nodes = [
        Node { depth: 0, role: "application".to_string(), name: "My App".to_string(), states: vec![], bounds: None },
        Node { depth: 1, role: "push button".to_string(), name: "Play; now".to_string(), states: vec!["enabled", "focusable"], bounds: Some((10, 20, 30, 40)) },
    ];
    assert_eq!(
        format_tree(&nodes),
        "2 depth=0 role=application name=My%20App states= bounds=; \
         depth=1 role=push%20button name=Play%3B%20now states=enabled,focusable bounds=10,20,30,40",
    );
    assert_eq!(format_tree(&[]), "0");
}

/// Role numbers map to the AT-SPI role names,
///  and an unknown number still prints.
#[test]
fn roles_have_names() {
    assert_eq!(role_name(0), "invalid");
    assert_eq!(role_name(7), "check box");
    assert_eq!(role_name(23), "frame");
    assert_eq!(role_name(29), "label");
    assert_eq!(role_name(43), "push button");
    assert_eq!(role_name(61), "text");
    assert_eq!(role_name(75), "application");
    assert_eq!(role_name(79), "entry");
    assert_eq!(role_name(9999), "role-9999");
}

/// Imports for the tests that read a fake application over a real private bus.
use std::{sync::Arc, time::Duration};

use crate::{
    a11y_service::{A11yService, REGISTRY_NAME, ROOT_PATH},
    bus_test_client,
    private_bus::PrivateBus,
};

/// The role number the fake application answers for its root:
///  `application`.
const APPLICATION_ROLE: u32 = 75;

/// The short budget the stalled-application test gives the read.
const SHORT_BUDGET: Duration = Duration::from_millis(400);

/// The budget a healthy read gets:
///  the shipped one.
const HEALTHY_BUDGET: Duration = super::READ_BUDGET;

/// How many times the short budget the stalled read may take before it counts as late.
const LATE_FACTOR: u32 = 10;

/// A fake application root that answers every `Accessible` call.
struct AnsweringRoot;

// The zbus::interface expansion below calls Result::unwrap, which the repository clippy.toml bans;
// the allow covers only this block. The reason is in
// doc/troubleshooting/zbus-interface-macro-clippy-disallowed-unwrap.md.
#[allow(
    clippy::disallowed_methods,
    reason = "zbus::interface expands to writeln!(..).unwrap() in its generated introspection method"
)]
const _: () = {
    /// The calls the tree reader makes.
    #[zbus::interface(name = "org.a11y.atspi.Accessible")]
    impl AnsweringRoot {
        /// The role number.
        async fn get_role(&self) -> u32 {
            return APPLICATION_ROLE;
        }

        /// The state words.
        async fn get_state(&self) -> Vec<u32> {
            return vec![0, 0];
        }

        /// The children.
        async fn get_children(&self) -> Vec<(String, zbus::OwnedObjectPath)> {
            return Vec::new();
        }

        /// The accessible name.
        #[zbus(property)]
        async fn name(&self) -> String {
            return "Fake App".to_owned();
        }
    }
};

/// A fake application root whose `GetRole` never answers.
struct StalledRoot;

// The zbus::interface expansion below calls Result::unwrap, which the repository clippy.toml bans;
// the allow covers only this block. The reason is in
// doc/troubleshooting/zbus-interface-macro-clippy-disallowed-unwrap.md.
#[allow(
    clippy::disallowed_methods,
    reason = "zbus::interface expands to writeln!(..).unwrap() in its generated introspection method"
)]
const _: () = {
    /// The call the tree reader makes first.
    #[zbus::interface(name = "org.a11y.atspi.Accessible")]
    impl StalledRoot {
        /// Never answers:
        ///  the future stays pending for the life of the connection.
        async fn get_role(&self) -> u32 {
            std::future::pending::<()>().await;
            return 0;
        }
    }
};

/// Starts the service and one fake application that has registered its root on it.
fn service_with_application(
    root: impl zbus::object_server::Interface,
) -> anyhow::Result<(A11yService, zbus::Connection)> {
    let bus = Arc::new(PrivateBus::start()?);
    let service = A11yService::start(Arc::clone(&bus))?;
    let application = crate::bus_step_bound::build_bounded(
        zbus::connection::Builder::address(bus.address()).serve_at(ROOT_PATH, root),
        "starting the fake application",
    )?;
    let unique = application.unique_name().map(ToString::to_string).unwrap_or_default();
    let registry = bus_test_client::Proxy::new(&application, REGISTRY_NAME, ROOT_PATH, "org.a11y.atspi.Socket")?;
    let _desktop: (String, zbus::OwnedObjectPath) =
        registry.call("Embed", &(unique, zbus::OwnedObjectPath::try_from(ROOT_PATH)?))?;
    return Ok((service, application));
}

/// A registered application that answers is read into one node.
///  This is the positive control for the stalled read.
#[test]
fn an_answering_application_is_read_into_a_node() -> anyhow::Result<()> {
    let (service, _application) = service_with_application(AnsweringRoot)?;
    let nodes = read_all_within(&service, HEALTHY_BUDGET)?;
    assert_eq!(nodes.len(), 1);
    assert_eq!(nodes[0].role, role_name(APPLICATION_ROLE));
    assert_eq!(nodes[0].name, "Fake App");
    return Ok(());
}

/// An application whose call never answers ends the read at the budget with the message that names it,
///  and the read thread is not left waiting.
#[test]
fn a_stalled_application_ends_the_read_at_the_budget() -> anyhow::Result<()> {
    let (service, _application) = service_with_application(StalledRoot)?;
    let started = std::time::Instant::now();
    let outcome = read_all_within(&service, SHORT_BUDGET);
    let elapsed = started.elapsed();
    let message = format!("{:#}", outcome.expect_err("a stalled application cannot be read"));
    assert!(message.contains("the application did not answer its accessibility tree"), "{message}");
    assert!(elapsed >= SHORT_BUDGET && elapsed < SHORT_BUDGET * LATE_FACTOR, "ended at {elapsed:?}");
    return Ok(());
}
