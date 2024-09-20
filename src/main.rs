use mkencbox::{crypto::cbcpbkdf2::CbcPbkdf2, pack::tar::Tar};

mod os_args;

fn main() {
    let args = os_args::OsArgs::parse();

    let pack_alg = Tar::new();
    let crypto_alg = CbcPbkdf2::new(args.salt, args.key_file);
    let processor = mkencbox::process::Process::new(
        args.process,
        Box::new(pack_alg),
        Box::new(crypto_alg),
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
