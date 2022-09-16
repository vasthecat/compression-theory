let data = Huffman.read_file "../texts/Тест_6.txt";;
Huffman.compress "hello.gsch1" data;;

let data' = Huffman.decompress "hello.gsch1";;
List.iter (Printf.printf "%c") data';;

