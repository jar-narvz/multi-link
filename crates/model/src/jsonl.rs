//! Newline-terminated JSON records, preserving an unfinished final line.

use std::{
    fs::File,
    io::{self, BufRead, BufReader},
    path::Path,
};

use crate::{SkipReason, Skipped};
use serde_json::Value;

/// One complete line: either a parsed value or a recoverable skip.
pub type Record = Result<(usize, Value), Skipped>;

/// Reads complete lines only, including CRLF. Even valid complete JSON without
/// a final newline is held in `pending`, never yielded or reported as skipped.
/// I/O failures are returned separately from malformed JSON.
pub struct JsonlReader<R> {
    reader: R,
    display_path: String,
    line: usize,
    pending: Vec<u8>,
    finished: bool,
}

impl<R: BufRead> JsonlReader<R> {
    pub fn new(reader: R, display_path: impl Into<String>) -> Self {
        Self {
            reader,
            display_path: display_path.into(),
            line: 0,
            pending: Vec::new(),
            finished: false,
        }
    }

    /// Unterminated bytes at EOF, available after exhausting the reader.
    /// The next line number is `pending_line()`; a tailer can prepend these
    /// bytes to later input without losing a partially written UTF-8 character.
    pub fn pending(&self) -> &[u8] {
        &self.pending
    }

    pub fn pending_line(&self) -> usize {
        self.line + 1
    }
}

impl JsonlReader<BufReader<File>> {
    /// `path` is used to open the file; `display_path` is copied into skips.
    pub fn from_path(path: impl AsRef<Path>, display_path: impl Into<String>) -> io::Result<Self> {
        Ok(Self::new(BufReader::new(File::open(path)?), display_path))
    }
}

impl<R: BufRead> Iterator for JsonlReader<R> {
    type Item = io::Result<Record>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }
        let mut bytes = Vec::new();
        match self.reader.read_until(b'\n', &mut bytes) {
            Err(error) => {
                self.finished = true;
                return Some(Err(error));
            }
            Ok(0) => {
                self.finished = true;
                return None;
            }
            Ok(_) if bytes.last() != Some(&b'\n') => {
                self.pending = bytes;
                self.finished = true;
                return None;
            }
            Ok(_) => {}
        }
        self.line += 1;
        bytes.pop();
        if bytes.last() == Some(&b'\r') {
            bytes.pop();
        }
        let empty = std::str::from_utf8(&bytes).is_ok_and(|line| line.trim().is_empty());
        let record = if empty {
            Err(SkipReason::EmptyLine)
        } else {
            serde_json::from_slice(&bytes).map_err(|_| SkipReason::InvalidJson)
        };
        Some(Ok(record.map(|value| (self.line, value)).map_err(
            |reason| Skipped {
                file: self.display_path.clone(),
                line: self.line,
                reason,
            },
        )))
    }
}
