# oax (OpenAPI executor)

A commmand-line OpenAPI execution client.

[![crates.io](https://img.shields.io/crates/v/oax)](https://crates.io/crates/oax)

## Installation

To install the latest version of the crate from [crates.io](https://crates.io/crates/oax), run:

```sh
$ cargo install oax
```

## Usage

```
oax --api-url <url> --spec-url <url> <method> <endpoint> [--param <name=value> ...]
```

### Arguments

- `<method>`: Method to execute (required).
- `<endpoint>`: Endpoint to call (required).

### Methods

- `get` (alias: `g`): Execute GET request.
- `post` (alias: `p`): Execute POST request.
- `put` (alias: `u`): Execute PUT request.
- `delete` (alias: `d`): Execute DELETE request.
- `options` (alias: `o`): Execute OPTIONS request.
- `head` (alias: `h`): Execute HEAD request.
- `patch` (alias: `a`): Execute PATCH request.
- `trace` (alias: `t`): Execute TRACE request.

### Global options

- `-a`, `--api-url`: Set URL to API server (required).
- `-s`, `--spec-url`: Set URL to specification (required).
- `-h`, `--help`: Print help.
- `-v`, `--version`: Print version.

### Method options

- `-p`, `--param`: Add request parameter (multiple).

## Changelog

For a release history, see [CHANGELOG.md](https://codeberg.org/frofor/oax/src/branch/main/doc/CHANGELOG.md).

## License

This crate is distributed under the terms of MIT License.

See [LICENSE](https://codeberg.org/frofor/oax/src/branch/main/LICENSE) for details.
