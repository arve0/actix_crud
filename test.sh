# output: body + status code, read cookies from file
base=localhost:8080

# clear user database
rm -f cookies
sqlite3 database.sqlite "delete from users"
sqlite3 database.sqlite "delete from user_sessions"
# register user
curl -w ' %{http_code}\n' -b cookies -c cookies -d 'username=adsf&password=1234' $base/user/register
# logout
curl -w ' %{http_code}\n' -b cookies -c cookies $base/user/logout
# login
curl -w ' %{http_code}\n' -b cookies -c cookies -d 'username=adsf&password=1234' $base/user/login
# insert data
curl -w ' %{http_code}\n' -b cookies -c cookies -X POST -d @src/test_data.json -H 'content-type: application/json' $base
# get inserted data
curl -w ' %{http_code}\n' -b cookies -c cookies $base/12345
# update inserted data
curl -w ' %{http_code}\n' -b cookies -c cookies -X PUT -d @src/test_data_update.json -H 'content-type: application/json' $base
# get updated data
curl -w ' %{http_code}\n' -b cookies -c cookies $base/12345
# delete data
curl -w ' %{http_code}\n' -b cookies -c cookies -X DELETE $base/12345
# logout
curl -w ' %{http_code}\n' -b cookies -c cookies $base/user/logout
