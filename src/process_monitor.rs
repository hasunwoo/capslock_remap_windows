use std::collections::HashMap;
use std::sync::OnceLock;
use std::time::Duration;

use futures::{Stream, StreamExt};
use serde::de::DeserializeOwned;
use serde::Deserialize;
use wmi::{COMLibrary, FilterValue, WMIConnection};

static DEFAULT_FILTER: OnceLock<HashMap<String, FilterValue>> = OnceLock::new();

fn default_filter() -> &'static HashMap<String, FilterValue> {
    DEFAULT_FILTER.get_or_init(|| {
        let mut filter = HashMap::<String, FilterValue>::new();
        filter.insert(
            "TargetInstance".to_owned(),
            FilterValue::is_a::<Process>().unwrap(),
        );
        filter
    })
}

#[derive(Clone, Debug)]
pub struct ProcessInfo {
    pub process_id: u32,
    pub name: String,
    pub executable_path: Option<String>,
}

impl From<Process> for ProcessInfo {
    fn from(p: Process) -> Self {
        ProcessInfo {
            process_id: p.process_id,
            name: p.name,
            executable_path: p.executable_path,
        }
    }
}

pub struct ProcessMonitor {
    wmi_con: WMIConnection,
}

impl ProcessMonitor {
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self {
            wmi_con: WMIConnection::new(COMLibrary::new()?)?,
        })
    }

    pub fn async_listen_process_spawn(
        &self,
    ) -> anyhow::Result<impl Stream<Item = anyhow::Result<ProcessInfo>>> {
        listen_process_event::<ProcessSpawnEvent>(&self.wmi_con)
    }

    pub fn async_listen_process_death(
        &self,
    ) -> anyhow::Result<impl Stream<Item = anyhow::Result<ProcessInfo>>> {
        listen_process_event::<ProcessDeathEvent>(&self.wmi_con)
    }
}

fn listen_process_event<E: DeserializeOwned + Into<ProcessInfo>>(
    wmi_con: &WMIConnection,
) -> anyhow::Result<impl Stream<Item = anyhow::Result<ProcessInfo>>> {
    let filter = default_filter();
    let stream = wmi_con.async_filtered_notification::<E>(filter, Some(Duration::from_secs(1)))?;
    let stream = stream.map(|e| Ok(e?.into()));
    Ok(stream)
}

#[derive(Deserialize, Debug)]
#[serde(rename = "__InstanceCreationEvent")]
#[serde(rename_all = "PascalCase")]
struct ProcessSpawnEvent {
    target_instance: Process,
}

impl From<ProcessSpawnEvent> for ProcessInfo {
    fn from(e: ProcessSpawnEvent) -> Self {
        e.target_instance.into()
    }
}

#[derive(Deserialize, Debug)]
#[serde(rename = "__InstanceDeletionEvent")]
#[serde(rename_all = "PascalCase")]
struct ProcessDeathEvent {
    target_instance: Process,
}

impl From<ProcessDeathEvent> for ProcessInfo {
    fn from(e: ProcessDeathEvent) -> Self {
        e.target_instance.into()
    }
}

#[derive(Deserialize, Debug)]
#[serde(rename = "Win32_Process")]
#[serde(rename_all = "PascalCase")]
struct Process {
    process_id: u32,
    name: String,
    executable_path: Option<String>,
}
