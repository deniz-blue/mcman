use mcman::store::{ObjectKey, Store};
use std::fs;
use tempfile::TempDir;

async fn store() -> (TempDir, Store) {
    let root = TempDir::new().expect("a temp dir");
    let store = Store::open(root.path()).await.expect("the store opens");
    (root, store)
}

#[tokio::test]
async fn bytes_written_in_chunks_land_under_their_own_hash() {
    let (_root, store) = store().await;

    let mut object = store.write_object().await.expect("a writer");
    object.write(b"hello ").await.expect("writes");
    object.write(b"world").await.expect("writes");
    let key = object.finish().await.expect("finishes");

    assert_eq!(key, ObjectKey::from(blake3::hash(b"hello world")));
    assert_eq!(
        fs::read(store.object_path(&key)).expect("the object is readable"),
        b"hello world"
    );
}

#[tokio::test]
async fn the_same_bytes_put_twice_leave_one_object() {
    let (_root, store) = store().await;
    let scratch = TempDir::new().expect("a temp dir");

    let one = scratch.path().join("one.jar");
    let two = scratch.path().join("two.jar");
    fs::write(&one, b"same").expect("writes");
    fs::write(&two, b"same").expect("writes");

    let first = store.put(&one).await.expect("puts");
    let second = store.put(&two).await.expect("puts the same bytes again");

    assert_eq!(first, second);
    let object = store.object_path(&first);
    assert!(
        fs::metadata(&object)
            .expect("the object exists")
            .permissions()
            .readonly(),
        "an object should be read-only"
    );
    assert_eq!(
        fs::read_dir(object.parent().expect("a shard"))
            .expect("reads")
            .count(),
        1
    );
}

#[tokio::test]
async fn an_exclusive_open_is_refused_while_another_store_is_open() {
    let (root, store) = store().await;

    Store::open_exclusive(root.path())
        .await
        .expect_err("an exclusive open should be refused while another store is open");

    drop(store);
    Store::open_exclusive(root.path())
        .await
        .expect("the store opens once no other store is open");
}

#[tokio::test]
async fn a_build_in_tmp_survives_another_store_opening() {
    let (root, store) = store().await;

    let build = store.tmp().expect("a build directory");
    fs::write(build.path().join("server.jar"), b"in progress").expect("writes");

    Store::open(root.path())
        .await
        .expect("a second store opens");

    assert!(
        build.path().join("server.jar").exists(),
        "a live build should outlast another store opening"
    );
}

#[tokio::test]
async fn opening_a_store_clears_what_a_crash_left_in_tmp() {
    let root = TempDir::new().expect("a temp dir");
    let leftover = root.path().join("tmp").join("half-written.jar");
    fs::create_dir_all(leftover.parent().expect("tmp")).expect("creates tmp");
    fs::write(&leftover, b"partial").expect("writes");

    Store::open(root.path()).await.expect("the store opens");

    assert!(!leftover.exists(), "tmp should be empty after open");
}
