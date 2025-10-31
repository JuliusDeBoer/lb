use asky::{Confirm, Password, Select, SelectOption, Text};
use clap::{Parser, Subcommand};
use eyre::{Context, Result, bail, ensure};
use serde::{Deserialize, Serialize};
use std::io::{self, BufRead};

#[derive(Serialize, Deserialize, Default)]
struct Config {
    gl_instance: Option<String>,
    gl_token: Option<String>,
    project: Option<u32>,
    issue: Option<u32>,
}

struct CompleteConfig {
    gl_instance: String,
    gl_token: String,
    project: u32,
    issue: u32,
}

impl TryInto<CompleteConfig> for Config {
    type Error = ();

    fn try_into(self) -> Result<CompleteConfig, Self::Error> {
        Ok(CompleteConfig {
            gl_instance: self.gl_instance.clone().ok_or(())?,
            gl_token: self.gl_token.clone().ok_or(())?,
            project: self.project.ok_or(())?,
            issue: self.issue.ok_or(())?,
        })
    }
}

#[derive(Deserialize, Debug)]
struct Issue {
    title: String,
}

#[derive(Deserialize, Debug)]
struct Project {
    id: u32,
    name: String,
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    Send,
    Configure,
}

fn send(cfg: CompleteConfig) -> Result<()> {
    let stdin = io::stdin();
    let mut buffer = String::new();

    for line in stdin.lock().lines() {
        buffer.push_str(format!("{}\\n", line?).as_str());
    }

    let client = reqwest::blocking::Client::new();
    let res = client
        .post(format!(
            "https://{}/api/v4/projects/{}/issues/{}/notes",
            cfg.gl_instance, cfg.project, cfg.issue,
        ))
        .body(format!("{{\"body\":\"{}\"}}", buffer))
        .header("PRIVATE-TOKEN", cfg.gl_token)
        .header("Content-Type", " application/json")
        .send()?;

    ensure!(
        res.status().is_success(),
        "Attempted to post content. However server returned non-successfully"
    );

    Ok(())
}

fn configure() -> Result<()> {
    let instance = Text::new("Enter target GitLab instance").prompt()?;
    let token = Password::new("Enter your GitLab personal access token").prompt()?;

    let client = reqwest::blocking::Client::new();

    let projects_res = client
        .get(format!("https://{}/api/v4/projects", instance))
        .header("PRIVATE-TOKEN", &token)
        .send()
        .context("Could not fetch programs")?;

    ensure!(
        projects_res.status().is_success(),
        "Attempted to fetch projects. However server returned non-sucessfully"
    );

    let projects: Vec<Project> = projects_res.json()?;

    let project_options = projects
        .into_iter()
        .map(|p| SelectOption {
            value: p.id,
            title: p.name,
            ..Default::default()
        })
        .collect();

    let selected_project = Select::new_complex("Select project", project_options).prompt()?;

    let mut selected_issue: u32;

    loop {
        selected_issue = Text::new("Select an issue")
            .initial("8")
            .validate(|v| {
                if v.parse::<u32>().is_ok() {
                    Ok(())
                } else {
                    Err("")
                }
            })
            .prompt()?
            .parse()?;

        let issue_res = client
            .get(format!(
                "https://{}/api/v4/projects/{}/issues/{}",
                instance, selected_project, selected_issue
            ))
            .header("PRIVATE-TOKEN", &token)
            .send()
            .context("Could not fetch programs")?;

        ensure!(
            issue_res.status().is_success(),
            "Attempted to fetch issue with id {}. However server returned non-sucessfully",
            selected_issue
        );

        let issue: Issue = issue_res.json()?;

        if Confirm::new(format!("Select the issue: \"{}\"?", issue.title).as_str()).prompt()? {
            break;
        }
    }

    confy::store(
        "lb",
        None,
        Config {
            gl_instance: Some(instance),
            gl_token: Some(token),
            issue: Some(selected_issue),
            project: Some(selected_project),
        },
    )?;

    Ok(())
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let cfg: Config = confy::load("lb", None)?;

    match cli.command {
        Command::Send => match cfg.try_into() {
            Ok(cfg) => {
                send(cfg)?;
            }
            Err(_) => {
                println!("Error: Looks like the configuration is not complete.");
                println!("Hint: Run `lb configure` to generate one.");
                bail!("Incomplete configuration");
            }
        },
        Command::Configure => match <Config as TryInto<CompleteConfig>>::try_into(cfg) {
            Ok(_) => {
                if Confirm::new("Looks like a config already exists. Want to create a new one?")
                    .initial(false)
                    .prompt()?
                {
                    configure()?;
                }
            }
            Err(_) => {
                configure()?;
            }
        },
    };

    Ok(())
}
