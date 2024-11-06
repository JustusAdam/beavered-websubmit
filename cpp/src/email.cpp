#include "email.hpp"
#include "lettre/lettre.hpp"
#include "slog/slog.hpp"

namespace email
{

    void send(const std::string &sender, const std::vector<std::string> &to, const std::string &subject, const std::string &body)
    {
        lettre::SmtpTransport transport = lettre::SmtpTransport::builder()
                                              .build();

        auto builder = lettre::Message::builder();
        builder.from(sender);
        for (const auto &recipient : to)
        {
            builder.to(recipient);
        }
        lettre::Message message = builder
                                      .subject(subject)
                                      .body(body)
                                      .build();

        transport.send(message);
    }

} // namespace email