let read_file filename =
    let chan = In_channel.open_bin filename in
    let str = In_channel.input_all chan in
    In_channel.close chan;
    (Array.of_seq (String.to_seq str));;

let write_file filename data =
    let chan = Out_channel.open_bin filename in
    Array.iter (Out_channel.output_char chan) data;
    Out_channel.close chan;;

let usage_msg = "gsch1 -i -o [-compress] [-decompress]"

let is_compress = ref true
let input_file = ref ""
let output_file = ref ""

let speclist = Arg.align ~limit:30 [
    ("-i", Arg.Set_string input_file, " Установить файл для ввода");
    ("-o", Arg.Set_string output_file, " Установить файл для вывода");
    ("-compress", Arg.Set is_compress, " Режим запаковки");
    ("-decompress", Arg.Clear is_compress, " Режим распаковки");
];;

Arg.parse speclist (fun _ -> ()) usage_msg;;

if !input_file == "" || !output_file == "" then
    Arg.usage speclist usage_msg
else begin
    if !is_compress then
        let data = read_file !input_file in
        let archive = Huffman.compress data in
        write_file !output_file archive;
    else begin
        let archive' = read_file !input_file in
        let data' = Huffman.decompress archive' in
        write_file !output_file data';
    end
end

