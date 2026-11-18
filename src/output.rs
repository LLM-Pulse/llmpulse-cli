use std::io::{self, IsTerminal};

use clap::ValueEnum;
use comfy_table::{Cell, ContentArrangement, Table, presets::UTF8_FULL_CONDENSED};
use serde_json::{Map, Value};

use crate::error::Result;

#[derive(Clone, Copy, Debug, Default, ValueEnum)]
pub enum OutputFormat {
    #[default]
    Json,
    Table,
    Csv,
}

pub fn print(value: &Value, format: OutputFormat) -> Result<()> {
    println!("{}", format_value(value, format)?);
    Ok(())
}

pub fn format_value(value: &Value, format: OutputFormat) -> Result<String> {
    match format {
        OutputFormat::Json => {
            if io::stdout().is_terminal() {
                Ok(serde_json::to_string_pretty(value)?)
            } else {
                Ok(serde_json::to_string(value)?)
            }
        }
        OutputFormat::Table => Ok(format_table(value)),
        OutputFormat::Csv => format_csv(value),
    }
}

pub fn extract_array(value: &Value) -> Option<&Vec<Value>> {
    if let Value::Array(rows) = value {
        return Some(rows);
    }
    let object = value.as_object()?;
    if let Some(rows) = object.get("data").and_then(Value::as_array) {
        return Some(rows);
    }
    object.values().find_map(Value::as_array)
}

pub fn display_payload(value: &Value) -> &Value {
    if let Some(data) = value.get("data") {
        data
    } else {
        value
    }
}

pub fn print_pagination_meta(value: &Value) {
    let Some(object) = value.as_object() else {
        return;
    };
    let Some(page) = object.get("page").and_then(Value::as_u64) else {
        return;
    };
    let Some(total) = object.get("total").and_then(Value::as_u64) else {
        return;
    };
    let per_page = object
        .get("per_page")
        .and_then(Value::as_u64)
        .unwrap_or(total.max(1));
    let pages = total.div_ceil(per_page);
    eprintln!("Page {page}/{pages} ({total} total)");
}

fn rows(value: &Value) -> Vec<Map<String, Value>> {
    let values = extract_array(value)
        .cloned()
        .unwrap_or_else(|| vec![value.clone()]);
    values
        .into_iter()
        .map(|value| match value {
            Value::Object(object) => object,
            value => Map::from_iter([("value".into(), value)]),
        })
        .collect()
}

fn format_table(value: &Value) -> String {
    let rows = rows(value);
    if rows.is_empty() {
        return "(no data)".into();
    }
    let columns = rows[0].keys().cloned().collect::<Vec<_>>();
    let mut table = Table::new();
    table
        .load_style(UTF8_FULL_CONDENSED)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(columns.iter().map(Cell::new));
    for row in rows {
        table.add_row(
            columns
                .iter()
                .map(|column| Cell::new(cell_string(row.get(column).unwrap_or(&Value::Null)))),
        );
    }
    table.to_string()
}

fn format_csv(value: &Value) -> Result<String> {
    let rows = rows(value);
    if rows.is_empty() {
        return Ok(String::new());
    }
    let columns = rows[0].keys().cloned().collect::<Vec<_>>();
    let mut writer = csv::Writer::from_writer(Vec::new());
    writer.write_record(
        columns
            .iter()
            .map(|column| sanitize_spreadsheet_cell(column.clone())),
    )?;
    for row in rows {
        writer.write_record(columns.iter().map(|column| {
            sanitize_spreadsheet_cell(cell_string(row.get(column).unwrap_or(&Value::Null)))
        }))?;
    }
    let bytes = writer.into_inner().map_err(|error| error.into_error())?;
    Ok(String::from_utf8_lossy(&bytes).trim_end().to_owned())
}

fn cell_string(value: &Value) -> String {
    match value {
        Value::Null => String::new(),
        Value::String(value) => value.clone(),
        Value::Bool(value) => value.to_string(),
        Value::Number(value) => value.to_string(),
        value => serde_json::to_string(value).unwrap_or_default(),
    }
}

