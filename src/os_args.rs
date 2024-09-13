use clap::{Arg, Command};
use std::process::exit;

pub struct OsArgs {
    pub salt: Option<String>,
    pub process: char,
    pub key_file: String,
    pub input_file: String,
    pub output_file: String,
}

const APP_NAME: &str = "mkencbox";

impl OsArgs {
    pub fn parse() -> Self {
        let id_salt = "SALT";
        let id_process = "PROCESS";
        let id_key_file = "KEY_FILE";
        let id_input = "INPUT";
        let id_output_file = "OUTPUT_FILE";

        let command = Command::new(APP_NAME)
            .arg(
                Arg::new(id_salt)
                    .help("Salt if you use.")
                    .required(false)
                    .long("salt")
                    .short('s'),
            )
            .arg(
                Arg::new(id_process)
                    .help("Encryption or decryption.")
                    .required(true)
                    .value_parser(["enc", "dec"]),
            )
            .arg(Arg::new(id_key_file).help("Key file path.").required(true))
            .arg(
                Arg::new(id_input)
                    .help("File (include encrypted) or directory. Directory is compressed with .tar.gz.")
                    .required(true),
            )
            .arg(
                Arg::new(id_output_file)
                    .help("Output file name. Decompress the output when .tar.gz is specified as the extension.")
                    .required(true),
            )
            .get_matches();

        let salt = command.get_one::<String>(id_salt).map(String::from);

        let process = match command.get_one::<String>(id_process) {
            Some(v) => {
                if v == "enc" {
                    'e'
                } else if v == "dec" {
                    'd'
                } else {
                    exit(1)
                }
            }
            None => exit(1),
        };

        let key_file = command.get_one::<String>(id_key_file).unwrap();
        let input_file = command.get_one::<String>(id_input).unwrap();
        let output_file = command.get_one::<String>(id_output_file).unwrap();

        OsArgs {
            salt,
            process,
            key_file: String::from(key_file),
            input_file: String::from(input_file),
            output_file: String::from(output_file),
        }
    }
}
