mod huffman;
mod priority_queue;

fn main() -> std::io::Result<()> {
    let data = vec![1, 2, 3, 4];
    let archive = huffman::compress(&data);
    println!("{:?}", archive);

    // std::fs::write("hi.txt", &data)?;
    // std::fs::write("hello.gsch1", &archive)?;

    let new_data = huffman::decompress(&archive);
    println!("{:?}", new_data);

    return Ok(());
}
