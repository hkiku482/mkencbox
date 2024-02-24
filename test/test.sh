#/bin/bash

ITER=600000
mkdir -p test/output

cargo run -- enc test/kfile.jpg test/text.txt test/output/enc
cargo run -- dec test/kfile.jpg test/output/enc test/output/dec
if [ $(md5sum test/text.txt | awk '{print $1}') = $(md5sum test/output/dec | awk '{print $1}') ]; then
    echo 'general: ok';
else
    echo 'general: fail';
fi

cargo run -- -s "0123456789abcdef" enc test/kfile.jpg test/text.txt test/output/enc2
cargo run -- -s "0123456789ABCDEF" dec test/kfile.jpg test/output/enc2 test/output/dec2
if [ $(md5sum test/text.txt | awk '{print $1}') = $(md5sum test/output/dec2 | awk '{print $1}') ]; then
    echo 'with salt: ok';
else
    echo 'with salt: fail';
fi

openssl enc -e -aes-256-cbc -pbkdf2 -iter $ITER -pass pass:$(sha256sum test/kfile.jpg | awk '{print $1}')0$(md5sum test/kfile.jpg | awk '{print $1}') -S "0123456789ABCDEF" -in test/text.txt -out test/output/enc3
cargo run -- -s "0123456789ABCDEF" dec test/kfile.jpg test/output/enc3 test/output/dec3
if [ $(md5sum test/text.txt | awk '{print $1}') = $(md5sum test/output/dec3 | awk '{print $1}') ]; then
    echo 'enc as openssl: ok';
else
    echo 'enc as openssl: fail';
fi

cargo run -- enc test/kfile.jpg test/text.txt test/output/enc4
openssl enc -d -aes-256-cbc -pbkdf2 -iter $ITER -pass pass:$(sha256sum test/kfile.jpg | awk '{print $1}')0$(md5sum test/kfile.jpg | awk '{print $1}') -in test/output/enc4 -out test/output/dec4
if [ $(md5sum test/text.txt | awk '{print $1}') = $(md5sum test/output/dec4 | awk '{print $1}') ]; then
    echo 'dec as openssl: ok';
else
    echo 'dec as openssl: fail';
fi
