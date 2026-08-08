use super::ScenarioBuilder;

#[derive(Debug, Clone, Copy)]
pub(super) struct CentralBoxBounds {
    pub wall_thickness: u32,
    pub outer_x0: u32,
    pub outer_y0: u32,
    pub outer_x1: u32,
    pub outer_y1: u32,
    pub inner_x0: u32,
    pub inner_y0: u32,
    pub inner_x1: u32,
    pub inner_y1: u32,
}

pub(super) fn central_box_bounds(width: u32, height: u32) -> CentralBoxBounds {
    let wall_thickness = 4u32;
    let inner_size = (width.min(height) / 2).max(64);
    let outer_size = inner_size + 2 * wall_thickness;

    let center_x = width / 2;
    let center_y = height / 2;

    let outer_x0 = center_x - outer_size / 2;
    let outer_y0 = center_y - outer_size / 2;
    let outer_x1 = outer_x0 + outer_size;
    let outer_y1 = outer_y0 + outer_size;

    let inner_x0 = outer_x0 + wall_thickness;
    let inner_y0 = outer_y0 + wall_thickness;
    let inner_x1 = outer_x1 - wall_thickness;
    let inner_y1 = outer_y1 - wall_thickness;

    CentralBoxBounds {
        wall_thickness,
        outer_x0,
        outer_y0,
        outer_x1,
        outer_y1,
        inner_x0,
        inner_y0,
        inner_x1,
        inner_y1,
    }
}

/// Outer corner radius of the wall built by [`add_titanium_hollow_box`].
pub(super) fn corner_radius(bounds: CentralBoxBounds) -> u32 {
    let outer_size = bounds.outer_x1 - bounds.outer_x0;
    (outer_size / 16).max(bounds.wall_thickness)
}

/// Smallest inset from the axis-aligned inner rect that still clears the rounded
/// corners of the cavity.
///
/// The cavity's corner arc has radius `r = corner_radius - wall_thickness`, centred
/// `r` cells in along each axis. A point `m` cells in from the rect corner along both
/// axes lies within the arc - i.e. inside the fluid - only when
/// `(r - m) * sqrt(2) <= r`, which reduces to `m >= r * (1 - 1/sqrt(2))`.
///
/// That bound is the exact tangent point, which is not good enough on its own: the wall
/// is rasterized to whole cells and callers truncate to integer coordinates, so a point
/// sitting exactly on the arc still resolves to a solid cell. One cell of slack is added
/// to land strictly inside the fluid.
///
/// Consumers that place entities against `inner_*` must respect this, or they will
/// spawn inside the wall once the radius grows with the grid.
pub(super) fn rounded_corner_inset(bounds: CentralBoxBounds) -> f32 {
    let arc_radius = corner_radius(bounds).saturating_sub(bounds.wall_thickness) as f32;
    arc_radius * (1.0 - std::f32::consts::FRAC_1_SQRT_2) + 1.0
}

pub(super) fn add_titanium_hollow_box(
    builder: ScenarioBuilder,
    bounds: CentralBoxBounds,
) -> ScenarioBuilder {
    let (builder, titanium) = builder.register_material("titanium", [0.6, 0.6, 0.65, 1.0]);
    let radius = corner_radius(bounds);
    builder.fill_rounded_hollow_rect(
        bounds.outer_x0,
        bounds.outer_y0,
        bounds.outer_x1,
        bounds.outer_y1,
        bounds.wall_thickness,
        radius,
        titanium,
    )
}

pub(super) fn normalized_position(bounds: CentralBoxBounds, x: u32, y: u32) -> (f32, f32) {
    let inner_width = (bounds.inner_x1 - bounds.inner_x0) as f32;
    let inner_height = (bounds.inner_y1 - bounds.inner_y0) as f32;
    let nx = (x - bounds.inner_x0) as f32 / inner_width;
    let ny = (y - bounds.inner_y0) as f32 / inner_height;
    (nx, ny)
}

pub(super) fn lcg_next(seed: &mut u32) -> u32 {
    *seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
    *seed
}

pub(super) fn lcg_unit(seed: &mut u32) -> f32 {
    (lcg_next(seed) >> 8) as f32 / ((u32::MAX >> 8) as f32)
}
