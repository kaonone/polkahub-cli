use circle_rs::{Infinite, Progress};
use reqwest;
use serde_derive::{Deserialize, Serialize};
use serde_json::Value;
use structopt::StructOpt;
use termion::{color, style};

pub const POLKAHUB_URL: &str = "https://api.polkahub.org/api/v1/projects";

pub fn print_green(s: &str) {
    print!(
        "{}{}{}",
        color::Fg(color::LightGreen),
        s,
        color::Fg(color::Reset)
    )
}
pub fn print_red(s: &str) {
    print!("{}{}{}", color::Fg(color::Red), s, color::Fg(color::Reset))
}
pub fn print_blue(s: &str) {
    print!(
        "{}{}{}",
        color::Fg(color::LightBlue),
        s,
        color::Fg(color::Reset)
    )
}

pub fn print_italic(s: &str) {
    print!("{}{}{}", style::Italic, s, style::Reset);
}

#[derive(StructOpt, Debug, Serialize, Deserialize, PartialEq)]
pub struct Project {
    pub account_id: u64,
    pub project_name: String,
}
#[derive(Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct Payload {
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
                let p = &s.payload;

                print_green("done\n");
                print_blue("https ");
                println!(" -> {:?}", p.http_url);
                print_blue("ws    ");
                println!(" -> {:?}", p.ws_url);
                print_italic("remote");
                println!(" -> {:?}", p.repo_url);
            }
            Response::Fail(e) => {
                print_red("Could not create project.\n");
                println!("Reason: {:?}", e.reason);
            }
        }
    }
}

impl Project {
    pub fn new() -> Project {
        Project::from_args()
    }
    #[allow(unused)]
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

        parse_response(result.to_string())
    }
}

pub fn parse_response(r: String) -> Result<Response, reqwest::Error> {
    let response = match serde_json::from_str(&r) {
        Ok(r) => Response::Success(r),
        Err(_) => parse_failure(r),
    };
    Ok(response)
}

pub fn parse_failure(r: String) -> Response {
    match serde_json::from_str(&r) {
        Ok(r) => Response::Fail(NotCreated { ..r }),
        Err(e) => Response::Fail(NotCreated {
            status: "json parse error".to_owned(),
            reason: e.to_string(),
        }),
    }
}
