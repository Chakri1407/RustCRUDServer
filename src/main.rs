mod database;
mod handlers;
mod models;
mod utils;
mod constants;

use crate::database::set_database;
use crate::handlers::handle_client;
use dotenv::dotenv;
use std::env;
use std::net::TcpListener;

fn main() {
    dotenv().ok();
    let database_url = match env::var("DATABASE_URL") {
        Ok(url) => url,
        Err(_) => {
            println!("Error: DATABASE_URL must be set in environment");
            return;
        }
    };

    if let Err(e) = set_database(&database_url) {
        println!("Error setting up database: {}", e);
        return;
    }

    let listener = TcpListener::bind(format!("0.0.0.0:8080")).unwrap();
    println!("Server listening on port 8080");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let db_url = database_url.clone();
                handle_client(stream, &db_url);
            }
            Err(e) => {
                println!("Error: {}", e);
            }
        }
    }
}
