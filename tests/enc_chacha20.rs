use common::{kfile, prepare, relative_path};
use mkencbox::{Chacha20, Process, Tar, Target};
use std::fs::read;

mod common;

#[tokio::test]
async fn test_chacha() {
    let tag = "test_chacha";
    prepare(tag);
    let kfile = kfile();
    let pack_alg = Tar::new();
    let crypto_alg = Chacha20::new(None, &kfile);

    let (infile, outfile) = relative_path(tag, "a.txt", "a.txt.enc");

    let processor = Process::new(
        Target::Enc,
        Box::new(pack_alg),
        Box::new(crypto_alg),
        &infile,
        &outfile,
    );
    processor.execute().await.unwrap();

    let (_, decfile) = relative_path(tag, "", "a.txt.dec");
    let pack_alg = Tar::new();
    let crypto_alg = Chacha20::new(None, &kfile);
    let processor = Process::new(
        Target::Dec,
        Box::new(pack_alg),
        Box::new(crypto_alg),
        &outfile,
        &decfile,
    );
    processor.execute().await.unwrap();

    let plain = read(&infile).unwrap();
    let encrypted = read(&outfile).unwrap();
    let decrypted = read(&decfile).unwrap();
    assert_eq!(vec![250, 62, 4, 190, 89], encrypted);
    assert_eq!(plain, decrypted);

    // with salt
    let pack_alg = Tar::new();
    let crypto_alg = Chacha20::new(Some("salt".into()), &kfile);

    let (infile, outfile) = relative_path(tag, "a.txt", "a.txt.salt.enc");

    let processor = Process::new(
        Target::Enc,
        Box::new(pack_alg),
        Box::new(crypto_alg),
        infile,
        &outfile,
    );
    processor.execute().await.unwrap();

    let (_, decfile) = relative_path(tag, "", "a.txt.salt.dec");
    let pack_alg = Tar::new();
    let crypto_alg = Chacha20::new(Some("salt".into()), &kfile);
    let processor = Process::new(
        Target::Dec,
        Box::new(pack_alg),
        Box::new(crypto_alg),
        &outfile,
        &decfile,
    );
    processor.execute().await.unwrap();

    let salt_encrypted = read(&outfile).unwrap();
    let salt_decrypted = read(&decfile).unwrap();
    assert_ne!(encrypted, salt_encrypted);
    assert_eq!(vec![229, 25, 140, 139, 136], salt_encrypted);
    assert_eq!(plain, salt_decrypted);
}
