use csv::ReaderBuilder;
use once_cell::sync::OnceCell;

// use core::fmt;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::sync::Arc;
use std::collections::{HashMap, HashSet};
use std::error::Error;
use h3server::RequestInCSV;



#[derive(PartialEq)]
enum DiffType {
    DIFFERENT,
    DISTINCT,
}

struct ReportDiff {
    diff: DiffType,
    diff_header: String,
    // h2_vs: HashSet<String>,
    h3_vs: HashSet<String>,
}

static PACKAGE_NAME: OnceCell<String> = OnceCell::new();



static FILE_HANDLE: OnceCell<Arc<File>> = OnceCell::new();

fn init_file() -> Arc<File> {

    let package_name = PACKAGE_NAME.get().expect("failed to get package name in init file");

    let report_dir = std::path::Path::new("report");
    // Create the data directory if it doesn't exist
    if !report_dir.exists() {
        std::fs::create_dir(report_dir).expect("failed to create report dir");
    }
    let txt_name = format!("{}.txt", package_name);
    let file_path = report_dir.join(txt_name).to_string_lossy().to_string();

    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(file_path)
        .expect("Failed to open report txt file");

    Arc::new(file)
}

fn get_file_handle() -> Arc<File> {
    // Initialize the FILE_HANDLE cell if it hasn't been initialized
    FILE_HANDLE.get_or_init(init_file).clone()
}

fn write_to_file(data: &str) -> std::io::Result<()> {
    // Get the singleton file handle
    let mut file = get_file_handle();
    
    // Lock for writing
    let _ = file.write_all(data.as_bytes());
    file.flush()?;

    Ok(())
}



fn read_csv(file_path: &str) -> Result<Vec<RequestInCSV>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let mut reader = ReaderBuilder::new().has_headers(true).from_reader(file);

    let mut records = Vec::new();
    for result in reader.deserialize() {
        let record: RequestInCSV = result?;
        records.push(record);
    }
    Ok(records)
}


fn analyze_requests(reqs_vec: Vec<RequestInCSV>) {
    let mut group_uri = "".to_string();
    let mut common_uri_vec: Vec<RequestInCSV> = Vec::new();
    // get group of requests with same url:
    // compare requests with same url, inherently same app from csv
    for r in reqs_vec {
        if r.uri == group_uri {
            common_uri_vec.push(r);
        } else {
            if common_uri_vec.len() > 0 { compare_h2_h3_request(&common_uri_vec); }
            common_uri_vec.clear();
            group_uri = r.uri.clone();
            common_uri_vec.push(r);
        }
    }
}




// fn analyze_group(reqs_vec: &Vec<RequestInCSV>) {
//     let mut app_name_map: HashMap<String, Vec<RequestInCSV>> = HashMap::new();
//     for r in reqs_vec {
        
//         let r_clone = RequestInCSV {
//             _id: r._id.clone(),
//             app: r.app.clone(),
//             withquic: r.withquic.clone(),
//             uri: r.uri.clone(),
//             method: r.method.clone(),
//             version: r.version.clone(),
//             header: r.header.clone(),
//             bodytype: r.bodytype.clone(),
//             bodyplaintext: r.bodyplaintext.clone(),
//         };

//         match app_name_map.get_mut(&r.app.clone()) {
//             Some(v) => { v.push(r_clone); },
//             None => {
//                 let mut v = Vec::new();
//                 v.push(r_clone);
//                 app_name_map.insert(r.app.clone(), v);
//             },
//         }
//     }

//     // compare difference and log
//     for k in app_name_map.keys() {
//         let req_group = app_name_map.get(k)
//             .expect("Failed Comparision for accessing request group");
//         compare_h2_h3_request(req_group);
//     }
// }


