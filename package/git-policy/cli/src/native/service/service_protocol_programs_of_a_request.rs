//! What:
//!  The programs and transport settings one request or wake carries,
//!  every name and value checked,
//!  read from the socket.
//! Why:
//!   These values become the environment of a child a push starts,
//!  so a name this release never carries and a value a line cannot hold whole are refused rather than passed on;
//! the values themselves live only in the service's memory (`service_push_programs_held_in_memory`).
//! It sits beside `service_protocol_decode_lines_to_messages.rs`,
//!  which reads every other field of those messages.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const programs = programsOf(value.programs); // PushPrograms, or a refusal
//! ```

use serde_json::Value;

/// What:
///  The programs and transport settings one request carries,
///  every name and value checked.
/// Why:
///   The line is a socket's:
/// a name this release would never carry is refused rather than put into a child's environment,
/// and so is a value that is not text or holds a NUL.
/// The flag reaches no file:
/// it says only that the command chose a transport.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const programsOf = (entry) => ({ transport: entry.transport === true, notText: (entry.not_text ?? []).map(checkName), variables: entry.variables.map(([name, value]) => { if (!CARRIED.includes(name)) throw ...; return [name, value]; }) });
/// ```
pub(crate) fn programs_of(entry: &Value) -> Result<crate::service_push_programs_held_in_memory::PushPrograms, String> {
    let mut variables: Vec<(String, String)> = Vec::new();
    for item in entry.get("variables").and_then(Value::as_array).ok_or("the programs carry no list of variables")? {
        let pair: &Vec<Value> = item.as_array().filter(|pair| { return pair.len() == 2; }).ok_or("a carried variable is not a name and a value")?;
        let (Some(name), Some(value)) = (pair[0].as_str(), pair[1].as_str()) else {
            return Err(String::from("a carried variable is not text"));
        };
        if !crate::service_push_programs_held_in_memory::CARRIED.contains(&name) {
            return Err(format!("a carried variable this release never carries: {name}"));
        }
        if value.contains('\0') {
            return Err(String::from("a carried variable holds a NUL"));
        }
        variables.push((String::from(name), String::from(value)));
    }
    // The names whose value could not travel, which a request that predates them leaves out entirely.
    let mut not_text: Vec<String> = Vec::new();
    if let Some(field) = entry.get("not_text") {
        // A field that is there and is not a list is a line this release cannot read, not an older request.
        let names: &Vec<Value> = field.as_array().ok_or("the names whose value could not travel are not a list")?;
        for item in names {
            let name: &str = item.as_str().ok_or("a name whose value could not travel is not text")?;
            if !crate::service_push_programs_held_in_memory::CARRIED.contains(&name) {
                return Err(format!("a variable this release never carries was named as one whose value could not travel: {name}"));
            }
            not_text.push(String::from(name));
        }
    }
    let transport: bool = entry.get("transport").and_then(Value::as_bool).unwrap_or(false);
    return Ok(crate::service_push_programs_held_in_memory::PushPrograms { variables, not_text, transport });
}
