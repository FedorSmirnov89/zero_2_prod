use actix_web::http::header::LOCATION;
use actix_web::HttpResponse;

pub fn e500<Err>(e: Err) -> actix_web::Error
where
    Err: std::fmt::Debug + std::fmt::Display + 'static,
{
    actix_web::error::ErrorInternalServerError(e)
}

pub fn see_other(location: &str) -> HttpResponse {
    HttpResponse::SeeOther()
        .insert_header((LOCATION, location))
        .finish()
}