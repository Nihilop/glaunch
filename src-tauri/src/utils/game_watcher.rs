use notify::{Watcher, RecursiveMode, watcher, DebouncedEvent};
use std::path::Path;
use std::sync::mpsc::channel;
use std::time::Duration;
use tokio::sync::broadcast;
use crate::utils::AppError;

pub struct GameWatcher {
    tx: broadcast::Sender<GameWatchEvent>,
}

#[derive(Clone, Debug)]
pub enum GameWatchEvent {
    NewGame(String),
    GameRemoved(String),
    GameUpdated(String),
}

impl GameWatcher {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(100);
        Self { tx }
    }

    pub fn get_receiver(&self) -> broadcast::Receiver<GameWatchEvent> {
        self.tx.subscribe()
    }

    pub async fn watch_directory(&self, path: impl AsRef<Path>) -> Result<(), AppError> {
        let (tx_fs, rx_fs) = channel();

        let mut watcher = watcher(tx_fs, Duration::from_secs(2))
            .map_err(|e| AppError {
                message: format!("Failed to create watcher: {}", e)
            })?;

        watcher.watch(path, RecursiveMode::Recursive)
            .map_err(|e| AppError {
                message: format!("Failed to watch directory: {}", e)
            })?;

        let tx = self.tx.clone();

        tokio::spawn(async move {
            while let Ok(event) = rx_fs.recv() {
                match event {
                    DebouncedEvent::Create(path) => {
                        if let Some(ext) = path.extension() {
                            if ext == "acf" {
                                let _ = tx.send(GameWatchEvent::NewGame(
                                    path.to_string_lossy().to_string()
                                ));
                            }
                        }
                    },
                    DebouncedEvent::Remove(path) => {
                        if let Some(ext) = path.extension() {
                            if ext == "acf" {
                                let _ = tx.send(GameWatchEvent::GameRemoved(
                                    path.to_string_lossy().to_string()
                                ));
                            }
                        }
                    },
                    DebouncedEvent::Write(path) => {
                        if let Some(ext) = path.extension() {
                            if ext == "acf" {
                                let _ = tx.send(GameWatchEvent::GameUpdated(
                                    path.to_string_lossy().to_string()
                                ));
                            }
                        }
                    },
                    _ => {}
                }
            }
        });

        Ok(())
    }
}