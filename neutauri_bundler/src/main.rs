use clap::Parser;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Path to Congfig
    #[clap(short, long)]
    config: String,
}

fn main() {
    let args = Args::parse();
    println!("{}", args.config);
}
