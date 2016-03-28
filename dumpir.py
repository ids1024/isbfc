import sys

from parser import parse, optimize
from parser import OUTPUT, INPUT, LOOP, ENDLOOP, MOVE
from parser import ADD, SET, MULCOPY, SCAN, LOADOUT


def dumpir(code):
    tokens = parse(code)    
    tokens = optimize(tokens)
    for token, value in tokens:
        if token == INPUT:
            print('input')
        elif token == LOOP:
            print('loop')
        elif token == ENDLOOP:
            print('endloop')
        elif token == MOVE:
            print('move(offset=%d)' % value)
        elif token == ADD:
            print('add(offset=%d, value=%d)' % value)
        elif token == SET:
            print('set(offset=%d, value=%d)' % value)
        elif token == MULCOPY:
            print('mulcopy(src=%d, dest=%d, mul=%d)' % value)
        elif token == SCAN:
            print('scan(offset=%d)' % value)
        elif token == LOADOUT:
            print('loadout(add=%d)' % value)
        elif token == OUTPUT:
            print('output')
        else:
            print('TOKEN', i, 'NOT HANDLED')

if __name__ == '__main__':
    with open(sys.argv[1]) as bffile:
        dumpir(bffile.read())
