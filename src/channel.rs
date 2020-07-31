use serde::{
    Deserialize,
    Deserializer,
    Serialize,
    Serializer,
};
use serde::de::Error as DeserializerError;
use std::error::Error;
use std::fmt::{
    Debug,
    Display,
    Formatter,
    Result as FmtResult,
};
use std::str::FromStr;

pub enum ParseError {
    Delay,
    Frequency,
    Locked,
    Modulation,
    Priority,
}

impl Debug for ParseError {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "{}", self)
    }
}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "{}", match self {
            Self::Delay => "Could not parse the delay",
            Self::Frequency => "Could not parse the frequency",
            Self::Locked => "Could not paprse the locked state",
            Self::Modulation => "Could not parse the modulation",
            Self::Priority => "Could not parse the priority",
        })
    }
}

impl Error for ParseError {}

#[derive(Clone, Copy)]
pub enum Modulation {
    AmplitudeModulation,
    FrequencyModulation,
}

impl FromStr for Modulation {
    type Err = ParseError;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        match string {
            "AM" => Ok(Self::AmplitudeModulation),
            "FM" => Ok(Self::FrequencyModulation),
            _ => Err(ParseError::Modulation),
        }
    }
}

impl<'de> Deserialize<'de> for Modulation {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where D: Deserializer<'de> {
        let string: String = Deserialize::deserialize(deserializer)?;

        match string.as_str() {
            "AM" => Ok(Self::AmplitudeModulation),
            "FM" => Ok(Self::FrequencyModulation),
            _ => Err(D::Error::custom("Could not deserialize modulation")),
        }
    }
}

impl Serialize for Modulation {
    fn serialize<S>(&self, serializer: S) -> Result <S::Ok, S::Error>
            where S: Serializer {
        serializer.serialize_str(match self {
            Self::AmplitudeModulation => "AM",
            Self::FrequencyModulation => "FM",
        })
    }
}

#[derive(Deserialize, Serialize)]
pub struct Channel {
    pub id: usize,
    pub tag: String,
    pub frequency: f64,
    pub modulation: Modulation,
    pub delay: usize,
    pub locked: bool,
    pub priority: bool,
}
