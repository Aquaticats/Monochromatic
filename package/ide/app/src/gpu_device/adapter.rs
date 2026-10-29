//! What an adapter reports about itself,
//!  and the two variables the person may set to choose one.

/// The `wgpu` crate as Slint re-exports it,
///  so the device is built with exactly the wgpu release (30) that Slint's renderer was built against.
use slint::wgpu_30::wgpu;
/// Error text for the failures below.
use std::fmt;

/// The name of the variable that picks an adapter by its name,
///  as `wgpu::util::initialize_adapter_from_env` reads it (wgpu 30).
pub const NAME_VARIABLE: &str = "WGPU_ADAPTER_NAME";

/// The name of the variable that orders integrated and discrete GPUs,
///  as `wgpu::PowerPreference::from_env` reads it (wgpu 30).
pub const POWER_VARIABLE: &str = "WGPU_POWER_PREF";

/// The values of `WGPU_POWER_PREF` that wgpu 30 accepts,
///  in the order the error message names them.
pub const POWER_VALUES: &str = "low, high, none";

/// What:
///  Whether a kind of adapter is a candidate:
///  only discrete and integrated GPUs are.
/// Why:
///  Every other kind
///  (the CPU kind above all,
///  but also virtual and unknown ones)
///  is refused,
///  so no CPU adapter is ever chosen.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function isHardware(kind: DeviceType): boolean {
///   return kind === 'DiscreteGpu' || kind === 'IntegratedGpu';
/// }
/// ```
pub(super) fn is_hardware(kind: wgpu::DeviceType) -> bool {
    return kind == wgpu::DeviceType::DiscreteGpu || kind == wgpu::DeviceType::IntegratedGpu;
}

/// What:
///  What an adapter reports about itself,
///  as far as the choice and the error messages need it.
/// Why:
///  The choice is a pure function of these values,
///  so it is tested without a GPU.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type AdapterSummary = { name: string; deviceType: DeviceType; driver: string };
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AdapterSummary {
    /// The adapter's name,
    ///  such as `AMD Radeon RX 7600 (RADV NAVI33)`.
    pub name: String,
    /// The kind of device the driver reports.
    pub device_type: wgpu::DeviceType,
    /// The driver's name,
    ///  such as `radv` or `llvmpipe`.
    pub driver: String,
}

/// Writes an adapter the way the error messages list it.
impl fmt::Display for AdapterSummary {
    /// `name (kind, driver)`.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        return write!(
            formatter,
            "{} ({:?}, {})",
            self.name, self.device_type, self.driver
        );
    }
}

/// What:
///  The two variables the person may set to choose an adapter,
///  as text and exactly as read.
///  `Option<String>` is text or nothing (`string | undefined`).
/// Why:
///  The choice is a pure function of these values and the adapter list,
///  so tests pass them in and never read the process environment,
///  and the error messages show what was read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Variables = { adapterName?: string; powerPreference?: string };
/// ```
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Variables {
    /// The value of `WGPU_ADAPTER_NAME`,
    ///  or nothing when it is unset or empty.
    pub adapter_name: Option<String>,
    /// The value of `WGPU_POWER_PREF`,
    ///  or nothing when it is unset or empty.
    pub power_preference: Option<String>,
}

/// What:
///  `read_variable` reads one environment variable as text;
///  `None` when it is unset or empty.
///  `var_os` returns the raw operating-system string,
///  and `to_string_lossy` turns bytes that are not UTF-8 into replacement characters.
/// Why:
///  An empty value (`WGPU_POWER_PREF= ide`) is how a shell unsets a variable for one command.
///  Bytes that are not UTF-8 then match no adapter name and no power value,
///  so they become an error that names the value,
///  and never silently count as unset.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function readVariable(name: string): string | undefined {
///   const value = process.env[name];
///   return value === '' ? undefined : value;
/// }
/// ```
fn read_variable(name: &str) -> Option<String> {
    let value = std::env::var_os(name)?;
    let text = value.to_string_lossy().into_owned();
    if text.is_empty() {
        return None;
    }
    return Some(text);
}

/// Reads both variables from the process environment.
impl Variables {
    /// What:
    ///  The values of `WGPU_ADAPTER_NAME` and `WGPU_POWER_PREF` now.
    /// Why:
    ///  Only `open_hardware_device` calls it.
    ///  Everything else takes a `Variables` value as an argument.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function fromEnvironment(): Variables;
    /// ```
    pub fn from_environment() -> Self {
        return Self {
            adapter_name: read_variable(NAME_VARIABLE),
            power_preference: read_variable(POWER_VARIABLE),
        };
    }
}

/// Writes which variables were read and what they held.
impl fmt::Display for Variables {
    /// `WGPU_ADAPTER_NAME="x" or not set, WGPU_POWER_PREF="y" or not set`.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write_variable(formatter, NAME_VARIABLE, &self.adapter_name)?;
        write!(formatter, ", ")?;
        return write_variable(formatter, POWER_VARIABLE, &self.power_preference);
    }
}

/// What:
///  Write one variable as `NAME="value"`,
///  or `NAME not set`.
/// Why:
///  `Variables` shows both variables the same way.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function writeVariable(name: string, value: string | undefined): string;
/// ```
fn write_variable(
    formatter: &mut fmt::Formatter<'_>,
    name: &str,
    value: &Option<String>,
) -> fmt::Result {
    return match value {
        Some(text) => write!(formatter, "{name}=\"{text}\""),
        None => write!(formatter, "{name} not set"),
    };
}

/// What:
///  How `WGPU_POWER_PREF` orders the hardware adapters:
///  integrated first,
///  discrete first,
///  or the loader's order.
/// Why:
///  `low`,
///  `high`,
///  and `none` are the three values wgpu 30 accepts;
///  anything else is an error here (wgpu itself ignores it).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PowerOrder = 'integratedFirst' | 'discreteFirst' | 'loaderOrder';
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PowerOrder {
    /// `low`:
    ///  integrated GPUs before discrete ones.
    IntegratedFirst,
    /// `high`:
    ///  discrete GPUs before integrated ones.
    DiscreteFirst,
    /// `none`:
    ///  no preference,
    ///  the order the Vulkan loader lists.
    LoaderOrder,
}

/// What:
///  Parse a `WGPU_POWER_PREF` value the way `wgpu::PowerPreference::from_env` does
///  (case-insensitive `low`,
///  `high`,
///  `none`),
///  but say so when the value is none of them.
/// Why:
///  wgpu returns nothing for both "unset" and "garbage",
///  so a misspelled variable would silently choose nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parsePowerOrder(value: string): PowerOrder | undefined;
/// ```
pub(super) fn parse_power_order(value: &str) -> Option<PowerOrder> {
    return match value.to_lowercase().as_str() {
        "low" => Some(PowerOrder::IntegratedFirst),
        "high" => Some(PowerOrder::DiscreteFirst),
        "none" => Some(PowerOrder::LoaderOrder),
        _ => None,
    };
}
