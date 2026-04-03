use std::io::Write;

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
    MultiWriter { writer1: w1, writer2: w2 }
}