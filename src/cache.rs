use std::path::PathBuf;

use rand::{ distributions::Alphanumeric, Rng as _ };

const NAME: &str = "bookcorpus";

#[derive(Clone, Debug)]
pub struct Cache {
    path: PathBuf,
}

impl Cache {
    /// Creates a new cache object location
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    /// Creates a new cache object location
    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    pub(crate) fn temp_path(&self) -> PathBuf {
        let mut path = self.path().clone();
        path.push("tmp");
        std::fs::create_dir_all(&path).ok();

        let s: String = rand
            ::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(7)
            .map(char::from)
            .collect();
        path.push(s);
        path.to_path_buf()
    }
}

impl Default for Cache {
    fn default() -> Self {
        let mut path = match std::env::var("DATASET_HOME") {
            Ok(home) => home.into(),
            Err(_) => {
                let mut cache = dirs::home_dir().expect("Cache directory cannot be found");
                cache.push(".cache");
                cache.push("datasets");
                cache
            }
        };
        path.push(NAME);
        Self::new(path)
    }
}
