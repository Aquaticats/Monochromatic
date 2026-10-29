//! The fake's introspected interfaces against the interface XML files of Plasma's own source.
//!
//! The XML files under `tests/fixtures/plasma-workspace-6.7.5/` are copied unchanged from
//! KDE/plasma-workspace tag v6.7.5 (`libnotificationmanager/dbus/`),
//!  the version installed on the host.
//! Names and signatures of every method and signal must match.
//! The `NoReply` annotation on `update` and `terminate` is not part of zbus's introspection output,
//! so the fixtures are checked for it here and the calls are checked at runtime in `jobs`.

use std::collections::BTreeMap;

use nested_wayland_session::{
    job_server::{JOB_VIEW_SERVER_PATH, KUISERVER_NAME},
    job_view::JOB_VIEW_INTERFACE,
    notification_mode::NotificationMode,
    notification_service::{NOTIFICATIONS_INTERFACE, NOTIFICATIONS_NAME, NOTIFICATIONS_PATH},
};
use zbus::zvariant::OwnedObjectPath;
use zbus_xml::{ArgDirection, Interface, Node};

use super::support::{proxy, start, Fixture};

/// Plasma's XML for `org.freedesktop.Notifications`.
const NOTIFICATIONS_XML: &str = include_str!("../fixtures/plasma-workspace-6.7.5/org.freedesktop.Notifications.xml");

/// Plasma's XML for `org.kde.JobViewServer`.
const JOB_VIEW_SERVER_XML: &str = include_str!("../fixtures/plasma-workspace-6.7.5/org.kde.JobViewServer.xml");

/// Plasma's XML for `org.kde.JobViewServerV2`.
const JOB_VIEW_SERVER_V2_XML: &str = include_str!("../fixtures/plasma-workspace-6.7.5/org.kde.JobViewServerV2.xml");

/// Plasma's XML for `org.kde.JobViewV3`.
const JOB_VIEW_V3_XML: &str = include_str!("../fixtures/plasma-workspace-6.7.5/org.kde.JobViewV3.xml");

/// Plasma's XML for `org.kde.kuiserver`.
const KUISERVER_XML: &str = include_str!("../fixtures/plasma-workspace-6.7.5/org.kde.kuiserver.xml");

/// Members of one interface by name,
///  each with its signature:
///  inputs for methods,
///  outputs after `->`.
#[derive(Debug, Default, PartialEq)]
struct Members {
    /// Method name to `in-signature->out-signature`.
    methods: BTreeMap<String, String>,
    /// Signal name to signature.
    signals: BTreeMap<String, String>,
    /// Property name to signature.
    properties: BTreeMap<String, String>,
}

/// Reduces one parsed interface to its members.
fn members_of(interface: &Interface<'_>) -> Members {
    let mut members = Members::default();
    for method in interface.methods() {
        let mut inputs = String::new();
        let mut outputs = String::new();
        for argument in method.args() {
            // A method argument without a direction is an input, by the specification.
            if argument.direction() == Some(ArgDirection::Out) {
                outputs.push_str(&argument.ty().to_string());
            } else {
                inputs.push_str(&argument.ty().to_string());
            }
        }
        members.methods.insert(method.name().to_string(), format!("{inputs}->{outputs}"));
    }
    for signal in interface.signals() {
        let signature: String = signal.args().iter().map(|argument| return argument.ty().to_string()).collect();
        members.signals.insert(signal.name().to_string(), signature);
    }
    for property in interface.properties() {
        members.properties.insert(property.name().to_string(), property.ty().to_string());
    }
    return members;
}

/// Parses XML and returns the members of the named interface.
fn parse_members(xml: &str, interface_name: &str) -> Members {
    let node = Node::from_reader(xml.as_bytes()).expect("the XML parses");
    let interface = node
        .interfaces()
        .iter()
        .find(|candidate| return candidate.name().as_str() == interface_name)
        .unwrap_or_else(|| panic!("interface {interface_name} is not in the XML"));
    return members_of(interface);
}

/// Introspects `path` on the fake and returns the members of `interface_name`.
fn introspect(fixture: &Fixture, path: &str, interface_name: &str) -> anyhow::Result<Members> {
    let introspectable = proxy(&fixture.client, KUISERVER_NAME, path, "org.freedesktop.DBus.Introspectable")?;
    let xml: String = introspectable.call("Introspect", &())?;
    return Ok(parse_members(&xml, interface_name));
}

/// The three server interfaces equal Plasma's,
///  member for member,
///  with no job created.
#[test]
fn job_server_interfaces_equal_plasmas() -> anyhow::Result<()> {
    let fixture = start(NotificationMode::Kde)?;
    for (name, xml) in [
        ("org.kde.JobViewServer", JOB_VIEW_SERVER_XML),
        ("org.kde.JobViewServerV2", JOB_VIEW_SERVER_V2_XML),
        ("org.kde.kuiserver", KUISERVER_XML),
    ] {
        let fake = introspect(&fixture, JOB_VIEW_SERVER_PATH, name)?;
        let plasma = parse_members(xml, name);
        assert_eq!(fake, plasma, "{name} differs from Plasma's XML");
        assert!(!plasma.methods.is_empty(), "{name} has no methods in the XML");
    }
    return Ok(());
}

