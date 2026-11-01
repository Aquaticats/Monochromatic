//! The adapter choice is decided from adapter descriptions and variable values alone,
//!  so these tests need no GPU and no display,
//!  and none of them reads the process environment.

/// The choice and the errors under test,
///  and the description of the device that is opened.
use super::{AdapterSummary, GpuDeviceError, Variables, choose_adapters, device_description};
/// The power order the choice parses and the order it returns.
use super::{adapter::PowerOrder, choice::AdapterOrder};
/// The adapter kinds,
///  the limits,
///  and the allocator's memory hint.
use slint::wgpu_30::wgpu::{DeviceType, Limits, MemoryHints};

/// An adapter description of the given kind.
fn adapter(name: &str, device_type: DeviceType, driver: &str) -> AdapterSummary {
    return AdapterSummary {
        name: name.to_string(),
        device_type,
        driver: driver.to_string(),
    };
}

/// The variables as a shell would have set them.
fn variables(adapter_name: Option<&str>, power_preference: Option<&str>) -> Variables {
    return Variables {
        adapter_name: adapter_name.map(str::to_string),
        power_preference: power_preference.map(str::to_string),
    };
}

/// No variable set.
fn nothing_set() -> Variables {
    return Variables::default();
}

/// What a desktop laptop lists when the display runs on the integrated GPU.
fn integrated_listed_first() -> [AdapterSummary; 3] {
    return [
        adapter(
            "AMD Radeon Graphics (RADV RAPHAEL_MENDOCINO)",
            DeviceType::IntegratedGpu,
            "radv",
        ),
        adapter("NVIDIA GeForce RTX 4060", DeviceType::DiscreteGpu, "nvidia"),
        adapter("llvmpipe (LLVM 19.1.7)", DeviceType::Cpu, "llvmpipe"),
    ];
}

/// What the loader lists after `DRI_PRIME=1` put the discrete GPU first.
fn discrete_listed_first() -> [AdapterSummary; 3] {
    return [
        adapter("NVIDIA GeForce RTX 4060", DeviceType::DiscreteGpu, "nvidia"),
        adapter(
            "AMD Radeon Graphics (RADV RAPHAEL_MENDOCINO)",
            DeviceType::IntegratedGpu,
            "radv",
        ),
        adapter("llvmpipe (LLVM 19.1.7)", DeviceType::Cpu, "llvmpipe"),
    ];
}

/// The indexes of a successful choice.
fn indexes(listed: &[AdapterSummary], set: &Variables) -> Vec<usize> {
    return choose_adapters(listed, set)
        .unwrap_or_else(|error| panic!("the choice failed: {error}"))
        .indexes;
}

/// The error of a failed choice.
fn failure(listed: &[AdapterSummary], set: &Variables) -> GpuDeviceError {
    return match choose_adapters(listed, set) {
        Ok(order) => panic!("the choice must fail, but gave {order:?}"),
        Err(error) => error,
    };
}

#[test]
fn the_loader_order_is_kept_with_the_integrated_adapter_listed_first() {
    assert_eq!(
        indexes(&integrated_listed_first(), &nothing_set()),
        vec![0, 1]
    );
}

#[test]
fn the_loader_order_is_kept_with_the_discrete_adapter_listed_first() {
    assert_eq!(
        indexes(&discrete_listed_first(), &nothing_set()),
        vec![0, 1]
    );
}

#[test]
fn equal_kinds_keep_the_loader_order_and_cpu_adapters_are_dropped_from_between() {
    let listed = [
        adapter("Integrated A", DeviceType::IntegratedGpu, "radv"),
        adapter("Discrete A", DeviceType::DiscreteGpu, "radv"),
        adapter("llvmpipe", DeviceType::Cpu, "llvmpipe"),
        adapter("Integrated B", DeviceType::IntegratedGpu, "anv"),
        adapter("Discrete B", DeviceType::DiscreteGpu, "nvidia"),
    ];
    assert_eq!(indexes(&listed, &nothing_set()), vec![0, 1, 3, 4]);
}

