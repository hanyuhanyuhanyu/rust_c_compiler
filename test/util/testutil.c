#include<stdio.h>
int cnt=0;
int _p(int v) {
  if (cnt++) printf("%c",',');
  printf("%d", v);
  return v;
}