#!/usr/bin/env bash

echo Seeding the database...
echo 'Login in. You need to have a fully authenticated (with username) administrator account.'
export RETRONOMICON_TOKEN="$(retronomicon login)"

retronomicon teams create --slug nes-maintainer "NES Maintainer"
retronomicon teams create --slug cps-maintainer "Capcom Play System Maintainer"

retronomicon systems create --slug nes NES --description "Nintendo Entertainment System" --team nes-maintainer --manufacturer "Nintendo"
retronomicon systems create --slug cps CPS --description "Capcom Play System" --team cps-maintainer --manufacturer "Capcom"

retronomicon games update-from-dat "$(dirname $0)/dats/nes.dat" nes


