use mcman::manifest::Manifest;
use miette::{IntoDiagnostic, Result, WrapErr};

#[tokio::main]
async fn main() -> Result<()> {
    let path = std::env::current_dir().into_diagnostic()?.join("mcman.kdl");
    let text = tokio::fs::read_to_string(&path)
        .await
        .into_diagnostic()
        .wrap_err_with(|| format!("reading manifest {}", path.display()))?;
    let manifest = Manifest::parse(&path.to_string_lossy(), &text)?;

    println!("{manifest:#?}");

    Ok(())
}
