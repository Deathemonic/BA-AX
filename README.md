<div>
    <img src=".github/resources/archive.png" alt="logo" />
</div>

# Blue Archive - Asset Extractor
A tool and library that extracts **Blue Archive** assets.

## Install

### Release
You can download the latest pre-build binaries at [Releases](https://github.com/Deathemonic/BA-AX/releases)

[Windows](https://github.com/Deathemonic/BA-AX/releases/latest/download/baax-windows-x86_64.zip) | [Linux](https://github.com/Deathemonic/BA-AX/releases/latest/download/baax-linux-x86_64.zip) | [MacOS](https://github.com/Deathemonic/BA-AX/releases/latest/download/baax-macos-aarch64.zip)

### Cargo
```shell
cargo install --git "https://github.com/Deathemonic/BA-AX" --locked baax-cli
```

## Usage

```shell
# Extracting MediaResources
baax extract media --input BGM.zip --output ./output

# Extracting Global PC MediaResources
baax extract media --input BGM.molru --output ./output

# Extracting TableBundles
baax extract table --input Excel.zip --output ./output

# Extracting DB using a SQLCipher key (Hex)
baax extract table --input ExcelDB.db --output ./output --key "0000..."


# Extracting MediaResources explicit format (available: auto (default) | zip | pack)
baax extract media --input BGM.nolru --output ./output --format pack

# Extracting Excel with FlatBuffer decoding
baax extract table --input Excel.zip --output ./output --flatbuffer gl-123.flat

# Extracting Excel with explicit format (available: json (default) | xlsx)
baax extract table --input ExcelDB.db --output ./output --flatbuffer gl-123.flat --key "0000..." --format xlsx


# Converting a raw flatbuffer bytes
baax convert flatbuffer --input file.bytes --output ./output --flat gl-123.flat

# Converting raw to custom format (available: json (default) | xlsx)
baax convert flatbuffer --input file.bytes --output ./output --flat gl-123.flat --format xlsx

# Convert pack files back to regular zip files
baax convert pack --input file.molru --output ./output

```

### Flat Files

You can find `.flat` files here: https://github.com/Deathemonic/BA-TG/releases.
You need to match the `.flat` files and version of excel you are decoding if not it will result on a inaccurate or fail dump.

To know which one is the correct flat you can use this schema:

```
japan-1.71.449178.flat
  ^         ^
Region  Game Version
```


## Building

1. Install [rustup](https://rustup.rs)
2. Clone this repository
```sh
git clone https://github.com/Deathemonic/BA-AX
cd BA-AX
```
3. Build using `cargo`
```sh
cargo build
```

## Library
```toml
baax = { git = "https://github.com/Deathemonic/BA-AX" }
```

### FAQ

Why it doesn't do repack?
> Sole purpose of `BA-AX` just just for extracting.

Why this doesn't provide a way to fetch SQLCipher keys?
> Doing that requires to call to the official game server which this project is not aiming to do.

### Other Projects
- [BA-AD](https://github.com/Deathemonic/BA-AD): A tool and library that downloads the latest **Blue Archive** assets.
- [BA-MU](https://github.com/Deathemonic/BA-MU): A tool that re-dump AssetBundle for **Blue Archive**.
- [BA-FB](https://github.com/Deathemonic/BA-FB): A tool for dumping and generating **Blue Archive** flatbuffers.
- [BA-BR](https://github.com/Deathemonic/BA-BR): A tool that repacks AssetBundle for **Blue Archive**. 
- [BA-CY](https://github.com/Deathemonic/BA-CY): A library for handling **Blue Archive** Cryptography.


### Acknowledgement
- [respectZ/blue-archive-viewer](https://github.com/respectZ/blue-archive-viewer)

---

<sub>**Copyright** - Blue Archive is a registered trademark of NAT GAMES Co., Ltd., NEXON Korea Corp., and Yostar, Inc.
This project is not affiliated with, endorsed by, or connected to NAT GAMES Co., Ltd., NEXON Korea Corp., NEXON GAMES
Co., Ltd., IODivision, Yostar, Inc., or any of their subsidiaries or affiliates. All game assets, content, and materials
are copyrighted by their respective owners and are used for informational and educational purposes only.</sub>
