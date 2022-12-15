#  window_size = 5


def match(msg, pos1, pos2, length):
    for i in range(length):
        if msg[pos1 + i] != msg[pos2 + i]:
            return False
    return True


def lz77_encode(msg, window_size):
    ptr = 0
    encoded = []
    while ptr < len(msg):
        save_l, save_offset = -1, -1
        l = 1
        while ptr + l < len(msg):
            found = False
            for offset in range(1, window_size):
                if ptr - offset < 0:
                    break
                if match(msg, ptr - offset, ptr, l):
                    found = True
                    break
            if found:
                save_l, save_offset = l, offset
                l += 1
            else:
                break
        if save_l != -1:
            l, offset = save_l, save_offset
            t = (offset, l, msg[ptr + l])
            ptr += l + 1
        else:
            t = (0, 0, msg[ptr])
            ptr += 1
        encoded.append(t)
    return encoded


def lz77_decode(encoded):
    buf = []
    ptr = 0
    for offset, length, char in encoded:
        if length == 0:
            buf.append(char)
            ptr += 1
        else:
            for i in range(length):
                buf.append(buf[ptr - offset + i])
            buf.append(char)
            ptr += length + 1
    return buf




s = list("abacabacabadaca")
enc = lz77_encode(s, 5)
print(enc)
dec = lz77_decode(enc)
print(dec)
print(dec == s)
