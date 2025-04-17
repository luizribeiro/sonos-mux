use super::{AudioBuffer, AudioInput, InputError};
use crossbeam_channel::Sender;
use std::{thread, time::Duration};

#[derive(Debug, Default)]
pub struct SilenceInput {
    thread_handle: Option<thread::JoinHandle<()>>,
    running: bool,
}

impl Clone for SilenceInput {
    fn clone(&self) -> Self {
        // We don't clone the thread handle, just create a new instance
        Self {
            thread_handle: None,
            running: false,
        }
    }
}

impl SilenceInput {
    pub fn new() -> Self {
        Self::default()
    }
}

impl AudioInput for SilenceInput {
    fn start(&mut self, sender: Sender<AudioBuffer>) -> Result<(), InputError> {
        if self.running {
            return Ok(());
        }

        self.running = true;

        self.thread_handle = Some(thread::spawn(move || {
            while sender.send(vec![0; 1024]).is_ok() {
                thread::sleep(Duration::from_millis(100));
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
