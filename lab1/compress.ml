let read_file filename =
    let chan = In_channel.open_bin filename in
    let str = In_channel.input_all chan in
    In_channel.close chan;
    (Array.of_seq (String.to_seq str));;

let write_file filename data =
    let chan = Out_channel.open_bin filename in
    Array.iter (Out_channel.output_char chan) data;
    Out_channel.close chan;;

let data = read_file "../texts/Тест_6.txt";;
let archive = Huffman.compress data;;
write_file "hello.gsch1" archive;;

let archive' = read_file "hello.gsch1";;
let data' = Huffman.decompress archive';;
write_file "decompressed.txt" data';;

