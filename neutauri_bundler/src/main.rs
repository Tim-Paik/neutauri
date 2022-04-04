mod bundle;
mod data;

fn main() -> std::io::Result<()> {
    let arg = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "neutauri.toml".into());
    bundle::bundle(arg)?;
    Ok(())
}
