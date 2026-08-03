use mcman::manifest::Manifest;
use miette::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let path = std::env::current_dir().unwrap().join("mcman.kdl");
    let text = tokio::fs::read_to_string(path).await.unwrap();
    let manifest = knus::parse::<Manifest>("mcman.kdl", &text)?;

    println!("{manifest:#?}");

    Ok(())
}
