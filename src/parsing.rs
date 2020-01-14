use anyhow::{anyhow, Result};
use circle_rs::{Infinite, Progress};
use reqwest;
use rpassword;
use serde_derive::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    io::{self, Write},
    path::Path,
    str::FromStr,
    string::ToString,
};
use structopt::StructOpt;
use termion::{color, style};
use tokio::{fs::File, io::AsyncReadExt};
use toml;

pub const INSTALL_URL: &str = "https://api-test.polkahub.org/api/v1/install";
pub const FIND_URL: &str = "https://api-test.polkahub.org/api/v1/find";
pub const REGISTER_URL: &str = "https://api-test.polkahub.org/api/v1/signup";
pub const POLKAHUB_URL: &str = "https://api-test.polkahub.org/api/v1/projects";
pub const HELP_NOTION: &str = "Try running `polkahub help` to see all available options";
const MIN_PASSWORD_LENGTH: usize = 8;
const MAX_PASSWORD_LENGTH: usize = 50;

pub fn print_green(s: &str) {
    let green = color::Fg(color::LightGreen);
    print!("{}{}{}", green, s, color::Fg(color::Reset))
}

pub fn print_red(s: &str) {
    print!("{}{}{}", color::Fg(color::Red), s, color::Fg(color::Reset))
}

pub fn print_yellow(s: &str) {
    let yellow = color::Fg(color::LightYellow);
    print!("{}{}{}", yellow, s, color::Fg(color::Reset))
}

pub fn print_blue(s: &str) {
    let blue = color::Fg(color::LightBlue);
    print!("{}{}{}", blue, s, color::Fg(color::Reset))
}

pub fn print_italic(s: &str) {
    print!("{}{}{}", style::Italic, s, style::Reset);
}

/// Main hub config
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Hub {
    parachain: Option<Parachain>,
    chainspec: Option<Chainspec>,
    node: Option<Node>,
}

