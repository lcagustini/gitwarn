#!/bin/bash

BRANCH="main"
AFTER=`git rev-list -n 1 $BRANCH`
MESSAGE=`git log --format=%B -n 1 $AFTER`
AUTHOR=`git show -s --format='%an' $AFTER`
FULLMESSAGE=$AUTHOR" pushed to "$BRANCH": "$MESSAGE

sh echo.sh "$FULLMESSAGE"
#cd /home/lucas/gitwarn && `./target/release/gitwarn $FULLMESSAGE` && cd -
