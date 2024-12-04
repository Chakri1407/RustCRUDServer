// Import required dependencies for PostgreSQL database connection, 
// error handling, TCP networking, I/O operations, and environment variable management
use postgres::{Client,NoTls};
use postgres::Error as PostgresError;
use std::net::{TcpListener,TcpStream};
use std::io::{Read,Write};
use std::env;
use dotenv::dotenv; 

#[macro_use]
extern crate serde_derive;

//Model : User struct with id,name, email
#[derive(Serialize,Deserialize)]
struct User {
    id: i32,
    name: String,
    email: String,
}

//CONSTANTS
const OK_RESPONSE:&str = "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n";
const NOT_FOUND:&str = "HTTP/1.1 404 Not Found\r\n\r\n";
const INTERNAL_SERVER_ERROR:&str = "HTTP/1.1 500 Internal Server Error\r\n\r\n";

//main function 
fn main() {
    // Load the .env file to access environment variables.
    // If the .env file doesn't exist or fails to load, `dotenv().ok()` ensures the program continues without crashing.
    dotenv().ok();
    
    // Retrieve the database URL from the environment variable `DATABASE_URL`.
    // If the variable is not set, print an error message and exit the program.
    let database_url = match env::var("DATABASE_URL") {
        Ok(url) => url,
        Err(_) => {
            println!("Error: DATABASE_URL must be set in environment");
            return;
        }
    };

    //set database
    if let Err(e) = set_database(&database_url) {
        println!("Error setting up database: {}", e);
        return;
    }

    //start server and print port
    let listener = TcpListener::bind(format!("0.0.0.0:8080")).unwrap();
    println!("Server listening on port 8080");

    //handle the client 
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

// handle_client function 
fn handle_client(mut stream: TcpStream, db_url: &str) {
    let mut buffer = [0; 1024];
    let mut request = String::new();
 
    match stream.read(&mut buffer){
        Ok(size) => {
            request.push_str(String::from_utf8_lossy(&buffer[..size]).as_ref());

            let (status_line, content) = match &*request {
                r if r.starts_with("POST /users") => handle_post_request(r, db_url),
                r if r.starts_with("GET /users/") => handle_get_request(r, db_url),
                r if r.starts_with("GET /users") => handle_get_all_request(r, db_url),
                r if r.starts_with("PUT /users") => handle_put_request(r, db_url),
                r if r.starts_with("DELETE /users/") => handle_delete_request(r, db_url),
                _ => (NOT_FOUND.to_string(),"404 Not Found".to_string()),
            };

            stream.write_all(format!("{}{}", status_line, content).as_bytes()).unwrap();
        }
        Err(e) => {
            println!("Error: {}", e);
        }
    }
}

// CONTROLLERS

//Handle_post_request function 
fn handle_post_request(request: &str, db_url: &str) -> (String, String){
    match (get_user_request_body(&request), Client::connect(db_url, NoTls)) {
        (Ok(user), Ok(mut client)) => {
            client.execute("INSERT INTO users (name, email) VALUES ($1, $2)", &[&user.name, &user.email]).unwrap();
            (OK_RESPONSE.to_string(), "user created".to_string())
        }
        _ => (INTERNAL_SERVER_ERROR.to_string(), "Error ".to_string()),
    }
}

//Handle_get_request function
fn handle_get_request(request: &str, db_url: &str) -> (String, String) {
    match (get_id(request).parse::<i32>(), Client::connect(db_url, NoTls)){
        (Ok(id), Ok(mut client)) => 
        match client.query_one("SELECT * FROM users WHERE id = $1", &[&id]) {
            Ok(row) => {
                let user = User {
                    id: row.get(0),
                    name: row.get(1),
                    email: row.get(2),
                };
                (OK_RESPONSE.to_string(), serde_json::to_string(&user).unwrap())
            }
            _ => (NOT_FOUND.to_string(), "User not found".to_string()),
        }
        _ => (INTERNAL_SERVER_ERROR.to_string(), "Error".to_string()),
    }
}

// handle get all request
fn handle_get_all_request(request: &str, db_url: &str) -> (String, String) {
    match Client::connect(db_url, NoTls) {
        Ok(mut client) => {
            let mut users = Vec::new();
            for row in client.query("SELECT * FROM users", &[]).unwrap() {
                users.push(User {
                    id: row.get(0),
                    name: row.get(1),
                    email: row.get(2),
                });
            }
            (OK_RESPONSE.to_string(), serde_json::to_string(&users).unwrap())
        }
        _ => (INTERNAL_SERVER_ERROR.to_string(), "Error".to_string()),
    }
} 

//Handle_put_request function
fn handle_put_request(request: &str, db_url: &str) -> (String, String) {
    match (get_id(&request).parse::<i32>(), get_user_request_body(&request), Client::connect(db_url, NoTls)) {
        (Ok(id), Ok(user), Ok(mut client)) => {
            client.execute("UPDATE users SET name = $1, email = $2 WHERE id = $3", &[&user.name, &user.email, &id]).unwrap();
            (OK_RESPONSE.to_string(), "User updated".to_string())
        }
        _ => (INTERNAL_SERVER_ERROR.to_string(), "Error".to_string()),
    }
}

//Handle_delete_request function
fn handle_delete_request(request: &str, db_url: &str) -> (String, String) {
    match (get_id(&request).parse::<i32>(), Client::connect(db_url, NoTls)) {
        (Ok(id), Ok(mut client)) => {
            let rows_affected = client.execute("DELETE FROM users WHERE id = $1", &[&id]).unwrap();
            if rows_affected == 0 {
                return (NOT_FOUND.to_string(), "User not found".to_string());
            }
            (OK_RESPONSE.to_string(), "User deleted".to_string())
        }
        _ => (INTERNAL_SERVER_ERROR.to_string(), "Error".to_string()),
    }
}

// set database function
fn set_database(db_url: &str) -> Result<(), PostgresError> {
    //Connect to database
    let mut client = Client::connect(db_url, NoTls)?;
    //Create table
    client.batch_execute("
        CREATE TABLE IF NOT EXISTS users (
            id SERIAL PRIMARY KEY,
            name VARCHAR NOT NULL,
            email VARCHAR NOT NULL
        )"
    )?;
    Ok(()) 
}

//get_id function 
fn get_id(request: &str) -> &str {
    request.split("/").nth(2).unwrap_or_default().split_whitespace().next().unwrap_or_default()
}

//deserialize user from request body with the id 
fn get_user_request_body(request: &str) -> Result<User, serde_json::Error> {
     serde_json::from_str(&request.split("\r\n\r\n").last().unwrap_or_default())
}