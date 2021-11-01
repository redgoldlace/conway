use app::{App, Options};
use std::{error::Error, time::Duration};

pub mod app;
pub mod cell;
pub mod world;

fn main() -> Result<(), Box<dyn Error>> {
    let stdout = std::io::stdout();
    let mut stdout = stdout.lock();

    App::new(Options {
        output: &mut stdout,
        tick_length: Duration::from_millis(100),
    })
    .run()
}
