CFLAGS=-std=c11 -g -static
.PHONY: test
TESTDIR = test
test: 
	sh ./test/run.sh
