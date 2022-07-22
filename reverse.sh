o=""
for (( pos=64; pos>=0 ; pos-=2 )); do
  o+=${1:pos:2}
done
echo $o

