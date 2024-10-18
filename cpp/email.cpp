#include "email.hpp"
#include "lettre/lettre.hpp"
#include "slog/slog.hpp"

namespace email {

void send_email(const config::Config& config, const std::string& to, const std::string& subject, const std::string& body) {
    // Implementation of send_email
    // This is a placeholder and needs to be properly implemented
    lettre::SmtpTransport transport = lettre::SmtpTransport::new(config.smtp_server())
        .port(config.smtp_port())
        .credentials(lettre::Credentials::new(config.smtp_user(), config.smtp_pass()))
        .build();

    lettre::Message message = lettre::Message::builder()
        .from(config.smtp_from())
        .to(to)
        .subject(subject)
        .body(body)
        .build();

    transport.send(&message);
}

} // namespace email