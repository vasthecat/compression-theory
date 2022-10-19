mod huffman;
mod priority_queue;

fn main() {
    let data = vec![1, 2, 3, 4];
    let archive = huffman::compress(&data);

    std::fs::write("hi.txt", &data);
    std::fs::write("hello.gsch1", &archive);
    println!("{:?}", archive);
}
