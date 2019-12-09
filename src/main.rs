use polkahub_lib::Project;
use reqwest;


#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    let project = Project::new();
    println!("{:?}", project);

    Ok(())
}
