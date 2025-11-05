# CSV Combine - Quick Start Guide

## Installation

```bash
# Download from releases or build from source
cargo build --release
```

## Basic Usage

```bash
csv_combine [DIRECTORY_OR_FILE]
```

## Examples

### 1. Process Current Directory
```bash
csv_combine
```

### 2. Process Specific Directory
```bash
csv_combine /path/to/csv/files
```

### 3. Process Single File
```bash
csv_combine data/employees.csv
```

## How File Grouping Works

### ✅ Compatible (≥50% column overlap) → MERGED
```
file1.csv: A, B, C
file2.csv: A, B, D
→ combined_hash.csv: A, B, C, D
```

### ❌ Incompatible (<50% overlap) → SEPARATED
```
file1.csv: A, B
file2.csv: X, Y
→ single_hash1.csv: A, B
→ single_hash2.csv: X, Y
```

## Output Files

| Pattern | Description |
|---------|-------------|
| `combined_{hash}.csv` | Multiple compatible files merged |
| `single_{hash}.csv` | Single file with unique headers |

## Supported Formats

- ✅ `.csv` - CSV files
- ✅ `.xlsx` - Excel 2007+
- ✅ `.xls` - Excel 97-2003
- ✅ `.xlsm` - Excel with macros
- ✅ `.xlsb` - Excel binary
- ✅ `.ods` - OpenDocument

## Column Handling

| Scenario | Behavior |
|----------|----------|
| Extra columns | Added to merged header |
| Missing columns | Filled with empty strings |
| Reordered columns | Automatically mapped correctly |

## Quick Compatibility Check

```
Overlap = Common Columns ÷ Total Unique Columns

Examples:
• [A,B] + [A,B,C]     = 2÷3 = 66% ✅
• [A,B,C] + [A,B,D]   = 2÷4 = 50% ✅
• [A,B] + [C,D]       = 0÷4 = 0%  ❌
```

## Common Issues

**No files found?**
- Check file extensions
- Verify directory path

**Files not combining?**
- Check column overlap (must be ≥50%)
- Verify column names match (case-sensitive)

**Missing data in output?**
- Expected for files with fewer columns
- Empty strings fill missing values

## Advanced

### Running Tests
```bash
cargo test
```

### Build from Source
```bash
git clone <repo-url>
cd csv_combine
cargo build --release
./target/release/csv_combine
```

## Documentation

📖 [README.md](README.md) - Full documentation
📋 [CHANGELOG.md](CHANGELOG.md) - Version history
📝 [RELEASE_NOTES.md](RELEASE_NOTES.md) - Detailed release info

## Support

🐛 [Report Issues](https://github.com/your-username/csv_combine/issues)
💡 [Request Features](https://github.com/your-username/csv_combine/issues/new)

---

**Version**: 0.1.0 | **License**: See LICENSE file | **Author**: Drew Chase
