use std::{collections::HashMap, fs::create_dir_all, process::Stdio};

use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::{Child, Command},
};
use uuid::Uuid;

use crate::{
    auth::AuthMethod,
    error::Error,
    json::version::meta::vanilla::{Arguments, Element, Value, VersionMeta},
    minecraft::{config::Memory, emitter::Event, parse::ParseRule},
    util::json::read_json,
};

use super::{config::Config, CLASSPATH_SEPARATOR};
use super::{emitter::Emitter, loader::Loader};

/// Launches the Minecraft game with the specified configuration and arguments.
///
/// # Parameters
/// - `config`: The configuration for the Minecraft launch.
/// - `emitter`: An optional emitter for logging progress.
///
/// # Returns
/// A result containing the child process that was spawned to run the game.
pub async fn launch<T: Loader>(
    config: &Config<T>,
    emitter: Option<&Emitter>,
) -> crate::Result<Child> {
    let version_name = config.get_version_name();
    let mut arguments = Vec::<String>::with_capacity(100);
    let meta: VersionMeta = read_json(&config.get_version_json_path()).await?;
    let profile_dir = config.profile.clone().map(|p| p.root.join(p.name));
    let current_dir = profile_dir.as_ref().unwrap_or(&config.game_dir);

    let meta_arguments = meta.arguments.unwrap_or_else(|| Arguments {
        game: meta
            .minecraft_arguments
            .unwrap_or_default()
            .split_whitespace()
            .map(|argument| Element::String(argument.to_string()))
            .collect(),
        jvm: vec![
            Element::String("-Djava.library.path=${natives_directory}".to_string()),
            Element::String("-cp".to_string()),
            Element::String("${classpath}".to_string()),
        ],
    });

    let mut variables = HashMap::<&'static str, String>::with_capacity(20);

    let mut insert_var = |key: &'static str, value: String| {
        variables.insert(key, value);
    };

    // Authentication variables
    match &config.authentication {
        AuthMethod::Microsoft {
            username,
            xuid,
            uuid,
            access_token,
            ..
        } => {
            insert_var("${auth_player_name}", username.clone());
            insert_var("${auth_xuid}", xuid.clone());
            insert_var("${auth_uuid}", uuid.clone());
            insert_var("${auth_access_token}", access_token.clone());
            insert_var("${user_type}", "msa".to_string());
        }
        AuthMethod::Offline { username, uuid } => {
            let uuid = uuid.clone().unwrap_or(Uuid::new_v4().to_string());
            insert_var("${auth_player_name}", username.to_string());
            insert_var("${auth_xuid}", uuid.clone());
            insert_var("${auth_uuid}", uuid);
            insert_var("${auth_access_token}", "token".to_string());
            insert_var("${user_type}", "mojang".to_string());
        }
    }
    // Using original Minecraft launcher's client id for authentication.
    insert_var("${clientid}", "00000000402b5328".to_string());
    insert_var("${user_properties}", "".to_string());

    // Launcher variables
    insert_var("${launcher_name}", env!("CARGO_PKG_NAME").to_string());
    insert_var("${launcher_version}", env!("CARGO_PKG_VERSION").to_string());

    // Game configuration variables
    insert_var("${version_name}", version_name.clone());
    insert_var(
        "${game_directory}",
        current_dir.to_string_lossy().into_owned(),
    );

    let assets_dir = config.get_assets_path();

    insert_var("${assets_root}", assets_dir.to_string_lossy().into_owned());
    insert_var(
        "${game_assets}",
        assets_dir
            .join("virtual")
            .join("legacy")
            .to_string_lossy()
            .into_owned(),
    );
    insert_var("${assets_index_name}", meta.asset_index.id);
    insert_var("${version_type}", meta.r#type);
    insert_var(
        "${natives_directory}",
        config
            .get_natives_path()
            .join(&config.version)
            .to_string_lossy()
            .into_owned(),
    );

    let libraries_path = config.get_libraries_path();
    insert_var("${classpath}", {
        let mut cp: Vec<String> = meta
            .libraries
            .iter()
            .filter_map(|lib| {
                if lib.skip_args {
                    return None;
                }

                lib.downloads.as_ref().and_then(|downloads| {
                    downloads.artifact.as_ref().and_then(|artifact| {
                        artifact.path.as_ref().and_then(|path| {
                            if lib.rules.parse_rule() && lib.natives.is_none() {
                                Some(libraries_path.join(path).to_string_lossy().into_owned())
                            } else {
                                None
                            }
                        })
                    })
                })
            })
            .collect();

        cp.push(config.get_version_jar_path().to_string_lossy().into_owned());

        cp.join(CLASSPATH_SEPARATOR)
    });

    fn replace_each(variables: &HashMap<&'static str, String>, arg: String) -> String {
        variables.iter().fold(arg, |arg, (k, v)| arg.replace(*k, v))
    }

    // Forge JVM variables
    insert_var(
        "${library_directory}",
        libraries_path.to_string_lossy().into_owned(),
    );
    insert_var("${classpath_separator}", CLASSPATH_SEPARATOR.to_string());

    match &config.memory {
        Some(memory) => arguments.push(format!(
            "-Xmx{}",
            match memory {
                Memory::Gigabyte(m) => format!("{}G", m),
                Memory::Megabyte(m) => format!("{}M", m),
            }
        )),
        None => arguments.push("-Xmx2G".to_string()),
    }

    meta_arguments.jvm.iter().for_each(|arg| match arg {
        Element::String(e) => arguments.push(replace_each(&variables, e.clone())),
        Element::Class(e) => {
            if e.rules.parse_rule() {
                match &e.value {
                    Value::Single(e) => arguments.push(replace_each(&variables, e.clone())),
                    Value::Multiple(e) => {
                        e.iter()
                            .for_each(|v| arguments.push(replace_each(&variables, v.clone())));
                    }
                }
            }
        }
    });

    config.custom_java_args.iter().for_each(|arg| {
        arguments.push(replace_each(&variables, arg.clone()));
    });

    arguments.push(meta.main_class.to_owned());

    meta_arguments.game.iter().for_each(|arg| {
        if let Element::String(e) = arg {
            arguments.push(replace_each(&variables, e.clone()))
        }
    });

    config.custom_args.iter().for_each(|arg| {
        arguments.push(replace_each(&variables, arg.clone()));
    });

    let java_path = config
        .get_java_path(&meta.java_version.unwrap_or_default())
        .await?;

    create_dir_all(current_dir)?;

    let mut child = Command::new(java_path)
        .args(arguments)
        .stdout(Stdio::piped())
        .current_dir(current_dir)
        .spawn()?;

    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| Error::Take("Child -> stdout".to_string()))?;

    if let Some(emitter) = emitter {
        let emitter = emitter.clone();
        tokio::spawn(async move {
            let mut reader = BufReader::new(stdout).lines();
            while let Some(line) = reader.next_line().await.unwrap() {
                emitter.emit(Event::Console, line).await;
            }
        });
    }

    Ok(child)
}
