use crossterm::event::{self, Event};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;
use yazelix_screen::{
    BoidsAnimation, BoidsVariant, GAME_OF_LIFE_RANDOM_STYLES, GameOfLifeAnimation,
    GameOfLifeCellStyle, KITTY_FRAME_SEQUENCE_STYLE, KittyFrameSequence,
    MAGICIAN_EDGE_INSET_COLUMNS, MAGICIAN_EDGE_INSET_ROWS, MAGICIAN_FRAME_DELAY, MANDELBROT_STYLE,
    MandelbrotAnimation, RawModeGuard, ScreenAnimationContext, ScreenFrameProducer,
    default_magician_frame_dir, ensure_default_magician_frame_dir, enter_screen_mode,
    game_of_life_spec, is_game_of_life_style, leave_screen_mode,
    magician_default_generation_available, magician_frame_paths, mandelbrot_frame_delay,
    play_kitty_png_frame_sequence, render_screen_frame, require_magician_frame_assets,
    resolve_random_animation_style_with_magician, terminal_height, terminal_width,
};

const KITTY_IMAGE_ID: u32 = 7_930_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StandaloneStyle {
    Boids(BoidsVariant),
    GameOfLife(&'static str),
    Mandelbrot,
    KittyFrames,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Args {
    style: String,
    cell_style: GameOfLifeCellStyle,
    kitty_frame_dir: Option<PathBuf>,
    kitty_frame_count: Option<usize>,
    help: bool,
}

struct ScreenModeGuard;

impl ScreenModeGuard {
    fn new() -> std::io::Result<Self> {
        enter_screen_mode()?;
        Ok(Self)
    }
}

impl Drop for ScreenModeGuard {
    fn drop(&mut self) {
        let _ = leave_screen_mode();
    }
}

fn main() {
    match run(std::env::args().skip(1)) {
        Ok(()) => {}
        Err(message) => {
            eprintln!("{message}");
            std::process::exit(1);
        }
    }
}

fn run(args: impl IntoIterator<Item = String>) -> Result<(), String> {
    let parsed = parse_args(args)?;
    if parsed.help {
        print_help();
        return Ok(());
    }

    run_screen(parsed)
}

fn parse_args(args: impl IntoIterator<Item = String>) -> Result<Args, String> {
    let mut help = false;
    let mut style = None;
    let mut cell_style = GameOfLifeCellStyle::FullBlock;
    let mut kitty_frame_dir = None;
    let mut kitty_frame_count = None;
    let mut iter = args.into_iter();

    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "-h" | "--help" | "help" => help = true,
            "--cell-style" => {
                let Some(raw) = iter.next() else {
                    return Err("Missing value after --cell-style".to_string());
                };
                cell_style = GameOfLifeCellStyle::parse(&raw).map_err(|error| {
                    format!(
                        "Invalid --cell-style value `{}`. Expected full_block or dotted",
                        error.normalized()
                    )
                })?;
            }
            "--kitty-frame-dir" => {
                let Some(raw) = iter.next() else {
                    return Err("Missing value after --kitty-frame-dir".to_string());
                };
                kitty_frame_dir = Some(PathBuf::from(raw));
            }
            "--kitty-frame-count" => {
                let Some(raw) = iter.next() else {
                    return Err("Missing value after --kitty-frame-count".to_string());
                };
                let count = raw.parse::<usize>().map_err(|_| {
                    format!(
                        "Invalid --kitty-frame-count value `{raw}`. Expected a positive integer"
                    )
                })?;
                if count == 0 {
                    return Err(
                        "Invalid --kitty-frame-count value `0`. Expected a positive integer"
                            .to_string(),
                    );
                }
                kitty_frame_count = Some(count);
            }
            other if style.is_none() => style = Some(other.to_string()),
            other => {
                return Err(format!("Unexpected argument `{other}`. Try `yzs --help`"));
            }
        }
    }

    Ok(Args {
        style: style.unwrap_or_else(|| "random".to_string()),
        cell_style,
        kitty_frame_dir,
        kitty_frame_count,
        help,
    })
}

fn print_help() {
    println!("Show standalone Yazelix terminal screen animations");
    println!();
    println!("Usage:");
    println!("  yzs [STYLE] [--cell-style full_block|dotted]");
    println!("      [--kitty-frame-dir DIR] [--kitty-frame-count N]");
    println!();
    println!("Styles:");
    println!("  boids");
    println!("  boids_predator");
    println!("  boids_schools");
    println!("  mandelbrot");
    println!("  magician");
    println!("  game_of_life_gliders");
    println!("  game_of_life_oscillators");
    println!("  game_of_life_bloom");
    println!("  random");
    println!();
    println!("Notes:");
    println!("  Runs outside Zellij and outside a Yazelix session");
    println!("  random skips magician unless PNG frames or host ImageMagick are available");
    println!(
        "  magician can generate cached frames with host ImageMagick or use --kitty-frame-dir"
    );
    println!("  Press any key to exit");
}

