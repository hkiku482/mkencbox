## mkencbox

Key file based encryptor for file or directory tree.

### Usage

```
Usage: mkencbox [OPTIONS] <PROCESS> <KEY_FILE> <INPUT> [OUTPUT]

Arguments:
  <PROCESS>   Encrypt or decrypt process [possible values: enc, dec]
  <KEY_FILE>  Key file path
  <INPUT>     Input name
  [OUTPUT]    Output name

Options:
  -s, --salt <SALT>  Salt
  -m, --mode <MODE>  Encryption algorithm [default: chacha20] [possible values: cbc, chacha20]
  -h, --help         Print help
  -V, --version      Print version
```

### The following 2 have the same behavior.

```
openssl enc -e -aes-256-cbc -pbkdf2 -iter 600000 -pass pass:$(sha256sum KFILE | awk '{print $1}')0$(md5sum KFILE | awk '{print $1}') -S "0123456789ABCDEF" -in INPUT -out OUTPUT
```

```
./mkencbox -m cbc -s "0123456789ABCDEF" enc KFILE INPUT OUTPUT
```

### General use

```
./mkencbox enc KFILE INPUT OUTPUT
```

### Tips

#### en/decrypt huge file

`/tmp` directory size may be limited by your OS.

```
export TMPDIR=/not/ramdisk
```

### More info

```
./mkencbox --help
```
