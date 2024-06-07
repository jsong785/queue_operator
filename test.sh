#!/bin/sh
curl -X POST localhost:31995 -d '{ "item": "hi" }' -H "Content-Type: application/json"
curl -X POST localhost:31995 -d '{ "item": "hi1" }' -H "Content-Type: application/json"
curl -X POST localhost:31995 -d '{ "item": "hi2" }' -H "Content-Type: application/json"
#curl -X GET localhost:31995
#curl -X GET localhost:31995
#curl -X GET localhost:31995
