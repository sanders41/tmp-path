use std::path::PathBuf;
use std::sync::Mutex;

use tmp_path::tmp_path;

#[tmp_path]
fn captured_tmp_path() -> PathBuf {
    tmp_path.to_path_buf()
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

    let entries = std::fs::read_dir(tmp_path).expect("tmp_path should be readable");
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
fn test_tmp_path_is_deleted_when_the_function_returns() {
    let path = captured_tmp_path();

    assert!(!path.exists());
}

#[test]
fn test_tmp_path_is_deleted_when_contents_remain() {
    let path = {
        #[tmp_path]
        fn write_a_file() -> PathBuf {
            std::fs::write(tmp_path.join("test_file"), "test").expect("write should succeed");
            tmp_path.to_path_buf()
        }

        write_a_file()
    };

    assert!(!path.exists());
}

#[test]
fn test_tmp_path_is_deleted_on_panic() {
    static PANICKED_PATH: Mutex<Option<PathBuf>> = Mutex::new(None);

    #[tmp_path]
    fn panics() {
        *PANICKED_PATH.lock().expect("lock should not be poisoned") = Some(tmp_path.to_path_buf());
        panic!("boom");
    }

    let previous_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let result = std::panic::catch_unwind(panics);
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
fn test_each_tmp_path_is_unique() {
    let first = captured_tmp_path();
    let second = captured_tmp_path();

    assert_ne!(first, second);
}
