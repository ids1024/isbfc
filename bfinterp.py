import sys
import collections

import getch

def interp(code):
    code = ''.join(i for i in code if i in '.,[]+-<>')
    i = 0
    loops = []
    mem = bytearray(1024)
    cur = 512
    while i < len(code)-1:
        #print("%d:%s cur:%d mem[cur]:%d" % (i, code[i], cur, mem[cur]))
        #print(loops)
        if code[i] == '.':
            print(chr(mem[cur]), end='')
        elif code[i] == ',':
            mem[cur] == ord(getch.getch())
        elif code[i] == '<':
            cur -= 1
        elif code[i] == '>':
            cur += 1
        elif code[i] == '+':
            if mem[cur] == 255:
                cur = 0
            else:
                mem[cur] += 1
        elif code[i] == '-':
            if mem[cur] == 0:
                mem[cur] = 255
            else:
                mem[cur] -= 1
        elif code[i] == '[':
            loops.append(i)
        elif code[i] == ']':
            if mem[cur]:
                i = loops[-1]
            else:
                loops.pop()

        i += 1

if __name__ == '__main__':
    with open(sys.argv[1]) as bffile:
        interp(bffile.read())
