# clipboard-herald

An application which rewrites URLs in your clipboard for you in order to generate pretty embeds. For example, `x.com` -> `fixupx.com`.

Currently only supports Windows.

## Configuration

Configuration can be found in `config.toml` with the following format:

```toml
[twitter] # a unique name for this rewrite
replace = "x.com" # the domain to be replaced
with = "fixupx.com" # the target domain to replace with
```

The `config.toml` file must be in the same directory as the executable.

## Building

To build in release config, first install [Rust](https://www.rust-lang.org/) and then run

```shell
$ cargo build --release
```

The executable can then be found at `target/release/clipboard-herald.exe`. 
