use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File},
    io::BufReader,
    path::PathBuf,
};

use clap::Parser;

fn default_template_deps() -> Vec<String> {
    vec![]
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
struct TemplateUnit {
    #[serde(default = "default_template_deps")]
    pub requires: Vec<String>,
    #[serde(default = "default_template_deps")]
    pub after: Vec<String>,
    #[serde(default = "default_template_deps")]
    pub wants: Vec<String>
}

fn default_inherit_requires() -> bool {
    true
}

fn default_inherit_after() -> bool {
    true
}

fn default_inherit_wants() -> bool {
    true
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
struct InstanceUnit {
    pub name: String,
    pub description: String,
    pub requires: Option<Vec<String>>,
    pub after: Option<Vec<String>>,
    pub wants: Option<Vec<String>>,
    #[serde(default = "default_inherit_requires")]
    pub inherit_requires: bool,
    #[serde(default = "default_inherit_after")]
    pub inherit_after: bool,
    #[serde(default = "default_inherit_wants")]
    pub inherit_wants: bool,
    pub requires_mounts_for: Option<Vec<String>>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
enum Restart {
    No,
    Always,
    OnSuccess,
    OnFailure,
    OnWatchdog,
    OnAbort,
}

impl core::fmt::Display for Restart {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Restart::No => "no",
            Restart::Always => "always",
            Restart::OnSuccess => "on-success",
            Restart::OnFailure => "on-failure",
            Restart::OnWatchdog => "on-watchdog",
            Restart::OnAbort => "on-abort",
        })
        .unwrap();
        Ok(())
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
enum RemainAfterExit {
    No,
    Yes,
}

impl core::fmt::Display for RemainAfterExit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            RemainAfterExit::No => "no",
            RemainAfterExit::Yes => "yes",
        })
        .unwrap();
        Ok(())
    }
}

fn default_remain_after_exit() -> Option<RemainAfterExit> {
    Some(RemainAfterExit::No)
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
enum ServiceType {
    Simple,
    OneShot,
    Forking,
    Notify,
    DBus,
    Idle,
}

impl core::fmt::Display for ServiceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(
            self.serialize(serde_yaml::value::Serializer)
                .unwrap()
                .as_str()
                .unwrap(),
        )
        .unwrap();
        Ok(())
    }
}

