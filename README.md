# MAC IP printer

Print MAC address, IPv4/IPv6 address and its ranges according to format specifier.

## Format

### Format Specifier

| Format character | Description          |
| ---              | ---                  |
| `m`              | MAC address          |
| `i`              | IPv4 address         |
| `x`              | IPv6 address         |
| `X`              | IPv6 full address    |
| `n`              | Number               |
| `%`              | `%` character itself |

Number specifier can be prefixed by padding length, such as `%10n`.
If padding length starts with `0`, `0` is used as padding character.
Otherwise, space (` `) is sued as padding character.

### Escape

| Escaped string | Translation          |
| ---            | ---                  |
| `\n`           | Newline character    |
| `\\`           | `\` character itself |

## Argument

### MAC address

MAC address can be specified as colon separated HEX string or just a number.

```console
$ macipr %m aa:bb:cc:dd:ee:ff
aa:bb:cc:dd:ee:ff

$ macipr %m 0
00:00:00:00:00:00
```

### IPv4 address

IPv4 address can be specified as dot separated digits or just a number.

```console
$ macipr %i 192.168.0.1
192.168.0.1

$ macipr %i 180000000
10.186.149.0
```

### IPv6 address

IPv6 address can be specified as colon separated digits or just a number.

```console
$ macipr %x ::1
::1

$ macipr %x 100000000000000
::5af3:107a:4000
```

If `%X` specifier is used, IPv6 address is displayed in full address format.

```console
$ macipr %X ::1
0000:0000:0000:0000:0000:0000:0000:0001

$ macipr %X 340282366920938463463374607431768211455
ffff:ffff:ffff:ffff:ffff:ffff:ffff:ffff
```

### Number

```console
$ macipr %n 12345
12345

$ macipr :%5n: 123
:  123:

$ macipr :%05n: 45
:00045:
```

### Multiple addresses

```console
$ macipr "%m, %i" 0 192.168.0.1
0:00:00:00:00:00, 192.168.0.1
```

## Range

### Start-end range

This type of range specifies start and end addresses with hyphen (`-`) separator.
Range includes both start and end addresses. End address can be less than start address.

```console
$ macipr %m 0-5
00:00:00:00:00:00
00:00:00:00:00:01
00:00:00:00:00:02
00:00:00:00:00:03
00:00:00:00:00:04
00:00:00:00:00:05

$ macipr %i 192.168.1.1-192.168.0.254
192.168.1.1
192.168.1.0
192.168.0.255
192.168.0.254
```

### Offset range

This type of range specifies start address and offset from it with plus (`+`) separator.
Offset should be a number. If offset is prefixed by minus `-`, it represents a negative number.

```console
$ macipr %m 01:02:03:04:05:06+3
01:02:03:04:05:06
01:02:03:04:05:07
01:02:03:04:05:08
01:02:03:04:05:09

$ macipr %i 172.16.0.1+4
172.16.0.1
172.16.0.2
172.16.0.3
172.16.0.4
172.16.0.5

$ macipr %i 10+-9
0.0.0.10
0.0.0.9
0.0.0.8
0.0.0.7
0.0.0.6
0.0.0.5
0.0.0.4
0.0.0.3
0.0.0.2
0.0.0.1
```

### Multiple ranges

If multiple ranges are specified, printing continues for the longest range.
Shorter ranges are looped as many times as necessary.

```console
$ macipr "%m, %i, %i" 0+9 192.168.0.1 10.0.0.1-10.0.0.5
00:00:00:00:00:00, 192.168.0.1, 10.0.0.1
00:00:00:00:00:01, 192.168.0.1, 10.0.0.2
00:00:00:00:00:02, 192.168.0.1, 10.0.0.3
00:00:00:00:00:03, 192.168.0.1, 10.0.0.4
00:00:00:00:00:04, 192.168.0.1, 10.0.0.5
00:00:00:00:00:05, 192.168.0.1, 10.0.0.1
00:00:00:00:00:06, 192.168.0.1, 10.0.0.2
00:00:00:00:00:07, 192.168.0.1, 10.0.0.3
00:00:00:00:00:08, 192.168.0.1, 10.0.0.4
00:00:00:00:00:09, 192.168.0.1, 10.0.0.5
```
