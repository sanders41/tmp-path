use std::path::PathBuf;
use std::sync::Mutex;

use tmp_path::tmp_path;

#[tmp_path]
fn captured_tmp_path() -> PathBuf {
    tmp_path
}

fn takes_owned_path(path: Option<PathBuf>) -> bool {
    path.is_some_and(|path| path.is_dir())
}

#[test]
#[tmp_path]
fn test_tmp_path() {
    assert!(tmp_path.exists());
}

#[test]
#[tmp_path]
fn test_tmp_path_is_an_empty_directory() {
    assert!(tmp_path.is_dir());

    let entries = std::fs::read_dir(&tmp_path).expect("tmp_path should be readable");
    assert_eq!(entries.count(), 0);
}

#[test]
#[tmp_path]
fn test_tmp_path_is_inside_the_system_temp_dir() {
    assert!(tmp_path.starts_with(std::env::temp_dir()));

    let name = tmp_path
        .file_name()
        .expect("tmp_path should have a file name")
        .to_string_lossy()
        .into_owned();

    assert!(name.starts_with(&format!("tmp-path-{}-", std::process::id())));
}

#[test]
#[tmp_path]
fn test_tmp_path_is_writable() {
    let test_file = tmp_path.join("test_file");
    std::fs::write(&test_file, "test").expect("tmp_path should be writable");

    assert!(test_file.is_file());
}

#[test]
#[tmp_path]
fn test_tmp_path_can_be_pushed_to() {
    tmp_path.push("nested");
    std::fs::create_dir(&tmp_path).expect("nested directory should be creatable");

    assert!(tmp_path.is_dir());
    assert!(tmp_path.ends_with("nested"));
}

#[test]
#[tmp_path]
fn test_tmp_path_can_be_moved() {
    assert!(takes_owned_path(Some(tmp_path)));
}

#[test]
fn test_tmp_path_outlives_the_function_that_created_it() {
    let path = captured_tmp_path();

    assert!(path.is_dir());
}

#[test]
fn test_tmp_path_is_deleted_when_the_thread_finishes() {
    let path = std::thread::spawn(captured_tmp_path)
        .join()
        .expect("thread should not panic");

    assert!(!path.exists());
}

#[test]
fn test_tmp_path_is_deleted_when_contents_remain() {
    #[tmp_path]
    fn write_a_file() -> PathBuf {
        std::fs::write(tmp_path.join("test_file"), "test").expect("write should succeed");
        tmp_path
    }

    let path = std::thread::spawn(write_a_file)
        .join()
        .expect("thread should not panic");

    assert!(!path.exists());
}

#[test]
fn test_the_original_directory_is_deleted_after_tmp_path_is_mutated() {
    static ORIGINAL: Mutex<Option<PathBuf>> = Mutex::new(None);

    #[tmp_path]
    fn mutates() {
        *ORIGINAL.lock().expect("lock should not be poisoned") = Some(tmp_path.clone());
        tmp_path.push("nested");
        std::fs::create_dir(&tmp_path).expect("nested directory should be creatable");
    }

    std::thread::spawn(mutates)
        .join()
        .expect("thread should not panic");

    let original = ORIGINAL
        .lock()
        .expect("lock should not be poisoned")
        .clone()
        .expect("the original tmp_path should have been recorded");

    assert!(!original.exists());
}

#[test]
fn test_tmp_path_is_deleted_on_panic() {
    static PANICKED_PATH: Mutex<Option<PathBuf>> = Mutex::new(None);

    #[tmp_path]
    fn panics() {
        *PANICKED_PATH.lock().expect("lock should not be poisoned") = Some(tmp_path);
        panic!("boom");
    }

    let previous_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let result = std::thread::spawn(panics).join();
    std::panic::set_hook(previous_hook);

    assert!(result.is_err());

    let path = PANICKED_PATH
        .lock()
        .expect("lock should not be poisoned")
        .clone()
        .expect("the panicking function should have recorded its tmp_path");

    assert!(!path.exists());
}

#[test]
fn test_all_directories_from_one_thread_are_deleted() {
    let paths = std::thread::spawn(|| vec![captured_tmp_path(), captured_tmp_path()])
        .join()
        .expect("thread should not panic");

    assert_eq!(paths.len(), 2);
    assert_ne!(paths[0], paths[1]);

    for path in &paths {
        assert!(!path.exists());
    }
}

#[test]
fn test_each_tmp_path_is_unique() {
    let first = captured_tmp_path();
    let second = captured_tmp_path();

    assert_ne!(first, second);
}
