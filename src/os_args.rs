use clap::{Arg, Command};
use mkencbox::Target;
use std::{
    io::{BufReader, Read},
    path::PathBuf,
    process::exit,
};

#[derive(Debug)]
pub struct OsArgs {
    pub salt: Option<String>,
    pub process: Target,
    pub key_file: PathBuf,
    pub input: PathBuf,
    pub output: PathBuf,
}

const APP_NAME: &str = "mkencbox";

impl OsArgs {
    pub fn parse() -> Self {
        const ID_SALT: &str = "SALT";
        const ID_PROCESS: &str = "PROCESS";
        const ID_KEY_FILE: &str = "KEY_FILE";
        const ID_INFILE: &str = "INPUT";
        const ID_OUTFILE: &str = "OUTPUT";

        let command = Command::new(APP_NAME)
            .arg(
                Arg::new(ID_SALT)
                    .help("Salt")
                    .required(false)
                    .long("salt")
                    .short('s'),
            )
            .arg(
                Arg::new(ID_PROCESS)
                    .help("Execution. `auto` decrypt requires salt at the beginning of the input")
                    .required(true)
                    .value_parser(["enc", "dec", "auto"]),
            )
            .arg(Arg::new(ID_KEY_FILE).help("Key file path").required(true))
            .arg(Arg::new(ID_INFILE).help("Input name").required(true))
            .arg(Arg::new(ID_OUTFILE).help("Output name"))
            .get_matches();

        let salt = command.get_one::<String>(ID_SALT).map(String::from);

        let input_file = PathBuf::from(command.get_one::<String>(ID_INFILE).unwrap());
        let process = match command.get_one::<String>(ID_PROCESS) {
            Some(v) => match v.as_str() {
                "enc" => Target::Enc,
                "dec" => Target::Dec,
                "auto" => {
                    if input_file.is_file() {
                        let file = std::fs::File::open(&input_file).unwrap();
                        let mut reader = BufReader::new(file);
                        let mut buffer = [0u8; 8];
                        match reader.read(&mut buffer) {
                            Ok(read) => {
                                if read == 8 {
                                    let salt_str = "Salted__".as_bytes();
                                    if buffer.to_vec() == salt_str.to_vec() {
                                        Target::Dec
                                    } else {
                                        Target::Enc
                                    }
                                } else {
                                    Target::Enc
                                }
                            }
                            Err(_) => Target::Enc,
                        }
                    } else {
                        Target::Enc
                    }
                }
                _ => {
                    exit(1);
                }
            },
            None => {
                exit(1);
            }
        };

        let key_file = command.get_one::<String>(ID_KEY_FILE).unwrap();
        let output_file = match command.get_one::<String>(ID_OUTFILE) {
            Some(s) => PathBuf::from(s.clone()),
            None => match process {
                Target::Enc => {
                    let mut s = input_file.clone();
                    let mut path_str = s.to_str().unwrap().to_string();
                    path_str.push_str(".enc");
                    s = PathBuf::from(path_str);
                    s
                }
                Target::Dec => {
                    let mut s = input_file.clone();
                    match s.extension() {
                        Some(_) => {
                            let mut parent = s.parent().unwrap().to_path_buf();
                            let s = s.file_stem().unwrap();
                            parent.push(s);
                            parent
                        }
                        None => {
                            s.set_extension("dec");
                            s
                        }
                    }
                }
            },
        };

        OsArgs {
            salt,
            process,
            key_file: PathBuf::from(key_file),
            input: input_file,
            output: output_file,
        }
    }
}