/// The per-job interface equals Plasma's,
///  member for member.
#[test]
fn job_view_interface_equals_plasmas() -> anyhow::Result<()> {
    let fixture = start(NotificationMode::Kde)?;
    // The first interface version grants the job,
    // so a change to the second version's signature fails the server comparison and not this one.
    let server = proxy(&fixture.client, "org.kde.JobViewServer", JOB_VIEW_SERVER_PATH, "org.kde.JobViewServer")?;
    let job_path: OwnedObjectPath = server.call("requestView", &("org.example.player", "folder", 3_i32))?;
    let fake = introspect(&fixture, job_path.as_str(), JOB_VIEW_INTERFACE)?;
    let plasma = parse_members(JOB_VIEW_V3_XML, JOB_VIEW_INTERFACE);
    assert_eq!(fake, plasma, "{JOB_VIEW_INTERFACE} differs from Plasma's XML");
    assert_eq!(plasma.signals.len(), 4, "the XML lists four V3 signals");
    return Ok(());
}

/// The notification interface matches Plasma's,
///  except for members the recorder deliberately omits.
#[test]
fn notification_interface_matches_plasmas_except_for_the_omitted_members() -> anyhow::Result<()> {
    let fixture = start(NotificationMode::Freedesktop)?;
    let fake = introspect_notifications(&fixture)?;
    let mut plasma = parse_members(NOTIFICATIONS_XML, NOTIFICATIONS_INTERFACE);
    // Plasma-only members, each with its reason:
    //  Inhibit, UnInhibit, Inhibited: the `inhibitions` capability, which the recorder does not report.
    //  NotificationReplied: the `inline-reply` capability, which the recorder does not report.
    //  ActivationToken: sent before ActionInvoked on Wayland by Plasma's UI, which the recorder does not draw.
    let omitted_methods = ["Inhibit", "UnInhibit"];
    let omitted_signals = ["NotificationReplied", "ActivationToken"];
    let omitted_properties = ["Inhibited"];
    for name in omitted_methods {
        assert!(plasma.methods.remove(name).is_some(), "{name} is no longer in Plasma's XML");
    }
    for name in omitted_signals {
        assert!(plasma.signals.remove(name).is_some(), "{name} is no longer in Plasma's XML");
    }
    for name in omitted_properties {
        assert!(plasma.properties.remove(name).is_some(), "{name} is no longer in Plasma's XML");
    }
    assert_eq!(fake, plasma, "the notification interface differs from Plasma's XML");
    return Ok(());
}

/// Introspects the notification object,
///  which lives on its own path and name.
fn introspect_notifications(fixture: &Fixture) -> anyhow::Result<Members> {
    let introspectable = proxy(
        &fixture.client,
        NOTIFICATIONS_NAME,
        NOTIFICATIONS_PATH,
        "org.freedesktop.DBus.Introspectable",
    )?;
    let xml: String = introspectable.call("Introspect", &())?;
    return Ok(parse_members(&xml, NOTIFICATIONS_INTERFACE));
}

/// The fixtures carry the `NoReply` annotation that the runtime tests rely on.
#[test]
fn plasmas_xml_marks_update_and_terminate_no_reply() {
    for method in ["update", "terminate"] {
        let marker = format!("<method name=\"{method}\">");
        let start = JOB_VIEW_V3_XML.find(&marker).expect("the method is in the XML");
        let tail = JOB_VIEW_V3_XML.get(start..).expect("the offset is inside the XML");
        let end = tail.find("</method>").expect("the method element closes");
        let body = tail.get(..end).expect("the offset is inside the element");
        assert!(body.contains("org.freedesktop.DBus.Method.NoReply\" value=\"true\""), "{method}");
    }
}

/// The comparison can tell two different interfaces apart,
///  so a pass above means something.
#[test]
fn the_comparison_detects_a_wrong_signature() {
    let right = parse_members(JOB_VIEW_SERVER_V2_XML, "org.kde.JobViewServerV2");
    let wrong_xml = JOB_VIEW_SERVER_V2_XML.replace("name=\"capabilities\" type=\"i\"", "name=\"capabilities\" type=\"u\"");
    assert_ne!(wrong_xml, JOB_VIEW_SERVER_V2_XML, "the replacement found nothing to change");
    let wrong = parse_members(&wrong_xml, "org.kde.JobViewServerV2");
    assert_ne!(right, wrong);
    assert_eq!(right.methods.get("requestView").map(String::as_str), Some("sia{sv}->o"));
    assert_eq!(wrong.methods.get("requestView").map(String::as_str), Some("sua{sv}->o"));
}
