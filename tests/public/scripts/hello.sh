#!/bin/bash

echo "Content-type: text/html"
echo
cat <<EOF
<html>
Hello $Query_name
</html>
EOF
