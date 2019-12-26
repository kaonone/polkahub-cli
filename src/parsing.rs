use anyhow::{anyhow, Result};
use circle_rs::{Infinite, Progress};
use reqwest;
use serde_derive::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{io, str::FromStr, string::ToString};
use structopt::StructOpt;
use termion::{color, style};

pub const POLKAHUB_URL: &str = "http://localhost:8080/api/v1/projects";
pub const INSTALL_URL: &str = "http://localhost:8080/api/v1/install";
pub const FIND_URL: &str = "http://localhost:8080/api/v1/find";
// pub const INSTALL_URL: &str = "https://api.polkahub.org/api/v1/install";
// pub const FIND_URL: &str = "https://api.polkahub.org/api/v1/find";
// pub const POLKAHUB_URL: &str = "https://api.polkahub.org/api/v1/projects";
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

///
/// create project in polkahub registry,
/// find all available versions for deploy,
/// deploy specific version of your project to production
#[derive(StructOpt, Debug, Serialize, Deserialize, PartialEq)]
pub struct Project {
    /// create, find, install <name> <version>
    pub action: String,
    /// project name
    pub name: Option<String>,
    /// install specific version
    pub version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct Payload {
    pub repo_url: String,
    pub http_url: String,
    pub ws_url: String,
    pub repository_created: bool,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct Created {
    pub status: String,
    pub payload: Payload,
}
#[derive(Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct Found {
    pub status: String,
    pub payload: Vec<String>,
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
    Created(Created),
    Found(Found),
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
        match &self {
            Response::Created(s) => {
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
            _ => unreachable!(),
        }
    }
    pub fn handle_find(&self, name: &str) {
        match self {
            Response::Found(s) => {
                let p = &s.payload;

                p.iter().for_each(|v| {
                    println!("{} {}", name, v);
                })
            }
            Response::Fail(e) => {
                print_red("Could not find project.\n");
                println!("Reason: {}", e.reason);
            }
            _ => unreachable!(),
        }
    }
    pub fn handle_install(&self) {
        // match &self {
        //     Response::Created(s) => {
        //         let mut p = s.payload;

        //         print_green("done\n");
        //         print_blue("https ");
        //         println!(" -> {:?}", p.http_url);
        //         print_blue("ws    ");
        //         println!(" -> {:?}", p.ws_url);
        //         print_italic("remote");
        //         println!(" -> {:?}", p.repo_url);
        //     }
        //     Response::Fail(e) => {
        //         print_red("Could not create project.\n");
        //         println!("Reason: {}", e.reason);
        //     }
        // }
    }
}

impl Project {
    pub fn new() -> Project {
        Project::from_args()
    }

    pub fn err(&self, e: Failure) -> Result<()> {
        let frame: String = e.status.chars().map(|_| '—').collect();
        println!(" {}", frame);
        print_red(&format!(" {}", e.status));
        println!("\n {}", frame);
        Err(anyhow!("{}", e.reason))
    }
    pub async fn create(&self) -> Result<()> {
        let response = self.send_create_request(POLKAHUB_URL).await?;
        response.handle_create();
        Ok(())
    }
    pub async fn find(&self) -> Result<()> {
        let response = self.send_find_request(FIND_URL).await?;
        let name = if let Some(n) = &self.name { &n } else { "" };
        response.handle_find(name);
        Ok(())
    }
    pub async fn install(&self) -> Result<()> {
        println!("{:?}", self);
        let response = self.send_install_request(INSTALL_URL).await?;
        response.handle_install();
        Ok(())
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
    async fn send_create_request(&self, url: &str) -> Result<Response> {
        let name = self.name.clone().unwrap_or("".to_string());
        self.check_zero_len(
            name.clone(),
            "You must provide name to create a project.".into(),
        )?;
        let body = json!({
            "account_id": 1,
            "project_name": name,
        });
        
        println!("\nCreating {} project", name);
        self.post_request(url, body).await
    }

    async fn send_find_request(&self, url: &str) -> Result<Response> {
        let name = self.name.clone().unwrap_or("".to_string());
        self.check_zero_len(
            name.clone(),
            "You must provide a project name to look for.".into(),
        )?;

        let body = json!({
            "id": 1,
            "project_name": name,
        });

        println!("Looking for {} project", name);
        self.post_request(url, body).await
    }
    
    async fn send_install_request(&self, url: &str) -> Result<Response> {
        let name = self.name.clone().unwrap_or("".to_string());
        let version = self.version.clone().unwrap_or("".to_string());
        self.check_zero_len(name.clone(), "You must provide a project name.".into())?;
        self.check_zero_len(
            version.clone(),
            "You must provide specific version to install.".into(),
        )?;
        
        let body = json!({
            "account_id": 1,
            "project_name": name,
            "version": version,
        });
        
        println!("Deploying {} project with version {}", name, version);
        self.post_request(url, body).await
    }

    async fn post_request(&self, url: &str, body: Value) -> Result<Response> {
        let client = reqwest::Client::new();
        let mut loader = Infinite::new().to_stderr();
        loader.set_msg("");

        let _ = loader.start();
        let result: Value = client.post(url).json(&body).send().await?.json().await?;
        let _ = loader.stop();

        parse_response(result.to_string())
    }

    fn check_zero_len(&self, s: String, reason: String) -> Result<()> {
        if s.len() == 0 {
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
    match serde_json::from_str(&r) {
        Ok(r) => Ok(Response::Created(r)),
        Err(_) => match serde_json::from_str(&r) {
            Ok(r) => Ok(Response::Found(r)),
            Err(e) => {
                print_blue(&format!("serde success fail response {:?}\n", e));
                Ok(parse_failure(&r))
            }
        },
    }
}

pub fn parse_failure(r: &str) -> Response {
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