fn run_screen(args: Args) -> Result<(), String> {
    let resolved_style = resolve_style(&args.style, None)?;
    let kitty_sequence = match resolved_style {
        StandaloneStyle::KittyFrames => Some(build_kitty_frame_sequence(&args)?),
        _ => None,
    };

    let _raw = RawModeGuard::new().map_err(|error| format!("Could not enter raw mode: {error}"))?;
    let _screen = ScreenModeGuard::new()
        .map_err(|error| format!("Could not enter alternate screen mode: {error}"))?;

    if let Some(sequence) = kitty_sequence {
        return play_kitty_png_frame_sequence(&sequence, None, terminal_width, terminal_height)
            .map_err(|error| format!("Could not render Kitty frame sequence: {error}"));
    }

    let mut width = terminal_width();
    let mut height = terminal_height();
    let mut animation = build_animation(resolved_style, width, height, args.cell_style);
    let frame_delay = frame_delay(resolved_style);

    loop {
        render_screen_frame(&animation.render_frame())
            .map_err(|error| format!("Could not render screen frame: {error}"))?;
        if poll_for_keypress(frame_delay)? {
            break;
        }

        let current_width = terminal_width();
        let current_height = terminal_height();
        if current_width != width || current_height != height {
            width = current_width;
            height = current_height;
            animation.resize(context_for_style(resolved_style, width, height));
            continue;
        }

        animation.advance_frame();
    }

    Ok(())
}

fn resolve_style(raw: &str, random_index: Option<usize>) -> Result<StandaloneStyle, String> {
    resolve_style_with_magician_availability(
        raw,
        random_index,
        magician_default_generation_available(),
    )
}

fn resolve_style_with_magician_availability(
    raw: &str,
    random_index: Option<usize>,
    include_magician: bool,
) -> Result<StandaloneStyle, String> {
    let normalized = raw.trim().to_ascii_lowercase();
    if normalized == "random" {
        return resolve_style_with_magician_availability(
            resolve_random_animation_style_with_magician(random_index, include_magician),
            None,
            include_magician,
        );
    }

    if let Some(variant) = BoidsVariant::from_style_name(&normalized) {
        return Ok(StandaloneStyle::Boids(variant));
    }
    if normalized == MANDELBROT_STYLE {
        return Ok(StandaloneStyle::Mandelbrot);
    }
    if normalized == KITTY_FRAME_SEQUENCE_STYLE {
        return Ok(StandaloneStyle::KittyFrames);
    }
    if is_game_of_life_style(&normalized) {
        let style = GAME_OF_LIFE_RANDOM_STYLES
            .iter()
            .find(|candidate| **candidate == normalized)
            .copied()
            .expect("is_game_of_life_style matched known standalone style");
        return Ok(StandaloneStyle::GameOfLife(style));
    }

    Err(format!(
        "Unsupported standalone yzs style `{normalized}`. Try `yzs --help`"
    ))
}

fn build_kitty_frame_sequence(args: &Args) -> Result<KittyFrameSequence, String> {
    let explicit_frame_dir = args.kitty_frame_dir.clone();
    let frame_dir = explicit_frame_dir.clone().map(Ok).unwrap_or_else(|| {
        default_magician_frame_dir()
            .map(Ok)
            .unwrap_or_else(|| ensure_default_magician_frame_dir())
    })
    .map_err(|error| {
        format!(
            "Style `magician` requires PNG frames or host ImageMagick: {error}. Install ImageMagick `magick`, or pass --kitty-frame-dir /path/to/frames"
        )
    })?;
    let frame_paths = if explicit_frame_dir.is_none() && args.kitty_frame_count.is_none() {
        require_magician_frame_assets(&frame_dir).map_err(|error| error.to_string())?;
        magician_frame_paths(&frame_dir)
    } else {
        kitty_frame_paths(&frame_dir, args.kitty_frame_count)?
    };

    Ok(KittyFrameSequence {
        frame_paths,
        frame_delay: MAGICIAN_FRAME_DELAY,
        image_id: KITTY_IMAGE_ID,
        attribution: None,
        edge_inset_columns: MAGICIAN_EDGE_INSET_COLUMNS,
        edge_inset_rows: MAGICIAN_EDGE_INSET_ROWS,
    })
}

