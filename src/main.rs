use polkahub_lib::{Project, POLKAHUB_URL};
use reqwest;

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    let project = Project::new();
    let hub = &format!("{}", POLKAHUB_URL);
    let response = project.send_create_request(hub).await?;
    response.process();

    Ok(())
}
