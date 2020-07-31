use super::channel::Channel;

use std::error::Error;

/// This trait defines the interface that can be used to communicate with a scanner from the rest
/// of the program.
pub trait Scanner {
    /// Download information on one particular channel (as requested by id) from the Scanner.
    fn download_channel(&self, id: usize) -> Result<Channel, Box<dyn Error>>;

    /// Upload channel information for the requested channel to the Scanner.
    fn upload_channel(&self, id: usize, channel: Channel) -> Result<(), Box<dyn Error>>;

    /// The number of channels supported by the Scanner.
    fn number_of_channels(&self) -> usize;
}
