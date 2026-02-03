use crate::container::DatabaseType;
use crate::seed::models::Row;
use anyhow::Result;

/// Build batch INSERT statement optimized for the database type
pub fn build_batch_insert(
    db_type: DatabaseType,
    table: &str,
    columns: &[String],
    rows: &[Row],
) -> Result<String> {
    if rows.is_empty() {
        return Ok(String::new());
    }

    match db_type {
        DatabaseType::Postgres => build_postgres_insert(table, columns, rows),
        DatabaseType::MySQL => build_mysql_insert(table, columns, rows),
        DatabaseType::SQLServer => build_sqlserver_insert(table, columns, rows),
    }
}

/// Build PostgreSQL INSERT statement
fn build_postgres_insert(table: &str, columns: &[String], rows: &[Row]) -> Result<String> {
    let mut sql = format!(
        "INSERT INTO {} ({}) VALUES ",
        escape_identifier(table),
        columns
            .iter()
            .map(|c| escape_identifier(c))
            .collect::<Vec<_>>()
            .join(", ")
    );

    let values: Vec<String> = rows
        .iter()
        .map(|row| {
            let vals = columns
                .iter()
                .map(|col| {
                    row.get(col)
                        .map(|v| escape_value(v))
                        .unwrap_or_else(|| "NULL".to_string())
                })
                .collect::<Vec<_>>()
                .join(", ");
            format!("({})", vals)
        })
        .collect();

    sql.push_str(&values.join(", "));
    sql.push(';');

    Ok(sql)
}

/// Build MySQL INSERT statement
fn build_mysql_insert(table: &str, columns: &[String], rows: &[Row]) -> Result<String> {
    // MySQL uses similar syntax to PostgreSQL for multi-row inserts
    let mut sql = format!(
        "INSERT INTO {} ({}) VALUES ",
        escape_identifier_mysql(table),
        columns
            .iter()
            .map(|c| escape_identifier_mysql(c))
            .collect::<Vec<_>>()
            .join(", ")
    );

    let values: Vec<String> = rows
        .iter()
        .map(|row| {
            let vals = columns
                .iter()
                .map(|col| {
                    row.get(col)
                        .map(|v| escape_value(v))
                        .unwrap_or_else(|| "NULL".to_string())
                })
                .collect::<Vec<_>>()
                .join(", ");
            format!("({})", vals)
        })
        .collect();

    sql.push_str(&values.join(", "));
    sql.push(';');

    Ok(sql)
}

/// Build SQL Server INSERT statement
fn build_sqlserver_insert(table: &str, columns: &[String], rows: &[Row]) -> Result<String> {
    let mut sql = format!(
        "INSERT INTO {} ({}) VALUES ",
        escape_identifier_sqlserver(table),
        columns
            .iter()
            .map(|c| escape_identifier_sqlserver(c))
            .collect::<Vec<_>>()
            .join(", ")
    );

    let values: Vec<String> = rows
        .iter()
        .map(|row| {
            let vals = columns
                .iter()
                .map(|col| {
                    row.get(col)
                        .map(|v| escape_value(v))
                        .unwrap_or_else(|| "NULL".to_string())
                })
                .collect::<Vec<_>>()
                .join(", ");
            format!("({})", vals)
        })
        .collect();

    sql.push_str(&values.join(", "));
    sql.push(';');

    Ok(sql)
}

/// Escape identifier for PostgreSQL (double quotes)
fn escape_identifier(name: &str) -> String {
    format!("\"{}\"", name.replace('"', "\"\""))
}

/// Escape identifier for MySQL (backticks)
fn escape_identifier_mysql(name: &str) -> String {
    format!("`{}`", name.replace('`', "``"))
}

/// Escape identifier for SQL Server (square brackets)
fn escape_identifier_sqlserver(name: &str) -> String {
    format!("[{}]", name.replace(']', "]]"))
}

