#[tokio::main]
async fn main() {
    let api = bookcorpus::api::Api::default();
    let res = api.download().await;
    let files = api.unzip().unwrap();

    println!("Downloaded dataset with files {files:?}");

    match res {
        Ok(path) => println!("Downloaded {}", path.to_string_lossy()),
        Err(err) => println!("Could not download dataset {}", err),
    }
}
