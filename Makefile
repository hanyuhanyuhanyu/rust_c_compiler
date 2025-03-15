CFLAGS=-std=c11 -g -static
.PHONY: t subt
TESTDIR = test
t: 
	sh ./test/run.sh
subt: 
	sh ./test/run.sh sub
