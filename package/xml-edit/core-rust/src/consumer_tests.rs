//! What:
//!  An end-to-end walk of the edits the file-enforcer JetBrains rule needs, on the synthetic fixtures.
//! Why:
//!  It proves the crate's surface covers the consumer: find the base server, rewrite its option values,
//!  add a scoped entry to both files, and leave comments and layout alone.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! describe('LSP4IJ policy edits', () => { /* base lookup, option rewrite, scoped entry */ });
//! ```

use crate::fixtures::{LANGUAGE_XML, USER_DEFINED_XML};
use crate::{parse_xml, XmlDocument, XmlElementId, XmlElementSpec};

/// Build a one-line `<option name=".." value=".." />` description.
fn option(name: &str, value: &str) -> XmlElementSpec {
    return XmlElementSpec::named("option").with_attribute("name", name).with_attribute("value", value);
}

/// Check the whole policy flow: locate, rewrite attribute values, append scoped entries, stay re-readable.
#[test]
fn policy_flow_edits_both_files() {
    let commented: String = USER_DEFINED_XML.replace("<map>", "<map>\n        <!-- keep me -->");
    let user_defined: XmlDocument = parse_xml(&commented).expect("fixture parses");
    let mut base: Option<XmlElementId> = None;
    for entry in user_defined.descendants_named(user_defined.root(), "entry") {
        let name: Option<XmlElementId> = user_defined.find_child(
            user_defined.descendants_named(entry, "UserDefinedLanguageServerItemSettings")[0],
            "option",
            "name",
            "templateId",
        );
        if let Some(found) = name
            && user_defined.attribute(found, "value") == Some("harper-ls")
        {
            base = Some(entry);
        }
    }
    let base_entry: XmlElementId = base.expect("base server found");
    let key: String = user_defined.attribute(base_entry, "key").expect("entry key").to_string();
    assert_eq!(key, "srv-1");

    let language: XmlDocument = parse_xml(LANGUAGE_XML).expect("fixture parses");
    let language_map: XmlElementId = language.descendants_named(language.root(), "map")[0];
    let mut found_entry: Option<XmlElementId> = None;
    for candidate in language.children_named(language_map, "entry") {
        if language.attribute(candidate, "key") == Some(key.as_str()) {
            found_entry = Some(candidate);
        }
    }
    let language_entry: XmlElementId = found_entry.expect("entry for the base key");
    let settings: XmlElementId = language.children(language.children(language_entry)[0])[0];
    let config: XmlElementId = language.find_child(settings, "option", "name", "configurationContent").expect("config");
    let merged: String = "{\n  \"harper-ls.linters.Keep\": true,\n  \"harper-ls.linters.SplitWords\": false\n}".to_string();
    let language_edited: XmlDocument = language.set_attribute(config, "value", &merged).expect("rewrite");
    let map: XmlElementId = language_edited.descendants_named(language_edited.root(), "map")[0];
    let scoped: XmlElementSpec = XmlElementSpec::named("entry").with_attribute("key", "harper-ls-agents-claude-md").with_child(
        XmlElementSpec::named("value").with_child(
            XmlElementSpec::named("LanguageServerDefinitionSettings").with_child(option("configurationContent", &merged)),
        ),
    );
    let language_done: XmlDocument = language_edited.append_child(map, &scoped).expect("append");
    let reread: XmlDocument = parse_xml(language_done.source()).expect("output is well-formed");
    let appended: XmlElementId = reread.descendants_named(reread.root(), "entry")[1];
    let appended_settings: XmlElementId = reread.descendants_named(appended, "option")[0];
    assert_eq!(reread.attribute(appended_settings, "value"), Some(merged.as_str()));
    assert_eq!(reread.attribute(appended, "key"), Some("harper-ls-agents-claude-md"));

    let user_map: XmlElementId = user_defined.descendants_named(user_defined.root(), "map")[0];
    let mapping: XmlElementSpec = XmlElementSpec::named("option").with_attribute("name", "mappings").with_child(
        XmlElementSpec::named("ServerMappingSettings").with_attribute("languageId", "markdown").with_child(
            XmlElementSpec::named("fileType").with_child(XmlElementSpec::named("FileTypeSettings").with_child(
                XmlElementSpec::named("option").with_attribute("name", "patterns").with_child(
                    XmlElementSpec::named("option").with_attribute("value", "AGENTS.md"),
                ),
            )),
        ),
    );
    let user_entry: XmlElementSpec = XmlElementSpec::named("entry").with_attribute("key", "harper-ls-agents-claude-md").with_child(
        XmlElementSpec::named("value").with_child(
            XmlElementSpec::named("UserDefinedLanguageServerItemSettings")
                .with_child(option("commandLine", "harper-ls"))
                .with_child(mapping),
        ),
    );
    let user_done: XmlDocument = user_defined.append_child(user_map, &user_entry).expect("append");
    assert!(user_done.source().contains("<!-- keep me -->"));
    assert!(user_done.source().contains("                        <option value=\"AGENTS.md\" />"));
    assert!(parse_xml(user_done.source()).is_ok());
    let untouched: String = user_done.source().replace(&user_done.source()[user_done.source().find("        <entry key=\"harper").expect("scoped entry")..user_done.source().find("      </map>").expect("map end")], "");
    assert_eq!(untouched, commented);
}
