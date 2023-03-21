#!/bin/bash

source .env

REPO="/home/lucas/spacelines"
BRANCHES="develop-1.6 develop-1.7 develop-2.0 develop-2.1 master"

cd $REPO

while true; do
	declare -A BEFORE
	for BRANCH in $BRANCHES; do
		BEFORE[$BRANCH]=`git rev-list -n 1 $BRANCH`
	done

	for BRANCH in $BRANCHES; do
		git reset --hard
		git clean -f -d
		git checkout $BRANCH
		git pull
	done

	for BRANCH in $BRANCHES; do
		AFTER=`git rev-list -n 1 $BRANCH`

		if [ "${BEFORE[$BRANCH]}" != "$AFTER" ]; then
			MESSAGE=`git log --format=%B -n 1 $AFTER`
			AUTHOR=`git show -s --format='%an' $AFTER`
			FULLMESSAGE=$AUTHOR" pushed to "$BRANCH": "$MESSAGE
			cd /home/lucas/gitwarn && ./target/release/gitwarn "$FULLMESSAGE" && cd -
		fi

		echo $BRANCH": "${BEFORE[$BRANCH]}" -> "$AFTER
	done
	sleep 300
done
