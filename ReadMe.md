# Rust CRUD API 

This project demonstrates a simple CRUD (Create, Read, Update, Delete) server built in Rust using TCP networking, PostgreSQL as the database, and Docker for containerization.


## Features

- CRUD Operations: Manage users with the following fields:
    - id (integer, auto-increment)
    - name (string)
    - email (string)
- Environment Configuration: Load configuration from .env file.
- PostgreSQL Integration: Database connection and query execution using the postgres crate.
- JSON Serialization/Deserialization: Use serde for JSON handling.
- Error Handling: Handle various errors including database issues and invalid requests.
- Dockerized Environment: Includes a Dockerfile for building the application and Docker Compose support for setting up the PostgreSQL database.

## Prerequisites

- Rust (1.75 or later) 
- Docker and Docker Compose
- PostgreSQL
- Git

## API Endpoints

- `POST /users` - Create a new user
- `GET /users` - Retrieve all users
- `GET /users/{id}` - Retrieve a specific user
- `PUT /users/{id}` - Update a user
- `DELETE /users/{id}` - Delete a user

## Setup and Installation

1. Clone the repository:
``` bash 
git clone <repository-url>
cd rust_crud_api
```
2. Create a .env file:
``` 
DATABASE_URL=postgresql://username:password@localhost:5432/database_name
``` 
Replace username, password, and database_name with your PostgreSQL credentials.

3. Build and Run the Containers: 
Ensure Docker is running on your machine.
``` 
docker-compose up -d 
```
4. Build and run the application:
```
cargo run
``` 
## Docker Configuration
### Docker Compose
The application uses Docker Compose to manage two services:

1. PostgreSQL Database:

- Image: postgres:latest

- Port: 5432

- Default credentials:

    - Username: postgres

    - Password: postgres

    - Database: rust_crud

2. Rust Application:

    - Custom Dockerfile

    - Port: 8080

    - Dependencies: Waits for database to be healthy

### Database Schema
``` 
CREATE TABLE IF NOT EXISTS users (
    id SERIAL PRIMARY KEY,
    name VARCHAR NOT NULL,
    email VARCHAR NOT NULL
)
``` 

### Dependencies

``` 
[dependencies]
postgres = "*"
serde = { version = "*", features = ["derive"] }
serde_json = "*"
dotenv = "*"
tokio = { version = "*", features = ["full"] }

``` 

### Error Handling

The API returns appropriate HTTP status codes: 
   - 200: Successful operation 
   - 404: Resource not found
   - 500: Internal server error

### Development
To run in development mode:
``` 
cargo run
```
To run tests:
```
cargo test
```

### Docker Commands
Build and start containers:
```
docker-compose up --build
```
Stop containers:
```
docker-compose down
```
View logs:
```
docker-compose logs -f
```
### Troubleshooting
1. Database Connection Issues:

- Verify PostgreSQL is running: docker ps

- Check database logs: docker-compose logs db

- Ensure correct DATABASE_URL in .env

2. Application Issues:

- Check application logs: docker-compose logs rustapp

- Verify port availability: Port 8080 should be free

- Ensure proper environment variables are set

## License
This project is licensed under the MIT License.
    