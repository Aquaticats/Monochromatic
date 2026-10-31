//! What:
//!  The XML half of the JetBrains Harper policy:
//! find the base language server and write server entries into the IDE's two settings files.
//! Why:
//!  `monochromatic-xml-edit` splices only the bytes it changes,
//! so comments, indentation and unmanaged entries survive.
//!  Existing entries are edited attribute by attribute (an option's `value`),
//! and a nested `mappings` option is replaced only when it differs from the policy,
//! so a comment inside a managed entry survives every run that has nothing to change.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function upsertLanguageEntry(xml: string, key: string, config: string, schema: string): string
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
fn edit_failure(path: &str, error: &dyn std::fmt::Display) -> Diagnostic {
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
fn option_spec(name: &str, value: &str) -> XmlElementSpec {
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
fn mappings_spec(language_id: &str, file_names: &[String]) -> XmlElementSpec {
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

/// An `entry` description: a key and the `value` element holding one container of options.
fn entry_spec(key: &str, container: &str, options: Vec<XmlElementSpec>) -> XmlElementSpec {
    let mut holder = XmlElementSpec::named(container);
    for option in options {
        holder = holder.with_child(option);
    }
    return XmlElementSpec::named("entry").with_attribute("key", key).with_child(XmlElementSpec::named("value").with_child(holder));
}

/// Append an entry to the last `map` element,
/// where the IDE keeps its server entries.
fn append_entry(path: &str, document: &XmlDocument, spec: &XmlElementSpec) -> Result<String, Diagnostic> {
    let maps = document.descendants_named(document.root(), "map");
    let Some(last) = maps.last() else {
        let message = format!("{path}: the settings file has no map element to hold a server entry");
        return Err(Diagnostic::new("harper-xml-invalid", &message).with_path(path));
    };
    let edited = document.append_child(*last, spec).map_err(|error| return edit_failure(path, &error))?;
    return Ok(edited.source().to_string());
}

/// Replace a whole entry with its description.
fn replace_entry(path: &str, document: &XmlDocument, entry: XmlElementId, spec: &XmlElementSpec) -> Result<String, Diagnostic> {
    let edited = document.replace_element(entry, spec).map_err(|error| return edit_failure(path, &error))?;
    return Ok(edited.source().to_string());
}

/// What:
///  Make one option of an entry hold a value:
/// change the `value` attribute in place,
/// or append the option when the entry lacks it.
/// Why:
///  An attribute edit keeps comments and layout around the option.
///  With `overwrite` false an existing option is left alone,
/// for options the policy only seeds.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function ensureOption(xml: string, key: string, container: string, name: string, value: string, overwrite: boolean): string
/// ```
fn ensure_option(path: &str, text: &str, key: &str, container: &str, name: &str, value: &str, overwrite: bool) -> Result<String, Diagnostic> {
    let document = parse(path, text)?;
    let missing = || return edit_failure(path, &format!("the entry {key} has no {container} element"));
    let entry = find_entry(&document, key).ok_or_else(missing)?;
    let holder = *document.descendants_named(entry, container).first().ok_or_else(missing)?;
    match document.find_child(holder, "option", "name", name) {
        Some(option) => {
            if !overwrite || document.attribute(option, "value") == Some(value) {
                return Ok(text.to_string());
            }
            let edited = document.set_attribute(option, "value", value).map_err(|error| return edit_failure(path, &error))?;
            return Ok(edited.source().to_string());
        }
        None => {
            let edited = document.append_child(holder, &option_spec(name, value)).map_err(|error| return edit_failure(path, &error))?;
            return Ok(edited.source().to_string());
        }
    }
}

/// What:
///  Write the policy's configuration and schema into the base server's entry,
/// or into a scoped server's entry,
/// in `LanguageServersSettings.xml`.
/// Why:
///  An absent entry is appended;
/// an existing one has its two JSON options set and its two seed options created when missing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function upsertLanguageEntry(path: string, xml: string, key: string, config: string, schema: string): string
/// ```
pub fn upsert_language_entry(path: &str, text: &str, key: &str, config: &str, schema: &str) -> Result<String, Diagnostic> {
    let document = parse(path, text)?;
    let Some(entry) = find_entry(&document, key) else {
        let options = vec![
            option_spec("configurationContent", config),
            option_spec("configurationSchemaContent", schema),
            option_spec("experimentalContent", "{}"),
            option_spec("initializationOptionsContent", "{}"),
        ];
        return append_entry(path, &document, &entry_spec(key, LANGUAGE_CONTAINER, options));
    };
    if document.descendants_named(entry, LANGUAGE_CONTAINER).is_empty() {
        let options = vec![option_spec("configurationContent", config), option_spec("configurationSchemaContent", schema)];
        return replace_entry(path, &document, entry, &entry_spec(key, LANGUAGE_CONTAINER, options));
    }
    let with_config = ensure_option(path, text, key, LANGUAGE_CONTAINER, "configurationContent", config, true)?;
    let with_schema = ensure_option(path, &with_config, key, LANGUAGE_CONTAINER, "configurationSchemaContent", schema, true)?;
    let with_experimental = ensure_option(path, &with_schema, key, LANGUAGE_CONTAINER, "experimentalContent", "{}", false)?;
    return ensure_option(path, &with_experimental, key, LANGUAGE_CONTAINER, "initializationOptionsContent", "{}", false);
}

/// What:
///  The file-name patterns a `mappings` option binds to its language id,
/// when it has the shape the policy writes.
/// Why:
///  Comparing decoded values decides whether the nested structure needs replacing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function currentMapping(doc: XmlDocument, mappings: ElementId): { languageId: string; fileNames: string[] } | undefined
/// ```
fn current_mapping(document: &XmlDocument, mappings: XmlElementId) -> Option<(String, Vec<String>)> {
    let settings = document.descendants_named(mappings, "ServerMappingSettings");
    if settings.len() != 1 {
        return None;
    }
    let language_id = document.attribute(settings[0], "languageId")?.to_string();
    let patterns = document.descendants_named(settings[0], "option").into_iter().find(|option| return document.attribute(*option, "name") == Some("patterns"))?;
    let mut names: Vec<String> = Vec::new();
    for child in document.children_named(patterns, "option") {
        names.push(document.attribute(child, "value")?.to_string());
    }
    return Some((language_id, names));
}

/// What:
///  Make a scoped server's `mappings` option bind the given files to the given language id.
/// Why:
///  A matching option is left byte for byte;
/// a different or absent one is replaced or appended as a whole.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function ensureMappings(path: string, xml: string, key: string, languageId: string, fileNames: string[]): string
/// ```
fn ensure_mappings(path: &str, text: &str, key: &str, language_id: &str, file_names: &[String]) -> Result<String, Diagnostic> {
    let document = parse(path, text)?;
    let missing = || return edit_failure(path, &format!("the entry {key} has no {USER_CONTAINER} element"));
    let entry = find_entry(&document, key).ok_or_else(missing)?;
    let holder = *document.descendants_named(entry, USER_CONTAINER).first().ok_or_else(missing)?;
    let wanted = mappings_spec(language_id, file_names);
    let Some(existing) = document.find_child(holder, "option", "name", "mappings") else {
        let edited = document.append_child(holder, &wanted).map_err(|error| return edit_failure(path, &error))?;
        return Ok(edited.source().to_string());
    };
    if current_mapping(&document, existing).is_some_and(|(id, names)| return id == language_id && names == file_names) {
        return Ok(text.to_string());
    }
    let edited = document.replace_element(existing, &wanted).map_err(|error| return edit_failure(path, &error))?;
    return Ok(edited.source().to_string());
}

/// What:
///  What a scoped server entry in `UserDefinedLanguageServerSettings.xml` must say.
/// Why:
///  One value carries the many facts the entry's options hold.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ScopedUserEntry = { key: string; name: string; languageId: string; fileNames: string[]; copied: [string, string][] }
/// ```
pub struct ScopedUserEntry<'a> {
    /// Server id used as the entry key and the `serverId` option.
    pub key: &'a str,
    /// Display name for the `serverName` option.
    pub name: &'a str,
    /// Language id the files map to.
    pub language_id: &'a str,
    /// File names mapped to the server.
    pub file_names: &'a [String],
    /// Options copied from the base server, in output order.
    pub copied: &'a [(String, String)],
}

/// What:
///  Write a scoped server entry into `UserDefinedLanguageServerSettings.xml`.
/// Why:
///  An absent entry is appended whole;
/// an existing one has each option set in place and its mapping compared.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function upsertUserEntry(path: string, xml: string, entry: ScopedUserEntry): string
/// ```
pub fn upsert_user_entry(path: &str, text: &str, entry: &ScopedUserEntry) -> Result<String, Diagnostic> {
    let document = parse(path, text)?;
    let mut options: Vec<(String, String)> = entry.copied.to_vec();
    options.push(("serverId".to_string(), entry.key.to_string()));
    options.push(("serverName".to_string(), entry.name.to_string()));
    let Some(existing) = find_entry(&document, entry.key) else {
        let mut specs: Vec<XmlElementSpec> = options.iter().map(|(name, value)| return option_spec(name, value)).collect();
        specs.push(mappings_spec(entry.language_id, entry.file_names));
        return append_entry(path, &document, &entry_spec(entry.key, USER_CONTAINER, specs));
    };
    if document.descendants_named(existing, USER_CONTAINER).is_empty() {
        let mut specs: Vec<XmlElementSpec> = options.iter().map(|(name, value)| return option_spec(name, value)).collect();
        specs.push(mappings_spec(entry.language_id, entry.file_names));
        return replace_entry(path, &document, existing, &entry_spec(entry.key, USER_CONTAINER, specs));
    }
    let mut current = text.to_string();
    for (name, value) in &options {
        current = ensure_option(path, &current, entry.key, USER_CONTAINER, name, value, true)?;
    }
    return ensure_mappings(path, &current, entry.key, entry.language_id, entry.file_names);
}

/// Tests for the XML helpers.
#[cfg(test)]
#[path = "harper_xml_tests.rs"]
mod tests;
