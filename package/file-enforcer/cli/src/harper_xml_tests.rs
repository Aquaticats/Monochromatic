//! What:
//!  Tests for the Harper XML helpers:
//! base server recognition,
//! attribute-level entry edits,
//! appended entries,
//! and the nested mappings comparison.
//! Why:
//!  Comments and layout around managed entries must survive,
//! and a document that already conforms must come back byte for byte.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! describe('harper xml', () => { it('keeps comments in managed entries', () => {}); });
//! ```

/// Imports the fixtures.
use crate::harper_fixtures::{BASE_KEY, LANGUAGE_XML, SCOPED_KEY, USER_XML};
/// Imports the helpers under test.
use crate::harper_xml::{find_base_entry, find_entry, option_value, parse};
/// Imports the edits under test.
use crate::harper_xml_edit::{ScopedUserEntry, upsert_language_entry, upsert_user_entry};

/// The path the helpers name in diagnostics.
const PATH: &str = "options/f.xml";

/// Decoded value of an option in a named entry of a document.
fn value_in(xml: &str, key: &str, name: &str) -> Option<String> {
    let document = parse(PATH, xml).expect("well-formed");
    let entry = find_entry(&document, key)?;
    return option_value(&document, entry, name);
}

/// Owned strings from literals.
fn owned(names: &[&str]) -> Vec<String> {
    return names.iter().map(|name| return (*name).to_string()).collect();
}

/// A scoped server entry description for the tests.
fn scoped<'a>(file_names: &'a [String], copied: &'a [(String, String)]) -> ScopedUserEntry<'a> {
    return ScopedUserEntry { key: SCOPED_KEY, name: "Harper (docs)", language_id: "markdown", file_names, copied };
}

/// The base server is found by any one of its template id,
/// its name or its command line,
/// and never among the excluded ids.
#[test]
fn the_base_server_is_recognized_by_any_of_three_facts() {
    let by_template = USER_XML.replace("harper-ls --stdio", "other").replace("Harper Language Server", "Renamed");
    let by_name = USER_XML.replace("harper-ls --stdio", "other").replace("<option name=\"templateId\" value=\"harper-ls\" />", "");
    let by_command = USER_XML.replace("Harper Language Server", "Renamed").replace("<option name=\"templateId\" value=\"harper-ls\" />", "");
    for xml in [USER_XML.to_string(), by_template, by_name, by_command] {
        let document = parse(PATH, &xml).expect("well-formed");
        let found = find_base_entry(&document, &[]);
        assert_eq!(found.map(|(_, key)| return key), Some(BASE_KEY.to_string()), "{xml}");
    }
    let document = parse(PATH, USER_XML).expect("well-formed");
    assert!(find_base_entry(&document, &[BASE_KEY.to_string()]).is_none(), "an excluded id is skipped");
    let unrelated = parse(PATH, &USER_XML.replace("harper-ls", "other").replace("Harper Language Server", "x")).expect("well-formed");
    assert!(find_base_entry(&unrelated, &[]).is_none());
}

/// A document that is not well-formed is refused with the file's path.
#[test]
fn malformed_xml_is_refused_with_the_path() {
    let error = parse(PATH, "<application><map></application>").expect_err("refused");
    assert_eq!(error.code, "harper-xml-invalid");
    assert_eq!(error.path.as_deref(), Some(PATH));
}

/// An existing language entry has its two JSON options set in place;
/// the comment in the entry and every other byte survive.
#[test]
fn an_existing_language_entry_is_edited_attribute_by_attribute() {
    let edited = upsert_language_entry(PATH, LANGUAGE_XML, BASE_KEY, "{\n  \"a\": false\n}", "{\"properties\":{}}").expect("edited");
    assert!(edited.contains("<!-- keep this comment -->"));
    assert_eq!(value_in(&edited, BASE_KEY, "configurationContent").as_deref(), Some("{\n  \"a\": false\n}"));
    assert!(edited.contains("&#10;"), "newlines are written as references");
    let restored = upsert_language_entry(PATH, &edited, BASE_KEY, &value_in(LANGUAGE_XML, BASE_KEY, "configurationContent").unwrap_or_default(), "{\"properties\":{}}")
        .expect("edited back");
    assert_eq!(restored, LANGUAGE_XML, "putting the old values back restores the exact original bytes");
}

/// A document that already holds the wanted values comes back unchanged.
#[test]
fn a_conforming_language_entry_is_not_rewritten() {
    let config = value_in(LANGUAGE_XML, BASE_KEY, "configurationContent").unwrap_or_default();
    let schema = value_in(LANGUAGE_XML, BASE_KEY, "configurationSchemaContent").unwrap_or_default();
    assert_eq!(upsert_language_entry(PATH, LANGUAGE_XML, BASE_KEY, &config, &schema).expect("kept"), LANGUAGE_XML);
}

