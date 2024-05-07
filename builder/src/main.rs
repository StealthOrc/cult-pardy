use std::path::Path;
use std::env;
use std::env::current_dir;
use std::path::PathBuf;


use anyhow::Result;
use fs_extra::dir::{copy, CopyOptions, create_all, remove};


use tokio::process::Command;
#[tokio::main]
async fn main() -> Result<()> {

    let args: Vec<String> = env::args().collect();
    println!("{:?}", args);

    if args.contains(&"--build".to_string()) {
        buildApp(&args).await?;
        build_Server(&args).await?;
        if args.contains(&"--run".to_string()) {
            start_server().await?;
        }
    } else if args.contains(&"--run".to_string()) {
        buildApp(&args).await?;
        run_server(&args).await?;

    } else {
        buildApp(&args).await?;
        build_Server(&args).await?;
    }
    println!("{:?}", args);
    Ok(())
}


async fn start_server() -> anyhow::Result<()>{
    println!("Starting server!");
    let mut current_dir = env::current_dir()?;
    current_dir = PathBuf::from(current_dir);
    current_dir.push("target");
    current_dir.push("release");
    println!("{:?}", current_dir.clone());
    env::set_current_dir(current_dir.clone())?;
    let mut shell = Command::new("powershell")
        .arg("./server.exe")
        .current_dir(current_dir)
        .spawn()?;
    shell.wait().await?;
    println!("Server started!");
    Ok(())
}



async fn build_Server(args: &Vec<String>) -> anyhow::Result<()>{
    println!("Building server!");
    let binding = env::current_dir()?;
    let parent_dir = binding.parent().expect("?");
    env::set_current_dir(parent_dir)?;

    let current_dir = env::current_dir()?;

    if args.contains(&"--fix".to_string()) {
        cargo_fix(&args).await.expect("TODO: panic message");
    }


    let mut shell = Command::new("powershell")
        .arg("cargo")
        .arg("build")
        .arg("--bin")//
        .arg("server")//
        .arg("--release")//
        .current_dir(current_dir)
        .spawn()?;
    shell.wait().await?;
    println!("Server build!");
    Ok(())
}


async fn cargo_fix(_args: &Vec<String>) -> anyhow::Result<()>{
    let current_dir = env::current_dir()?;
    let mut shell = Command::new("powershell")
        .arg("cargo")
        .arg("fix")
        .arg("--allow-dirty")//
        .arg("--allow-staged")//
        .current_dir(current_dir)
        .spawn()?;
    shell.wait().await?;
    println!("Cargo fix!");
    Ok(())
}


async fn run_server(args: &Vec<String>) -> anyhow::Result<()>{
    println!("Run server!");
    let binding = env::current_dir()?;
    let parent_dir = binding.parent().expect("?");
    env::set_current_dir(parent_dir)?;

    let current_dir = env::current_dir()?;

    if args.contains(&"--fix".to_string()) {
        cargo_fix(&args).await.expect("TODO: panic message");
    }

    let mut shell = Command::new("powershell")
        .arg("cargo")
        .arg("run")
        .arg("--bin")
        .arg("server")//
        .arg("--release")//
        .current_dir(current_dir)
        .spawn()?;
    println!("Server running!");
    shell.wait().await?;
    Ok(())
}


async fn buildApp(args: &Vec<String>) -> anyhow::Result<()>{
    if let Ok(current_dir) = env::current_dir() {
        if let Some(file_name) = current_dir.file_name() {
            if let Some(dir_name) = file_name.to_str() {
                match dir_name {
                    "cult-pardy" => {},
                    "release" => {
                        let parent_dir = current_dir.parent().expect("?");
                        env::set_current_dir(parent_dir.parent().expect("?"))?;
                    },
                     "debug" => {
                        let parent_dir = current_dir.parent().expect("?");
                        env::set_current_dir(parent_dir.parent().expect("?"))?;
                    }
                    _ => {}
                }
            }
        }
    }



    let mut dir = current_dir()?;
    dir.push("app");
    env::set_current_dir(dir)?;



    let current_dir = env::current_dir()?;



    if args.contains(&"--fix".to_string()) {
        cargo_fix(&args).await.expect("TODO: panic message");
    }


    let mut shell = Command::new("powershell")
        .arg("trunk")
        .arg("build")
        .arg("--release")//
        .current_dir(current_dir)
        .spawn()?;
    shell.wait().await?;

    let source_dir = "./dist/";
    let target_dir_release = "../target/release/www";
    let target_dir_debug = "../target/debug/www";

    if Path::new(target_dir_release).exists() {
        remove(target_dir_release)?;
        println!("Removed old data from: {}", target_dir_release);
    }

    create_directory_if_not_exists(target_dir_release).expect("TODO: panic message");

    // Copy the contents of the source directory to the target directory
    copy(source_dir, target_dir_release, &CopyOptions {
        overwrite: true,
        skip_exist: false,
        buffer_size: 0,
        copy_inside: false,
        content_only: true,
        depth: 0,
    }).expect("TODO: panic message");



    if Path::new(target_dir_debug).exists() {
        remove(target_dir_debug)?;
        println!("Removed old data from: {}", target_dir_debug);
    }

    create_directory_if_not_exists(target_dir_debug).expect("TODO: panic message");

    // Copy the contents of the source directory to the target directory
    copy(source_dir, target_dir_debug, &CopyOptions {
        overwrite: true,
        skip_exist: false,
        buffer_size: 0,
        copy_inside: false,
        content_only: true,
        depth: 0,
    }).expect("TODO: panic message");

    println!("Folder copied successfully!");
    println!("Shell opened successfully!");
    println!("App build!");
    Ok(())
}


fn create_directory_if_not_exists(dir_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    if !Path::new(dir_path).exists() {
        create_all(dir_path, true)?;
        println!("Created directory: {}", dir_path);
    }
    Ok(())
}

