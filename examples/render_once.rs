use std::io::{self, Write};

use yazelix_screen::{MandelbrotAnimation, ScreenAnimationContext, ScreenFrameProducer};

fn main() -> io::Result<()> {
    let mut args = std::env::args().skip(1);
    let width = args
        .next()
        .map(|raw| raw.parse::<usize>())
        .transpose()
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidInput, error))?
        .unwrap_or(80);
    let height = args
        .next()
        .map(|raw| raw.parse::<usize>())
        .transpose()
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidInput, error))?
        .unwrap_or(24);
    let frame_index = args
        .next()
        .map(|raw| raw.parse::<usize>())
        .transpose()
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidInput, error))?
        .unwrap_or(0);
    let context = ScreenAnimationContext {
        resolved_width: width,
        resolved_height: height,
        inner_width: width,
        size_class: "wide",
    };
    let mut animation = MandelbrotAnimation::new(context);
    for _ in 0..frame_index {
        animation.advance_frame();
    }
    let stdout = io::stdout();
    let mut stdout = stdout.lock();

    for line in animation.render_frame() {
        if let Err(error) = writeln!(stdout, "{line}") {
            if error.kind() == io::ErrorKind::BrokenPipe {
                return Ok(());
            }
            return Err(error);
        }
    }

    Ok(())
}
