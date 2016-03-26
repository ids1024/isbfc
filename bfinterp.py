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
    skiploop = False
    while i < len(tokens)-1:
        #print("%d:%s cur:%d mem[cur]:%d" % (i, code[i], cur, mem[cur]))
        #print(loops)
        token, value = tokens[i]

        if skiploop:
            if token == LOOPEND:
                skiploop = False
            continue

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
            mem[cur+dest] = newval
        elif token == SCAN:
            while mem[cur] != 0:
                cur += value
        elif token == LOOPSTART:
            if mem[cur]:
                loops.append(i)
            else:
                skiploop = True
        elif token == LOOPEND:
            if mem[cur] == 0:
                loops.pop()
            else:
                i = loops[-1]
        else:
            raise ValueError('Token not handled')

        i += 1

if __name__ == '__main__':
    with open(sys.argv[1]) as bffile:
        interp(bffile.read())