#[test]
fn a_machine_that_lists_only_a_cpu_adapter_gets_no_hardware_adapter_error() {
    let listed = [adapter(
        "llvmpipe (LLVM 19.1.7)",
        DeviceType::Cpu,
        "llvmpipe",
    )];
    assert!(
        matches!(
            failure(&listed, &nothing_set()),
            GpuDeviceError::NoHardwareAdapter { .. }
        ),
        "a CPU adapter must never be chosen"
    );
}

#[test]
fn a_cpu_adapter_listed_first_is_skipped_for_the_hardware_one() {
    let listed = [
        adapter("llvmpipe (LLVM 19.1.7)", DeviceType::Cpu, "llvmpipe"),
        adapter("AMD Radeon RX 7600", DeviceType::DiscreteGpu, "radv"),
    ];
    assert_eq!(indexes(&listed, &nothing_set()), vec![1]);
}

#[test]
fn unknown_and_virtual_kinds_are_not_hardware_either() {
    let listed = [
        adapter("something", DeviceType::Other, "unknown"),
        adapter("virtio", DeviceType::VirtualGpu, "venus"),
    ];
    assert!(matches!(
        failure(&listed, &nothing_set()),
        GpuDeviceError::NoHardwareAdapter { .. }
    ));
}

#[test]
fn no_variable_value_lets_a_cpu_or_virtual_adapter_be_chosen() {
    let listed = [
        adapter("llvmpipe (LLVM 19.1.7)", DeviceType::Cpu, "llvmpipe"),
        adapter("virtio", DeviceType::VirtualGpu, "venus"),
        adapter("something", DeviceType::Other, "unknown"),
        adapter("AMD Radeon RX 7600", DeviceType::DiscreteGpu, "radv"),
    ];
    for set in [
        nothing_set(),
        variables(Some("llvmpipe"), None),
        variables(None, Some("low")),
        variables(None, Some("high")),
        variables(None, Some("none")),
        variables(Some("a"), Some("high")),
    ] {
        if let Ok(order) = choose_adapters(&listed, &set) {
            assert_eq!(order.indexes, vec![3], "with {set}");
        }
    }
}

#[test]
fn power_preference_low_puts_the_integrated_adapter_first_in_either_loader_order() {
    assert_eq!(
        indexes(&integrated_listed_first(), &variables(None, Some("low"))),
        vec![0, 1]
    );
    assert_eq!(
        indexes(&discrete_listed_first(), &variables(None, Some("low"))),
        vec![1, 0]
    );
}

#[test]
fn power_preference_high_puts_the_discrete_adapter_first_in_either_loader_order() {
    assert_eq!(
        indexes(&integrated_listed_first(), &variables(None, Some("high"))),
        vec![1, 0]
    );
    assert_eq!(
        indexes(&discrete_listed_first(), &variables(None, Some("high"))),
        vec![0, 1]
    );
}

#[test]
fn power_preference_none_and_any_letter_case_are_accepted_as_wgpu_accepts_them() {
    assert_eq!(
        indexes(&discrete_listed_first(), &variables(None, Some("none"))),
        vec![0, 1]
    );
    assert_eq!(
        indexes(&discrete_listed_first(), &variables(None, Some("LOW"))),
        vec![1, 0]
    );
    assert_eq!(
        indexes(&integrated_listed_first(), &variables(None, Some("High"))),
        vec![1, 0]
    );
}

#[test]
fn power_preference_keeps_loader_order_among_equal_kinds() {
    let listed = [
        adapter("Discrete A", DeviceType::DiscreteGpu, "radv"),
        adapter("Integrated A", DeviceType::IntegratedGpu, "radv"),
        adapter("Discrete B", DeviceType::DiscreteGpu, "nvidia"),
        adapter("Integrated B", DeviceType::IntegratedGpu, "anv"),
    ];
    assert_eq!(
        indexes(&listed, &variables(None, Some("low"))),
        vec![1, 3, 0, 2]
    );
    assert_eq!(
        indexes(&listed, &variables(None, Some("high"))),
        vec![0, 2, 1, 3]
    );
}