fn kitty_frame_paths(frame_dir: &Path, frame_count: Option<usize>) -> Result<Vec<PathBuf>, String> {
    if let Some(count) = frame_count {
        let paths = (0..count)
            .map(|index| explicit_count_frame_path(frame_dir, index))
            .collect::<Vec<_>>();
        ensure_kitty_frame_paths_exist(&paths)?;
        return Ok(paths);
    }

    let mut paths = fs::read_dir(frame_dir)
        .map_err(|error| {
            format!(
                "Could not read Kitty frame directory `{}`: {error}",
                frame_dir.display()
            )
        })?
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| {
            format!(
                "Could not inspect Kitty frame directory `{}`: {error}",
                frame_dir.display()
            )
        })?;

    paths.retain(|path| {
        path.extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| extension.eq_ignore_ascii_case("png"))
    });
    paths.sort_by_key(|path| frame_path_sort_key(path));
    ensure_kitty_frame_paths_exist(&paths)?;
    Ok(paths)
}

fn explicit_count_frame_path(frame_dir: &Path, index: usize) -> PathBuf {
    let padded = frame_dir.join(format!("frame_{index:03}.png"));
    if padded.is_file() {
        return padded;
    }
    frame_dir.join(format!("frame_{index}.png"))
}

fn frame_path_sort_key(path: &Path) -> (usize, usize, String) {
    let stem = path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("");
    let number = stem
        .rsplit_once('_')
        .and_then(|(_, raw)| raw.parse::<usize>().ok());

    match number {
        Some(number) => (0, number, stem.to_string()),
        None => (1, usize::MAX, stem.to_string()),
    }
}

fn ensure_kitty_frame_paths_exist(paths: &[PathBuf]) -> Result<(), String> {
    if paths.is_empty() {
        return Err("Kitty frame directory does not contain any PNG frames".to_string());
    }

    if let Some(missing) = paths.iter().find(|path| !path.is_file()) {
        return Err(format!("Missing Kitty frame asset: {}", missing.display()));
    }

    Ok(())
}

fn build_animation(
    style: StandaloneStyle,
    width: usize,
    height: usize,
    cell_style: GameOfLifeCellStyle,
) -> Box<dyn ScreenFrameProducer> {
    let context = context_for_style(style, width, height);
    match style {
        StandaloneStyle::Boids(variant) => {
            Box::new(BoidsAnimation::with_variant(context, cell_style, variant))
        }
        StandaloneStyle::GameOfLife(style_name) => {
            Box::new(GameOfLifeAnimation::new(style_name, context, cell_style))
        }
        StandaloneStyle::Mandelbrot => Box::new(MandelbrotAnimation::new(context)),
        StandaloneStyle::KittyFrames => {
            unreachable!("Kitty frame sequences do not use the text animation engine")
        }
    }
}

fn context_for_style(
    style: StandaloneStyle,
    width: usize,
    height: usize,
) -> ScreenAnimationContext {
    match style {
        StandaloneStyle::GameOfLife(_) => game_of_life_context(width, height),
        StandaloneStyle::Boids(_) | StandaloneStyle::Mandelbrot | StandaloneStyle::KittyFrames => {
            full_screen_context(width, height)
        }
    }
}

fn game_of_life_context(width: usize, height: usize) -> ScreenAnimationContext {
    let size_class = size_class(width);
    let spec = game_of_life_spec(size_class);
    ScreenAnimationContext {
        resolved_width: width,
        resolved_height: height,
        inner_width: fit_inner_width(width, spec.minimum_inner_width),
        size_class,
    }
}

fn full_screen_context(width: usize, height: usize) -> ScreenAnimationContext {
    ScreenAnimationContext {
        resolved_width: width,
        resolved_height: height,
        inner_width: width,
        size_class: size_class(width),
    }
}

fn size_class(width: usize) -> &'static str {
    if width < 44 {
        "narrow"
    } else if width < 72 {
        "medium"
    } else if width < 120 {
        "wide"
    } else {
        "hero"
    }
}

fn fit_inner_width(resolved_width: usize, minimum_width: usize) -> usize {
    resolved_width.saturating_sub(6).max(minimum_width)
}

fn frame_delay(style: StandaloneStyle) -> Duration {
    match style {
        StandaloneStyle::Boids(_) => Duration::from_millis(70),
        StandaloneStyle::Mandelbrot => mandelbrot_frame_delay(),
        StandaloneStyle::GameOfLife(_) => Duration::from_millis(160),
        StandaloneStyle::KittyFrames => MAGICIAN_FRAME_DELAY,
    }
}

