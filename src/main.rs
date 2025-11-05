use anyhow::Result;
use calamine::{open_workbook_auto, Reader};
use log::*;
use std::collections::{HashMap, HashSet};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use system_pause::pause;

#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::env_logger::builder()
        .format_timestamp(None)
        .filter_level(LevelFilter::Debug)
        .init();

    let input_path = PathBuf::from(
        std::env::args().nth(1).unwrap_or(
            std::env::current_dir()
                .unwrap_or(PathBuf::from("./"))
                .to_string_lossy()
                .to_string(),
        ),
    );

    info!("Searching for files in: {}", input_path.display());
    let files = get_files(&input_path)?;
    info!("Found {} files to process", files.len());

    if files.is_empty() {
        warn!("No CSV or Excel files found!");
        return Ok(());
    }

    // Read all file headers and contents
    let mut file_data: Vec<(PathBuf, Vec<Vec<String>>)> = Vec::new();

    for file_path in files {
        info!("Reading: {}", file_path.display());
        match get_file_contents(&file_path) {
            Ok(data) => {
                if data.is_empty() {
                    warn!("File is empty: {}", file_path.display());
                    continue;
                }
                file_data.push((file_path, data));
            }
            Err(e) => {
                warn!("Failed to read file {}: {}", file_path.display(), e);
                continue;
            }
        }
    }

    // Group files by header compatibility (>= 50% overlap)
    let mut groups: Vec<Vec<usize>> = Vec::new();

    for i in 0..file_data.len() {
        let mut added_to_group = false;

        // Try to add to an existing compatible group
        for group in &mut groups {
            let group_representative = group[0];
            let header1 = &file_data[i].1[0];
            let header2 = &file_data[group_representative].1[0];

            if headers_are_compatible(header1, header2) {
                group.push(i);
                added_to_group = true;
                break;
            }
        }

        // Create a new group if not compatible with any existing group
        if !added_to_group {
            groups.push(vec![i]);
        }
    }

    info!("Found {} compatible header groups", groups.len());

    // Process each group
    let mut files_created = 0;
    for group in groups {
        // Collect all headers from the group and merge them
        let mut all_headers: Vec<Vec<String>> = Vec::new();
        for &file_idx in &group {
            all_headers.push(file_data[file_idx].1[0].clone());
        }

        let merged_header = merge_headers(&all_headers);
        let header_hash = generate_header_hash(&merged_header);

        info!(
            "Processing group with merged headers: {} ({} files)",
            merged_header.join(", "),
            group.len()
        );

        if group.len() == 1 {
            // Single file - copy with merged header (should be same as original)
            let file_idx = group[0];
            let (file_path, data) = &file_data[file_idx];
            let output_filename = format!("single_{}.csv", header_hash);
            info!("Copying single file: {}", file_path.display());

            let mapped_rows = map_rows_to_header(&data[0], &merged_header, &data[1..]);
            write_combined_csv(&output_filename, &merged_header, &mapped_rows)?;

            info!("Created: {} (1 file, {} data rows)", output_filename, data.len() - 1);
            files_created += 1;
        } else {
            // Multiple compatible files - combine them
            let output_filename = format!("combined_{}.csv", header_hash);
            info!("Combining {} compatible files into: {}", group.len(), output_filename);

            let mut all_data: Vec<Vec<String>> = Vec::new();

            for &file_idx in &group {
                let (file_path, data) = &file_data[file_idx];
                info!("  - Including: {} (headers: {})", file_path.display(), data[0].join(", "));

                // Map rows from this file's header to the merged header
                let mapped_rows = map_rows_to_header(&data[0], &merged_header, &data[1..]);
                all_data.extend(mapped_rows);
            }

            write_combined_csv(&output_filename, &merged_header, &all_data)?;
            info!(
                "Created: {} ({} files, {} data rows)",
                output_filename,
                group.len(),
                all_data.len()
            );
            files_created += 1;
        }
    }

    info!("Processing complete! Created {} output files", files_created);
    pause!("All CSV files have been processed successfully, press enter to continue.");

    Ok(())
}