#[test]
fn a_bad_power_preference_is_an_error_that_names_the_accepted_values() {
    let error = failure(&integrated_listed_first(), &variables(None, Some("fast")));
    assert_eq!(
        error,
        GpuDeviceError::BadPowerPreference {
            value: "fast".to_string()
        }
    );
    let message = error.to_string();
    for needle in ["WGPU_POWER_PREF", "\"fast\"", "low, high, none", "unset"] {
        assert!(message.contains(needle), "missing {needle:?} in: {message}");
    }
}

#[test]
fn the_adapter_name_matches_a_substring_ignoring_letter_case() {
    let listed = integrated_listed_first();
    assert_eq!(
        indexes(&listed, &variables(Some("geforce rtx"), None)),
        vec![1]
    );
    assert_eq!(
        indexes(&listed, &variables(Some("RAPHAEL_mendocino"), None)),
        vec![0]
    );
}

#[test]
fn the_adapter_name_wins_over_the_loader_order() {
    assert_eq!(
        indexes(&integrated_listed_first(), &variables(Some("nvidia"), None)),
        vec![1]
    );
    assert_eq!(
        indexes(&discrete_listed_first(), &variables(Some("amd"), None)),
        vec![1]
    );
}

#[test]
fn an_adapter_name_matching_two_hardware_adapters_keeps_both_in_loader_order() {
    let listed = [
        adapter(
            "AMD Radeon RX 7600 (RADV NAVI33)",
            DeviceType::DiscreteGpu,
            "radv",
        ),
        adapter(
            "AMD Radeon Graphics (RADV RAPHAEL)",
            DeviceType::IntegratedGpu,
            "radv",
        ),
        adapter("llvmpipe", DeviceType::Cpu, "llvmpipe"),
    ];
    assert_eq!(indexes(&listed, &variables(Some("radv"), None)), vec![0, 1]);
}

#[test]
fn the_adapter_name_filters_and_the_power_preference_orders_what_is_left() {
    let listed = [
        adapter(
            "AMD Radeon RX 7600 (RADV NAVI33)",
            DeviceType::DiscreteGpu,
            "radv",
        ),
        adapter(
            "AMD Radeon Graphics (RADV RAPHAEL)",
            DeviceType::IntegratedGpu,
            "radv",
        ),
        adapter("NVIDIA GeForce RTX 4060", DeviceType::DiscreteGpu, "nvidia"),
    ];
    assert_eq!(
        indexes(&listed, &variables(Some("radv"), Some("low"))),
        vec![1, 0]
    );
}

#[test]
fn an_unmatched_adapter_name_is_an_error_that_names_the_adapters_seen() {
    let error = failure(&integrated_listed_first(), &variables(Some("quadro"), None));
    assert!(
        matches!(error, GpuDeviceError::AdapterNameUnmatched { .. }),
        "{error:?}"
    );
    let message = error.to_string();
    for needle in [
        "WGPU_ADAPTER_NAME=\"quadro\"",
        "matches the name of no Vulkan adapter",
        "AMD Radeon Graphics (RADV RAPHAEL_MENDOCINO) (IntegratedGpu, radv)",
        "NVIDIA GeForce RTX 4060 (DiscreteGpu, nvidia)",
        "llvmpipe (LLVM 19.1.7) (Cpu, llvmpipe)",
        "unset WGPU_ADAPTER_NAME",
    ] {
        assert!(message.contains(needle), "missing {needle:?} in: {message}");
    }
}

