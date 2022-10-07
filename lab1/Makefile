.PHONY: all clean
all: gsch1

gsch1: compress.ml huffman.mli huffman.ml priority_queue.ml
	ocamlc -c huffman.mli
	ocamlopt -o gsch1 priority_queue.ml huffman.ml compress.ml

clean:
	rm -f *.cmi *.cmx *.o

fullclean: clean
	rm -f *.gsch* gsch* *.txt