fn generate_header_hash(header: &[String]) -> String {
    let mut hasher = DefaultHasher::new();
    header.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

fn headers_are_compatible(header1: &[String], header2: &[String]) -> bool {
    let set1: HashSet<&String> = header1.iter().collect();
    let set2: HashSet<&String> = header2.iter().collect();

    let intersection: HashSet<_> = set1.intersection(&set2).collect();
    let union: HashSet<_> = set1.union(&set2).collect();

    if union.is_empty() {
        return false;
    }

    let overlap_percentage = (intersection.len() as f64) / (union.len() as f64);

    // Files are compatible if they have >= 50% overlap
    overlap_percentage >= 0.5
}

fn merge_headers(headers: &[Vec<String>]) -> Vec<String> {
    let mut merged = Vec::new();
    let mut seen = HashSet::new();

    // Add all unique headers while preserving order
    // Start with the first header to maintain column order preference
    for header_set in headers {
        for col in header_set {
            if !seen.contains(col) {
                seen.insert(col.clone());
                merged.push(col.clone());
            }
        }
    }

    merged
}

fn map_rows_to_header(
    old_header: &[String],
    new_header: &[String],
    rows: &[Vec<String>],
) -> Vec<Vec<String>> {
    // Create a mapping from old column names to their indices
    let old_col_map: HashMap<&String, usize> = old_header
        .iter()
        .enumerate()
        .map(|(idx, col)| (col, idx))
        .collect();

    let mut mapped_rows = Vec::new();

    for row in rows {
        let mut new_row = vec![String::new(); new_header.len()];

        for (new_idx, col_name) in new_header.iter().enumerate() {
            if let Some(&old_idx) = old_col_map.get(col_name) {
                if old_idx < row.len() {
                    new_row[new_idx] = row[old_idx].clone();
                }
            }
            // If column doesn't exist in old header, leave it as empty string
        }

        mapped_rows.push(new_row);
    }

    mapped_rows
}

fn get_file_contents(path: impl AsRef<Path>) -> Result<Vec<Vec<String>>> {
    let path = path.as_ref();
    if let Some(extension) = path.extension() {
        return match extension.to_string_lossy().to_lowercase().as_ref() {
            "csv" => read_csv_file(path),
            "xlsx" | "xls" | "xlsm" | "xlsb" | "ods" => read_excel_file(path),
            _ => Err(anyhow::anyhow!(
                "Unsupported file extension: {:?}",
                extension
            )),
        };
    }
    Err(anyhow::Error::msg("File has no extension"))
}

fn read_csv_file(path: impl AsRef<Path>) -> Result<Vec<Vec<String>>> {
    let mut reader = csv::Reader::from_path(path)?;
    let mut data = Vec::new();

    // Read and include the header
    if let Ok(headers) = reader.headers() {
        let header_row: Vec<String> = headers.iter().map(|s| s.to_string()).collect();
        data.push(header_row);
    }

    // Read all data rows
    for result in reader.records() {
        let record = result?;
        let row: Vec<String> = record.iter().map(|s| s.to_string()).collect();
        data.push(row);
    }

    Ok(data)
}

fn read_excel_file(path: impl AsRef<Path>) -> Result<Vec<Vec<String>>> {
    let mut workbook = open_workbook_auto(path.as_ref())?;

    // Get the first sheet
    let sheet_names = workbook.sheet_names().to_vec();
    if sheet_names.is_empty() {
        return Err(anyhow::anyhow!("Excel file has no sheets"));
    }

    let sheet_name = &sheet_names[0];
    info!("Reading sheet: {}", sheet_name);

    let range = workbook.worksheet_range(sheet_name)?;

    let mut data = Vec::new();
    for row in range.rows() {
        let row_data: Vec<String> = row.iter().map(|cell| cell.to_string()).collect();
        data.push(row_data);
    }

    Ok(data)
}

fn write_combined_csv(output_path: &str, header: &[String], data: &[Vec<String>]) -> Result<()> {
    let mut writer = csv::Writer::from_path(output_path)?;

    // Write header
    writer.write_record(header)?;

    // Write all data rows
    for row in data {
        writer.write_record(row)?;
    }

    writer.flush()?;
    Ok(())
}

fn get_files(search_path: impl AsRef<Path>) -> Result<Vec<PathBuf>> {
    let search_path = search_path.as_ref();
    if search_path.is_dir() {
        let mut files = Vec::new();
        for entry in walkdir::WalkDir::new(search_path) {
            let entry = entry?;
            let path = entry.path();
            if is_path_valid(path) {
                files.push(PathBuf::from(path));
            }
        }
        Ok(files)
    } else if search_path.is_file() {
        Ok(vec![search_path.to_path_buf()])
    } else {
        Err(anyhow::anyhow!(
            "{:?} is not a file nor is a directory, I don't know how you got here?!?!?!?",
            search_path
        ))
    }
}

fn is_path_valid(file_path: impl AsRef<Path>) -> bool {
    let file_path = file_path.as_ref();
    if file_path.is_file()
        && let Some(extension) = file_path.extension()
    {
        let ext = extension.to_string_lossy().to_lowercase();
        return matches!(
            ext.as_ref(),
            "csv" | "xlsx" | "xls" | "xlsm" | "xlsb" | "ods"
        );
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_is_path_valid_csv() {
        let test_dir = TempDir::new().unwrap();
        let csv_path = test_dir.path().join("test.csv");
        fs::File::create(&csv_path).unwrap();

        assert!(is_path_valid(&csv_path));
    }

    #[test]
    fn test_is_path_valid_xlsx() {
        let test_dir = TempDir::new().unwrap();
        let xlsx_path = test_dir.path().join("test.xlsx");
        fs::File::create(&xlsx_path).unwrap();

        assert!(is_path_valid(&xlsx_path));
    }

    #[test]
    fn test_is_path_valid_invalid_extension() {
        let test_dir = TempDir::new().unwrap();
        let txt_path = test_dir.path().join("test.txt");
        fs::File::create(&txt_path).unwrap();

        assert!(!is_path_valid(&txt_path));
    }

    #[test]
    fn test_is_path_valid_directory() {
        let test_dir = TempDir::new().unwrap();
        assert!(!is_path_valid(test_dir.path()));
    }

    #[test]
    fn test_read_csv_file() {
        let test_dir = TempDir::new().unwrap();
        let csv_path = test_dir.path().join("test.csv");

        let csv_content = "Name,Age,City\nAlice,30,New York\nBob,25,Los Angeles\nCharlie,35,Chicago";
        let mut file = fs::File::create(&csv_path).unwrap();
        file.write_all(csv_content.as_bytes()).unwrap();

        let result = read_csv_file(&csv_path).unwrap();

        assert_eq!(result.len(), 4);
        assert_eq!(result[0], vec!["Name", "Age", "City"]);
        assert_eq!(result[1], vec!["Alice", "30", "New York"]);
        assert_eq!(result[2], vec!["Bob", "25", "Los Angeles"]);
        assert_eq!(result[3], vec!["Charlie", "35", "Chicago"]);
    }

    #[test]
    fn test_read_csv_file_empty() {
        let test_dir = TempDir::new().unwrap();
        let csv_path = test_dir.path().join("empty.csv");
        fs::File::create(&csv_path).unwrap();

        let result = read_csv_file(&csv_path).unwrap();
        // An empty CSV file still has an empty header row
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].len(), 0);
    }

    #[test]
    fn test_write_combined_csv() {
        let test_dir = TempDir::new().unwrap();
        let output_path = test_dir.path().join("output.csv");

        let header = vec!["Name".to_string(), "Age".to_string(), "City".to_string()];
        let data = vec![
            vec!["Alice".to_string(), "30".to_string(), "New York".to_string()],
            vec!["Bob".to_string(), "25".to_string(), "Los Angeles".to_string()],
        ];

        write_combined_csv(
            output_path.to_str().unwrap(),
            &header,
            &data,
        ).unwrap();

        assert!(output_path.exists());

        let result = read_csv_file(&output_path).unwrap();
        assert_eq!(result.len(), 3);
        assert_eq!(result[0], header);
        assert_eq!(result[1], data[0]);
        assert_eq!(result[2], data[1]);
    }

    #[test]
    fn test_get_files_from_directory() {
        let test_dir = TempDir::new().unwrap();

        // Create test files
        fs::File::create(test_dir.path().join("file1.csv")).unwrap();
        fs::File::create(test_dir.path().join("file2.xlsx")).unwrap();
        fs::File::create(test_dir.path().join("file3.txt")).unwrap(); // Should be ignored

        let files = get_files(test_dir.path()).unwrap();

        assert_eq!(files.len(), 2);
        assert!(files.iter().any(|f| f.file_name().unwrap() == "file1.csv"));
        assert!(files.iter().any(|f| f.file_name().unwrap() == "file2.xlsx"));
        assert!(!files.iter().any(|f| f.file_name().unwrap() == "file3.txt"));
    }

    #[test]
    fn test_get_files_single_file() {
        let test_dir = TempDir::new().unwrap();
        let csv_path = test_dir.path().join("single.csv");
        fs::File::create(&csv_path).unwrap();

        let files = get_files(&csv_path).unwrap();

        assert_eq!(files.len(), 1);
        assert_eq!(files[0], csv_path);
    }

    #[test]
    fn test_get_file_contents_csv() {
        let test_dir = TempDir::new().unwrap();
        let csv_path = test_dir.path().join("test.csv");

        let csv_content = "A,B,C\n1,2,3\n4,5,6";
        let mut file = fs::File::create(&csv_path).unwrap();
        file.write_all(csv_content.as_bytes()).unwrap();

        let result = get_file_contents(&csv_path).unwrap();

        assert_eq!(result.len(), 3);
        assert_eq!(result[0], vec!["A", "B", "C"]);
        assert_eq!(result[1], vec!["1", "2", "3"]);
        assert_eq!(result[2], vec!["4", "5", "6"]);
    }

    #[test]
    fn test_csv_with_quotes_and_commas() {
        let test_dir = TempDir::new().unwrap();
        let csv_path = test_dir.path().join("complex.csv");

        let csv_content = "Name,Description,Price\nProduct1,\"A product, with comma\",10.99\nProduct2,\"Another \"\"quoted\"\" item\",20.50";
        let mut file = fs::File::create(&csv_path).unwrap();
        file.write_all(csv_content.as_bytes()).unwrap();

        let result = read_csv_file(&csv_path).unwrap();

        assert_eq!(result.len(), 3);
        assert_eq!(result[0], vec!["Name", "Description", "Price"]);
        assert_eq!(result[1], vec!["Product1", "A product, with comma", "10.99"]);
        assert_eq!(result[2], vec!["Product2", "Another \"quoted\" item", "20.50"]);
    }

    #[test]
    fn test_generate_header_hash() {
        let header1 = vec!["Name".to_string(), "Age".to_string(), "City".to_string()];
        let header2 = vec!["Name".to_string(), "Age".to_string(), "City".to_string()];
        let header3 = vec!["Name".to_string(), "Age".to_string()];

        let hash1 = generate_header_hash(&header1);
        let hash2 = generate_header_hash(&header2);
        let hash3 = generate_header_hash(&header3);

        // Same headers should produce same hash
        assert_eq!(hash1, hash2);
        // Different headers should produce different hash
        assert_ne!(hash1, hash3);
        // Hash should be a hex string
        assert!(hash1.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_grouping_files_by_headers() {
        let test_dir = TempDir::new().unwrap();

        // Create files with same headers
        let file1_content = "Name,Age\nAlice,30\nBob,25";
        let file2_content = "Name,Age\nCharlie,35\nDave,40";

        let mut file1 = fs::File::create(test_dir.path().join("file1.csv")).unwrap();
        file1.write_all(file1_content.as_bytes()).unwrap();

        let mut file2 = fs::File::create(test_dir.path().join("file2.csv")).unwrap();
        file2.write_all(file2_content.as_bytes()).unwrap();

        let data1 = get_file_contents(test_dir.path().join("file1.csv")).unwrap();
        let data2 = get_file_contents(test_dir.path().join("file2.csv")).unwrap();

        // Both files should have the same header
        assert_eq!(data1[0], data2[0]);
        assert_eq!(data1[0], vec!["Name", "Age"]);
    }

    #[test]
    fn test_grouping_files_with_different_headers() {
        let test_dir = TempDir::new().unwrap();

        // Create files with different headers
        let file1_content = "Name,Age\nAlice,30";
        let file2_content = "Product,Price\nWidget,10.99";

        let mut file1 = fs::File::create(test_dir.path().join("file1.csv")).unwrap();
        file1.write_all(file1_content.as_bytes()).unwrap();

        let mut file2 = fs::File::create(test_dir.path().join("file2.csv")).unwrap();
        file2.write_all(file2_content.as_bytes()).unwrap();

        let data1 = get_file_contents(test_dir.path().join("file1.csv")).unwrap();
        let data2 = get_file_contents(test_dir.path().join("file2.csv")).unwrap();

        // Files should have different headers
        assert_ne!(data1[0], data2[0]);
        assert_eq!(data1[0], vec!["Name", "Age"]);
        assert_eq!(data2[0], vec!["Product", "Price"]);

        // Should generate different hashes
        let hash1 = generate_header_hash(&data1[0]);
        let hash2 = generate_header_hash(&data2[0]);
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_headers_are_compatible_exact_match() {
        let header1 = vec!["Name".to_string(), "Age".to_string()];
        let header2 = vec!["Name".to_string(), "Age".to_string()];

        assert!(headers_are_compatible(&header1, &header2));
    }

    #[test]
    fn test_headers_are_compatible_superset() {
        let header1 = vec!["Name".to_string(), "Age".to_string()];
        let header2 = vec!["Name".to_string(), "Age".to_string(), "City".to_string()];

        // 2 common out of 3 total = 66.7% overlap - should be compatible
        assert!(headers_are_compatible(&header1, &header2));
    }

    #[test]
    fn test_headers_are_compatible_partial_overlap() {
        let header1 = vec!["Name".to_string(), "Age".to_string(), "City".to_string()];
        let header2 = vec!["Name".to_string(), "Age".to_string(), "Country".to_string()];

        // 2 common out of 4 total = 50% overlap - should be compatible
        assert!(headers_are_compatible(&header1, &header2));
    }

    #[test]
    fn test_headers_are_not_compatible() {
        let header1 = vec!["Name".to_string(), "Age".to_string()];
        let header2 = vec!["Product".to_string(), "Price".to_string()];

        // 0 common out of 4 total = 0% overlap - not compatible
        assert!(!headers_are_compatible(&header1, &header2));
    }

    #[test]
    fn test_headers_are_not_compatible_low_overlap() {
        let header1 = vec!["Name".to_string(), "Age".to_string(), "City".to_string()];
        let header2 = vec!["Name".to_string(), "Product".to_string(), "Price".to_string()];

        // 1 common out of 5 total = 20% overlap - not compatible
        assert!(!headers_are_compatible(&header1, &header2));
    }

    #[test]
    fn test_merge_headers_identical() {
        let headers = vec![
            vec!["Name".to_string(), "Age".to_string()],
            vec!["Name".to_string(), "Age".to_string()],
        ];

        let merged = merge_headers(&headers);
        assert_eq!(merged, vec!["Name", "Age"]);
    }

    #[test]
    fn test_merge_headers_superset() {
        let headers = vec![
            vec!["Name".to_string(), "Age".to_string()],
            vec!["Name".to_string(), "Age".to_string(), "City".to_string()],
        ];

        let merged = merge_headers(&headers);
        assert_eq!(merged, vec!["Name", "Age", "City"]);
    }

    #[test]
    fn test_merge_headers_different_order() {
        let headers = vec![
            vec!["Name".to_string(), "Age".to_string(), "City".to_string()],
            vec!["Age".to_string(), "Name".to_string(), "Country".to_string()],
        ];

        let merged = merge_headers(&headers);
        // Should preserve order from first header, then add new columns
        assert_eq!(merged, vec!["Name", "Age", "City", "Country"]);
    }

    #[test]
    fn test_map_rows_to_header_same_headers() {
        let old_header = vec!["Name".to_string(), "Age".to_string()];
        let new_header = vec!["Name".to_string(), "Age".to_string()];
        let rows = vec![
            vec!["Alice".to_string(), "30".to_string()],
            vec!["Bob".to_string(), "25".to_string()],
        ];

        let mapped = map_rows_to_header(&old_header, &new_header, &rows);

        assert_eq!(mapped.len(), 2);
        assert_eq!(mapped[0], vec!["Alice", "30"]);
        assert_eq!(mapped[1], vec!["Bob", "25"]);
    }

    #[test]
    fn test_map_rows_to_header_with_new_columns() {
        let old_header = vec!["Name".to_string(), "Age".to_string()];
        let new_header = vec!["Name".to_string(), "Age".to_string(), "City".to_string()];
        let rows = vec![
            vec!["Alice".to_string(), "30".to_string()],
            vec!["Bob".to_string(), "25".to_string()],
        ];

        let mapped = map_rows_to_header(&old_header, &new_header, &rows);

        assert_eq!(mapped.len(), 2);
        assert_eq!(mapped[0], vec!["Alice", "30", ""]);
        assert_eq!(mapped[1], vec!["Bob", "25", ""]);
    }

    #[test]
    fn test_map_rows_to_header_reordered_columns() {
        let old_header = vec!["Name".to_string(), "Age".to_string(), "City".to_string()];
        let new_header = vec!["City".to_string(), "Name".to_string(), "Age".to_string()];
        let rows = vec![vec!["Alice".to_string(), "30".to_string(), "NYC".to_string()]];

        let mapped = map_rows_to_header(&old_header, &new_header, &rows);

        assert_eq!(mapped.len(), 1);
        assert_eq!(mapped[0], vec!["NYC", "Alice", "30"]);
    }

    #[test]
    fn test_map_rows_to_header_mixed_columns() {
        let old_header = vec!["Name".to_string(), "Age".to_string()];
        let new_header = vec![
            "Name".to_string(),
            "Age".to_string(),
            "City".to_string(),
            "Country".to_string(),
        ];
        let rows = vec![vec!["Alice".to_string(), "30".to_string()]];

        let mapped = map_rows_to_header(&old_header, &new_header, &rows);

        assert_eq!(mapped.len(), 1);
        assert_eq!(mapped[0], vec!["Alice", "30", "", ""]);
    }
}
