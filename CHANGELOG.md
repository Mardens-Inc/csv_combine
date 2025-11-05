# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2025-01-XX

### Added
- Initial release of CSV Combine
- Smart header grouping algorithm with 50% overlap threshold
- Multi-format file support (CSV, Excel, ODS)
- Automatic column alignment and merging
- Intelligent handling of:
  - Files with extra columns
  - Files with missing columns
  - Files with reordered columns
  - Files with completely different schemas
- Recursive directory traversal
- Comprehensive logging system
- Hash-based output file naming
- 26 unit and integration tests
- Complete documentation (README.md)
- Project metadata and keywords

### Features
- `headers_are_compatible()` - Calculates column overlap percentage
- `merge_headers()` - Creates superset of all columns while preserving order
- `map_rows_to_header()` - Maps data from old header to new header format
- `generate_header_hash()` - Creates unique identifier for header combinations
- Support for CSV files with quotes and special characters
- Support for Excel workbooks (first sheet only)
- Automatic empty string filling for missing columns

### Documentation
- Comprehensive README with 5 detailed examples
- Release notes with usage examples
- Inline code documentation
- Test coverage documentation
- Troubleshooting guide

### Testing
- Path validation tests
- CSV reading tests (including empty files and special characters)
- Excel file reading tests
- Header compatibility detection tests
- Header merging tests
- Row mapping tests (including edge cases)
- Integration tests for complete workflows

[0.1.0]: https://github.com/your-username/csv_combine/releases/tag/v0.1.0
