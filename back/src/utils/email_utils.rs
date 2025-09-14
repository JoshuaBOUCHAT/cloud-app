use std::{env, sync::LazyLock};

use lettre::Transport;
use lettre::{
    Message, SmtpTransport,
    message::{Mailbox, header::ContentType},
    transport::smtp::authentication::Credentials,
};

use crate::errors::AppResult;

pub static APP_MAIL_BOX: LazyLock<Mailbox> = LazyLock::new(|| {
    env::var("EMAIL")
        .expect("EMAIL not define")
        .parse()
        .unwrap()
});

pub static MAILER: LazyLock<SmtpTransport> = LazyLock::new(|| {
    let password = env::var("EMAIL_TOKEN").expect("EMAIL_TOKEN not define");
    let email = env::var("EMAIL").expect("EMAIL not define");
    let creds = Credentials::new(email, password);

    // Create a connection to our email provider
    // In this case, we are using Namecheap's Private Email
    // You can use any email provider you want
    let mailer = SmtpTransport::relay("smtp.gmail.com")
        .unwrap()
        .credentials(creds)
        .build();
    mailer
});

pub fn send_mail(destination: &str, subject: &str, body: impl Into<String>) -> AppResult<()> {
    println!("sending mail to {}", destination);
    let msg = Message::builder()
        .from(APP_MAIL_BOX.clone())
        .to(destination.parse().expect("Invalid email address"))
        .subject(subject)
        .header(ContentType::TEXT_HTML)
        .body(body.into())?;

    MAILER.send(&msg)?;

    Ok(())
}
