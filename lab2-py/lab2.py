import argparse


class Weighted:
    def __init__(self, val, w):
        self.val = val
        self.w = w

    def __repr__(self):
        return f"Weighted({self.val}={self.w})"
    __str__ = __repr__


class Leaf:
    def __init__(self, value):
        self.value = value

    def __repr__(self):
        return f"Leaf({self.value})"
    __str__ = __repr__


class Node:
    def __init__(self, left, right):
        self.left = left
        self.right = right

    def __repr__(self):
        return f"Node({self.left}, {self.right})"
    __str__ = __repr__


class BitWriter:
    def __init__(self):
        self._buffer = []
        self._run = []

    def _get_byte(self):
        byte = 0
        for power, bit in enumerate(self._run):
            byte |= bit << power
        return byte

    def write_bit(self, bit):
        if len(self._run) == 8:
            byte = self._get_byte()
            self._buffer.append(byte)
            self._run.clear()
        self._run.append(bit)

    def write_bits(self, bits):
        for bit in bits:
            self.write_bit(bit)

    def get_buffer(self):
        if len(self._run) > 0:
            byte = self._get_byte()
            r = len(self._run) % 8
            return (self._buffer.copy() + [byte], r)
        else:
            return (self._buffer.copy(), 8)


class BitReader:
    def __init__(self, data, r):
        self._buffer = []
        self._data = data[::-1]
        self._ptr = 0
        self.total = len(self._data)
        self._r = r

    def _next_byte(self):
        print(f"{self._ptr}/{self.total}")
        run = []
        if self._ptr < len(self._data):
            byte = self._data[self._ptr]
            self._ptr += 1
            while byte != 0:
                run.append(byte % 2)
                byte //= 2
            while len(run) != (8 if len(self._data) != 0 else self._r):
                run.append(0)
            self._buffer.extend(run[::-1])
            return True
        return False

    def read_bit(self):
        if len(self._buffer) == 0:
            if not self._next_byte():
                return None
        return self._buffer.pop()


def partition(lst):
    def inner(xs, l, r, prev):
        m = (l + r) // 2
        half = (r - l) // 2
        val1 = xs[m]
        val2 = xs[-1] - xs[m]
        diff = abs(val1 - val2)
        if prev is not None and prev[1] < diff:
            return prev[0]

        if diff < 1e-5 or half == 0:
            return m
        if val1 > val2:
            return inner(xs, l, r - half, (m, diff))
        else:
            return inner(xs, l + half, r, (m, diff))
    return inner(lst, 0, len(lst) - 1, None)


def prefix_sum(p):
    pf = []
    for i in range(len(p)):
        if i == 0:
            pf.append(p[i].w)
        else:
            pf.append(p[i].w + pf[i - 1])
    return pf


def make_tree(lst):
    pf = prefix_sum(lst)
    m = partition(pf)
    l, r = lst[:m + 1], lst[m + 1:]

    if len(l) == 1:
        l_tree = Leaf(l[0].val)
    else:
        l_tree = make_tree(l)

    if len(r) == 1:
        r_tree = Leaf(r[0].val)
    else:
        r_tree = make_tree(r)

    return Node(l_tree, r_tree)


def get_code(t, codes=None, run=None):
    if codes is None:
        codes = {}
    if run is None:
        run = []

    if isinstance(t, Node):
        get_code(t.left, codes, run + [0])
        get_code(t.right, codes, run + [1])
    else:
        codes[t.value] = run
    return codes


def get_probabilities(data):
    counts = {}
    for i in data:
        counts[i] = counts.get(i, 0) + 1
    counts = list(sorted(counts.items(), key=lambda p: p[1]))
    return [Weighted(c, i) for i, (c, _) in enumerate(counts)]


def read_metadata(data):
    r = data[0]
    dict_len = data[1]
    d = []
    for i in range(dict_len):
        val = data[2 + i * 2]
        w = data[2 + i * 2 + 1]
        d.append(Weighted(val, w))
    return r, d


def compress(data):
    p = get_probabilities(data)
    t = make_tree(p)
    code = get_code(t)
    print(code)

    writer = BitWriter()
    for byte in data:
        writer.write_bits(code[byte])
    compressed, r = writer.get_buffer()

    metadata = [r, len(p)]
    for prob in p:
        metadata.append(prob.val)
        metadata.append(prob.w)

    return metadata + compressed


def decompress(archive):
    r, p = read_metadata(archive)
    tree = make_tree(p)
    data = archive[2 + len(p) * 2:]

    reader = BitReader(data, r)
    cur_node = tree
    result = []
    while True:
        bit = reader.read_bit()
        if bit is None:
            break

        if isinstance(cur_node, Node):
            if bit == 0:
                cur_node = cur_node.left
            elif bit == 1:
                cur_node = cur_node.right

        if isinstance(cur_node, Leaf):
            result.append(cur_node.value)
            cur_node = tree
    return result


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('-i', '--input', required=True)
    parser.add_argument('-o', '--output', required=True)
    parser.add_argument('--compress', default=True, action='store_true')
    parser.add_argument('--decompress', default=False, action='store_true')
    args = parser.parse_args()

    with open(args.input, "rb") as f:
        data = f.read()

    if args.decompress:
        data = list(data)
        decompressed = decompress(data)
        with open(args.output, "wb") as f:
            f.write(bytearray(decompressed))
    elif args.compress:
        archive = compress(data)
        with open(args.output, "wb") as f:
            f.write(bytearray(archive))


if __name__ == "__main__":
    main()
