```
openssl enc -e -aes-256-cbc -pbkdf2 -iter 600000 -pass pass:$(sha256sum tests/resource/keyfile | awk '{print $1}')0$(md5sum tests/resource/keyfile | awk '{print $1}') -S "0123456789ABCDEF" -in tests/resource/files/a.txt -out a.txt.enc
openssl enc -e -aes-256-cbc -pbkdf2 -iter 600000 -pass pass:$(sha256sum tests/resource/keyfile | awk '{print $1}')0$(md5sum tests/resource/keyfile | awk '{print $1}') -S "0123456789ABCDEF" -in tests/resource/files/b.txt -out b.txt.enc
openssl enc -e -aes-256-cbc -pbkdf2 -iter 600000 -pass pass:$(sha256sum tests/resource/keyfile | awk '{print $1}')0$(md5sum tests/resource/keyfile | awk '{print $1}') -S "0123456789ABCDEF" -in tests/resource/files/c.txt -out c.txt.enc
```