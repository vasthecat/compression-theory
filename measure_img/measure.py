import subprocess as sb
from time import time

def run(command):
    return sb.run(command.split(), capture_output=True) \
               .stdout \
               .decode() \
               .strip()

def filesize(path):
    with open(path, "rb") as f:
        data = f.read()
    return len(data)

tests = [
"4.1.04.tiff",
"4.1.05.tiff",
"4.1.06.tiff",
"4.1.08.tiff",
"4.2.01.tiff",
"4.2.03.tiff",
"4.2.05.tiff",
"4.2.07.tiff",
"5.1.09.tiff",
"5.1.11.tiff",
"5.1.13.tiff",
"5.1.14.tiff",
"5.2.10.tiff",
"5.3.01.tiff",
"5.3.02.tiff",
"boat.512.tiff",
"gray21.512.tiff",
"house.tiff",
"ruler.512.tiff",
]

labs = [
    "./lab10", "./lab11"
]

result_compress = {k: {v: 0 for v in labs} for k in tests}
result_speed_comp = {k: {v: 0 for v in labs} for k in tests}
result_speed_decomp = {k: {v: 0 for v in labs} for k in tests}

for lab in labs:
    for test in tests:
        print(f"Testing {lab}/{test}")
        t1 = time()
        run(f"{lab} -i ../images/{test} -o test.gs --compress")
        t2 = time()

        t3 = time()
        run(f"{lab} -i test.gs -o test.out --decompress")
        t4 = time()

        size = filesize(f"../images/{test}")
        arch_size = filesize("test.gs")

        result_speed_comp[test][lab] = size / (t2 - t1) / (2 ** 10)
        result_speed_decomp[test][lab] = arch_size / (t4 - t3) / (2 ** 10)
        result_compress[test][lab] = size / arch_size

print(result_compress)
print(result_speed_comp)
print(result_speed_decomp)
print()
print("compress")
for test in tests:
    size = filesize(f"../images/{test}")
    print(f"{test}({size}) & ", end="")
    for lab in labs:
        print(f"{round(result_compress[test][lab], 5)} & ", end="")
    print()

print("speed_comp")
for test in tests:
    size = filesize(f"../images/{test}")
    print(f"{test}({size}) & ", end="")
    for lab in labs:
        print(f"{round(result_speed_comp[test][lab], 6)} & ", end="")
    print()

print("speed_decomp")
for test in tests:
    size = filesize(f"../images/{test}")
    print(f"{test}({size}) & ", end="")
    for lab in labs:
        print(f"{round(result_speed_decomp[test][lab], 6)} & ", end="")
    print()

