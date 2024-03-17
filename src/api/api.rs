use std::io::{Read, Error};

use actix_web::{get, post, patch, HttpResponse, Responder, HttpRequest};
use actix_web::web::{Json, Path, Data, Bytes};
use actix_multipart::form::MultipartForm;
use actix_files::NamedFile;

use log::error;
use uuid::Uuid;
use async_stream::stream;

use crate::database::Database;
use crate::models::file_models::{GetFileUrl, MasterKey, UploadData, UploadForm, UserCreateProfile, UserLogin, UserSessionIdMatch, UuidStruct, UuidUrl};
use crate::models::{FileInfo, UserInfo};

static MAX_SIZE_ALLOWED: usize = 50_000_000;

#[post("/login")]
async fn login_user(body: Json<UserLogin>, db: Data<Database>) -> impl Responder {
    let user_login = body.into_inner();
    let user_info = db.get_user_by_name(user_login.user_name).await;

    match user_info {
        Some(user) => {
            if user.password == user_login.password {
                let res = db.into_inner().set_session(UserSessionIdMatch {
                    user_id: user.uuid,
                    session_id: Uuid::new_v4().to_string(),
                }).await;
                return HttpResponse::Ok().body(res.unwrap().session_id);
            }
            return HttpResponse::Ok().body("Wrong password");
        },
        None => HttpResponse::Ok().body("username doesn't exist")
    }
}

//TODO property test out DO NOT commit until then
#[get("/get_file/{session_id}")]
async fn get_file(
    session: Path<UuidUrl>,
    uuid_file: Json<UuidStruct>,
    db: Data<Database>
) -> impl Responder {
    let file_data_opt = db.get_file(uuid_file.into_inner().uuid).await;

    match file_data_opt {
        Some(file_data) => {
            let file_path = file_data.uuid;
            if let Ok(mut file) = NamedFile::open(file_path.clone()) {
                let my_data_stream = stream! {
                    let mut chunk =vec![0u8; 10 * 1024 * 1024];//chunk size
                    loop {
                        match file.read(&mut chunk) {
                            Ok(n) => {
                                if n == 0 {
                                    break;
                                }

                                yield Result::<Bytes, Error>::Ok(Bytes::from(chunk[..n].to_vec()));
                            }
                            Err(e) => {
                                error!("Error reading file: {}", e);
                                yield Result::<Bytes, Error>::Err(e);
                                break;
                            }
                        }
                    }
                };
                HttpResponse::Ok()
                    .content_type("application/octet-stream")
                    .streaming(my_data_stream)
            } else {
                HttpResponse::NotFound().body("file not found")
            }
        }
        None => HttpResponse::BadRequest().body("file user not file")
    }
}

//problem of this method is that it expect client app to build path
#[post("/file_upload_data/{session_id}")]
async fn info_to_upload(
    db: Data<Database>,
    session_id: Path<UuidUrl>,
    body: Json<UploadData>,
) -> impl Responder {
    let upload_data = body.into_inner();
    if db.verify_session(session_id.into_inner().session_id).await {
        return HttpResponse::BadRequest().body("couldnt verify session")
    }
    match db.put_file(FileInfo {
        name: upload_data.name.clone(),
        uuid: Uuid::new_v4().to_string(),
        path: upload_data.path.clone(),
        user_id: upload_data.user_id,
    }).await {
        Some(_) => HttpResponse::Ok().body("file info in database"),
        None => HttpResponse::BadRequest().body("it's not in database")
    }
}

#[post("/upload_file/{session_id}")]
async fn upload(
    MultipartForm(form): MultipartForm<UploadForm>,
    session_id: Path<UuidUrl>,
    db: Data<Database>
) -> impl Responder {
    if !db.verify_session(session_id.into_inner().session_id).await {
        return HttpResponse::BadRequest().body("session not found")
    }
    if form.file.size > MAX_SIZE_ALLOWED {
        return HttpResponse::PreconditionFailed().body("file too big");
    }
    let path = format!(".\\user_files\\{}", form.file.file_name.clone().unwrap());
    log::info!("saving file to {path}");
    match form.file.file.persist(path.clone()) {
        Ok(_) => HttpResponse::Ok().body("file saved"),
        Err(_) => HttpResponse::BadRequest().body("file not saved"),
    }
}

#[get("/get_all_files_info/{session_id}")]
async fn get_all_allowed_files_info(path: Path<UuidUrl>, db: Data<Database>) -> impl Responder {
    let all_files = db.get_all_file_info(path.into_inner().session_id).await;
    match all_files {
        Some(files) => HttpResponse::Ok().json(files),
        None => HttpResponse::NoContent().body("i dunno what could be problem")
    }
}

#[get("/all_allowed_files/{session_id}")]
async fn get_all_allowed(path: Path<UuidUrl>, db: Data<Database>) -> impl Responder {
    let uuid = path.into_inner();
    if db.verify_session(uuid.session_id.clone()).await {
        let res = db
            .get_all_users_files(uuid.session_id.clone()).
            await;
        return match res {
            Some(files) => HttpResponse::Ok().json(files),
            None => HttpResponse::NotFound().body("some how i cant: "),
        }
    }
    HttpResponse::Unauthorized().body("session was not found")
}

#[post("/set_user/{master_key}")]
async fn set_user(
    loading_user: Json<UserCreateProfile>,
    _path: Path<MasterKey>,
    db: Data<Database>
) -> impl Responder {
    let user_login = loading_user.into_inner();
    let user_info = UserInfo {
        accessible_files_uuids: Vec::new(),
        uuid: Uuid::new_v4().to_string(),
        user_name: user_login.user_name.clone(),
        password: user_login.password,
        users_path: "/".to_string() + &*user_login.user_name,
    };
    let res = db
        .add_new_user(user_info).await;
    match res {
        Some(user) => HttpResponse::Ok().body(user.uuid),
        None => HttpResponse::PreconditionFailed().body("this user creation failed")
    }
}

