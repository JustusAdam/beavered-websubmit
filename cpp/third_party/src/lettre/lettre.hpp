#pragma once

namespace lettre {

class Message;

class MessageBuilder {
public:
    MessageBuilder& from(const std::string& from);

    MessageBuilder& to(const std::string& to);

    MessageBuilder& subject(const std::string& subject);

    MessageBuilder& body(const std::string& body);

    Message build();
};

class Message {
public: 
    static MessageBuilder builder();
};

class SmtpTransport;

class Credentials {
private: 
    std::string username;
    std::string password;
public: 
    Credentials(const std::string& username, const std::string& password);
};

class SmtpTransportBuilder {
public:
    SmtpTransportBuilder& host(const std::string& host);

    SmtpTransportBuilder& port(int port);

    SmtpTransportBuilder& credentials(const Credentials& credentials);

    SmtpTransport build();
};

class SmtpTransport {
public: 
    static SmtpTransportBuilder builder(const std::string& host);

    void send(const Message& message);
};

}