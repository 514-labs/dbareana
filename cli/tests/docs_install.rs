use dbarena::docs::{install_pack, InstallOptions};

#[tokio::test]
#[ignore]
async fn install_postgres_pack() {
    let options = InstallOptions {
        force: true,
        keep_source: false,
        accept_license: true,
    };
    let manifest = install_pack("postgres", "16", options).await.unwrap();
    assert!(manifest.doc_count > 0);
}

#[tokio::test]
#[ignore]
async fn install_mysql_pack() {
    let options = InstallOptions {
        force: true,
        keep_source: false,
        accept_license: true,
    };
    let manifest = install_pack("mysql", "8.0", options).await.unwrap();
    assert!(manifest.doc_count > 0);
}

#[tokio::test]
#[ignore]
async fn install_sqlserver_pack() {
    let options = InstallOptions {
        force: true,
        keep_source: false,
        accept_license: true,
    };
    let manifest = install_pack("sqlserver", "2022-latest", options).await.unwrap();
    assert!(manifest.doc_count > 0);
}
