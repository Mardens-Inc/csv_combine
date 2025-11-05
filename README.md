# CSV Combine

A smart CSV and Excel file combiner that intelligently groups files by header compatibility and merges them with automatic column alignment.

## Features

- **Multi-format Support**: Reads CSV and Excel files (.csv, .xlsx, .xls, .xlsm, .xlsb, .ods)
- **Smart Header Grouping**: Automatically groups files based on column header compatibility
- **Intelligent Merging**: Merges files with similar headers (≥50% overlap) into a single output
- **Column Alignment**: Automatically aligns columns and fills missing values with empty strings
- **Flexible Header Handling**: Handles files with extra, missing, or reordered columns
- **Recursive Directory Search**: Automatically finds all compatible files in a directory
- **Detailed Logging**: Provides clear feedback about which files are being processed

## Installation

### Prerequisites

- Rust 1.70 or higher
- Cargo (comes with Rust)

### Building from Source

```bash
# Clone the repository
git clone <repository-url>
cd csv_combine

# Build the project
cargo build --release

# The binary will be available at target/release/csv_combine
```

## Usage

### Basic Usage

```bash
# Process all CSV/Excel files in the current directory
csv_combine

# Process files in a specific directory
csv_combine /path/to/directory

# Process a single file
csv_combine /path/to/file.csv
```

### How It Works

The program uses intelligent header compatibility detection:

1. **Read all files** and extract their headers
2. **Group files** by header compatibility (≥50% column overlap)
3. **Merge headers** within each group into a superset of all columns
4. **Align data** by mapping rows to the merged header (missing columns filled with empty strings)
5. **Write output** files with descriptive names based on header hash

### Output Files

- **Multiple compatible files**: `combined_{hash}.csv`
  - Contains merged data from all files in the group
  - Headers are the union of all columns

- **Single unique file**: `single_{hash}.csv`
  - Contains data from a file with unique headers
  - No other files share ≥50% column overlap

### Examples

#### Example 1: Files with Same Headers

**Input:**
```
file1.csv: Name, Age, City
file2.csv: Name, Age, City
```

**Output:**
```
combined_abc123.csv: Name, Age, City
  - All rows from file1 and file2 combined
```

#### Example 2: Files with Extra Columns

**Input:**
```
file1.csv: Name, Age
file2.csv: Name, Age, City
file3.csv: Name, Age, City, Country
```

**Compatibility:**
- Overlap: 2 common columns out of 4 total = 50%+ ✓
- All three files are compatible!

**Output:**
```
combined_def456.csv: Name, Age, City, Country
  - file1 rows: Name, Age filled; City, Country empty
  - file2 rows: Name, Age, City filled; Country empty
  - file3 rows: All columns filled
```

#### Example 3: Files with Partial Overlap

**Input:**
```
file1.csv: Name, Age, City
file2.csv: Name, Age, Country
```

**Compatibility:**
- Common columns: Name, Age (2)
- Total unique columns: Name, Age, City, Country (4)
- Overlap: 2/4 = 50% ✓

**Output:**
```
combined_ghi789.csv: Name, Age, City, Country
  - file1 rows: Name, Age, City filled; Country empty
  - file2 rows: Name, Age, Country filled; City empty
```

#### Example 4: Completely Different Headers

**Input:**
```
employees.csv: Name, Age, Department
products.csv: Product, Price, SKU
```

**Compatibility:**
- Common columns: 0
- Overlap: 0/6 = 0% ✗

**Output:**
```
single_jkl012.csv: Name, Age, Department (employees data)
single_mno345.csv: Product, Price, SKU (products data)
```

#### Example 5: Low Overlap (Incompatible)

**Input:**
```
file1.csv: Name, Age, City
file2.csv: Name, Product, Price
```

**Compatibility:**
- Common columns: Name (1)
- Total unique columns: 5
- Overlap: 1/5 = 20% ✗

**Output:**
```
single_pqr678.csv: Name, Age, City
single_stu901.csv: Name, Product, Price
```

## Compatibility Rules

Files are considered **compatible** if they share ≥50% of their columns:

```
Overlap Percentage = (Common Columns) / (Total Unique Columns)

Compatible:   Overlap ≥ 50%  →  Files merged together
Incompatible: Overlap < 50%  →  Files separated
```

### Why 50%?

This threshold ensures:
- ✓ Files with the same columns are always merged
- ✓ Files with a few extra columns are merged
- ✓ Files with minor variations are merged
- ✗ Files with fundamentally different schemas are separated

## Logging

The program provides detailed debug logging:

```
[INFO] Searching for files in: /path/to/directory
[INFO] Found 5 files to process
[INFO] Reading: file1.csv
[INFO] Found 2 compatible header groups
[INFO] Processing group with merged headers: Name, Age, City (3 files)
[INFO]   - Including: file1.csv (headers: Name, Age)
[INFO]   - Including: file2.csv (headers: Name, Age, City)
[INFO]   - Including: file3.csv (headers: Name, Age, City)
[INFO] Created: combined_abc123.csv (3 files, 150 data rows)
[INFO] Processing complete! Created 2 output files
```

## Development

### Running Tests

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run a specific test
cargo test test_headers_are_compatible
```

### Test Coverage

The project includes 26 comprehensive tests covering:
- File path validation
- CSV and Excel reading
- Header compatibility detection
- Header merging logic
- Row mapping with missing columns
- Complex scenarios with quotes and special characters

### Project Structure

```
csv_combine/
├── src/
│   └── main.rs           # Main application code
├── Cargo.toml            # Project dependencies
├── Cargo.lock            # Locked dependencies
└── README.md             # This file
```

## Technical Details

### Supported File Types

- **CSV**: `.csv` files (with proper quote and comma handling)
- **Excel**: `.xlsx`, `.xls`, `.xlsm`, `.xlsb` (reads first sheet)
- **OpenDocument**: `.ods` (reads first sheet)

### Dependencies

- `csv` - CSV reading and writing
- `calamine` - Excel file support
- `anyhow` - Error handling
- `walkdir` - Directory traversal
- `log` + `pretty_env_logger` - Logging
- `tokio` - Async runtime
- `system-pause` - User interaction

## Limitations

- Only processes the first sheet of Excel workbooks
- Files must have headers in the first row
- Column matching is case-sensitive
- Large files are processed in memory (consider RAM usage)

## Troubleshooting

**Problem**: "No CSV or Excel files found"
- Check that your directory contains `.csv` or Excel files
- Verify the file extensions are correct

**Problem**: Files not being combined
- Check the header overlap percentage
- Files need ≥50% column overlap to be compatible
- Use debug logging to see compatibility decisions

**Problem**: Missing data in output
- This is expected! Files with fewer columns will have empty values for missing columns
- Check the merged header to see all available columns

## Contributing

Contributions are welcome! Please feel free to submit issues or pull requests.
