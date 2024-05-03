use std::path::Path;
use std::env;
use std::env::current_dir;
use std::path::PathBuf;
use std::process::Stdio;
use cult_common::*;
use anyhow::Result;
use fs_extra::dir::{copy, CopyOptions, create_all, remove};
use tokio::{fs, io};
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

#[tokio::main]
async fn main() -> Result<()> {


    if let Ok(current_dir) = env::current_dir() {
        if let Some(file_name) = current_dir.file_name() {
            if let Some(dir_name) = file_name.to_str() {
                match dir_name {
                    "cult-pardy" => {},
                    "release" => {
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

    let binding = env::current_dir()?;
    let parent_dir = binding.parent().expect("?");
    env::set_current_dir(parent_dir)?;

    let current_dir = env::current_dir()?;
    let mut shell = Command::new("powershell")
        .arg("cargo")
        .arg("build")
        .arg("--bin")//
        .arg("server")//
        .arg("--release")//
        .current_dir(current_dir)
        .spawn()?;
    shell.wait().await?;



    Ok(())
}

fn create_directory_if_not_exists(dir_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    if !Path::new(dir_path).exists() {
        create_all(dir_path, true)?;
        println!("Created directory: {}", dir_path);
    }
    Ok(())
}

