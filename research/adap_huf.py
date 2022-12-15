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


msg = "aabcdad"
root = Leaf(0, 0)

for char in msg:
    inside, node, _ = find_char(root, char)
    if not inside:
        root = insert_char(root, char)
    else:
        increase_weights(node)
        root = get_root(node)
    print(root)

