use anyhow::Ok;
use neutauri_data as data;

const TEMPLATE: &str = include_str!("../../neutauri.toml.example");

pub(crate) fn init() -> anyhow::Result<()> {
    let config = TEMPLATE;
    let config = config.replace(
        "Neutauri Demo",
        &inquire::Text::new("The name of your program? (for window title)")
            .with_placeholder("Neutauri App")
            .with_default("Neutauri App")
            .prompt()?,
    );
    let config = config.replace(
        "web_src",
        &inquire::Text::new("Where is your web source code? (relative to the current directory)")
            .with_placeholder("web_src")
            .with_default("web_src")
            .prompt()?,
    );
    let config = config.replace(
        "neutauri_demo",
        &inquire::Text::new("The name of your output target?")
            .with_placeholder("app")
            .with_default("app")
            .prompt()?,
    );
    let config = config.replacen(
        "Small",
        inquire::Select::new(
            "The default size of the window?",
            vec!["Small", "Medium", "Large"],
        )
        .prompt()?,
        1,
    );
    let config_path = data::normalize_path(std::path::Path::new("./neutauri.toml"));
    std::fs::write(&config_path, config)?;
    eprintln!(
        "The configuration file has been written to \"{}\"",
        config_path.display()
    );
    Ok(())
}
