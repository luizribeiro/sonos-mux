use super::{AudioBuffer, AudioInput, InputError};
use crossbeam_channel::Sender;
use std::{fs, path::PathBuf, thread};

#[derive(Debug)]
pub struct FileInput {
    path: PathBuf,
    loop_playback: bool,
    thread_handle: Option<thread::JoinHandle<()>>,
    running: bool,
}

impl Clone for FileInput {
    fn clone(&self) -> Self {
        // We don't clone the thread handle, just create a new instance
        Self {
            path: self.path.clone(),
            loop_playback: self.loop_playback,
            thread_handle: None,
            running: false,
        }
    }
}

impl FileInput {
    pub fn new(path: &str, loop_playback: bool) -> Result<Self, InputError> {
        let path = PathBuf::from(path);
        if !path.exists() {
            return Err(InputError::Initialization(format!(
                "File does not exist: {:?}",
                path
            )));
        }

        Ok(Self {
            path,
            loop_playback,
            thread_handle: None,
            running: false,
        })
    }
}

impl AudioInput for FileInput {
    fn start(&mut self, sender: Sender<AudioBuffer>) -> Result<(), InputError> {
        if self.running {
            return Ok(());
        }

        let path = self.path.clone();
        let loop_playback = self.loop_playback;

        self.running = true;
        self.thread_handle = Some(thread::spawn(move || {
            let data = match fs::read(&path) {
                Ok(data) => data,
                Err(e) => {
                    eprintln!("Error reading file: {}", e);
                    return;
                }
            };

            let mut frames = Vec::with_capacity(data.len() / 2);
            for c in data.chunks_exact(2) {
                frames.push(i16::from_le_bytes([c[0], c[1]]));
            }

            loop {
                for chunk in frames.chunks(1024) {
                    if sender.send(chunk.to_vec()).is_err() {
                        return;
                    }
                }

                if !loop_playback {
                    break;
                }
            }
        }));

        Ok(())
    }

    fn stop(&mut self) -> Result<(), InputError> {
        self.running = false;
        if let Some(handle) = self.thread_handle.take() {
            let _ = handle.join();
        }
        Ok(())
    }
}