/// Escape value (for all databases)
fn escape_value(value: &str) -> String {
    // Try parsing as number or boolean - no quotes needed
    if value.parse::<i64>().is_ok()
        || value.parse::<f64>().is_ok()
        || value.eq_ignore_ascii_case("true")
        || value.eq_ignore_ascii_case("false")
        || value.eq_ignore_ascii_case("null")
    {
        return value.to_string();
    }

    // Check if it's a timestamp in RFC3339 format
    if value.contains('T') && (value.contains('Z') || value.contains('+') || value.contains('-')) {
        // Likely a timestamp, wrap in quotes
        return format!("'{}'", value.replace('\'', "''"));
    }

    // String value - escape single quotes
    format!("'{}'", value.replace('\'', "''"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn make_row(data: Vec<(&str, &str)>) -> Row {
        data.iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    #[test]
    fn test_postgres_insert_single_row() {
        let columns = vec!["id".to_string(), "name".to_string()];
        let rows = vec![make_row(vec![("id", "1"), ("name", "Alice")])];

        let sql = build_postgres_insert("users", &columns, &rows).unwrap();
        assert_eq!(sql, "INSERT INTO \"users\" (\"id\", \"name\") VALUES (1, 'Alice');");
    }

    #[test]
    fn test_postgres_insert_multiple_rows() {
        let columns = vec!["id".to_string(), "name".to_string()];
        let rows = vec![
            make_row(vec![("id", "1"), ("name", "Alice")]),
            make_row(vec![("id", "2"), ("name", "Bob")]),
        ];

        let sql = build_postgres_insert("users", &columns, &rows).unwrap();
        assert_eq!(
            sql,
            "INSERT INTO \"users\" (\"id\", \"name\") VALUES (1, 'Alice'), (2, 'Bob');"
        );
    }

    #[test]
    fn test_mysql_insert() {
        let columns = vec!["id".to_string(), "name".to_string()];
        let rows = vec![make_row(vec![("id", "1"), ("name", "Alice")])];

        let sql = build_mysql_insert("users", &columns, &rows).unwrap();
        assert_eq!(sql, "INSERT INTO `users` (`id`, `name`) VALUES (1, 'Alice');");
    }

    #[test]
    fn test_sqlserver_insert() {
        let columns = vec!["id".to_string(), "name".to_string()];
        let rows = vec![make_row(vec![("id", "1"), ("name", "Alice")])];

        let sql = build_sqlserver_insert("users", &columns, &rows).unwrap();
        assert_eq!(sql, "INSERT INTO [users] ([id], [name]) VALUES (1, 'Alice');");
    }

    #[test]
    fn test_escape_value_numbers() {
        assert_eq!(escape_value("42"), "42");
        assert_eq!(escape_value("3.14"), "3.14");
        assert_eq!(escape_value("-100"), "-100");
    }

    #[test]
    fn test_escape_value_booleans() {
        assert_eq!(escape_value("true"), "true");
        assert_eq!(escape_value("false"), "false");
        assert_eq!(escape_value("TRUE"), "TRUE");
        assert_eq!(escape_value("FALSE"), "FALSE");
    }

    #[test]
    fn test_escape_value_strings() {
        assert_eq!(escape_value("hello"), "'hello'");
        assert_eq!(escape_value("it's"), "'it''s'");
        assert_eq!(escape_value("O'Brien"), "'O''Brien'");
    }

    #[test]
    fn test_escape_value_timestamps() {
        assert_eq!(
            escape_value("2024-01-01T00:00:00Z"),
            "'2024-01-01T00:00:00Z'"
        );
        assert_eq!(
            escape_value("2024-01-01T00:00:00+00:00"),
            "'2024-01-01T00:00:00+00:00'"
        );
    }

    #[test]
    fn test_batch_insert_all_databases() {
        let columns = vec!["id".to_string(), "name".to_string()];
        let rows = vec![
            make_row(vec![("id", "1"), ("name", "Alice")]),
            make_row(vec![("id", "2"), ("name", "Bob")]),
        ];

        // Test all three database types
        let pg_sql = build_batch_insert(DatabaseType::Postgres, "users", &columns, &rows).unwrap();
        assert!(pg_sql.contains("INSERT INTO"));
        assert!(pg_sql.contains("\"users\""));

        let mysql_sql = build_batch_insert(DatabaseType::MySQL, "users", &columns, &rows).unwrap();
        assert!(mysql_sql.contains("INSERT INTO"));
        assert!(mysql_sql.contains("`users`"));

        let mssql_sql =
            build_batch_insert(DatabaseType::SQLServer, "users", &columns, &rows).unwrap();
        assert!(mssql_sql.contains("INSERT INTO"));
        assert!(mssql_sql.contains("[users]"));
    }

    #[test]
    fn test_empty_rows() {
        let columns = vec!["id".to_string()];
        let rows = vec![];

        let sql = build_batch_insert(DatabaseType::Postgres, "users", &columns, &rows).unwrap();
        assert_eq!(sql, "");
    }
}
