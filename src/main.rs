mod canisterlist;
extern crate candid;
use serde::Deserialize;
use serde_json;
use serde_json::to_string_pretty;
use std::ops::Index;
use std::process::Command;
use std::hash::{Hash, Hasher};

#[derive(Debug, Deserialize)]
struct TraitType {
    trait_type: String,
    value: String,
}

fn main() {
    let (trait_object_array, trait_array) = fetch_canister_data("Ktlvk-giaaa-aaaap-aayja-cai".to_string());
    let traits_value = canister_data_to_traits_value(trait_object_array,trait_array.clone());
    let (traits_count, traits_freq) = get_traits_count_freq_number(reverse_mat(traits_value));
    let rarity_mat = rare_calc(traits_freq);
    let mut rarity_score = score_calc(rarity_mat);
    let mut rarity_rank = rare_rank(rarity_score.clone());
    rarity_score = add_max_min_minus_to_rarity_score(rarity_score); 
    println!("{:#?}",rarity_score);
}

fn fetch_canister_data(input: String) -> (Vec<std::collections::HashMap<String, String>> , Vec<String>) {
    let mut trait_object_array: Vec<std::collections::HashMap<String, String>> = Vec::new();
    let mut trait_array: Vec<String> = Vec::new();
    let output = Command::new("dfx")
        .arg("canister")
        .arg("call")
        .arg("--network")
        .arg("ic")
        .arg(input)
        .arg("getTokens")
        .output()
        .expect("failed to deploy websites");

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let pretty_string = to_string_pretty(&stdout).expect("can't convert string");

        let string_data = pretty_string.replace("\\\\22", r#"""#);
        let mut data = string_data.as_str();

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
            //success split one nft trait object
            let res;
            (res, _) = record.split_at(end + 1);
            let json_array: Vec<TraitType> = serde_json::from_str(res).unwrap();
            let mut my_object = std::collections::HashMap::new();
            for item in json_array {
                let key = item.trait_type;
                if !trait_array.contains(&key.clone()) {
                    trait_array.push(key.clone());
                }
                my_object.insert(key.clone(), item.value);
            }
            trait_object_array.push(my_object);
            cnt += 1;
            if index == 0 {
                break;
            }
            pointer += 1;
        }
    } 
    (trait_object_array, trait_array)
}

fn canister_data_to_traits_value(canister_data: Vec<std::collections::HashMap<String, String>> ,trait_list: Vec<String>) -> std::vec::Vec<std::vec::Vec<String>> {
    let mut result: Vec<Vec<String>> = Vec::new();
    for json in canister_data {
        let mut sub_result:Vec<String> = Vec::new();
        for nft_trait in trait_list.clone() {
            let temp_trait = nft_trait.clone();
            if json.contains_key(&temp_trait) {
                let tmp_json = json.clone();
                let tmp_trait = temp_trait.clone();
                let trait_value = tmp_json.get(&tmp_trait);
                // .unwrap(); 
                let default_value = "NA".to_string();
                sub_result.push(trait_value.unwrap_or(&default_value).to_string());
            }
            else {
                sub_result.push("NA".to_string());
            }
        }
        result.push(sub_result);
    }
    result
}

fn reverse_mat(input: Vec<Vec<String>>) -> Vec<Vec<String>> {
    let mut output: Vec<Vec<String>> = Vec::new();
    let row_len = input[0].len();
    let mut index = 0;
    while index < row_len {
        let col: Vec<String> = Vec::new();
        output.push(col);
        index += 1;
    }
    for row in input {
        for (index, value) in row.iter().enumerate() {
            let mut temp_col:Vec<String> = output[index].clone();
            temp_col.push(value.to_string());
            output[index] = temp_col;
        }
    }
    output
}

fn get_traits_count_freq_number(input: Vec<Vec<String>>) -> (Vec<Vec<i32>>, Vec<Vec<f64>>) {
    let mut traits_count: Vec<Vec<i32>> = Vec::new();
    let mut traits_freq: Vec<Vec<f64>> = Vec::new();
    let no_tokens:i32 = input[0].len() as i32;
    for col in input {
        let mut count_col: Vec<i32> = Vec::new();
        let mut freq_col: Vec<f64> = Vec::new();
        let temp_col = col.clone();
        for value in temp_col {
            let tmp_col = col.clone();
            let count:i32 = tmp_col.iter().filter(|&v| *v == value).count() as i32;
            count_col.push(count.clone());
            freq_col.push(count.clone() as f64/no_tokens as f64);
        }
        traits_count.push(count_col);
        traits_freq.push(freq_col);
    }
    (traits_count, traits_freq)
}

