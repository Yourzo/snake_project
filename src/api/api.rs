use std::fs;
use futures_lite::stream::StreamExt;
use actix_multipart::Multipart;
use actix_web::{get, post, patch, HttpResponse, Responder, HttpRequest};
use actix_web::http::header::CONTENT_LENGTH;
use uuid::Uuid;
use actix_web::web::{Json, Path, Data};
use mime::{Mime};
use fs::File;
use crate::database::Database;
use crate::models::file_models::{GetFileUrl, MasterKey, UserCreateProfile, UserLogin, UserSessionIdMatch, UuidUrl};
use crate::models::{FileInfo, UserInfo};


// Current version tested
#[post("/login")]
async fn login_user(body: Json<UserLogin>, db: Data<Database>) -> impl Responder {
    let user_login = body.into_inner();
    let mut buffer = Uuid::encode_buffer();
    let session_id_created = Uuid::new_v4().simple().encode_lower(&mut buffer);

    let user_info = db.get_user_by_name(user_login.user_name).await;
    match user_info {
        Some(user) => {
            if user.password == user_login.password {
                let res = db.into_inner().set_session(UserSessionIdMatch {
                    user_id: user.uuid,
                    session_id: String::from(session_id_created)
                }).await;
                return HttpResponse::Ok().body(res.unwrap().session_id);
            }
            return HttpResponse::Ok().body("Wrong password");
        },
        None => HttpResponse::Ok().body("username doesn't exist")
    }
}

//server will receive file one at the time, client will decide where will file be placed
//TODO
#[post("/upload_file")]
async fn upload(mut payload: Multipart, req: HttpRequest, db: Data<Database>) -> impl Responder {
    let max_file_size: usize = 50_000;
    let content_length: usize = match req.headers().get(CONTENT_LENGTH) {
        Some(hv) => hv.to_str().unwrap_or("0").parse().unwrap(),
        None => 0,
    };
    let mut buffer = Uuid::encode_buffer();
    let session_id_created = Uuid::new_v4().simple().encode_lower(&mut buffer);

    if content_length == 0 || content_length > max_file_size {
        return HttpResponse::BadRequest().body("file is too big for this");
    }

    if let Ok(Some(mut field)) = payload.try_next().await {
        let filetype: Option<&Mime>  = field.content_type();
        match filetype {
            Some(file_type) => {
                db.into_inner().put_file(FileInfo {
                    uuid: String::from(session_id_created),
                    name: String::from(field.name()),
                    path: String::from("custom path to be added"),
                    content_type: file_type.to_string(),

                }).await;
                let file = File::create(field.name());
                return HttpResponse::Ok().body("file created");
            }
            None => return HttpResponse::BadRequest().body("there is no content type"),
        }
    }

    HttpResponse::Ok().into()
}

#[get("/get_all_file_info/{session_id}")]
async fn get_all_allowed_files_info(path: Path<UuidUrl>, db: Data<Database>) -> impl Responder {
    let all_files = db.get_all_file_info(path.into_inner().session_id).await;
    match all_files {
        Some(files) => HttpResponse::Ok().json(files),
        None => HttpResponse::NoContent().body("i dunno what could be problem")
    }
}

// #[get("/get_file/{session_id}/{file_id}")]
// async fn get_file(path: Path<GetFileUrl>, db: Data<Database>) -> impl Responder {
//     if db.verify_session(path.into_inner().session_id) {
//         //TODO have to figure out how to send "stream" to let user download files
//     }
// }

#[get("/all_allowed_files/{session_id}")]
async fn get_all_allowed(path: Path<UuidUrl>, db: Data<Database>) -> impl Responder {
    let uuid = path.into_inner();
    if db.verify_session(uuid.session_id.clone()).await {
        let res = db.get_all_users_files(uuid.session_id.clone()).await;
        match res {
            Some(files) => return HttpResponse::Ok().json(files),
            None => return HttpResponse::NotFound().body("some how i cant: ")
        }
    }
    HttpResponse::Unauthorized().body("nouh")
}

// #[get("/download_file/{session_id}/{file_id}")]
// async fn download_file(path: Path<GetFileUrl>, db: Data<Database>) -> impl  Responder {
//     HttpResponse::Ok().content_type()
// }

#[post("/set_user/{master_key}")]
async fn set_user(
    loading_user: Json<UserCreateProfile>,
    path: Path<MasterKey>,
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
