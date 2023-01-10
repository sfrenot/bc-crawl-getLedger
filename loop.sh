#!/bin/bash
a=0
while :
do
  set `find blocks -depth 3 | wc -l`
  echo -n `date`
  echo -n " // " 
  echo $(($1-$a))  
  a=$1
  sleep 600
done
