use actix_web::FromRequest;
use async_trait::async_trait;
use std::pin::Pin;

use crate::auth::bearer_manager::Claims;

pub struct FromClaim<T: TryFromClaim>(pub T);

#[async_trait]
pub trait TryFromClaim: Sized + Send {
    async fn try_from_claim(claims: &Claims) -> Result<Self, actix_web::Error>;
}

impl<T> FromRequest for FromClaim<T>
where
    T: TryFromClaim,
{
    type Error = actix_web::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;

    fn from_request(
        req: &actix_web::HttpRequest,
        _payload: &mut actix_web::dev::Payload,
    ) -> Self::Future {
        let req = req.clone(); // Actix HttpRequest est cheap à clone
        Box::pin(async move {
            let claims = Claims::extract(&req).await?;
            let from_claim = T::try_from_claim(&claims).await?;
            Ok(FromClaim(from_claim))
        })
    }
}
