use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use blake3;
use blake3::Hash;
use rusqlite::{Connection, Error, Params};
use crate::filesystem::error::{AppError, AppResult, InternalError};


const SIGN_FILE_NAME: &'static str = ".afilia_repo";
const DB_FILE_NAME: &'static str = "afilia_repo.db";
const REPO_FORMAT_VERSION : &'static str = "1.0";

#[derive(Serialize, Deserialize)]
struct RepositoryID {
    uuid: Uuid,
    name: String,
    sign: String
}

impl RepositoryID {

    pub fn new(name: &str, payload: &str) -> RepositoryID {
        let repo_uuid = Uuid::new_v4();
        let repo_id = Self {
            uuid: repo_uuid,
            name: String::from(name),
            sign: format!("{}", RepositoryID::sign(&repo_uuid, name, payload).to_hex())
        };
        return repo_id;
    }

    pub fn serialize(&self, path: &str) {
        let path = PathBuf::from(path);
        let mut file = File::create(path.join(SIGN_FILE_NAME).as_path()).unwrap();
        let content = serde_json::to_string_pretty(self).unwrap();
        writeln!(&mut file, "{}", content).unwrap();
    }

    fn sign(repo_uuid: &Uuid, name: &str, payload: &str) -> Hash {
        return blake3::hash(format!("{}:{}:{}", repo_uuid, name, payload).as_bytes());
    }
}

struct RepositoryDB {
    name: String,
    conn: Option<Connection>
}

impl RepositoryDB {

    pub fn new(name: &str, path: &PathBuf) -> RepositoryDB {
        Self {
            name: String::from(name),
            conn: Connection::open(path.join(DB_FILE_NAME).as_path()).ok()
        }
    }

    pub fn execute<P: Params>(&self, sql: &str, params: P) -> AppResult<usize> {
        match self.conn.as_ref().unwrap().execute(sql, params) {
            Ok(updates) => Ok(updates),
            Err(err) => Err(AppError::from_error(err, ""))
        }
    }

    pub fn create(&self) -> AppResult<()> {
        let sql_script = [
            "CREATE TABLE storage_unit (id INTEGER PRIMARY KEY, path VARCHAR NOT NULL, file_count INTEGER DEFAULT 0)",
            "CREATE TABLE IF NOT EXISTS main_catalog (
                 id CHAR(36) PRIMARY KEY,
                 hash BLOB NOT NULL,
                 storage_path VARCHAR(
                 created TIMESTAMP DEFAULT CURRENT_TIMESTAMP NOT NULL,
                 modified TIMESTAMP DEFAULT CURRENT_TIMESTAMP NOT NULL)",
            "CREATE TABLE storage_unit (
                 id INTEGER PRIMARY KEY,
                 path VARCHAR NOT NULL,
                 file_count INTEGER DEFAULT 0)",
            "CREATE TABLE IF NOT EXISTS queue (
                 id CHAR(36) PRIMARY KEY,
                 hash BLOB NOT NULL,
                 created TIMESTAMP DEFAULT CURRENT_TIMESTAMP NOT NULL,
                 modified TIMESTAMP DEFAULT CURRENT_TIMESTAMP NOT NULL)",
            "CREATE TABLE IF NOT EXISTS parameter (
                 key VARCHAR(32) PRIMARY KEY,
                 value VARCHAR(256),
                 created TIMESTAMP DEFAULT CURRENT_TIMESTAMP NOT NULL,
                 modified TIMESTAMP DEFAULT CURRENT_TIMESTAMP NOT NULL)"
        ];
        for sql in sql_script {
            self.execute(sql, []);
        }

        return Ok(());
    }

    fn connect(path: &PathBuf) -> Option<Connection> {
        match Connection::open(path.as_path()) {
            Ok(conn) => Some(conn),
            Err(_) => None
        }

    }
}

pub struct Repository {
    id: RepositoryID,
    database: RepositoryDB,
    path: PathBuf
}

impl Repository {

    pub fn create(path: &str, name: &str, payload: &str) -> Repository {
        let repopath = PathBuf::from(path);
        let repository = Self {
            id: RepositoryID::new(name, payload),
            database: RepositoryDB::new("repo1",&repopath),
            path: repopath
        };
        repository.id.serialize(path);
        repository.database.create();
        return repository;
    }
}