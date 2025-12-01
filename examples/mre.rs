use std::{fs::File, io::Write, str::FromStr};

use clap::{App, Arg};
use msi::Value;

fn main() {
    let matches = App::new("mre")
        .arg(Arg::with_name("schema").required(true))
        .arg(Arg::with_name("output").required(true).default_value("out.msi"))
        .get_matches();
    let schema = matches.value_of("schema").unwrap();
    let output = matches.value_of("output").unwrap();

    let mut template = File::open(schema).unwrap();
    let mut file = File::options()
        .create(true)
        .read(true)
        .write(true)
        .truncate(true)
        .open(output)
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
