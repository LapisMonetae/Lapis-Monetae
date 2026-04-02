use lmt_cli_lib::{lmt_cli, TerminalOptions};

#[tokio::main]
async fn main() {
    let result = lmt_cli(TerminalOptions::new().with_prompt("$ "), None).await;
    if let Err(err) = result {
        println!("{err}");
    }
}
