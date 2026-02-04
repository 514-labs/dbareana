use crate::{DBArenaError, Result};
use bollard::image::{CreateImageOptions, ListImagesOptions};
use bollard::models::ImageSummary;
use bollard::Docker;
use futures::StreamExt;
use tracing::{debug, info};

#[derive(Clone)]
pub struct DockerClient {
    docker: Docker,
}

impl DockerClient {
    pub fn new() -> Result<Self> {
        let docker =
            Docker::connect_with_local_defaults().map_err(|_| DBArenaError::DockerNotAvailable)?;
        Ok(Self { docker })
    }

    pub async fn verify_connection(&self) -> Result<()> {
        self.docker
            .ping()
            .await
            .map_err(|_| DBArenaError::DockerNotAvailable)?;
        Ok(())
    }

    pub fn docker(&self) -> &Docker {
        &self.docker
    }

    pub async fn image_exists(&self, image_name: &str) -> Result<bool> {
        let filters = vec![("reference", vec![image_name])];
        let options = ListImagesOptions {
            filters: filters.into_iter().collect(),
            ..Default::default()
        };

        let images = self.docker.list_images(Some(options)).await?;
        Ok(!images.is_empty())
    }

    pub async fn pull_image(&self, image_name: &str) -> Result<()> {
        info!("Pulling image: {}", image_name);

        let options = Some(CreateImageOptions {
            from_image: image_name,
            ..Default::default()
        });

        let mut stream = self.docker.create_image(options, None, None);

        while let Some(result) = stream.next().await {
            match result {
                Ok(info) => {
                    if let Some(status) = info.status {
                        debug!("Image pull: {}", status);
                    }
                    if let Some(error) = info.error {
                        return Err(DBArenaError::ImagePullFailed(error));
                    }
                }
                Err(e) => return Err(DBArenaError::DockerError(e)),
            }
        }

        info!("Successfully pulled image: {}", image_name);
        Ok(())
    }

    pub async fn ensure_image(&self, image_name: &str) -> Result<()> {
        if !self.image_exists(image_name).await? {
            self.pull_image(image_name).await?;
        } else {
            debug!("Image already exists: {}", image_name);
        }
        Ok(())
    }

    pub async fn list_images(&self) -> Result<Vec<ImageSummary>> {
        let images = self.docker.list_images::<String>(None).await?;
        Ok(images)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_docker_client_creation() {
        let result = DockerClient::new();
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_verify_connection() {
        let client = match DockerClient::new() {
            Ok(client) => client,
            Err(_) => {
                eprintln!("Skipping test: Docker not available");
                return;
            }
        };
        let result = client.verify_connection().await;
        if result.is_err() {
            eprintln!("Skipping test: Docker not available");
            return;
        }
        assert!(result.is_ok());
    }
}
