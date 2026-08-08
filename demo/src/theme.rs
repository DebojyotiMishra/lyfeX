//! Apple-inspired visual theme: Geist / Geist Mono fonts, translucent "glass" panels,
//! rounded controls, and a systemBlue accent.

use std::sync::Arc;

use egui::{
    Color32, Context, Frame, FontData, FontDefinitions, FontFamily, FontId, Margin, Rounding,
    Shadow, Stroke, TextStyle, Theme, ThemePreference, Vec2,
};

/// Corner radius for floating panels (control bar, inspector, tooltips).
pub(crate) const RADIUS_PANEL: f32 = 10.0;
/// Corner radius for buttons, sliders, and other small controls.
pub(crate) const RADIUS_CONTROL: f32 = 6.0;

/// macOS systemBlue (dark mode variant).
pub(crate) fn accent() -> Color32 {
    Color32::from_rgb(10, 132, 255)
}

pub(crate) fn panel_bg() -> Color32 {
    Color32::from_rgba_unmultiplied(30, 30, 32, 218)
}

pub(crate) fn panel_border() -> Color32 {
    Color32::from_rgba_unmultiplied(255, 255, 255, 28)
}

fn panel_shadow() -> Shadow {
    Shadow {
        offset: Vec2::new(0.0, 10.0),
        blur: 28.0,
        spread: 0.0,
        color: Color32::from_black_alpha(90),
    }
}

/// A translucent, rounded "glass" panel matching the rest of the HUD.
pub(crate) fn panel_frame() -> Frame {
    Frame::none()
        .fill(panel_bg())
        .stroke(Stroke::new(1.0, panel_border()))
        .rounding(Rounding::same(RADIUS_PANEL))
        .shadow(panel_shadow())
        .inner_margin(Margin::same(12.0))
}

pub(crate) fn heading_font(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name("geist-semibold".into()))
}

pub(crate) fn medium_font(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name("geist-medium".into()))
}

/// Install Geist/Geist Mono fonts and an Apple-inspired dark visual style on `ctx`.
pub(crate) fn install(ctx: &Context) {
    install_fonts(ctx);
    install_style(ctx);
    // `panel_frame` and the accent colors are hardcoded for a dark background, so
    // pin the preference rather than letting egui follow the system appearance.
    ctx.set_theme(ThemePreference::Dark);
}

fn install_fonts(ctx: &Context) {
    let mut fonts = FontDefinitions::default();

    let embed = [
        ("Geist-Regular", include_bytes!("../assets/fonts/Geist-Regular.ttf").as_slice()),
        ("Geist-Medium", include_bytes!("../assets/fonts/Geist-Medium.ttf").as_slice()),
        ("Geist-SemiBold", include_bytes!("../assets/fonts/Geist-SemiBold.ttf").as_slice()),
        (
            "GeistMono-Regular",
            include_bytes!("../assets/fonts/GeistMono-Regular.ttf").as_slice(),
        ),
        (
            "GeistMono-Medium",
            include_bytes!("../assets/fonts/GeistMono-Medium.ttf").as_slice(),
        ),
    ];
    for (name, bytes) in embed {
        fonts
            .font_data
            .insert(name.to_owned(), Arc::new(FontData::from_static(bytes)));
    }

    fonts
        .families
        .entry(FontFamily::Proportional)
        .or_default()
        .insert(0, "Geist-Regular".to_owned());
    fonts
        .families
        .entry(FontFamily::Monospace)
        .or_default()
        .insert(0, "GeistMono-Regular".to_owned());
    fonts.families.insert(
        FontFamily::Name("geist-medium".into()),
        vec!["Geist-Medium".to_owned()],
    );
    fonts.families.insert(
        FontFamily::Name("geist-semibold".into()),
        vec!["Geist-SemiBold".to_owned()],
    );
    fonts.families.insert(
        FontFamily::Name("geist-mono-medium".into()),
        vec!["GeistMono-Medium".to_owned()],
    );

    ctx.set_fonts(fonts);
}

