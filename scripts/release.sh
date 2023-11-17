#!/usr/bin/env bash

for CORE_NAME in "$@"; do
  FILE_NAME="$(find . -iname "${CORE_NAME}_*.rbf" | sort -r | head -n 1)"
  VERSION="$(echo "$FILE_NAME" | sed -E 's/.*_([0-9]{8})\.rbf/\1/')"
  RELEASE_DATE="$(echo "$VERSION" | sed -E 's/([0-9]{4})([0-9]{2})([0-9]{2})/\1-\2-\3/')"
  echo "Releasing $CORE_NAME (file $FILE_NAME) (version $VERSION date $RELEASE_DATE)"

#  retronomicon cores list

  retronomicon cores releases \
      --core "mister-$CORE_NAME" \
      create --platform mister-fpga \
             --version "$VERSION" \
             --date-released "${RELEASE_DATE}" \
             --files "$FILE_NAME" \
             --notes 'Automated release from Distribution_MiSTer'
done
