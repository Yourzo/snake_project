use surrealdb::engine::remote::ws::{Client, Ws};
use surrealdb::opt::auth::Root;
use surrealdb::{Surreal, Error};
use crate::models::file_models::{UserFilesToShow, UserSessionIdMatch};
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

    pub async fn put_file(&self, file_info: FileInfo) -> Option<FileInfo> {
        let res = self
            .client
            .create(("file", file_info.uuid.clone()))
            .content(file_info)
            .await;
        match res {
            Ok(Some(file)) => Some(file),
            Ok(None) => None,
            Err(_) => None
        }
    }

    pub async fn get_user_by_name(&self, user_name: String) -> Option<UserInfo> {
        let res = self
            .client
            .select(("user", user_name))
            .await;
        match res {
            Ok(might_be_user) => match might_be_user {
                Some(user) => Some(user),
                None => None,
            }
            Err(_) => None
        }
    }

    pub async fn get_user_by_id(&self, user_id: String) -> Option<UserInfo> {
        let result = self
            .client
            .select(("user", user_id))
            .await;
        match result {
            Ok(might_be_user) => match might_be_user {
                Some(user) => Some(user),
                None => None
            },
            Err(_) => None
        }
    }

    pub async fn get_all_users_files(&self, user_id: String) -> Option<Vec<String>> {
        let user_profile = self.get_user_by_id(user_id).await;
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
            Ok(session) => match session {
                Some(matching) => Some(matching),
                None => None
            }
            Err(_) => None
        }
    }

    pub async fn verify_session(&self, session_id: String) -> bool {
        let res: Result<Option<UserSessionIdMatch>, Error> = self.client.select(("session", session_id.clone())).await;
        match res {
            Ok(session_id) => {
                match session_id {
                    Some(session) => true,
                    None => false,
                }
            }
            Err(_) => false,
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
        match res {
            Ok(session) => match session {
                Some(session_found) => Some(session_found),
                None => None
            }
            Err(_) => None,
        }
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
}