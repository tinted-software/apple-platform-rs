use clap::Parser;
use serde::Deserialize;
use std::path::PathBuf;
use std::process::Command;

#[derive(Parser, Debug)]
#[clap(name = "xcrun")]
// xcrun <options> <tool> <tool_arguments>
pub struct Xcrun {
    #[clap(long)]
    version: bool,
    #[clap(short, long)]
    verbose: bool,
    #[clap(long)]
    sdk: Option<String>,
    #[clap(long)]
    toolchain: Option<String>,
    #[clap(short, long)]
    log: bool,
    #[clap(short, long)]
    find: Option<String>,
    #[clap(short, long)]
    no_cache: bool,
    #[clap(short, long)]
    kill_cache: bool,
    #[clap(long)]
    show_sdk_path: bool,
    #[clap(long)]
    show_sdk_version: bool,
    #[clap(short, long)]
    show_sdk_target_triple: bool,
    #[clap(long)]
    show_sdk_toolchain_path: bool,
    #[clap(long)]
    show_sdk_toolchain_version: bool,

    #[clap(long)]
    run: Option<String>,

    arguments: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
/// Contains a list of SDKs that xcrun can use.
pub struct XcrunConfiguration {
    #[serde(rename = "sdk")]
    pub sdks: Vec<Sdk>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Sdk {
    /// The name of the SDK.
    pub name: String,
    /// The path to the SDK.
    pub path: String,
    /// The version of the SDK.
    pub version: String,
    /// The target triple of the SDK.
    pub target_triple: String,
    /// The minimum deployment target of the SDK on macOS.
    pub macosx_deployment_target: String,
    /// The minimum deployment target of the SDK on iOS.
    pub ios_deployment_target: String,
}

fn main() {
    let xdg_config_home = std::env::var("XDG_CONFIG_HOME").unwrap_or_else(|_| {
        let home = std::env::var("HOME").unwrap();
        format!("{}/.config", home)
    });
    let configuration_path = PathBuf::from(xdg_config_home).join("xcrun/config.toml");

    if !configuration_path.exists() {
        eprintln!(
            "SDK configuration file not found at {:?}",
            configuration_path
        );
        std::process::exit(1);
    }

    let configuration: XcrunConfiguration =
        toml::from_str(&std::fs::read_to_string(configuration_path).unwrap()).unwrap();
    let xcrun = Xcrun::parse();

    if xcrun.version {
        println!("xcrun 1.0.0");
    }

    if xcrun.show_sdk_path {
        for sdk in &configuration.sdks {
            println!("{}", sdk.path);
        }
    }

    if xcrun.show_sdk_version {
        for sdk in &configuration.sdks {
            println!("{}", sdk.version);
        }
    }

    if xcrun.show_sdk_target_triple {
        for sdk in &configuration.sdks {
            println!("{}", sdk.target_triple);
        }
    }

    if xcrun.show_sdk_toolchain_path {
        for sdk in &configuration.sdks {
            println!("{}", sdk.path);
        }
    }

    if xcrun.show_sdk_toolchain_version {
        for sdk in &configuration.sdks {
            println!("{}", sdk.version);
        }
    }

    if let Some(tool) = &xcrun.find {
        let sdk = configuration
            .sdks
            .iter()
            .find(|sdk| {
                if let Some(sdk_name) = &xcrun.sdk {
                    sdk_name == &sdk.name
                } else {
                    PathBuf::from(&sdk.path).join("bin").join(&tool).exists()
                        || PathBuf::from(&sdk.path)
                            .join("usr/bin")
                            .join(&tool)
                            .exists()
                }
            })
            .unwrap_or_else(|| {
                eprintln!("xcrun: error: tool not found: {}", tool);
                std::process::exit(1);
            });

        // Locate the tool in the SDK.
        let tool_path = find_tool(&sdk, tool).unwrap_or_else(|| {
            eprintln!("xcrun: error: tool not found: {}", tool);
            std::process::exit(1);
        });

        println!("{}", tool_path.display());
    }

    if !xcrun.arguments.is_empty() {
        let tool = xcrun.arguments[0].clone();
        let sdk = configuration
            .sdks
            .iter()
            .find(|sdk| {
                if let Some(sdk_name) = &xcrun.sdk {
                    sdk_name == &sdk.name
                } else {
                    PathBuf::from(&sdk.path).join("bin").join(&tool).exists()
                        || PathBuf::from(&sdk.path)
                            .join("usr/bin")
                            .join(&tool)
                            .exists()
                }
            })
            .unwrap();
        let mut command = Command::new(find_tool(&sdk, &tool).unwrap());
        command.env("SDKROOT", sdk.path.clone());
        command.env(
            "PATH",
            format!("{}:{}", sdk.path, std::env::var("PATH").unwrap()),
        );
        command.env(
            "LD_LIBRARY_PATH",
            format!(
                "{}/lib:{}",
                sdk.path,
                std::env::var("LD_LIBRARY_PATH").unwrap_or_default()
            ),
        );
        command.args(&xcrun.arguments[1..]);
        command.stdin(std::process::Stdio::inherit());
        command.stderr(std::process::Stdio::inherit());
        command.stdout(std::process::Stdio::inherit());

        if xcrun.log {
            println!(
                "xcrun: info: invoking command: \n\t\"{}\"",
                xcrun.arguments[0..].join(" ")
            );
        }

        let status = command.status().unwrap();
        std::process::exit(status.code().unwrap());
    }

    if xcrun.verbose {
        println!("{:?}", xcrun);
    }
}

fn find_tool(sdk: &Sdk, tool: &str) -> Option<PathBuf> {
    let tool_path = PathBuf::from(&sdk.path).join("bin").join(tool);
    if tool_path.exists() {
        return Some(tool_path);
    }

    let tool_path = PathBuf::from(&sdk.path).join("usr/bin").join(tool);
    if tool_path.exists() {
        return Some(tool_path);
    }

    None
}
