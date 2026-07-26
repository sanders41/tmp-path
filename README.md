# tmp-path

This create contains a macro intended for test purposes that creates a temporary directory available
to the function as `tmp_path`, a `&Path`.

The directory is created inside the system temporary directory and is removed when the function
returns, including when it panics.

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
