import sys
import os
import subprocess

BUFSIZE = 8192

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
            tokens.append((ADD, (0, 1)))
        elif i == '-':
            tokens.append((ADD, (0, -1)))
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
            newtokens.append((ADD, (0, add)))
            add = 0
        elif move and token != MOVE:
            newtokens.append((MOVE, move))
            move = 0

        if token == ADD:
            assert value[0] == 0
            add += value[1]
        elif token == MOVE:
            move += value
        else:
            newtokens.append((token, value))
    if add:
        newtokens.append((ADD, (0, add)))
    elif move:
        newtokens.append((MOVE, move))

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
                # ADD before clear does nothing, so remove it
                if i>0 and newtokens[i-1][0] == ADD:
                    del newtokens[i-1]
                    i -= 1
                value = 0
                # ADD after SET can be simplified to SET
                if i<len(newtokens) and newtokens[i][0] == ADD:
                    assert newtokens[i][1][0] == 0
                    value = newtokens[i][1][1]
                    del newtokens[i]
                newtokens.insert(i, (SET, (0, value)))
        i += 1

    # Optimize MOVE + SET + MOVE and MOVE + ADD + MOVE
    i = 0
    while i < len(newtokens)-2:
        if (newtokens[i][0] == MOVE and
             newtokens[i+1][0] in (SET, ADD) and
             newtokens[i+2][0] == MOVE):

            opp = newtokens[i+1][0] # SET or ADD
            assert newtokens[i+1][1][0] == 0
            value = newtokens[i+1][1][1]
            offset = newtokens[i][1]
            move = offset + newtokens[i+2][1]
            del newtokens[i:i+3]
            newtokens.insert(i, (opp, (offset, value)))
            newtokens.insert(i+1, (MOVE, move))

        i += 1

    return newtokens

def compile(code):
    tokens = parse(code)
    tokens = optimize(tokens)
    output = """.section .bss
    .lcomm mem, """ + str(BUFSIZE) + """
    .set startidx, mem + """ + str(int(BUFSIZE/2)) + """
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
            offset, value = value
            if offset == 0:
                dest = "%r12"
            else:
                dest = str(offset*8)+"(%rbx)"
            if value == 1 and dest == "%r12":
                output += "    inc " + dest + "\n"
            elif value >= 1:
                output += "    add $" + str(value) + ", " + dest + "\n"
            elif value == -1 and dest == "%r12":
                output += "    dec " + dest + "\n"
            elif value <= -1:
                output += "    sub $" + str(-value) + ", " + dest + "\n"
        elif token == SET:
                offset, value = value
                if offset == 0:
                    output += "    movq $" + str(value) + ", %r12\n"
                else:
                    output += "    movq $" + str(value) + ", "+str(offset*8)+"(%rbx)\n"
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
