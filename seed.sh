#!/bin/bash

PLATFORM_MISTER=$(cargo run -p retronomicon -- platforms create MiSTerFPGA --slug mister --team root --description "MiSTer Platform for DE-10 Nano" | jq -r .slug)
SYSTEM_CHESS=$(cargo run -p retronomicon -- systems create Chess --slug chess --description "The game of chess" --manufacturer "Gary Chess" --team root | jq -r .slug)
CORE_CHESS=$(cargo run -p retronomicon -- cores create Chess-Mister --slug mister-chess --system "$SYSTEM_CHESS" --team root --description 'Chess core for the MiSTer' | jq -r .slug)
