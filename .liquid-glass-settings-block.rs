pub struct AgentLiquidGlassSettings {
    pub enabled: bool,
    pub power_factor: f32,
    pub width: f32,
    pub height: f32,
    pub a: f32,
    pub b: f32,
    pub c: f32,
    pub d: f32,
    pub f_power: f32,
    pub noise: f32,
    pub glow_weight: f32,
    pub glow_edge0: f32,
    pub glow_edge1: f32,
    pub glow_bias: f32,
    pub chromatic_aberration: f32,
    pub aberration_samples: u32,
    pub blur_radius: f32,
    pub blur_iterations: u32,
    pub blur_downscale: f32,
    pub mouse_control: bool,
    pub position: [f32; 2],
    pub pixel_scale: f32,
    pub camera_position: [f32; 2],
    pub velocity: f32,
    pub camera_velocity: f32,
    pub current_bg: usize,
    pub glass_variant: usize,
}

const LIQUID_GLASS_BACKGROUND_COUNT: usize = 11;
const LIQUID_GLASS_VARIANT_SIZES: &[(f32, f32)] = &[
    (3.5, 3.5),
    (3.5, 3.5),
    (8.0, 0.8),
    (1.2, 1.2),
    (4.0, 4.0),
    (6.0, 1.0),
    (6.0, 4.0),
    (5.0, 5.0),
    (3.0, 6.0),
    (3.0, 7.0),
    (1.5, 1.5),
    (2.0, 2.0),
    (2.5, 2.5),
    (4.0, 4.0),
    (3.0, 1.5),
    (8.0, 3.0),
    (2.5, 8.0),
    (2.5, 8.0),
    (3.0, 5.0),
    (2.0, 2.0),
    (2.0, 2.0),
    (5.0, 0.5),
    (4.0, 3.0),
    (3.0, 2.0),
];

impl Default for AgentLiquidGlassSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            power_factor: 3.0,
            width: 3.5,
            height: 3.5,
            a: 0.7,
            b: 2.3,
            c: 5.2,
            d: 6.9,
            f_power: 1.0,
            noise: 0.0,
            glow_weight: 0.054,
            glow_edge0: 1.0,
            glow_edge1: -1.0,
            glow_bias: 0.353,
            chromatic_aberration: 0.0,
            aberration_samples: 1,
            blur_radius: 2.0,
            blur_iterations: 1,
            blur_downscale: 0.5,
            mouse_control: false,
            position: [512.0, 384.0],
            pixel_scale: 100.0,
            camera_position: [0.0, 0.0],
            velocity: 2.0,
            camera_velocity: 2.0,
            current_bg: 0,
            glass_variant: 0,
        }
    }
}

