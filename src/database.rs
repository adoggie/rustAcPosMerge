
use sqlx::mssql::{Mssql, MssqlRow};
use sqlx::Row;
use sqlx::{Connection, Result};

#[derive(Debug)]
struct PythonPosition {
    id: i32,
    position: f64,
    // Add more fields as needed
}

fn query_table() -> Result<()> {
    // Establish a connection to the MS SQL Server database
    let conn_str = "server=my_server;user=my_username;password=my_password;database=my_database";
    let mut conn = sqlx::mssql::MssqlConnection::connect(conn_str)?;

    // Execute the query
    let rows = sqlx::query("SELECT id, position FROM python_position")
        .fetch_all(&mut conn)?;

    // Iterate over the records
    for row in rows {
        let position = PythonPosition {
            id: row.try_get("id")?,
            position: row.try_get("position")?,
        };
        println!("Position: {:?}", position);
    }

    Ok(())
}

fn write_table() -> Result<()> {
    // Establish a connection to the MS SQL Server database
    let conn_str = "server=my_server;user=my_username;password=my_password;database=my_database";
    let mut conn = sqlx::mssql::MssqlConnection::connect(conn_str)?;

    // Create a new PythonPosition instance
    let new_position = PythonPosition {
        id: 1,
        position: 3.14,
        name: "John Doe".to_string(),
    };

    // Insert the new_position into the PythonPosition table
    sqlx::query("INSERT INTO PythonPosition (id, position, name) VALUES (?, ?, ?)")
        .bind(new_position.id)
        .bind(new_position.position)
        .bind(new_position.name)
        .execute(&mut conn)?;

    Ok(())
}

fn msql_main() {
    match query_python_positions() {
        Ok(_) => {}
        Err(e) => eprintln!("Error: {}", e),
    }
}