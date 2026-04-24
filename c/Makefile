CC      = gcc
CFLAGS  = -Wall -Wextra -std=c11
CFLAGS2 = -Wall -Wextra -std=gnu11   # part2 needs POSIX fork/waitpid
CFLAGS3 = -Wall -Wextra -std=c11 -O2 # part3 optimised for latency benchmark

.PHONY: all run run-asan run2 run2-asan run3 run-bonus clean

all: main part2 part3 bonus

# Part 1: array horror
run: main
	./main

run-asan: main-asan
	./main-asan

main: main.c
	$(CC) $(CFLAGS) -o main main.c

main-asan: main.c
	$(CC) $(CFLAGS) -fsanitize=address,undefined -o main-asan main.c

# Part 2: pointer lifetime horror
run2: part2
	./part2

run2-asan: part2-asan
	./part2-asan

part2: part2.c
	$(CC) $(CFLAGS2) -o part2 part2.c

part2-asan: part2.c
	$(CC) $(CFLAGS2) -fsanitize=address,undefined -o part2-asan part2.c

# Part 3: performance & verbosity
run3: part3
	./part3

part3: part3.c
	$(CC) $(CFLAGS3) -o part3 part3.c

clean:
	rm -f main main-asan part2 part2-asan part3 bonus

# Bonus: cryptic function pointer riddle
run-bonus: bonus
	./bonus

bonus: bonus.c
	$(CC) $(CFLAGS) -o bonus bonus.c -lm
