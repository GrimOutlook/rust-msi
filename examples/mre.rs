use std::{fs::File, io::Write, str::FromStr};

use clap::{Parser, command};
use clio::ClioPath;
use msi::Value;
use uuid::Uuid;

/// Simple minimum reproducible example showing that a base MSI becomes invalid
/// after writing a property to the file.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Filename of the Schema to use
    #[arg(value_parser = clap::value_parser!(ClioPath))]
    schema: ClioPath,

    /// Filename of the MSI to create
    #[arg(value_parser = clap::value_parser!(ClioPath), default_value = "mre.msi")]
    output: ClioPath,

    /// Revision Number (SummaryInformation property #9) to use for the generated MSI.
    /// `msitools` doesn't provide a way to set this so during testing, when
    /// trying to make the MSIs as similar as possible, we may want to set this
    /// based on external information (e.g an `msitools` generated MSI)
    #[arg(long, short ,value_parser = clap::value_parser!(Uuid), default_value_t = Uuid::new_v4())]
    revision_number: Uuid,
}

fn main() {
    let args = Args::parse();
    let mut template = File::open(args.schema.to_path_buf()).unwrap();
    let mut file = File::options()
        .create(true)
        .read(true)
        .write(true)
        .truncate(true)
        .open(args.output.to_path_buf())
        .expect("Failed to open broken.msi");
    std::io::copy(&mut template, &mut file).unwrap();
    // Probably not needed but whatever
    file.flush().unwrap();

    let mut package = msi::Package::open(file).expect("Failed to create MSI");

    // Summary Information
    let sum = package.summary_info_mut();
    sum.set_uuid(
        *uuid::fmt::Braced::from_str("{11111111-1111-1111-1111-111111111111}")
            .unwrap()
            .as_uuid(),
    ); // PID 9
    package.flush().unwrap();

    // Required Property Information
    package
        .insert_rows(msi::Insert::into("Property").rows(vec![
            vec![Value::from("ProductName"), Value::from("MyProduct")],
            vec![
                Value::from("ProductCode"),
                Value::from("{11111111-1111-1111-1111-111111111113}"),
            ],
            vec![Value::from("ProductVersion"), Value::from("0.0.0")],
            vec![Value::from("ProductLanguage"), Value::from("1033")],
            vec![Value::from("Manufacturer"), Value::from("MyManufacturer")],
        ]))
        .unwrap();

    package.flush().unwrap();
}
