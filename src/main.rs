extern crate csv;
extern crate libusb;

use clap::{
    Arg,
    App,
};
use program_scanner::{
    Channel,
    find_device,
};
use program_scanner::csv_util::{
    read_csv,
    write_csv,
};

fn download(file: &str) {
    let context = libusb::Context::new().unwrap();
    let scanner = find_device(&context)
        .expect("Could not open a device");

    let mut channels = Vec::<Channel>::new();

    for id in 0..scanner.number_of_channels() {
        if let Ok(channel) = scanner.download_channel(id + 1) {
            channels.push(channel);
        }
    }

    write_csv(file, channels).unwrap();
}

fn upload(file: &str) {
    let channels = read_csv(file).unwrap();

    let context = libusb::Context::new().unwrap();
    let scanner = find_device(&context)
        .expect("Could not open a device");

    for channel in channels {
        let id = channel.id;

        if let Err(_) = scanner.upload_channel(channel.id, channel) {
            eprintln!("Could not upload channel {}", id);
        };
    }

}

fn main() {
    let matches = App::new("Scanner Programmer")
        .about("Programming your scanner")
        .subcommand(App::new("download")
            .about("Download a list of channels from the device")
            .arg(Arg::new("file")
                 .long("file")
                 .short('f')
                 .required(true)
                 .value_name("CSV FILE")
                 .takes_value(true)
                 .about("The file in which to store channel information.")))
        .subcommand(App::new("upload")
            .about("Uploads a list of channels to the device")
            .arg(Arg::new("file")
                 .long("file")
                 .short('f')
                 .required(true)
                 .value_name("CSV FILE")
                 .takes_value(true)
                 .about("The file from which to read channel information")))
        .get_matches();

    if let Some(subcommand) = matches.subcommand_matches("download") {
        if let Some(file) = subcommand.value_of("file") {
            return download(file);
        }
    } else if let Some(subcommand) = matches.subcommand_matches("upload") {
        if let Some(file) = subcommand.value_of("file") {
            return upload(file);
        }
    }
}
