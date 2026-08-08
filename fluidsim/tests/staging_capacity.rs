//! The staging buffer is shared by every host->device upload. On a grid small enough
//! that the concentration buffer is smaller than the reaction-rule payload, uploading a
//! full rule set used to overrun it.

use fluidsim::gpu::{GpuReactionRule, GpuReactionRuleConfig, GpuSimulation, GpuSimulationCreateInfo};

/// Matches `MAX_REACTION_RULES` in `fluidsim::gpu`.
const MAX_REACTION_RULES: usize = 16;

#[test]
fn full_rule_set_uploads_on_a_grid_smaller_than_the_rule_payload() {
    // 4x4 with one species is 64 bytes of concentration, against 16 rules x 44 bytes
    // = 704 bytes of rule data. Before the fix the staging buffer was sized from the
    // former and the rule upload panicked slicing past the end of the mapping.
    let width = 4u32;
    let height = 4u32;
    let cell_count = (width * height) as usize;

    let initial_concentrations = vec![vec![0.0f32; cell_count]];
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

    // Also routed through the shared staging buffer, and the prerequisite for
    // uploading rules at all.
    sim.init_reaction_pipeline(&vec![293.15f32; cell_count])
        .expect("reaction pipeline should initialize");

    let rules: Vec<GpuReactionRule> = (0..MAX_REACTION_RULES)
        .map(|_| {
            GpuReactionRule::new(GpuReactionRuleConfig {
                reactant_a_index: 0,
                reactant_b_index: None,
                product_a_index: None,
                product_b_index: None,
                catalyst_index: None,
                kinetic_model: 0,
                rate: 0.0,
                km_reactant_a: 0.0,
                km_reactant_b: 0.0,
                enthalpy: 0.0,
                entropy: 0.0,
            })
        })
        .collect();

    sim.upload_reaction_rules(&rules)
        .expect("a full rule set should upload on a small grid");
}
