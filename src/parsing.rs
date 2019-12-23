use anyhow::{anyhow, Result};
use circle_rs::{Infinite, Progress};
use reqwest;
use serde_derive::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{io, str::FromStr, string::ToString};
use structopt::StructOpt;
use termion::{color, style};

pub const INSTALL_URL: &str = "https://api.polkahub.org/api/v1/install";
pub const FIND_URL: &str = "https://api.polkahub.org/api/v1/find";
pub const POLKAHUB_URL: &str = "https://api.polkahub.org/api/v1/projects";
pub const HELP_NOTION: &str = "Try running `polkahub help` to see all available options";

pub fn print_green(s: &str) {
    let green = color::Fg(color::LightGreen);
    let reset = color::Fg(color::Reset);
    print!("{}{}{}", green, s, reset)
}
pub fn print_red(s: &str) {
    print!("{}{}{}", color::Fg(color::Red), s, color::Fg(color::Reset))
}
pub fn print_blue(s: &str) {
    let blue = color::Fg(color::LightBlue);
    let reset = color::Fg(color::Reset);
    print!("{}{}{}", blue, s, reset)
}
pub fn print_italic(s: &str) {
    print!("{}{}{}", style::Italic, s, style::Reset);
}

#[derive(StructOpt, Debug, Serialize, Deserialize, PartialEq)]
pub struct Project {
    pub action: String,
    #[structopt(name = "name", long, short)]
    pub name: Option<String>,
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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct Failure {
    pub status: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Action {
    Install,
    Create,
    Find,
    Help,
    InputError(Failure),
}
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum Response {
    Success(Success),
    Fail(Failure),
}

impl FromStr for Action {
    type Err = io::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "create" => Ok(Action::Create),
            "find" => Ok(Action::Find),
            "help" => Ok(Action::Help),
            "install" => Ok(Action::Install),
            _ => Ok(Action::InputError(Failure {
                status: "input error".to_owned(),
                reason: format!("{} - is invalid action. {}", s, HELP_NOTION),
            })),
        }
    }
}

impl Response {
    /// Destructure and act upon the result
    pub fn handle_create(&self) {
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
                println!("Reason: {}", e.reason);
            }
        }
    }
}

impl Project {
    pub fn new() -> Project {
        Project::from_args()
    }

    pub fn err(&self, e: Failure) -> Result<()> {
        print_red(&format!("{}\n", e.status));
        println!();
        Err(anyhow!("{}", e.reason))
    }
    pub async fn create(&self) -> Result<()> {
        let response = self.send_create_request(POLKAHUB_URL).await?;
        response.handle_create();
        Ok(())
    }
    pub async fn send_create_request(&self, url: &str) -> Result<Response> {
        let client = reqwest::Client::new();

        let mut loader = Infinite::new().to_stderr();
        let name = self.name.clone().unwrap_or("".to_string());
        self.check_name(
            name.clone(),
            "You must provide name to create a project.".into(),
        )?;
        println!("\nCreating {} project", name);
        let body = json!({
            "account_id": 1,
            "project_name": name,
        });
        loader.set_msg("");
        let _ = loader.start();
        let result: Value = client.post(url).json(&body).send().await?.json().await?;
        let _ = loader.stop();

        parse_response(result.to_string())
    }

    pub fn parse_action(&self) -> Action {
        let a_parsed = Action::from_str(&self.action);
        match a_parsed {
            Ok(action) => action,
            Err(e) => {
                println!("{} {:?}", self.action, e);
                Action::InputError(Failure {
                    status: "Input error".to_owned(),
                    reason: format!("{} - is invalid action. {}", self.action, HELP_NOTION),
                })
            }
        }
    }
    pub fn check_name(&self, name: String, reason: String) -> Result<()> {
        if name.len() == 0 {
            let f = Failure {
                status: "Input error".to_owned(),
                reason,
            };
            self.err(f)?;
        }
        Ok(())
    }
}

pub fn parse_response(r: String) -> Result<Response> {
    let response = match serde_json::from_str(&r) {
        Ok(r) => Response::Success(r),
        Err(_) => parse_failure(r),
    };
    Ok(response)
}

pub fn parse_failure(r: String) -> Response {
    match serde_json::from_str(&r) {
        Ok(r) => Response::Fail(Failure { ..r }),
        Err(e) => Response::Fail(Failure {
            status: "json parse error".to_owned(),
            reason: e.to_string(),
        }),
    }
}

pub fn print_help() -> Result<()> {
    println!("Usage:");
    print_blue("help ");
    println!(" - list all possible options");
    print_blue("install ");
    println!(" - launch parachain node");
    print_blue("find ");
    println!(" - find all versions of your project");
    print_blue("create ");
    println!(" - register new parachain and create endpoints");
    Ok(())
}