impl From<settings::AgentLiquidGlassSettingsContent> for AgentLiquidGlassSettings {
    fn from(content: settings::AgentLiquidGlassSettingsContent) -> Self {
        let defaults = Self::default();
        let glass_variant = sanitize_usize(
            content.glass_variant,
            defaults.glass_variant,
            LIQUID_GLASS_VARIANT_SIZES.len().saturating_sub(1),
        );
        let (variant_width, variant_height) = LIQUID_GLASS_VARIANT_SIZES[glass_variant];

        Self {
            enabled: content.enabled.unwrap_or(defaults.enabled),
            power_factor: sanitize_f32(content.power_factor, defaults.power_factor, 1.001, 6.0),
            width: sanitize_f32(content.width, variant_width, 0.01, 10.0),
            height: sanitize_f32(content.height, variant_height, 0.01, 10.0),
            a: sanitize_f32(content.a, defaults.a, 0.0, 5.0),
            b: sanitize_f32(content.b, defaults.b, 0.0, 6.0),
            c: sanitize_f32(content.c, defaults.c, 0.0, 6.0),
            d: sanitize_f32(content.d, defaults.d, 0.0, 10.0),
            f_power: sanitize_f32(content.f_power, defaults.f_power, -1.5, 6.0),
            noise: sanitize_f32(content.noise, defaults.noise, 0.0, 0.3),
            glow_weight: sanitize_f32(content.glow_weight, defaults.glow_weight, -1.0, 1.0),
            glow_edge0: sanitize_f32(content.glow_edge0, defaults.glow_edge0, -1.0, 1.0),
            glow_edge1: sanitize_f32(content.glow_edge1, defaults.glow_edge1, -1.0, 1.0),
            glow_bias: sanitize_f32(content.glow_bias, defaults.glow_bias, -1.0, 1.0),
            chromatic_aberration: sanitize_f32(
                content.chromatic_aberration,
                defaults.chromatic_aberration,
                0.0,
                0.02,
            ),
            aberration_samples: sanitize_u32(
                content.aberration_samples,
                defaults.aberration_samples,
                1,
                8,
            ),
            blur_radius: sanitize_f32(content.blur_radius, defaults.blur_radius, 0.0, 10.0),
            blur_iterations: sanitize_u32(content.blur_iterations, defaults.blur_iterations, 0, 10),
            blur_downscale: sanitize_f32(content.blur_downscale, defaults.blur_downscale, 0.1, 1.0),
            mouse_control: content.mouse_control.unwrap_or(defaults.mouse_control),
            position: sanitize_pair(
                content.position,
                defaults.position,
                [0.0, 0.0],
                [1024.0, 768.0],
            ),
            pixel_scale: sanitize_f32(content.pixel_scale, defaults.pixel_scale, 1.0, 512.0),
            camera_position: sanitize_pair(
                content.camera_position,
                defaults.camera_position,
                [-1024.0, -1024.0],
                [1024.0, 1024.0],
            ),
            velocity: sanitize_f32(content.velocity, defaults.velocity, 0.0, 20.0),
            camera_velocity: sanitize_f32(
                content.camera_velocity,
                defaults.camera_velocity,
                0.0,
                20.0,
            ),
            current_bg: sanitize_usize(
                content.current_bg,
                defaults.current_bg,
                LIQUID_GLASS_BACKGROUND_COUNT.saturating_sub(1),
            ),
            glass_variant,
        }
    }
}

/// A threshold for when to automatically compact a thread.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AutoCompactThreshold {
    /// Compact once the context window is at least this full, as a fraction in
    /// the range `(0.0, 1.0]`.
    Percentage(f64),
    /// Compact once at least this many tokens have been used.
    TokensUsed(u64),
    /// Compact once fewer than this many tokens remain in the context window.
    TokensRemaining(u64),
}

impl AutoCompactThreshold {
    /// The threshold used when none is configured, or when the configured value
    /// is invalid (90% of the context window).
    pub const DEFAULT: Self = Self::Percentage(0.9);
}

impl fmt::Display for AutoCompactThreshold {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Percentage(percent) => write!(formatter, "{}%", percent * 100.0),
            Self::TokensUsed(tokens) => write!(formatter, "{tokens}"),
            Self::TokensRemaining(tokens) => write!(formatter, "-{tokens}"),
        }
    }
}

fn sanitize_f32(value: Option<f32>, fallback: f32, min: f32, max: f32) -> f32 {
    value
        .filter(|value| value.is_finite())
        .unwrap_or(fallback)
        .clamp(min, max)
}

fn sanitize_u32(value: Option<u32>, fallback: u32, min: u32, max: u32) -> u32 {
    value.unwrap_or(fallback).clamp(min, max)
}

fn sanitize_usize(value: Option<usize>, fallback: usize, max: usize) -> usize {
    value.unwrap_or(fallback).min(max)
}

fn sanitize_pair(
    value: Option<[f32; 2]>,
    fallback: [f32; 2],
    min: [f32; 2],
    max: [f32; 2],
) -> [f32; 2] {
    let value = value.unwrap_or(fallback);

    [
        sanitize_f32(Some(value[0]), fallback[0], min[0], max[0]),
        sanitize_f32(Some(value[1]), fallback[1], min[1], max[1]),
    ]
}

