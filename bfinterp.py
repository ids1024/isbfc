# NOTE: This may not properly/at all. The compiler
# is the focus.

import sys
import collections

import getch

from parser import parse, optimize
from parser import OUTPUT, INPUT, LOOP, ENDLOOP, MOVE
from parser import ADD, SET, MULCOPY, SCAN, LOADOUT

BUFSIZE = 8192

def interp(code):
    tokens = parse(code)
    tokens = optimize(tokens)
    i = 0
    loops = []
    mem = bytearray(BUFSIZE)
    cur = int(BUFSIZE/2)
    outbuff = ''
    while i < len(tokens)-1:
        #print("%d:%s cur:%d mem[cur]:%d" % (i, code[i], cur, mem[cur]))
        #print(loops)
        token, value = tokens[i]

        if token == OUTPUT:
            print(outbuff, end='')
            outbuff = ''
        elif token == LOADOUT:
            offset, add = value
            outbuff += chr((mem[cur+offset] + add)%256)
        elif token == INPUT:
            mem[cur] == ord(getch.getch())
        elif token == MOVE:
            cur += value
        elif token == ADD:
            offset, add = value
            newval = mem[cur+offset] + add
            newval %= 256
            mem[cur+offset] = newval
        elif token == SET:
            offset, val = value
            mem[cur+offset] = val
        elif token == MULCOPY:
            src, dest, mul = value
            newval = mem[cur+dest] + mem[cur+src] * mul
            newval %= 256
            mem[cur+dest] = newval
        elif token == SCAN:
            while mem[cur] != 0:
                cur += value
        elif token == LOOP:
            if mem[cur]:
                loops.append(i)
            else:
                skiploop = 1
                while i < len(tokens)-1 and skiploop:
                    i += 1
                    token = tokens[i][0]
                    if token == ENDLOOP:
                        skiploop -= 1
                    elif token == LOOP:
                        skiploop += 1

        elif token == ENDLOOP:
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
