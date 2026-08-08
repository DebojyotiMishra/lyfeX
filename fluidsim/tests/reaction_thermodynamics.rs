//! The reaction extent used to depend only on the rate constant, so a rule with
//! strongly positive ΔG ran uphill at full speed. The extent is now scaled by
//! exp(-ΔG/RT) when ΔG > 0.

use fluidsim::gpu::{GpuReactionRule, GpuReactionRuleConfig, GpuSimulation, GpuSimulationCreateInfo};

const INITIAL_CONCENTRATION: f32 = 1.0;
const INITIAL_TEMPERATURE: f32 = 293.15;

/// Runs one mass-action step of `A -> (nothing)` at the given ΔH with ΔS = 0,
/// and returns the mean remaining concentration of A. `None` if there is no device.
fn remaining_reactant_after_one_step(enthalpy_j_per_mol: f32) -> Option<f32> {
    let width = 4u32;
    let height = 4u32;
    let cell_count = (width * height) as usize;

    let initial_concentrations = vec![vec![INITIAL_CONCENTRATION; cell_count]];
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
            return None;
        }
    };

    sim.init_reaction_pipeline(&vec![INITIAL_TEMPERATURE; cell_count])
        .expect("reaction pipeline should initialize");

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
        enthalpy: enthalpy_j_per_mol,
        entropy: 0.0,
    });
    sim.upload_reaction_rules(&[rule])
        .expect("rule should upload");

    sim.step_reactions(1.0).expect("reaction step should run");

    let concentrations = sim
        .read_concentrations()
        .expect("concentrations should read back");
    let species_a = &concentrations[0];
    Some(species_a.iter().sum::<f32>() / species_a.len() as f32)
}

#[test]
fn endergonic_reactions_are_suppressed_and_exergonic_ones_are_not() {
    // ΔG = 0: exp(0) = 1, so the gate is inert and the reaction consumes everything
    // it would have before this change. This is the case every bundled scenario hits,
    // since the default kinetics path supplies ΔH = ΔS = 0.
    let Some(neutral) = remaining_reactant_after_one_step(0.0) else {
        return;
    };
    assert!(
        neutral < 0.01,
        "thermoneutral reaction should run to completion, {neutral} left"
    );

    // Strongly exergonic: also unaffected, since the gate only applies for ΔG > 0.
    let exergonic = remaining_reactant_after_one_step(-5.0e4).expect("device was available");
    assert!(
        exergonic < 0.01,
        "exergonic reaction should run to completion, {exergonic} left"
    );

    // Endergonic at a realistic magnitude: exp(-5e4 / (8.314 * 293.15)) ~ 1e-9, so the
    // reactant should be essentially untouched. Before this change it was consumed just
    // as fast as the exergonic case.
    let endergonic = remaining_reactant_after_one_step(5.0e4).expect("device was available");
    assert!(
        endergonic > 0.99,
        "endergonic reaction should be suppressed, only {endergonic} left"
    );
}
