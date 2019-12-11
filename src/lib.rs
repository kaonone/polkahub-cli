use circle_rs::{Infinite, Progress};
use reqwest;
use serde_derive::{Deserialize, Serialize};
use serde_json::Value;
use structopt::StructOpt;

pub const POLKAHUB_URL: &str = "https://api.polkahub.org/api/v1/projects";

#[derive(StructOpt, Debug, Serialize, Deserialize, PartialEq)]
pub struct Project {
    pub account_id: u64,
    pub project_name: String,
}
#[derive(Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct Payload {
    pub status: String,
    pub repo_url: String,
    pub http_url: String,
    pub ws_url: String,
    pub repository_created: bool,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct Success {
    pub status: String,
    pub payload: Payload,
}
#[derive(Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct NotCreated {
    pub status: String,
    pub reason: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum Response {
    Success(Success),
    Fail(NotCreated),
}

impl Response {
    /// Destructure and act upon the result
    pub fn process(&self) {
        match self {
            Response::Success(s) => {
                println!("success {:#?}", s);
            }
            Response::Fail(e) => {
                println!("failed {:#?}", e);
            }
        }
    }
}

impl Project {
    pub fn new() -> Project {
        Project::from_args()
    }
    pub fn from(id: u64, name: String) -> Project {
        Project {
            account_id: id,
            project_name: name,
        }
    }
    pub async fn send_create_request(&self, url: &str) -> Result<Response, reqwest::Error> {
        let client = reqwest::Client::new();

        let mut loader = Infinite::new().to_stderr();
        println!(
            "\nCreating {:?} project. id: {:?} ",
            self.project_name, self.account_id,
        );
        loader.set_msg("");
        let _ = loader.start();

        let result: Value = client.post(url).json(self).send().await?.json().await?;

        let _ = loader.stop();
        println!("\n{:#?}", result);

        parse_response(result.to_string())
    }
}

pub fn parse_response(r: String) -> Result<Response, reqwest::Error> {
    let response = match serde_json::from_str(&r) {
        Ok(r) => Response::Success(Success { ..r }),
        Err(e) => Response::Fail(NotCreated {
            status: "json parse error".to_owned(),
            reason: e.to_string(),
        }),
    };
    Ok(response)
}

#[cfg(test)]
mod tests {
    use super::*;
    const P_ID: u64 = 5;
    const P_NAME: &str = "NAME";
    
    #[test]
    fn test_parse_works() {
        let project = Project::from(P_ID, P_NAME.to_owned());
        assert_eq!(
            project,
            Project {
                account_id: P_ID,
                project_name: String::from(P_NAME.to_owned()),
            }
        );
    }
}
