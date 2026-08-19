mod id_types;
mod instructions;
mod parse;
mod program;
mod run;
mod types;

use clap::Parser;

/// Spirv emu
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Path to spirv
    #[arg(short, long)]
    path: String,
}

fn main() {
    let args = Args::parse();

    let prog = parse::parse(&args.path);
    run::run(prog);
}
