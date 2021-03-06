use crate::build::*;
use crate::channel::*;
use crate::err::SdResult;

use chrono::Local;
use std::env;
use std::process::Command;

use std::collections::HashMap;

#[derive(Default, Debug)]
pub struct SystemEnv {
    map: HashMap<ShadowConst, ConstVal>,
}

const BUILD_OS: ShadowConst = "BUILD_OS";
const RUST_VERSION: ShadowConst = "RUST_VERSION";
const RUST_CHANNEL: ShadowConst = "RUST_CHANNEL";
const CARGO_VERSION: ShadowConst = "CARGO_VERSION";
const CARGO_TREE: ShadowConst = "CARGO_TREE";
// const CARGO_METADATA: ShadowConst = "CARGO_METADATA";
const PKG_VERSION: ShadowConst = "PKG_VERSION";

impl SystemEnv {
    fn init(&mut self, std_env: &HashMap<String, String>) -> SdResult<()> {
        let mut update_val = |c: ShadowConst, v: String| {
            if let Some(mut val) = self.map.get_mut(c) {
                val.t = ConstType::Str;
                val.v = v;
            }
        };
        if let Ok(out) = Command::new("rustup").arg("default").output() {
            update_val(
                RUST_CHANNEL,
                String::from_utf8(out.stdout)?.trim().to_string(),
            );
        }

        if let Ok(out) = Command::new("rustc").arg("-V").output() {
            update_val(
                RUST_VERSION,
                String::from_utf8(out.stdout)?.trim().to_string(),
            );
        }

        if let Ok(out) = Command::new("cargo").arg("-V").output() {
            update_val(
                CARGO_VERSION,
                String::from_utf8(out.stdout)?.trim().to_string(),
            );
        }

        if let Ok(out) = Command::new("cargo").arg("tree").output() {
            let input = String::from_utf8(out.stdout)?;
            if let Some(index) = input.find('\n') {
                let lines = filter_private_registry(input[index..].split('\n').collect());
                update_val(CARGO_TREE, lines);
            }
        }

        if let Ok(_out) = Command::new("cargo")
            .args(&["metadata", "--format-version", "1"])
            .output()
        {
            //TODO completed

            // update_val(
            //     CARGO_METADATA,
            //     String::from_utf8(out.stdout)?.trim().to_string(),
            // );
        }

        if let Some(v) = std_env.get("CARGO_PKG_VERSION") {
            update_val(PKG_VERSION, v.to_string());
        }

        Ok(())
    }
}

fn filter_private_registry(lines: Vec<&str>) -> String {
    let mut tree = "\n".to_string();
    for line in lines {
        let val = if let Some(index) = line.find("(registry `") {
            format!("{}(private)", line[..index].to_string())
        } else {
            line.to_string()
        };
        if tree.trim().is_empty() {
            tree.push_str(&val);
        } else {
            tree = format!("{}\n{}", tree, val);
        }
    }
    tree
}

pub fn new_system_env(std_env: &HashMap<String, String>) -> HashMap<ShadowConst, ConstVal> {
    let mut env = SystemEnv::default();
    env.map.insert(
        BUILD_OS,
        ConstVal {
            desc: "display build system os".to_string(),
            v: format!("{}-{}", env::consts::OS, env::consts::ARCH),
            t: ConstType::Str,
        },
    );

    env.map.insert(
        RUST_CHANNEL,
        ConstVal::new("display build system rust channel"),
    );
    env.map.insert(
        RUST_VERSION,
        ConstVal::new("display build system rust version"),
    );
    env.map.insert(
        CARGO_VERSION,
        ConstVal::new("display build system cargo version"),
    );

    env.map.insert(
        CARGO_TREE,
        ConstVal::new("display build cargo dependencies.It's used by rust version 1.44.0"),
    );

    // env.map.insert(
    //     CARGO_METADATA,
    //     ConstVal::new("display build cargo dependencies by metadata.It's use by exec command `cargo metadata`"),
    // );

    env.map.insert(
        PKG_VERSION,
        ConstVal::new("display build current project version"),
    );

    if let Err(e) = env.init(std_env) {
        println!("{}", e.to_string());
    }
    env.map
}

#[derive(Default, Debug)]
pub struct Project {
    map: HashMap<ShadowConst, ConstVal>,
}

const PROJECT_NAME: ShadowConst = "PROJECT_NAME";
const BUILD_TIME: ShadowConst = "BUILD_TIME";
const BUILD_RUST_CHANNEL: ShadowConst = "BUILD_RUST_CHANNEL";

pub fn new_project(std_env: &HashMap<String, String>) -> HashMap<ShadowConst, ConstVal> {
    let mut project = Project::default();
    project.map.insert(
        BUILD_TIME,
        ConstVal {
            desc: "display project build time".to_string(),
            v: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            t: ConstType::Str,
        },
    );
    project.map.insert(
        BUILD_RUST_CHANNEL,
        ConstVal {
            desc: "display project build by rust channel [debug or release]".to_string(),
            v: build_channel().to_string(),
            t: ConstType::Str,
        },
    );
    project
        .map
        .insert(PROJECT_NAME, ConstVal::new("display project name"));

    if let (Some(v), Some(mut val)) = (
        std_env.get("CARGO_PKG_NAME"),
        project.map.get_mut(PROJECT_NAME),
    ) {
        val.t = ConstType::Str;
        val.v = v.to_string();
    }

    project.map
}
