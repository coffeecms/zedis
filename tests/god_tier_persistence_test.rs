#[cfg(test)]
mod tests {
    use zedis::core::storage::Db;
    use zedis::persistence::Persistence;
    use zedis::core::structs::sso_string::ZedisString;
    use std::sync::Arc;
    use std::fs;

    #[test]
    fn test_god_tier_persistence_rdb() {
        // 1. Setup DB with Data
        let db = Arc::new(Db::new(16));
        db.set_string("key1".to_string(), "TopG".to_string());
        db.list_push("list1".to_string(), "Task1".to_string());
        db.hset("hash1".to_string(), "field1".to_string(), "value1".to_string());

        // 2. Save RDB (Binary Stream)
        let rdb_path = "test_dump.rdb";
        let res = Persistence::save_rdb(&db, rdb_path);
        assert!(res.is_ok(), "RDB Save failed");

        // 3. Load RDB
        let loaded_db_res = Persistence::load_rdb(rdb_path);
        assert!(loaded_db_res.is_ok(), "RDB Load failed");
        let loaded_db = loaded_db_res.unwrap();

        // 4. Verify Data integrity
        assert_eq!(loaded_db.get_string("key1"), Some("TopG".to_string()));
        
        let list = loaded_db.list_range("list1", 0, -1);
        assert_eq!(list.len(), 1);
        assert_eq!(list[0], "Task1");
        
        // 5. Cleanup
        let _ = fs::remove_file(rdb_path);
    }
}
