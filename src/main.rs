use mkencbox::{CbcPbkdf2, Chacha20, Crypto, Process, Tar};

mod mode;
mod os_args;

fn main() {
    let args = os_args::OsArgs::parse();

    let pack_alg = Tar::new();
    let crypto_alg: Box<dyn Crypto> = match args.mode {
        mode::Mode::Cbc => Box::new(CbcPbkdf2::new(args.salt, args.key_file)),
        mode::Mode::Chacha => Box::new(Chacha20::new(args.salt, args.key_file)),
    };
    let processor = Process::new(
        args.process,
        Box::new(pack_alg),
        crypto_alg,
        args.input,
        args.output,
    );
    match processor.execute() {
        Ok(_) => {}
        Err(e) => {
            panic!("{e:?}");
        }
    }
}
