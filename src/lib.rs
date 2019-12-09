use serde_derive::{Deserialize, Serialize};
use structopt::StructOpt;
use reqwest;
use std::{thread, time::{Duration, Instant}};

pub const POLKAHUB_URL: &str = "";
pub const PROJECTS: &str = "/api/v1/projects";

#[derive(StructOpt, Debug, Serialize, Deserialize, PartialEq)]
pub struct Project {
    pub account_id: u64,
    pub project_name: String,
}
#[derive(Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct Payload {
    repository_created: bool,
    repo_url: String,
    http_url: String,
    ws_url: String,
}
#[derive(Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct Response {
    status: String,
    payload: Payload,
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
    pub async fn send_create_request(&self) -> Result<(), reqwest::Error> {
        let client = reqwest::Client::new();
        let result = client
            .post("https://jsonplaceholder.typicode.com/posts")
            .json(self)
            .send()
            .await?
            .json()
            .await?;
        
        thread::sleep(Duration::from_secs(20));
        println!("{:#?}", result);
        Ok(())
    }
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
    // #[test]
    // fn test_send_works() {
    //     let project = Project::from(P_ID, P_NAME.to_owned());
    //     let result = project.send_create_request();
    //     assert_eq!(result, ());
    // }
}
