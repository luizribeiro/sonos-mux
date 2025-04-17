use super::input::{Input, Frames};
use async_trait::async_trait;

pub struct HttpInput;

impl HttpInput {
    pub fn new(_url: &str) -> anyhow::Result<Self> {
        Ok(Self)
    }
}

#[async_trait]
impl Input for HttpInput {
    async fn next_frames(&mut self) -> Option<Frames> {
        None
    }
}