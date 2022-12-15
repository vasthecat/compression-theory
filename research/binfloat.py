a = 0.35

res = []
for i in range(100):
    a *= 2
    if a > 1:
        res.append(1)
        a = a - int(a)
    else:
        res.append(0)

g = []
for i, q in enumerate(res, 1):
    g.append(q * pow(2, -i))
print(sum(g))

