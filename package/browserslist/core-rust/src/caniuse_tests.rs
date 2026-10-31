//! What:
//!  Tests for the caniuse-lite decoder on a synthetic revision and on the committed real revision.
//! Why:
//!  The positional `released` cut and the empty-slot handling are the parts browserslist's results depend on.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! describe('parseAgents', () => { /* synthetic and fixture revisions */ });
//! ```

use crate::caniuse::{Agent, AgentTable, parse_agents};
use crate::error::BrowserslistError;

/// Browser-name table of the synthetic revision.
const BROWSERS: &str = "module.exports={A:\"ie\",B:\"chrome\"};\n";
/// Version table of the synthetic revision.
const VERSIONS: &str = "module.exports={A:\"5.5\",B:\"11\",C:\"100\",D:\"101\",E:\"102\",\"0\":\"103\"};\n";
/// Agents of the synthetic revision: ie has two released slots and three reserved ones.
const AGENTS: &str = "module.exports={A:{C:[\"A\",\"\",\"B\",\"\",\"\",\"\"]},B:{C:[\"C\",\"D\",\"E\",\"0\",\"\",\"\"]}};\n";

/// The decoder maps codes to versions and empty strings to empty slots.
#[test]
fn decodes_slots_and_names() {
    let table = parse_agents(AGENTS, BROWSERS, VERSIONS).expect("synthetic revision parses");
    let ie = table.get("ie").expect("ie present");
    assert_eq!(ie.slots.len(), 6);
    assert_eq!(ie.versions(), vec!["5.5", "11"]);
    assert_eq!(ie.released(), vec!["5.5", "11"]);
    let chrome = table.get("chrome").expect("chrome present");
    assert_eq!(chrome.versions(), vec!["100", "101", "102", "103"]);
    assert_eq!(chrome.released(), vec!["100", "101", "102"]);
    assert_eq!(table.get("opera"), None);
}

/// `released` cuts three raw slots even when they hold versions, and tolerates arrays shorter than three.
#[test]
fn released_cut_is_positional() {
    let short = Agent { name: "x".to_string(), slots: vec![Some("1".to_string()), None] };
    assert_eq!(short.released(), Vec::<&str>::new());
    assert_eq!(short.versions(), vec!["1"]);
    let full = Agent {
        name: "y".to_string(),
        slots: vec![Some("1".to_string()), Some("2".to_string()), Some("3".to_string()), Some("4".to_string())],
    };
    assert_eq!(full.released(), vec!["1"]);
}

/// `require` is case-insensitive and fails with browserslist's own wording, echoing the written spelling.
#[test]
fn require_matches_browserslist_wording() {
    let table = AgentTable::new(vec![Agent { name: "chrome".to_string(), slots: Vec::new() }]);
    assert_eq!(table.require("Chrome").expect("found").name, "chrome");
    let error = table.require("Foo").expect_err("unknown browser");
    assert_eq!(error, BrowserslistError::Browserslist { message: "Unknown browser Foo".to_string() });
}

/// Unknown codes and malformed shapes are errors, never silently dropped.
#[test]
fn rejects_inconsistent_data() {
    let unknown_version = "module.exports={A:{C:[\"Z\"]}};\n";
    let message = parse_agents(unknown_version, BROWSERS, VERSIONS).expect_err("unknown version code").to_string();
    assert_eq!(message.contains("version code Z"), true, "{message}");
    let unknown_browser = "module.exports={Q:{C:[]}};\n";
    let message = parse_agents(unknown_browser, BROWSERS, VERSIONS).expect_err("unknown browser code").to_string();
    assert_eq!(message.contains("browser code Q"), true, "{message}");
    for agents in [
        "module.exports={A:{}};\n",
        "module.exports={A:{C:[1]}};\n",
        "module.exports=[];\n",
    ] {
        assert_eq!(parse_agents(agents, BROWSERS, VERSIONS).is_err(), true, "{agents}");
    }
    assert_eq!(parse_agents(AGENTS, "module.exports={A:1};\n", VERSIONS).is_err(), true);
    assert_eq!(parse_agents(AGENTS, "module.exports=1;\n", VERSIONS).is_err(), true);
    assert_eq!(parse_agents(AGENTS, BROWSERS, "module.exports=[];\n").is_err(), true);
}

/// A parse failure inside a file names that file.
#[test]
fn literal_errors_name_the_file() {
    let error = parse_agents("module.exports={A:1,};\n", BROWSERS, VERSIONS).expect_err("trailing comma");
    assert_eq!(error.to_string().contains("caniuse-lite data/agents.js"), true);
    let error = parse_agents(AGENTS, "oops", VERSIONS).expect_err("bad browsers file");
    assert_eq!(error.to_string().contains("caniuse-lite data/browsers.js"), true);
}

/// The committed real revision decodes, and the facts the spec records hold for it.
#[test]
fn decodes_the_committed_revision() {
    let table = parse_agents(
        include_str!("../fixtures/data/agents.js.txt"),
        include_str!("../fixtures/data/browsers.js.txt"),
        include_str!("../fixtures/data/browserVersions.js.txt"),
    )
    .expect("real revision parses");
    assert_eq!(table.get("chrome").expect("chrome").released().last().copied(), Some("156"));
    assert_eq!(table.get("and_chr").expect("and_chr").released(), vec!["156"]);
    assert_eq!(table.get("and_ff").expect("and_ff").released(), vec!["159"]);
    assert_eq!(table.get("android").expect("android").released().last().copied(), Some("156"));
    assert_eq!(table.get("chrome").expect("chrome").slots.len(), 163);
    assert_eq!(table.get("ie").expect("ie").released().len(), 7);
}
