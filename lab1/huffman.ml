type bit = Zero | One
let bit_from_int = function
    | 0 -> Zero
    | _ -> One
let int_from_bit = function
    | Zero -> 0
    | One -> 1

let read_file filename =
    let chan = In_channel.open_bin filename in
    let str = In_channel.input_all chan in
    In_channel.close chan;
    (Array.of_seq (String.to_seq str));;

type 'a tree = Leaf of 'a | Node of 'a tree * 'a tree

let get_weights xs =
    let mp = Hashtbl.create 256 in
    for i = 0 to Array.length xs - 1 do
        match Hashtbl.find_opt mp xs.(i) with
        | None -> Hashtbl.add mp xs.(i) 1
        | Some v -> Hashtbl.replace mp xs.(i) (v + 1)
    done;
    let l = Hashtbl.fold (fun k v acc -> (k, v) :: acc) mp [] in
    let l' = List.sort (fun (k, v) (k', v') -> Int.min v v') l in
    List.mapi (fun i (x, _) -> (x, i)) l';;

(* TODO: Избавиться от Option.get *)
let rec from_queue queue =
    if Priority_queue.is_single queue
    then
        let (_, value, _) = Option.get (Priority_queue.extract queue)
        in value
    else
        let (p1, v1, queue) = Option.get (Priority_queue.extract queue) in
        let (p2, v2, queue) = Option.get (Priority_queue.extract queue) in
        from_queue (Priority_queue.insert queue (p1 + p2) (Node (v1, v2)));;

let get_code tree =
    let rec aux acc run = function
        | Leaf value -> (value, List.rev run) :: acc
        | Node (left, right) ->
            (aux acc (Zero :: run) left) @ (aux acc (One :: run) right)
    in Hashtbl.of_seq (List.to_seq (aux [] [] tree));;

type metadata = {
    weights : (char * int) list;
    tree : char tree;
    code : (char, bit list) Hashtbl.t;
    remainder : char;
};;

let dump_metadata metadata =
    let wlen = List.length metadata.weights in
    let weights = List.fold_right
        (fun (character, weight) acc -> acc @ [character; Char.chr weight])
        metadata.weights
        [] in
    metadata.remainder :: Char.chr wlen :: weights

let load_metadata (data : char array) =
    let remainder = data.(0) in
    let wlen = int_of_char data.(1) in
    let weights = ref [] in
    for i = 0 to wlen - 1 do
        let character = data.(2 + i * 2) in
        let weight = int_of_char data.(2 + i * 2 + 1) in
        weights := !weights @ [(character, weight)]
    done;

    let queue = List.fold_right
        (fun (character, weight) acc ->
            Priority_queue.insert acc weight (Leaf character))
        !weights
        Priority_queue.Empty in
    let tree = from_queue queue in
    let code = get_code tree in
    { remainder = remainder; weights = !weights; tree = tree; code = code }

let skip_metadata data =
    let size = Array.length data in
    let wlen = int_of_char data.(1) in
    Array.sub data (2 + wlen * 2) (size - wlen * 2 - 2);;

let compute_metadata data =
    let weights = get_weights data in
    let queue = List.fold_right
        (fun (character, weight) acc ->
            Priority_queue.insert acc weight (Leaf character))
        weights
        Priority_queue.Empty in
    let tree = from_queue queue in
    let code = get_code tree in
    { remainder = char_of_int 0;
      weights = weights; tree = tree;
      code = code };;

class bitstream_out (filename : string) =
    object (self)
        val out_chan = Out_channel.open_bin filename
        val mutable buffer : bit list = []

        method private dump_byte =
            let byte = ref 0 in
                for i = 0 to min 7 (List.length buffer - 1) do
                    let bit = int_from_bit (List.hd buffer) in
                    byte := Int.logor !byte (Int.shift_left bit i);
                    buffer <- List.tl buffer
                done;
            Out_channel.output_char out_chan (char_of_int !byte)

        method write_metadata metadata =
            let dumped = dump_metadata metadata in
            List.iter (Out_channel.output_char out_chan) dumped
        method write_bit bit =
            buffer <- buffer @ [bit];
            if List.length buffer == 8 then self#dump_byte
        method write_bits = function
            | [] -> ()
            | bit :: bits ->
                self#write_bit bit;
                self#write_bits bits
        method close =
            let remainder = (8 - List.length buffer) mod 8 in begin
                if remainder != 0 then self#dump_byte;
                Out_channel.seek out_chan 0L;
                Out_channel.output_char out_chan (char_of_int remainder);
                Out_channel.flush out_chan;
                Out_channel.close out_chan
            end
    end;;

class bitstream_in (filename : string) =
    object (self)
        val in_chan = In_channel.open_bin filename
        val mutable buffer : int list = []

        method private read_byte =
            let bits = match In_channel.input_byte in_chan with
            | None -> []
            | Some byte ->
                let rec aux acc = function
                    | 8 -> List.rev acc
                    | n -> let shifted = Int.shift_right byte n in
                           let bit = Int.logand 1 shifted in
                           aux (bit :: acc) (n + 1)
                in aux [] 0
            in buffer <- buffer @ bits

        method read_bit =
            if List.length buffer == 0 then self#read_byte;
            match buffer with
            | [] -> None
            | hd :: tl -> buffer <- tl; Some (bit_from_int hd)
        method close = In_channel.close in_chan
    end;;

class bit_decompressor (archive : char array) =
    object (self)
        val metadata = load_metadata archive
        method metadata = metadata
        val data = skip_metadata archive
        val mutable buffer : int list = []
        val mutable ptr = 0

        method private get_opt arr idx =
            try Some arr.(idx)
            with Invalid_argument _ -> None

        method private read_byte =
            let bits = match self#get_opt data ptr with
            | None -> []
            | Some byte ->
                let num = int_of_char byte in
                let rec aux acc = function
                    | 8 -> List.rev acc
                    | n -> let shifted = Int.shift_right num n in
                           let bit = Int.logand 1 shifted in
                           aux (bit :: acc) (n + 1)
                in aux [] 0
            in begin
                let bits = if ptr == Array.length data - 1 then
                    List.of_seq (
                        Seq.drop (int_of_char metadata.remainder) 
                                 (List.to_seq (List.rev bits)))
                else bits
                in begin
                    buffer <- buffer @ bits;
                    ptr <- ptr + 1
                end
            end

        method read_bit =
            if List.length buffer == 0 then self#read_byte;
            match buffer with
            | [] -> None
            | hd :: tl -> buffer <- tl; Some (bit_from_int hd)
    end;;

let decompress_old stream tree =
    let rec aux acc = function
    | Leaf value -> aux (value :: acc) tree
    | Node (left, right) ->
        match stream#read_bit with
        | None -> List.rev acc
        | Some Zero -> aux acc left
        | Some One -> aux acc right
    in aux [] tree;;

let compress filename bin_data =
    let metadata = compute_metadata bin_data in
    let stream = new bitstream_out filename in
    stream#write_metadata metadata;
    Array.iter (fun c -> stream#write_bits (Hashtbl.find metadata.code c)) bin_data;
    stream#close;;

let decompress filename =
    let data = read_file filename in
    let decompressor = new bit_decompressor data in
    let tree = decompressor#metadata.tree in
    let rec aux acc = function
    | Leaf value -> aux (value :: acc) tree
    | Node (left, right) ->
        match decompressor#read_bit with
        | None -> List.rev acc
        | Some Zero -> aux acc left
        | Some One -> aux acc right
    in aux [] tree;;
