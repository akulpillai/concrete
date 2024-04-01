#include<stdio.h>
int foo(int a, int b);
int bar(int a);
int baz(int a);

int main(int argc, char **argv) {
   printf("Output:%d\n", bar((int)(argv[0][0])));
   printf("Output:%d\n", baz((int)(argv[0][0])));
}

int bar(int a) {
   if (a > 5) {
      // The crash is possible from this location.
      return foo(a, a-2);
   }
   return 0;
}

int baz(int a) {
   // The crash is not possible from this location.
   return foo(a, a+2);
}

// We should start this function.
int foo(int a, int b) {
   int ret = 0;
   int arr[10];
   if (a > b) {
      // This is the crash.
      arr[b] = 0;
      ret = arr[b];
   }
   return ret;
}