#[test]
fn an_adapter_name_matching_only_a_cpu_adapter_is_refused_like_any_cpu_adapter() {
    let error = failure(
        &integrated_listed_first(),
        &variables(Some("LLVMPIPE"), None),
    );
    assert!(
        matches!(
            &error,
            GpuDeviceError::AdapterNameMatchesOnlyRefusedKinds { matched, .. }
                if matched == &vec!["llvmpipe (LLVM 19.1.7) (Cpu, llvmpipe)".to_string()]
        ),
        "{error:?}"
    );
    let message = error.to_string();
    for needle in [
        "refuses on purpose",
        "llvmpipe (LLVM 19.1.7) (Cpu, llvmpipe)",
        "SLINT_WGPU_CPU",
        "NVIDIA GeForce RTX 4060",
    ] {
        assert!(message.contains(needle), "missing {needle:?} in: {message}");
    }
}

#[test]
fn an_adapter_name_matching_only_a_virtual_adapter_is_refused_too() {
    let listed = [
        adapter("virtio", DeviceType::VirtualGpu, "venus"),
        adapter("AMD Radeon RX 7600", DeviceType::DiscreteGpu, "radv"),
    ];
    assert!(matches!(
        failure(&listed, &variables(Some("virtio"), None)),
        GpuDeviceError::AdapterNameMatchesOnlyRefusedKinds { .. }
    ));
}

#[test]
fn a_name_matching_both_a_cpu_and_a_hardware_adapter_keeps_only_the_hardware_one() {
    let listed = [
        adapter("llvmpipe pro", DeviceType::Cpu, "llvmpipe"),
        adapter("Pro Card", DeviceType::DiscreteGpu, "radv"),
    ];
    assert_eq!(indexes(&listed, &variables(Some("pro"), None)), vec![1]);
}

#[test]
fn a_bad_power_preference_wins_over_every_other_failure() {
    let listed = [adapter("llvmpipe", DeviceType::Cpu, "llvmpipe")];
    assert!(matches!(
        failure(&listed, &variables(Some("zzz"), Some("fast"))),
        GpuDeviceError::BadPowerPreference { .. }
    ));
}

#[test]
fn the_reason_names_the_loader_order_when_no_variable_is_set() {
    let order =
        choose_adapters(&integrated_listed_first(), &nothing_set()).expect("hardware is listed");
    let reason = order.why(0, 0);
    for needle in [
        "no variable reordered the adapters",
        "Vulkan loader's order",
        "candidate 1 of 2",
        "index 0 in the loader's list",
    ] {
        assert!(reason.contains(needle), "missing {needle:?} in: {reason}");
    }
    let later = order.why(1, 1);
    assert!(
        later.contains("chosen after the 1 before it refused the device"),
        "{later}"
    );
}

#[test]
fn the_reason_names_the_variable_that_decided() {
    let by_name = choose_adapters(&integrated_listed_first(), &variables(Some("rtx"), None))
        .expect("the name matches");
    assert!(
        by_name
            .why(0, 1)
            .contains("WGPU_ADAPTER_NAME=\"rtx\" keeps only"),
        "{}",
        by_name.why(0, 1)
    );
    let by_power = choose_adapters(&integrated_listed_first(), &variables(None, Some("high")))
        .expect("high is accepted");
    assert!(
        by_power
            .why(0, 1)
            .contains("WGPU_POWER_PREF=high puts discrete GPUs first"),
        "{}",
        by_power.why(0, 1)
    );
    let by_low = choose_adapters(&integrated_listed_first(), &variables(None, Some("low")))
        .expect("low is accepted");
    assert!(
        by_low
            .why(0, 0)
            .contains("WGPU_POWER_PREF=low puts integrated GPUs first"),
        "{}",
        by_low.why(0, 0)
    );
    let by_none = choose_adapters(&integrated_listed_first(), &variables(None, Some("none")))
        .expect("none is accepted");
    let none_reason = by_none.why(0, 0);
    assert!(
        none_reason.contains("WGPU_POWER_PREF=none keeps the loader's order")
            && none_reason.contains("no variable reordered the adapters"),
        "{none_reason}"
    );
}

#[test]
fn the_choice_remembers_the_parsed_power_preference() {
    let order: AdapterOrder =
        choose_adapters(&integrated_listed_first(), &variables(None, Some("LOW")))
            .expect("low is accepted");
    assert_eq!(order.power, Some(PowerOrder::IntegratedFirst));
}

