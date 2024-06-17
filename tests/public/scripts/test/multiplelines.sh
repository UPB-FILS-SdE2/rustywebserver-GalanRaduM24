#!/bin/sh
echo "Content-type: text/html"
echo
read text
read usr
echo "Message $text received from $usr"