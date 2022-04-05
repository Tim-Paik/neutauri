use gumdrop::Options;

mod bundle;
mod dev;
mod data;

#[derive(Debug, Options)]
struct Args {
    #[options(help = "print help information")]
    help: bool,
    #[options(help = "print version information")]
    version: bool,

    #[options(command)]
    command: Option<Command>,
}

#[derive(Debug, Clone, Options)]
enum Command {
    #[options(help = "pack a neutauri project")]
    Bundle(BundleOpts),
    #[options(help = "run the project in the current directory in development mode")]
    Dev(DevOpts),
}

#[derive(Debug, Clone, Options)]
struct BundleOpts {
    #[options(help = "print help information")]
    help: bool,
    #[options(help = "path to the config file [default: neutauri.toml]")]
    config: Option<String>,
}

#[derive(Debug, Clone, Options)]
struct DevOpts {
    #[options(help = "print help information")]
    help: bool,
    #[options(help = "path to the config file [default: neutauri.toml]")]
    config: Option<String>,
}

fn print_help_and_exit(args: Args) {
    if args.command.is_some() {
        Args::parse_args_default_or_exit();
        std::process::exit(0);
    }
    eprintln!(
        "Usage: {:?} [SUBCOMMAND] [OPTIONS]",
        std::env::args()
            .into_iter()
            .nth(0)
            .unwrap_or_else(|| "neutauri_bundler".to_string())
    );
    eprintln!();
    eprintln!("{}", args.self_usage());
    eprintln!();
    eprintln!("Available commands:");
    eprintln!("{}", args.self_command_list().unwrap());
    std::process::exit(0);
}

fn main() -> wry::Result<()> {
    let args = std::env::args().collect::<Vec<_>>();
    let args = Args::parse_args(&args[1..], gumdrop::ParsingStyle::default()).unwrap_or_else(|e| {
        eprintln!("{}: {}", args[0], e);
        std::process::exit(2);
    });
    match args.command.clone() {
        Some(command) => match command {
            Command::Bundle(opts) => {
                if opts.help_requested() {
                    print_help_and_exit(args);
                }
                let config_path = opts.config.unwrap_or_else(|| "neutauri.toml".to_string());
                bundle::bundle(config_path)?;
            }
            Command::Dev(opts) => {
                if opts.help_requested() {
                    print_help_and_exit(args);
                }
                let config_path = opts.config.unwrap_or_else(|| "neutauri.toml".to_string());
                dev::dev(config_path)?;
            },
        },
        None => print_help_and_exit(args),
    }
    Ok(())
}
