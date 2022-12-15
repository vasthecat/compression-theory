import argparse


def to_bin(x):
    res = []
    for _ in range(8):
        res.append(x % 2)
        x //= 2
    return res


def from_bits(bits):
    res = 0
    for i, bit in enumerate(bits):
        res |= bit * (1 << i)
    return res


class Leaf:
    def __init__(self, value, weight=0):
        self.weight = weight
        self.value = value
        self.parent = None

    def __repr__(self):
        return f"Leaf({self.value}, w={self.weight})"
        return f"Leaf({self.value})"
    __str__ = __repr__


class Node:
    def __init__(self, left, right, weight=0):
        self.weight = weight
        self.left = left
        self.right = right

    def make_swap(self):
        self.left, self.right = self.right, self.left

    def replace(self, n1, n2):
        if self.left == n1:
            self.left = n2
        elif self.right == n1:
            self.right = n2

    def __repr__(self):
        return f"Node(w={self.weight}, {self.left}, {self.right})"
        return f"Node({self.left}, {self.right})"
    __str__ = __repr__


def find_leader(tree, fweight):
    if tree.weight == fweight:
        return tree

    if isinstance(tree, Node):
        if tree.right.weight == fweight:
            return tree.right
        if tree.left.weight == fweight:
            return tree.left

        leader = find_leader(tree.right, fweight)
        if leader is not None:
            return leader
        return find_leader(tree.left, fweight)
    elif isinstance(tree, Leaf):
        if tree.weight == fweight:
            return tree
        else:
            return None


def swap_nodes(node1, node2):
    if node1.parent == node2.parent:
        node1.parent.make_swap()
    else:
        node1.parent.replace(node1, node2)
        node2.parent.replace(node2, node1)
        node1.parent, node2.parent = node2.parent, node1.parent


def dfs(tree, code=None, fvalue=None, fweight=None):
    if code is None:
        code = []

    if isinstance(tree, Node):
        lf = l, lp = dfs(tree.left, code + [0], fvalue, fweight)
        rf = r, rp = dfs(tree.right, code + [1], fvalue, fweight)
        if lp:
            return lf
        elif rp:
            return rf
    elif isinstance(tree, Leaf):
        res = True
        if fvalue is not None:
            res = res and fvalue == tree.value
        if fweight is not None:
            res = res and fweight == tree.weight
        return (tree, code), res
    return (tree, []), False


def get_root(tree):
    cur = tree
    while cur.parent is not None:
        cur = cur.parent
    return cur


def increase_weights(node):
    parent = node.parent
    if parent is not None:
        sibling = parent.right
        leader = find_leader(get_root(node), node.weight)
        if leader != parent and leader != node:
            swap_nodes(leader, node)
            parent = node.parent
        node.weight += 1
        increase_weights(parent)
    else:
        node.weight += 1


def insert_char(tree, char):
    (nyt, _), _ = dfs(tree, fweight=0)
    parent = Node(None, None)
    new = Leaf(char, 0)

    if nyt.parent is not None:
        nyt.parent.replace(nyt, parent)
    parent.parent = nyt.parent
    nyt.parent = parent
    new.parent = parent
    parent.left = nyt
    parent.right = new

    increase_weights(new)
    return get_root(parent)


def find_char(tree, char):
    (node, code), res = dfs(tree, fvalue=char)
    return res, node, code


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
            r = len(self._run)
            return (self._buffer.copy() + [byte], r)
        else:
            return (self._buffer.copy(), 8)


class BitReader:
    def __init__(self, data, r):
        self._buffer = []
        self._data = data
        self._ptr = 0
        self._r = r

    def _next_byte(self):
        run = []
        if self._ptr < len(self._data):
            byte = self._data[self._ptr]
            while byte != 0:
                run.append(byte % 2)
                byte //= 2
            while len(run) != 8:
                run.append(0)
            if self._ptr == len(self._data) - 1:
                run = run[:self._r]
            self._buffer.extend(run[::-1])
            self._ptr += 1
            return len(run) != 0
        return False

    def read_bit(self):
        if len(self._buffer) == 0:
            if not self._next_byte():
                return None
        return self._buffer.pop()


def compress(data):
    root = Leaf(0, 0)

    writer = BitWriter()
    writer.write_bit(0)
    for char in data:
        inside, node, code = find_char(root, char)
        if not inside:
            (_, nyt_code), _ = dfs(root, fweight=0)
            writer.write_bits(nyt_code)
            writer.write_bits(to_bin(char))
            root = insert_char(root, char)
        else:
            writer.write_bits(code)
            increase_weights(node)
            root = get_root(node)
    compressed, r = writer.get_buffer()
    return [r] + compressed


def decompress(archive):
    r = archive[0]
    data = archive[1:]
    root = Leaf(0, 0)

    reader = BitReader(data, r)
    cur_node = root
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
            # Был прочитан NYT
            if cur_node.weight == 0:
                bits = []
                for _ in range(8):
                    bits.append(reader.read_bit())
                char = from_bits(bits)
                root = insert_char(root, char)
            else:
                char = cur_node.value
                increase_weights(cur_node)
                root = get_root(cur_node)
            result.append(char)
            cur_node = root
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

