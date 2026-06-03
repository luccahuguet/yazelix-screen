use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Duration;

use crate::KittyFrameSequence;

pub const MAGICIAN_ATTRIBUTION: &str = "ascii magician GIF by 1mposter";
pub const MAGICIAN_FRAME_COUNT: usize = 198;
pub const MAGICIAN_FRAME_DELAY: Duration = Duration::from_millis(90);
pub const MAGICIAN_FRAME_DIR_NAME: &str = "ascii_magician_1mposter_frames";
pub const MAGICIAN_GIF_NAME: &str = "ascii_magician_1mposter.gif";
pub const MAGICIAN_EDGE_INSET_COLUMNS: usize = 8;
pub const MAGICIAN_EDGE_INSET_ROWS: usize = 8;

pub fn magician_frame_path(frame_dir: &Path, frame_index: usize) -> PathBuf {
    frame_dir.join(format!(
        "frame_{:03}.png",
        frame_index % MAGICIAN_FRAME_COUNT
    ))
}

pub fn magician_frame_paths(frame_dir: &Path) -> Vec<PathBuf> {
    (0..MAGICIAN_FRAME_COUNT)
        .map(|frame_index| magician_frame_path(frame_dir, frame_index))
        .collect()
}

pub fn require_magician_frame_assets(frame_dir: &Path) -> io::Result<()> {
    for frame_path in magician_frame_paths(frame_dir) {
        if !frame_path.is_file() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("Missing magician GIF frame asset: {}", frame_path.display()),
            ));
        }
    }
    Ok(())
}

pub fn magician_frame_assets_available(frame_dir: &Path) -> bool {
    require_magician_frame_assets(frame_dir).is_ok()
}

pub fn magician_frame_sequence(
    frame_dir: &Path,
    image_id: u32,
    attribution: Option<String>,
) -> KittyFrameSequence {
    magician_frame_sequence_with_edge_insets(
        frame_dir,
        image_id,
        attribution,
        MAGICIAN_EDGE_INSET_COLUMNS,
        MAGICIAN_EDGE_INSET_ROWS,
    )
}

pub fn magician_frame_sequence_with_edge_insets(
    frame_dir: &Path,
    image_id: u32,
    attribution: Option<String>,
    edge_inset_columns: usize,
    edge_inset_rows: usize,
) -> KittyFrameSequence {
    KittyFrameSequence {
        frame_paths: magician_frame_paths(frame_dir),
        frame_delay: MAGICIAN_FRAME_DELAY,
        image_id,
        attribution,
        edge_inset_columns,
        edge_inset_rows,
    }
}

pub fn bundled_magician_frame_dir_from_exe() -> Option<PathBuf> {
    let exe = std::env::current_exe().ok()?;
    let package_root = exe.parent()?.parent()?;
    Some(
        package_root
            .join("share")
            .join("yazelix_screen")
            .join(MAGICIAN_FRAME_DIR_NAME),
    )
}

pub fn bundled_magician_gif_from_exe() -> Option<PathBuf> {
    let exe = std::env::current_exe().ok()?;
    let package_root = exe.parent()?.parent()?;
    Some(
        package_root
            .join("share")
            .join("yazelix_screen")
            .join(MAGICIAN_GIF_NAME),
    )
}

pub fn source_magician_frame_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("assets")
        .join("third_party")
        .join(MAGICIAN_FRAME_DIR_NAME)
}

pub fn source_magician_gif_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("assets")
        .join("third_party")
        .join(MAGICIAN_GIF_NAME)
}

pub fn default_magician_frame_dir() -> Option<PathBuf> {
    bundled_magician_frame_dir_from_exe()
        .filter(|path| magician_frame_assets_available(path))
        .or_else(|| {
            let source_dir = source_magician_frame_dir();
            magician_frame_assets_available(&source_dir).then_some(source_dir)
        })
        .or_else(|| {
            default_magician_cache_frame_dir().filter(|path| magician_frame_assets_available(path))
        })
}

pub fn default_magician_gif_path() -> Option<PathBuf> {
    bundled_magician_gif_from_exe()
        .filter(|path| path.is_file())
        .or_else(|| {
            let source_gif = source_magician_gif_path();
            source_gif.is_file().then_some(source_gif)
        })
}