/// Missing seed options are created,
/// an absent entry is appended with all four options,
/// and an entry without its options element is replaced.
#[test]
fn absent_options_and_entries_are_created() {
    let bare = LANGUAGE_XML.replace("              <option name=\"experimentalContent\" value=\"{}\" />\n", "");
    let seeded = upsert_language_entry(PATH, &bare, BASE_KEY, "{}", "{}").expect("seeded");
    assert_eq!(value_in(&seeded, BASE_KEY, "experimentalContent").as_deref(), Some("{}"));
    let appended = upsert_language_entry(PATH, LANGUAGE_XML, SCOPED_KEY, "{\"x\": 1}", "{}").expect("appended");
    assert_eq!(value_in(&appended, SCOPED_KEY, "configurationContent").as_deref(), Some("{\"x\": 1}"));
    assert_eq!(value_in(&appended, SCOPED_KEY, "initializationOptionsContent").as_deref(), Some("{}"));
    assert!(appended.contains("<!-- keep this comment -->"));
    let hollow = LANGUAGE_XML.replace("LanguageServerDefinitionSettings", "Other");
    let replaced = upsert_language_entry(PATH, &hollow, BASE_KEY, "{\"y\": 2}", "{}").expect("replaced");
    assert_eq!(value_in(&replaced, BASE_KEY, "configurationContent").as_deref(), Some("{\"y\": 2}"));
}

/// A settings file without any map element cannot hold a new entry.
#[test]
fn a_file_without_a_map_cannot_receive_an_entry() {
    let error = upsert_language_entry(PATH, "<application/>", SCOPED_KEY, "{}", "{}").expect_err("refused");
    assert_eq!(error.code, "harper-xml-invalid");
}

/// A scoped user-defined entry is appended with the copied options,
/// its id,
/// its name and its mapping.
#[test]
fn a_scoped_user_entry_is_appended_with_copied_options_and_mapping() {
    let files = owned(&["AGENTS.md", "CLAUDE.md"]);
    let copied = vec![("commandLine".to_string(), "harper-ls --stdio".to_string()), ("workingDir".to_string(), "/work".to_string())];
    let edited = upsert_user_entry(PATH, USER_XML, &scoped(&files, &copied)).expect("appended");
    assert_eq!(value_in(&edited, SCOPED_KEY, "commandLine").as_deref(), Some("harper-ls --stdio"));
    assert_eq!(value_in(&edited, SCOPED_KEY, "serverId").as_deref(), Some(SCOPED_KEY));
    assert_eq!(value_in(&edited, SCOPED_KEY, "serverName").as_deref(), Some("Harper (docs)"));
    assert!(edited.contains("<option value=\"AGENTS.md\" />") && edited.contains("<option value=\"CLAUDE.md\" />"));
    assert!(edited.contains("languageId=\"markdown\""));
    let again = upsert_user_entry(PATH, &edited, &scoped(&files, &copied)).expect("kept");
    assert_eq!(again, edited, "a conforming entry is not rewritten");
}

/// An existing scoped entry is updated in place:
/// a comment inside it survives a name change,
/// and a changed mapping is replaced as a whole.
#[test]
fn an_existing_scoped_user_entry_is_updated_in_place() {
    let files = owned(&["AGENTS.md"]);
    let copied: Vec<(String, String)> = Vec::new();
    let first = upsert_user_entry(PATH, USER_XML, &scoped(&files, &copied)).expect("appended");
    let commented = first.replace("<option name=\"serverId\"", "<!-- mine -->\n              <option name=\"serverId\"");
    let renamed_entry = ScopedUserEntry { name: "Renamed", ..scoped(&files, &copied) };
    let renamed = upsert_user_entry(PATH, &commented, &renamed_entry).expect("renamed");
    assert!(renamed.contains("<!-- mine -->"), "a comment inside a managed entry survives an attribute edit");
    assert_eq!(value_in(&renamed, SCOPED_KEY, "serverName").as_deref(), Some("Renamed"));
    let more = owned(&["AGENTS.md", "CLAUDE.md"]);
    let remapped = upsert_user_entry(PATH, &renamed, &ScopedUserEntry { name: "Renamed", ..scoped(&more, &copied) }).expect("remapped");
    assert!(remapped.contains("<option value=\"CLAUDE.md\" />"));
    assert!(remapped.contains("<!-- mine -->"), "only the mapping subtree was replaced");
    let other_language = upsert_user_entry(PATH, &remapped, &ScopedUserEntry { language_id: "text", name: "Renamed", ..scoped(&more, &copied) })
        .expect("language changed");
    assert!(other_language.contains("languageId=\"text\""));
}

/// An entry that lacks its options element is replaced whole,
/// and an entry that lacks its mapping gets one appended.
#[test]
fn a_hollow_or_unmapped_user_entry_is_completed() {
    let files = owned(&["AGENTS.md"]);
    let copied: Vec<(String, String)> = Vec::new();
    let first = upsert_user_entry(PATH, USER_XML, &scoped(&files, &copied)).expect("appended");
    let hollow = first.replacen("UserDefinedLanguageServerItemSettings", "Other", 2);
    let replaced = upsert_user_entry(PATH, &hollow, &scoped(&files, &copied)).expect("hollow");
    assert!(replaced.matches("UserDefinedLanguageServerItemSettings").count() >= 2);
    assert_eq!(value_in(&replaced, SCOPED_KEY, "serverName").as_deref(), Some("Harper (docs)"));
    let start = first.find("              <option name=\"mappings\">").expect("mapping start");
    let end = first[start..].find("              </option>\n            </UserDefinedLanguageServerItemSettings>").expect("mapping end") + start;
    let unmapped = format!("{}{}", &first[..start], &first[end + "              </option>\n".len()..]);
    let completed = upsert_user_entry(PATH, &unmapped, &scoped(&files, &copied)).expect("completed");
    assert!(completed.contains("<option name=\"mappings\">"));
}
