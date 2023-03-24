use gumdrop::Options;
mod bundle;
mod dev;
mod init;

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
    #[options(help = "initialize a neutauri project")]
    Init(InitOpts),
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

#[derive(Debug, Clone, Options)]
struct InitOpts {
    #[options(help = "print help information")]
    help: bool,
}

fn print_help_and_exit(args: Args) {
    if args.command.is_some() {
        Args::parse_args_default_or_exit();
        std::process::exit(0);
    }
    eprintln!(
        "Usage: {:?} [SUBCOMMAND] [OPTIONS]",
        std::env::args()
            .next()
            .unwrap_or_else(|| "neutauri_bundler".to_string())
    );
    eprintln!();
    eprintln!("{}", args.self_usage());
    eprintln!();
    eprintln!("Available commands:");
    eprintln!("{}", args.self_command_list().unwrap());
    std::process::exit(0);
}

fn main() -> anyhow::Result<()> {
    let args = std::env::args().collect::<Vec<_>>();
    let args = Args::parse_args(&args[1..], gumdrop::ParsingStyle::default()).unwrap_or_else(|e| {
        eprintln!("{}: {}", args[0], e);
        std::process::exit(2);
    });
    match args.command.clone() {
        Some(command) => match command {
            Command::Bundle(opts) => {
                if opts.help_requested() {
                    eprintln!("Package according to the configuration in neutauri.toml");
                    eprintln!();
                    print_help_and_exit(args);
                }
                let config_path = opts.config.unwrap_or_else(|| "neutauri.toml".to_string());
                bundle::bundle(config_path)?;
            }
            Command::Dev(opts) => {
                if opts.help_requested() {
                    eprintln!("Check the configuration in neutauri.toml and start directly");
                    eprintln!();
                    print_help_and_exit(args);
                }
                let config_path = opts.config.unwrap_or_else(|| "neutauri.toml".to_string());
                dev::dev(config_path)?;
            }
            Command::Init(opts) => {
                if opts.help_requested() {
                    eprintln!("Interactively create a neutauri.toml configuration file");
                    eprintln!();
                    print_help_and_exit(args);
                }
                init::init()?;
            }
        },
        None => print_help_and_exit(args),
    }
    Ok(())
}