/// Every remedy that every message about a hardware choice must carry.
const CHOICE_REMEDIES: [&str; 6] = [
    "WGPU_ADAPTER_NAME",
    "WGPU_POWER_PREF",
    "DRI_PRIME",
    "MESA_VK_DEVICE_SELECT",
    "Variables read:",
    "Adapters seen, in the Vulkan loader's order",
];

#[test]
fn the_message_without_a_hardware_adapter_names_the_adapters_seen_and_every_remedy() {
    let listed = [adapter(
        "llvmpipe (LLVM 19.1.7)",
        DeviceType::Cpu,
        "llvmpipe",
    )];
    let message = failure(&listed, &nothing_set()).to_string();
    for needle in [
        "llvmpipe (LLVM 19.1.7) (Cpu, llvmpipe)",
        "refused on purpose",
        "mesa-vulkan-drivers",
        "vulkan-loader",
        "--device /dev/dri",
        "vulkaninfo --summary",
        "VK_DRIVER_FILES",
        "WGPU_ADAPTER_NAME not set, WGPU_POWER_PREF not set",
        "Order used: none",
    ]
    .into_iter()
    .chain(CHOICE_REMEDIES)
    {
        assert!(message.contains(needle), "missing {needle:?} in: {message}");
    }
}

#[test]
fn the_message_with_no_adapter_at_all_says_the_loader_found_no_driver() {
    let message = failure(&[], &nothing_set()).to_string();
    assert!(message.contains("found no driver at all"), "{message}");
}

#[test]
fn the_message_of_a_refused_device_names_each_adapter_the_order_used_and_every_remedy() {
    let listed = integrated_listed_first();
    let order =
        choose_adapters(&listed, &variables(None, Some("high"))).expect("hardware is listed");
    let report =
        super::SelectionReport::new(&listed, &order.indexes, variables(None, Some("high")));
    let message = GpuDeviceError::DeviceRefused {
        refusals: vec![
            "Adapter A: limit max_storage_buffers_per_shader_stage".to_string(),
            "Adapter B: out of memory".to_string(),
        ],
        report,
    }
    .to_string();
    for needle in [
        "Adapter A: limit",
        "Adapter B: out of memory",
        "update the vulkan driver",
        "WGPU_POWER_PREF=\"high\"",
        "Order used: NVIDIA GeForce RTX 4060 (DiscreteGpu, nvidia), AMD Radeon Graphics",
    ]
    .into_iter()
    .chain(CHOICE_REMEDIES)
    {
        assert!(
            message.to_lowercase().contains(&needle.to_lowercase()),
            "missing {needle:?} in: {message}"
        );
    }
}

#[test]
fn the_name_errors_carry_every_remedy_too() {
    let unmatched =
        failure(&integrated_listed_first(), &variables(Some("quadro"), None)).to_string();
    let only_cpu = failure(
        &integrated_listed_first(),
        &variables(Some("llvmpipe"), None),
    )
    .to_string();
    for message in [unmatched, only_cpu] {
        for needle in CHOICE_REMEDIES {
            assert!(message.contains(needle), "missing {needle:?} in: {message}");
        }
    }
}

/// The device asks wgpu's allocator for small blocks:
/// with the default hint a bare device holds 193.6 MiB of video memory and a process 458.2 MiB,
/// with this one 13.6 and 234.4.
#[test]
fn the_device_description_asks_for_the_memory_usage_hint_and_keeps_the_limits() {
    let limits = Limits {
        max_texture_dimension_2d: 16_384,
        ..Limits::default()
    };
    let description = device_description(limits);
    assert!(
        matches!(description.memory_hints, MemoryHints::MemoryUsage),
        "the hint is {:?}",
        description.memory_hints
    );
    assert_eq!(description.required_limits.max_texture_dimension_2d, 16_384);
    assert_eq!(description.label, Some("ide gpu device"));
}
