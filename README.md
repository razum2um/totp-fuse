# TOTP FUSE

A FUSE (Filesystem in Userspace) implementation that provides TOTP (Time-based One-Time Password) codes as "live-updated" files.

## Usage

Mount the TOTP filesystem with the following command:

```bash
totp-fuse -m /path/to/mountpoint -f filename -u username -s secret
```

## Example

```bash
totp-fuse --mountpoint ~/totp --filename mycode --username myuser --secret JBSWY3DPEHPK3PXP
```

This will create a file at `~/totp/mycode` that contains the username and the current TOTP code. The code inside the file will update automatically every 30 seconds.

## Unmounting

After stopping the process (at least on macOS), you still also need:

```bash
umount ~/totp
```

## Features

- Mount standard TOTP codes as virtual files
- Real-time code generation
- Simple command-line interface

## Prerequisites

- Rust toolchain (latest stable version)
- FUSE development libraries
  - On macOS: `brew install macfuse`
  - On Linux: `sudo apt-get install libfuse-dev` (Ubuntu/Debian)
  - On FreeBSD: `pkg install fusefs-libs`

## License

MIT

## Rationale

I had a special legacy tool that a frequent periodic re-authentication and could only read files. While tools like `oathtool` exist, they require either piping to stdin (not always supported) or shell-outs. This filesystem approach works with any application that can read files.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
