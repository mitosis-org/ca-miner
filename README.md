# 🚀 CA Miner - High-Performance Contract Address Miner

A blazing-fast Rust-based tool for mining Ethereum contract addresses with specific prefixes or postfixes using CREATE2 and CREATE3 deployment patterns.

## ✨ Features

- 🔥 **High Performance**: Multi-threaded parallel processing with optimal batch sizes
- 🎯 **Dual Patterns**: Support for both CREATE2 and CREATE3 address generation
- 🎨 **Pattern Matching**: Mine addresses with custom prefixes, postfixes, or both
- 🔤 **Case Sensitivity**: Support for both case-sensitive (EIP-55) and case-insensitive matching
- 📊 **Real-time Progress**: Beautiful colored output with live progress indicators
- ⚡ **Optimized Performance**: Up to millions of salts per second on modern hardware
- 🎲 **Flexible Salt Generation**: Sequential or random salt generation modes

## 🛠️ Installation

### Download Pre-built Binaries

Download the latest release for your platform from the [GitHub Releases](https://github.com/mitosis-org/ca-miner/releases) page:

- **Linux (x86_64)**: `ca-miner-linux-x86_64.tar.gz`
- **Linux (aarch64)**: `ca-miner-linux-aarch64.tar.gz`
- **Linux (musl)**: `ca-miner-linux-x86_64-musl.tar.gz`
- **macOS (Intel)**: `ca-miner-macos-x86_64.tar.gz`
- **macOS (Apple Silicon)**: `ca-miner-macos-aarch64.tar.gz`
- **Windows**: `ca-miner-windows-x86_64.exe.zip`

### Installation Steps

1. Download the appropriate binary for your platform
2. Extract the archive:
   ```bash
   tar -xzf ca-miner-linux-x86_64.tar.gz  # Linux/macOS
   unzip ca-miner-windows-x86_64.exe.zip  # Windows
   ```
3. Make it executable (Linux/macOS):
   ```bash
   chmod +x ca-miner
   ```
4. Run the tool:
   ```bash
   ./ca-miner --help
   ```

### Build from Source

Requirements:

- Rust 1.70+ installed ([rustup.rs](https://rustup.rs/))

```bash
git clone https://github.com/mitosis-org/ca-miner.git
cd ca-miner
cargo build --release
```

The binary will be available at `target/release/ca-miner`.

## 🚀 Quick Start

### CREATE2 Mining

Mine a CREATE2 address with "dead" prefix:

```bash
./ca-miner create2 \
  0x4e59b44847b379578588920cA78FbF26c0B4956C \
  0x1234567890123456789012345678901234567890123456789012345678901234 \
  dead \
  --max-iterations 1000000
```

### CREATE3 Mining

Mine a CREATE3 address with "cafe" postfix:

```bash
./ca-miner create3 \
  0x4e59b44847b379578588920cA78FbF26c0B4956C \
  "https://example.com/init" \
  cafe \
  --postfix \
  --max-iterations 1000000
```

## 📖 Usage

### CREATE2 Mode

```bash
ca-miner create2 <FACTORY> <BYTECODE_HASH> <PREFIX> [OPTIONS]
```

**Arguments:**

- `FACTORY`: Factory contract address (e.g., `0x4e59b44847b379578588920cA78FbF26c0B4956C`)
- `BYTECODE_HASH`: 32-byte bytecode hash in hex format (e.g., `0x1234...`)
- `PREFIX`: Desired address prefix in hex (e.g., `dead`, `cafe`)

### CREATE3 Mode

```bash
ca-miner create3 <FACTORY> <URL> <PREFIX> [OPTIONS]
```

**Arguments:**

- `FACTORY`: Factory contract address
- `URL`: Initialization URL string
- `PREFIX`: Desired address prefix in hex

### Options

| Option                        | Description                              | Default          |
| ----------------------------- | ---------------------------------------- | ---------------- |
| `--start-salt <SALT>`         | Starting salt value                      | `0`              |
| `--max-iterations <N>`        | Maximum iterations to try                | `10,000,000,000` |
| `--batch-size <SIZE>`         | Processing batch size                    | `100,000`        |
| `--random`                    | Use random salts instead of sequential   | `false`          |
| `--case-sensitive`            | Use EIP-55 checksum matching             | `false`          |
| `--postfix`                   | Match postfix instead of prefix          | `false`          |
| `--postfix-pattern <PATTERN>` | Pattern for dual prefix+postfix matching | -                |

## 🎯 Examples

### Basic Prefix Mining

```bash
# Mine addresses starting with "dead"
./ca-miner create2 \
  0x4e59b44847b379578588920cA78FbF26c0B4956C \
  0x1234567890123456789012345678901234567890123456789012345678901234 \
  dead
```

### Case-Sensitive Matching (EIP-55)

```bash
# Mine with proper Ethereum checksum
./ca-miner create2 \
  0x4e59b44847b379578588920cA78FbF26c0B4956C \
  0x1234567890123456789012345678901234567890123456789012345678901234 \
  DeaD \
  --case-sensitive
```

### Postfix Mining

```bash
# Mine addresses ending with "beef"
./ca-miner create2 \
  0x4e59b44847b379578588920cA78FbF26c0B4956C \
  0x1234567890123456789012345678901234567890123456789012345678901234 \
  beef \
  --postfix
```

### Dual Pattern Matching

```bash
# Mine addresses with both prefix "dead" AND postfix "beef"
./ca-miner create2 \
  0x4e59b44847b379578588920cA78FbF26c0B4956C \
  0x1234567890123456789012345678901234567890123456789012345678901234 \
  dead \
  --postfix-pattern beef
```

### Random Salt Generation

```bash
# Use random salts for better distribution
./ca-miner create2 \
  0x4e59b44847b379578588920cA78FbF26c0B4956C \
  0x1234567890123456789012345678901234567890123456789012345678901234 \
  cafe \
  --random \
  --max-iterations 5000000
```

### Performance Tuning

```bash
# Optimize for your hardware
./ca-miner create2 \
  0x4e59b44847b379578588920cA78FbF26c0B4956C \
  0x1234567890123456789012345678901234567890123456789012345678901234 \
  dead \
  --batch-size 500000 \
  --max-iterations 100000000
```

## 📊 Performance

Typical performance on modern hardware:

| Hardware                    | Performance    |
| --------------------------- | -------------- |
| Apple M1 Pro (10 cores)     | ~40M salts/sec |
| Intel i7-10700K (8 cores)   | ~25M salts/sec |
| AMD Ryzen 7 5800X (8 cores) | ~30M salts/sec |

Performance scales with:

- CPU core count
- Memory bandwidth
- Pattern complexity (longer patterns = slower)
- Case sensitivity (EIP-55 is slower)

## 🔧 Technical Details

### CREATE2 Address Generation

```
address = keccak256(0xff + factory + salt + keccak256(bytecode))[12:]
```

### CREATE3 Address Generation

Uses Solady's CREATE3 implementation:

1. Deploy proxy contract via CREATE2
2. Deploy actual contract via CREATE from proxy (nonce=1)

### Optimization Features

- **Parallel Processing**: Utilizes all CPU cores via Rayon
- **Batch Processing**: Optimal batch sizes for cache efficiency
- **Fast Hex Matching**: Custom hex comparison without string allocation
- **Memory Efficient**: Minimal allocations in hot paths

## 🎨 Output Features

The miner provides beautiful, colored terminal output with:

- 🚀 Colored headers and status messages
- 📊 Real-time progress indicators with spinning animations
- ⚡ Live performance metrics (salts/sec)
- 🎯 Formatted results with proper spacing
- 📈 Performance summaries

## 🛡️ Safety & Security

- **Memory Safe**: Written in Rust with zero unsafe code
- **Overflow Protection**: All arithmetic operations are checked
- **Input Validation**: Comprehensive validation of all inputs
- **Error Handling**: Graceful error handling with descriptive messages

## 🧪 Testing

Run the test suite:

```bash
cargo test
```

Run with optimizations:

```bash
cargo test --release
```

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request. For major changes, please open an issue first to discuss what you would like to change.

### Development Setup

```bash
git clone https://github.com/mitosis-org/ca-miner.git
cd ca-miner
cargo build
cargo test
```

### Performance Profiling

```bash
# Profile with flamegraph
cargo flamegraph --bin ca-miner -- create2 <args>

# Benchmark with criterion
cargo bench
```

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- [Alloy](https://github.com/alloy-rs/alloy) - Ethereum library for Rust
- [Rayon](https://github.com/rayon-rs/rayon) - Parallel processing
- [Clap](https://github.com/clap-rs/clap) - Command line parsing
- [Solady](https://github.com/Vectorized/solady) - CREATE3 implementation reference

---

⭐ **Star this repository if you find it useful!**

For questions, issues, or feature requests, please open an issue on GitHub.
