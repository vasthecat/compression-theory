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
    "test1.txt", "test2.txt", "test3.txt", "test4.txt", "test5.txt",
    "test6.txt", "test7.txt", "test8.txt", "test9.txt", "test10.txt"
]

labs = [
    "./lab1", "./lab2", "./lab3", "python3 lab4.py",
    "./lab5", "./lab6", "./lab7", "./lab8",
]

result_compress = {k: {v: 0 for v in labs} for k in tests}
result_speed_comp = {k: {v: 0 for v in labs} for k in tests}
result_speed_decomp = {k: {v: 0 for v in labs} for k in tests}

for lab in labs:
    for test in tests:
        print(f"Testing {lab}/{test}")
        t1 = time()
        run(f"{lab} -i ../texts/{test} -o test.gs --compress")
        t2 = time()

        t3 = time()
        run(f"{lab} -i test.gs -o test.out --decompress")
        t4 = time()

        size = filesize(f"../texts/{test}")
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
    size = filesize(f"../texts/{test}")
    print(f"{test}({size}) & ", end="")
    for lab in labs:
        print(f"{round(result_compress[test][lab], 5)} & ", end="")
    print()

print("speed_comp")
for test in tests:
    size = filesize(f"../texts/{test}")
    print(f"{test}({size}) & ", end="")
    for lab in labs:
        print(f"{round(result_speed_comp[test][lab], 6)} & ", end="")
    print()

print("speed_decomp")
for test in tests:
    size = filesize(f"../texts/{test}")
    print(f"{test}({size}) & ", end="")
    for lab in labs:
        print(f"{round(result_speed_decomp[test][lab], 6)} & ", end="")
    print()

