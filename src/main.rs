use std::{
    collections::HashMap,
    fs,
    path::PathBuf,
    time::{Duration, Instant},
};

use async_channel::{unbounded, Receiver, Sender};
use async_process::{Command, Stdio};
use futures_lite::{io::BufReader, prelude::*};
use serde::Deserialize;
use smol::{future, io, Timer};
use smol_macros::main;

#[derive(Clone, Debug, Deserialize)]
struct Config {
    // additional arguments for swww
    swww_args: String,

    // fallback background image path
    default: String,

    // "tag" -> "background image path"
    tags: HashMap<String, String>,
}

impl Config {
    fn load() -> io::Result<Self> {
        let config_path = PathBuf::new()
            .join(dirs_next::config_dir().unwrap())
            .join("river-swww/config.json");

        serde_json::from_str(&fs::read_to_string(config_path)?)
            .map_err(|e| io::Error::new(std::io::ErrorKind::Other, e))
    }
}

#[derive(Clone, Debug, Deserialize)]
struct OutputInfo {
    name: String,
    focused_tags: u64,
}

impl OutputInfo {
    pub fn tag(&self) -> u64 {
        ((self.focused_tags as f64).log2() as u64) + 1
    }
}

async fn monitor_bedload(tx: Sender<OutputInfo>) -> io::Result<()> {
    let mut bedload = Command::new("river-bedload")
        .arg("-watch")
        .arg("outputs")
        .arg("-minified")
        .stdout(Stdio::piped())
        .spawn()?;

    let mut lines = BufReader::new(bedload.stdout.take().unwrap()).lines();
    while let Some(line) = lines.next().await {
        let outputs: Vec<OutputInfo> = serde_json::from_str(&line?)?;

        for output in outputs {
            tx.send(output).await.unwrap();
        }
    }

    Ok(())
}

struct SwwwBackend {
    config: Config,

    // map from "output name" -> pending update, used for debouncing
    pending_updates: HashMap<String, PendingUpdate>,
}

#[derive(Clone, Debug)]
struct PendingUpdate {
    // the proposed background path
    path: String,

    // when this update was requested
    queued_at: Instant,
}

impl PendingUpdate {
    fn new(path: &str) -> Self {
        Self {
            path: path.to_string(),
            queued_at: Instant::now(),
        }
    }

    fn wait_time(&self) -> Option<Duration> {
        SwwwBackend::DEBOUNCE_TIME.checked_sub(Instant::now().duration_since(self.queued_at))
    }
}

impl SwwwBackend {
    pub const DEBOUNCE_TIME: Duration = Duration::from_millis(30);

    fn new(config: &Config) -> Self {
        Self {
            config: config.clone(),
            pending_updates: HashMap::new(),
        }
    }

    async fn update_outputs(&mut self, rx: Receiver<OutputInfo>) -> io::Result<()> {
        let mut next_payload: Option<OutputInfo> = None;

        loop {
            // Process all pending updates that are past deadline
            self.apply_pending().await;

            if next_payload.is_none() {
                // no payload queued, await the earliest timer or next event
                let least_wait_time = self
                    .pending_updates
                    .values()
                    .map(|p| p.wait_time())
                    .min_by(|o, p| p.cmp(o))
                    .flatten();

                future::race(
                    async {
                        next_payload = Some(rx.recv().await.unwrap());
                    },
                    async {
                        if let Some(least_wait_time) = least_wait_time {
                            // wait for earliest debounce timer to expire
                            Timer::after(least_wait_time).await;
                        } else if self.pending_updates.is_empty() {
                            // no pending updates, wait forever until another payload comes in
                            future::pending::<()>().await;
                        }
                        // else, we have a timer that's expired and should continue the loop
                    },
                )
                .await;

                if next_payload.is_none() {
                    // timer firing won the race, apply pending first
                    continue;
                }
            }

            let output = next_payload.clone().unwrap();
            next_payload = None;

            // Map output to background image
            let desired_path = self
                .config
                .tags
                .get(&output.tag().to_string())
                .unwrap_or(&self.config.default);

            // update pending update or refresh it
            let pending = match self.pending_updates.get(&output.name) {
                Some(pending) => {
                    if &pending.path != desired_path {
                        // we're trying to change backgrounds, update pending and reset the debounce timer
                        PendingUpdate::new(desired_path)
                    } else {
                        pending.clone()
                    }
                }
                None => PendingUpdate::new(desired_path),
            };

            // Updating pending entry
            self.pending_updates
                .insert(output.name.to_string(), pending.clone());
        }
    }

    async fn apply_pending(&mut self) {
        let mut done = Vec::new();

        // apply all pending events that have fired
        for (output, pending) in &self.pending_updates {
            if pending.wait_time().is_none() {
                _ = self.apply_background(output, &pending.path).await;
                done.push(output.clone());
            }
        }

        // clear old entries
        for key in done {
            self.pending_updates.remove(&key);
        }
    }

    async fn apply_background(&self, output: &str, path: &str) -> io::Result<()> {
        Command::new("swww")
            .arg("img")
            .arg("-o")
            .arg(output)
            .args(self.config.swww_args.split(" "))
            .arg(path)
            .spawn()?;
        Ok(())
    }
}

main! {
    async fn main() -> std::io::Result<()> {
        let (tx, rx) = unbounded();

        let config = Config::load()?;
        let mut backend = SwwwBackend::new(&config);

        monitor_bedload(tx).or(backend.update_outputs(rx)).await?;

        Ok(())
    }
}
