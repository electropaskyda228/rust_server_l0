use tokio_postgres::NoTls;
use tokio::sync::RwLock;
use clap::Parser;

mod database;
mod models;

use serde_json::json;
use axum::{Router, routing::{get, post}, extract::State, Json, response::IntoResponse, http::StatusCode};
use std::sync::Arc;
use models::Order;
use std::net::SocketAddr;
use log::{info, error};


#[derive(Parser, Debug)]
#[command(version, about, long_about=None)]
struct ConnectionArgs {
    // Server host
    #[arg(long)]
    server_host: String,

    // Server port
    #[arg(long)]
    server_port: u16,

    // Database user
    #[arg(long)]
    db_user: String,

    // Database user's password
    #[arg(long)]
    db_password: String,

    // Database's name
    #[arg(long)]
    db_name: String,

    // Database host
    #[arg(long)]
    db_host: String,

    // Database port
    #[arg(long)]
    db_port: u16
}


#[tokio::main]
async fn main() {

    // Make logging
    log4rs::init_file("log4rs.yaml", Default::default()).unwrap();

    // Parse command line
    let conn_args = ConnectionArgs::parse();

    // Make links to database from command line's information
    let db_url = format!(
        "postgres://{}:{}@{}:{}/{}", 
        conn_args.db_user, conn_args.db_password, conn_args.db_host, conn_args.db_port, conn_args.db_name
    );

    // Connect to the database
    let (client, connection) = tokio_postgres::connect(&db_url, NoTls).await.expect("Can't get connection with database");

    // Run a new asynchronous task for connection to the database
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            print!("Database connection error: {}", e);
        }
    });

    // Make object of connection (indeed it is structure)
    let connection_class = database::new(client);

    // Make our app by using Router
    let app = Router::new()
        .route("/add", post(add_order))
        .route("/get", get(get_orders))
        .with_state(Arc::new(RwLock::new(   
            OrdersState { 
                orders: match connection_class.get_all_orders().await {
                    Ok(orders) => {
                        info!("Successfully get orders: {:?}", orders);
                        orders
                    }
                    Err(e) => {
                        error!("Failed to get orders: {:?}", e);
                        Vec::new()
                    }
                },
                connection_class
            }
        )));


    // Make address of our server
    let addr = SocketAddr::from(([127, 0, 0, 1], conn_args.server_port));

    // Run server
    axum::Server::bind(&addr)
    .serve(app.into_make_service())
    .await
    .unwrap();


}
// Type of state for our cash
type OrdersStateType = Arc<RwLock<OrdersState>>;

// Struct for our cash
pub struct OrdersState{
    pub orders: Vec<Order>,
    pub connection_class: database::Connection,
}

// Function which would be called when user get address '/add'
pub async fn add_order(
    State(state): State<OrdersStateType>, 
    Json(order): Json<Order>
) -> impl IntoResponse {
    let mut state = state.write().await;
    
    match state.connection_class.insert_into_order(&order).await {
        Ok(_) => {
            info!("Order added successfully: {:?}", order);
            state.orders.push(order.clone());
            let pretty_json_order = serde_json::to_string_pretty(&order).unwrap();
            (StatusCode::OK, pretty_json_order)
        }
        Err(e) => {
            let error_response = json!({
                "success": false,
                "message": e.to_string(),
            });
            error!("Failed to add order: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, error_response.to_string())
        }
    }
}

// Function which would be called when user get address '/get'
async fn get_orders(State(state): State<OrdersStateType>) -> impl IntoResponse {
    let pretty_json_orders = serde_json::to_string_pretty(
        &state.read().await.orders
    ).unwrap();
    info!("Fetched all orders");
    (StatusCode::OK, pretty_json_orders)
}