///Parachain meta info
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct Parachain {
    name: String,
    description: String,
    version: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Chainspec {
    version: String,
    path: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Node {
    telemetry_url: String,
    listen_addr: String,
}

///
/// create project in polkahub registry,
/// find all available versions for deploy,
/// deploy specific version of your project to production
#[derive(StructOpt, Debug, Serialize, Deserialize, PartialEq)]
pub struct Project {
    /// create <name>, find <name>, install <name> <version>
    ///
    pub action: String,
    /// project name
    ///
    pub name: Option<String>,
    ///alias your deployed version in your environment
    ///
    #[structopt(short = "a")]
    pub alias: Option<String>,
    ///pick up your Hub.toml
    ///
    #[structopt(short = "h")]
    pub hub_file: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct Payload {
    pub repo_url: String,
    pub http_url: String,
    pub ws_url: String,
    pub repository_created: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct InstallPayload {
    pub http_url: String,
    pub ws_url: String,
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

#[derive(Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct Installed {
    pub status: String,
    pub payload: InstallPayload,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct Registered {
    pub status: String,
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
    Register,
    Help,
    InputError(Failure),
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum Response {
    Created(Created),
    Found(Found),
    Installed(Installed),
    Registered(Registered),
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
            "register" => Ok(Action::Register),
            _ => Ok(Action::InputError(Failure {
                status: "input error".to_owned(),
                reason: format!("{} - is invalid action. {}", s, HELP_NOTION),
            })),
        }
    }
}

impl Default for Hub {
    fn default() -> Self {
        Hub {
            parachain: None,
            chainspec: None,
            node: None,
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
                let _ = err(Failure {
                    status: "Could not create project.\n".into(),
                    reason: format!("Reason: {}", e.reason),
                });
            }
            _ => unreachable!(),
        }
    }

    pub fn handle_install(&self) {
        match &self {
            Response::Installed(s) => {
                let p = &s.payload;

                print_green("done\n");
                print_blue("https ");
                println!(" -> {:?}", p.http_url);
                print_blue("ws    ");
                println!(" -> {:?}", p.ws_url);
            }
            Response::Fail(e) => {
                let _ = err(Failure {
                    status: "Could not create project.\n".into(),
                    reason: format!("Reason: {}", e.reason),
                });
            }
            _ => unreachable!(),
        }
    }

    pub fn handle_find(&self, name: &str) {
        match self {
            Response::Found(s) => {
                let p = &s.payload;

                if p.is_empty() {
                    print_green("Looks like no versions deployed yet!\n");
                    print!("");
                } else {
                    p.iter().for_each(|v| {
                        println!("{} {}", name, v);
                    })
                }
            }
            Response::Fail(e) => {
                let _ = err(Failure {
                    status: "Could not find project.\n".into(),
                    reason: format!("Reason: {}", e.reason),
                });
            }
            _ => unreachable!(),
        }
    }

    pub fn handle_register(&self) {
        match &self {
            Response::Registered(_s) => {
                print_green("done\n");
            }
            Response::Fail(e) => {
                let _ = err(Failure {
                    status: "Could not register new user.\n".into(),
                    reason: format!("Reason: {}", e.reason),
                });
            }
            _ => unreachable!(),
        }
    }
}

impl Project {
    pub fn new() -> Project {
        Project::from_args()
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
        let response = self.send_install_request(INSTALL_URL).await?;
        response.handle_install();
        Ok(())
    }

    pub async fn register(&self) -> Result<()> {
        let (email, password) = (read_email()?, read_password()?);
        let response = self
            .send_register_request(REGISTER_URL, &email, &password)
            .await?;
        response.handle_register();
        Ok(())
    }

    pub fn parse_action(&self) -> Action {
        let a_parsed = Action::from_str(&self.action);
        match a_parsed {
            Ok(action) => action,
            Err(_) => Action::InputError(Failure {
                status: "Input error".to_owned(),
                reason: format!("{} - is invalid action. {}", self.action, HELP_NOTION),
            }),
        }
    }

    async fn send_create_request(&self, url: &str) -> Result<Response> {
        let name = self.name.clone().unwrap_or_else(|| "".to_string());
        check_zero_len(&name, "You must provide name to create a project.".into())?;
        let body = json!({
            "account_id": 1,
            "project_name": name,
        });
        println!("\nCreating {} project", name);
        self.post_request(url, body).await
    }

    async fn send_find_request(&self, url: &str) -> Result<Response> {
        let name = self.name.clone().unwrap_or_else(|| "".to_string());
        check_zero_len(&name, "You must provide a project name to look for.".into())?;

        let body = json!({
            "account_id": 1,
            "project_name": name,
        });

        println!("\nLooking for {} project", name);
        self.post_request(url, body).await
    }

    async fn send_install_request(&self, url: &str) -> Result<Response> {
        let base = self.version_split()?;
        let (name, version) = self.persist_hub(base.clone()).await?;
        let body = json!({
            "account_id": 1,
            "app_name": name,
            "project_name": base.0,
            "version": version,
        });
        println!("\nDeploying {} project with version {}", name, version);
        self.post_request(url, body).await
    }

    async fn send_register_request(
        &self,
        url: &str,
        email: &str,
        password: &str,
    ) -> Result<Response> {
        let body = json!({
            "email": email,
            "password": password,
        });
        println!("\nRegistration new user with email {}", email);
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

    /// split project with version
    fn version_split(&self) -> Result<(String, String)> {
        let s = self.name.clone().unwrap_or_else(|| "".to_string());
        check_version(s.clone())?;

        let project_name = s.split('@').nth(0).unwrap_or(&s).to_string();
        let v = s.split('@').nth(1).unwrap_or("").to_string();

        Ok((project_name, v))
    }

    /// if Hub.toml is present, use its data over flags
    async fn persist_hub(&self, project: (String, String)) -> Result<(String, String)> {
        let hub_file = self.hub_file.clone().unwrap_or_else(|| {
            // print warning if you provide an alias but have name in Hub.toml
            // (priority concerns)
            if self.alias.is_none() {
                print_yellow("WARN: ");
                print_italic("No Hub.toml path provided, looking in root directory\n");
            }
            "".to_string()
        });
        let hub = read_hubfile(hub_file).await?;
        // if hub exist take values from there
        let (app_name, version) = if let Some(p) = hub.parachain {
            (p.name, p.version)
        } else {
            // or take either alias or project name if none provided
            if let Some(alias) = self.alias.clone() {
                (alias, project.1)
            } else {
                project
            }
        };
        Ok((app_name, version))
    }
}

pub fn parse_response(r: String) -> Result<Response> {
    match serde_json::from_str(&r) {
        Ok(r) => Ok(Response::Created(r)),
        Err(_) => match serde_json::from_str(&r) {
            Ok(r) => Ok(Response::Found(r)),
            Err(_) => match serde_json::from_str(&r) {
                Ok(r) => Ok(Response::Installed(r)),
                Err(_) => match serde_json::from_str(&r) {
                    Ok(r) => Ok(Response::Registered(r)),
                    Err(_) => Ok(parse_failure(&r)),
                },
            },
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

pub fn err(e: Failure) -> Result<()> {
    let frame: String = e.status.chars().map(|_| '—').collect();
    println!(" {}", frame);
    print_red(&format!(" {}", e.status));
    println!(" {}", frame);
    println!("{}", e.reason);
    Err(anyhow!("{}", e.reason))
}

fn check_zero_len(s: &str, reason: String) -> Result<()> {
    if s.is_empty() {
        let f = Failure {
            status: "Input error".to_owned(),
            reason,
        };
        err(f)
    } else {
        Ok(())
    }
}

fn check_version(s: String) -> Result<()> {
    check_zero_len(&s, "You must provide a project name.".into())?;
    if !s.contains('@') {
        let f = Failure {
            status: "Input error".to_owned(),
            reason: "You must provide specific version to install: <project_name>@<version>"
                .to_owned(),
        };
        err(f)
    } else {
        Ok(())
    }
}

pub(crate) async fn read_hubfile(path: String) -> Result<Hub> {
    let trimmed = path.split("Hub.toml").nth(0).unwrap_or_else(|| &path);
    let file_path = Path::new(&trimmed).join("Hub.toml");
    let mut hub_file = vec![];
    let mut file = match File::open(file_path).await {
        Ok(f) => f,
        Err(_) => return Ok(Hub::default()),
    };
    file.read_to_end(&mut hub_file).await?;
    match String::from_utf8(hub_file) {
        Ok(f) => Ok(parse_toml(&f)),
        Err(_) => Ok(Hub::default()),
    }
}

fn parse_toml(f: &str) -> Hub {
    match toml::from_str::<Hub>(f) {
        Ok(hub) => hub,
        Err(_) => Hub::default(),
    }
}

fn read_email() -> Result<String> {
    let mut stream = std::fs::OpenOptions::new().write(true).open("/dev/tty")?;
    write!(stream, "Email: ")?;
    stream.flush()?;
    let mut email = String::new();
    std::io::stdin().read_line(&mut email)?;
    let email = email.trim();
    if !&email.contains('@') {
        let msg = "Email is invalid".to_string();
        return Err(std::io::Error::new(std::io::ErrorKind::Other, msg).into());
    }
    Ok(email.to_string())
}

fn read_password() -> Result<String> {
    let password = rpassword::read_password_from_tty(Some("Password: ")).unwrap();
    let confirm_password = rpassword::read_password_from_tty(Some("Confirm Password: ")).unwrap();
    if password.len() < MIN_PASSWORD_LENGTH {
        let msg = format!("Password shorter than {} characters", MIN_PASSWORD_LENGTH);
        return Err(std::io::Error::new(std::io::ErrorKind::Other, msg).into());
    }
    if password.len() > MAX_PASSWORD_LENGTH {
        let msg = format!("Password longer than {} characters", MAX_PASSWORD_LENGTH);
        return Err(std::io::Error::new(std::io::ErrorKind::Other, msg).into());
    }
    if password != confirm_password {
        let msg = "Password does not equal Confirm password".to_string();
        return Err(std::io::Error::new(std::io::ErrorKind::Other, msg).into());
    }
    Ok(password)
}
