# tmp-path

This create contains a macro intended for test purposes that creates a temporary directory named
`tmp_path` as a Pathbuff.

## Installation

Note that you also need to install the [tempfile](https://docs.rs/tempfile/latest/tempfile/index.html)
crate.

```sh
cargo add --dev tmp-path tempfile
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
