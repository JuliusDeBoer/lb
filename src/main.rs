use asky::{Confirm, Password, Select, SelectOption, Text};
use clap::{Parser, Subcommand};
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
struct Project {
    id: u32,
    name: String,
}

#[derive(Deserialize, Debug)]
struct Issue {
    id: u32,
    iid: u32,
    title: String,
    state: String,
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

fn send(cfg: CompleteConfig) {
    let stdin = io::stdin();
    let mut buffer = String::new();

    for line in stdin.lock().lines() {
        buffer.push_str(format!("{}\\n", line.unwrap()).as_str());
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
        .send()
        .unwrap();

    if !res.status().is_success() {
        println!(
            "Failed to send!\n\nResponse body:\n\n{}",
            res.text().unwrap()
        );
        // FIXME(Julius)
        std::process::exit(1)
    }
}

fn configure() {
    let instance = Text::new("Enter target Gitlab instance").prompt().unwrap();
    let token = Password::new("What is your token").prompt().unwrap();

    let client = reqwest::blocking::Client::new();

    let projects_res = client
        .get(format!("https://{}/api/v4/projects", instance))
        .header("PRIVATE-TOKEN", &token)
        .send()
        .unwrap();

    if !projects_res.status().is_success() {
        // FIXME(Julius)
        return;
    }

    let projects: Vec<Project> = projects_res.json().unwrap();
    let project_options = projects
        .into_iter()
        .map(|p| SelectOption {
            value: p.id,
            title: p.name,
            ..Default::default()
        })
        .collect();

    let selected_project = Select::new_complex("Select project", project_options)
        .prompt()
        .unwrap();

    let selected_issue = Text::new("Select an issue")
        .initial("8")
        .validate(|v| {
            if v.parse::<u32>().is_ok() {
                Ok(())
            } else {
                Err("")
            }
        })
        .prompt()
        .unwrap();

    // TODO(Julius): Check if project is valid

    confy::store(
        "lb",
        None,
        Config {
            gl_instance: Some(instance),
            gl_token: Some(token),
            issue: Some(selected_issue.parse::<u32>().unwrap()),
            project: Some(selected_project),
        },
    )
    .unwrap();
}

fn main() {
    let cli = Cli::parse();
    let cfg: Config = confy::load("lb", None).unwrap();

    match cli.command {
        Command::Send => match cfg.try_into() {
            Ok(cfg) => send(cfg),
            Err(_) => {
                println!("Error: Looks like the configuration is not complete.");
                println!("Hint: Run `lb configure` to generate one.");
                std::process::exit(1);
            }
        },
        Command::Configure => match <Config as TryInto<CompleteConfig>>::try_into(cfg) {
            Ok(_) => {
                if Confirm::new("Looks like a config already exists. Want to create a new one?")
                    .initial(false)
                    .prompt()
                    .unwrap()
                {
                    configure();
                }
            }
            Err(_) => configure(),
        },
    }
}
