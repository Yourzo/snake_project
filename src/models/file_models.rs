use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing;

#[derive(Serialize, Deserialize, Debug)]
pub struct FileInfo {
    pub uuid: String,
    pub name: String,
    pub path: String,
    pub content_type: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UserCreateProfile {
    pub user_name: String,
    pub password: String,

}
#[derive(Serialize, Deserialize, Debug)]
pub struct UserId {
    pub uuid: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MasterKey {
    pub master_key: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserInfo {
    pub accessible_files_uuids: Vec<String>,
    pub uuid: String,
    pub user_name: String,
    pub password: String,
    pub users_path: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserLogin {
    pub user_name: String,
    pub password: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UuidUrl {
    pub session_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetFileUrl {
    pub session_id: String,
    pub file_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UserSessionIdMatch {
    pub user_id: String,
    pub session_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UserFilesToShow {
    pub file_ids: Vec<String>,
    pub file_names: Vec<String>,
    pub file_paths: Vec<String>,
}
