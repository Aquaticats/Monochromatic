//! Controls for the opt-in that turns the service path on.

use super::*;

/// Only the two explicit spellings turn it on, and only with what they need.
#[test]
fn only_the_explicit_values_turn_the_service_path_on() {
    let runtime: PathBuf = PathBuf::from("/run/user/1000");
    assert_eq!(service_address("1", Some(runtime.as_path())), Some(Bind::Path(runtime.join(SOCKET_RELATIVE))));
    assert_eq!(service_address("1", None), None, "no runtime directory, no address");
    assert_eq!(service_address("abstract:name", None), Some(Bind::Abstract(String::from("name"))));
    for value in ["", "0", "yes", "true", "abstract:", "path:/x"] {
        assert_eq!(service_address(value, Some(runtime.as_path())), None, "{value:?}");
    }
}
