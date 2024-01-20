# Furia Torrent Client

Furia is a simple BitTorrent client written in Rust. 
It's designed to be straightforward and efficient, downloading torrent files to your local machine with minimal configuration.

## Usage

To use Furia, simply pass the path to a `.torrent` file as an argument:

```
furia ./torrent.file
```


Furia will then download the data contained in the torrent to the same folder.

## Installation

To install Furia, you'll need to have Rust installed on your machine. You can download Rust from the official website: https://www.rust-lang.org/tools/install

Once you have Rust installed, you can clone this repository and build the project:

```
git clone https://github.com/ksnll/furia.git
cd furia
cargo build --release
```


The `furia` executable will be in the `target/release` directory. You can move it to a directory in your `PATH` for easier access.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

Furia is released under the MIT License. See the `LICENSE` file for more details.
