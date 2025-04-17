use super::input::{Input, Frames};
use async_trait::async_trait;
use std::{path::PathBuf, fs};

pub struct FileInput {
    frames: Frames,
    pos: usize,
}

impl FileInput {
    pub fn new(path: PathBuf) -> anyhow::Result<Self> {
        let data = fs::read(path)?;
        let mut frames = Vec::with_capacity(data.len() / 2);
        for c in data.chunks_exact(2) {
            frames.push(i16::from_le_bytes([c[0], c[1]]));
        }
        Ok(Self { frames, pos: 0 })
    }
}

#[async_trait]
impl Input for FileInput {
    async fn next_frames(&mut self) -> Option<Frames> {
        if self.pos >= self.frames.len() {
            return None;
        }
        let end = (self.pos + 1024).min(self.frames.len());
        let slice = self.frames[self.pos..end].to_vec();
        self.pos = end;
        Some(slice)
    }
}