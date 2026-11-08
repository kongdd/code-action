//! Code Action 命令行版：与桌面版共用 core 逻辑，配置也取自同一份 config.json。

use code_action_core::{
    actions::{self, Action, Mode},
    history,
    store::{config_path, Storage, Store},
};
use std::process::ExitCode;

const USAGE: &str = "\
Code Action 命令行版

用法：
  ca list [关键词…]                     列出 VSCode 最近打开的项目
  ca find <表达式>                      用 fs 检索本地文件
  ca open <路径> [--vscode|--zed|--dir|--term]
                                        打开路径，默认用系统默认程序
  ca rm <路径>                          从 VSCode 历史中移除
  以 - 开头的路径请放在 -- 之后。
";

#[tokio::main]
async fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match run(&args).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

async fn run(args: &[String]) -> Result<(), String> {
    let Some(command) = args.first() else {
        print!("{USAGE}");
        return Ok(());
    };
    let rest = &args[1..];
    match command.as_str() {
        "list" => list(rest).await,
        "find" => find(rest).await,
        "open" => open(rest).await,
        "rm" => remove(rest),
        "help" | "-h" | "--help" if rest.is_empty() => {
            print!("{USAGE}");
            Ok(())
        }
        "help" | "-h" | "--help" => Err("help 不接受参数".into()),
        other => Err(format!("未知命令：{other}\n\n{USAGE}")),
    }
}

/// 配置不存在时使用默认值；读取或解析失败时中止，避免操作错误的数据库。
fn load_store() -> Result<Store, String> {
    let path = config_path().ok_or("无法定位配置目录")?;
    load_store_at(path)
}

fn load_store_at(path: std::path::PathBuf) -> Result<Store, String> {
    Storage::load(path.clone()).map(|storage| storage.data)
        .map_err(|error| format!("无法读取配置 {}：{error}", path.display()))
}

async fn list(args: &[String]) -> Result<(), String> {
    let store = load_store()?;
    let items = code_action_core::search(&store, Mode::Vscode, &args.join(" "), None).await?;
    for item in items {
        println!("{}", item.path);
    }
    Ok(())
}

async fn find(args: &[String]) -> Result<(), String> {
    let query = args.join(" ");
    if query.is_empty() {
        return Err("请提供检索表达式".into());
    }
    let store = load_store()?;
    let items = code_action_core::search(&store, Mode::Fs, &query, None).await?;
    for item in items {
        println!("{}", item.path);
    }
    Ok(())
}

fn parse_open(args: &[String]) -> Result<(Action, &str), String> {
    let mut action = None;
    let mut target = None;
    let mut positional = false;
    for arg in args {
        if !positional && arg == "--" {
            positional = true;
            continue;
        }
        let option = if positional { None } else {
            match arg.as_str() {
                "--vscode" => Some(Action::Vscode),
                "--zed" => Some(Action::Zed),
                "--dir" => Some(Action::Directory),
                "--term" => Some(Action::Terminal),
                _ if arg.starts_with('-') => return Err(format!("未知选项：{arg}")),
                _ => None,
            }
        };
        if let Some(option) = option {
            if action.replace(option).is_some() { return Err("只能指定一种打开方式".into()); }
        } else if target.replace(arg.as_str()).is_some() {
            return Err("只能指定一个路径".into());
        }
    }
    Ok((action.unwrap_or(Action::Default), target.ok_or("请提供要打开的路径")?))
}

async fn open(args: &[String]) -> Result<(), String> {
    let (action, target) = parse_open(args)?;
    let store = load_store()?;
    actions::open(&store.settings, action, target).await
}

fn parse_remove(args: &[String]) -> Result<&str, String> {
    match args {
        [target] if !target.starts_with('-') => Ok(target),
        [separator, target] if separator == "--" => Ok(target),
        _ => Err("请指定一个要移除的路径（以 - 开头的路径须放在 -- 之后）".into()),
    }
}

fn remove(args: &[String]) -> Result<(), String> {
    let target = parse_remove(args)?;
    let store = load_store()?;
    history::remove(&store.settings, target)
}

#[cfg(test)]
#[path = "../tests/cli.rs"]
mod tests;
