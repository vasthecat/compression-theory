class bit_stream (outfile : string) =
    object (self)
        val out_fd = open_out_bin outfile
        val mutable buffer : int list = []
        method write_bit bit = buffer <- bit :: buffer
        method dump_byte =
            let byte = ref 0 in
                for i = 0 to 7 do
                    byte := Int.logor !byte (Int.shift_left (List.hd buffer) i);
                    buffer <- List.tl buffer
                done;
            output_char out_fd (char_of_int !byte)
    end;;

let stream = new bit_stream "hello.gsch1";;

stream#write_bit 1;;
stream#write_bit 1;;
stream#write_bit 0;;
stream#write_bit 1;;
stream#write_bit 1;;
stream#write_bit 0;;
stream#write_bit 0;;
stream#write_bit 0;;

stream#dump_byte;;
