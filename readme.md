## mkencbox

### The following 2 have the same behavior.

```
openssl enc -e -aes-256-cbc -pbkdf2 -iter 600000 -pass pass:$(sha256sum KFILE | awk '{print $1}')0$(md5sum KFILE | awk '{print $1}') -S "0123456789ABCDEF" -in INPUT -out OUTPUT
```

```
./mkencbox -s "0123456789ABCDEF" enc KFILE INPUT OUTPUT
```

### General use

```
./mkencbox enc KFILE INPUT OUTPUT
```

### More info

```
./mkencbox --help
```
