use std::io;
use std::path::{Path, PathBuf};
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

pub fn source_magician_frame_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("assets")
        .join("third_party")
        .join(MAGICIAN_FRAME_DIR_NAME)
}

pub fn default_magician_frame_dir() -> Option<PathBuf> {
    bundled_magician_frame_dir_from_exe()
        .filter(|path| path.is_dir())
        .or_else(|| {
            let source_dir = source_magician_frame_dir();
            source_dir.is_dir().then_some(source_dir)
        })
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
