#!/bin/bash

curl -s http://localhost:8080/openapi.json | jq '.' > openapi_pretty.json
head -200 openapi_pretty.json
