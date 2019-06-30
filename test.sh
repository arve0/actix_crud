curl -X PUT -d @src/test_data.json -H 'content-type: application/json' localhost:8080/
echo ""
curl localhost:8080/12345
echo ""
curl -X PUT -d @src/test_data_update.json -H 'content-type: application/json' localhost:8080/
echo ""
curl localhost:8080/12345
echo ""
curl -X DELETE localhost:8080/12345
