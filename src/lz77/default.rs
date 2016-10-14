use std::cmp;
use std::collections::HashMap;

use super::Code;
use super::Sink;
use super::Lz77Encode;
use super::CompressionMode;

#[derive(Debug)]
pub struct DefaultLz77Encoder {
    window_size: u16,
    buf: Vec<u8>,
}
impl DefaultLz77Encoder {
    pub fn new() -> Self {
        Self::with_window_size(super::MAX_WINDOW_SIZE)
    }
    pub fn with_window_size(size: u16) -> Self {
        DefaultLz77Encoder {
            window_size: cmp::min(size, super::MAX_WINDOW_SIZE),
            buf: Vec::new(),
        }
    }
}
impl Lz77Encode for DefaultLz77Encoder {
    fn encode<S>(&mut self, buf: &[u8], sink: S)
        where S: Sink
    {
        self.buf.extend_from_slice(buf);
        if self.buf.len() >= self.window_size as usize * 8 {
            self.flush(sink);
        }
    }
    fn flush<S>(&mut self, mut sink: S)
        where S: Sink
    {
        let mut prefix_table = HashMap::new();
        let mut i = 0;
        while i < cmp::max(3, self.buf.len()) - 3 {
            let key = prefix(&self.buf[i..]);
            let matched = prefix_table.insert(key, i);
            if let Some(j) = matched {
                let distance = i - j;
                if distance <= self.window_size as usize {
                    let length = 3 + longest_common_prefix(&self.buf, i + 3, j + 3);
                    sink.consume(Code::new_pointer(length, distance as u16));
                    i += length as usize;
                    continue;
                }
            }
            sink.consume(Code::new_literal(self.buf[i]));
            i += 1;
        }
        for b in &self.buf[i..] {
            sink.consume(Code::Literal(*b));
        }
        self.buf.clear();
    }
    fn compression_mode(&self) -> CompressionMode {
        CompressionMode::Balance
    }
    fn window_size(&self) -> u16 {
        self.window_size
    }
}

fn prefix(buf: &[u8]) -> [u8; 3] {
    unsafe { [*buf.get_unchecked(0), *buf.get_unchecked(1), *buf.get_unchecked(2)] }
}

fn longest_common_prefix(buf: &[u8], i: usize, j: usize) -> u16 {
    buf[i..]
        .iter()
        .take(super::MAX_LENGTH as usize - 3)
        .zip(&buf[j..])
        .take_while(|&(x, y)| x == y)
        .count() as u16
}
