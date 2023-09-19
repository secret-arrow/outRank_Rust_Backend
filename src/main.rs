extern crate candid;
use serde::Deserialize;
use serde_json;
use serde_json::to_string_pretty;
use std::process::Command;
use statrs::distribution::ChiSquared;
use statrs::distribution::ContinuousCDF;
use actix_cors::Cors;
use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use serde::Serialize;
use std::fs;

#[derive(Debug, Deserialize)]
struct TraitType {
    trait_type: String,
    value: String,
}

#[derive(Serialize)]
struct MyResponse {
    rarity_rank: Vec<Vec<f64>>,
    rarity_score: Vec<Vec<f64>>,
    trait_independence: Vec<Vec<f64>>,
    trait_cramers_v: Vec<Vec<f64>>,
    trait_normalize: Vec<Vec<f64>>,
    trait_array: Vec<String>
}

#[derive(Deserialize)]
struct MyQueryParams {
    canister_id: String
}

#[get("/get_canister_data")]
async fn my_endpoint(query_params: web::Query<MyQueryParams>) -> impl Responder {
    let canister_id = &query_params.canister_id;
    // let (trait_object_array, trait_array) = fetch_canister_data(canister_id.to_owned());
    let (trait_object_array, trait_array) = fetch_test_data();
    println!("fetch data");
    let traits_value = canister_data_to_traits_value(trait_object_array,trait_array.clone());
    println!("traits_value");
    let (traits_count, traits_freq) = get_traits_count_freq_number(reverse_mat(traits_value.clone()));
    println!("traits_count_freq");
    let rarity_mat = rare_calc(traits_freq.clone());
    println!("rarity_mat");
    let mut rarity_score = score_calc(rarity_mat);
    println!("rarity_score");
    let rarity_rank = rare_rank(rarity_score.clone());
    println!("rarity_rank");
    rarity_score = add_max_min_minus_to_rarity_score(rarity_score); 
    println!("rarity_score_new");
    let trait_independence = trait_independence(traits_freq.clone());
    println!("trait_independence");
    let trait_cramers_v = trait_cramers_v(traits_freq.clone());
    println!("trait_cramers_v");
    let trait_normalize = trait_normalize(reverse_mat(traits_value.clone()), traits_count, traits_freq);
    println!("trait_normalize");
    let response = MyResponse {
        rarity_rank: rarity_rank,
        rarity_score: rarity_score,
        trait_independence: trait_independence,
        trait_cramers_v: trait_cramers_v,
        trait_normalize: trait_normalize,
        trait_array: trait_array
    };
    HttpResponse::Ok().json(response)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {

    HttpServer::new(|| {
        App::new()
            .wrap(
                Cors::default()
                    .allow_any_origin()
                    .allowed_methods(vec!["GET", "POST"])
                    .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
                    .max_age(3600),
            )
            .service(my_endpoint)
    })
    .bind("127.0.0.1:8000")?
    .run()
    .await
}

fn fetch_test_data() -> (Vec<std::collections::HashMap<String, String>> , Vec<String>) {
    let mut trait_object_array: Vec<std::collections::HashMap<String, String>> = Vec::new();
    let mut trait_array: Vec<String> = Vec::new();

    let test_data_string = fs::read_to_string("testdata.txt").unwrap();

    let string_data = test_data_string.replace("\"", "");
    let mut data = string_data.as_str();

    while 1 == 1 {
        let start = data.find("{").unwrap_or(0);

        if start == 0 {
            break;
        }

        (_, data) = data.split_at(start+1);
        let end = data.find("}").unwrap_or(0);
        
        let mut res;
        (res, data) = data.split_at(end);

        let mut my_object = std::collections::HashMap::new();

        while 1 == 1 {
            let key_val;
            let spos = res.find(",").unwrap_or(0);

            if spos == 0 {
                key_val = res;
            }
            else{
                (key_val, _) = res.split_at(spos);
                (_, res) = res.split_at(spos+1);
            }
            let mpos = key_val.find(":").unwrap_or(0);
            let (key, _) = key_val.split_at(mpos);
            let (_, val) = key_val.split_at(mpos+1);

            my_object.insert(key.to_string(), val.to_string());

            if !trait_array.contains(&key.to_string()) {
                trait_array.push(key.to_string());
            }

            if spos == 0 {
                break;
            }
        }
        trait_object_array.push(my_object);
    }

    (trait_object_array, trait_array)
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

fn get_traits_count_freq_number(input: Vec<Vec<String>>) -> (Vec<Vec<f64>>, Vec<Vec<f64>>) {
    let mut traits_count: Vec<Vec<f64>> = Vec::new();
    let mut traits_freq: Vec<Vec<f64>> = Vec::new();
    let no_tokens:i32 = input[0].len() as i32;
    for col in input {
        let mut count_col: Vec<f64> = Vec::new();
        let mut freq_col: Vec<f64> = Vec::new();
        let temp_col = col.clone();
        for value in temp_col {
            let tmp_col = col.clone();
            let count:f64 = tmp_col.iter().filter(|&v| *v == value).count() as f64;
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
    let output: f64;
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

        for value in col {
            let rank = ranks.iter().position(|&x| x == value).unwrap() as f64;
            rank_values.push(rank);
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

fn trait_independence(input: Vec<Vec<f64>>) -> Vec<Vec<f64>> {
    let mut output: Vec<Vec<f64>> = vec![vec![0.0; input.len()]; input.len()];
    for out_val in 0 ..=input.len() -2 {
        for in_val in out_val+1 ..=input.len() -1 {
            let key_set= independent_test(input[out_val].clone(), input[in_val].clone());
            let (chi2, dof) = calculate_chi2_dof(key_set);
            let chi_squared = ChiSquared::new(dof as f64).unwrap();
            let critical_value = chi_squared.inverse_cdf(0.95);
            output[in_val][out_val] = format!("{:.1$}", chi2, 4).parse::<f64>().unwrap();
            output[out_val][in_val] = format!("{:.1$}", critical_value, 4).parse::<f64>().unwrap();
        }
    }
    output
}

fn trait_cramers_v(input: Vec<Vec<f64>>) -> Vec<Vec<f64>> {
    let mut output: Vec<Vec<f64>> = vec![vec![0.0; input.len()]; input.len()];
    for out_val in 0 ..=input.len() -2 {
        for in_val in out_val+1 ..=input.len() -1 {
            let key_set= independent_test(input[out_val].clone(), input[in_val].clone());
            let (chi2, _) = calculate_chi2_dof(key_set.clone());
            let mut sum = 0.0;
            for col in key_set.clone() {
                for item in col {
                    sum += item;
                }
            }
            let dimension: f64; 
            if key_set.len() > key_set[0].len() {
                dimension = key_set[0].len() as f64 - 1.0;
            }
            else{
                dimension = key_set.len() as f64 - 1.0;
            }
            let sqrt = ((chi2/sum)/dimension).sqrt();
            output[in_val][out_val] = format!("{:.1$}", sqrt, 4).parse::<f64>().unwrap();
        }
    }
    output
}

fn independent_test(first: Vec<f64>, second: Vec<f64>) -> Vec<Vec<f64>> {
    let first_unique_array: Vec<f64> = get_unique_array(first.clone());
    let second_unique_array: Vec<f64> = get_unique_array(second.clone());
    let mut key_set:Vec<Vec<f64>> = vec![vec![0.0; first_unique_array.len()]; second_unique_array.len()];
    for index in 0..first.len() {
        if let Some(col)= second_unique_array.iter().position(|&x| x == second[index]) {
            if let Some(row)= first_unique_array.iter().position(|&x| x == first[index]) {
                key_set[col][row] += 1.0;
            }
        }
    }
    key_set
}

fn calculate_chi2_dof(input: Vec<Vec<f64>>) -> (f64, usize) {
    let row_totals: Vec<f64> = input.iter().map(|row| row.iter().sum()).collect();
    let column_totals: Vec<f64> = (0..input[0].len())
        .map(|col| input.iter().map(|row| row[col]).sum())
        .collect();
    let grand_total: f64 = row_totals.iter().sum();
    let expected: Vec<Vec<f64>> = input
        .iter()
        .enumerate()
        .map(|(col_index, col)| {
            col.iter()
                .enumerate()
                .map(|(row, _)| (row_totals[col_index] * column_totals[row]) / grand_total)
                .collect()
        })
        .collect();
    let chi2_statistic = chi2_contingency(&input, &expected);
    let dof: usize = (input.len() - 1) * (input[0].len() - 1);
    (chi2_statistic, dof)
}

fn chi2_contingency(input: &[Vec<f64>], expected: &[Vec<f64>]) -> f64 {
    let mut chi2_statistic = 0.0;
    for (i, row) in input.iter().enumerate() {
        for (j, &input_value) in row.iter().enumerate() {
            let expected_value = expected[i][j] as f64;
            chi2_statistic += (input_value as f64 - expected_value).powi(2) / expected_value;
        }
    }
    chi2_statistic
}

fn get_unique_array(input: Vec<f64>) -> Vec<f64> {
    let mut output: Vec<f64> = Vec::new();
    for val in input {
        if !output.contains(&val) {
            output.push(val);
        }
    }
    output.sort_by(|a, b| a.partial_cmp(b).unwrap());
    output
} 

fn trait_normalize(traits_value: Vec<Vec<String>>, traits_count: Vec<Vec<f64>>, traits_freq: Vec<Vec<f64>>) -> Vec<Vec<f64>> {
    let mut w: Vec<i32> = Vec::new();
    for col in traits_value.clone() {
        let mut value_list: Vec<String> = Vec::new();
        for val in col {
            if !value_list.contains(&val) {
                value_list.push(val);
            }
        }
        w.push(value_list.len() as i32);
    } 
    let mut output: Vec<Vec<f64>> = Vec::new();
    let style_list: Vec<String> = vec!["geometric".to_string(), "harmonic".to_string(), "arithmetic".to_string()];
    let counts_control_list: Vec<bool> = vec![true, false];
    for style in style_list {
        for counts_control in counts_control_list.clone() {
            let counts;
            if counts_control == false {
                counts = traits_count.clone();
            }
            else {
                counts = traits_freq.clone();
            }
            let temp_nor:Vec<f64> = normalize_calc(w.clone(),counts,style.clone(),counts_control.clone());
            let max = temp_nor.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();
            let min = temp_nor.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();
            let mut result: Vec<f64> = Vec::new(); 
            for (_, item) in temp_nor.iter().enumerate() {
                result.push(( item - min ) / (max - min));
            }
            output.push(result);
        }
    }
    output
}

fn normalize_calc(w: Vec<i32>, counts: Vec<Vec<f64>>, style: String, counts_control: bool) -> Vec<f64> {
    let mut weights: Vec<Vec<f64>> = Vec::new();
    let mut weight_sum: f64 = 0.0;
    if style == "geometric" && counts_control == true {
        for element in w {
            let temp = element as f64;
            weights.push(vec![temp.recip(); counts[0].len()]);
            weight_sum += temp.recip();
        }
    }
    else {
        for element in w {
            weights.push(vec![element as f64; counts[0].len()]);
            weight_sum += element as f64;
        }
    }

    let mut normalized_rarity: Vec<f64> = Vec::new();

    if style == "geometric" {
        if counts_control == false {
            for row_index in 0..counts[0].len() {
                let mut sum = 0.0;
                for col_index in 0..counts.len() {
                    sum += counts[col_index][row_index].ln()*weights[col_index][row_index];
                }
                sum = sum.powf(1.0/weight_sum);
                normalized_rarity.push(f64::exp(sum)/counts.len() as f64);
            }
        }
        else{
            for row_index in 0..counts[0].len() {
                let mut sum = 1.0;
                for col_index in 0..counts.len() {
                    sum *= counts[col_index][row_index].powf(weights[col_index][row_index]);
                }
                sum = sum.powf(1.0/weight_sum);
                normalized_rarity.push(sum);
            }
        }
    }
    else if style == "harmonic" {
        for row_index in 0..counts[0].len() {
            let mut sum = 0.0;
            for col_index in 0..counts.len() {
                sum += weights[col_index][row_index] / counts[col_index][row_index];
            }
            sum = sum/weight_sum;
            normalized_rarity.push(sum.powf(-1.0)*weight_sum);
        }
    }
    else {
        for row_index in 0..counts[0].len() {
            let mut sum = 0.0;
            for col_index in 0..counts.len() {
                sum += counts[col_index][row_index] * weights[col_index][row_index];
            }
            sum = sum/weight_sum;
            normalized_rarity.push(sum);
        }
    }

    normalized_rarity
}