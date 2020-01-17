use anyhow::{anyhow, Result};
use circle_rs::{Infinite, Progress};
use reqwest::{self, header};
use rpassword;
use serde_derive::{Deserialize, Serialize};
use serde_json::{json, Value};
use structopt::StructOpt;
use termion::{color, style};
use tokio::{fs::File, io::AsyncReadExt};
use toml;

use std::{
    env,
    io::{self, Read, Write},
    path::{Path, PathBuf},
    str::FromStr,
    string::ToString,
};

pub const CREATE_URL: &str = "https://api-test.polkahub.org/api/v1/projects";
pub const INSTALL_URL: &str = "https://api-test.polkahub.org/api/v1/install";
pub const FIND_URL: &str = "https://api-test.polkahub.org/api/v1/find";
pub const REGISTER_URL: &str = "https://api-test.polkahub.org/api/v1/signup";
pub const LOGIN_URL: &str = "https://api-test.polkahub.org/api/v1/login";
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

#[derive(Debug, Serialize, Deserialize)]
struct PolkahubConfig {
    token: String,
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

#[derive(Debug, Deserialize)]
pub struct CreatedPayload {
    pub repo_url: String,
    pub http_url: String,
    pub ws_url: String,
    pub repository_created: bool,
}

#[derive(Debug, Deserialize)]
pub struct InstalledPayload {
    pub http_url: String,
    pub ws_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct Failure {
    pub status: String,
    pub reason: String,
}

#[derive(Deserialize, Debug)]
#[serde(tag = "status")]
enum CreatedResponse {
    #[serde(rename = "ok")]
    OkResult { payload: CreatedPayload },
    #[serde(rename = "error")]
    ErrResult { reason: String },
}

#[derive(Deserialize, Debug)]
#[serde(tag = "status")]
enum FoundResponse {
    #[serde(rename = "ok")]
    OkResult { payload: Vec<String> },
    #[serde(rename = "error")]
    ErrResult { reason: String },
}

#[derive(Deserialize, Debug)]
#[serde(tag = "status")]
enum InstalledResponse {
    #[serde(rename = "ok")]
    OkResult { payload: InstalledPayload },
    #[serde(rename = "error")]
    ErrResult { reason: String },
}

#[derive(Deserialize, Debug)]
#[serde(tag = "status")]
enum RegisteredResponse {
    #[serde(rename = "ok")]
    OkResult,
    #[serde(rename = "error")]
    ErrResult { reason: String },
}

#[derive(Deserialize, Debug)]
#[serde(tag = "status")]
enum LoginedResponse {
    #[serde(rename = "ok")]
    OkResult { token: String },
    #[serde(rename = "error")]
    ErrResult { reason: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Action {
    Install,
    Create,
    Find,
    Register,
    Login,
    Help,
    InputError(Failure),
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
            "auth" => Ok(Action::Login),
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

impl CreatedResponse {
    pub fn handle(&self) {
        match &self {
            CreatedResponse::OkResult { payload } => {
                print_green("done\n");
                print_blue("https ");
                println!(" -> {:?}", payload.http_url);
                print_blue("ws    ");
                println!(" -> {:?}", payload.ws_url);
                print_italic("remote");
                println!(" -> {:?}", payload.repo_url);
            }
            CreatedResponse::ErrResult { reason } => {
                let _ = err(Failure {
                    status: "Could not create project.\n".into(),
                    reason: format!("Reason: {}", reason),
                });
            }
        }
    }
}

impl InstalledResponse {
    pub fn handle(&self) {
        match &self {
            InstalledResponse::OkResult { payload } => {
                print_green("done\n");
                print_blue("https ");
                println!(" -> {:?}", payload.http_url);
                print_blue("ws    ");
                println!(" -> {:?}", payload.ws_url);
            }
            InstalledResponse::ErrResult { reason } => {
                let _ = err(Failure {
                    status: "Could not create project.\n".into(),
                    reason: format!("Reason: {}", reason),
                });
            }
        }
    }
}

impl FoundResponse {
    pub fn handle(&self, name: &str) {
        match self {
            FoundResponse::OkResult { payload } => {
                if payload.is_empty() {
                    print_green("Looks like no versions deployed yet!\n");
                    print!("");
                } else {
                    payload.iter().for_each(|v| {
                        println!("{} {}", name, v);
                    })
                }
            }
            FoundResponse::ErrResult { reason } => {
                let _ = err(Failure {
                    status: "Could not find project.\n".into(),
                    reason: format!("Reason: {}", reason),
                });
            }
        }
    }
}

impl RegisteredResponse {
    pub fn handle(&self) {
        match &self {
            RegisteredResponse::OkResult => {
                print_green("done\n");
            }
            RegisteredResponse::ErrResult { reason } => {
                let _ = err(Failure {
                    status: "Could not register new user.\n".into(),
                    reason: format!("Reason: {}", reason),
                });
            }
        }
    }
}

impl LoginedResponse {
    pub fn handle(&self) {
        match &self {
            LoginedResponse::OkResult { token } => match store_token(token) {
                Ok(()) => print_green("done\n"),
                Err(reason) => {
                    let _ = err(Failure {
                        status: "Could not login.\n".into(),
                        reason: format!("Reason: {}", reason),
                    });
                }
            },
            LoginedResponse::ErrResult { reason } => {
                let _ = err(Failure {
                    status: "Could not login.\n".into(),
                    reason: format!("Reason: {}", reason),
                });
            }
        }
    }
}

impl Project {
    pub fn new() -> Project {
        Project::from_args()
    }

    pub async fn create(&self) -> Result<()> {
        self.send_create_request(CREATE_URL).await?.handle();
        Ok(())
    }

    pub async fn find(&self) -> Result<()> {
        let name = if let Some(n) = &self.name { &n } else { "" };
        self.send_find_request(FIND_URL).await?.handle(&name);
        Ok(())
    }

    pub async fn install(&self) -> Result<()> {
        self.send_install_request(INSTALL_URL).await?.handle();
        Ok(())
    }

    pub async fn register(&self) -> Result<()> {
        let (email, password) = (read_email()?, read_password_with_confirmation()?);
        self.send_register_request(REGISTER_URL, &email, &password)
            .await?
            .handle();
        Ok(())
    }

    pub async fn login(&self) -> Result<()> {
        let (email, password) = (read_email()?, read_password()?);
        self.send_login_request(LOGIN_URL, &email, &password)
            .await?
            .handle();
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

    async fn send_create_request(&self, url: &str) -> Result<CreatedResponse> {
        let name = self.name.clone().unwrap_or_else(|| "".to_string());
        check_zero_len(&name, "You must provide name to create a project.".into())?;
        let body = json!({
            "project_name": name,
        });
        println!("\nCreating {} project", name);
        let response = self.post_request_with_token(url, body).await?;
        serde_json::from_str(&response).map_err(|e| e.into())
    }

    async fn send_find_request(&self, url: &str) -> Result<FoundResponse> {
        let name = self.name.clone().unwrap_or_else(|| "".to_string());
        check_zero_len(&name, "You must provide a project name to look for.".into())?;

        let body = json!({
            "project_name": name,
        });

        println!("\nLooking for {} project", name);
        let response = self.post_request_with_token(url, body).await?;
        serde_json::from_str(&response).map_err(|e| e.into())
    }

    async fn send_install_request(&self, url: &str) -> Result<InstalledResponse> {
        let base = self.version_split()?;
        let (name, version) = self.persist_hub(base.clone()).await?;
        let body = json!({
            "app_name": name,
            "project_name": base.0,
            "version": version,
        });
        println!("\nDeploying {} project with version {}", name, version);
        let response = self.post_request_with_token(url, body).await?;
        serde_json::from_str(&response).map_err(|e| e.into())
    }

    async fn send_register_request(
        &self,
        url: &str,
        email: &str,
        password: &str,
    ) -> Result<RegisteredResponse> {
        let body = json!({
            "email": email,
            "password": password,
        });
        println!("\nRegistration new user with email {}", email);
        let response = self.post_request(url, body).await?;
        serde_json::from_str(&response).map_err(|e| e.into())
    }

    async fn send_login_request(
        &self,
        url: &str,
        email: &str,
        password: &str,
    ) -> Result<LoginedResponse> {
        let body = json!({
            "email": email,
            "password": password,
        });
        println!("\nLogin user with email {}", email);
        let response = self.post_request(url, body).await?;
        serde_json::from_str(&response).map_err(|e| e.into())
    }

    async fn post_request(&self, url: &str, body: Value) -> Result<String> {
        let client = reqwest::Client::new();
        let mut loader = Infinite::new().to_stderr();
        loader.set_msg("");

        let _ = loader.start();
        let result = client.post(url).json(&body).send().await?.text().await?;
        let _ = loader.stop();

        Ok(result)
    }

    async fn post_request_with_token(&self, url: &str, body: Value) -> Result<String> {
        let token = read_token().map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("{:?}. Invalid token, please registered and auth first.", e),
            )
        })?;
        let mut headers = header::HeaderMap::new();
        let auth_data =
            header::HeaderValue::from_str(&format!("Bearer {}", token)).map_err(|e| {
                std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("{:?}. Invalid token, please registered and auth first.", e),
                )
            })?;
        headers.insert(header::AUTHORIZATION, auth_data);
        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()?;
        let mut loader = Infinite::new().to_stderr();
        loader.set_msg("");

        let _ = loader.start();
        let result = client.post(url).json(&body).send().await?.text().await?;
        let _ = loader.stop();

        Ok(result)
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
    print_blue("register ");
    println!(" - create a new user in Polkahub");
    print_blue("auth ");
    println!(" - log in to Polkahub");
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

fn read_password_with_confirmation() -> Result<String> {
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

fn read_password() -> Result<String> {
    let password = rpassword::read_password_from_tty(Some("Password: ")).unwrap();
    if password.len() < MIN_PASSWORD_LENGTH {
        let msg = format!("Password shorter than {} characters", MIN_PASSWORD_LENGTH);
        return Err(std::io::Error::new(std::io::ErrorKind::Other, msg).into());
    }
    if password.len() > MAX_PASSWORD_LENGTH {
        let msg = format!("Password longer than {} characters", MAX_PASSWORD_LENGTH);
        return Err(std::io::Error::new(std::io::ErrorKind::Other, msg).into());
    }
    Ok(password)
}

fn store_token(token: &str) -> Result<()> {
    let config = PolkahubConfig {
        token: token.to_string(),
    };
    let data = toml::to_string(&config)?;
    let path = polkahub_home_path();
    std::fs::create_dir_all(&path)?;
    let file_path = path.join("config");
    let mut file = std::fs::File::create(&file_path)?;
    file.write_all(data.as_bytes())?;
    Ok(())
}

fn read_token() -> Result<String> {
    let file_path = polkahub_home_path().join("config");
    let mut file = std::fs::File::open(&file_path)?;
    let mut data = String::new();
    file.read_to_string(&mut data)?;
    Ok(toml::from_str::<PolkahubConfig>(&data)?.token)
}

fn polkahub_home_path() -> PathBuf {
    if let Ok(polkahub_home) = env::var("POLKAHUB_HOME") {
        return Path::new(&polkahub_home).to_owned();
    }
    let home = env::var("HOME").expect("please set environment variable $HOME");
    Path::new(&home).join(".polkahub")
}
