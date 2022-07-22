use anyhow::Result;
use serialport::SerialPort;
use std::fs;

pub struct SerialPerLine {
    line: String,
    buf: [u8; 1024],
    file: Option<fs::File>,
    port: Box<dyn SerialPort>,
    process_line: fn(&String, &mut Option<fs::File>) -> Result<()>,
}

impl SerialPerLine {
    pub fn new(
        port: Box<dyn SerialPort>,
        process_line: fn(&String, &mut Option<fs::File>) -> Result<()>,
    ) -> Self {
        Self {
            line: {
                let mut line = String::new();
                line.reserve(1024);
                line
            },
            buf: [0; 1024],
            file: None,
            port,
            process_line,
        }
    }

    pub fn read(&mut self) -> Result<()> {
        match self.port.read(&mut self.buf) {
            Ok(size) => {
                for i in 0..size {
                    let c = self.buf[i] as char;

                    if c == '\0' || c == '\r' {
                        continue;
                    }

                    if c == '\n' {
                        (self.process_line)(&mut self.line, &mut self.file)?;
                        self.line.clear();
                    } else {
                        self.line.push(c);
                    }
                }
            }
            Err(_) => {}
        }
        Ok(())
    }
}
