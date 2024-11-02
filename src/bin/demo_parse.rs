

fn main() {
    let file_name = "demodump";

    match read_protobuf_messages_from_file(&file_name) {
        Ok(msgs) => {println!("msg has {}", msgs.len())},
        Err(e) => {println!("err: {}", e)},
    }
}