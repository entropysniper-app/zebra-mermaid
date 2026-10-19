fn main() {
    std::process::exit(zebra_mermaid_cli::run(std::env::args_os().skip(1).collect()));
}