// Mirrors Exports::CellSanitizer: a lone trigger character (the "-" blank
// placeholder) is inert and stays as it is.
const FORMULA_PREFIXES: [char; 6] = ['=', '+', '-', '@', '\t', '\r'];

fn sanitize_spreadsheet_cell(value: String) -> String {
    if value.starts_with(FORMULA_PREFIXES) && value.chars().nth(1).is_some() {
        format!("'{value}")
    } else {
        value
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn extracts_data_arrays_first() {
        let value = json!({"meta": [], "data": [{"id": 1}]});
        assert_eq!(extract_array(&value).unwrap()[0]["id"], 1);
    }

    #[test]
    fn csv_quotes_fields_and_blocks_formulas() {
        let value = json!([{"name": "=1+1", "text": "hello, world"}]);
        let csv = format_value(&value, OutputFormat::Csv).unwrap();
        assert!(csv.contains("'=1+1"));
        assert!(csv.contains("\"hello, world\""));
    }

    // Same rule as Exports::CellSanitizer on the server: the CLI builds its CSV
    // from JSON locally, so the server's output=csv never runs on this path.
    #[test]
    fn sanitizes_cells_like_the_server_cell_sanitizer() {
        let cases = [
            ("=1+1", "'=1+1"),
            ("+1", "'+1"),
            ("-2+3", "'-2+3"),
            ("@SUM(A1)", "'@SUM(A1)"),
            ("\t=1+1", "'\t=1+1"),
            ("\r=1+1", "'\r=1+1"),
            ("=ü", "'=ü"),
            ("=", "="),
            ("-", "-"),
            ("\t", "\t"),
            ("", ""),
            ("a=1", "a=1"),
            (" =1", " =1"),
            ("Zürich", "Zürich"),
        ];
        for (input, expected) in cases {
            assert_eq!(
                sanitize_spreadsheet_cell(input.to_owned()),
                expected,
                "{input:?}"
            );
        }
    }

    #[test]
    fn csv_keeps_control_prefixed_formulas_and_formula_headers_literal() {
        let value = json!([{
            "=cmd": "x",
            "note": "\t=HYPERLINK(\"http://x\")",
            "line": "\r=1+1",
            "dash": "-",
            "text": "Zürich, \"quoted\""
        }]);
        let csv = format_value(&value, OutputFormat::Csv).unwrap();
        let mut reader = csv::Reader::from_reader(csv.as_bytes());
        let headers = reader.headers().unwrap().clone();
        let record = reader.records().next().unwrap().unwrap();
        let cell = |name: &str| {
            let index = headers.iter().position(|header| header == name).unwrap();
            record.get(index).unwrap().to_owned()
        };

        assert!(
            headers.iter().any(|header| header == "'=cmd"),
            "{headers:?}"
        );
        assert_eq!(cell("note"), "'\t=HYPERLINK(\"http://x\")");
        assert_eq!(cell("line"), "'\r=1+1");
        assert_eq!(cell("dash"), "-");
        assert_eq!(cell("text"), "Zürich, \"quoted\"");
    }

    #[test]
    fn table_handles_empty_arrays() {
        assert_eq!(
            format_value(&json!([]), OutputFormat::Table).unwrap(),
            "(no data)"
        );
    }

    #[test]
    fn table_renders_utf8_full_condensed_with_wide_characters() {
        let value = json!([
            {"brand": "Zürich Café", "city": "東京", "mentions": 12},
            {"brand": "Acme", "city": null, "mentions": 3}
        ]);
        let expected = "\
┌─────────────┬──────┬──────────┐
│ brand       ┆ city ┆ mentions │
╞═════════════╪══════╪══════════╡
│ Zürich Café ┆ 東京 ┆ 12       │
│ Acme        ┆      ┆ 3        │
└─────────────┴──────┴──────────┘";
        assert_eq!(format_value(&value, OutputFormat::Table).unwrap(), expected);
    }
}
