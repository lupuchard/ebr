#include <stdio.h>

extern int thang_main();

int main(int argc, char **argv) {
	printf("Out: %i \n", thang_main());
	return 0;
}

/*
clang -c shim.c
ar rcs shim.a shim.o
*/

/*
llc thang.ll
clang -o thang thang.s shim.a
*/
