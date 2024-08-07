#[tokio::main]
async fn main() {
    let api = bookcorpus::api::Api::default();
    let res = api.download().await;

    match res {
        Ok(path) => println!("Downloaded {}", path.to_string_lossy()),
        Err(err) => println!("Could not download dataset {}", err),
    }
}
