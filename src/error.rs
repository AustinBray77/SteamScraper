use std::{error::Error, fmt::Display};

#[derive(Debug)]
pub struct SteamError {
    msg: &'static str,
}

impl SteamError {
    pub fn new(msg: &'static str) -> Self {
        SteamError { msg: msg }
    }

    pub fn boxed_new(msg: &'static str) -> Box<Self> {
        Box::new(Self::new(msg))
    }
}

impl Display for SteamError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.msg.fmt(f)
    }
}

impl Error for SteamError {}
