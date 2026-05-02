use std::io::{self, Write};

use arboard::Clipboard;

pub struct MultiWriter<'a, W1: Write, W2: Write> {
    pub writer1: &'a mut W1,
    pub writer2: &'a mut W2,
}

impl<'a, W1: Write, W2: Write> Write for MultiWriter<'a, W1, W2> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.writer1.write_all(buf)?;
        self.writer2.write_all(buf)?;
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.writer1.flush()?;
        self.writer2.flush()?;
        Ok(())
    }
}

pub fn tee<'a, W1: Write, W2: Write>(w1: &'a mut W1, w2: &'a mut W2) -> MultiWriter<'a, W1, W2> {
    MultiWriter {
        writer1: w1,
        writer2: w2,
    }
}

pub struct ClipboardWriter {
    buffer: Vec<u8>,
    clipboard: Clipboard,
}

impl ClipboardWriter {
    pub fn new() -> io::Result<Self> {
        let clipboard = Clipboard::new()
            .map_err(|e| io::Error::other(format!("Failed to access clipboard: {}", e)))?;

        Ok(Self {
            buffer: Vec::new(),
            clipboard,
        })
    }

    fn copy_to_clipboard(&mut self) -> io::Result<()> {
        let text = String::from_utf8_lossy(&self.buffer);
        self.clipboard
            .set_text(text.as_ref())
            .map_err(|e| io::Error::other(format!("Failed to set clipboard: {}", e)))
    }
}

impl Write for ClipboardWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.buffer.extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.copy_to_clipboard()
    }
}

impl Drop for ClipboardWriter {
    fn drop(&mut self) {
        let _ = self.copy_to_clipboard();
    }
}
