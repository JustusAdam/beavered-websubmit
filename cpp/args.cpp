#include "args.hpp"
#include <iostream>
#include "clap/clap.hpp"

namespace args {

Args parse_args(int argc, char* argv[]) {
    auto matches = clap::App::new_("lecture-qa")
        .version("1.0")
        .author("Your Name")
        .about("Lecture Q&A application")
        .arg(
            clap::Arg::with_name("config")
                .short_('c')
                .long_("config")
                .value_name("FILE")
                .help("Sets a custom config file")
                .takes_value(true)
        )
        .get_matches_from(argc, argv);

    Args args;
    args.config = matches.value_of("config").unwrap_or("config.toml");
    return args;
}

} // namespace args