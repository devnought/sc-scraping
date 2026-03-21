use std::{borrow::Cow, error::Error, fmt::Display};

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum ScScrapingError<'a> {
    Citizen { message: Cow<'a, str> },
    Organization { message: Cow<'a, str> },
}

impl<'a> ScScrapingError<'a> {
    pub fn citizen<S>(message: S) -> Self
    where
        S: Into<Cow<'a, str>>,
    {
        Self::Citizen {
            message: message.into(),
        }
    }

    pub fn organization<S>(message: S) -> Self
    where
        S: Into<Cow<'a, str>>,
    {
        Self::Organization {
            message: message.into(),
        }
    }
}

impl Error for ScScrapingError<'_> {}
impl Display for ScScrapingError<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Citizen { message } | Self::Organization { message } => write!(f, "{message}"),
        }
    }
}
