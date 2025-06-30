fn main() {
    println!("📊 Reports Service - Starting simple version");
    
    // Simple infinite loop to keep service running
    loop {
        std::thread::sleep(std::time::Duration::from_secs(10));
        println!("📊 Reports Service - Still running...");
    }
} 