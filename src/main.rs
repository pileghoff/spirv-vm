mod emu_parse;
mod emu_run;
mod emu_types;
mod id_types;

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

    let prog = emu_parse::parse(&args.path);
    emu_run::run(prog);
}
