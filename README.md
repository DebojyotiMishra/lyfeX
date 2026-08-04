# Lyfe
[![Status: Actively Maintained](https://img.shields.io/badge/status-actively%20maintained-2ea44f)](https://github.com/thavlik/lyfe/pulse)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-0366d6)](https://github.com/thavlik/lyfe#license)

Lyfe is a GPU-accelerated 2D chemical transport sandbox written in Rust on top of Vulkan. The project currently includes a Lean-backed reaction layer, coarse semantic snapshots, thermal transport, membrane leak channels, and moving enzyme entities. It represents "pre-production R&D" for a video game.

At a high level:

- `fluidsim` runs the fine-grid simulation on the GPU.
- `kinetics` builds coarse semantic snapshots and invokes Lean for low-frequency rule evaluation.
- `lean` contains the `lyfe-rules` executable that decides which reactions are active.
- `renderer` visualizes concentrations and temperature.
- `demo` ties everything together into an interactive desktop application.

## Screenshots
<p>
  <img width="200" src="images/screenshot_0_enzymes.webp">
  <img width="200" src="images/screenshot_1_leak_channels.webp">
</p>

## Current Feature Set

### Fine-grid GPU simulation

- Multi-species transport on a dense `[species][cell]` buffer layout.
- Explicit diffusion on Vulkan compute shaders with ping-pong buffers.
- Solid geometry and material masks for impermeable walls and embedded structures.
- Per-cell temperature field with a separate thermal diffusion pass.
- Optional charge-correction / electrochemical transport heuristics for ionic systems.
- Shared render/simulation Vulkan context so rendering can bind live simulation buffers directly.

### Lean-backed kinetics and semantics

- `fluidsim` builds a coarse semantic snapshot once per simulated second.
- `kinetics` serializes that snapshot to JSON and sends it to Lean (`lyfe-rules`)
- Lean returns compact reaction directives instead of replacing the simulation state.
- The returned directives can carry:
  - Mass-action or Michaelis-Menten kinetics.
  - Tile-local applicability.
  - Thermodynamic metadata such as $\Delta H$, $\Delta G$, $\Delta S$, and activation energy.
- The GPU reaction pass consumes those directives and updates concentration and temperature fields in-place.

### Chemistry currently implemented

- Strong acid/base neutralization: $\mathrm{H^+ + OH^- \rightarrow H_2O}$.
- Weak-acid buffer behavior for acetic acid / acetate systems.
- Direct neutralization of acetic acid by hydroxide.
- Catalyst-gated phosphorylation rule for hexokinase.
- Michaelis-Menten support for catalyst-driven reactions.

### Membranes, leaks, and enzymes

- Leak channels embedded in solid boundaries for directional transport experiments.
- Electrochemical leak heuristics that preserve directional flow while damping unstable local charge separation.
- Moving enzyme entities with drift, rotation, and thermal/circulation heuristics.
- Enzyme-specific GPU pass for entity-mediated catalysis separate from dissolved catalyst rules.

### Inspection and debugging

- Async coarse inspection readback for hover tooltips.
- Detail mode with pinned probe callouts around the simulation viewport.
- Thermal visualization overlay.
- Performance overlay for frame-time monitoring.
- Smoke-test mode that renders a few frames and exits.

### UI and visual design

- Apple-inspired dark theme: translucent "glass" panels, rounded controls, a
  systemBlue accent, and rounded corners on the simulation's own wall geometry
  (not just the UI chrome).
- Geist and Geist Mono are embedded directly in the binary (`demo/assets/fonts`,
  SIL Open Font License) and used for all HUD text.
- All styling is centralized in `demo/src/theme.rs` (fonts, colors, corner
  radii) so panels stay visually consistent as the UI grows.

## Workspace Layout

- `fluidsim`: core simulation crate.
  - GPU transport, reaction, leak, enzyme, and thermal compute passes.
  - Scenario builders and coarse semantic snapshot generation, including
    rounded-corner solid geometry (`solid.rs::fill_rounded_hollow_rect`).
  - Inspection, material, and species registries.
- `kinetics`: low-frequency semantic evaluation crate.
  - Snapshot/update types.
  - Lean bridge and evaluator.
  - Rule-engine configuration and diagnostics.
- `lean`: Lean 4 rule engine.
  - Owns the semantic rule definitions.
- `renderer`: Vulkan rendering and egui overlay crate.
  - `context.rs` creates the Vulkan instance/device/swapchain; surface
    creation is implemented for Linux (Xlib/Xcb/Wayland) and macOS
    (`VK_EXT_metal_surface` via MoltenVK).
- `demo`: interactive application, scenario runner, and UI/theme.

## Simulation Flow

Each frame, the demo advances the fine-grid simulation on the GPU and renders the current concentration or temperature field. On a slower cadence, the simulation also performs a semantic pass:

1. Build a coarse snapshot from the current grid.
2. Send that snapshot to Lean through the `kinetics` crate.
3. Receive reaction directives for the tiles where rules are active.
4. Upload those directives back to the GPU.
5. Continue the fine-grid simulation with updated kinetics parameters.

This split keeps high-frequency transport on the GPU while moving rule selection and reaction semantics into Lean.

## Scenarios

The demo currently ships with six scenarios:

- `basic`: the original Na/K/Cl transport demo inside a hollow titanium box with a temperature split.
- `acid-base`: strong acid / strong base mixing with exothermic neutralization.
- `buffers`: weak-acid buffer against NaOH, including acetate/acetic-acid equilibrium.
- `catalyst`: dissolved hexokinase driving glucose phosphorylation.
- `enzyme`: moving enzyme entities performing the same phosphorylation chemistry as localized actors.
- `leak`: buffered ionic system with membrane leak channels for K+ and Na+ transport.

## Building

### Requirements

- Rust 2024 edition toolchain.
- Vulkan 1.2-capable GPU and working Vulkan driver.
- Lean 4 and Lake.
- Linux (X11 or Wayland) or macOS. Linux is the primary target; macOS is
  supported via MoltenVK but needs the extra setup below.

#### macOS setup

Vulkan isn't native to macOS, so install the loader and MoltenVK (the
Vulkan-over-Metal translation layer) and shaderc via Homebrew:

```bash
brew install vulkan-loader molten-vk shaderc
```

`shaderc-sys` only auto-detects native libraries at `/usr/local/lib`, not
Homebrew's `/opt/homebrew/lib` on Apple Silicon, so point it there explicitly
when building:

```bash
export SHADERC_LIB_DIR=/opt/homebrew/lib
```

At runtime, binaries need `/opt/homebrew/lib` on the dynamic linker's search
path to find the Vulkan loader; `.cargo/config.toml` bakes that in as an
rpath at link time, so no environment variables are needed to run the demo,
probes, or `cargo test`. (An earlier version of this setup used
`DYLD_LIBRARY_PATH` — don't reach for that instead: macOS SIP strips
`DYLD_*` variables when exec'ing SIP-restricted binaries, including `cargo`
itself, so they aren't reliably forwarded to spawned processes like
`cargo test`'s test binaries.) MoltenVK's ICD manifest is auto-discovered by
the Vulkan loader from its default search path, so `VK_ICD_FILENAMES` isn't
needed either.

Rust (`rustup`) and Lean (`elan`) toolchains aren't macOS system tools either;
install them with `brew install rustup elan-init` if not already present.

### Build the Lean rule engine

The simulation initializes the kinetics layer by default, so the Lean executable should be built before running the demo:

```bash
cd lean
lake build
cd ..
```

By default, the Rust side looks for the binary in one of these locations:

- `LYFE_LEAN_BINARY`
- `lean/.lake/build/bin/lyfe-rules`
- `../lean/.lake/build/bin/lyfe-rules`
- `lyfe-rules` on `PATH`

If you build the Lean binary somewhere else:

```bash
export LYFE_LEAN_BINARY=/absolute/path/to/lyfe-rules
```

#### macOS note

Lean 4.16.0's bundled toolchain links with LLVM/`ld64.lld` 15.0.1, which predates
macOS's `__DATA_CONST` read-only segment enforcement; the resulting binary is
refused by dyld (`__DATA_CONST segment missing SG_READ_ONLY flag`, `SIGABRT`).
Build with the system compiler/linker instead, and point `LIBRARY_PATH` at the
toolchain's bundled `libgmp`/`libuv` (Lean's internal linker flags for these are
dropped when `LEAN_CC` is overridden):

```bash
export LEAN_CC=/usr/bin/cc
export LIBRARY_PATH="$(dirname "$(dirname "$(elan which lean)")")/lib:$LIBRARY_PATH"
cd lean && lake build && cd ..
```

### Build the Rust workspace

```bash
cargo build --release
```

## Running The Demo

No special environment variables are needed on macOS at this point (see
[macOS setup](#macos-setup)) beyond `LYFE_LEAN_BINARY`, and only if the Lean
binary isn't in one of the default lookup locations.

Show CLI help:

```bash
cargo run -p demo -- --help
```

Run the default scenario:

```bash
cargo run --release -p demo
```

Run a specific scenario:

```bash
cargo run --release -p demo -- acid-base
cargo run --release -p demo -- buffers
cargo run --release -p demo -- catalyst
cargo run --release -p demo -- enzyme
cargo run --release -p demo -- leak
```

Useful flags:

- `--detail`: render the sim as an inset with pinned inspection probes.
- `--smoke-test`: render 5 frames and exit.
- `--present-mode auto|fifo|mailbox`: choose Vulkan present mode. `auto` prefers `fifo` on X11 for capture compatibility.

Examples:

```bash
cargo run --release -p demo -- --detail leak
cargo run --release -p demo -- --smoke-test basic
cargo run --release -p demo -- --present-mode fifo enzyme
```

## Controls

General controls:

- Mouse hover: inspect the coarse cell under the cursor.
- `Space`: pause or resume the simulation.
- `+` / `-`: increase or decrease the inspection mip factor.
- Hold `T`: show the thermal view.
- `Tab`: toggle the performance overlay.
- `Shift+R`: reset the current scenario.
- `Escape`: quit.

Leak editor controls:

- Use the "Create" panel to add leak channels.
- Left click selects a leak channel or confirms placement/transform.
- `R`: rotate a leak channel by 45 degrees while placing or transforming it.
- With a leak channel selected, `T` enters transform mode.
- `Delete`: remove the selected leak channel.

## Tests And Probes

The `fluidsim` crate includes probe binaries and regression tests for the newer chemistry and transport paths:

- `acid_base_probe`: checks center-window neutralization and exothermic heating.
- `buffer_probe`: checks weak-acid / hydroxide consumption and acetate formation.
- `leak_probe`: checks K+ inward flow, Na+ outward flow, mass conservation, and bounded local charge error.

Run them with:

```bash
cargo test -p fluidsim
```

## Recent Changes

- **Fixed:** the `leak` scenario crashed on startup with `ERROR_OUT_OF_POOL_MEMORY`. Its
  descriptor pool (`fluidsim/src/gpu/pipelines.rs::init_leak_pipeline`) reserved 4
  `STORAGE_BUFFER` descriptors but allocated 2 descriptor sets × 3 bindings each (6
  needed) — undersized by construction, so it only ever surfaced when a scenario
  actually seeded leak channels at startup (only `leak` does).
- **Fixed:** `--smoke-test` could hang forever on any scenario running below 5 FPS.
  The exit condition (`frame_count >= 5`) shared its counter with the FPS-averaging
  logic, which resets that same counter to 0 every second — so a scenario slower
  than 5 FPS (like `leak`, whose tuned diffusion/time-scale values need roughly
  1.8x the substeps of other scenarios) could never accumulate 5 frames before the
  reset zeroed it out again. Smoke-test frame counting now uses its own dedicated
  counter (`demo/src/app.rs::smoke_test_frame_count`), independent of the FPS window.
- **Removed:** a leftover debug border in the visualization shader
  (`renderer/shaders/visualization.frag`) that pulsed through the color spectrum
  around the full simulation viewport on every frame, explicitly marked `// DEBUG`
  in the source. It was never meant to ship.
- **Fixed:** `cargo test -p fluidsim` couldn't run on macOS at all — its probe
  binaries build a headless Vulkan context (`fluidsim/src/gpu/setup.rs`) that,
  unlike the windowed renderer path, never requested `VK_KHR_portability_enumeration`,
  so instance creation failed with `VK_ERROR_INCOMPATIBLE_DRIVER` wherever MoltenVK
  is the only ICD available. Fixed the same way as the windowed path: request
  portability enumeration on the instance and enable `VK_KHR_portability_subset`
  on the device where supported.
- **Fixed:** even after the above, `cargo test` still couldn't find `libvulkan.dylib`
  via `DYLD_LIBRARY_PATH`, because macOS SIP strips `DYLD_*` variables when exec'ing
  SIP-restricted binaries — `cargo` itself is one, so the variable set in the shell
  never reached `cargo test`'s spawned test binaries. Replaced the env-var approach
  with an rpath baked into every binary at link time (`.cargo/config.toml`), which
  isn't affected by SIP's `DYLD_*` stripping since it's embedded in the Mach-O file.
  This also means `DYLD_LIBRARY_PATH` is no longer needed to run the demo directly,
  and `VK_ICD_FILENAMES` turned out to be unnecessary too — Homebrew's Vulkan loader
  auto-discovers MoltenVK's ICD manifest from its own default search path.
- **Fixed:** the rounded box geometry (see below) initially broke the `leak_probe`
  regression test — the probe's leak channels sit close enough to the box corners
  (within the rounding radius, at the grid resolution the test uses) that rounding
  exposed previously-walled-off fluid cells right where they attach, disturbing
  local charge neutrality just past the test's tolerance. Tuned the corner radius
  formula (`fluidsim/src/scenario/helpers.rs::add_titanium_hollow_box`) down so it
  stays clear of channel attachment points at small grid sizes while still scaling
  up visibly on the demo's full-resolution grids.
- Added macOS support: Vulkan surface creation via `VK_EXT_metal_surface`/MoltenVK,
  build/runtime setup documented in [macOS setup](#macos-setup).
- Added the Apple-inspired UI theme (Geist/Geist Mono fonts, rounded panels and
  controls, rounded simulation-box geometry) described in
  [UI and visual design](#ui-and-visual-design).

## Notes

- The Lean layer is the source of truth for active semantic rules. Adding new rule families is intended to happen in Lean first, with Rust remaining mostly rule-agnostic.
- The simulation is intentionally split into a fast fine-grid transport loop and a slower semantic reasoning loop.
- Linux is the best-tested platform and assumes working native Vulkan presentation support; macOS works through MoltenVK (see [macOS setup](#macos-setup)) but is less battle-tested.
- Capturing with OBS might not work properly on Linux due to use of low-level Vulkan presentation.
- Chemical species are currently represented with molecular formulae. Plans are in motion to represent species as full structural formulae.

## Contributing

Contributions are welcome. Feel free to open an issue to begin discussion on new features.

## License

All of the code in this repository is released under Apache 2.0 / MIT dual-license.

The embedded Geist and Geist Mono fonts (`demo/assets/fonts`) are © The Geist
Project Authors / Vercel and licensed separately under the SIL Open Font
License 1.1 (`demo/assets/fonts/LICENSE.txt`).