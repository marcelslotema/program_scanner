extern crate libusb;

use libusb::DeviceHandle;
use std::error::Error;
use std::fmt::{
    Debug,
    Display,
    Formatter,
    Result as FmtResult,
};
use std::num::ParseIntError;
use std::time::Duration;
use super::super::channel::{
    Channel,
    Modulation,
    ParseError,
};
use super::super::scanner::Scanner;

const USB_TIMEOUT: Duration = Duration::from_secs(1);

const ENDPOINT_OUT : u8 = 0x02;
const ENDPOINT_IN : u8 = 0x81;

/// Parse a string as a boolean value.
fn parse_bool(s: &str) -> Result<bool, ParseIntError> {
    let value = s.parse::<usize>()?;
    Ok(value != 0)
}

/// Parse a string as a usize.
fn parse_usize(s: &str) -> Result<usize, ParseIntError> {
    s.parse::<usize>()
}

/// Parse Channel information from a string.
fn parse_channel(string: &str) -> Result<Channel, ParseError> {
        let parts : Vec<&str> = string.split(',').collect();

        // Ignore the id
        let tag = parts[1].trim().to_string();
        let frequency = parse_usize(parts[2])
            .map_err(|_| ParseError::Frequency)?;
        let modulation = parts[3].parse::<Modulation>()?;
        // Ignore CTCSS / DCS for now
        let delay = parse_usize(parts[5])
            .map_err(|_| ParseError::Delay)?;
        let locked = parse_bool(parts[6])
            .map_err(|_| ParseError::Locked)?;
        let priority = parse_bool(parts[7])
            .map_err(|_| ParseError::Priority)?;

        Ok(Channel {
            id: 0,
            tag,
            frequency: (frequency * 100) as f64,
            modulation,
            delay,
            locked,
            priority,
        })
}

/// Serialize channel information
fn serialize_channel(channel: &Channel) -> String {
    format!(
        "{tag:16},{frequency:08},{modulation},0,{delay},{locked},{priority}",
        tag=channel.tag,
        frequency=(channel.frequency as usize) / 100,
        modulation=match channel.modulation {
            Modulation::AmplitudeModulation => "AM",
            Modulation::FrequencyModulation => "FM",
        },
        delay=channel.delay,
        locked=channel.locked as usize,
        priority=channel.priority as usize,
    )
}

/// Different kind of errors that can occur when interacting with the scanner.
enum UBC125XLTError {
    // Could not enter Programming Mode
    ProgrammingEnterError,
    // Could not leave Programming Mode
    ProgrammingExitError,
    // Channel data could not be uploaded
    UploadError,
    // Not all data could be written to the scanner
    WriteError,
}

impl Debug for UBC125XLTError {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "{}", self)
    }
}

impl Display for UBC125XLTError {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "{}", match self {
            Self::ProgrammingEnterError => "Could not enter Programming Mode",
            Self::ProgrammingExitError => "Could not leave Programming Mode",
            Self::UploadError => "Could not upload channel data",
            Self::WriteError => "Could not write to device",
        })
    }
}

impl Error for UBC125XLTError {}

enum Request {
    // Retrieve information on the scanner model
    Model,
    // Retrieve information on the scanner version
    Version,

    // Have the scanner enter Programming Mode
    EnterProgrammingMode,
    // Have the scanner leave Programming Mode
    ExitProgrammingMode,

    // Retrieve information on a channel from the scanner
    DownloadChannel(usize),
    UploadChannel(usize, Channel),
}

pub struct UBC125XLT <'a> {
    handle: DeviceHandle<'a>,
}

impl<'a> Drop for UBC125XLT<'a> {
    fn drop(&mut self) {
        self.exit_remote_mode().unwrap();
    }
}

