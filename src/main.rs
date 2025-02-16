use indicatif::ProgressStyle;
use mkencbox::{Chacha20, Process, Tar};
use tokio::sync::mpsc::channel;

mod os_args;

#[tokio::main]
async fn main() {
    let args = os_args::OsArgs::parse();

    let pack_alg = Tar::new();
    let crypto_alg = Box::new(Chacha20::new(args.salt, args.key_file));
    let processor = Process::new(
        args.process,
        Box::new(pack_alg),
        crypto_alg,
        args.input,
        args.output,
    );

    let (tx, mut rx) = channel(4);
    let processor = if args.progress {
        processor.bypass_progress(tx)
    } else {
        processor
    };

    let run_progress = args.progress;
    let handle = tokio::spawn(async move {
        if !run_progress {
            return;
        }
        let p_size = 100;
        let pb = indicatif::ProgressBar::new(p_size);
        pb.set_style(
            ProgressStyle::with_template("{spinner:.green} [{bar:.cyan/blue}] {pos}% ").unwrap(),
        );
        let mut current = 0;
        while let Some(p) = rx.recv().await {
            let inc = ((p as f64 / u8::MAX as f64) * p_size as f64).ceil() as u64;
            pb.inc(inc - current);
            current = inc;
        }
        pb.finish_with_message("done");
    });

    match processor.execute().await {
        Ok(_) => {
            handle.abort();
        }
        Err(e) => {
            panic!("{e:?}");
        }
    }
}
