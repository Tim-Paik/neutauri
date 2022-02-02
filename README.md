# Neutauri

[English](README.md) / [中文](README.zh-CN.md)

Under construction...

## Compilation Guide

### Linux

#### Dependency installation

Archlinux:
```shell
$ sudo pacman -Syu
$ sudo pacman -S --needed webkit2gtk base-devel curl wget openssl gtk3
```

#### Compile

```shell
$ git clone https://github.com/Tim-Paik/neutauri.git
$ cd neutauri
$ cargo build --release --bin neutauri_runtime
$ cargo build --release --bin neutauri_bundler
```
Then you can find the `neutauri_bundler` executable in the `target/release` directory.

### Windows

#### Dependency installation

**You need to install Visual Studio to use Rust**, please install it yourself.

[Install Rust](https://www.rust-lang.org/zh-CN/tools/install)

[Install Webview2](https://developer.microsoft.com/microsoft-edge/webview2)

#### Compile

```posh
PS C:\SomePath> git clone https://github.com/Tim-Paik/neutauri.git
PS C:\SomePath> cd .\neutauri\
PS C:\SomePath> cargo build --release --bin neutauri_runtime
PS C:\SomePath> cargo build --release --bin neutauri_bundler
```
Then you can find the `neutauri_bundler.exe` executable in the `.\target\release\` folder.

## License

[MPL2.0 License](LICENSE)