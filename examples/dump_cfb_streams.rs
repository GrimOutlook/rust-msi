use itertools::Itertools;
use msi::decode_streamname;
use std::str::FromStr;

use clap::{App, Arg};

fn main() {
    let matches = App::new("dump_cfb_streams")
        .arg(
            Arg::with_name("msi")
                .required(true)
                .help("MSI to dump the streams from"),
        )
        .arg(Arg::from_usage(
            "-a --all 'Dump all streams from the given MSI recursively'",
        ))
        .get_matches();
    let msi = matches.value_of("msi").unwrap();
    let all = matches.is_present("all");

    // If --all was specified, just dump all of the streams recursively
    if all {
        let mut comp = cfb::open(msi).unwrap();
        let entries = comp.read_root_storage().collect::<Vec<_>>();
        std::fs::create_dir("root").unwrap();
        return dump_streams_in_entries_to_location(
            &mut comp,
            &entries,
            &std::path::PathBuf::from_str("root").unwrap(),
        );
    }

    // Else we show the user the streams at the current level and ask what they
    // want to do.
    todo!()
}
fn dump_streams_in_entries_to_location<T: std::io::Seek + std::io::Read>(
    cfb_info: &mut cfb::CompoundFile<T>,
    entries: &[cfb::Entry],
    output_location: &std::path::Path,
) {
    for entry in entries {
        if entry.is_storage() {
            let streamname = msi::decode_streamname(entry.name()).0;
            std::fs::create_dir(&streamname).unwrap();
            dump_streams_in_entries_to_location(
                cfb_info,
                &cfb_info
                    .read_storage(entry.name())
                    .expect("Failed to read storage for selection")
                    .collect_vec(),
                &(output_location.join(streamname)),
            );
            continue;
        }

        if !entry.is_stream() {
            continue;
        }

        let mut stream = cfb_info
            .open_stream(entry.name())
            .expect("Failed to read stream for selection");
        let stream_output_path =
            output_location.join(decode_streamname(entry.name()).0 + ".dump");
        println!(
            "Dumping stream [{}] to [{:?}]",
            entry.name(),
            stream_output_path
        );
        let mut new_file = std::fs::File::options()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(stream_output_path)
            .expect("Failed to create new file");

        std::io::copy(&mut stream, &mut new_file)
            .expect("Failed to copy data from stream");
    }
}
