use std::path::PathBuf;
use tokio::fs;
use uuid::Uuid;

pub struct Storage {
    base_path: PathBuf,
}

impl Storage {
    pub fn new(base_path: PathBuf) -> Self {
        Self { base_path }
    }

    pub async fn save_file(
        &self,
        file_name: &str,
        content: &[u8],
    ) -> Result<PathBuf, std::io::Error> {
        let file_id = Uuid::new_v4();
        let file_path = self.base_path.join(format!("{}_{}", file_id, file_name));

        fs::write(&file_path, content).await?;
        Ok(file_path)
    }

    pub async fn get_file(&self, file_path: &PathBuf) -> Result<Vec<u8>, std::io::Error> {
        fs::read(file_path).await
    }

    pub async fn delete_file(&self, file_path: &PathBuf) -> Result<(), std::io::Error> {
        fs::remove_file(file_path).await
    }
}
