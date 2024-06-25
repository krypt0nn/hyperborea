use std::io::Write;

use crate::args::Args;

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct Shell {
    pub prefix: String,
    pub new_lines: bool
}

impl Shell {
    #[inline]
    pub fn new(prefix: impl ToString, new_lines: bool) -> Self {
        Self {
            prefix: prefix.to_string(),
            new_lines
        }
    }

    pub fn poll(&self) -> anyhow::Result<Args> {
        let stdin = std::io::stdin();
        let mut stdout = std::io::stdout();

        let mut command = String::new();

        if self.new_lines {
            stdout.write_all(b"\n")?;
        }

        stdout.write_all(self.prefix.as_bytes())?;
        stdout.flush()?;

        stdin.read_line(&mut command)?;

        if self.new_lines {
            stdout.write_all(b"\n")?;
            stdout.flush()?;
        }

        Ok(Args::parse(&command))
    }
}
