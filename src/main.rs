use std::process;

use hyprmonis::Config;

fn main() -> std::io::Result<()> {
    let config = Config::build().unwrap_or_else(|err| {
        eprintln!("Environment error: {err}");
        process::exit(1);
    });

    hyprmonis::run(config)?;

    Ok(())
}