/// req_group for requests with same uri, while running the same app
fn compare_h2_h3_request(req_group: &Vec<RequestInCSV>) {

    let app_using = PACKAGE_NAME.get().expect("failed to get PACKAGE_NAME in report").to_string();

    let uri_using = req_group.get(0).unwrap().uri.clone();
    
    let mut h2_vec: Vec<RequestInCSV> = Vec::new();
    let mut h3_vec: Vec<RequestInCSV> = Vec::new();

    for r in req_group {
        if r.version == "HTTP/3" {
            h3_vec.push(r.clone());
        } else if !r.withquic {
            h2_vec.push(r.clone());
        } else {
            continue;
        }
    }

    // expect difference:
    // h2_header_map: {k1: {v1, v2, v3}}
    // h3_header_map: {k1: {v4}}
    let mut h2_header_map: HashMap<String, HashSet<String>> = HashMap::new();
    let mut h2_ids: Vec<String> = Vec::new();
    for r in h2_vec.clone() {
        let header = serde_json::from_str::<HashMap<String, String>>(&r.header)
            .expect("serde_json bad deserialization on header");
        for (k, v) in header.iter() {
            h2_header_map.entry(k.to_string()).or_insert_with(HashSet::new).insert(v.clone());
        }
        h2_ids.push(r._id.clone());
    }

    let mut h3_header_map: HashMap<String, HashSet<String>> = HashMap::new();
    let mut h3_ids: Vec<String> = Vec::new();
    for r in h3_vec.clone() {
        let header = serde_json::from_str::<HashMap<String, String>>(&r.header)
            .expect("serde_json bad deserialization on header");
        for (k, v) in header.iter() {
            h3_header_map.entry(k.to_string()).or_insert_with(HashSet::new).insert(v.clone());
        }
        h3_ids.push(r._id.clone());
    }


    // compare header difference
    let mut diff_vec: Vec<ReportDiff> = Vec::new();
    
    for (k, h3_vs) in h3_header_map.iter() {
        if h2_header_map.contains_key(k) {
            // check for same key in h2, h3 header, if the value is different
            let h2_vs = h2_header_map.get(k).expect("Error impossible");
            for v in h3_vs {
                if !h2_vs.contains(v) {

                    let mut diff_h3_vs = HashSet::new();
                    diff_h3_vs.insert(v.clone());
                    
                    let diff_part = ReportDiff {
                        diff: DiffType::DIFFERENT,
                        diff_header: k.clone(),
                        // h2_vs: h2_vs.clone(),
                        h3_vs: diff_h3_vs,
                    };
                    diff_vec.push(diff_part);

                }
            }
        } else {
            // check if a key only exists in h3, not in h2
            let diff_part = ReportDiff {
                diff: DiffType::DISTINCT,
                diff_header: k.clone(),
                // h2_vs: HashSet::new(),
                h3_vs: h3_vs.clone(),
            };
            diff_vec.push(diff_part);
        }
    }


    // compare body difference
    let mut h2_body_set = HashSet::new();
    for r in h2_vec {
        if let Some(b) = r.bodyplaintext {
            h2_body_set.insert(b.clone());            
        }
    }

    let mut h3_body_set = HashSet::new();
    for r in h3_vec {
        if let Some(b) = r.bodyplaintext {
            h3_body_set.insert(b.clone());            
        }
    }

    for b in h3_body_set {
        if !h2_body_set.contains(&b) {

            let mut h3_b = HashSet::new();
            h3_b.insert(b.clone());
            let diff_part = ReportDiff {
                diff: DiffType::DISTINCT,
                diff_header: "body".to_string(),
                // h2_vs: h2_body_set.clone(),
                h3_vs: h3_b,
            };
            diff_vec.push(diff_part);
        }
    }

    
    // generate to report
    if diff_vec.len() > 0 {
        write_group_diff(app_using, uri_using, diff_vec);
    }
    
}

fn write_group_diff(app_using: String, uri_using: String, diff_vec: Vec<ReportDiff>) {

    let title = format!("\n\nWhile using app: {}\nUri: {}\n", app_using, uri_using);
    write_to_file(&title).expect("Write failed 1");

    for d in diff_vec {
        if d.diff_header == "body" {
            write_to_file("\nhttp3 request has specific body:\n").expect("Write failed 2");
            if let Some(s) = d.h3_vs.iter().next() {
                write_to_file(s).expect("Write failed 3");
            }
        } else if d.diff == DiffType::DIFFERENT {
            write_to_file("\nhttp3 request has different header:\n").expect("Write failed 4");
            if let Some(s) = d.h3_vs.iter().next() {
                
                // ingore specific difference
                if s.contains("\"Android WebView\";v=\"129\", \"Not=A?Brand\";v=\"8\", \"Chromium\";v=\"129\"") {
                    continue;
                }

                let fmt_s = format!("\nHeader: {}\nValue in http3:\n{}\n", d.diff_header, s);
                write_to_file(&fmt_s).expect("Write failed 5");
            }
        } else {
            write_to_file("\nhttp3 request has distinct header:\n").expect("Write failed 6");
            let fmt_1 = format!("\nHttp3 specific Header: {}\n", d.diff_header);
            write_to_file(&fmt_1).expect("Write failed 7");
            write_to_file("\nCaptured value of this header:\n").expect("Write failed 8");
            for v in d.h3_vs {
                let fmt_2 = format!("{}\n", v);
                write_to_file(&fmt_2).expect("Write failed 8");
            }
        }
        write_to_file("\n\n= = = = = = = = = = = = = = = = = = = =\n\n").expect("Write failed 9");
    }

    write_to_file("\n\n+ + + + + + + + + + + + + + + + + + + +\n\n").expect("Write failed 10");
    
}




fn main() {

    // get package name
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        println!("need 2 args for the generating csv, the second arg as package name");
        return;
    }
    let package_name = args[1].clone();
    PACKAGE_NAME.get_or_init(|| {
        package_name.clone()
    });

    // get csv file
    let data_dir = std::path::Path::new("csvdata");
    let csv_name = format!("{}.csv", &package_name);
    let csv_path = data_dir.join(csv_name);
    match read_csv(&csv_path.to_string_lossy().to_string()) {
        Ok(records) => {
            println!("Complete deserializing csv table");
            // todo: add progress bar if you can
            analyze_requests(records);
        }
        Err(e) => eprintln!("Error reading CSV file: {}", e),
    }
}