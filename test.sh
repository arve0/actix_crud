curl -X POST -d @src/test_data.json -H 'content-type: application/json' localhost:8080/
echo ""
curl localhost:8080/12345
echo ""
curl -X PUT -d @src/test_data_update.json -H 'content-type: application/json' localhost:8080/
echo ""
curl localhost:8080/12345
echo ""
curl -X DELETE localhost:8080/12345
echo ""
sqlite3 database.sqlite "delete from users"
sqlite3 database.sqlite "delete from user_sessions"
curl -d 'username=adsf&password=1234' localhost:8080/user/register -c cookies
echo ""
curl localhost:8080/user/logout -b cookies
echo ""
curl -d 'username=adsf&password=1234' localhost:8080/user/login -c cookies
echo ""
curl localhost:8080/user/logout -b cookies
echo ""