fn poll_for_keypress(timeout: Duration) -> Result<bool, String> {
    if !event::poll(timeout).map_err(|error| format!("Could not poll for keypress: {error}"))? {
        return Ok(false);
    }

    loop {
        match event::read().map_err(|error| format!("Could not read terminal event: {error}"))? {
            Event::Key(_) => return Ok(true),
            _ => {
                if !event::poll(Duration::from_millis(0))
                    .map_err(|error| format!("Could not poll terminal event queue: {error}"))?
                {
                    return Ok(false);
                }
            }
        }
    }
}

// Test lane: default
#[cfg(test)]
mod tests {
    use super::*;

    // Defends: the standalone binary owns a small no-session style surface instead of borrowing yzx screen's config/session-only styles.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn resolve_style_accepts_only_standalone_animation_styles() {
        assert_eq!(
            resolve_style("boids", None).unwrap(),
            StandaloneStyle::Boids(BoidsVariant::Predator)
        );
        assert_eq!(
            resolve_style("game_of_life_bloom", None).unwrap(),
            StandaloneStyle::GameOfLife("game_of_life_bloom")
        );
        assert_eq!(
            resolve_style("mandelbrot", None).unwrap(),
            StandaloneStyle::Mandelbrot
        );
        assert_eq!(
            resolve_style("magician", None).unwrap(),
            StandaloneStyle::KittyFrames
        );
        assert!(resolve_style("static", None).is_err());
        assert!(resolve_style("logo", None).is_err());
    }

    // Defends: random standalone playback skips image-backed magician unless default frame assets can be generated or resolved.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn random_style_skips_magician_when_unavailable() {
        let mut saw_kitty_frames = false;

        for index in 0..yazelix_screen::random_animation_slot_count() * 2 {
            let resolved =
                resolve_style_with_magician_availability("random", Some(index), false).unwrap();
            saw_kitty_frames |= resolved == StandaloneStyle::KittyFrames;
            assert!(matches!(
                resolved,
                StandaloneStyle::Boids(_)
                    | StandaloneStyle::GameOfLife(_)
                    | StandaloneStyle::Mandelbrot
            ));
        }

        assert!(!saw_kitty_frames);
    }

    // Defends: hosts that can resolve magician assets can opt random back into the image-backed family.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn random_style_can_include_magician_when_available() {
        let mut saw_kitty_frames = false;

        for index in 0..yazelix_screen::random_animation_slot_count_with_magician(true) * 2 {
            let resolved =
                resolve_style_with_magician_availability("random", Some(index), true).unwrap();
            saw_kitty_frames |= resolved == StandaloneStyle::KittyFrames;
            assert!(matches!(
                resolved,
                StandaloneStyle::Boids(_)
                    | StandaloneStyle::GameOfLife(_)
                    | StandaloneStyle::Mandelbrot
                    | StandaloneStyle::KittyFrames
            ));
        }

        assert!(saw_kitty_frames);
    }

    // Defends: standalone Game of Life keeps the same minimum-width sizing contract as the integrated screen renderer.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn game_of_life_context_preserves_inner_width_floor() {
        let context = game_of_life_context(20, 10);

        assert_eq!(context.size_class, "narrow");
        assert_eq!(
            context.inner_width,
            game_of_life_spec("narrow").minimum_inner_width
        );
        assert_eq!(context.resolved_height, 10);
    }

    // Defends: CLI parsing keeps the package preview simple while still exposing dotted Game of Life cells for parity with Yazelix.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn parse_args_accepts_style_and_cell_style_without_session_config() {
        let parsed = parse_args([
            "game_of_life_gliders".to_string(),
            "--cell-style".to_string(),
            "dotted".to_string(),
        ])
        .unwrap();

        assert_eq!(parsed.style, "game_of_life_gliders");
        assert_eq!(parsed.cell_style, GameOfLifeCellStyle::Dotted);
        assert_eq!(parsed.kitty_frame_dir, None);
        assert_eq!(parsed.kitty_frame_count, None);
        assert!(!parsed.help);
    }

    // Defends: standalone Kitty playback is opt-in to caller-provided frame assets instead of borrowing Yazelix runtime paths.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn parse_args_accepts_kitty_frame_assets() {
        let parsed = parse_args([
            "magician".to_string(),
            "--kitty-frame-dir".to_string(),
            "/tmp/magician_frames".to_string(),
            "--kitty-frame-count".to_string(),
            "198".to_string(),
        ])
        .unwrap();

        assert_eq!(parsed.style, "magician");
        assert_eq!(
            parsed.kitty_frame_dir,
            Some(PathBuf::from("/tmp/magician_frames"))
        );
        assert_eq!(parsed.kitty_frame_count, Some(198));
    }
}
