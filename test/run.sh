#!/bin/sh
cnt=0
tmp=".tmp"
test_file="test"
test_folder="test_files"
if [ ! -z "$1" ]; then
  tmp="${tmp}_$1"
  test_file="${test_file}_$1"
  test_folder="${test_folder}_$1"
fi
assert() {
  cnt=$((cnt+1))
  input=$1
  expect=$2
  if [ -z "$input" ] || [ -z "$expect" ]; then
    echo "not enough argument"
    return
  fi
  file_name="output_$cnt"
  cargo run -q -- "$input" > $tmp/$file_name.s
  cc -z noexecstack -o $tmp/$file_name $tmp/$file_name.s
  $tmp/$file_name
  actual="$?"
  if [ "$actual" = "$expect" ]; then
    echo "($cnt) $input => $actual"
  else
    echo "($cnt) $input => $expect expected, but got $actual"
  fi
}
assert_file() {
  cnt=$((cnt+1))
  file=$1
  if [ -z "$file" ]; then
    echo "not enough argument"
  fi
  file_name="output_$cnt"
  input="$(head -n -1 $file)" 
  cargo run -q -- "$input"> $tmp/$file_name.s
  cc -z noexecstack -o $tmp/$file_name $tmp/$file_name.s
  $tmp/$file_name
  actual="$?"
  expect="$(tail -n 1 $file)"
  if [ "$actual" = $expect ]; then
    echo "($cnt) $input => $actual"
  else
    echo "($cnt) $input => $expect expected, but got $actual"
  fi
}
SCRIPTDIR="$( cd -- "$(dirname "$0")" >/dev/null 2>&1 ; pwd -P )"
cd $SCRIPTDIR
if [ -d $tmp ]; then
  rm $tmp/* -f
else
  mkdir $tmp
fi
while read -r line; do 
  eval "assert $line"
done < $test_file
for f in "$(ls $test_folder)"; do
  if [ -z "$f" ]; then break; fi
  assert_file $test_folder/$f
done
echo OK