pub fn default_magician_cache_frame_dir() -> Option<PathBuf> {
    std::env::var_os("XDG_CACHE_HOME")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var_os("HOME")
                .filter(|value| !value.is_empty())
                .map(|home| PathBuf::from(home).join(".cache"))
        })
        .map(|cache_root| {
            cache_root
                .join("yazelix_screen")
                .join(MAGICIAN_FRAME_DIR_NAME)
        })
}

pub fn imagemagick_available() -> bool {
    Command::new("magick")
        .arg("-version")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|status| status.success())
}

pub fn magician_default_generation_available() -> bool {
    default_magician_frame_dir().is_some()
        || (default_magician_gif_path().is_some()
            && default_magician_cache_frame_dir().is_some()
            && imagemagick_available())
}

pub fn ensure_default_magician_frame_dir() -> io::Result<PathBuf> {
    if let Some(frame_dir) = default_magician_frame_dir() {
        return Ok(frame_dir);
    }

    let gif_path = default_magician_gif_path().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            format!("Missing magician source GIF asset: {MAGICIAN_GIF_NAME}"),
        )
    })?;
    let frame_dir = default_magician_cache_frame_dir().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            "Could not resolve a cache directory for magician PNG frames; set XDG_CACHE_HOME or HOME",
        )
    })?;
    generate_magician_frame_assets(&gif_path, &frame_dir)?;
    Ok(frame_dir)
}

pub fn generate_magician_frame_assets(gif_path: &Path, frame_dir: &Path) -> io::Result<()> {
    fs::create_dir_all(frame_dir)?;
    remove_generated_magician_frames(frame_dir)?;

    let output_pattern = frame_dir.join("frame_%03d.png");
    let output = Command::new("magick")
        .arg(gif_path)
        .arg("-coalesce")
        .arg(&output_pattern)
        .output()
        .map_err(|source| {
            if source.kind() == io::ErrorKind::NotFound {
                io::Error::new(
                    io::ErrorKind::NotFound,
                    "ImageMagick `magick` was not found on PATH; install ImageMagick or pass --kitty-frame-dir",
                )
            } else {
                source
            }
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!(
                "ImageMagick failed to generate magician PNG frames from {}: {}",
                gif_path.display(),
                stderr.trim()
            ),
        ));
    }

    require_magician_frame_assets(frame_dir)
}

fn remove_generated_magician_frames(frame_dir: &Path) -> io::Result<()> {
    for entry in fs::read_dir(frame_dir)? {
        let path = entry?.path();
        let is_generated_frame = path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.starts_with("frame_"))
            && path
                .extension()
                .and_then(|extension| extension.to_str())
                .is_some_and(|extension| extension.eq_ignore_ascii_case("png"));
        if is_generated_frame {
            fs::remove_file(path)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test lane: default

    // Defends: the magician animation asset contract lives with yazelix_screen instead of main Yazelix.
    #[test]
    fn magician_frame_sequence_uses_owned_asset_contract() {
        let frame_dir = Path::new("/frames");
        let sequence = magician_frame_sequence(frame_dir, 77, Some("credit".to_string()));

        assert_eq!(sequence.frame_paths.len(), MAGICIAN_FRAME_COUNT);
        assert_eq!(
            sequence.frame_paths[0],
            PathBuf::from("/frames/frame_000.png")
        );
        assert_eq!(
            sequence.frame_paths[MAGICIAN_FRAME_COUNT - 1],
            PathBuf::from("/frames/frame_197.png")
        );
        assert_eq!(
            magician_frame_path(frame_dir, MAGICIAN_FRAME_COUNT),
            sequence.frame_paths[0]
        );
        assert_eq!(sequence.frame_delay, MAGICIAN_FRAME_DELAY);
        assert_eq!(sequence.image_id, 77);
        assert_eq!(sequence.attribution.as_deref(), Some("credit"));
        assert_eq!(sequence.edge_inset_columns, MAGICIAN_EDGE_INSET_COLUMNS);
        assert_eq!(sequence.edge_inset_rows, MAGICIAN_EDGE_INSET_ROWS);
    }

    // Defends: integrated consumers can choose pane-chrome-specific padding without changing the standalone magician default.
    #[test]
    fn magician_frame_sequence_accepts_consumer_owned_edge_insets() {
        let frame_dir = Path::new("/runtime/magician");
        let sequence = magician_frame_sequence_with_edge_insets(
            frame_dir,
            77,
            Some("credit".to_string()),
            12,
            14,
        );

        assert_eq!(sequence.edge_inset_columns, 12);
        assert_eq!(sequence.edge_inset_rows, 14);
    }
}
