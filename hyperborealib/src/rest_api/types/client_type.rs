use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ClientType {
    Thin,
    Thick,
    Server,
    File
}

impl Default for ClientType {
    #[inline]
    fn default() -> Self {
        Self::Thin
    }
}

impl FromStr for ClientType {
    type Err = ();

    fn from_str(str: &str) -> Result<Self, Self::Err> {
        match str {
            "thin"   => Ok(Self::Thin),
            "thick"  => Ok(Self::Thick),
            "server" => Ok(Self::Server),
            "file"   => Ok(Self::File),

            _ => Err(())
        }
    }
}

impl std::fmt::Display for ClientType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Thin   => write!(f, "thin"),
            Self::Thick  => write!(f, "thick"),
            Self::Server => write!(f, "server"),
            Self::File   => write!(f, "file")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_types() -> &'static [(ClientType, &'static str)] {
        &[
            (ClientType::Thin,   "thin"),
            (ClientType::Thick,  "thick"),
            (ClientType::Server, "server"),
            (ClientType::File,   "file")
        ]
    }

    #[test]
    fn serialize() {
        for (client_type, name) in get_types() {
            assert_eq!(client_type.to_string(), *name);
        }
    }

    #[test]
    fn deserialize() -> Result<(), ()> {
        for (client_type, name) in get_types() {
            assert_eq!(ClientType::from_str(name)?, *client_type);
        }

        Ok(())
    }
}