fn default_service_type() -> Option<ServiceType> {
    None
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
struct Service {
    pub environment_file: Option<String>,
    pub exec_start_pre: Option<String>,
    pub exec_start: Option<String>,
    pub exec_stop: Option<String>,
    pub group: Option<String>,
    #[serde(default = "default_remain_after_exit")]
    pub remain_after_exit: Option<RemainAfterExit>,
    pub restart: Option<Restart>,
    pub timeout_start_sec: Option<u32>,
    #[serde(default = "default_service_type", rename = "Type")]
    pub service_type: Option<ServiceType>,
    pub user: Option<String>,
    pub working_directory: Option<String>,
}

fn default_wanted_by() -> String {
    "default.target".into()
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Install {
    #[serde(default = "default_wanted_by")]
    pub wanted_by: String,
}

pub fn default_install() -> Install {
    Install {
        wanted_by: default_wanted_by(),
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
struct TemplateServiceDef {
    pub unit: TemplateUnit,
    pub service: Service,
    #[serde(default = "default_install")]
    pub install: Install,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
struct InstanceServiceDef {
    pub unit: InstanceUnit,
    pub service: Option<Service>,
    pub install: Option<Install>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
struct TemplatesAndInstances {
    pub template: TemplateServiceDef,
    pub instances: Vec<InstanceServiceDef>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
struct DefinitionFile {
    pub defs: Vec<TemplatesAndInstances>,
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(value_name = "FILE")]
    definitions_file: PathBuf,
    #[arg(value_name = "OUTPUT_DIRECTORY")]
    out_dst: PathBuf,
}

fn resolve_service_section(
    instance_service: Option<Service>,
    template_service: Service,
    mut memo: String,
) -> String {
    let mut environment_file = template_service.environment_file;
    let mut exec_start_pre = template_service.exec_start_pre;
    let mut exec_start = template_service.exec_start;
    let mut exec_stop = template_service.exec_stop;
    let mut group = template_service.group;
    let mut remain_after_exit = template_service.remain_after_exit;
    let mut restart = template_service.restart;
    let mut service_type = template_service.service_type;
    let mut timeout_start_sec = template_service.timeout_start_sec;
    let mut user = template_service.user;
    let mut working_directory = template_service.working_directory;

    if let Some(i) = instance_service {
        if i.environment_file.is_some() {
            environment_file = i.environment_file;
        }
        if i.exec_start_pre.is_some() {
            exec_start_pre = i.exec_start_pre;
        }
        if i.exec_start.is_some() {
            exec_start = i.exec_start;
        }
        if i.exec_stop.is_some() {
            exec_stop = i.exec_stop;
        }
        if i.group.is_some() {
            group = i.group;
        }
        if i.remain_after_exit.is_some() {
            remain_after_exit = i.remain_after_exit;
        }
        if i.restart.is_some() {
            restart = i.restart;
        }
        if i.service_type.is_some() {
            service_type = i.service_type;
        }
        if i.timeout_start_sec.is_some() {
            timeout_start_sec = i.timeout_start_sec;
        }
        if i.user.is_some() {
            user = i.user;
        }
        if i.working_directory.is_some() {
            working_directory = i.working_directory;
        }
    }

    memo += "\n[Service]\n";

    if let Some(v) = environment_file {
        memo += &format!("EnvironmentFile={}\n", v);
    }
    if let Some(v) = exec_start_pre {
        memo += &format!("ExecStartPre={}\n", v);
    }
    if let Some(v) = exec_start {
        memo += &format!("ExecStart={}\n", v);
    }
    if let Some(v) = exec_stop {
        memo += &format!("ExecStop={}\n", v);
    }
    if let Some(v) = group {
        memo += &format!("Group={}\n", v);
    }
    if let Some(v) = remain_after_exit {
        memo += &format!("RemainAfterExit={}\n", v);
    }
    if let Some(v) = restart {
        memo += &format!("Restart={}\n", v);
    }
    if let Some(v) = timeout_start_sec {
        memo += &format!("TimeoutStartSec={}\n", v);
    }
    if let Some(v) = service_type {
        memo += &format!("Type={}\n", v);
    }
    if let Some(v) = user {
        memo += &format!("User={}\n", v);
    }
    if let Some(v) = working_directory {
        memo += &format!("WorkingDirectory={}\n", v);
    }

    memo
}

fn resolve(instance: InstanceServiceDef, template: TemplateServiceDef) -> String {
    let mut memo = String::from("; THIS FILE IS GENERATED BY gen-systemd-svc\n");
    memo += "; DO NOT EDIT THIS FILE DIRECTLY!\n";
    memo += "\n[Unit]\n";
    memo += &format!("Description={}\n", instance.unit.description);

    let requires: Vec<String> = match instance.unit.inherit_requires {
        true => {
            let mut v = template.unit.requires.clone();
            v.extend(instance.unit.requires.unwrap_or_default());
            v
        }
        false => instance.unit.requires.unwrap_or_default(),
    };

    for req in requires {
        memo += &format!("Requires={}\n", req);
    }

    let afters: Vec<String> = match instance.unit.inherit_after {
        true => {
            let mut v = template.unit.after.clone();
            v.extend(instance.unit.after.unwrap_or_default());
            v
        }
        false => instance.unit.after.unwrap_or_default(),
    };

    for after in afters {
        memo += &format!("After={}\n", after);
    }

    let wants: Vec<String> = match instance.unit.inherit_wants {
        true => {
            let mut v = template.unit.wants.clone();
            v.extend(instance.unit.wants.unwrap_or_default());
            v
        }
        false => instance.unit.wants.unwrap_or_default(),
    };

    for want in wants {
        memo += &format!("Wants={}\n", want);
    }

    if let Some(v) = instance.unit.requires_mounts_for {
        memo += &format!("RequiresMountsFor={}\n", v.join(" "));
    }

    // SERVICE PART
    let mut memo = resolve_service_section(instance.service, template.service, memo);

    // INSTALL PART
    memo += "\n[Install]\n";
    memo += &format!(
        "WantedBy={}\n",
        instance.install.unwrap_or(template.install).wanted_by
    );
    memo
}

fn main() {
    let cli = Cli::parse();
    let file = File::open(cli.definitions_file.as_path()).unwrap();
    let reader = BufReader::new(file);
    let def_file: DefinitionFile = serde_yaml::from_reader(reader).unwrap();

    for def in def_file.defs {
        for instance in def.instances {
            let name = instance.unit.name.clone();
            println!("Generating definition for {}", name);
            let filename = format!("{}.service", name);
            let resolved = resolve(instance, def.template.clone());
            let dst = cli.out_dst.join(filename);
            println!("Writing {:?}", dst);
            fs::write(dst, resolved).expect("Unable to write file")
        }
    }
}
