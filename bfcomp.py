import sys
import os
import subprocess

OUTPUT=0
INPUT=1
LOOPSTART=2
LOOPEND=3
MOVE=4
ADD=5
SET=6

def parse(code):
    code = ''.join(i for i in code if i in '.,[]+-<>')
    tokens = []
    for i in code:
        if i == '+':
            tokens.append((ADD, 1))
        elif i == '-':
            tokens.append((ADD, -1))
        elif i == '>':
            tokens.append((MOVE, 1))
        elif i == '<':
            tokens.append((MOVE, -1))
        elif i == '[':
            tokens.append((LOOPSTART, None))
        elif i == ']':
            tokens.append((LOOPEND, None))
        elif i == ',':
            tokens.append((INPUT, None))
        if i == '.':
            tokens.append((OUTPUT, None))
    return tokens

def optimize(tokens):
    add = 0
    move = 0
    newtokens = []
    for token, value in tokens:
        if add and token != ADD:
            newtokens.append((ADD, add))
            add = 0
        elif move and token != MOVE:
            newtokens.append((MOVE, move))
            move = 0

        if token == ADD:
            add += value
        elif token == MOVE:
            move += value
        else:
            newtokens.append((token, value))

    # Optimize out clear loop
    i = 0
    loop = 0
    while i < len(newtokens):
        if newtokens[i][0] == LOOPSTART:
            loop += 1
            j = i + 1
            while j < len(newtokens) and newtokens[j][0] != LOOPEND:
                if newtokens[j][0] != ADD:
                    break
                j += 1
            else:
                del newtokens[i:j+1]
                newtokens.insert(i, (SET, 0))
        i += 1

    return newtokens

def compile(code):
    tokens = parse(code)
    tokens = optimize(tokens)
    output = """.section .bss
    .lcomm mem, 8192
    .set startidx, mem + 4096
.section .text
.global _start
_start:
    movq $0, %r12
    movq $startidx, %rbx
"""
    loops = []
    loopnum = 0
    for token, value in tokens:
        if token == ADD:
            if value == 1:
                output += "    inc %r12\n"
            elif value > 1:
                output += "    add $" + str(value) + ", %r12\n"
            elif value == -1:
                output += "    dec %r12\n"
            elif value < -1:
                output += "    sub $" + str(-value) + ", %r12\n"
        elif token == SET:
                output += "    movq $" + str(value) + ", %r12\n"
        elif token == MOVE:
            if value:
                output += "    movq %r12, (%rbx)\n"
                if value > 0:
                    output += "    add $" + str(8*value) + ", %rbx\n"
                else:
                    output += "    sub $" + str(-8*value) + ", %rbx\n"
                output += "    movq (%rbx), %r12\n"
        elif token == LOOPSTART:
            loopnum += 1
            loops.append(loopnum)
            output += "    cmp $0, %r12\n" \
                      "    jz endloop" + str(loopnum) + '\n' \
                      "    loop" + str(loopnum) + ":\n"
        elif token == LOOPEND:
            loop = str(loops.pop())
            output += "    cmp $0, %r12\n" \
                      "    jnz loop" + loop + '\n' \
                      "    endloop" + loop + ':\n'
        elif token == INPUT:
            output += """
    movq $0, %rax
    movq $0, %rdi
    movq %rbx, %rsi
    movq $1, %rdx
    syscall
    movq (%rbx), %r12

"""
        elif token == OUTPUT:
            output += """
    movq %r12, (%rbx)
    movq $1, %rax
    movq $1, %rdi
    movq %rbx, %rsi
    movq $1, %rdx
    syscall

"""

    # Exit syscall
    output += """

    movq $60, %rax
    movq $0, %rdi
    syscall
"""

    return output

if __name__ == '__main__':
    print("Compiling...")
    with open(sys.argv[1]) as bffile:
        output = compile(bffile.read())
    name = os.path.splitext(sys.argv[1])[0]
    with open(name + '.s', 'w') as asmfile:
        asmfile.write(output)
    print("Assembling...")
    status = subprocess.call(['as', '-g', name+'.s', '-o', name+'.o'])
    if status == 0:
        print("Linking...")
        subprocess.call(['ld', name+'.o', '-o', name])
