use actix_multipart::form::MultipartForm;
use actix_web::{get, post, patch, HttpResponse, Responder, HttpRequest};
use uuid::Uuid;
use actix_web::web::{Json, Path, Data};
use crate::database::Database;
use crate::models::file_models::{
    GetFileUrl, MasterKey, UploadData, UploadForm,
    UserCreateProfile, UserLogin, UserSessionIdMatch, UuidUrl
};
use crate::models::{FileInfo, UserInfo};

static MAX_SIZE_ALLOWED: usize = 50_000_000;

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
        let res = db.get_all_users_files(uuid.session_id.clone()).await;
        match res {
            Some(files) => return HttpResponse::Ok().json(files),
            None => return HttpResponse::NotFound().body("some how i cant: "),
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
