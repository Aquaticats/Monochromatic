//! The adapter choice:
//!  which adapters may be used and in which order,
//!  decided from adapter descriptions and variable values alone.

/// The adapter descriptions and the variables the choice reads.
use super::adapter::{
    AdapterSummary, NAME_VARIABLE, POWER_VARIABLE, PowerOrder, Variables, is_hardware,
    parse_power_order,
};
/// The failures the choice reports.
use super::error::{GpuDeviceError, SelectionReport};
/// The `wgpu` crate as Slint re-exports it.
use slint::wgpu_30::wgpu;

/// What:
///  The adapters in the order they are tried,
///  and the facts the reason for the choice is made from.
///  `Vec<usize>` is an array of indexes into the loader's list.
/// Why:
///  The device opens on the first adapter that accepts it,
///  and the launcher logs why that one was first.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type AdapterOrder = { indexes: number[]; variables: Variables; power: PowerOrder | undefined };
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AdapterOrder {
    /// Indexes into the loader's list,
    ///  in the order to try them.
    pub indexes: Vec<usize>,
    /// The variables that were read.
    pub variables: Variables,
    /// The parsed `WGPU_POWER_PREF`,
    ///  or nothing when it was not set.
    pub power: Option<PowerOrder>,
}

/// The reason for the choice.
impl AdapterOrder {
    /// What:
    ///  Say in one sentence why the adapter at `position` in this order (index `loader_index` in the loader's list)
    ///  is the one in use.
    /// Why:
    ///  The launcher logs it,
    ///  so a person can see whether the system's order or a variable decided.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function why(position: number, loaderIndex: number): string;
    /// ```
    pub fn why(&self, position: usize, loader_index: usize) -> String {
        let mut parts: Vec<String> = Vec::new();
        if let Some(name) = &self.variables.adapter_name {
            parts.push(format!(
                "{NAME_VARIABLE}=\"{name}\" keeps only the hardware adapters whose name contains it"
            ));
        }
        match self.power {
            Some(PowerOrder::IntegratedFirst) => parts.push(format!(
                "{POWER_VARIABLE}={} puts integrated GPUs first",
                self.variables.power_preference.clone().unwrap_or_default()
            )),
            Some(PowerOrder::DiscreteFirst) => parts.push(format!(
                "{POWER_VARIABLE}={} puts discrete GPUs first",
                self.variables.power_preference.clone().unwrap_or_default()
            )),
            Some(PowerOrder::LoaderOrder) => parts.push(format!(
                "{POWER_VARIABLE}={} keeps the loader's order",
                self.variables.power_preference.clone().unwrap_or_default()
            )),
            None => {}
        }
        if self.variables.adapter_name.is_none()
            && !matches!(
                self.power,
                Some(PowerOrder::IntegratedFirst | PowerOrder::DiscreteFirst)
            )
        {
            parts.push(
                "no variable reordered the adapters, so the system's choice applies: the Vulkan loader's order \
                 (Mesa puts the display server's GPU first, or the one DRI_PRIME or MESA_VK_DEVICE_SELECT names)"
                    .to_string(),
            );
        }
        let mut place = format!(
            "candidate {} of {} (index {} in the loader's list)",
            position + 1,
            self.indexes.len(),
            loader_index
        );
        if position > 0 {
            place.push_str(&format!(
                ", chosen after the {position} before it refused the device"
            ));
        }
        parts.push(place);
        return parts.join("; ");
    }
}

/// What:
///  The candidates with the adapters of one kind moved to the front;
///  each group keeps the loader's order.
/// Why:
///  `WGPU_POWER_PREF=low` and `high` move a kind first and drop nothing,
///  so the other kind stays behind as the next adapter to try.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function kindFirst(adapters: AdapterSummary[], candidates: number[], kind: DeviceType): number[];
/// ```
fn kind_first(
    adapters: &[AdapterSummary],
    candidates: &[usize],
    kind: wgpu::DeviceType,
) -> Vec<usize> {
    let mut front: Vec<usize> = Vec::new();
    let mut back: Vec<usize> = Vec::new();
    for index in candidates {
        if adapters[*index].device_type == kind {
            front.push(*index);
        } else {
            back.push(*index);
        }
    }
    front.extend(back);
    return front;
}

/// What:
///  `choose_adapters` decides which adapters may be used and in which order.
///  `&[T]` is a borrowed slice (a read-only view of an array).
///  `Result<T, E>` is a value or an error (a function that may throw).
/// Why:
///  The candidates are the hardware adapters (discrete and integrated) in the loader's order.
///  `WGPU_ADAPTER_NAME` keeps those whose name contains the value (case-insensitive),
///  and `WGPU_POWER_PREF` moves integrated (`low`) or discrete (`high`) GPUs first,
///  both without dropping the rest.
///  A machine that lists only CPU adapters gets an error,
///  never its first adapter,
///  and a name that matches only a CPU,
///  virtual,
///  or unknown adapter gets one too.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function chooseAdapters(adapters: AdapterSummary[], variables: Variables): AdapterOrder; // throws GpuDeviceError
/// ```
pub fn choose_adapters(
    adapters: &[AdapterSummary],
    variables: &Variables,
) -> Result<AdapterOrder, GpuDeviceError> {
    let power = match &variables.power_preference {
        None => None,
        Some(value) => match parse_power_order(value) {
            Some(order) => Some(order),
            None => {
                return Err(GpuDeviceError::BadPowerPreference {
                    value: value.clone(),
                });
            }
        },
    };
    // The name keeps the adapters whose lowercase name contains the lowercase value, as wgpu's helper does.
    let mut matching: Vec<usize> = Vec::new();
    for (index, adapter) in adapters.iter().enumerate() {
        let named = match &variables.adapter_name {
            None => true,
            Some(wanted) => adapter.name.to_lowercase().contains(&wanted.to_lowercase()),
        };
        if named {
            matching.push(index);
        }
    }
    let mut candidates: Vec<usize> = Vec::new();
    let mut refused_kinds: Vec<String> = Vec::new();
    for index in matching {
        if is_hardware(adapters[index].device_type) {
            candidates.push(index);
        } else {
            refused_kinds.push(adapters[index].to_string());
        }
    }
    if candidates.is_empty() {
        let report = SelectionReport::new(adapters, &[], variables.clone());
        if variables.adapter_name.is_none() {
            return Err(GpuDeviceError::NoHardwareAdapter { report });
        }
        if refused_kinds.is_empty() {
            return Err(GpuDeviceError::AdapterNameUnmatched { report });
        }
        return Err(GpuDeviceError::AdapterNameMatchesOnlyRefusedKinds {
            report,
            matched: refused_kinds,
        });
    }
    match power {
        Some(PowerOrder::IntegratedFirst) => {
            candidates = kind_first(adapters, &candidates, wgpu::DeviceType::IntegratedGpu);
        }
        Some(PowerOrder::DiscreteFirst) => {
            candidates = kind_first(adapters, &candidates, wgpu::DeviceType::DiscreteGpu);
        }
        Some(PowerOrder::LoaderOrder) | None => {}
    }
    return Ok(AdapterOrder {
        indexes: candidates,
        variables: variables.clone(),
        power,
    });
}