impl<'a> UBC125XLT<'a> {
    pub fn new(mut handle: DeviceHandle<'a>) -> Result<Self, Box<dyn Error>> {
        if let Ok(_) = handle.unconfigure() {
            // In some cases, the device cannot be unconfigured. Most of the time,
            // that also means that the right active configuration is already set.
            handle.set_active_configuration(1)?;
        }
        handle.detach_kernel_driver(1)?;
        handle.claim_interface(1)?;

        let scanner = UBC125XLT { handle };

        // Enter remote mode so that all operations will work. Programming Mode is
        // left using the Drop trait.
        scanner.enter_remote_mode()?;

        Ok(scanner)
    }

    pub fn model(&self) -> Result<String, Box<dyn Error>> {
        Ok(self.request(Request::Model)?)
    }

    pub fn version(&self) -> Result<String, Box<dyn Error>> {
        Ok(self.request(Request::Version)?)
    }

    pub fn enter_remote_mode(&self) -> Result<(), Box<dyn Error>> {
        let result = self.request(Request::EnterProgrammingMode)?;

        if result == "OK" {
            Ok(())
        } else {
            Err(Box::new(UBC125XLTError::ProgrammingEnterError))
        }
    }

    pub fn exit_remote_mode(&self) -> Result<(), Box<dyn Error>> {
        let result = self.request(Request::ExitProgrammingMode)?;

        if result == "OK" {
            Ok(())
        } else {
            Err(Box::new(UBC125XLTError::ProgrammingExitError))
        }
    }

    fn request(&self, request: Request) -> Result<String, Box<dyn Error>> {
        let request_string = match request {
            Request::Model => "MDL".to_string(),
            Request::Version => "VER".to_string(),
            Request::EnterProgrammingMode => "PRG".to_string(),
            Request::ExitProgrammingMode => "EPG".to_string(),
            Request::DownloadChannel(id) => format!("CIN,{}", id),
            Request::UploadChannel(id, channel) => format!("CIN,{},{}", id, serialize_channel(&channel)),
        };
        self.write(request_string)?;
        let mut response = self.read()?;

        // The response starts with the request, followed by a comma (e.g.
        // "MDL,<model string>"). That first part is not interesting. Since all
        // commands are three letters long, the response can always be split at the
        // same position.
        Ok(response.split_off(4).trim().to_string())
    }

    /// Read raw data from the scanner.
    fn read(&self) -> Result<String, Box<dyn Error>> {
        let mut response = Vec::<u8>::new();

        while !response.contains(&('\r' as u8)) {
            // Assume 64 bytes is big enough for a single read
            let mut raw_buffer = [0x00; 64];

            let bytes_read = self.handle.read_bulk(
                ENDPOINT_IN,
                &mut raw_buffer,
                USB_TIMEOUT,
            )?;

            let mut buffer : Vec<u8> = raw_buffer.to_vec();
            buffer.truncate(bytes_read);
            response.append(&mut buffer);
        }

        Ok(String::from_utf8(response.to_vec())?)
    }

    /// Write raw data to the scanner.
    fn write(&self, mut buffer: String) -> Result<(), Box<dyn Error>> {
        buffer.push('\r');
        let raw_buffer = buffer.as_bytes();

        let bytes_written = self.handle.write_bulk(ENDPOINT_OUT, &raw_buffer, USB_TIMEOUT)?;

        if bytes_written == raw_buffer.len() {
            Ok(())
        } else {
            Err(Box::new(UBC125XLTError::WriteError))
        }
    }
}

impl<'a> Scanner for UBC125XLT<'a> {
    fn download_channel(&self, id: usize) -> Result<Channel, Box<dyn Error>> {
        let result = self.request(Request::DownloadChannel(id))?;
        let mut channel = parse_channel(&result)?;
        channel.id = id;

        Ok(channel)
    }

    fn upload_channel(&self, id: usize, channel: Channel) -> Result<(), Box<dyn Error>> {
        let result = self.request(Request::UploadChannel(id, channel))?;

        if result == "OK" {
            Ok(())
        } else {
            Err(Box::new(UBC125XLTError::UploadError))
        }
    }

    fn number_of_channels(&self) -> usize {
        return 500;
    }
}
