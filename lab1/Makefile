.PHONY: all clean
all: gsch1

gsch1: compress.ml huffman.ml priority_queue.ml
	ocamlopt -o gsch1 priority_queue.ml huffman.ml compress.ml

clean:
	rm -f *.cmi *.cmx *.o *.gsch* gsch*
