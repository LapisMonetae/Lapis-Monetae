use lmt_cli_lib::lmt_cli;
use wasm_bindgen::prelude::*;
use workflow_terminal::Options;
use workflow_terminal::Result;

#[wasm_bindgen]
pub async fn load_lmt_wallet_cli() -> Result<()> {
    let options = Options { ..Options::default() };
    lmt_cli(options, None).await?;
    Ok(())
}
