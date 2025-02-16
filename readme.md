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
      --progress     Show progress
  -h, --help         Print help
  -V, --version      Print version
```

### General use

```
./mkencbox enc KFILE INPUT OUTPUT
```

### Tips

#### en/decrypt huge file

`/tmp` directory size may be limited by your OS.

```
EXPORT=TMPDIR=/not/ramdisk
```

### More info

```
./mkencbox --help
```
