use dlms_client::connection_pool::{ConnectionPool, ConnectionKey};

#[tokio::main]
async fn main() {
    let pool = ConnectionPool::default_config();
    let key = ConnectionKey::tcp_hdlc("127.0.0.1".to_string(), 4059);

    let id = pool.register_connection(key.clone()).await;
    println!("After register:");
    let stats = pool.stats().await;
    println!("  total_connections: {}", stats.total_connections);
    println!("  active_connections: {}", stats.active_connections);
    println!("  idle_connections: {}", stats.idle_connections);

    let result = pool.mark_in_use(&key, id).await;
    println!("\nmark_in_use returned: {}", result);
    
    println!("After mark_in_use:");
    let stats = pool.stats().await;
    println!("  total_connections: {}", stats.total_connections);
    println!("  active_connections: {}", stats.active_connections);
    println!("  idle_connections: {}", stats.idle_connections);
}
