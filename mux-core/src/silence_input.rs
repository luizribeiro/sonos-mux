use super::input::{Input, Frames};
use async_trait::async_trait;

pub struct SilenceInput;

impl SilenceInput {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Input for SilenceInput {
    async fn next_frames(&mut self) -> Option<Frames> {
        Some(vec![0; 1024])
    }
}