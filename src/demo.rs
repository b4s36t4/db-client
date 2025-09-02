use crate::database::{ConnectionConfig, DatabasePool, DatabaseType};
use anyhow::Result;

pub async fn create_demo_database() -> Result<()> {
    let config = ConnectionConfig {
        name: "Demo SQLite Database".to_string(),
        database_type: DatabaseType::SQLite,
        connection_string: "sqlite:demo.db".to_string(),
        ssl_config: None,
    };

    let pool = DatabasePool::connect(&config).await?;

    // Create demo tables
    let create_users_table = r#"
        CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            email TEXT UNIQUE NOT NULL,
            age INTEGER,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )
    "#;

    let create_orders_table = r#"
        CREATE TABLE IF NOT EXISTS orders (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id INTEGER NOT NULL,
            product_name TEXT NOT NULL,
            quantity INTEGER NOT NULL DEFAULT 1,
            price DECIMAL(10,2) NOT NULL,
            order_date DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (user_id) REFERENCES users(id)
        )
    "#;

    let create_categories_table = r#"
        CREATE TABLE IF NOT EXISTS categories (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL UNIQUE,
            description TEXT
        )
    "#;

    // Execute table creation
    if let Err(e) = pool.execute_query(create_users_table).await {
        eprintln!("Error creating users table: {}", e);
        return Err(e);
    }
    if let Err(e) = pool.execute_query(create_orders_table).await {
        eprintln!("Error creating orders table: {}", e);
        return Err(e);
    }
    if let Err(e) = pool.execute_query(create_categories_table).await {
        eprintln!("Error creating categories table: {}", e);
        return Err(e);
    }

    // Insert demo data
    let insert_users = r#"
        INSERT OR REPLACE INTO users (id, name, email, age) VALUES
        (1, 'John Doe', 'john@example.com', 30),
        (2, 'Jane Smith', 'jane@example.com', 25),
        (3, 'Bob Johnson', 'bob@example.com', 35),
        (4, 'Alice Brown', 'alice@example.com', 28),
        (5, 'Charlie Wilson', 'charlie@example.com', 42)
    "#;

    let insert_orders = r#"
        INSERT OR REPLACE INTO orders (id, user_id, product_name, quantity, price) VALUES
        (1, 1, 'Laptop', 1, 999.99),
        (2, 1, 'Mouse', 2, 25.50),
        (3, 2, 'Keyboard', 1, 75.00),
        (4, 3, 'Monitor', 1, 299.99),
        (5, 2, 'Webcam', 1, 89.99),
        (6, 4, 'Headphones', 1, 149.99),
        (7, 5, 'Tablet', 1, 399.99),
        (8, 3, 'Phone', 1, 699.99)
    "#;

    let insert_categories = r#"
        INSERT OR REPLACE INTO categories (id, name, description) VALUES
        (1, 'Electronics', 'Electronic devices and gadgets'),
        (2, 'Computers', 'Computer hardware and accessories'),
        (3, 'Audio', 'Audio equipment and accessories'),
        (4, 'Mobile', 'Mobile phones and accessories')
    "#;

    if let Err(e) = pool.execute_query(insert_users).await {
        eprintln!("Error inserting users: {}", e);
        return Err(e);
    }
    if let Err(e) = pool.execute_query(insert_orders).await {
        eprintln!("Error inserting orders: {}", e);
        return Err(e);
    }
    if let Err(e) = pool.execute_query(insert_categories).await {
        eprintln!("Error inserting categories: {}", e);
        return Err(e);
    }

    println!("Demo database created successfully with sample data!");
    Ok(())
}
