use actix_session::SessionExt;
use actix_web::{
    Error, HttpResponse,
    dev::{Service, ServiceRequest, ServiceResponse},
};

pub async fn auth_middleware<S>(req: ServiceRequest, next: S) -> Result<ServiceResponse, Error>
where
    S: Service<ServiceRequest, Response = ServiceResponse, Error = Error>,
{
    let session = req.get_session();

    let maybe_user_id = session.get::<i32>("user_id").unwrap();
    let maybe_verified = session.get::<bool>("verified").unwrap();

    if let (Some(_id), Some(true)) = (maybe_user_id, maybe_verified) {
        // utilisateur connecté et vérifié
        next.call(req).await
    } else {
        // pas connecté ou non vérifié
        let (req, _pl) = req.into_parts();
        let msg = if maybe_user_id.is_none() {
            "Not connected"
        } else {
            "Account not verified"
        };

        Ok(ServiceResponse::new(
            req,
            HttpResponse::Unauthorized().json(msg),
        ))
    }
}
