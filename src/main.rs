#![windows_subsystem = "windows"]

#[allow(unused)]
mod process_monitor;

use crate::process_monitor::ProcessInfo;
use std::collections::HashMap;
use std::process::Command;

use anyhow::Context;
use process_monitor::ProcessMonitor;

use futures::StreamExt;
use sysinfo::{ProcessRefreshKind, RefreshKind, System};
use tokio::sync::watch;

const CHECK_PROCESS_NAMES: &[&str] = &["blacklist_01.exe"];
const SPAWN_PROCESS_PATH: &str = "./CapsLockCtrlEscape_ahkv2.exe";

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let (toggle_tx, toggle_rx) = watch::channel(true);
    //start process toggle task
    tokio::spawn(async move {
        toggle_process_task(SPAWN_PROCESS_PATH, toggle_rx)
            .await
            .unwrap();
    });
    //start watching for target process
    process_moniter_task(CHECK_PROCESS_NAMES, toggle_tx).await
}

async fn toggle_process_task(
    spawn_command: &str,
    mut toggle: watch::Receiver<bool>,
) -> anyhow::Result<()> {
    let mut child = None;

    loop {
        if *toggle.borrow() {
            if child.is_none() {
                let process = Command::new(spawn_command)
                    .spawn()
                    .with_context(|| format!("unable to spawn process: {spawn_command}"))?;
                child = Some(process);
            }
        } else if let Some(mut child) = child.take() {
            child.kill().context("unable to kill child process")?;
        }

        //wait until toggle value changes
        if toggle.changed().await.is_err() {
            break;
        }
    }
    Ok(())
}

async fn process_moniter_task(
    process_list: &[&str],
    toggle: watch::Sender<bool>,
) -> anyhow::Result<()> {
    let mut process_status = get_initial_process_status_map(process_list)?;

    let process_monitor = ProcessMonitor::new()?;
    let mut spawn_event = process_monitor.async_listen_process_spawn()?;
    let mut death_event = process_monitor.async_listen_process_death()?;

    let mut process_event = |info: Option<anyhow::Result<ProcessInfo>>, status: bool| {
        if let Some(Ok(process_info)) = info {
            if let Some(s) = process_status.get_mut(&*process_info.name.to_lowercase()) {
                //update process running status
                *s = status;
                //check if any process in blacklist are running
                let is_any_running = process_status.values().any(|s| *s);
                //set process toggle to false if any of process in blacklist are running
                let _ = toggle.send(!is_any_running);
            }
        }
    };

    loop {
        tokio::select! {
            info = spawn_event.next() => {
                process_event(info, true);
            },
            info = death_event.next() => {
                process_event(info, false);
            }
        }
    }
}

fn system_with_process_status() -> System {
    System::new_with_specifics(RefreshKind::new().with_processes(ProcessRefreshKind::new()))
}

fn is_process_running(system: &System, process_name: &str) -> bool {
    system
        .processes_by_exact_name(process_name)
        .next()
        .is_some()
}

fn get_initial_process_status_map(process_list: &[&str]) -> anyhow::Result<HashMap<String, bool>> {
    let s = system_with_process_status();
    let process_status = HashMap::from_iter(
        process_list
            .iter()
            .map(|&name| (name.to_lowercase(), is_process_running(&s, name))),
    );
    Ok(process_status)
}
