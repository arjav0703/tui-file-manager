mod structures;
use structures::Directory;

#[tokio::main]
async fn main() {
    println!("Hello, world!");

    let mut current_dir = Directory::new("tui-file-manager".to_string(), ".".to_string());

    current_dir.scan_and_add().await.unwrap();
    dbg!(current_dir);
}
