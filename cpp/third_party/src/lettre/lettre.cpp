#include "lettre.hpp"

namespace lettre
{

    MessageBuilder &MessageBuilder::from(const std::string &from)
    {
        return *this;
    }

    MessageBuilder &MessageBuilder::to(const std::string &from)
    {
        return *this;
    }

    MessageBuilder &MessageBuilder::subject(const std::string &from)
    {
        return *this;
    }

    MessageBuilder &MessageBuilder::body(const std::string &from)
    {
        return *this;
    }

    Message MessageBuilder::build()
    {
        return Message();
    }

    MessageBuilder Message::builder()
    {
        return MessageBuilder();
    }

    Credentials::Credentials(const std::string &username, const std::string &password) : username(username), password(password) {}

    SmtpTransportBuilder &SmtpTransportBuilder::host(const std::string &host)
    {
        return *this;
    }

    SmtpTransportBuilder &SmtpTransportBuilder::port(int port)
    {
        return *this;
    }

    SmtpTransportBuilder &SmtpTransportBuilder::credentials(const Credentials &host)
    {
        return *this;
    }

    SmtpTransport SmtpTransportBuilder::build()
    {
        return SmtpTransport();
    }

    SmtpTransportBuilder SmtpTransport::builder(const std::string &host)
    {
        return SmtpTransportBuilder();
    }

    void SmtpTransport::send(const Message &message)
    {
    }
}