use lettre::sendmail::{self, SendmailTransport};
use lettre::{SendableEmail, Transport};
use lettre_email::Email;

#[cfg_attr(not(feature = "v-ann-lib"), paralegal::marker{ sink, arguments = [3, 4] })]
#[cfg_attr(not(feature = "v-ann-lib"), paralegal::marker{ scopes, arguments = [2] })]
pub(crate) fn my_send(
    log: slog::Logger,
    sender: String,
    recipients: Vec<String>,
    subject: String,
    text: String,
) -> Result<(), lettre::sendmail::error::Error> {
    let mut mailer = SendmailTransport::new();

    let mut builder = Email::builder()
        .from(sender.clone())
        .subject(subject.clone())
        .text(text.clone());
    for recipient in recipients {
        builder = builder.to(recipient);
    }

    //debug!(log, "Email: Subject {}\nText: {}!", subject, text);

    let email = builder.build();
    match email {
        Ok(result) => mailer.send(result.into())?,
        // Cannot print the error here, since it may leak information
        Err(_) => {
            println!("couldn't construct email");
        }
    }

    Ok(())
}
