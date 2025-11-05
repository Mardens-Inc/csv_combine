# Release Notes - CSV Combine v0.1.0

## 🎉 Initial Release

We're excited to announce the first release of **CSV Combine** - a smart CSV and Excel file combiner that intelligently groups files by header compatibility and merges them with automatic column alignment!

## 🚀 Key Features

### Smart Header Grouping
- **Automatic compatibility detection** using a 50% column overlap threshold
- Files with similar headers are automatically grouped and merged
- Files with incompatible headers are kept separate

### Multi-Format Support
- ✅ CSV files (`.csv`)
- ✅ Excel files (`.xlsx`, `.xls`, `.xlsm`, `.xlsb`)
- ✅ OpenDocument Spreadsheet (`.ods`)

### Intelligent Column Handling
- 🔄 **Automatic column alignment** - handles files with different column orders
- ➕ **Extra column support** - files with additional columns are merged seamlessly
- 📝 **Missing column handling** - empty values automatically filled for missing columns
- 🔀 **Reordered columns** - data correctly mapped regardless of column order

### User-Friendly Features
- 📁 Recursive directory search for all compatible files
- 📊 Detailed logging showing exactly what's happening
- 🏷️ Smart file naming using header signatures
- ⏸️ Pause on completion for review

## 📦 What's Included

### Core Functionality
- Smart file grouping algorithm
- Header compatibility detection
- Column merging and alignment
- Multi-format file reading (CSV & Excel)
- Automatic output file generation

### Quality Assurance
- ✅ **26 comprehensive tests** covering all major functionality
- ✅ 100% test pass rate
- ✅ Robust error handling
- ✅ Edge case coverage (empty files, special characters, quotes)

## 💾 Installation

### Pre-built Binaries
Download the appropriate binary for your platform from the [Releases](../../releases) page.

### Build from Source
```bash
# Clone the repository
git clone https://github.com/your-username/csv_combine.git
cd csv_combine

# Build with Cargo
cargo build --release

# Binary will be at target/release/csv_combine
```

## 🎯 Quick Start

```bash
# Process files in current directory
csv_combine

# Process files in a specific directory
csv_combine /path/to/data

# Process a single file
csv_combine /path/to/file.csv
```

## 📝 Usage Examples

### Example 1: Combining Files with Same Headers
```bash
# Input files:
#   employees_jan.csv: Name, Age, Department
#   employees_feb.csv: Name, Age, Department

csv_combine ./employees

# Output: combined_abc123.csv
#   Contains all rows from both files
```

### Example 2: Files with Extra Columns
```bash
# Input files:
#   old_data.csv:  Name, Age
#   new_data.csv:  Name, Age, City, Country

csv_combine ./data

# Output: combined_def456.csv with headers: Name, Age, City, Country
#   old_data rows: Name, Age filled; City, Country empty
#   new_data rows: All columns filled
```

### Example 3: Mixed Data Sources
```bash
# Input files:
#   customers.csv: Name, Email, Phone
#   products.csv:  Product, Price, SKU

csv_combine ./mixed_data

# Output: Two separate files (no overlap)
#   single_aaa111.csv: Customer data
#   single_bbb222.csv: Product data
```

## 🔧 Compatibility Rules

Files are merged when they share **≥50% of their columns**:

```
Overlap = Common Columns / Total Unique Columns

Examples:
  - [Name, Age] + [Name, Age, City]
    → 2/3 = 66.7% ✅ Compatible

  - [Name, Age, City] + [Name, Age, Country]
    → 2/4 = 50% ✅ Compatible

  - [Name, Age] + [Product, Price]
    → 0/4 = 0% ❌ Incompatible
```

## 📊 Output Files

The tool generates intelligently named output files:

- **`combined_{hash}.csv`** - Multiple compatible files merged together
- **`single_{hash}.csv`** - Single file with unique headers

The hash is based on the column headers, ensuring consistent naming across runs.

## 🧪 Testing

All functionality is thoroughly tested:

```bash
# Run all 26 tests
cargo test

# Tests cover:
#   ✅ File reading (CSV & Excel)
#   ✅ Header compatibility detection
#   ✅ Column merging logic
#   ✅ Row mapping with missing columns
#   ✅ Special characters and quotes
#   ✅ Edge cases (empty files, single files)
```

## ⚠️ Known Limitations

- Only processes the **first sheet** of Excel workbooks
- Files must have **headers in the first row**
- Column matching is **case-sensitive**
- Large files are processed **in-memory** (consider available RAM)

## 📚 Documentation

- [README.md](README.md) - Complete user guide with detailed examples
- [Cargo.toml](Cargo.toml) - Project configuration
- Code documentation via `cargo doc --open`

## 🛠️ Technical Stack

- **Language**: Rust 2024 edition
- **CSV Processing**: `csv` crate
- **Excel Support**: `calamine` crate
- **Error Handling**: `anyhow`
- **Directory Traversal**: `walkdir`
- **Logging**: `log` + `pretty_env_logger`
- **Async Runtime**: `tokio`

## 🐛 Bug Reports & Feature Requests

Found a bug or have a feature idea? Please [open an issue](../../issues) on GitHub!

## 📄 License

Copyright © 2024 Drew Chase

## 🙏 Acknowledgments

Special thanks to the Rust community and the maintainers of:
- The `csv` crate for robust CSV handling
- The `calamine` crate for Excel file support
- All other dependency maintainers

---

## 📅 Release Timeline

**Version 0.1.0** - Released 2025-01-XX
- Initial public release
- Smart header grouping algorithm
- Multi-format file support
- Comprehensive test suite

---

## 🔮 Future Roadmap

Potential features for future releases:
- [ ] Configuration file support
- [ ] Custom compatibility thresholds
- [ ] Multiple sheet processing for Excel
- [ ] Streaming mode for very large files
- [ ] GUI interface
- [ ] Custom output naming patterns
- [ ] Data validation and cleaning options

---

**Full Changelog**: [View on GitHub](../../compare/v0.0.0...v0.1.0)

**Download**: [Release Assets](../../releases/tag/v0.1.0)
