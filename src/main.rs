use clap::{Args, Parser};

#[derive(Parser)]
#[command(author, version, about, long_about=None)]
struct Cli {
    #[command(flatten)]
    run: CliArgs,
}

#[derive(Args)]
struct CliArgs {
    file: std::path::PathBuf,
}

fn main() {
    let cli = Cli::parse();
    match chip8_rs::run(cli.run.file) {
        Ok(()) => {}
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(exitcode::DATAERR)
        }
    }
}
