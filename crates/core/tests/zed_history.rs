use super::*;

#[test]
fn reads_installed_zed_history_when_present() {
    let path = database().unwrap();
    if path.exists() { read_database(&path).unwrap(); }
}

#[test]
fn reads_local_and_ssh_history_in_order() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("zed.sqlite");
    let db = Connection::open(&path).unwrap();
    db.execute_batch("CREATE TABLE remote_connections (id INTEGER, kind TEXT, host TEXT, user TEXT, port INTEGER);
        CREATE TABLE workspaces (workspace_id INTEGER, paths TEXT, paths_order TEXT, remote_connection_id INTEGER, timestamp TEXT);
        INSERT INTO remote_connections VALUES (1, 'ssh', 'example.com', 'alice', 2222);
        INSERT INTO workspaces VALUES (1, '/older', '0', NULL, '2025');
        INSERT INTO workspaces VALUES (2, '/one\n/two spaces', '1,0', 1, '2026');").unwrap();
    assert_eq!(read_database(&path).unwrap(), [
        "ssh://alice@example.com:2222/two%20spaces", "ssh://alice@example.com:2222/one", "/older",
    ]);
}
