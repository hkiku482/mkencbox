use std::path::PathBuf;

use crate::os_args::OsArgs;
mod os_args;

fn main() {
    let args = OsArgs::parse();
    let salt = args.salt.map(|s| hex::decode(s).unwrap());
    if args.process == 'e' {
        mkencbox::mkencbox::enc(
            &PathBuf::from(args.key_file),
            &PathBuf::from(args.input_file),
            &PathBuf::from(args.output_file),
            salt.as_ref(),
        )
        .unwrap();
    } else if args.process == 'd' {
        mkencbox::mkencbox::dec(
            &PathBuf::from(args.key_file),
            &PathBuf::from(args.input_file),
            &PathBuf::from(args.output_file),
            salt.as_ref(),
        )
        .unwrap();
    }
}
