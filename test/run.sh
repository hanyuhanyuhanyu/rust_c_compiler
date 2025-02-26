#!/bin/sh
cnt=0
assert() {
  cnt=$((cnt+1))
  input=$1
  expect=$2
  file_name="output_$cnt"
  cargo run -q $input > .tmp/$file_name.s
  cc -z noexecstack -o .tmp/$file_name .tmp/$file_name.s
  .tmp/$file_name
  actual="$?"
  if [ "$actual" = "$expect" ]; then
    echo "$input => $actual"
  else
    echo "$input => $expect expected, but got $actual"
    exit 1
  fi
}
SCRIPTDIR="$( cd -- "$(dirname "$0")" >/dev/null 2>&1 ; pwd -P )"
cd $SCRIPTDIR
rm .tmp/* -f
while IFS= read -r i; do 
  assert $i
done < test
rm .tmp/* -f
echo OK