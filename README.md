# tmp-path

This create contains a macro intended for test purposes that creates a temporary directory available
to the function as `tmp_path`, a `PathBuf`.

The directory is created inside the system temporary directory and is removed when the thread that
created it finishes, including when it panics. Because the test harness runs each test on its own
thread, the directory lives for the duration of the test, so a helper function can create it and
return paths to the test that called it.

```rs
#[tmp_path]
fn mock_config() -> Config {
    // tmp_path is still valid in the test that calls this
}
```

## Installation

```sh
cargo add --dev tmp-path
```

## Usage

```rs
use std::fs::File;

use tmp_path::tmp_path;

#[test]
#[tmp_path]
fn test_example() {
    let test_file = &tmp_path.join("test_file");
    File::create(&test_file).unwrap();

    assert!(test_file.is_file());
}
```
