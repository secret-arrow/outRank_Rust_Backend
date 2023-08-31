mod canisterlist;

use std::process::Command;
use serde_json::to_string_pretty;


#[tokio::main]
async fn main() {
    let canister_list = canisterlist::get_canister_list();

    let mut trait_list:Vec<String> = Vec::new();

    for canister in canister_list{
        let output = Command::new("dfx")
            .arg("canister")
            .arg("call")
            .arg("--network")
            .arg("ic")
            .arg(canister)
            .arg("getTokens")
            .output()
            .expect("failed to deploy websites");

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let pretty_string = to_string_pretty(&stdout).expect("can't convert string");
            trait_list.push(pretty_string);
            // println!("{}", stdout);
        }else {
            // let stderr = String::from_utf8_lossy(&output.stderr);
            // println!("Command failed: {}", stderr);
            trait_list.push(String::from("None"));
        }
    }
}


//trait normalization comparizon
async fn trait_normalization(nft_collections_trait: String) -> Vec<u32>{
    //normalize each collections traits
}

//QR codes
async fn qrcode_generate(nft_collections_trait: String) -> Vec<u32>{
    //generate qrcode
}

//Barcodes
async fn barcode_generate(nft_collections_trait: String) -> Vec<u32>{
    //generate barcode
}
