/// Debug test for tar upload mechanism
use bollard::Docker;
use std::io::Write;
use tar::{Builder, Header};

#[tokio::test]
#[ignore]
async fn debug_tar_upload() {
    // Connect to Docker
    let docker = Docker::connect_with_local_defaults().expect("Failed to connect");

    // Use one of the existing test containers
    let container_id = "d9c7c1d0b0a3";

    // Create a simple tar archive
    let mut tar_data = Vec::new();
    {
        let mut ar = Builder::new(&mut tar_data);

        let content = b"CREATE TABLE debug_test (id INT);";
        let mut header = Header::new_gnu();
        header.set_size(content.len() as u64);
        header.set_mode(0o644);
        header.set_cksum();

        // Try uploading with simple path first
        ar.append_data(&mut header, "test_script.sql", &content[..])
            .expect("Failed to append");
        ar.finish().expect("Failed to finish tar");
    }

    println!("Created tar archive with {} bytes", tar_data.len());

    // Upload to /tmp
    let result = docker
        .upload_to_container(
            container_id,
            Some(bollard::container::UploadToContainerOptions {
                path: "/tmp".to_string(),
                ..Default::default()
            }),
            tar_data.into(),
        )
        .await;

    match result {
        Ok(_) => println!("Upload succeeded"),
        Err(e) => println!("Upload failed: {}", e),
    }

    // Check if file exists
    let exec = docker
        .create_exec(
            container_id,
            bollard::exec::CreateExecOptions {
                cmd: Some(vec!["ls", "-la", "/tmp/"]),
                attach_stdout: Some(true),
                attach_stderr: Some(true),
                ..Default::default()
            },
        )
        .await
        .expect("Failed to create exec");

    use bollard::exec::StartExecResults;
    use futures::StreamExt;

    if let StartExecResults::Attached { output: mut stream, .. } =
        docker.start_exec(&exec.id, None).await.expect("Failed to start exec")
    {
        println!("\nDirectory listing:");
        while let Some(Ok(msg)) = stream.next().await {
            print!("{}", msg);
        }
    }
}

#[tokio::test]
#[ignore]
async fn debug_tar_upload_with_directory() {
    // Connect to Docker
    let docker = Docker::connect_with_local_defaults().expect("Failed to connect");

    // Use one of the existing test containers
    let container_id = "d9c7c1d0b0a3";

    // Create a tar archive with subdirectory
    let mut tar_data = Vec::new();
    {
        let mut ar = Builder::new(&mut tar_data);

        let content = b"CREATE TABLE debug_test2 (id INT);";
        let mut header = Header::new_gnu();
        header.set_size(content.len() as u64);
        header.set_mode(0o644);
        header.set_cksum();

        // Try with subdirectory path
        ar.append_data(&mut header, "dbarena_init/test_script.sql", &content[..])
            .expect("Failed to append");
        ar.finish().expect("Failed to finish tar");
    }

    println!("Created tar archive with {} bytes", tar_data.len());

    // Upload to /tmp
    let result = docker
        .upload_to_container(
            container_id,
            Some(bollard::container::UploadToContainerOptions {
                path: "/tmp".to_string(),
                ..Default::default()
            }),
            tar_data.into(),
        )
        .await;

    match result {
        Ok(_) => println!("Upload succeeded"),
        Err(e) => println!("Upload failed: {}", e),
    }

    // Check if directory and file exist
    let exec = docker
        .create_exec(
            container_id,
            bollard::exec::CreateExecOptions {
                cmd: Some(vec!["ls", "-la", "/tmp/dbarena_init/"]),
                attach_stdout: Some(true),
                attach_stderr: Some(true),
                ..Default::default()
            },
        )
        .await
        .expect("Failed to create exec");

    use bollard::exec::StartExecResults;
    use futures::StreamExt;

    if let StartExecResults::Attached { output: mut stream, .. } =
        docker.start_exec(&exec.id, None).await.expect("Failed to start exec")
    {
        println!("\nDirectory listing:");
        while let Some(Ok(msg)) = stream.next().await {
            print!("{}", msg);
        }
    }
}
