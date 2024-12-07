use std::fs::read;

use mkencbox::{CbcPbkdf2, Process, Tar, Target};
use test_util::{dir_entries, kfile, relative_path};

mod test_util;

#[test]
fn file_enc_dec_test() {
    let tag = "file_enc_dec_test";
    test_util::prepare(tag);
    let kfile = kfile();
    let pack_alg = Tar::new();
    let crypto_alg = CbcPbkdf2::new(None, &kfile);

    let (infile, outfile) = relative_path(tag, "a.txt", "a.txt.enc");

    let processor = Process::new(
        Target::Enc,
        Box::new(pack_alg),
        Box::new(crypto_alg),
        &infile,
        &outfile,
    );
    processor.execute().unwrap();

    let a = read(&outfile).unwrap();
    let salted = "Salted__".as_bytes().to_vec();
    assert_eq!(&a[..8], salted);

    let (_, dec_out) = relative_path(tag, "", "a.txt");
    let pack_alg = Tar::new();
    let crypto_alg = CbcPbkdf2::new(None, kfile);
    let processor = Process::new(
        Target::Dec,
        Box::new(pack_alg),
        Box::new(crypto_alg),
        &outfile,
        &dec_out,
    );
    processor.execute().unwrap();
    let e = read(&infile).unwrap();
    let a = read(&dec_out).unwrap();
    assert_eq!(e, a);

    let result = processor.execute();
    assert!(result.is_err())
}

#[test]
fn file_enc_salt_test() {
    let tag = "file_enc_salt_test";
    test_util::prepare(tag);
    let pack_alg = Tar::new();
    let kfile = kfile();
    let crypto_alg = CbcPbkdf2::new(Some("0123456789ABCDEF".to_string()), &kfile);

    let (infile, outfile) = relative_path(tag, "files/c.txt", "c.txt.enc");

    let processor = Process::new(
        Target::Enc,
        Box::new(pack_alg),
        Box::new(crypto_alg),
        &infile,
        &outfile,
    );
    processor.execute().unwrap();

    let (e, a) = relative_path(tag, "exp_salt/c.txt.enc", "c.txt.enc");
    let e = read(e).unwrap();
    let a = read(a).unwrap();
    assert_eq!(e, a);

    let (_, dec_out) = relative_path(tag, "", "c.txt");
    let pack_alg = Tar::new();
    let crypto_alg = CbcPbkdf2::new(Some("0123456789ABCDEF".to_string()), kfile);
    let processor = Process::new(
        Target::Dec,
        Box::new(pack_alg),
        Box::new(crypto_alg),
        &outfile,
        &dec_out,
    );
    processor.execute().unwrap();
    let e = read(&infile).unwrap();
    let a = read(&dec_out).unwrap();
    assert_eq!(e, a);

    let result = processor.execute();
    assert!(result.is_err())
}

#[test]
fn directory_enc_test() {
    let tag = "directory_enc_test";
    test_util::prepare(tag);
    let pack_alg = Tar::new();
    let kfile = kfile();
    let crypto_alg = CbcPbkdf2::new(None, &kfile);

    let (infile, outfile) = relative_path(tag, "dir", "dir.enc");

    let processor = Process::new(
        Target::Enc,
        Box::new(pack_alg),
        Box::new(crypto_alg),
        &infile,
        &outfile,
    );
    processor.execute().unwrap();

    let a = read(&outfile).unwrap();
    let salted = "Salted__".as_bytes().to_vec();
    assert_eq!(&a[..8], salted);

    let (_, dec_out) = relative_path(tag, "", "dir");
    let pack_alg = Tar::new();
    let crypto_alg = CbcPbkdf2::new(None, kfile);
    let processor = Process::new(
        Target::Dec,
        Box::new(pack_alg),
        Box::new(crypto_alg),
        &outfile,
        &dec_out,
    );
    processor.execute().unwrap();

    assert_eq!(dir_entries(infile), dir_entries(dec_out));

    let result = processor.execute();
    assert!(result.is_err())
}

#[test]
fn directory_enc_salt_test() {
    let tag = "directory_enc_salt_test";
    test_util::prepare(tag);
    let pack_alg = Tar::new();
    let kfile = kfile();
    let crypto_alg = CbcPbkdf2::new(Some("0123456789ABCDEF".to_string()), &kfile);

    let (infile, outfile) = relative_path(tag, "dir", "dir.enc");

    let processor = Process::new(
        Target::Enc,
        Box::new(pack_alg),
        Box::new(crypto_alg),
        &infile,
        &outfile,
    );
    processor.execute().unwrap();

    let (_, dec_out) = relative_path(tag, "", "dir");
    let pack_alg = Tar::new();
    let crypto_alg = CbcPbkdf2::new(Some("0123456789ABCDEF".to_string()), kfile);
    let processor = Process::new(
        Target::Dec,
        Box::new(pack_alg),
        Box::new(crypto_alg),
        &outfile,
        &dec_out,
    );
    processor.execute().unwrap();

    assert_eq!(dir_entries(infile), dir_entries(dec_out));

    let result = processor.execute();
    assert!(result.is_err())
}
