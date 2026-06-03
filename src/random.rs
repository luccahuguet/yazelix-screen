use std::time::{SystemTime, UNIX_EPOCH};

pub const GAME_OF_LIFE_RANDOM_STYLES: &[&str] = &[
    "game_of_life_gliders",
    "game_of_life_oscillators",
    "game_of_life_bloom",
];
pub const BOIDS_RANDOM_STYLES: &[&str] = &["boids_predator", "boids_schools"];
pub const MANDELBROT_STYLE: &str = "mandelbrot";
pub const KITTY_FRAME_SEQUENCE_STYLE: &str = "magician";

pub fn random_animation_slot_count() -> usize {
    random_animation_slot_count_with_magician(false)
}

pub fn random_animation_slot_count_with_magician(include_magician: bool) -> usize {
    random_animation_family_count(include_magician)
        * lcm(
            GAME_OF_LIFE_RANDOM_STYLES.len(),
            BOIDS_RANDOM_STYLES.len().max(1),
        )
}

pub fn random_animation_styles() -> Vec<&'static str> {
    random_animation_styles_with_magician(false)
}

pub fn random_animation_styles_with_magician(include_magician: bool) -> Vec<&'static str> {
    let mut styles = Vec::new();
    styles.extend_from_slice(GAME_OF_LIFE_RANDOM_STYLES);
    styles.extend_from_slice(BOIDS_RANDOM_STYLES);
    styles.push(MANDELBROT_STYLE);
    if include_magician {
        styles.push(KITTY_FRAME_SEQUENCE_STYLE);
    }
    styles
}

pub fn resolve_random_animation_style(random_index: Option<usize>) -> &'static str {
    resolve_random_animation_style_with_magician(random_index, false)
}

pub fn resolve_random_animation_style_with_magician(
    random_index: Option<usize>,
    include_magician: bool,
) -> &'static str {
    let subpool_width = lcm(
        GAME_OF_LIFE_RANDOM_STYLES.len(),
        BOIDS_RANDOM_STYLES.len().max(1),
    );
    let family_count = random_animation_family_count(include_magician);
    let slot_count = family_count * subpool_width;
    let selected = random_index.unwrap_or_else(|| system_random_index(slot_count)) % slot_count;
    let family = selected % family_count;
    let family_index = (selected / family_count) % subpool_width;

    match family {
        0 => GAME_OF_LIFE_RANDOM_STYLES[family_index % GAME_OF_LIFE_RANDOM_STYLES.len()],
        1 => BOIDS_RANDOM_STYLES[family_index % BOIDS_RANDOM_STYLES.len()],
        2 => MANDELBROT_STYLE,
        _ => KITTY_FRAME_SEQUENCE_STYLE,
    }
}

fn random_animation_family_count(include_magician: bool) -> usize {
    3 + usize::from(include_magician)
}

fn system_random_index(max_len: usize) -> usize {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos() as usize;
    nanos % max_len.max(1)
}

fn lcm(left: usize, right: usize) -> usize {
    if left == 0 || right == 0 {
        return 0;
    }
    left / gcd(left, right) * right
}

fn gcd(mut left: usize, mut right: usize) -> usize {
    while right != 0 {
        let remainder = left % right;
        left = right;
        right = remainder;
    }
    left
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test lane: default

    // Defends: default random animation avoids image-backed magician unless the caller has verified the frame assets are available.
    #[test]
    fn random_animation_style_rotates_across_default_text_families() {
        let mut game_of_life_count = 0;
        let mut boids_count = 0;
        let mut mandelbrot_count = 0;

        for index in 0..random_animation_slot_count() {
            match resolve_random_animation_style(Some(index)) {
                style if GAME_OF_LIFE_RANDOM_STYLES.contains(&style) => game_of_life_count += 1,
                style if BOIDS_RANDOM_STYLES.contains(&style) => boids_count += 1,
                MANDELBROT_STYLE => mandelbrot_count += 1,
                other => panic!("unexpected random style {other}"),
            }
        }

        assert_eq!(game_of_life_count, 6);
        assert_eq!(boids_count, 6);
        assert_eq!(mandelbrot_count, 6);
    }

    // Defends: callers that prove magician assets are available can opt random back into the fourth image-backed family.
    #[test]
    fn random_animation_style_can_include_magician_family() {
        let mut game_of_life_count = 0;
        let mut boids_count = 0;
        let mut mandelbrot_count = 0;
        let mut kitty_frame_count = 0;

        for index in 0..random_animation_slot_count_with_magician(true) {
            match resolve_random_animation_style_with_magician(Some(index), true) {
                style if GAME_OF_LIFE_RANDOM_STYLES.contains(&style) => game_of_life_count += 1,
                style if BOIDS_RANDOM_STYLES.contains(&style) => boids_count += 1,
                MANDELBROT_STYLE => mandelbrot_count += 1,
                KITTY_FRAME_SEQUENCE_STYLE => kitty_frame_count += 1,
                other => panic!("unexpected random style {other}"),
            }
        }

        assert_eq!(game_of_life_count, 6);
        assert_eq!(boids_count, 6);
        assert_eq!(mandelbrot_count, 6);
        assert_eq!(kitty_frame_count, 6);
    }
}
