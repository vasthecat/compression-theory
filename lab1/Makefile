.PHONY: all clean
all: gsch1

gsch1: compress.ml
	ocamlopt -o gsch1 compress.ml

clean:
	rm -f *.cmi *.cmx *.o *.gsch* gsch*
