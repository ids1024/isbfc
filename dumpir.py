import sys

from parser import parse, optimize
from parser import OUTPUT, INPUT, LOOPSTART, LOOPEND, MOVE
from parser import ADD, SET, MULCOPY, SCAN


def dumpir(code):
    tokens = parse(code)    
    tokens = optimize(tokens)
    for token, value in tokens:
        if token == OUTPUT:
            print('output')
        elif token == INPUT:
            print('input')
        elif token == LOOPSTART:
            print('loopstart')
        elif token == LOOPEND:
            print('loopend')
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
        else:
            print('TOKEN', i, 'NOT HANDLED')

if __name__ == '__main__':
    with open(sys.argv[1]) as bffile:
        dumpir(bffile.read())
