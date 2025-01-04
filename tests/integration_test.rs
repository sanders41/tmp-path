use tmp_path::tmp_path;

#[test]
#[tmp_path]
fn test_tmp_path() {
    assert!(tmp_path.exists());
}
