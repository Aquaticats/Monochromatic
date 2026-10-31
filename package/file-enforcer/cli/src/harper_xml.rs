//! What:
//!  The XML half of the JetBrains Harper policy:
//! parse the IDE's settings files,
//! find the base language server,
//! and describe the entries the policy writes.
//! Why:
//!  `monochromatic-xml-edit` splices only the bytes it changes,
//! so comments,
//! indentation and unmanaged entries survive.
//!  The edits themselves are in `harper_xml_edit.rs`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function findBaseEntry(xml: XmlDocument, excluded: string[]): { id: ElementId; key: string } | undefined
//! ```

/// Imports `Diagnostic` from `monochromatic_file_enforcer::diagnostic`.
use monochromatic_file_enforcer::diagnostic::Diagnostic;
/// Imports `XmlDocument`,
/// `XmlElementId`,
/// `XmlElementSpec`,
/// `parse_xml` from `monochromatic_xml_edit`.
use monochromatic_xml_edit::{XmlDocument, XmlElementId, XmlElementSpec, parse_xml};

/// The element that holds one server's options in `LanguageServersSettings.xml`.
pub const LANGUAGE_CONTAINER: &str = "LanguageServerDefinitionSettings";

/// The element that holds one server's options in `UserDefinedLanguageServerSettings.xml`.
pub const USER_CONTAINER: &str = "UserDefinedLanguageServerItemSettings";

/// What:
///  How the base Harper server is recognized among the user-defined servers.
/// Why:
///  The retired TypeScript policy matched any one of three facts,
/// so a server the user renamed or reinstalled is still found.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type BaseServerMatch = { templateId: string; serverNameEquals: string; commandLineIncludes: string };
/// ```
pub struct BaseServerMatch {
    /// The `templateId` option value that identifies the server.
    pub template_id: &'static str,
    /// The `serverName` option value that identifies the server.
    pub server_name: &'static str,
    /// Text the `commandLine` option contains for the server.
    pub command_line_includes: &'static str,
}

/// The base server the Harper policy derives from.
pub const HARPER_MATCH: BaseServerMatch =
    BaseServerMatch { template_id: "harper-ls", server_name: "Harper Language Server", command_line_includes: "harper-ls" };

/// What:
///  Parse a settings file,
/// naming the file when it is not well-formed.
/// Why:
///  A broken settings file fails the run with its path instead of being rewritten.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parse(path: string, text: string): XmlDocument
/// ```
pub fn parse(path: &str, text: &str) -> Result<XmlDocument, Diagnostic> {
    match parse_xml(text) {
        Ok(document) => return Ok(document),
        Err(error) => {
            let message = format!("{path} is not well-formed XML this editor accepts: {error}");
            return Err(Diagnostic::new("harper-xml-invalid", &message).with_path(path));
        }
    }
}

/// Report an edit refusal with the file's path.
pub fn edit_failure(path: &str, error: &dyn std::fmt::Display) -> Diagnostic {
    let message = format!("{path}: the settings edit was refused: {error}");
    return Diagnostic::new("harper-xml-edit-failed", &message).with_path(path);
}

/// What:
///  The decoded `value` of the first `option` element named `name` anywhere under `scope`.
/// Why:
///  Mirrors how the IDE's settings are read:
/// an option is found by name and its `value` attribute is the setting.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function optionValue(doc: XmlDocument, scope: ElementId, name: string): string | undefined
/// ```
pub fn option_value(document: &XmlDocument, scope: XmlElementId, name: &str) -> Option<String> {
    for option in document.descendants_named(scope, "option") {
        if document.attribute(option, "name") == Some(name) {
            return document.attribute(option, "value").map(str::to_string);
        }
    }
    return None;
}

/// The first `entry` element with this decoded `key`,
/// wherever it sits.
pub fn find_entry(document: &XmlDocument, key: &str) -> Option<XmlElementId> {
    for entry in document.descendants_named(document.root(), "entry") {
        if document.attribute(entry, "key") == Some(key) {
            return Some(entry);
        }
    }
    return None;
}

/// What:
///  The key of the first server entry that matches the base-server facts and is not excluded.
/// Why:
///  The managed scoped servers must never be taken for the base server they derive from.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function findBaseServer(doc: XmlDocument, excluded: string[]): { id: ElementId; key: string } | undefined
/// ```
pub fn find_base_entry(document: &XmlDocument, excluded: &[String]) -> Option<(XmlElementId, String)> {
    for entry in document.descendants_named(document.root(), "entry") {
        let Some(key) = document.attribute(entry, "key") else { continue };
        if excluded.iter().any(|id| return id == key) {
            continue;
        }
        let template = option_value(document, entry, "templateId");
        let name = option_value(document, entry, "serverName");
        let command = option_value(document, entry, "commandLine");
        let by_template = template.as_deref() == Some(HARPER_MATCH.template_id);
        let by_name = name.as_deref() == Some(HARPER_MATCH.server_name);
        let by_command = command.is_some_and(|text| return text.contains(HARPER_MATCH.command_line_includes));
        if by_template || by_name || by_command {
            return Some((entry, key.to_string()));
        }
    }
    return None;
}

/// One `<option name=".." value=".." />` description.
pub fn option_spec(name: &str, value: &str) -> XmlElementSpec {
    return XmlElementSpec::named("option").with_attribute("name", name).with_attribute("value", value);
}

/// What:
///  The description of the `mappings` option that binds file names to a language id.
/// Why:
///  The same shape the retired policy rendered and the IDE reads back.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function mappingsSpec(languageId: string, fileNames: string[]): ElementSpec
/// ```
pub fn mappings_spec(language_id: &str, file_names: &[String]) -> XmlElementSpec {
    let mut patterns = XmlElementSpec::named("option").with_attribute("name", "patterns");
    for file_name in file_names {
        patterns = patterns.with_child(XmlElementSpec::named("option").with_attribute("value", file_name));
    }
    let settings = XmlElementSpec::named("FileTypeSettings").with_child(patterns);
    let mapping = XmlElementSpec::named("ServerMappingSettings")
        .with_attribute("languageId", language_id)
        .with_child(XmlElementSpec::named("fileType").with_child(settings));
    return XmlElementSpec::named("option").with_attribute("name", "mappings").with_child(mapping);
}

/// An `entry` description:
/// a key and the `value` element holding one container of options.
pub fn entry_spec(key: &str, container: &str, options: Vec<XmlElementSpec>) -> XmlElementSpec {
    let mut holder = XmlElementSpec::named(container);
    for option in options {
        holder = holder.with_child(option);
    }
    return XmlElementSpec::named("entry").with_attribute("key", key).with_child(XmlElementSpec::named("value").with_child(holder));
}

/// Tests for the XML helpers.
#[cfg(test)]
#[path = "harper_xml_tests.rs"]
mod tests;
