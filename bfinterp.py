import sys
import collections

import getch

from parser import parse, optimize
from parser import OUTPUT, INPUT, LOOPSTART, LOOPEND, MOVE
from parser import ADD, SET, MULCOPY, SCAN

BUFSIZE = 8192

def interp(code):
    tokens = parse(code)
    tokens = optimize(tokens)
    i = 0
    loops = []
    mem = bytearray(BUFSIZE)
    cur = int(BUFSIZE/2)
    while i < len(tokens)-1:
        #print("%d:%s cur:%d mem[cur]:%d" % (i, code[i], cur, mem[cur]))
        #print(loops)
        token, value = tokens[i]
        if token == OUTPUT:
            print(chr(mem[cur]), end='')
        elif token == INPUT:
            mem[cur] == ord(getch.getch())
        elif token == MOVE:
            cur += value
        elif token == ADD:
            offset, add = value
            newval = mem[cur+offset] + add
            while newval < 0:
                newval += 256
            while newval > 255:
                newval -= 256
            mem[cur+offset] = newval
        elif token == SET:
            offset, val = value
            mem[cur+offset] = val
        elif token == MULCOPY:
            src, dest, mul = value
            newval = mem[cur+dest] + mem[cur+src] * mul
            while newval < 0:
                newval += 256
            while newval > 255:
                newval -= 256
            mem[dest+offset] = newval
        elif token == SCAN:
            while mem[cur] != 0:
                cur += value
        elif token == LOOPSTART:
            loops.append(i)
        elif token == LOOPEND:
            if mem[cur]:
                i = loops[-1]
            else:
                loops.pop()
        else:
            raise ValueError('Token not handled')

        i += 1

if __name__ == '__main__':
    with open(sys.argv[1]) as bffile:
        interp(bffile.read())
