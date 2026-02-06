use dbarena::docs::{install_pack, InstallOptions};

static INSTALL_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

fn with_temp_docs_dir() -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("failed to create temp dir");
    std::env::set_var("DBARENA_DOCS_DIR", dir.path());
    std::env::set_var("DBARENA_DOCS_MAX_PAGES", "80");
    dir
}

#[tokio::test]
#[ignore]
async fn install_postgres_pack() {
    let _guard = INSTALL_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let _dir = with_temp_docs_dir();
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
    let _guard = INSTALL_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let _dir = with_temp_docs_dir();
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
    let _guard = INSTALL_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let _dir = with_temp_docs_dir();
    let options = InstallOptions {
        force: true,
        keep_source: false,
        accept_license: true,
    };
    let manifest = install_pack("sqlserver", "2022-latest", options).await.unwrap();
    assert!(manifest.doc_count > 0);
}
