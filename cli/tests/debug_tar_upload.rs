/// Debug test for tar upload mechanism
use bollard::Docker;
use tar::{Builder, Header};
use uuid::Uuid;

use bollard::container::{
    Config, CreateContainerOptions, RemoveContainerOptions, StartContainerOptions,
    StopContainerOptions,
};
use bollard::image::CreateImageOptions;
use futures::StreamExt;

async fn ensure_image(docker: &Docker, image: &str) {
    let options = Some(CreateImageOptions {
        from_image: image,
        ..Default::default()
    });
    let mut stream = docker.create_image(options, None, None);
    while let Some(_item) = stream.next().await {}
}

async fn create_test_container(docker: &Docker) -> String {
    let image = "alpine:3.19";
    ensure_image(docker, image).await;

    let name = format!("dbarena-debug-{}", Uuid::new_v4());
    let config = Config {
        image: Some(image),
        cmd: Some(vec!["sleep", "120"]),
        ..Default::default()
    };

    let create = docker
        .create_container(
            Some(CreateContainerOptions {
                name,
                platform: None,
            }),
            config,
        )
        .await
        .expect("Failed to create container");

    docker
        .start_container(&create.id, None::<StartContainerOptions<String>>)
        .await
        .expect("Failed to start container");

    create.id
}

async fn cleanup_container(docker: &Docker, container_id: &str) {
    let _ = docker
        .stop_container(container_id, Some(StopContainerOptions { t: 2 }))
        .await;
    let _ = docker
        .remove_container(
            &container_id,
            Some(RemoveContainerOptions {
                force: true,
                ..Default::default()
            }),
        )
        .await;
}

#[tokio::test]
#[ignore]
async fn debug_tar_upload() {
    // Connect to Docker
    let docker = match Docker::connect_with_local_defaults() {
        Ok(docker) => docker,
        Err(_) => return,
    };
    if docker.ping().await.is_err() {
        return;
    }

    let container_id = create_test_container(&docker).await;

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
            &container_id,
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
            &container_id,
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

    if let StartExecResults::Attached {
        output: mut stream, ..
    } = docker
        .start_exec(&exec.id, None)
        .await
        .expect("Failed to start exec")
    {
        println!("\nDirectory listing:");
        while let Some(Ok(msg)) = stream.next().await {
            print!("{}", msg);
        }
    }

    cleanup_container(&docker, &container_id).await;
}

#[tokio::test]
#[ignore]
async fn debug_tar_upload_with_directory() {
    // Connect to Docker
    let docker = match Docker::connect_with_local_defaults() {
        Ok(docker) => docker,
        Err(_) => return,
    };
    if docker.ping().await.is_err() {
        return;
    }

    let container_id = create_test_container(&docker).await;

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
            &container_id,
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
            &container_id,
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

    if let StartExecResults::Attached {
        output: mut stream, ..
    } = docker
        .start_exec(&exec.id, None)
        .await
        .expect("Failed to start exec")
    {
        println!("\nDirectory listing:");
        while let Some(Ok(msg)) = stream.next().await {
            print!("{}", msg);
        }
    }

    cleanup_container(&docker, &container_id).await;
}