fn rare_calc(input: Vec<Vec<f64>>) -> Vec<Vec<f64>> {
    let mut output: Vec<Vec<f64>> = Vec::new();
    let mut min_col: Vec<f64> = Vec::new();
    let mut max_col: Vec<f64> = Vec::new();
    let mut arithmetic_col: Vec<f64> = Vec::new();
    let mut harmonic_col: Vec<f64> = Vec::new();
    let mut geometric_col: Vec<f64> = Vec::new();

    for (index_row, _) in input[0].iter().enumerate() {
        let mut param_array: Vec<f64> = Vec::new();
        for (index_col, _) in input.iter().enumerate() {
            param_array.push(input[index_col][index_row]);
        }
        min_col.push(*param_array.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap());
        max_col.push(*param_array.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap());
        arithmetic_col.push(wpmean(param_array.clone(), 1));
        harmonic_col.push(wpmean(param_array.clone(), -1));
        geometric_col.push(wpmean(param_array.clone(), 0));
    }
    output.push(min_col);
    output.push(max_col);
    output.push(arithmetic_col);
    output.push(harmonic_col);
    output.push(geometric_col);
    output
}

fn score_calc(input: Vec<Vec<f64>>) -> Vec<Vec<f64>> {
    let mut output: Vec<Vec<f64>> = Vec::new();
    for col in input {
        let min = col.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();
        let max = col.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();
        let mut new_col: Vec<f64> = Vec::new();
        let temp_col = col.clone();
        for value in temp_col {
            if min == max {
                new_col.push(0.0 as f64);
            }
            else {
                new_col.push(( value - min ) / ( max - min ));
            }
        }
        output.push(new_col);
    }
    output
}

fn wpmean(input: Vec<f64>, p: i32) -> f64 {
    let mut output: f64 = 0.0 as f64;
    let lenth: f64 = input.clone().len() as f64;
    if p==0 {
        let mut multi: f64 = 1.0 as f64;
        for val in input {
            multi *= val;
        }
        output = f64::powf(multi, 1.0 as f64/lenth);
    }
    else {
        let mut mean: f64 = 0.0 as f64;
        for val in input {
            mean += f64::powf(val, p as f64);
        }
        mean /= lenth as f64;
        output = f64::powf(mean, 1.0 as f64 / p as f64);
    }
    output
}

fn rare_rank(input: Vec<Vec<f64>>) -> Vec<Vec<f64>> {
    let mut output: Vec<Vec<f64>> = Vec::new();
    for col in input {
        let mut ranks: Vec<f64> = col.clone();
        ranks.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let mut rank_values: Vec<f64> = Vec::new();
        let mut current_rank = 1.0;
        let mut prev_value = ranks[0];

        for value in ranks {
            if value != prev_value {
                current_rank += 1.0;
            }
            rank_values.push(current_rank);
            prev_value = value;
        }

        output.push(rank_values);
    }
    output
}

fn add_max_min_minus_to_rarity_score(input: Vec<Vec<f64>>) -> Vec<Vec<f64>> {
    let mut output: Vec<Vec<f64>> = Vec::new();
    for col in input.clone() {
        output.push(col.clone());
    }
    let mut new_col: Vec<f64> = Vec::new();
    let temp_input:Vec<Vec<f64>> = input.clone(); 
    for (index, _) in temp_input[0].iter().enumerate() {
        new_col.push(input[1][index] - input[0][index]);
    }
    output.push(new_col);
    output
}

// fn trait_independence(input: Vec<Vec<f64>>) -> Vec<Vec<f64>> {
//     let mut output: Vec<Vec<f64>> = Vec::new();
//     let out_array: Vec<i32> = (0 as i32 ..=input.len() as i32 - 2 as i32).collect();
//     let in_array: Vec<i32> = (1 as i32 ..=input.len() as i32 - 1 as i32).collect();
//     for out_val in out_array {
//         for in_val in in_array {
//         }
//     }
//     output
// }

// fn trait_cramers_v(input: Vec<Vec<f64>>) -> Vec<Vec<f64>> {
//     let mut output: Vec<Vec<f64>> = Vec::new();
//     output
// }

fn independent_test(first: Vec<f64>, second: Vec<f64>) -> f64 {
    let mut output = 0.0 as f64;
    let first_unique_array: Vec<f64> = get_unique_array(first.clone());
    let second_unique_array: Vec<f64> = get_unique_array(second.clone());
    let mut key_set:Vec<Vec<i32>> = vec![vec![0; first_unique_array.len()]; second_unique_array.len()];
    for index in 0..first.len()-1 {
        if let Some(col)= second_unique_array.iter().position(|&x| x == second[index]) {
            if let Some(row)= first_unique_array.iter().position(|&x| x == first[index]) {
                key_set[col][row] += 1;
            }
        }
    }
    // calulate chi2_contingency
    
    output
}

fn get_unique_array(input: Vec<f64>) -> Vec<f64> {
    let mut output: Vec<f64> = Vec::new();
    for val in input {
        if !output.contains(&val) {
            output.push(val);
        }
    }
    output
} 