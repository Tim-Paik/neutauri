# Neutauri

[English](README.md) / [中文](README.zh-CN.md)

施工中...

## 编译指南

### Linux

#### 依赖安装

Archlinux:
```shell
$ sudo pacman -Syu
$ sudo pacman -S --needed webkit2gtk base-devel curl wget openssl gtk3
```

#### 编译

```shell
$ git clone https://github.com/Tim-Paik/neutauri.git
$ cd neutauri
$ cargo build --release --bin neutauri_runtime
$ cargo build --release --bin neutauri_bundler
```
然后你可以在 `target/release` 目录下找到 `neutauri_bundler` 可执行文件。

### Windows

#### 依赖安装

**你需要安装 Visual Studio 来使用 Rust**，请自行安装。

[安装 Rust](https://www.rust-lang.org/zh-CN/tools/install)

[安装 Webview2](https://developer.microsoft.com/microsoft-edge/webview2)

#### 编译

```posh
PS C:\SomePath> git clone https://github.com/Tim-Paik/neutauri.git
PS C:\SomePath> cd .\neutauri\
PS C:\SomePath> cargo build --release --bin neutauri_runtime
PS C:\SomePath> cargo build --release --bin neutauri_bundler
```
然后你可以在 `.\target\release\` 文件夹下找到 `neutauri_bundler.exe` 可执行文件。

## 开源协议

[MPL2.0 License](LICENSE)