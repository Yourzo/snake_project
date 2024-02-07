use surrealdb::engine::remote::ws::{Client, Ws};
use surrealdb::opt::auth::Root;
use surrealdb::{Surreal, Error};
use crate::models::file_models::{UserId, UserFilesToShow, UserSessionIdMatch};
use crate::models::{FileInfo, UserInfo};

#[derive(Clone)]
pub struct Database {
    client: Surreal<Client>,
    name_space: String,
    db_name: String,
}

impl Database {
    pub async fn init() -> Result<Self, Error> {
        let client = Surreal::new::<Ws>("127.0.0.1:8000").await?;
        client.signin( Root {
            username: "root",
            password: "root",
        }).await?;
        client.use_ns("surreal").use_db("file system").await.unwrap();
        Ok(Database {
            client,
            name_space: String::from("surreal"),
            db_name: String::from("file system")
        })
    }

    pub async fn add_allowed_file_to_user(&self, user_id: String, file_id: String) -> Option<UserInfo> {
        let user_info: Result<Option<UserInfo>, Error> = self
            .client
            .select(("user", user_id.clone()))
            .await;
        match user_info {
            Ok(Some(mut info)) => {
                let res: Result<Option<UserInfo>, Error> = self
                    .client
                    .update(("user", user_id))
                    .content(info.accessible_files_uuids.push(file_id))
                    .await;
                return Some(info);
            }
            Ok(None) => None,
            Err(_) => None
        }
    }

    pub async fn add_new_user(&self, user_info: UserInfo) -> Option<UserInfo> {
        let res = self
            .client
            .create(("user", &user_info.uuid.clone()))
            .content(user_info.clone())
            .await;
        let _rec: Result<Option<UserId>, Error> = self
            .client
            .create(("user_id", &user_info.user_name.clone()))
            .content(UserId {
                uuid: user_info.uuid
            })
            .await;
        res.unwrap_or_else(|_| None)
    }

    pub async fn put_file(&self, file_info: FileInfo) -> Option<FileInfo> {
        let res = self
            .client
            .create(("file", file_info.uuid.clone()))
            .content(file_info)
            .await;
        res.unwrap_or_else(|_| None)
    }

    pub async fn get_user_by_name(&self, user_name: String) -> Option<UserInfo> {
        let rec: Result<Option<UserId>, Error> = self
            .client
            .select(("user_id", user_name))
            .await;
        match rec {
            Ok(Some(id)) => {
                let res = self
                    .client
                    .select(("user", id.uuid))
                    .await;
                res.unwrap_or_else(|_| None)
            }
            Ok(None) => None,
            Err(_) => None
        }
    }

    pub async fn get_user_by_session_id(&self, session_id: String) -> Option<UserInfo> {
        let user_id = self
            .get_user_id_from_session_id(session_id)
            .await;
        match user_id {
            Some(user_id_found) => {
                let result = self
                    .client
                    .select(("user", user_id_found))
                    .await;
                result.unwrap_or_else(|_| None)
            }
            None => None
        }
    }

    pub async fn get_all_users_files(&self, session_id: String) -> Option<Vec<String>> {
        let user_profile = self
            .get_user_by_session_id(session_id)
            .await;
        match user_profile {
            Some(user) => Some(user.accessible_files_uuids),
            None => None
        }
    }

    pub async fn set_session(&self, session_match: UserSessionIdMatch) -> Option<UserSessionIdMatch> {
        let res = self
            .client
            .create(("session", session_match.session_id.clone()))
            .content(session_match)
            .await;
        match res {
            Ok(Some(matching)) => Some(matching),
            Ok(None) => None,
            Err(_) => None,
        }
    }

    pub async fn verify_session(&self, session_id: String) -> bool {
        let res: Result<Option<UserSessionIdMatch>, Error> = self
            .client
            .select(("session", session_id.clone()))
            .await;
        match res {
            Ok(Some(_)) => true,
            Ok(None) => false,
            Err(_) => false,
        }
    }

    pub async fn get_user_id_from_session_id(&self, session_id: String) -> Option<String>  {
        let res: Result<Option<UserSessionIdMatch>, Error> = self
            .client
            .select(("session", session_id))
            .await;
        match res {
            Ok(Some(sess_user_match)) => Some(sess_user_match.user_id),
            Ok(None) => None,
            Err(_) => None
        }
    }

    pub async fn get_file(&self, file_id: String) -> Option<FileInfo> {
        let res = self
            .client
            .select(("file", file_id))
            .await;
        res.unwrap_or_else(|_| None)
    }

    pub async fn remove_session(&self, session_id: String) -> Option<UserSessionIdMatch> {
        let res = self
            .client
            .delete(("session", session_id))
            .await;
        res.unwrap_or_else(|_| None)
    }

    pub async fn get_all_file_info(&self, user_id: String) -> Option<UserFilesToShow> {
        let res = self.get_all_users_files(user_id).await;
        let mut ret_val = UserFilesToShow {
            file_ids: Vec::new(),
            file_names: Vec::new(),
            file_paths: Vec::new(),
        };
        match res {
            Some(user_files) => {

                for user_file in user_files {

                    let file: FileInfo = self.client.select(("file", user_file)).await.unwrap().unwrap();
                    ret_val.file_ids.push(file.uuid);
                    ret_val.file_names.push(file.name);
                    ret_val.file_paths.push(file.path);
                }
                Some(ret_val)
            }
            None => None
        }
    }

    pub async fn put_master_key(&self, master_key: String) -> Option<String> {
        let res = self
            .client
            .create(("master_key", 0))
            .content(master_key)
            .await;
        res.unwrap_or_else(|_| None)
    }
}