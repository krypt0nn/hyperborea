#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct Args {
    args: Vec<String>
}

impl Args {
    pub fn parse(command: impl AsRef<str>) -> Self {
        let command = command.as_ref()
            .chars()
            .collect::<Vec<_>>();

        let mut i = 0;
        let n = command.len();

        let mut arg = String::new();
        let mut args = Vec::new();

        while i < n {
            if command[i].is_whitespace() {
                if !arg.is_empty() {
                    args.push(arg.clone());

                    arg.clear();
                }
            }

            else {
                arg.push(command[i]);
            }

            i += 1;
        }

        if !arg.is_empty() {
            args.push(arg);
        }

        Self { args }
    }

    #[inline]
    pub fn from_env() -> Self {
        Self {
            args: std::env::args()
                .skip(1)
                .collect::<Vec<_>>()
        }
    }

    #[inline]
    pub fn command(&self) -> Option<&str> {
        self.args.first().map(|arg| arg.as_str())
    }

    #[inline]
    pub fn args(&self) -> &[String] {
        if self.args.len() <= 1 {
            return &[];
        }

        &self.args[1..]
    }
}
