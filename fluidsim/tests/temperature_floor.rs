//! An endergonic reaction (ΔG > 0) cools the cell by `-dG * extent / cp` every substep.
//! Nothing bounded that below, and the thermal pass reads stored temperatures raw when
//! accumulating flux, so a cell driven negative became a heat sink for its neighbours
//! before its own value was ever clamped.

use fluidsim::gpu::{GpuReactionRule, GpuReactionRuleConfig, GpuSimulation, GpuSimulationCreateInfo};

/// Matches `MIN_TEMPERATURE_KELVIN` in `reaction.comp` and `thermal_diffusion.comp`.
const MIN_TEMPERATURE_KELVIN: f32 = 1.0;

const INITIAL_TEMPERATURE: f32 = 293.15;

#[test]
fn endergonic_reaction_cannot_drive_temperature_below_the_floor() {
    let width = 4u32;
    let height = 4u32;
    let cell_count = (width * height) as usize;

    let initial_concentrations = vec![vec![1.0f32; cell_count]];
    let solid_mask = vec![0u32; cell_count];
    let material_ids = vec![0u32; cell_count];
    let diffusion_coeffs = vec![1.0f32];
    let species_charges = vec![0i32];

    let mut sim = match GpuSimulation::new(GpuSimulationCreateInfo {
        width,
        height,
        species_count: 1,
        initial_concentrations: &initial_concentrations,
        solid_mask_data: &solid_mask,
        material_ids_data: &material_ids,
        diffusion_coeffs_data: &diffusion_coeffs,
        species_charges_data: &species_charges,
    }) {
        Ok(sim) => sim,
        Err(error) => {
            eprintln!("skipping: no usable Vulkan device ({error})");
            return;
        }
    };

    sim.init_reaction_pipeline(&vec![INITIAL_TEMPERATURE; cell_count])
        .expect("reaction pipeline should initialize");

    // Strongly endergonic: ΔG = ΔH - TΔS = 1e7 J/mol with ΔS = 0. Consuming the full
    // unit of reactant yields dT = -1e7 / 4184 ≈ -2390 K, far past absolute zero from
    // a 293.15 K start.
    let rule = GpuReactionRule::new(GpuReactionRuleConfig {
        reactant_a_index: 0,
        reactant_b_index: None,
        product_a_index: None,
        product_b_index: None,
        catalyst_index: None,
        kinetic_model: 0, // mass action
        rate: 1.0,
        km_reactant_a: 0.0,
        km_reactant_b: 0.0,
        enthalpy: 1.0e7,
        entropy: 0.0,
    });
    sim.upload_reaction_rules(&[rule])
        .expect("rule should upload");

    sim.step_reactions(1.0).expect("reaction step should run");

    let temperatures = sim.read_temperatures().expect("temperatures should read back");
    assert_eq!(temperatures.len(), cell_count);

    for (index, &t) in temperatures.iter().enumerate() {
        assert!(
            t >= MIN_TEMPERATURE_KELVIN,
            "cell {index} fell to {t} K, below the {MIN_TEMPERATURE_KELVIN} K floor"
        );
    }

    // No assertion that the grid actually cooled. Thermodynamic gating now suppresses
    // this rule to an extent of ~0, so the floor is unreachable by this route - the
    // clamp is retained as defence in depth for any future path that writes a
    // temperature directly. See reaction_thermodynamics.rs for the gating itself.
}
