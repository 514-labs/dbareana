use crate::Result;
use bollard::Docker;
use std::path::Path;
use tar::{Builder, Header};

/// Copy a file to a container using Docker's tar upload API
pub async fn copy_file_to_container(
    docker: &Docker,
    container_id: &str,
    local_path: &Path,
    container_path: &str,
) -> Result<()> {
    // Read the local file
    let file_content = std::fs::read(local_path).map_err(|e| {
        crate::DBArenaError::InitScriptNotFound(format!(
            "Failed to read init script '{}': {}",
            local_path.display(),
            e
        ))
    })?;

    // Get file name
    let file_name = local_path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| {
            crate::DBArenaError::InitScriptNotFound(format!(
                "Invalid file name: {}",
                local_path.display()
            ))
        })?;

    // Create a tar archive in memory
    let mut tar_data = Vec::new();
    {
        let mut ar = Builder::new(&mut tar_data);

        // Create tar header
        let mut header = Header::new_gnu();
        header.set_size(file_content.len() as u64);
        header.set_mode(0o644);
        header.set_cksum();

        // Add file to tar
        ar.append_data(&mut header, file_name, &file_content[..])
            .map_err(|e| {
                crate::DBArenaError::Other(format!("Failed to create tar archive: {}", e))
            })?;

        ar.finish().map_err(|e| {
            crate::DBArenaError::Other(format!("Failed to finalize tar archive: {}", e))
        })?;
    }

    // Upload tar to container
    docker
        .upload_to_container(
            container_id,
            Some(bollard::container::UploadToContainerOptions {
                path: container_path.to_string(),
                ..Default::default()
            }),
            tar_data.into(),
        )
        .await?;

    Ok(())
}

/// Copy multiple files to a container
pub async fn copy_files_to_container(
    docker: &Docker,
    container_id: &str,
    local_paths: &[&Path],
    container_dir: &str,
) -> Result<()> {
    // Parse the container directory to get parent and subdirectory name
    let parent_dir = std::path::Path::new(container_dir)
        .parent()
        .and_then(|p| p.to_str())
        .unwrap_or("/tmp");

    let dir_name = std::path::Path::new(container_dir)
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| {
            crate::DBArenaError::Other(format!("Invalid container directory: {}", container_dir))
        })?;

    // Create a tar archive with multiple files in a subdirectory
    let mut tar_data = Vec::new();
    {
        let mut ar = Builder::new(&mut tar_data);

        for local_path in local_paths {
            let file_content = std::fs::read(local_path).map_err(|e| {
                crate::DBArenaError::InitScriptNotFound(format!(
                    "Failed to read init script '{}': {}",
                    local_path.display(),
                    e
                ))
            })?;

            let file_name = local_path
                .file_name()
                .and_then(|n| n.to_str())
                .ok_or_else(|| {
                    crate::DBArenaError::InitScriptNotFound(format!(
                        "Invalid file name: {}",
                        local_path.display()
                    ))
                })?;

            // Add file with subdirectory path (e.g., "dbarena_init/script.sql")
            let tar_path = format!("{}/{}", dir_name, file_name);

            let mut header = Header::new_gnu();
            header.set_size(file_content.len() as u64);
            header.set_mode(0o644);
            header.set_cksum();

            ar.append_data(&mut header, &tar_path, &file_content[..])
                .map_err(|e| {
                    crate::DBArenaError::Other(format!("Failed to create tar archive: {}", e))
                })?;
        }

        ar.finish().map_err(|e| {
            crate::DBArenaError::Other(format!("Failed to finalize tar archive: {}", e))
        })?;
    }

    // Upload tar to parent directory (e.g., /tmp)
    // The tar will extract files into /tmp/dbarena_init/
    docker
        .upload_to_container(
            container_id,
            Some(bollard::container::UploadToContainerOptions {
                path: parent_dir.to_string(),
                ..Default::default()
            }),
            tar_data.into(),
        )
        .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tar_creation() {
        // Test that we can create a tar archive
        let mut tar_data = Vec::new();
        {
            let mut ar = Builder::new(&mut tar_data);

            let content = b"CREATE TABLE test;";
            let mut header = Header::new_gnu();
            header.set_size(content.len() as u64);
            header.set_mode(0o644);
            header.set_cksum();

            ar.append_data(&mut header, "test.sql", &content[..])
                .unwrap();
            ar.finish().unwrap();
        } // Drop ar here so we can access tar_data

        assert!(!tar_data.is_empty());
    }
}