fn install_style(ctx: &Context) {
    let mut style = (*ctx.style()).clone();

    style
        .text_styles
        .insert(TextStyle::Heading, heading_font(20.0));
    style
        .text_styles
        .insert(TextStyle::Button, medium_font(13.5));
    style.text_styles.insert(
        TextStyle::Body,
        FontId::new(13.5, FontFamily::Proportional),
    );
    style.text_styles.insert(
        TextStyle::Small,
        FontId::new(11.5, FontFamily::Proportional),
    );
    style
        .text_styles
        .insert(TextStyle::Monospace, FontId::new(12.5, FontFamily::Monospace));

    style.spacing.item_spacing = Vec2::new(8.0, 7.0);
    style.spacing.button_padding = Vec2::new(14.0, 7.0);
    style.spacing.window_margin = Margin::same(14.0);
    style.spacing.menu_margin = Margin::same(8.0);

    let visuals = &mut style.visuals;
    visuals.dark_mode = true;
    visuals.override_text_color = Some(Color32::from_rgba_unmultiplied(245, 245, 250, 235));
    visuals.hyperlink_color = accent();
    visuals.faint_bg_color = Color32::from_rgba_unmultiplied(255, 255, 255, 10);
    visuals.extreme_bg_color = Color32::from_rgba_unmultiplied(18, 18, 20, 235);
    visuals.code_bg_color = Color32::from_rgba_unmultiplied(255, 255, 255, 18);
    visuals.window_fill = panel_bg();
    visuals.window_stroke = Stroke::new(1.0, panel_border());
    visuals.window_rounding = Rounding::same(RADIUS_PANEL);
    visuals.window_shadow = panel_shadow();
    visuals.menu_rounding = Rounding::same(RADIUS_CONTROL);
    visuals.panel_fill = panel_bg();
    visuals.popup_shadow = panel_shadow();

    visuals.selection.bg_fill = accent().linear_multiply(0.55);
    visuals.selection.stroke = Stroke::new(1.0, accent());

    let w = &mut visuals.widgets;
    w.noninteractive.bg_fill = Color32::TRANSPARENT;
    w.noninteractive.weak_bg_fill = Color32::TRANSPARENT;
    w.noninteractive.bg_stroke = Stroke::new(1.0, panel_border());
    w.noninteractive.fg_stroke =
        Stroke::new(1.0, Color32::from_rgba_unmultiplied(235, 235, 245, 200));
    w.noninteractive.rounding = Rounding::same(RADIUS_CONTROL);

    w.inactive.bg_fill = Color32::from_rgba_unmultiplied(255, 255, 255, 20);
    w.inactive.weak_bg_fill = Color32::from_rgba_unmultiplied(255, 255, 255, 16);
    w.inactive.bg_stroke = Stroke::NONE;
    w.inactive.fg_stroke =
        Stroke::new(1.0, Color32::from_rgba_unmultiplied(245, 245, 250, 220));
    w.inactive.rounding = Rounding::same(RADIUS_CONTROL);

    w.hovered.bg_fill = Color32::from_rgba_unmultiplied(255, 255, 255, 34);
    w.hovered.weak_bg_fill = Color32::from_rgba_unmultiplied(255, 255, 255, 30);
    w.hovered.bg_stroke = Stroke::new(1.0, Color32::from_rgba_unmultiplied(255, 255, 255, 40));
    w.hovered.fg_stroke = Stroke::new(1.0, Color32::WHITE);
    w.hovered.rounding = Rounding::same(RADIUS_CONTROL);
    w.hovered.expansion = 0.5;

    w.active.bg_fill = accent();
    w.active.weak_bg_fill = accent();
    w.active.bg_stroke = Stroke::new(1.0, accent());
    w.active.fg_stroke = Stroke::new(1.0, Color32::WHITE);
    w.active.rounding = Rounding::same(RADIUS_CONTROL);

    w.open.bg_fill = Color32::from_rgba_unmultiplied(255, 255, 255, 24);
    w.open.weak_bg_fill = Color32::from_rgba_unmultiplied(255, 255, 255, 20);
    w.open.bg_stroke = Stroke::new(1.0, panel_border());
    w.open.fg_stroke = Stroke::new(1.0, Color32::WHITE);
    w.open.rounding = Rounding::same(RADIUS_CONTROL);

    // `set_style` would only write whichever theme is active at install time, leaving
    // the other holding egui's defaults. Install into both so a theme switch can't
    // strip the fonts, rounding, and accent while `panel_frame` keeps painting dark.
    let style = Arc::new(style);
    ctx.set_style_of(Theme::Dark, style.clone());
    ctx.set_style_of(Theme::Light, style);
}
