mod canisterlist;
extern crate candid;
use serde::Deserialize;
use serde_json;
use serde_json::to_string_pretty;
use std::process::Command;

#[derive(Debug, Deserialize)]
struct TraitType {
    trait_type: String,
    value: String,
}

fn main() {
    let canister_list = canisterlist::get_canister_list();

    let mut trait_list: Vec<Vec<std::collections::HashMap<String, String>>> = Vec::new();

    for canister in canister_list {
        // if canister == "jx6g3-miaaa-aaaap-aamwa-cai" {
        let output = Command::new("dfx")
            .arg("canister")
            .arg("call")
            .arg("--network")
            .arg("ic")
            .arg(canister.clone())
            .arg("getTokens")
            .output()
            .expect("failed to deploy websites");

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let pretty_string = to_string_pretty(&stdout).expect("can't convert string");

            let string_data = pretty_string.replace("\\\\22", r#"""#);
            let mut data = string_data.as_str();
            let mut trait_object_array: Vec<std::collections::HashMap<String, String>> = Vec::new();

            let mut index = data.find("record { 0 :").unwrap_or(0);
            (_, data) = data.split_at(index);
            let mut pointer = 1;
            let mut cnt = 0;
            while 1 == 1 {
                if pointer < 1000 {
                    index = data
                        .find(format!("record {{ {} :", pointer).as_str())
                        .unwrap_or(0);
                } else {
                    let mut str = pointer.to_string();
                    str.insert_str(1, "_");
                    index = data
                        .find(format!("record {{ {} :", str).as_str())
                        .unwrap_or(0);
                }
                let mut record;
                if index == 0 {
                    record = data;
                } else {
                    (record, data) = data.split_at(index);
                }
                let start = record.find("[").unwrap_or(0);
                (_, record) = record.split_at(start);
                let end = record.find("]").unwrap_or(0);
                if start == 0 || end == 1 {
                    if index == 0 {
                        break;
                    } else {
                        pointer += 1;
                        continue;
                    }
                }
                let res;
                (res, _) = record.split_at(end + 1);
                let json_array: Vec<TraitType> = serde_json::from_str(res).unwrap();
                let mut my_object = std::collections::HashMap::new();
                for item in json_array {
                    my_object.insert(item.trait_type, item.value);
                }
                // let json_value: serde_json::Value = serde_json::to_value(my_object).unwrap();
                // println!("{:#?}", json_value["Background"]);
                trait_object_array.push(my_object);
                cnt += 1;
                if index == 0 {
                    break;
                }
                pointer += 1;
            }
            // println!("{:#?}", trait_object_array);
            trait_list.push(trait_object_array);
            println!(
                "Canister = {} , Total Count = {} , RealCount = {}",
                canister, pointer, cnt
            );
        } else {
            // let stderr = String::from_utf8_lossy(&output.stderr);
            println!("Canister = {} , Failed to fetch canister data", canister);
            trait_list.push([].to_vec());
        }
        // }
    }
    // println!("{:#?}", trait_list);
}

// //trait normalization comparizon
// async fn trait_normalization(nft_collections_trait: String) -> Vec<u32>{
//     //normalize each collections traits
// }

// //QR codes
// async fn qrcode_generate(nft_collections_trait: String) -> Vec<u32>{
//     //generate qrcode
// }

// //Barcodes
// async fn barcode_generate(nft_collections_trait: String) -> Vec<u32>{
//     //generate barcode
// }
