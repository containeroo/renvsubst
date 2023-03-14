#!/bin/bash

# File containing the test functions

output_file="testcases.md"

printf "# Testcases\n\n" > $output_file

function createTable () {
  echo "# $1" >> $output_file
  echo "| Test Name | Description | Test | Env | Result |" >> $output_file
  echo "| :--- | :--- | :--- | :--- | :--- |" >> $output_file

  while read line; do
      if [ "$line" == "#[test]" ]; then
          read line
          function_name=$(awk '{print $2}' <<< "$line"| cut -d'(' -f1)
          read -r line; description=$(sed 's/^.*: //' <<< "$line")
          read -r line; test=$(sed 's/^.*: //' <<< "$line")
          read -r line; env=$(sed 's/^.*: //' <<< "$line")
          read -r line; result=$(sed 's/^.*: //' <<< "$line")

          echo "| $function_name | $description | $test | $env | $result |" >> $output_file
      fi
  done < "$1"

  printf "\n\n" >> $output_file
}

files=$(find . -name "substitute.rs")
for file in $files; do
    createTable $file
done

echo "Table generated successfully in $output_file"
