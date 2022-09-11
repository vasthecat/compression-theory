let read_file filename =
    let chan = In_channel.open_bin filename in
    let str = In_channel.input_all chan in
    In_channel.close chan;
    (Array.of_seq (String.to_seq str));

module PriorityQueue = struct
    type 'a queue = Empty | Node of int * 'a * 'a queue * 'a queue
    let is_single = function
        | Node (_, _, Empty, Empty) -> true
        | _ -> false
    let rec insert queue priority value =
        match queue with
        | Empty -> Node (priority, value, Empty, Empty)
        | Node (priority', value', left, right) ->
            if priority <= priority'
            then Node (priority, value, insert right priority' value', left)
            else Node (priority', value', insert right priority value, left)
    let rec remove_top = function
        | Empty -> None
        | Node (priority, value, left, Empty) -> Some left
        | Node (priority, value, Empty, right) -> Some right
        | Node (priority, value,
                (Node (lprio, lelt, _, _) as left),
                (Node (rprio, relt, _, _) as right)) ->
            if lprio <= rprio
            then let branch = match remove_top left with
                   | None -> Empty
                   | Some v -> v
                 in Some (Node (lprio, lelt, branch, right))
            else let branch = match remove_top right with
                   | None -> Empty
                   | Some v -> v
                 in Some (Node (rprio, relt, left, branch))
    let extract = function
        | Empty -> None
        | Node (priority, value, _, _) as queue ->
            match remove_top queue with
            | None -> Some (priority, value, Empty)
            | Some node -> Some (priority, value, node)
end;;

type bit = Zero | One
let bit_from_int = function
    | 0 -> Zero
    | _ -> One
let int_from_bit = function
    | Zero -> 0
    | One -> 1

module Huffman = struct
    type 'a tree = Leaf of 'a | Node of 'a tree * 'a tree

    let get_weights xs =
        let mp = Hashtbl.create 128 in
        for i = 0 to Array.length xs - 1 do
            match Hashtbl.find_opt mp xs.(i) with
            | None -> Hashtbl.add mp xs.(i) 1
            | Some v -> Hashtbl.replace mp xs.(i) (v + 1)
        done;
        Hashtbl.fold (fun k v acc -> (k, v) :: acc) mp [];;

    (* TODO: Избавиться от Option.get *)
    let rec from_queue queue =
        if PriorityQueue.is_single queue
        then
            let (_, value, _) = Option.get (PriorityQueue.extract queue)
            in value
        else
            let (p1, v1, queue) = Option.get (PriorityQueue.extract queue) in
            let (p2, v2, queue) = Option.get (PriorityQueue.extract queue) in
            from_queue (PriorityQueue.insert queue (p1 + p2) (Node (v1, v2)));;

    let get_code tree =
        let rec aux acc run = function
            | Leaf value -> (value, List.rev run) :: acc
            | Node (left, right) ->
                (aux acc (Zero :: run) left) @ (aux acc (One :: run) right)
        in Hashtbl.of_seq (List.to_seq (aux [] [] tree))
end;;

let data = read_file "../texts/mytest.txt"
let weights = Huffman.get_weights data
let prq = List.fold_right
    (fun (character, weight) acc ->
        PriorityQueue.insert acc weight (Huffman.Leaf character))
    weights
    PriorityQueue.Empty;;
let tree = Huffman.from_queue prq;;
let code = Huffman.get_code tree;;

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

        method write_bit bit =
            buffer <- buffer @ [bit];
            if List.length buffer == 8 then self#dump_byte
        method write_bits = function
            | [] -> ()
            | bit :: bits ->
                self#write_bit bit;
                self#write_bits bits
        method close =
            if List.length buffer != 0 then self#dump_byte;
            Out_channel.flush out_chan;
            Out_channel.close out_chan
    end;;

let stream = new bitstream_out "hello.gsch1";;
Array.map (fun c -> stream#write_bits (Hashtbl.find code c)) data;;
stream#close;;

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

let stream = new bitstream_in "hello.gsch1";;

let rec decompress acc = function
    | Huffman.Leaf value -> decompress (value :: acc) tree
    | Huffman.Node (left, right) ->
        match stream#read_bit with
        | None -> List.rev acc
        | Some Zero -> decompress acc left
        | Some One -> decompress acc right

let read_data = decompress [] tree;;
List.iter (Printf.printf "%c") read_data;;
stream#close;;
